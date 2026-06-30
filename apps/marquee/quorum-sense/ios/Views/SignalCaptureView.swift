import SwiftUI

@MainActor
public struct SignalCaptureView: View {
    private let bridge: any QuorumCoreBridge
    private let inquiryThreadId: String
    private let extractor = PlatformSignalExtractor()
    private let speech = SpeechCaptureService()

    @State private var modality: SignalModality = .text
    @State private var rawCapture = ""
    @State private var draft: FieldSignalDraft?
    @State private var appendEvent: QuorumAppendEvent?
    @State private var persistedRecord: PersistedQueueRecordSummary?
    @State private var durableQueue: [PersistedQueueRecordSummary] = []
    @State private var privateDrafts: [FieldSignalDraft] = []
    @State private var normalizationNote: String?
    @State private var statusMessage: String?
    @State private var errorMessage: String?

    public init(
        bridge: any QuorumCoreBridge = PreviewQuorumCoreBridge(),
        inquiryThreadId: String = CaptureSessionContext.inquiryThreadId()
    ) {
        self.bridge = bridge
        self.inquiryThreadId = inquiryThreadId
    }

    public var body: some View {
        Form {
            Section("Capture") {
                Picker("Input", selection: $modality) {
                    ForEach(SignalModality.allCases) { item in
                        Text(item.label).tag(item)
                    }
                }

                if modality == .voiceTranscript {
                    speechSection
                } else {
                    TextEditor(text: $rawCapture)
                        .frame(minHeight: 120)
                }

                if let normalizationNote {
                    Text(normalizationNote)
                        .font(Brand.mono(11))
                        .foregroundStyle(Brand.inkMuted)
                }

                Button("Create Draft") {
                    Task { await createDraft() }
                }
                .disabled(captureText.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
            }

            if let draft {
                ConsentReviewView(draft: draft) { outcome in
                    handleConsentOutcome(outcome)
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

            if !privateDrafts.isEmpty {
                Section("Private drafts (this session)") {
                    ForEach(privateDrafts) { item in
                        Text(item.summary)
                            .font(Brand.sans(14))
                    }
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

            if let statusMessage {
                Section {
                    Text(statusMessage)
                        .font(Brand.sans(14))
                        .foregroundStyle(Brand.ok)
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
            await speech.refreshAuthorization()
            do {
                durableQueue = try await bridge.loadPersistedQueueRecords()
            } catch {
                errorMessage = String(describing: error)
            }
        }
        .onChange(of: speech.transcript) { _, transcript in
            if modality == .voiceTranscript, !transcript.isEmpty {
                rawCapture = transcript
            }
        }
        .onDisappear {
            speech.stopRecording()
        }
    }

    @ViewBuilder
    private var speechSection: some View {
        switch speech.authorizationState {
        case .undetermined:
            Button("Allow microphone and speech") {
                Task { await speech.requestAuthorization() }
            }
        case .denied, .restricted:
            Text("Speech capture unavailable. Enable Microphone and Speech Recognition in Settings.")
                .font(Brand.sans(14))
                .foregroundStyle(Brand.danger)
        case .authorized:
            if speech.isRecording {
                Button("Stop recording") {
                    speech.stopRecording()
                }
            } else {
                Button("Start voice capture") {
                    do {
                        try speech.startRecording()
                    } catch {
                        errorMessage = String(describing: error)
                    }
                }
            }
            if !speech.transcript.isEmpty {
                Text(speech.transcript)
                    .font(Brand.sans(15))
            }
        }
    }

    private var captureText: String {
        modality == .voiceTranscript ? speech.transcript : rawCapture
    }

    private func createDraft() async {
        appendEvent = nil
        persistedRecord = nil
        statusMessage = nil
        errorMessage = nil
        draft = nil

        do {
            let normalized = await extractor.normalizeCapture(modality: modality, text: captureText)
            normalizationNote = normalized.usedPlatformAI
                ? "Normalized with platform AI."
                : "Normalized locally (platform AI unavailable)."
            draft = try await bridge.draftFieldSignal(
                inquiryThreadId: inquiryThreadId,
                modality: normalized.input.modality,
                rawCapture: normalized.input.rawCapture
            )
        } catch {
            errorMessage = String(describing: error)
        }
    }

    private func handleConsentOutcome(_ outcome: ConsentReviewView.Outcome) {
        Task {
            statusMessage = nil
            errorMessage = nil
            switch outcome {
            case .accept(let reviewed, let decision):
                await queueConsented(reviewed, decision: decision)
            case .savePrivate(let reviewed):
                privateDrafts.append(reviewed)
                draft = nil
                statusMessage = "Saved private — not queued for sync."
            case .reject:
                draft = nil
                statusMessage = "Rejected — no sync event created."
            case .discard:
                draft = nil
                statusMessage = "Discarded."
            }
        }
    }

    private func queueConsented(_ reviewed: FieldSignalDraft, decision: ConsentDecision) async {
        do {
            guard decision.permitsQueue else {
                errorMessage = "Consent \(decision.label) cannot enter the queue."
                return
            }
            appendEvent = try await bridge.appendConsentedSignal(reviewed)
            persistedRecord = try await bridge.persistConsentedSignalToQueue(
                reviewed,
                consentDecision: decision
            )
            durableQueue = try await bridge.loadPersistedQueueRecords()
            draft = nil
            statusMessage = "Queued with consent: \(decision.label)."
        } catch {
            appendEvent = nil
            persistedRecord = nil
            errorMessage = String(describing: error)
        }
    }
}
