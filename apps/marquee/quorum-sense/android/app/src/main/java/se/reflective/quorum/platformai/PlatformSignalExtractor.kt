package se.reflective.quorum.platformai

import uniffi.quorum_ffi.SignalModality

data class CapturedSignalInput(
    val modality: SignalModality,
    val rawCapture: String,
)

class PlatformSignalExtractor {
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

