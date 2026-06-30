package se.reflective.quorum.platformai

import uniffi.quorum_ffi.SignalModality

data class CapturedSignalInput(
    val modality: SignalModality,
    val rawCapture: String,
)

data class NormalizedCaptureResult(
    val input: CapturedSignalInput,
    /** True when a platform AI path produced structured output; false for typed trim fallback. */
    val usedPlatformAI: Boolean,
)

class PlatformSignalExtractor {
    /** Normalize native capture into bridge input, with typed fallback when platform AI is unavailable (M5.7). */
    suspend fun normalizeCapture(modality: SignalModality, text: String): NormalizedCaptureResult {
        val trimmed = text.trim()
        val input = when (modality) {
            SignalModality.TEXT -> normalizeTextCapture(trimmed)
            SignalModality.VOICE_TRANSCRIPT -> normalizeVoiceTranscript(trimmed)
            SignalModality.IMAGE_OCR_TEXT -> normalizeImageOcrText(trimmed)
        }
        return NormalizedCaptureResult(input = input, usedPlatformAI = false)
    }

    suspend fun normalizeTextCapture(text: String): CapturedSignalInput =
        CapturedSignalInput(
            modality = SignalModality.TEXT,
            rawCapture = text.trim(),
        )

    suspend fun normalizeVoiceTranscript(transcript: String): CapturedSignalInput =
        CapturedSignalInput(
            modality = SignalModality.VOICE_TRANSCRIPT,
            rawCapture = transcript.trim(),
        )

    suspend fun normalizeImageOcrText(text: String): CapturedSignalInput =
        CapturedSignalInput(
            modality = SignalModality.IMAGE_OCR_TEXT,
            rawCapture = text.trim(),
        )
}
