import SwiftUI

@MainActor
public struct SignalCaptureView: View {
    private let bridge: any QuorumCoreBridge
    private let inquiryThreadId = "inq_mobile_launch_risks"

    @State private var modality: SignalModality = .voiceTranscript
    @State private var rawCapture = "The sales team says rollout is fine, but support is seeing confusion in every pilot."
    @State private var draft: FieldSignalDraft?
    @State private var appendEvent: QuorumAppendEvent?
    @State private var errorMessage: String?

    public init(bridge: any QuorumCoreBridge = PreviewQuorumCoreBridge()) {
        self.bridge = bridge
    }

    public var body: some View {
        Form {
            Section("Capture") {
                Picker("Input", selection: $modality) {
                    ForEach(SignalModality.allCases) { item in
                        Text(item.label).tag(item)
                    }
                }

                TextEditor(text: $rawCapture)
                    .frame(minHeight: 120)

                Button("Create Draft") {
                    Task {
                        appendEvent = nil
                        errorMessage = nil
                        do {
                            draft = try await bridge.draftFieldSignal(
                                inquiryThreadId: inquiryThreadId,
                                modality: modality,
                                rawCapture: rawCapture
                            )
                        } catch {
                            draft = nil
                            errorMessage = String(describing: error)
                        }
                    }
                }
            }

            if let draft {
                Section("Draft") {
                    LabeledContent("Workflow", value: draft.workflowId)
                    LabeledContent("Consent", value: draft.consentState)
                    LabeledContent("Confidence", value: String(format: "%.2f", draft.confidence))
                    Text(draft.summary)
                    Text(draft.contradiction)
                        .font(.footnote)
                        .foregroundStyle(.secondary)

                    Button("Consent And Queue") {
                        Task {
                            errorMessage = nil
                            do {
                                appendEvent = try await bridge.appendConsentedSignal(draft)
                            } catch {
                                appendEvent = nil
                                errorMessage = String(describing: error)
                            }
                        }
                    }
                }
            }

            if let appendEvent {
                Section("Queued Event") {
                    LabeledContent("Type", value: appendEvent.eventType)
                    LabeledContent("Sync", value: appendEvent.syncState)
                }
            }

            if let errorMessage {
                Section("Error") {
                    Text(errorMessage)
                        .font(.footnote)
                        .foregroundStyle(.red)
                }
            }
        }
    }
}

