import Foundation

// SignalModality, ConsentState, AppendEventType and SyncState are no longer
// declared here: UniFFI now generates them (CoreBridge/QuorumFFI.swift) straight
// from the UDL contract, so the wire enum *is* the domain enum — there are no
// String raw values to parse and an unknown case is unrepresentable. These
// extensions add only the host-side affordances the bindings don't emit: the
// modality picker's case list and the human-facing labels. The generated enums
// are already Equatable/Hashable/Sendable.

extension SignalModality: CaseIterable, Identifiable {
    // Synthesis of `allCases` only fires when the conformance sits beside the
    // enum's own declaration; this extension lives in a different file from the
    // generated enum, so the case list is spelled out by hand.
    public static var allCases: [SignalModality] { [.text, .voiceTranscript, .imageOcrText] }

    public var id: Self { self }

    public var label: String {
        switch self {
        case .text: "Text"
        case .voiceTranscript: "Voice"
        case .imageOcrText: "Photo OCR"
        }
    }
}

extension ConsentState {
    public var label: String {
        switch self {
        case .pending: "Pending"
        case .consented: "Consented"
        }
    }
}

extension AppendEventType {
    public var label: String {
        switch self {
        case .signalDraftConsented: "Signal draft consented"
        }
    }
}

extension SyncState {
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
