import SwiftUI

/// Editable draft review with typed consent actions (M3.7–M3.10).
@MainActor
public struct ConsentReviewView: View {
    public enum Outcome: Equatable {
        case accept(FieldSignalDraft, ConsentDecision)
        case savePrivate(FieldSignalDraft)
        case reject
        case discard
    }

    private let original: FieldSignalDraft
    private let onOutcome: (Outcome) -> Void

    @State private var rawCapture: String
    @State private var summary: String
    @State private var latentNeed: String
    @State private var contradiction: String

    public init(
        draft: FieldSignalDraft,
        onOutcome: @escaping (Outcome) -> Void
    ) {
        original = draft
        self.onOutcome = onOutcome
        _rawCapture = State(initialValue: draft.rawCapture)
        _summary = State(initialValue: draft.summary)
        _latentNeed = State(initialValue: draft.latentNeed)
        _contradiction = State(initialValue: draft.contradiction)
    }

    public var body: some View {
        Section("Review draft") {
            LabeledContent("Workflow", value: original.workflowId)
            LabeledContent("Confidence", value: String(format: "%.2f", original.confidence.value))

            TextField("Summary", text: $summary, axis: .vertical)
                .lineLimit(3...6)
            TextField("Raw capture", text: $rawCapture, axis: .vertical)
                .lineLimit(4...8)
            TextField("Latent need", text: $latentNeed, axis: .vertical)
                .lineLimit(2...4)
            TextField("Contradiction / tension", text: $contradiction, axis: .vertical)
                .lineLimit(2...4)

            Button("Accept and queue") {
                onOutcome(.accept(reviewedDraft, consentDecision))
            }
            .buttonStyle(.borderedProminent)

            Button("Save private (local only)") {
                onOutcome(.savePrivate(reviewedDraft))
            }

            Button("Reject") {
                onOutcome(.reject)
            }
            .foregroundStyle(Brand.danger)

            Button("Discard") {
                onOutcome(.discard)
            }
            .foregroundStyle(Brand.inkMuted)
        }
    }

    private var reviewedDraft: FieldSignalDraft {
        original.withEdits(
            rawCapture: rawCapture,
            summary: summary,
            latentNeed: latentNeed,
            contradiction: contradiction
        )
    }

    private var consentDecision: ConsentDecision {
        let edited = rawCapture != original.rawCapture
            || summary != original.summary
            || latentNeed != original.latentNeed
            || contradiction != original.contradiction
        return edited ? .editedAndAccepted : .accepted
    }
}

extension FieldSignalDraft {
    func withEdits(
        rawCapture: String,
        summary: String,
        latentNeed: String,
        contradiction: String
    ) -> FieldSignalDraft {
        FieldSignalDraft(
            workflowId: workflowId,
            draftId: draftId,
            inquiryThreadId: inquiryThreadId,
            modality: modality,
            rawCapture: rawCapture,
            summary: summary,
            latentNeed: latentNeed,
            contradiction: contradiction,
            confidence: confidence,
            consentState: consentState
        )
    }
}

extension ConsentDecision {
    var label: String {
        switch self {
        case .accepted: "Accepted"
        case .editedAndAccepted: "Edited and accepted"
        case .rejected: "Rejected"
        case .savedPrivate: "Saved private"
        case .expired: "Expired"
        }
    }
}
