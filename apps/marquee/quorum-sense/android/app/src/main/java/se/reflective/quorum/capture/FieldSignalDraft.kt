package se.reflective.quorum.capture

import uniffi.quorum_ffi.AppendEventType
import uniffi.quorum_ffi.ConsentState
import uniffi.quorum_ffi.SignalModality
import uniffi.quorum_ffi.SyncState

// SignalModality, ConsentState, AppendEventType and SyncState are no longer
// declared here: UniFFI now generates them (uniffi/quorum_ffi/quorum_ffi.kt)
// straight from the UDL contract, so the wire enum *is* the domain enum — there
// are no String wire values to parse and an unknown case is unrepresentable.
// These extensions add only the host-side affordance the bindings don't emit:
// the human-facing label. `entries` (the picker's case list) and structural
// equality come from the generated `enum class` for free.

val SignalModality.label: String
    get() = when (this) {
        SignalModality.TEXT -> "Text"
        SignalModality.VOICE_TRANSCRIPT -> "Voice"
        SignalModality.IMAGE_OCR_TEXT -> "Photo OCR"
    }

val ConsentState.label: String
    get() = when (this) {
        ConsentState.PENDING -> "Pending"
        ConsentState.CONSENTED -> "Consented"
    }

val AppendEventType.label: String
    get() = when (this) {
        AppendEventType.SIGNAL_DRAFT_CONSENTED -> "Signal draft consented"
    }

val SyncState.label: String
    get() = when (this) {
        SyncState.QUEUED_FOR_SYNC -> "Queued for sync"
    }

/** A model confidence score constrained to `0f..1f`, mirroring Rust `Confidence`. */
@JvmInline
value class Confidence private constructor(val value: Float) {
    companion object {
        /** Validated construction from untrusted input (e.g. core output). */
        fun of(value: Float): Confidence? =
            if (value.isFinite() && value in 0f..1f) Confidence(value) else null

        /** Trusted construction for compile-time-known-valid literals (previews/tests). */
        fun literal(value: Float): Confidence = Confidence(value)
    }
}

data class FieldSignalDraft(
    val workflowId: String,
    val draftId: String,
    val inquiryThreadId: String,
    val modality: SignalModality,
    val rawCapture: String,
    val summary: String,
    val latentNeed: String,
    val contradiction: String,
    val confidence: Confidence,
    val consentState: ConsentState,
)

data class QuorumAppendEvent(
    val workflowId: String,
    val eventType: AppendEventType,
    val draftId: String,
    val inquiryThreadId: String,
    val syncState: SyncState,
)
