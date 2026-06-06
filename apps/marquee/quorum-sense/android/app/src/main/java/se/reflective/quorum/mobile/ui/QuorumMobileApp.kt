package se.reflective.quorum.mobile.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SegmentedButton
import androidx.compose.material3.SegmentedButtonDefaults
import androidx.compose.material3.SingleChoiceSegmentedButtonRow
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch
import se.reflective.quorum.mobile.capture.FieldSignalDraft
import se.reflective.quorum.mobile.capture.QuorumAppendEvent
import se.reflective.quorum.mobile.capture.SignalModality
import se.reflective.quorum.mobile.corebridge.PreviewQuorumCoreBridge
import se.reflective.quorum.mobile.corebridge.QuorumCoreBridge

@Composable
fun QuorumMobileApp() {
    MaterialTheme {
        SignalCaptureScreen()
    }
}

@Composable
fun SignalCaptureScreen(
    bridge: QuorumCoreBridge = remember { PreviewQuorumCoreBridge() },
) {
    val scope = rememberCoroutineScope()
    val inquiryThreadId = "inq_mobile_launch_risks"

    var modality by remember { mutableStateOf(SignalModality.VOICE_TRANSCRIPT) }
    var rawCapture by remember {
        mutableStateOf("The sales team says rollout is fine, but support is seeing confusion in every pilot.")
    }
    var draft by remember { mutableStateOf<FieldSignalDraft?>(null) }
    var appendEvent by remember { mutableStateOf<QuorumAppendEvent?>(null) }

    Scaffold { padding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp),
        ) {
            Text("Quorum", style = MaterialTheme.typography.headlineMedium)

            ModalitySelector(
                selected = modality,
                onSelected = { modality = it },
            )

            OutlinedTextField(
                value = rawCapture,
                onValueChange = { rawCapture = it },
                label = { Text("Captured signal") },
                modifier = Modifier.fillMaxWidth(),
                minLines = 5,
            )

            Button(
                onClick = {
                    scope.launch {
                        appendEvent = null
                        draft = bridge.draftFieldSignal(
                            inquiryThreadId = inquiryThreadId,
                            modality = modality,
                            rawCapture = rawCapture,
                        )
                    }
                },
            ) {
                Text("Create Draft")
            }

            draft?.let { currentDraft ->
                DraftCard(
                    draft = currentDraft,
                    onConsent = {
                        scope.launch {
                            appendEvent = bridge.appendConsentedSignal(currentDraft)
                        }
                    },
                )
            }

            appendEvent?.let { event ->
                Card(modifier = Modifier.fillMaxWidth()) {
                    Column(
                        modifier = Modifier.padding(16.dp),
                        verticalArrangement = Arrangement.spacedBy(8.dp),
                    ) {
                        Text("Queued Event", style = MaterialTheme.typography.titleMedium)
                        Text(event.eventType)
                        Text(event.syncState)
                    }
                }
            }
        }
    }
}

@Composable
private fun ModalitySelector(
    selected: SignalModality,
    onSelected: (SignalModality) -> Unit,
) {
    SingleChoiceSegmentedButtonRow {
        SignalModality.entries.forEachIndexed { index, item ->
            SegmentedButton(
                selected = selected == item,
                onClick = { onSelected(item) },
                shape = SegmentedButtonDefaults.itemShape(
                    index = index,
                    count = SignalModality.entries.size,
                ),
            ) {
                Text(item.label)
            }
        }
    }
}

@Composable
private fun DraftCard(
    draft: FieldSignalDraft,
    onConsent: () -> Unit,
) {
    Card(modifier = Modifier.fillMaxWidth()) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            Text("Draft", style = MaterialTheme.typography.titleMedium)
            Text(draft.summary)
            Text("Consent: ${draft.consentState}")
            Text("Confidence: ${"%.2f".format(draft.confidence)}")
            Text(draft.contradiction, style = MaterialTheme.typography.bodySmall)

            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                Button(onClick = onConsent) {
                    Text("Consent And Queue")
                }
            }
        }
    }
}

