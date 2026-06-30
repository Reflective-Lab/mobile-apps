package se.reflective.quorum.consent

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import se.reflective.quorum.capture.FieldSignalDraft
import uniffi.quorum_ffi.ConsentDecision

/** Editable draft review with typed consent actions (M5.5). */
@Composable
fun ConsentReviewScreen(
    draft: FieldSignalDraft,
    onOutcome: (ConsentOutcome) -> Unit,
    modifier: Modifier = Modifier,
) {
    var rawCapture by remember(draft.draftId) { mutableStateOf(draft.rawCapture) }
    var summary by remember(draft.draftId) { mutableStateOf(draft.summary) }
    var latentNeed by remember(draft.draftId) { mutableStateOf(draft.latentNeed) }
    var contradiction by remember(draft.draftId) { mutableStateOf(draft.contradiction) }

    val reviewed = draft.withEdits(rawCapture, summary, latentNeed, contradiction)
    val decision = if (
        rawCapture != draft.rawCapture ||
        summary != draft.summary ||
        latentNeed != draft.latentNeed ||
        contradiction != draft.contradiction
    ) {
        ConsentDecision.EDITED_AND_ACCEPTED
    } else {
        ConsentDecision.ACCEPTED
    }

    Card(modifier = modifier.fillMaxWidth()) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            Text("Review draft", style = MaterialTheme.typography.titleMedium)
            Text("Confidence: ${"%.2f".format(draft.confidence.value)}")
            OutlinedTextField(
                value = summary,
                onValueChange = { summary = it },
                label = { Text("Summary") },
                modifier = Modifier.fillMaxWidth(),
                minLines = 2,
            )
            OutlinedTextField(
                value = rawCapture,
                onValueChange = { rawCapture = it },
                label = { Text("Raw capture") },
                modifier = Modifier.fillMaxWidth(),
                minLines = 3,
            )
            OutlinedTextField(
                value = latentNeed,
                onValueChange = { latentNeed = it },
                label = { Text("Latent need") },
                modifier = Modifier.fillMaxWidth(),
            )
            OutlinedTextField(
                value = contradiction,
                onValueChange = { contradiction = it },
                label = { Text("Contradiction / tension") },
                modifier = Modifier.fillMaxWidth(),
            )

            Button(onClick = { onOutcome(ConsentOutcome.Accept(reviewed, decision)) }) {
                Text("Accept and queue")
            }
            OutlinedButton(onClick = { onOutcome(ConsentOutcome.SavePrivate(reviewed)) }) {
                Text("Save private (local only)")
            }
            OutlinedButton(onClick = { onOutcome(ConsentOutcome.Reject) }) {
                Text("Reject")
            }
            OutlinedButton(onClick = { onOutcome(ConsentOutcome.Discard) }) {
                Text("Discard")
            }
        }
    }
}

sealed interface ConsentOutcome {
    data class Accept(val draft: FieldSignalDraft, val decision: ConsentDecision) : ConsentOutcome
    data class SavePrivate(val draft: FieldSignalDraft) : ConsentOutcome
    data object Reject : ConsentOutcome
    data object Discard : ConsentOutcome
}

private fun FieldSignalDraft.withEdits(
    rawCapture: String,
    summary: String,
    latentNeed: String,
    contradiction: String,
): FieldSignalDraft = copy(
    rawCapture = rawCapture,
    summary = summary,
    latentNeed = latentNeed,
    contradiction = contradiction,
)
