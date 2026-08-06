import SwiftUI

@MainActor
public struct DirectorNowView: View {
    private let bridge: any QuorumCoreBridge

    @State private var snapshot: DirectorSnapshot?
    @State private var snapshotSource: String = "…"
    @State private var statusMessage: String?
    @State private var errorMessage: String?
    @State private var gateInterrupt: GatePrompt?

    public init(bridge: any QuorumCoreBridge = PreviewQuorumCoreBridge()) {
        self.bridge = bridge
    }

    public var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 20) {
                if let snapshot {
                    HStack {
                        Text("Snapshot \(snapshot.version)")
                            .font(Brand.monoMedium(11))
                            .foregroundStyle(Brand.inkMuted)
                        Spacer()
                        Text(snapshotSource)
                            .font(Brand.monoMedium(11))
                            .foregroundStyle(Brand.accentDark)
                    }

                    if let subtitle = snapshot.frame.subtitle {
                        Text(subtitle)
                            .font(Brand.sansMedium(17))
                            .foregroundStyle(Brand.inkSoft)
                    }

                    Text(snapshot.frame.title)
                        .font(Brand.display(32))
                        .foregroundStyle(Brand.ink)
                        .fixedSize(horizontal: false, vertical: true)

                    if let now = snapshot.frame.now {
                        DirectorTaskCard(
                            now: now,
                            waitingFor: snapshot.frame.waitingFor,
                            blocking: snapshot.frame.blocking,
                            primary: snapshot.frame.primary,
                            onIntent: submit
                        )
                    }

                    if let prompt = snapshot.frame.prompt {
                        promptView(prompt, frameId: snapshot.frame.frameId)
                    }

                    gateActions(from: snapshot.frame, onIntent: submit)

                    ContextTrailView(
                        levels: snapshot.frame.contextTrail,
                        onIntent: submit
                    )

                    PresenceStrip(presence: snapshot.frame.presence)
                } else {
                    ProgressView("Loading Director")
                        .font(Brand.sans(15))
                }

                if let statusMessage {
                    Text(statusMessage)
                        .font(Brand.mono(12))
                        .foregroundStyle(Brand.ok)
                }

                if let errorMessage {
                    Text(errorMessage)
                        .font(Brand.mono(12))
                        .foregroundStyle(Brand.danger)
                }
            }
            .padding(24)
        }
        .scrollContentBackground(.hidden)
        .background(Brand.paper.ignoresSafeArea())
        .task {
            await refreshLoop()
        }
        .fullScreenCover(item: $gateInterrupt) { gate in
            if let snapshot {
                GateInterruptView(
                    snapshot: snapshot,
                    gate: gate,
                    onIntent: submit,
                    onDismiss: { gateInterrupt = nil }
                )
            }
        }
        .onChange(of: snapshot?.frame.prompt) { _, prompt in
            if case .gate(let gate) = prompt,
               snapshot?.frame.blocking == .blocksFormation {
                gateInterrupt = gate
            }
        }
    }

    private func refreshLoop() async {
        await loadSnapshot()
        let initialVersion = snapshot?.version ?? 0
        while !Task.isCancelled {
            let updated = await bridge.waitDirectorUpdate(
                sinceVersion: snapshot?.version ?? initialVersion,
                timeoutMs: 30_000
            )
            if updated {
                await loadSnapshot()
            }
        }
    }

    @ViewBuilder
    private func promptView(_ prompt: DirectorPrompt, frameId: String) -> some View {
        switch prompt {
        case .judgment(let judgment):
            JudgmentPromptView(
                prompt: judgment,
                frameId: frameId,
                onIntent: submit
            )
        case .gate(let gate):
            GatePromptView(prompt: gate, onIntent: submit)
        case .review(let review):
            ReviewPromptView(prompt: review, frameId: frameId, onIntent: submit)
        }
    }

    @ViewBuilder
    private func gateActions(from frame: DirectorFrame, onIntent: @escaping (DirectorIntent) -> Void) -> some View {
        ForEach(Array(frame.secondary.enumerated()), id: \.offset) { _, action in
            Button(action.label) {
                onIntent(action.intent)
            }
            .buttonStyle(.bordered)
            .controlSize(.large)
        }
    }

    private func loadSnapshot() async {
        do {
            snapshot = try await bridge.currentDirectorSnapshot()
            snapshotSource = await bridge.directorSnapshotSource()
            errorMessage = nil
        } catch {
            snapshot = nil
            snapshotSource = "error"
            errorMessage = String(describing: error)
        }
    }

    private func submit(_ intent: DirectorIntent) {
        Task {
            do {
                try await bridge.submitDirectorIntent(intent)
                statusMessage = intent.statusLabel
                errorMessage = nil
            } catch {
                statusMessage = nil
                errorMessage = String(describing: error)
            }
        }
    }
}

