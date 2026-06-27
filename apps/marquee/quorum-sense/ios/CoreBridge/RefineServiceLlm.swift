import Foundation

/// Cloud-fallback LLM backend (M6 compute placement). Implements the generated
/// UniFFI `LlmBackend` callback: the Rust refinement loop calls `complete`, and
/// this POSTs the prompt to the local Quorum refine-service, which holds the API
/// keys (so no key ever reaches the device). Returns `nil` on any failure — the
/// Rust refiner then falls back to its deterministic heuristics, so a draft is
/// always produced.
///
/// Blocking by design: `complete` is called synchronously from Rust, which runs
/// off the main thread behind the `QuorumCoreBridgeFFI` actor (ADR 0003), so
/// waiting on the request here never freezes the UI. iOS exempts loopback
/// (127.0.0.1) from App Transport Security, so cleartext to the local service
/// needs no Info.plist exception.
public final class RefineServiceLlm: LlmBackend, @unchecked Sendable {
    // iOS Simulator reaches the Mac's localhost directly. (A device build would
    // point this at the GC-Secrets backend instead.)
    private let endpoint: URL
    private let session: URLSession

    public init(endpointString: String = "http://127.0.0.1:8765/complete") {
        self.endpoint =
            URL(string: endpointString) ?? URL(string: "http://127.0.0.1:8765/complete")!
        let config = URLSessionConfiguration.ephemeral
        config.timeoutIntervalForRequest = 20
        config.waitsForConnectivity = false
        self.session = URLSession(configuration: config)
    }

    public func complete(prompt: String) -> String? {
        guard let body = try? JSONSerialization.data(withJSONObject: ["prompt": prompt]) else {
            return nil
        }
        var request = URLRequest(url: endpoint)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.httpBody = body

        let semaphore = DispatchSemaphore(value: 0)
        var result: String?
        let task = session.dataTask(with: request) { data, _, _ in
            defer { semaphore.signal() }
            guard
                let data,
                let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
                let text = json["text"] as? String,
                !text.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
            else { return }
            result = text
        }
        task.resume()
        semaphore.wait()
        return result
    }
}
