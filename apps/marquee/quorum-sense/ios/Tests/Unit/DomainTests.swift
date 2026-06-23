import Testing
@testable import QuorumMobile

/// Swift-side domain value types. The closed sets (modality/consent/event/sync)
/// are now UniFFI-generated enums, so "unknown wire value" is unrepresentable and
/// needs no Swift parse/reject coverage — the deep invariant coverage lives in
/// Rust. What remains host-only is `Confidence` validation (a float the UDL can't
/// constrain) and the UI affordances layered onto the generated enums.
@Suite("Domain value types")
struct DomainTests {
    @Test("Confidence accepts inclusive bounds and mid-range")
    func confidenceAccepts() {
        #expect(Confidence(0)?.value == 0)
        #expect(Confidence(1)?.value == 1)
        #expect(Confidence(0.67)?.value == 0.67)
    }

    @Test("Confidence rejects out-of-range and non-finite", arguments: [
        Float(-0.0001), Float(1.0001), Float.nan, Float.infinity, -Float.infinity,
    ])
    func confidenceRejects(_ value: Float) {
        #expect(Confidence(value) == nil)
    }

    @Test("modality exposes every case to the picker")
    func modalityCases() {
        #expect(SignalModality.allCases == [.text, .voiceTranscript, .imageOcrText])
        #expect(SignalModality.allCases.allSatisfy { $0.id == $0 })
    }

    @Test("human-facing labels are present")
    func labels() {
        #expect(SignalModality.voiceTranscript.label == "Voice")
        #expect(SignalModality.imageOcrText.label == "Photo OCR")
        #expect(ConsentState.pending.label == "Pending")
        #expect(AppendEventType.signalDraftConsented.label == "Signal draft consented")
        #expect(SyncState.queuedForSync.label == "Queued for sync")
    }
}