public struct DirectorTaskCard: View {
    public let now: NowTask
    public let waitingFor: WaitingFor
    public let blocking: BlockingState
    public let primary: PrimaryAction
    public let onIntent: (DirectorIntent) -> Void

    public var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            HStack(alignment: .top) {
                VStack(alignment: .leading, spacing: 4) {
                    Text("Current objective")
                        .directorOverline()
                    Text(now.objective)
                        .font(Brand.sansBold(20))
                        .foregroundStyle(Brand.ink)
                }

                Spacer()

                if let minutes = now.estimatedMinutes {
                    VStack(alignment: .trailing, spacing: 4) {
                        Text("\(minutes) min")
                            .font(Brand.monoMedium(13))
                            .foregroundStyle(Brand.accentDark)
                        Text(waitingFor.label)
                            .font(Brand.mono(11))
                            .foregroundStyle(Brand.inkMuted)
                    }
                }
            }

            if let needed = now.neededFromUser {
                VStack(alignment: .leading, spacing: 4) {
                    Text("Needed from you")
                        .directorOverline()
                    Text(needed)
                        .font(Brand.display(26))
                        .foregroundStyle(Brand.ink)
                }
            }

            if blocking != .notBlocking {
                Text(blocking.label)
                    .font(Brand.monoMedium(11))
                    .padding(.horizontal, 10)
                    .padding(.vertical, 6)
                    .background(Brand.danger.opacity(0.10), in: Capsule())
                    .foregroundStyle(Brand.danger)
            }

            Button {
                onIntent(primary.intent)
            } label: {
                Text(primary.label)
                    .frame(maxWidth: .infinity)
            }
            .buttonStyle(.borderedProminent)
            .controlSize(.large)
        }
        .padding(20)
        .background(Brand.surface, in: RoundedRectangle(cornerRadius: 28, style: .continuous))
        .overlay {
            RoundedRectangle(cornerRadius: 28, style: .continuous)
                .stroke(Brand.line)
        }
    }
}

public struct JudgmentPromptView: View {
    public let prompt: JudgmentPrompt
    public let frameId: String
    public let onIntent: (DirectorIntent) -> Void

    public var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            Text("Focused judgment")
                .directorOverline()
            Text(prompt.question)
                .font(Brand.sansBold(19))
                .foregroundStyle(Brand.ink)
            Text(prompt.body)
                .font(Brand.sans(15))
                .foregroundStyle(Brand.inkSoft)
                .padding(14)
                .background(Brand.surfaceMuted, in: RoundedRectangle(cornerRadius: 16, style: .continuous))

            HStack(spacing: 10) {
                ForEach(prompt.choices) { choice in
                    Button(choice.label) {
                        onIntent(.submitJudgment(frameId: frameId, choiceId: choice.choiceId))
                    }
                    .buttonStyle(.bordered)
                    .controlSize(.large)
                }
            }
        }
        .directorPanel()
    }
}

public struct GatePromptView: View {
    public let prompt: GatePrompt
    public let onIntent: (DirectorIntent) -> Void

