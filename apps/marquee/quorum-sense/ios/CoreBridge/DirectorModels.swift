import Foundation

// Mirrors `director-contracts` until UniFFI generated types replace this file.
// Canonical owner: `bedrock-platform/helms/crates/director-contracts`.

public struct DirectorSnapshot: Equatable, Sendable {
    public let version: UInt64
    public let frame: DirectorFrame
}

public struct DirectorFrame: Equatable, Sendable {
    public let frameId: String
    public let title: String
    public let subtitle: String?
    public let now: NowTask?
    public let waitingFor: WaitingFor
    public let primary: PrimaryAction
    public let secondary: [SecondaryAction]
    public let prompt: DirectorPrompt?
    public let presence: [PresenceHint]
    public let contextTrail: [ContextLevel]
    public let blocking: BlockingState
}

public struct NowTask: Equatable, Sendable {
    public let objective: String
    public let neededFromUser: String?
    public let estimatedMinutes: UInt32?
}

public enum WaitingFor: Equatable, Sendable {
    case nobody
    case participants(actorLabels: [String])
    case server
}

public struct PrimaryAction: Equatable, Sendable {
    public let label: String
    public let intent: DirectorIntent
}

public struct SecondaryAction: Equatable, Sendable {
    public let label: String
    public let intent: DirectorIntent
}

public enum DirectorPrompt: Equatable, Sendable {
    case judgment(JudgmentPrompt)
    case gate(GatePrompt)
    case review(ReviewPrompt)
}

public struct JudgmentPrompt: Equatable, Sendable {
    public let question: String
    public let body: String
    public let choices: [Choice]
}

public struct GatePrompt: Equatable, Sendable, Identifiable {
    public let gateId: String
    public let reason: String
    public let consequence: String
    public let deadlineMs: UInt64?

    public var id: String { gateId }
}

public struct ReviewPrompt: Equatable, Sendable {
    public let title: String
    public let primaryEvidence: String
}

public struct Choice: Equatable, Sendable, Identifiable {
    public let choiceId: String
    public let label: String

    public var id: String { choiceId }
}

public enum DirectorIntent: Equatable, Sendable {
    case openTask(frameId: String)
    case submitJudgment(frameId: String, choiceId: String)
    case respondGate(gateId: String, verdict: GateVerdict)
    case submitReview(frameId: String, stance: ReviewStance)
    case requestContext(level: ContextLevel)
}

// `GateVerdict`, `ReviewStance`, `ContextLevel`, and `BlockingState` are generated
// from the UDL in `CoreBridge/QuorumFFI.swift` (same pattern as `SignalModality`).

public struct PresenceHint: Equatable, Sendable, Identifiable {
    public let actorLabel: String
    public let status: String

    public var id: String { actorLabel }
}

public enum DirectorFixture {
    private static let frameId = "director-frame:session:procurement-security-review:1844"

    public static let quorumDecisionCheckpoint = DirectorSnapshot(
        version: 1844,
        frame: DirectorFrame(
            frameId: frameId,
            title: "The procurement formation has reached a decision checkpoint.",
            subtitle: "Good morning Kenneth.",
            now: NowTask(
                objective: "Evaluate Vendor X's security claims",
                neededFromUser: "Review the encryption section",
                estimatedMinutes: 4
            ),
            waitingFor: .nobody,
            primary: PrimaryAction(
                label: "Open Review",
                intent: .openTask(frameId: frameId)
            ),
            secondary: [
                SecondaryAction(
                    label: "Reject",
                    intent: .respondGate(
                        gateId: "gate:procurement-security-approval",
                        verdict: .reject
                    )
                ),
            ],
            prompt: .judgment(
                JudgmentPrompt(
                    question: "Does Vendor X's encryption section support the procurement claim?",
                    body: "Vendor X states that customer records are encrypted at rest with AES-256 and in transit with TLS 1.3. The claim does not state key ownership or customer-managed key support.",
                    choices: [
                        Choice(choiceId: "choice:yes", label: "Yes"),
                        Choice(choiceId: "choice:no", label: "No"),
                        Choice(choiceId: "choice:unsure", label: "Unsure"),
                    ]
                )
            ),
            presence: [
                PresenceHint(actorLabel: "Maria", status: "waiting_on_you"),
                PresenceHint(actorLabel: "Legal", status: "available"),
            ],
            contextTrail: [.task, .session, .formation, .organization],
            blocking: .blocksFormation
        )
    )
}
