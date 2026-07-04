package se.reflective.quorum.corebridge

import se.reflective.quorum.director.BlockingState
import se.reflective.quorum.director.Choice
import se.reflective.quorum.director.ContextLevel
import se.reflective.quorum.director.DirectorFrame
import se.reflective.quorum.director.DirectorIntent
import se.reflective.quorum.director.DirectorPrompt
import se.reflective.quorum.director.DirectorSnapshot
import se.reflective.quorum.director.GatePrompt
import se.reflective.quorum.director.GateVerdict
import se.reflective.quorum.director.JudgmentPrompt
import se.reflective.quorum.director.NowTask
import se.reflective.quorum.director.PresenceHint
import se.reflective.quorum.director.PrimaryAction
import se.reflective.quorum.director.ReviewPrompt
import se.reflective.quorum.director.ReviewStance
import se.reflective.quorum.director.SecondaryAction
import se.reflective.quorum.director.WaitingFor
import uniffi.quorum_ffi.BlockingState as FfiBlockingState
import uniffi.quorum_ffi.ContextLevel as FfiContextLevel
import uniffi.quorum_ffi.FfiChoice
import uniffi.quorum_ffi.FfiDirectorFrame
import uniffi.quorum_ffi.FfiDirectorIntent
import uniffi.quorum_ffi.FfiDirectorPrompt
import uniffi.quorum_ffi.FfiDirectorSnapshot
import uniffi.quorum_ffi.FfiGatePrompt
import uniffi.quorum_ffi.FfiJudgmentPrompt
import uniffi.quorum_ffi.FfiNowTask
import uniffi.quorum_ffi.FfiPresenceHint
import uniffi.quorum_ffi.FfiPrimaryAction
import uniffi.quorum_ffi.FfiReviewPrompt
import uniffi.quorum_ffi.FfiSecondaryAction
import uniffi.quorum_ffi.FfiWaitingFor
import uniffi.quorum_ffi.GateVerdict as FfiGateVerdict
import uniffi.quorum_ffi.ReviewStance as FfiReviewStance

/** Maps UniFFI director wire enums to Kotlin domain types in `director/DirectorModels.kt`. */
internal object DirectorBridgeMapping {
    fun toDomain(ffi: FfiDirectorSnapshot): DirectorSnapshot =
        DirectorSnapshot(
            version = ffi.version.toLong(),
            frame = toDomainFrame(ffi.frame),
        )

    fun toFfi(intent: DirectorIntent): FfiDirectorIntent =
        when (intent) {
            is DirectorIntent.OpenTask ->
                FfiDirectorIntent.OpenTask(frameId = intent.frameId)
            is DirectorIntent.SubmitJudgment ->
                FfiDirectorIntent.SubmitJudgment(
                    frameId = intent.frameId,
                    choiceId = intent.choiceId,
                )
            is DirectorIntent.RespondGate ->
                FfiDirectorIntent.RespondGate(
                    gateId = intent.gateId,
                    verdict = toFfi(intent.verdict),
                )
            is DirectorIntent.SubmitReview ->
                FfiDirectorIntent.SubmitReview(
                    frameId = intent.frameId,
                    stance = toFfi(intent.stance),
                )
            is DirectorIntent.RequestContext ->
                FfiDirectorIntent.RequestContext(level = toFfi(intent.level))
        }

    private fun toDomainFrame(ffi: FfiDirectorFrame): DirectorFrame =
        DirectorFrame(
            frameId = ffi.frameId,
            title = ffi.title,
            subtitle = ffi.subtitle,
            now = ffi.now?.let(::toDomainNow),
            waitingFor = toDomainWaitingFor(ffi.waitingFor),
            primary = toDomainPrimary(ffi.primary),
            secondary = ffi.secondary.map(::toDomainSecondary),
            prompt = ffi.prompt?.let(::toDomainPrompt),
            presence = ffi.presence.map(::toDomainPresence),
            contextTrail = ffi.contextTrail.map(::toDomainContextLevel),
            blocking = toDomainBlocking(ffi.blocking),
        )

    private fun toDomainNow(ffi: FfiNowTask): NowTask =
        NowTask(
            objective = ffi.objective,
            neededFromUser = ffi.neededFromUser,
            estimatedMinutes = ffi.estimatedMinutes?.toInt(),
        )

    private fun toDomainWaitingFor(ffi: FfiWaitingFor): WaitingFor =
        when (ffi) {
            is FfiWaitingFor.Nobody -> WaitingFor.Nobody
            is FfiWaitingFor.Participants ->
                WaitingFor.Participants(ffi.actorLabels)
            is FfiWaitingFor.Server -> WaitingFor.Server
        }

    private fun toDomainPrimary(ffi: FfiPrimaryAction): PrimaryAction =
        PrimaryAction(label = ffi.label, intent = toDomainIntent(ffi.intent))

    private fun toDomainSecondary(ffi: FfiSecondaryAction): SecondaryAction =
        SecondaryAction(label = ffi.label, intent = toDomainIntent(ffi.intent))