    public var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            Text("Gate")
                .directorOverline()
            Text(prompt.reason)
                .font(Brand.sansBold(18))
                .foregroundStyle(Brand.ink)
            Text(prompt.consequence)
                .font(Brand.sans(15))
                .foregroundStyle(Brand.inkSoft)
            if let deadlineMs = prompt.deadlineMs {
                Text("Deadline \(deadlineMs)")
                    .font(Brand.mono(12))
                    .foregroundStyle(Brand.inkMuted)
            }

            HStack(spacing: 10) {
                Button("Approve") {
                    onIntent(.respondGate(gateId: prompt.gateId, verdict: .approve))
                }
                .buttonStyle(.borderedProminent)
                .controlSize(.large)

                Button("Reject") {
                    onIntent(.respondGate(gateId: prompt.gateId, verdict: .reject))
                }
                .buttonStyle(.bordered)
                .controlSize(.large)
            }
        }
        .directorPanel()
    }
}

public struct ReviewPromptView: View {
    public let prompt: ReviewPrompt
    public let frameId: String
    public let onIntent: (DirectorIntent) -> Void

    public var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            Text("Review")
                .directorOverline()
            Text(prompt.title)
                .font(Brand.sansBold(18))
            Text(prompt.primaryEvidence)
                .font(Brand.sans(15))
                .foregroundStyle(Brand.inkSoft)

            HStack(spacing: 10) {
                ForEach([ReviewStance.agree, .disagree, .needMoreContext], id: \.self) { stance in
                    Button(stance.label) {
                        onIntent(.submitReview(frameId: frameId, stance: stance))
                    }
                    .buttonStyle(.bordered)
                }
            }
        }
        .directorPanel()
    }
}

public struct ContextTrailView: View {
    public let levels: [ContextLevel]
    public let onIntent: (DirectorIntent) -> Void

    public var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("Context escape")
                .directorOverline()
            HStack(spacing: 8) {
                ForEach(levels) { level in
                    Button(level.label) {
                        onIntent(.requestContext(level: level))
                    }
                    .buttonStyle(.bordered)
                    .font(Brand.mono(12))
                }
            }
        }
    }
}

struct PresenceStrip: View {
    let presence: [PresenceHint]

    var body: some View {
        HStack(spacing: -6) {
            ForEach(presence) { person in
                Text(String(person.actorLabel.prefix(1)))
                    .font(Brand.sansBold(12))
                    .foregroundStyle(Brand.surface)
                    .frame(width: 32, height: 32)
                    .background(person.status == "waiting_on_you" ? Brand.accent : Brand.inkMuted, in: Circle())
                    .overlay(Circle().stroke(Brand.paper, lineWidth: 2))
                    .accessibilityLabel("\(person.actorLabel), \(person.status)")
            }

            Text("Presence")
                .font(Brand.mono(11))
                .foregroundStyle(Brand.inkMuted)
                .padding(.leading, 14)
        }
    }
}

private extension WaitingFor {
    var label: String {
        switch self {
        case .nobody: "Nobody"
        case .participants(let labels): labels.joined(separator: ", ")
        case .server: "Server"
        }
    }
}

private extension DirectorIntent {
    var statusLabel: String {
        switch self {
        case .openTask: "Opened current task"
        case .submitJudgment: "Judgment submitted"
        case .respondGate(_, let verdict): "Gate signaled: \(verdict.wireLabel)"
        case .submitReview: "Review submitted"
        case .requestContext(let level): "Opened \(level.label.lowercased()) context"
        }
    }
}

extension View {
    func directorOverline() -> some View {
        font(Brand.monoMedium(11))
            .textCase(.uppercase)
            .foregroundStyle(Brand.inkMuted)
    }

    func directorPanel() -> some View {
        padding(18)
            .background(Brand.surface, in: RoundedRectangle(cornerRadius: 22, style: .continuous))
            .overlay {
                RoundedRectangle(cornerRadius: 22, style: .continuous)
                    .stroke(Brand.line)
            }
    }
}

#Preview("AI Director") {
    NavigationStack {
        DirectorNowView()
            .navigationTitle("Director")
    }
}
