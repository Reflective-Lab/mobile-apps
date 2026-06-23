package se.reflective.quorum.capture

enum class SignalModality(val wireName: String, val label: String) {
    TEXT("text", "Text"),
    VOICE_TRANSCRIPT("voice_transcript", "Voice"),
    IMAGE_OCR_TEXT("image_ocr_text", "Photo OCR"),
    ;

    companion object {
        fun fromWire(value: String): SignalModality? = entries.firstOrNull { it.wireName == value }
    }
}

/** Whether a captured signal has cleared consent for sync. Mirrors Rust `ConsentState`. */
enum class ConsentState(val wireName: String, val label: String) {
    PENDING("pending", "Pending"),
    CONSENTED("consented", "Consented"),
    ;

    companion object {
        fun fromWire(value: String): ConsentState? = entries.firstOrNull { it.wireName == value }
    }
}

/** The kind of event emitted when a draft is appended. Mirrors Rust `AppendEventType`. */
enum class AppendEventType(val wireName: String, val label: String) {
    SIGNAL_DRAFT_CONSENTED("SignalDraftConsented", "Signal draft consented"),
    ;

    companion object {
        fun fromWire(value: String): AppendEventType? =
            entries.firstOrNull { it.wireName == value }
    }
}

/** Where an appended event sits in the sync pipeline. Mirrors Rust `SyncState`. */
enum class SyncState(val wireName: String, val label: String) {
    QUEUED_FOR_SYNC("queued_for_sync", "Queued for sync"),
    ;

    companion object {
        fun fromWire(value: String): SyncState? = entries.firstOrNull { it.wireName == value }
    }
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
