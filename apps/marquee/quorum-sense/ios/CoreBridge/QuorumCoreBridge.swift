import Foundation

// Not @MainActor: the production bridge is an actor that runs the synchronous
// Rust FFI off the main thread (ADR 0003). Sendable so a `@MainActor` view can
// hold `any QuorumCoreBridge` and `await` across the actor boundary; the async
// methods let an actor- or main-actor-isolated witness satisfy them.
public protocol QuorumCoreBridge: Sendable {
    func workflowId() async -> String
    func draftFieldSignal(
        inquiryThreadId: String,
        modality: SignalModality,
        rawCapture: String
    ) async throws -> FieldSignalDraft
    func appendConsentedSignal(_ draft: FieldSignalDraft) async throws -> QuorumAppendEvent
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
    ) async throws -> FieldSignalDraft {
        FieldSignalDraft(
            workflowId: await workflowId(),
            draftId: "draft:\(inquiryThreadId):field-signal-v1",
            inquiryThreadId: inquiryThreadId,
            modality: modality,
            rawCapture: rawCapture,
            summary: String(rawCapture.trimmingCharacters(in: .whitespacesAndNewlines).prefix(96)),
            latentNeed: "needs earlier visibility into organizational ambiguity",
            contradiction: "participants report alignment while surfacing unresolved tension",
            confidence: Confidence(literal: 0.67),
            consentState: .pending
        )
    }

    public func appendConsentedSignal(_ draft: FieldSignalDraft) async throws -> QuorumAppendEvent {
        QuorumAppendEvent(
            workflowId: draft.workflowId,
            eventType: .signalDraftConsented,
            draftId: draft.draftId,
            inquiryThreadId: draft.inquiryThreadId,
            syncState: .queuedForSync
        )
    }
}
