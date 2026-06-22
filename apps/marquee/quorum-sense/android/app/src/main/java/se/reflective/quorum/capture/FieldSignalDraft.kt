package se.reflective.quorum.capture

enum class SignalModality(val wireName: String, val label: String) {
    TEXT("text", "Text"),
    VOICE_TRANSCRIPT("voice_transcript", "Voice"),
    IMAGE_OCR_TEXT("image_ocr_text", "Photo OCR"),
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
    val confidence: Float,
    val consentState: String,
)

data class QuorumAppendEvent(
    val workflowId: String,
    val eventType: String,
    val draftId: String,
    val inquiryThreadId: String,
    val syncState: String,
)

