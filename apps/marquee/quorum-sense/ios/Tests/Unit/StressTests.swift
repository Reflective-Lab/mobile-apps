import Testing
@testable import QuorumMobile

/// One independent draft+append round trip on the @MainActor bridge. Declared as
/// a standalone main-actor function so each task captures only an `Int` (Sendable)
/// — avoiding capturing a non-Sendable actor-isolated value across the task
/// boundary.
@MainActor
private func draftAppendRoundTrip(index: Int) async throws -> Bool {
    let bridge = PreviewQuorumCoreBridge()
    let draft = try await bridge.draftFieldSignal(
        inquiryThreadId: "inq_\(index)",
        modality: .text,
        rawCapture: "payload under load \(index)"
    )
    let event = try await bridge.appendConsentedSignal(draft)
    // Invariant under contention: ids stay paired.
    return event.draftId == draft.draftId
}

/// Concurrency stress: saturate the @MainActor bridge with thousands of
/// overlapping draft+append tasks. A deadlock or re-entrancy hang trips the
/// `.timeLimit` trait and fails the test instead of hanging the suite.
@Suite("Stress")
struct StressTests {
    @Test("thousands of concurrent draft+append complete without deadlock/hang", .timeLimit(.minutes(1)))
    func concurrentBridgeUseDoesNotDeadlock() async throws {
        let taskCount = 2_000

        let succeeded = try await withThrowingTaskGroup(of: Bool.self) { group in
            for i in 0..<taskCount {
                group.addTask { try await draftAppendRoundTrip(index: i) }
            }
            var count = 0
            for try await ok in group where ok { count += 1 }
            return count
        }

        #expect(succeeded == taskCount)
    }
}
