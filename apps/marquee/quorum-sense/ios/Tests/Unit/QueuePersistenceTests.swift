import Foundation
import Testing
@testable import QuorumMobile

@Suite("Queue persistence")
struct QueuePersistenceTests {
    @Test("file store survives re-open (app relaunch simulation)")
    func fileStoreSurvivesRelaunch() async throws {
        let subdirectory = "queue-test-\(UUID().uuidString)"
        let recordId = "draft:inq_test:field-signal-v1"
        let json = """
        {"schema_version":1,"record_id":"\(recordId)","queue_state":"queued","updated_at":"2026-06-06T12:02:00Z"}
        """

        let writer = try FileQueueStore(subdirectory: subdirectory)
        try await writer.save(recordId: recordId, json: json)

        let reader = try FileQueueStore(subdirectory: subdirectory)
        let loaded = try await reader.load(recordId: recordId)
        #expect(loaded == json)
        #expect(try await reader.allRecordIds() == [recordId])
    }

    @Test("persisted record round-trips through Rust validation and file store")
    func persistedRecordRoundTrip() async throws {
        let subdirectory = "queue-test-\(UUID().uuidString)"
        let store = try FileQueueStore(subdirectory: subdirectory)
        let persistence = QuorumQueuePersistence(store: store, clientVersion: "0.1.2-test")

        let bridge = try QuorumCoreBridgeFFI(queuePersistence: persistence)
        let draft = try await bridge.draftFieldSignal(
            inquiryThreadId: "inq_persist_test",
            modality: .text,
            rawCapture: "offline queue probe"
        )

        let summary = try await bridge.persistConsentedSignalToQueue(draft)
        #expect(summary.recordId == draft.draftId)
        #expect(summary.queueState == "queued")

        let reloadedBridge = try QuorumCoreBridgeFFI(
            queuePersistence: QuorumQueuePersistence(store: store, clientVersion: "0.1.2-test")
        )
        let records = try await reloadedBridge.loadPersistedQueueRecords()
        #expect(records.count == 1)
        #expect(records[0].recordId == draft.draftId)
        #expect(records[0].queueState == "queued")
    }
}
