import Foundation

public enum SignalModality: String, CaseIterable, Identifiable {
    case text
    case voiceTranscript = "voice_transcript"
    case imageOcrText = "image_ocr_text"

    public var id: String { rawValue }

    public var label: String {
        switch self {
        case .text: "Text"
        case .voiceTranscript: "Voice"
        case .imageOcrText: "Photo OCR"
        }
    }
}

public struct FieldSignalDraft: Equatable, Identifiable {
    public let workflowId: String
    public let draftId: String
    public let inquiryThreadId: String
    public let modality: SignalModality
    public let rawCapture: String
    public let summary: String
    public let latentNeed: String
    public let contradiction: String
    public let confidence: Float
    public let consentState: String

    public var id: String { draftId }
}

public struct QuorumAppendEvent: Equatable {
    public let workflowId: String
    public let eventType: String
    public let draftId: String
    public let inquiryThreadId: String
    public let syncState: String
}

