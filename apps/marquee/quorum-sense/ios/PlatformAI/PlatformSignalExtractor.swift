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

public struct PlatformSignalExtractor {
    public init() {}

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

