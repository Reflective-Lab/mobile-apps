import Foundation

/// Orchestrates Rust validation + native file durability for the offline queue.
public actor QuorumQueuePersistence {
    private let store: FileQueueStore
    private let clientVersion: String?
    private let capturePlatform: String

    public init(
        store: FileQueueStore,
        clientVersion: String? = Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String,
        capturePlatform: String = "ios"
    ) {
        self.store = store
        self.clientVersion = clientVersion
        self.capturePlatform = capturePlatform
    }

    public static func production() throws -> QuorumQueuePersistence {
        try QuorumQueuePersistence(store: FileQueueStore())
    }

    /// Build a validated record in Rust, then persist opaque JSON by `record_id`.
    public func persistConsentedSignal(
        _ draft: FieldSignalDraft,
        consentDecision: ConsentDecision = .accepted,
        offline: Bool = true
    ) async throws -> PersistedQueueRecordSummary {
        let now = QueueTimestamp.nowISO8601()
        let json = try quorumBuildPersistedQueueRecord(
            draft: Self.ffiDraft(draft),
            consentDecision: consentDecision,
            consentRecordedAt: now,
            updatedAt: now,
            clientVersion: clientVersion,
            offline: offline,
            capturePlatform: capturePlatform
        )
        guard let summary = PersistedQueueRecordSummary.fromJSON(json) else {
            throw FileQueueStore.StoreError.unreadableRecord(draft.draftId)
        }
        try await store.save(recordId: summary.recordId, json: json)
        return summary
    }

    /// Reload every stored record, validating each blob through Rust on read.
    public func loadPersistedRecords() async throws -> [PersistedQueueRecordSummary] {
        let blobs = try await store.loadAllJSON()
        var summaries: [PersistedQueueRecordSummary] = []
        summaries.reserveCapacity(blobs.count)

        for (recordId, json) in blobs.sorted(by: { $0.key < $1.key }) {
            try quorumValidatePersistedQueueRecord(recordJson: json)
            guard let summary = PersistedQueueRecordSummary.fromJSON(json) else {
                throw FileQueueStore.StoreError.unreadableRecord(recordId)
            }
            summaries.append(summary)
        }
        return summaries
    }

    /// Apply a Rust-validated transition and rewrite the stored JSON atomically.
    public func applyTransition(
        recordId: String,
        to nextState: QueueState
    ) async throws -> PersistedQueueRecordSummary {
        guard let current = try await store.load(recordId: recordId) else {
            throw FileQueueStore.StoreError.unreadableRecord(recordId)
        }
        let updated = try quorumApplyQueueTransition(
            recordJson: current,
            nextState: nextState,
            updatedAt: QueueTimestamp.nowISO8601()
        )
        guard let summary = PersistedQueueRecordSummary.fromJSON(updated) else {
            throw FileQueueStore.StoreError.unreadableRecord(recordId)
        }
        try await store.save(recordId: summary.recordId, json: updated)
        return summary
    }

    private static func ffiDraft(_ draft: FieldSignalDraft) -> FfiQuorumSignalDraft {
        FfiQuorumSignalDraft(
            workflowId: draft.workflowId,
            draftId: draft.draftId,
            inquiryThreadId: draft.inquiryThreadId,
            modality: draft.modality,
            rawCapture: draft.rawCapture,
            summary: draft.summary,
            latentNeed: draft.latentNeed,
            contradiction: draft.contradiction,
            confidence: draft.confidence.value,
            consentState: draft.consentState
        )
    }
}

/// In-memory store for previews and unit tests without touching disk.
public actor InMemoryQueueStore {
    private var records: [String: String] = [:]

    public init() {}

    public func save(recordId: String, json: String) {
        records[recordId] = json
    }

    public func load(recordId: String) -> String? {
        records[recordId]
    }

    public func loadAllJSON() -> [String: String] {
        records
    }
}
