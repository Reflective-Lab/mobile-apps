package se.reflective.quorum.director

data class DirectorSnapshot(
    val version: Long,
    val frame: DirectorFrame,
)

data class DirectorFrame(
    val frameId: String,
    val title: String,
    val subtitle: String?,
    val now: NowTask?,
    val waitingFor: WaitingFor,
    val primary: PrimaryAction,
    val secondary: List<SecondaryAction>,
    val prompt: DirectorPrompt?,
    val presence: List<PresenceHint>,
    val contextTrail: List<ContextLevel>,
    val blocking: BlockingState,
)

data class NowTask(
    val objective: String,
    val neededFromUser: String?,
    val estimatedMinutes: Int?,
)

sealed interface WaitingFor {
    data object Nobody : WaitingFor
    data class Participants(val actorLabels: List<String>) : WaitingFor
    data object Server : WaitingFor
}

data class PrimaryAction(
    val label: String,
    val intent: DirectorIntent,
)

data class SecondaryAction(
    val label: String,
    val intent: DirectorIntent,
)

sealed interface DirectorPrompt {
    data class Judgment(val prompt: JudgmentPrompt) : DirectorPrompt
    data class Gate(val prompt: GatePrompt) : DirectorPrompt
    data class Review(val prompt: ReviewPrompt) : DirectorPrompt
}

data class JudgmentPrompt(
    val question: String,
    val body: String,
    val choices: List<Choice>,
)

data class GatePrompt(
    val gateId: String,
    val reason: String,
    val consequence: String,
    val deadlineMs: Long?,
)

data class ReviewPrompt(
    val title: String,
    val primaryEvidence: String,
)

data class Choice(
    val choiceId: String,
    val label: String,
)

sealed interface DirectorIntent {
    data class OpenTask(val frameId: String) : DirectorIntent
    data class SubmitJudgment(val frameId: String, val choiceId: String) : DirectorIntent
    data class RespondGate(val gateId: String, val verdict: GateVerdict) : DirectorIntent
    data class SubmitReview(val frameId: String, val stance: ReviewStance) : DirectorIntent
    data class RequestContext(val level: ContextLevel) : DirectorIntent
}

enum class GateVerdict {
    APPROVE,
    REJECT,
}

enum class ReviewStance {
    AGREE,
    DISAGREE,
    NEED_MORE_CONTEXT,
}

enum class ContextLevel(val label: String) {
    TASK("Task"),
    LOCAL_CONTEXT("Local Context"),
    SESSION("Session"),
    FORMATION("Formation"),
    ORGANIZATION("Organization"),
    EVERYTHING("Everything"),
}

enum class BlockingState {
    NOT_BLOCKING,
    BLOCKS_FORMATION,
    BLOCKS_SESSION,
}

data class PresenceHint(
    val actorLabel: String,
    val status: String,
)

object DirectorFixture {
    private const val FRAME_ID = "director-frame:session:procurement-security-review:1844"

    val quorumDecisionCheckpoint = DirectorSnapshot(
        version = 1844L,
        frame = DirectorFrame(
            frameId = FRAME_ID,
            title = "The procurement formation has reached a decision checkpoint.",
            subtitle = "Good morning Kenneth.",
            now = NowTask(
                objective = "Evaluate Vendor X's security claims",
                neededFromUser = "Review the encryption section",
                estimatedMinutes = 4,
            ),
            waitingFor = WaitingFor.Nobody,
            primary = PrimaryAction(
                label = "Open Review",
                intent = DirectorIntent.OpenTask(FRAME_ID),
            ),
            secondary = listOf(
                SecondaryAction(
                    label = "Reject",
                    intent = DirectorIntent.RespondGate(
                        gateId = "gate:procurement-security-approval",
                        verdict = GateVerdict.REJECT,
                    ),
                ),
            ),
            prompt = DirectorPrompt.Judgment(
                JudgmentPrompt(
                    question = "Does Vendor X's encryption section support the procurement claim?",
                    body = "Vendor X states that customer records are encrypted at rest with AES-256 and in transit with TLS 1.3. The claim does not state key ownership or customer-managed key support.",
                    choices = listOf(
                        Choice("choice:yes", "Yes"),
                        Choice("choice:no", "No"),
                        Choice("choice:unsure", "Unsure"),
                    ),
                ),
            ),
            presence = listOf(
                PresenceHint("Maria", "waiting_on_you"),
                PresenceHint("Legal", "available"),
            ),
            contextTrail = listOf(
                ContextLevel.TASK,
                ContextLevel.SESSION,
                ContextLevel.FORMATION,
                ContextLevel.ORGANIZATION,
            ),
            blocking = BlockingState.BLOCKS_FORMATION,
        ),
    )
}
