import Foundation
@testable import QuorumMobile

/// Hand-written spy/stub for `QuorumCoreBridge`.
///
/// Records every call (for interaction assertions) and returns caller-configured
/// stubbed results, so view/consumer logic can be tested without the Rust core.
@MainActor
final class SpyQuorumCoreBridge: QuorumCoreBridge {
    enum TestError: Error { case unstubbed, boom }

    struct DraftCall: Equatable {
        let inquiryThreadId: String
        let modality: SignalModality
        let rawCapture: String
    }

    private(set) var draftCalls: [DraftCall] = []
    private(set) var appendCalls: [FieldSignalDraft] = []
    private(set) var directorIntentCalls: [DirectorIntent] = []
    private(set) var workflowIdCallCount = 0

    var workflowIdStub = "quorum.field_signal_capture.v1"
    var directorSnapshotStub: Result<DirectorSnapshot, Error> = .success(DirectorFixture.quorumDecisionCheckpoint)
    var directorSnapshotSourceStub = "fixture"
    var directorIntentStub: Result<Void, Error> = .success(())
    var draftStub: Result<FieldSignalDraft, Error> = .failure(TestError.unstubbed)
    var appendStub: Result<QuorumAppendEvent, Error> = .failure(TestError.unstubbed)
    var persistStub: Result<PersistedQueueRecordSummary, Error> = .failure(TestError.unstubbed)
    var loadPersistedStub: Result<[PersistedQueueRecordSummary], Error> = .success([])

    private(set) var persistCalls: [FieldSignalDraft] = []
    private(set) var loadPersistedCallCount = 0

    func workflowId() async -> String {
        workflowIdCallCount += 1
        return workflowIdStub
    }

    func currentDirectorSnapshot() async throws -> DirectorSnapshot {
        try directorSnapshotStub.get()
    }

    func directorSnapshotSource() async -> String {
        directorSnapshotSourceStub
    }

    func submitDirectorIntent(_ intent: DirectorIntent) async throws {
        directorIntentCalls.append(intent)
        try directorIntentStub.get()
    }

    func draftFieldSignal(
        inquiryThreadId: String,
        modality: SignalModality,
        rawCapture: String
    ) async throws -> FieldSignalDraft {
        draftCalls.append(DraftCall(inquiryThreadId: inquiryThreadId, modality: modality, rawCapture: rawCapture))
        return try draftStub.get()
    }

    func appendConsentedSignal(_ draft: FieldSignalDraft) async throws -> QuorumAppendEvent {
        appendCalls.append(draft)
        return try appendStub.get()
    }

    func persistConsentedSignalToQueue(_ draft: FieldSignalDraft) async throws -> PersistedQueueRecordSummary {
        persistCalls.append(draft)
        return try persistStub.get()
    }

    func loadPersistedQueueRecords() async throws -> [PersistedQueueRecordSummary] {
        loadPersistedCallCount += 1
        return try loadPersistedStub.get()
    }
}

/// Deterministic SplitMix64 RNG so property tests are reproducible across runs.
struct SeededRNG: RandomNumberGenerator {
    private var state: UInt64

    init(seed: UInt64) {
        state = seed == 0 ? 0x9E37_79B9_7F4A_7C15 : seed
    }

    mutating func next() -> UInt64 {
        state = state &+ 0x9E37_79B9_7F4A_7C15
        var z = state
        z = (z ^ (z >> 30)) &* 0xBF58_476D_1CE4_E5B9
        z = (z ^ (z >> 27)) &* 0x94D0_49BB_1331_11EB
        return z ^ (z >> 31)
    }
}

/// Fixture draft used by snapshot and boundary tests.
@MainActor
func fixtureDraft() -> FieldSignalDraft {
    FieldSignalDraft(
        workflowId: "quorum.field_signal_capture.v1",
        draftId: "draft:inq_mobile_launch_risks:field-signal-v1",
        inquiryThreadId: "inq_mobile_launch_risks",
        modality: .voiceTranscript,
        rawCapture: "The sales team says rollout is fine, but support is seeing confusion in every pilot.",
        summary: "The sales team says rollout is fine, but support is seeing confusion in every pilot.",
        latentNeed: "needs earlier visibility into organizational ambiguity",
        contradiction: "participants report alignment while surfacing unresolved tension",
        confidence: Confidence(literal: 0.67),
        consentState: .pending
    )
}
