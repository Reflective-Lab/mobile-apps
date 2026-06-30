package se.reflective.quorum.queue

import android.content.Context
import se.reflective.quorum.capture.FieldSignalDraft
import uniffi.quorum_ffi.ConsentDecision
import uniffi.quorum_ffi.FfiQuorumSignalDraft
import uniffi.quorum_ffi.quorumApplyQueueTransition
import uniffi.quorum_ffi.quorumBuildPersistedQueueRecord
import uniffi.quorum_ffi.quorumValidatePersistedQueueRecord

/** Orchestrates Rust validation + native file durability for the offline queue. */
class QuorumQueuePersistence(
    private val store: FileQueueStore,
    private val clientVersion: String?,
    private val capturePlatform: String = "android",
) {
    suspend fun persistConsentedSignal(
        draft: FieldSignalDraft,
        consentDecision: ConsentDecision = ConsentDecision.ACCEPTED,
        offline: Boolean = true,
    ): PersistedQueueRecordSummary {
        val now = QueueTimestamp.nowISO8601()
        val json = quorumBuildPersistedQueueRecord(
            draft = draft.toFfi(),
            consentDecision = consentDecision,
            consentRecordedAt = now,
            updatedAt = now,
            clientVersion = clientVersion,
            offline = offline,
            capturePlatform = capturePlatform,
        )
        val summary = persistedQueueRecordSummaryFromJSON(json)
            ?: throw FileQueueStore.StoreException("unreadable record for ${draft.draftId}")
        store.save(summary.recordId, json)
        return summary
    }

    suspend fun loadPersistedRecords(): List<PersistedQueueRecordSummary> {
        val blobs = store.loadAllJSON()
        return blobs.keys.sorted().map { recordId ->
            val json = blobs.getValue(recordId)
            quorumValidatePersistedQueueRecord(json)
            persistedQueueRecordSummaryFromJSON(json)
                ?: throw FileQueueStore.StoreException("unreadable record for $recordId")
        }
    }

    suspend fun applyTransition(
        recordId: String,
        nextState: uniffi.quorum_ffi.QueueState,
    ): PersistedQueueRecordSummary {
        val current = store.load(recordId)
            ?: throw FileQueueStore.StoreException("missing record $recordId")
        val updated = quorumApplyQueueTransition(
            recordJson = current,
            nextState = nextState,
            updatedAt = QueueTimestamp.nowISO8601(),
        )
        val summary = persistedQueueRecordSummaryFromJSON(updated)
            ?: throw FileQueueStore.StoreException("unreadable record for $recordId")
        store.save(summary.recordId, updated)
        return summary
    }

    companion object {
        fun production(context: Context, clientVersion: String?): QuorumQueuePersistence =
            QuorumQueuePersistence(
                store = FileQueueStore(context),
                clientVersion = clientVersion,
            )
    }

    private fun FieldSignalDraft.toFfi(): FfiQuorumSignalDraft =
        FfiQuorumSignalDraft(
            workflowId = workflowId,
            draftId = draftId,
            inquiryThreadId = inquiryThreadId,
            modality = modality,
            rawCapture = rawCapture,
            summary = summary,
            latentNeed = latentNeed,
            contradiction = contradiction,
            confidence = confidence.value,
            consentState = consentState,
        )
}
