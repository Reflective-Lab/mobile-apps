import Foundation

// Not @MainActor: the production bridge is an actor that runs the synchronous
// Rust FFI off the main thread (ADR 0003). Sendable so a `@MainActor` view can
// hold `any QuorumCoreBridge` and `await` across the actor boundary; the async
// methods let an actor- or main-actor-isolated witness satisfy them.
public protocol QuorumCoreBridge: Sendable {
    func workflowId() async -> String
    func currentDirectorSnapshot() async throws -> DirectorSnapshot
    func directorSnapshotSource() async -> String
    /// Block until SSE delivers a newer director snapshot, or `timeoutMs` elapses.
    func waitDirectorUpdate(sinceVersion: UInt64, timeoutMs: UInt32) async -> Bool
    func submitDirectorIntent(_ intent: DirectorIntent) async throws
    func draftFieldSignal(
        inquiryThreadId: String,
        modality: SignalModality,
        rawCapture: String
    ) async throws -> FieldSignalDraft
    func appendConsentedSignal(_ draft: FieldSignalDraft) async throws -> QuorumAppendEvent
    /// Build a Rust-validated record and persist opaque JSON (M4.6).
    func persistConsentedSignalToQueue(
        _ draft: FieldSignalDraft,
        consentDecision: ConsentDecision
    ) async throws -> PersistedQueueRecordSummary
    /// Reload durable queue records, validating each blob through Rust.
    func loadPersistedQueueRecords() async throws -> [PersistedQueueRecordSummary]
}

public struct PreviewQuorumCoreBridge: QuorumCoreBridge {
    private let queueStore = PreviewQueueStore()

    public init() {}

    public func workflowId() async -> String {
        "quorum.field_signal_capture.v1"
    }

    public func currentDirectorSnapshot() async throws -> DirectorSnapshot {
        DirectorFixture.quorumDecisionCheckpoint
    }

    public func directorSnapshotSource() async -> String {
        "fixture"
    }

    public func waitDirectorUpdate(sinceVersion: UInt64, timeoutMs: UInt32) async -> Bool {
        _ = sinceVersion
        _ = timeoutMs
        return false
    }

    public func submitDirectorIntent(_ intent: DirectorIntent) async throws {
        // Preview bridge is intentionally side-effect free; production routing
        // will hand this typed intent to helm-client once Plan 1 lands.
        _ = intent
    }

    public func draftFieldSignal(
        inquiryThreadId: String,
        modality: SignalModality,
        rawCapture: String
    ) async throws -> FieldSignalDraft {
        FieldSignalDraft(
            workflowId: await workflowId(),
            draftId: "draft:\(inquiryThreadId):field-signal-v1",
            inquiryThreadId: inquiryThreadId,
            modality: modality,
            rawCapture: rawCapture,
            summary: String(rawCapture.trimmingCharacters(in: .whitespacesAndNewlines).prefix(96)),
            latentNeed: "needs earlier visibility into organizational ambiguity",
            contradiction: "participants report alignment while surfacing unresolved tension",
            confidence: Confidence(literal: 0.67),
            consentState: .pending
        )
    }

    public func appendConsentedSignal(_ draft: FieldSignalDraft) async throws -> QuorumAppendEvent {
        QuorumAppendEvent(
            workflowId: draft.workflowId,
            eventType: .signalDraftConsented,
            draftId: draft.draftId,
            inquiryThreadId: draft.inquiryThreadId,
            syncState: .queuedForSync
        )
    }

    public func persistConsentedSignalToQueue(
        _ draft: FieldSignalDraft,
        consentDecision: ConsentDecision = .accepted
    ) async throws -> PersistedQueueRecordSummary {
        guard consentDecision.permitsQueue else {
            throw CoreBridgeError.consentDoesNotPermitQueue(consentDecision)
        }
        let summary = PersistedQueueRecordSummary(
            recordId: draft.draftId,
            queueState: "queued",
            updatedAt: QueueTimestamp.nowISO8601()
        )
        await queueStore.save(summary)
        return summary
    }

    public func loadPersistedQueueRecords() async throws -> [PersistedQueueRecordSummary] {
        await queueStore.all()
    }
}

private actor PreviewQueueStore {
    private var records: [PersistedQueueRecordSummary] = []

    func save(_ summary: PersistedQueueRecordSummary) {
        records.removeAll { $0.recordId == summary.recordId }
        records.append(summary)
    }

    func all() -> [PersistedQueueRecordSummary] {
        records.sorted { $0.recordId < $1.recordId }
    }
}
