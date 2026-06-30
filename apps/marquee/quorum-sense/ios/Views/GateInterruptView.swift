import SwiftUI

/// UX1 Gate Interrupt — full-screen blocking gate surface (see KB architecture §4).
@MainActor
public struct GateInterruptView: View {
    public let snapshot: DirectorSnapshot
    public let gate: GatePrompt
    public let onIntent: (DirectorIntent) -> Void
    public let onDismiss: () -> Void

    public init(
        snapshot: DirectorSnapshot,
        gate: GatePrompt,
        onIntent: @escaping (DirectorIntent) -> Void,
        onDismiss: @escaping () -> Void
    ) {
        self.snapshot = snapshot
        self.gate = gate
        self.onIntent = onIntent
        self.onDismiss = onDismiss
    }

    public var body: some View {
        ZStack(alignment: .topTrailing) {
            ScrollView {
                VStack(alignment: .leading, spacing: 28) {
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Cannot continue")
                            .font(Brand.display(34))
                            .foregroundStyle(Brand.ink)
                        if let subtitle = snapshot.frame.subtitle {
                            Text(subtitle)
                                .font(Brand.sansMedium(17))
                                .foregroundStyle(Brand.inkSoft)
                        }
                    }

                    VStack(alignment: .leading, spacing: 12) {
                        Text(gate.reason)
                            .font(Brand.sansBold(20))
                            .foregroundStyle(Brand.ink)
                            .fixedSize(horizontal: false, vertical: true)

                        Text(gate.consequence)
                            .font(Brand.sans(16))
                            .foregroundStyle(Brand.inkSoft)
                            .fixedSize(horizontal: false, vertical: true)
                    }
                    .padding(20)
                    .background(Brand.danger.opacity(0.08), in: RoundedRectangle(cornerRadius: 24, style: .continuous))
                    .overlay {
                        RoundedRectangle(cornerRadius: 24, style: .continuous)
                            .stroke(Brand.danger.opacity(0.25))
                    }

                    if let now = snapshot.frame.now {
                        VStack(alignment: .leading, spacing: 8) {
                            Text("Decision needed")
                                .directorOverline()
                            Text(now.neededFromUser ?? now.objective)
                                .font(Brand.sansBold(22))
                                .foregroundStyle(Brand.ink)
                        }
                    }

                    if let deadlineMs = gate.deadlineMs {
                        VStack(alignment: .leading, spacing: 4) {
                            Text("Deadline")
                                .directorOverline()
                            Text(deadlineLabel(deadlineMs))
                                .font(Brand.monoMedium(14))
                                .foregroundStyle(Brand.inkMuted)
                        }
                    }

                    PresenceStrip(presence: snapshot.frame.presence)

                    HStack(spacing: 12) {
                        Button {
                            onIntent(.respondGate(gateId: gate.gateId, verdict: .approve))
                            onDismiss()
                        } label: {
                            Text("Approve")
                                .frame(maxWidth: .infinity)
                        }
                        .buttonStyle(.borderedProminent)
                        .controlSize(.large)

                        Button {
                            onIntent(.respondGate(gateId: gate.gateId, verdict: .reject))
                            onDismiss()
                        } label: {
                            Text("Reject")
                                .frame(maxWidth: .infinity)
                        }
                        .buttonStyle(.bordered)
                        .controlSize(.large)
                    }
                }
                .padding(28)
            }
            .scrollContentBackground(.hidden)
            .background(Brand.paper.ignoresSafeArea())

            Button("Later") {
                onDismiss()
            }
            .font(Brand.sansMedium(15))
            .foregroundStyle(Brand.inkMuted)
            .padding(24)
        }
    }

    private func deadlineLabel(_ deadlineMs: UInt64) -> String {
        let seconds = TimeInterval(deadlineMs) / 1000
        let date = Date(timeIntervalSince1970: seconds)
        let formatter = DateFormatter()
        formatter.dateStyle = .medium
        formatter.timeStyle = .short
        return formatter.string(from: date)
    }
}

#if DEBUG
#Preview("Gate interrupt") {
    GateInterruptView(
        snapshot: DirectorSnapshot(
            version: 1901,
            frame: DirectorFrame(
                frameId: "director-frame:gate",
                title: "Cannot continue",
                subtitle: nil,
                now: NowTask(
                    objective: "Legal approval required",
                    neededFromUser: "Approve revised liability wording",
                    estimatedMinutes: nil
                ),
                waitingFor: .participants(actorLabels: ["Legal"]),
                primary: PrimaryAction(
                    label: "Approve",
                    intent: .respondGate(gateId: "gate:1", verdict: .approve)
                ),
                secondary: [],
                prompt: .gate(GatePrompt(
                    gateId: "gate:1",
                    reason: "Legal approval required before the formation can claim success.",
                    consequence: "The procurement formation cannot advance until this gate is resolved.",
                    deadlineMs: 1_735_689_600_000
                )),
                presence: [PresenceHint(actorLabel: "Legal", status: "waiting_on_you")],
                contextTrail: [.task, .session],
                blocking: .blocksFormation
            )
        ),
        gate: GatePrompt(
            gateId: "gate:1",
            reason: "Legal approval required before the formation can claim success.",
            consequence: "The procurement formation cannot advance until this gate is resolved.",
            deadlineMs: 1_735_689_600_000
        ),
        onIntent: { _ in },
        onDismiss: {}
    )
}
#endif
