package se.reflective.quorum.ui.director

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.unit.dp
import se.reflective.quorum.director.BlockingState
import se.reflective.quorum.director.Choice
import se.reflective.quorum.director.ContextLevel
import se.reflective.quorum.director.DirectorFixture
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
import se.reflective.quorum.director.SecondaryAction
import se.reflective.quorum.director.WaitingFor

@Composable
@OptIn(ExperimentalLayoutApi::class)
fun DirectorNowScreen(
    snapshot: DirectorSnapshot = DirectorFixture.quorumDecisionCheckpoint,
    onIntent: (DirectorIntent) -> Unit = {},
) {
    Column(
        modifier = Modifier
            .verticalScroll(rememberScrollState())
            .padding(24.dp),
        verticalArrangement = Arrangement.spacedBy(20.dp),
    ) {
        Text("Snapshot ${snapshot.version}", style = MaterialTheme.typography.labelSmall)
        snapshot.frame.subtitle?.let {
            Text(it, style = MaterialTheme.typography.titleMedium)
        }
        Text(snapshot.frame.title, style = MaterialTheme.typography.headlineLarge)

        snapshot.frame.now?.let { now ->
            DirectorTaskCard(
                now = now,
                waitingFor = snapshot.frame.waitingFor,
                blocking = snapshot.frame.blocking,
                primary = snapshot.frame.primary,
                onIntent = onIntent,
            )
        }

        snapshot.frame.prompt?.let { prompt ->
            when (prompt) {
                is DirectorPrompt.Judgment -> JudgmentPromptCard(
                    prompt = prompt.prompt,
                    frameId = snapshot.frame.frameId,
                    onIntent = onIntent,
                )
                is DirectorPrompt.Gate -> GatePromptCard(prompt = prompt.prompt, onIntent = onIntent)
                is DirectorPrompt.Review -> Unit
            }
        }

        snapshot.frame.secondary.forEach { action ->
            OutlinedButton(onClick = { onIntent(action.intent) }) {
                Text(action.label)
            }
        }

        ContextTrail(levels = snapshot.frame.contextTrail, onIntent = onIntent)
        PresenceStrip(presence = snapshot.frame.presence)
    }
}

@Composable
private fun DirectorTaskCard(
    now: NowTask,
    waitingFor: WaitingFor,
    blocking: BlockingState,
    primary: PrimaryAction,
    onIntent: (DirectorIntent) -> Unit,
) {
    Card(modifier = Modifier.fillMaxWidth(), shape = RoundedCornerShape(28.dp)) {
        Column(modifier = Modifier.padding(20.dp), verticalArrangement = Arrangement.spacedBy(16.dp)) {
            Text("Current objective", style = MaterialTheme.typography.labelSmall)
            Text(now.objective, style = MaterialTheme.typography.titleLarge)
            now.neededFromUser?.let {
                Text("Needed from you", style = MaterialTheme.typography.labelSmall)
                Text(it, style = MaterialTheme.typography.headlineSmall)
            }
            if (blocking != BlockingState.NOT_BLOCKING) {
                Text("Blocked", color = MaterialTheme.colorScheme.error)
            }
            Button(onClick = { onIntent(primary.intent) }, modifier = Modifier.fillMaxWidth()) {
                Text(primary.label)
            }
        }
    }
}

@Composable
@OptIn(ExperimentalLayoutApi::class)
private fun JudgmentPromptCard(
    prompt: JudgmentPrompt,
    frameId: String,
    onIntent: (DirectorIntent) -> Unit,
) {
    DirectorPanel {
        Text("Focused judgment", style = MaterialTheme.typography.labelSmall)
        Text(prompt.question, style = MaterialTheme.typography.titleMedium)
        Text(prompt.body)
        FlowRow(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            prompt.choices.forEach { choice: Choice ->
                OutlinedButton(onClick = {
                    onIntent(DirectorIntent.SubmitJudgment(frameId, choice.choiceId))
                }) {
                    Text(choice.label)
                }
            }
        }
    }
}

@Composable
private fun GatePromptCard(prompt: GatePrompt, onIntent: (DirectorIntent) -> Unit) {
    DirectorPanel {
        Text("Gate", style = MaterialTheme.typography.labelSmall)
        Text(prompt.reason, style = MaterialTheme.typography.titleMedium)
        Text(prompt.consequence)
        Row(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            Button(onClick = {
                onIntent(DirectorIntent.RespondGate(prompt.gateId, GateVerdict.APPROVE))
            }) { Text("Approve") }
            OutlinedButton(onClick = {
                onIntent(DirectorIntent.RespondGate(prompt.gateId, GateVerdict.REJECT))
            }) { Text("Reject") }
        }
    }
}

@Composable
@OptIn(ExperimentalLayoutApi::class)
private fun ContextTrail(levels: List<ContextLevel>, onIntent: (DirectorIntent) -> Unit) {
    FlowRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        levels.forEach { level ->
            OutlinedButton(onClick = { onIntent(DirectorIntent.RequestContext(level)) }) {
                Text(level.label)
            }
        }
    }
}

@Composable
private fun PresenceStrip(presence: List<PresenceHint>) {
    Row(verticalAlignment = Alignment.CenterVertically) {
        presence.forEach { person ->
            Box(
                modifier = Modifier.size(32.dp).clip(CircleShape).background(MaterialTheme.colorScheme.primary),
                contentAlignment = Alignment.Center,
            ) {
                Text(person.actorLabel.take(1), color = MaterialTheme.colorScheme.onPrimary)
            }
        }
        Spacer(modifier = Modifier.width(14.dp))
        Text("Presence", style = MaterialTheme.typography.labelSmall)
    }
}

@Composable
private fun DirectorPanel(content: @Composable ColumnScope.() -> Unit) {
    Card(modifier = Modifier.fillMaxWidth(), shape = RoundedCornerShape(22.dp)) {
        Column(modifier = Modifier.padding(18.dp), verticalArrangement = Arrangement.spacedBy(14.dp), content = content)
    }
}
