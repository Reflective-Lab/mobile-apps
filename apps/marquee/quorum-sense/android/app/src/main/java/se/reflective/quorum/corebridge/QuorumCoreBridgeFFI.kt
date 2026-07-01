package se.reflective.quorum.corebridge

import android.content.Context
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import se.reflective.quorum.app.clientVersion
import se.reflective.quorum.capture.Confidence
import se.reflective.quorum.capture.FieldSignalDraft
import se.reflective.quorum.capture.QuorumAppendEvent
import se.reflective.quorum.director.DirectorIntent
import se.reflective.quorum.director.DirectorSnapshot
import se.reflective.quorum.queue.PersistedQueueRecordSummary
import se.reflective.quorum.queue.QuorumQueuePersistence
import uniffi.quorum_ffi.ConsentDecision
import uniffi.quorum_ffi.FfiQuorumAppendEvent
import uniffi.quorum_ffi.FfiQuorumSignalDraft
import uniffi.quorum_ffi.SignalModality
import uniffi.quorum_ffi.quorumAppendConsentedSignal
import uniffi.quorum_ffi.quorumCurrentDirectorSnapshot
import uniffi.quorum_ffi.quorumDirectorSnapshotSource
import uniffi.quorum_ffi.quorumDraftFieldSignal
import uniffi.quorum_ffi.quorumFieldSignalWorkflowId
import uniffi.quorum_ffi.quorumSubmitDirectorIntent
import uniffi.quorum_ffi.quorumWaitDirectorUpdate

/**
 * Raised when the Rust core returns a value this app build cannot map into a
 * domain type. With modality/consent/event/sync now enums on the wire, the only
 * remaining unmappable value is a confidence outside `0..1` (a float the UDL
 * can't constrain). Distinct from the generated `QuorumException`, which the FFI
 * functions themselves throw when the *core* rejects input.
 */
sealed class CoreBridgeException(message: String) : Exception(message) {
    class ConfidenceOutOfRange(val value: Float) :
        CoreBridgeException("core returned confidence $value outside 0..1")
}

/**
 * Production [QuorumCoreBridge] backed by the Rust core through UniFFI.
 */
class QuorumCoreBridgeFFI(
    context: Context,
    queuePersistence: QuorumQueuePersistence? = null,
) : QuorumCoreBridge {
    private val queuePersistence: QuorumQueuePersistence =
        queuePersistence
            ?: QuorumQueuePersistence.production(
                context = context.applicationContext,
                clientVersion = context.applicationContext.clientVersion(),
            )

    override suspend fun workflowId(): String =
        withContext(Dispatchers.Default) { quorumFieldSignalWorkflowId() }

    override suspend fun currentDirectorSnapshot(): DirectorSnapshot =
        withContext(Dispatchers.Default) {
            DirectorBridgeMapping.toDomain(quorumCurrentDirectorSnapshot())
        }

    override suspend fun directorSnapshotSource(): String =
        withContext(Dispatchers.Default) { quorumDirectorSnapshotSource() }

    override suspend fun waitDirectorUpdate(sinceVersion: ULong, timeoutMs: UInt): Boolean =
        withContext(Dispatchers.Default) {
            quorumWaitDirectorUpdate(sinceVersion, timeoutMs)
        }

    override suspend fun submitDirectorIntent(intent: DirectorIntent) {
        withContext(Dispatchers.Default) {
            quorumSubmitDirectorIntent(DirectorBridgeMapping.toFfi(intent))
        }
    }

    override suspend fun draftFieldSignal(
        inquiryThreadId: String,
        modality: SignalModality,
        rawCapture: String,
    ): FieldSignalDraft =
        withContext(Dispatchers.Default) {
            quorumDraftFieldSignal(inquiryThreadId, modality, rawCapture).toDomain()
        }

    override suspend fun appendConsentedSignal(draft: FieldSignalDraft): QuorumAppendEvent =
        withContext(Dispatchers.Default) {
            quorumAppendConsentedSignal(draft.toFfi()).toDomain()
        }

    override suspend fun persistConsentedSignalToQueue(
        draft: FieldSignalDraft,
        consentDecision: ConsentDecision,
    ): PersistedQueueRecordSummary =
        queuePersistence.persistConsentedSignal(draft, consentDecision)

    override suspend fun loadPersistedQueueRecords(): List<PersistedQueueRecordSummary> =
        queuePersistence.loadPersistedRecords()

    override suspend fun submitEligibleQueueRecords(): Int =
        queuePersistence.submitEligibleRecords()

    private fun FfiQuorumSignalDraft.toDomain(): FieldSignalDraft {
        val parsedConfidence = Confidence.of(confidence)
            ?: throw CoreBridgeException.ConfidenceOutOfRange(confidence)
        return FieldSignalDraft(
            workflowId = workflowId,
            draftId = draftId,
            inquiryThreadId = inquiryThreadId,
            modality = modality,
            rawCapture = rawCapture,
            summary = summary,
            latentNeed = latentNeed,
            contradiction = contradiction,
            confidence = parsedConfidence,
            consentState = consentState,
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

    private fun FfiQuorumAppendEvent.toDomain(): QuorumAppendEvent =
        QuorumAppendEvent(
            workflowId = workflowId,
            eventType = eventType,
            draftId = draftId,
            inquiryThreadId = inquiryThreadId,
            syncState = syncState,
        )
}
