import SwiftUI

@MainActor
public struct SignalCaptureView: View {
    private let bridge: any QuorumCoreBridge
    private let inquiryThreadId = "inq_mobile_launch_risks"

    @State private var modality: SignalModality = .voiceTranscript
    @State private var rawCapture = "The sales team says rollout is fine, but support is seeing confusion in every pilot."
    @State private var draft: FieldSignalDraft?
    @State private var appendEvent: QuorumAppendEvent?
    @State private var persistedRecord: PersistedQueueRecordSummary?
    @State private var durableQueue: [PersistedQueueRecordSummary] = []
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
                        persistedRecord = nil
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
                    LabeledContent("Consent", value: draft.consentState.label)
                    LabeledContent("Confidence", value: String(format: "%.2f", draft.confidence.value))
                    Text(draft.summary)
                        .font(Brand.sans(15))
                        .foregroundStyle(Brand.ink)
                    Text(draft.contradiction)
                        .font(Brand.mono(12))
                        .foregroundStyle(Brand.inkMuted)

                    Button("Consent And Queue") {
                        Task {
                            errorMessage = nil
                            do {
                                appendEvent = try await bridge.appendConsentedSignal(draft)
                                persistedRecord = try await bridge.persistConsentedSignalToQueue(draft)
                                durableQueue = try await bridge.loadPersistedQueueRecords()
                            } catch {
                                appendEvent = nil
                                persistedRecord = nil
                                errorMessage = String(describing: error)
                            }
                        }
                    }
                }
            }

            if let appendEvent {
                Section("Queued Event") {
                    LabeledContent("Type", value: appendEvent.eventType.label)
                    LabeledContent("Sync", value: appendEvent.syncState.label)
                }
            }

            if let persistedRecord {
                Section("Durable Queue") {
                    LabeledContent("Record", value: persistedRecord.recordId)
                    LabeledContent("State", value: persistedRecord.queueState)
                    LabeledContent("Updated", value: persistedRecord.updatedAt)
                }
            }

            if !durableQueue.isEmpty {
                Section("Reloaded After Launch") {
                    ForEach(durableQueue) { record in
                        VStack(alignment: .leading, spacing: 4) {
                            Text(record.recordId)
                                .font(Brand.mono(12))
                            Text("\(record.queueState) · \(record.updatedAt)")
                                .font(Brand.sans(13))
                                .foregroundStyle(Brand.inkMuted)
                        }
                    }
                }
            }

            if let errorMessage {
                Section("Error") {
                    Text(errorMessage)
                        .font(Brand.mono(12))
                        .foregroundStyle(Brand.danger)
                }
            }
        }
        .scrollContentBackground(.hidden)
        .background(Brand.paper.ignoresSafeArea())
        .task {
            do {
                durableQueue = try await bridge.loadPersistedQueueRecords()
            } catch {
                errorMessage = String(describing: error)
            }
        }
    }
}
