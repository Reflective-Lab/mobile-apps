import Foundation

#if canImport(FoundationModels)
import FoundationModels
#endif

#if canImport(Speech)
import Speech
#endif

#if canImport(Vision)
import Vision
#endif

public struct CapturedSignalInput: Equatable {
    public let modality: SignalModality
    public let rawCapture: String
}

public struct NormalizedCaptureResult: Equatable {
    public let input: CapturedSignalInput
    /// True when a platform AI path produced structured output; false for typed trim fallback.
    public let usedPlatformAI: Bool
}

public struct PlatformSignalExtractor {
    public init() {}

    /// Normalize native capture into bridge input, with typed fallback when platform AI is unavailable (M3.6).
    public func normalizeCapture(modality: SignalModality, text: String) async -> NormalizedCaptureResult {
        let trimmed = text.trimmingCharacters(in: .whitespacesAndNewlines)
        #if canImport(FoundationModels)
        if #available(iOS 26.0, *), !trimmed.isEmpty {
            // Platform AI hook lands here; until then we fall through to deterministic normalization.
        }
        #endif
        let input: CapturedSignalInput
        switch modality {
        case .text:
            input = await normalizeTextCapture(trimmed)
        case .voiceTranscript:
            input = await normalizeVoiceTranscript(trimmed)
        case .imageOcrText:
            input = await normalizeImageOcrText(trimmed)
        }
        return NormalizedCaptureResult(input: input, usedPlatformAI: false)
    }

    public func normalizeTextCapture(_ text: String) async -> CapturedSignalInput {
        CapturedSignalInput(
            modality: .text,
            rawCapture: text.trimmingCharacters(in: .whitespacesAndNewlines)
        )
    }

    public func normalizeVoiceTranscript(_ transcript: String) async -> CapturedSignalInput {
        CapturedSignalInput(
            modality: .voiceTranscript,
            rawCapture: transcript.trimmingCharacters(in: .whitespacesAndNewlines)
        )
    }

    public func normalizeImageOcrText(_ text: String) async -> CapturedSignalInput {
        CapturedSignalInput(
            modality: .imageOcrText,
            rawCapture: text.trimmingCharacters(in: .whitespacesAndNewlines)
        )
    }
}

