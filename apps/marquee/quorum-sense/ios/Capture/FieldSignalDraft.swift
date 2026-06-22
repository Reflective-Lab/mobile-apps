import Foundation

public enum SignalModality: String, CaseIterable, Identifiable, Sendable {
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

/// Whether a captured signal has cleared consent for sync. Mirrors the Rust
/// `ConsentState`; raw values are the wire contract from the core.
public enum ConsentState: String, Sendable {
    case pending
    case consented

    public var label: String {
        switch self {
        case .pending: "Pending"
        case .consented: "Consented"
        }
    }
}

/// The kind of event emitted when a draft is appended. Mirrors Rust `AppendEventType`.
public enum AppendEventType: String, Sendable {
    case signalDraftConsented = "SignalDraftConsented"

    public var label: String {
        switch self {
        case .signalDraftConsented: "Signal draft consented"
        }
    }
}

/// Where an appended event sits in the sync pipeline. Mirrors Rust `SyncState`.
public enum SyncState: String, Sendable {
    case queuedForSync = "queued_for_sync"

    public var label: String {
        switch self {
        case .queuedForSync: "Queued for sync"
        }
    }
}

/// A model confidence score constrained to `0...1`, mirroring Rust `Confidence`.
public struct Confidence: Equatable, Sendable {
    public let value: Float

    /// Validated construction from untrusted input (e.g. core output).
    public init?(_ value: Float) {
        guard value.isFinite, (0...1).contains(value) else { return nil }
        self.value = value
    }

    /// Trusted construction for compile-time-known-valid literals (previews/tests),
    /// the Swift analogue of the same-module direct construction Rust uses.
    init(literal value: Float) {
        self.value = value
    }
}

public struct FieldSignalDraft: Equatable, Identifiable, Sendable {
    public let workflowId: String
    public let draftId: String
    public let inquiryThreadId: String
    public let modality: SignalModality
    public let rawCapture: String
    public let summary: String
    public let latentNeed: String
    public let contradiction: String
    public let confidence: Confidence
    public let consentState: ConsentState

    public var id: String { draftId }
}

public struct QuorumAppendEvent: Equatable, Sendable {
    public let workflowId: String
    public let eventType: AppendEventType
    public let draftId: String
    public let inquiryThreadId: String
    public let syncState: SyncState
}
