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
import uniffi.quorum_ffi.DirectorIntentKind
import uniffi.quorum_ffi.DirectorPromptKind
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
import uniffi.quorum_ffi.WaitingForKind

/** Maps UniFFI director wire DTOs to Kotlin domain types in `director/DirectorModels.kt`. */
internal object DirectorBridgeMapping {
    fun toDomain(ffi: FfiDirectorSnapshot): DirectorSnapshot =
        DirectorSnapshot(
            version = ffi.version,
            frame = toDomainFrame(ffi.frame),
        )

    fun toFfi(intent: DirectorIntent): FfiDirectorIntent =
        when (intent) {
            is DirectorIntent.OpenTask ->
                FfiDirectorIntent(
                    kind = DirectorIntentKind.OPEN_TASK,
                    frameId = intent.frameId,
                    choiceId = null,
                    gateId = null,
                    gateVerdict = null,
                    reviewStance = null,
                    contextLevel = null,
                )
            is DirectorIntent.SubmitJudgment ->
                FfiDirectorIntent(
                    kind = DirectorIntentKind.SUBMIT_JUDGMENT,
                    frameId = intent.frameId,
                    choiceId = intent.choiceId,
                    gateId = null,
                    gateVerdict = null,
                    reviewStance = null,
                    contextLevel = null,
                )
            is DirectorIntent.RespondGate ->
                FfiDirectorIntent(
                    kind = DirectorIntentKind.RESPOND_GATE,
                    frameId = null,
                    choiceId = null,
                    gateId = intent.gateId,
                    gateVerdict = toFfi(intent.verdict),
                    reviewStance = null,
                    contextLevel = null,
                )
            is DirectorIntent.SubmitReview ->
                FfiDirectorIntent(
                    kind = DirectorIntentKind.SUBMIT_REVIEW,
                    frameId = intent.frameId,
                    choiceId = null,
                    gateId = null,
                    gateVerdict = null,
                    reviewStance = toFfi(intent.stance),
                    contextLevel = null,
                )
            is DirectorIntent.RequestContext ->
                FfiDirectorIntent(
                    kind = DirectorIntentKind.REQUEST_CONTEXT,
                    frameId = null,
                    choiceId = null,
                    gateId = null,
                    gateVerdict = null,
                    reviewStance = null,
                    contextLevel = toFfi(intent.level),
                )
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
        when (ffi.kind) {
            WaitingForKind.NOBODY -> WaitingFor.Nobody
            WaitingForKind.PARTICIPANTS ->
                WaitingFor.Participants(ffi.actorLabels.orEmpty())
            WaitingForKind.SERVER -> WaitingFor.Server
        }

    private fun toDomainPrimary(ffi: FfiPrimaryAction): PrimaryAction =
        PrimaryAction(label = ffi.label, intent = toDomainIntent(ffi.intent))

    private fun toDomainSecondary(ffi: FfiSecondaryAction): SecondaryAction =
        SecondaryAction(label = ffi.label, intent = toDomainIntent(ffi.intent))

    private fun toDomainPrompt(ffi: FfiDirectorPrompt): DirectorPrompt? =
        when (ffi.kind) {
            DirectorPromptKind.JUDGMENT ->
                ffi.judgment?.let { judgment ->
                    DirectorPrompt.Judgment(
                        JudgmentPrompt(
                            question = judgment.question,
                            body = judgment.body,
                            choices = judgment.choices.map(::toDomainChoice),
                        ),
                    )
                }
            DirectorPromptKind.GATE ->
                ffi.gate?.let { gate ->
                    DirectorPrompt.Gate(
                        GatePrompt(
                            gateId = gate.gateId,
                            reason = gate.reason,
                            consequence = gate.consequence,
                            deadlineMs = gate.deadlineMs,
                        ),
                    )
                }
            DirectorPromptKind.REVIEW ->
                ffi.review?.let { review ->
                    DirectorPrompt.Review(
                        ReviewPrompt(
                            title = review.title,
                            primaryEvidence = review.primaryEvidence,
                        ),
                    )
                }
        }

    private fun toDomainChoice(ffi: FfiChoice): Choice =
        Choice(choiceId = ffi.choiceId, label = ffi.label)

    private fun toDomainPresence(ffi: FfiPresenceHint): PresenceHint =
        PresenceHint(actorLabel = ffi.actorLabel, status = ffi.status)

    private fun toDomainIntent(ffi: FfiDirectorIntent): DirectorIntent =
        when (ffi.kind) {
            DirectorIntentKind.OPEN_TASK ->
                DirectorIntent.OpenTask(ffi.frameId.orEmpty())
            DirectorIntentKind.SUBMIT_JUDGMENT ->
                DirectorIntent.SubmitJudgment(
                    frameId = ffi.frameId.orEmpty(),
                    choiceId = ffi.choiceId.orEmpty(),
                )
            DirectorIntentKind.RESPOND_GATE ->
                DirectorIntent.RespondGate(
                    gateId = ffi.gateId.orEmpty(),
                    verdict = toDomain(ffi.gateVerdict ?: FfiGateVerdict.REJECT),
                )
            DirectorIntentKind.SUBMIT_REVIEW ->
                DirectorIntent.SubmitReview(
                    frameId = ffi.frameId.orEmpty(),
                    stance = toDomain(ffi.reviewStance ?: FfiReviewStance.NEED_MORE_CONTEXT),
                )
            DirectorIntentKind.REQUEST_CONTEXT ->
                DirectorIntent.RequestContext(
                    toDomainContextLevel(ffi.contextLevel ?: FfiContextLevel.TASK),
                )
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
