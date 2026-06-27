import Foundation

/// Maps UniFFI director wire DTOs to Swift domain types in `DirectorModels.swift`.
/// Flat enums (`GateVerdict`, `ContextLevel`, …) are generated in `QuorumFFI.swift`
/// and cross unchanged; tagged shapes (`WaitingFor`, `DirectorPrompt`, `DirectorIntent`)
/// are mapped here.
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
            FfiDirectorIntent(
                kind: .openTask,
                frameId: frameId,
                choiceId: nil,
                gateId: nil,
                gateVerdict: nil,
                reviewStance: nil,
                contextLevel: nil
            )
        case .submitJudgment(let frameId, let choiceId):
            FfiDirectorIntent(
                kind: .submitJudgment,
                frameId: frameId,
                choiceId: choiceId,
                gateId: nil,
                gateVerdict: nil,
                reviewStance: nil,
                contextLevel: nil
            )
        case .respondGate(let gateId, let verdict):
            FfiDirectorIntent(
                kind: .respondGate,
                frameId: nil,
                choiceId: nil,
                gateId: gateId,
                gateVerdict: verdict,
                reviewStance: nil,
                contextLevel: nil
            )
        case .submitReview(let frameId, let stance):
            FfiDirectorIntent(
                kind: .submitReview,
                frameId: frameId,
                choiceId: nil,
                gateId: nil,
                gateVerdict: nil,
                reviewStance: stance,
                contextLevel: nil
            )
        case .requestContext(let level):
            FfiDirectorIntent(
                kind: .requestContext,
                frameId: nil,
                choiceId: nil,
                gateId: nil,
                gateVerdict: nil,
                reviewStance: nil,
                contextLevel: level
            )
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
            prompt: ffi.prompt.flatMap(domainPrompt),
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
        switch ffi.kind {
        case .nobody:
            .nobody
        case .participants:
            .participants(actorLabels: ffi.actorLabels ?? [])
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

    private static func domainPrompt(_ ffi: FfiDirectorPrompt) -> DirectorPrompt? {
        switch ffi.kind {
        case .judgment:
            guard let judgment = ffi.judgment else { return nil }
            return .judgment(
                JudgmentPrompt(
                    question: judgment.question,
                    body: judgment.body,
                    choices: judgment.choices.map(domainChoice)
                )
            )
        case .gate:
            guard let gate = ffi.gate else { return nil }
            return .gate(
                GatePrompt(
                    gateId: gate.gateId,
                    reason: gate.reason,
                    consequence: gate.consequence,
                    deadlineMs: gate.deadlineMs
                )
            )
        case .review:
            guard let review = ffi.review else { return nil }
            return .review(
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
        switch ffi.kind {
        case .openTask:
            .openTask(frameId: ffi.frameId ?? "")
        case .submitJudgment:
            .submitJudgment(
                frameId: ffi.frameId ?? "",
                choiceId: ffi.choiceId ?? ""
            )
        case .respondGate:
            .respondGate(
                gateId: ffi.gateId ?? "",
                verdict: ffi.gateVerdict ?? .reject
            )
        case .submitReview:
            .submitReview(
                frameId: ffi.frameId ?? "",
                stance: ffi.reviewStance ?? .needMoreContext
            )
        case .requestContext:
            .requestContext(level: ffi.contextLevel ?? .task)
        }
    }
}
