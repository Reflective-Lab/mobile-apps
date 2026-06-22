package se.reflective.quorum.corebridge

import se.reflective.quorum.capture.FieldSignalDraft
import se.reflective.quorum.capture.QuorumAppendEvent
import se.reflective.quorum.capture.SignalModality

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
            confidence = 0.67f,
            consentState = "pending",
        )
    }

    override suspend fun appendConsentedSignal(draft: FieldSignalDraft): QuorumAppendEvent =
        QuorumAppendEvent(
            workflowId = draft.workflowId,
            eventType = "SignalDraftConsented",
            draftId = draft.draftId,
            inquiryThreadId = draft.inquiryThreadId,
            syncState = "queued_for_sync",
        )
}

// Replace PreviewQuorumCoreBridge with a UniFFI generated adapter when the Rust
// bridge is generated from schemas/quorum-mobile.udl.

