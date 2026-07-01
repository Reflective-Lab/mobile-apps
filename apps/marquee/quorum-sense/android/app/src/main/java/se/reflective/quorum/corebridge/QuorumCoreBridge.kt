package se.reflective.quorum.corebridge

import se.reflective.quorum.capture.Confidence
import se.reflective.quorum.capture.FieldSignalDraft
import se.reflective.quorum.capture.QuorumAppendEvent
import se.reflective.quorum.director.DirectorFixture
import se.reflective.quorum.director.DirectorIntent
import se.reflective.quorum.director.DirectorSnapshot
import se.reflective.quorum.queue.PersistedQueueRecordSummary
import se.reflective.quorum.queue.QueueTimestamp
import se.reflective.quorum.queue.permitsQueue
import uniffi.quorum_ffi.AppendEventType
import uniffi.quorum_ffi.ConsentDecision
import uniffi.quorum_ffi.ConsentState
import uniffi.quorum_ffi.SignalModality
import uniffi.quorum_ffi.SyncState

interface QuorumCoreBridge {
    suspend fun workflowId(): String

    suspend fun currentDirectorSnapshot(): DirectorSnapshot

    suspend fun directorSnapshotSource(): String

    suspend fun waitDirectorUpdate(sinceVersion: ULong, timeoutMs: UInt): Boolean

    suspend fun submitDirectorIntent(intent: DirectorIntent)

    suspend fun draftFieldSignal(
        inquiryThreadId: String,
        modality: SignalModality,
        rawCapture: String,
    ): FieldSignalDraft

    suspend fun appendConsentedSignal(draft: FieldSignalDraft): QuorumAppendEvent

    suspend fun persistConsentedSignalToQueue(
        draft: FieldSignalDraft,
        consentDecision: ConsentDecision = ConsentDecision.ACCEPTED,
    ): PersistedQueueRecordSummary

    suspend fun loadPersistedQueueRecords(): List<PersistedQueueRecordSummary>

    suspend fun submitEligibleQueueRecords(): Int
}

class PreviewQuorumCoreBridge : QuorumCoreBridge {
    private val queueStore = mutableListOf<PersistedQueueRecordSummary>()

    override suspend fun workflowId(): String = "quorum.field_signal_capture.v1"

    override suspend fun currentDirectorSnapshot(): DirectorSnapshot =
        DirectorFixture.quorumDecisionCheckpoint

    override suspend fun directorSnapshotSource(): String = "fixture"

    override suspend fun waitDirectorUpdate(sinceVersion: ULong, timeoutMs: UInt): Boolean {
        return false
    }

    override suspend fun submitDirectorIntent(intent: DirectorIntent) {
        // Preview bridge is intentionally side-effect free; production routing
        // will hand this typed intent to helm-client once Plan 1 lands.
    }

    override suspend fun draftFieldSignal(
        inquiryThreadId: String,
        modality: SignalModality,
        rawCapture: String,
    ): FieldSignalDraft {
        val trimmed = rawCapture.trim()
        return FieldSignalDraft(
            workflowId = workflowId(),
            draftId = "draft:$inquiryThreadId:field-signal-v1",
            inquiryThreadId = inquiryThreadId,
            modality = modality,
            rawCapture = rawCapture,
            summary = trimmed.take(96),
            latentNeed = "needs earlier visibility into organizational ambiguity",
            contradiction = "participants report alignment while surfacing unresolved tension",
            confidence = Confidence.literal(0.67f),
            consentState = ConsentState.PENDING,
        )
    }

    override suspend fun appendConsentedSignal(draft: FieldSignalDraft): QuorumAppendEvent =
        QuorumAppendEvent(
            workflowId = draft.workflowId,
            eventType = AppendEventType.SIGNAL_DRAFT_CONSENTED,
            draftId = draft.draftId,
            inquiryThreadId = draft.inquiryThreadId,
            syncState = SyncState.QUEUED_FOR_SYNC,
        )

    override suspend fun persistConsentedSignalToQueue(
        draft: FieldSignalDraft,
        consentDecision: ConsentDecision,
    ): PersistedQueueRecordSummary {
        require(consentDecision.permitsQueue()) {
            "consent $consentDecision does not permit queue"
        }
        val summary = PersistedQueueRecordSummary(
            recordId = draft.draftId,
            queueState = "queued",
            updatedAt = QueueTimestamp.nowISO8601(),
        )
        queueStore.removeAll { it.recordId == summary.recordId }
        queueStore.add(summary)
        return summary
    }

    override suspend fun loadPersistedQueueRecords(): List<PersistedQueueRecordSummary> =
        queueStore.sortedBy { it.recordId }

    override suspend fun submitEligibleQueueRecords(): Int = 0
}
// bridge is generated from schemas/quorum-mobile.udl.