    private fun toDomainPrompt(ffi: FfiDirectorPrompt): DirectorPrompt =
        when (ffi) {
            is FfiDirectorPrompt.Judgment ->
                DirectorPrompt.Judgment(
                    JudgmentPrompt(
                        question = ffi.judgment.question,
                        body = ffi.judgment.body,
                        choices = ffi.judgment.choices.map(::toDomainChoice),
                    ),
                )
            is FfiDirectorPrompt.Gate ->
                DirectorPrompt.Gate(
                    GatePrompt(
                        gateId = ffi.gate.gateId,
                        reason = ffi.gate.reason,
                        consequence = ffi.gate.consequence,
                        deadlineMs = ffi.gate.deadlineMs?.toLong(),
                    ),
                )
            is FfiDirectorPrompt.Review ->
                DirectorPrompt.Review(
                    ReviewPrompt(
                        title = ffi.review.title,
                        primaryEvidence = ffi.review.primaryEvidence,
                    ),
                )
        }

    private fun toDomainChoice(ffi: FfiChoice): Choice =
        Choice(choiceId = ffi.choiceId, label = ffi.label)

    private fun toDomainPresence(ffi: FfiPresenceHint): PresenceHint =
        PresenceHint(actorLabel = ffi.actorLabel, status = ffi.status)

    private fun toDomainIntent(ffi: FfiDirectorIntent): DirectorIntent =
        when (ffi) {
            is FfiDirectorIntent.OpenTask ->
                DirectorIntent.OpenTask(ffi.frameId)
            is FfiDirectorIntent.SubmitJudgment ->
                DirectorIntent.SubmitJudgment(
                    frameId = ffi.frameId,
                    choiceId = ffi.choiceId,
                )
            is FfiDirectorIntent.RespondGate ->
                DirectorIntent.RespondGate(
                    gateId = ffi.gateId,
                    verdict = toDomain(ffi.verdict),
                )
            is FfiDirectorIntent.SubmitReview ->
                DirectorIntent.SubmitReview(
                    frameId = ffi.frameId,
                    stance = toDomain(ffi.stance),
                )
            is FfiDirectorIntent.RequestContext ->
                DirectorIntent.RequestContext(toDomainContextLevel(ffi.level))
        }

    private fun toDomain(ffi: FfiGateVerdict): GateVerdict =
        when (ffi) {
            FfiGateVerdict.APPROVE -> GateVerdict.APPROVE
            FfiGateVerdict.REJECT -> GateVerdict.REJECT
        }

    private fun toFfi(verdict: GateVerdict): FfiGateVerdict =
        when (verdict) {
            GateVerdict.APPROVE -> FfiGateVerdict.APPROVE
            GateVerdict.REJECT -> FfiGateVerdict.REJECT
        }

    private fun toDomain(ffi: FfiReviewStance): ReviewStance =
        when (ffi) {
            FfiReviewStance.AGREE -> ReviewStance.AGREE
            FfiReviewStance.DISAGREE -> ReviewStance.DISAGREE
            FfiReviewStance.NEED_MORE_CONTEXT -> ReviewStance.NEED_MORE_CONTEXT
        }

    private fun toFfi(stance: ReviewStance): FfiReviewStance =
        when (stance) {
            ReviewStance.AGREE -> FfiReviewStance.AGREE
            ReviewStance.DISAGREE -> FfiReviewStance.DISAGREE
            ReviewStance.NEED_MORE_CONTEXT -> FfiReviewStance.NEED_MORE_CONTEXT
        }

    private fun toDomainContextLevel(ffi: FfiContextLevel): ContextLevel =
        when (ffi) {
            FfiContextLevel.TASK -> ContextLevel.TASK
            FfiContextLevel.LOCAL_CONTEXT -> ContextLevel.LOCAL_CONTEXT
            FfiContextLevel.SESSION -> ContextLevel.SESSION
            FfiContextLevel.FORMATION -> ContextLevel.FORMATION
            FfiContextLevel.ORGANIZATION -> ContextLevel.ORGANIZATION
            FfiContextLevel.EVERYTHING -> ContextLevel.EVERYTHING
        }

    private fun toFfi(level: ContextLevel): FfiContextLevel =
        when (level) {
            ContextLevel.TASK -> FfiContextLevel.TASK
            ContextLevel.LOCAL_CONTEXT -> FfiContextLevel.LOCAL_CONTEXT
            ContextLevel.SESSION -> FfiContextLevel.SESSION
            ContextLevel.FORMATION -> FfiContextLevel.FORMATION
            ContextLevel.ORGANIZATION -> FfiContextLevel.ORGANIZATION
            ContextLevel.EVERYTHING -> FfiContextLevel.EVERYTHING
        }

    private fun toDomainBlocking(ffi: FfiBlockingState): BlockingState =
        when (ffi) {
            FfiBlockingState.NOT_BLOCKING -> BlockingState.NOT_BLOCKING
            FfiBlockingState.BLOCKS_FORMATION -> BlockingState.BLOCKS_FORMATION
            FfiBlockingState.BLOCKS_SESSION -> BlockingState.BLOCKS_SESSION
        }
}
