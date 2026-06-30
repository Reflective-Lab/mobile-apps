import Foundation

/// Maps UniFFI director wire enums to Swift domain types in `DirectorModels.swift`.
/// Tagged shapes (`WaitingFor`, `DirectorPrompt`, `DirectorIntent`) are proper
/// UniFFI interface enums on the wire (M3A.8); flat enums cross unchanged.
enum DirectorBridgeMapping {
    static func domainSnapshot(_ ffi: FfiDirectorSnapshot) -> DirectorSnapshot {
        DirectorSnapshot(
            version: ffi.version,
            frame: domainFrame(ffi.frame)
        )
    }

    static func ffiIntent(_ intent: DirectorIntent) -> FfiDirectorIntent {
        switch intent {
        case .openTask(let frameId):
            .openTask(frameId: frameId)
        case .submitJudgment(let frameId, let choiceId):
            .submitJudgment(frameId: frameId, choiceId: choiceId)
        case .respondGate(let gateId, let verdict):
            .respondGate(gateId: gateId, verdict: verdict)
        case .submitReview(let frameId, let stance):
            .submitReview(frameId: frameId, stance: stance)
        case .requestContext(let level):
            .requestContext(level: level)
        }
    }

    private static func domainFrame(_ ffi: FfiDirectorFrame) -> DirectorFrame {
        DirectorFrame(
            frameId: ffi.frameId,
            title: ffi.title,
            subtitle: ffi.subtitle,
            now: ffi.now.map(domainNow),
            waitingFor: domainWaitingFor(ffi.waitingFor),
            primary: domainPrimary(ffi.primary),
            secondary: ffi.secondary.map(domainSecondary),
            prompt: ffi.prompt.map(domainPrompt),
            presence: ffi.presence.map(domainPresence),
            contextTrail: ffi.contextTrail,
            blocking: ffi.blocking
        )
    }

    private static func domainNow(_ ffi: FfiNowTask) -> NowTask {
        NowTask(
            objective: ffi.objective,
            neededFromUser: ffi.neededFromUser,
            estimatedMinutes: ffi.estimatedMinutes
        )
    }

    private static func domainWaitingFor(_ ffi: FfiWaitingFor) -> WaitingFor {
        switch ffi {
        case .nobody:
            .nobody
        case .participants(let actorLabels):
            .participants(actorLabels: actorLabels)
        case .server:
            .server
        }
    }

    private static func domainPrimary(_ ffi: FfiPrimaryAction) -> PrimaryAction {
        PrimaryAction(label: ffi.label, intent: domainIntent(ffi.intent))
    }

    private static func domainSecondary(_ ffi: FfiSecondaryAction) -> SecondaryAction {
        SecondaryAction(label: ffi.label, intent: domainIntent(ffi.intent))
    }

    private static func domainPrompt(_ ffi: FfiDirectorPrompt) -> DirectorPrompt {
        switch ffi {
        case .judgment(let judgment):
            .judgment(
                JudgmentPrompt(
                    question: judgment.question,
                    body: judgment.body,
                    choices: judgment.choices.map(domainChoice)
                )
            )
        case .gate(let gate):
            .gate(
                GatePrompt(
                    gateId: gate.gateId,
                    reason: gate.reason,
                    consequence: gate.consequence,
                    deadlineMs: gate.deadlineMs
                )
            )
        case .review(let review):
            .review(
                ReviewPrompt(
                    title: review.title,
                    primaryEvidence: review.primaryEvidence
                )
            )
        }
    }

    private static func domainChoice(_ ffi: FfiChoice) -> Choice {
        Choice(choiceId: ffi.choiceId, label: ffi.label)
    }

    private static func domainPresence(_ ffi: FfiPresenceHint) -> PresenceHint {
        PresenceHint(actorLabel: ffi.actorLabel, status: ffi.status)
    }

    private static func domainIntent(_ ffi: FfiDirectorIntent) -> DirectorIntent {
        switch ffi {
        case .openTask(let frameId):
            .openTask(frameId: frameId)
        case .submitJudgment(let frameId, let choiceId):
            .submitJudgment(frameId: frameId, choiceId: choiceId)
        case .respondGate(let gateId, let verdict):
            .respondGate(gateId: gateId, verdict: verdict)
        case .submitReview(let frameId, let stance):
            .submitReview(frameId: frameId, stance: stance)
        case .requestContext(let level):
            .requestContext(level: level)
        }
    }
}
