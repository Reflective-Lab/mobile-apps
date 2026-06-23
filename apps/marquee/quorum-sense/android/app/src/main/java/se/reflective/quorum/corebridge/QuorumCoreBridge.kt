package se.reflective.quorum.corebridge

import se.reflective.quorum.capture.AppendEventType
import se.reflective.quorum.capture.Confidence
import se.reflective.quorum.capture.ConsentState
import se.reflective.quorum.capture.FieldSignalDraft
import se.reflective.quorum.capture.QuorumAppendEvent
import se.reflective.quorum.capture.SignalModality
import se.reflective.quorum.capture.SyncState

interface QuorumCoreBridge {
    suspend fun workflowId(): String

    suspend fun draftFieldSignal(
        inquiryThreadId: String,
        modality: SignalModality,
        rawCapture: String,
    ): FieldSignalDraft

    suspend fun appendConsentedSignal(draft: FieldSignalDraft): QuorumAppendEvent
}

class PreviewQuorumCoreBridge : QuorumCoreBridge {
    override suspend fun workflowId(): String = "quorum.field_signal_capture.v1"

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
}

// Replace PreviewQuorumCoreBridge with a UniFFI generated adapter when the Rust
// bridge is generated from schemas/quorum-mobile.udl.

