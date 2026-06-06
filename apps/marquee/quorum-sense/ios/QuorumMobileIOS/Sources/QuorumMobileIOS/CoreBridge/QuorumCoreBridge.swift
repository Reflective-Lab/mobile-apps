import Foundation

public protocol QuorumCoreBridge: Sendable {
    func workflowId() async -> String
    func draftFieldSignal(
        inquiryThreadId: String,
        modality: SignalModality,
        rawCapture: String
    ) async -> FieldSignalDraft
    func appendConsentedSignal(_ draft: FieldSignalDraft) async -> QuorumAppendEvent
}

public struct PreviewQuorumCoreBridge: QuorumCoreBridge {
    public init() {}

    public func workflowId() async -> String {
        "quorum.field_signal_capture.v1"
    }

    public func draftFieldSignal(
        inquiryThreadId: String,
        modality: SignalModality,
        rawCapture: String
    ) async -> FieldSignalDraft {
        FieldSignalDraft(
            workflowId: await workflowId(),
            draftId: "draft:\(inquiryThreadId):field-signal-v1",
            inquiryThreadId: inquiryThreadId,
            modality: modality,
            rawCapture: rawCapture,
            summary: String(rawCapture.trimmingCharacters(in: .whitespacesAndNewlines).prefix(96)),
            latentNeed: "needs earlier visibility into organizational ambiguity",
            contradiction: "participants report alignment while surfacing unresolved tension",
            confidence: 0.67,
            consentState: "pending"
        )
    }

    public func appendConsentedSignal(_ draft: FieldSignalDraft) async -> QuorumAppendEvent {
        QuorumAppendEvent(
            workflowId: draft.workflowId,
            eventType: "SignalDraftConsented",
            draftId: draft.draftId,
            inquiryThreadId: draft.inquiryThreadId,
            syncState: "queued_for_sync"
        )
    }
}

// Replace PreviewQuorumCoreBridge with a UniFFI generated adapter when the
// Rust bridge is generated from schemas/quorum-mobile.udl.

