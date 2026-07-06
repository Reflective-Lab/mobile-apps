package se.reflective.quorum.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SegmentedButton
import androidx.compose.material3.SegmentedButtonDefaults
import androidx.compose.material3.SingleChoiceSegmentedButtonRow
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import se.reflective.quorum.capture.CaptureSessionContext
import se.reflective.quorum.capture.FieldSignalDraft
import se.reflective.quorum.capture.QuorumAppendEvent
import se.reflective.quorum.capture.label
import se.reflective.quorum.consent.ConsentOutcome
import se.reflective.quorum.consent.ConsentReviewScreen
import se.reflective.quorum.corebridge.PreviewQuorumCoreBridge
import se.reflective.quorum.corebridge.QuorumCoreBridge
import se.reflective.quorum.director.DirectorSnapshot
import se.reflective.quorum.platformai.PlatformSignalExtractor
import se.reflective.quorum.queue.PersistedQueueRecordSummary
import se.reflective.quorum.queue.QueueBackgroundSubmit
import se.reflective.quorum.queue.label
import se.reflective.quorum.queue.permitsQueue
import se.reflective.quorum.ui.director.DirectorNowScreen
import se.reflective.quorum.ui.theme.QuorumTheme
import uniffi.quorum_ffi.SignalModality

@Composable
fun QuorumMobileApp(bridge: QuorumCoreBridge = PreviewQuorumCoreBridge()) {
    QuorumTheme {
        var showCapture by remember { mutableStateOf(false) }

        if (showCapture) {
            SignalCaptureScreen(
                bridge = bridge,
                onBack = { showCapture = false },
            )
        } else {
            DirectorRootScreen(
                bridge = bridge,
                onOpenSignalCapture = { showCapture = true },
            )
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun DirectorRootScreen(
    bridge: QuorumCoreBridge,
    onOpenSignalCapture: () -> Unit,
) {
    val scope = rememberCoroutineScope()
    var snapshot by remember { mutableStateOf<DirectorSnapshot?>(null) }
    var snapshotSource by remember { mutableStateOf("…") }

    LaunchedEffect(bridge) {
        while (isActive) {
            snapshot = bridge.currentDirectorSnapshot()
            snapshotSource = bridge.directorSnapshotSource()
            val version = snapshot?.version ?: 0L
            val updated = bridge.waitDirectorUpdate(version.toULong(), 30_000u)
            if (!updated) {
                // Poll again after timeout so fixture-only builds still refresh on intent.
            }
        }
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Director") },
                actions = {
                    TextButton(onClick = onOpenSignalCapture) {
                        Text("Signal Capture")
                    }
                },
            )
        },
    ) { padding ->
        Column(modifier = Modifier.padding(padding)) {
            Text(
                text = snapshotSource,
                style = MaterialTheme.typography.labelSmall,
                modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp),
            )
            snapshot?.let { current ->
                DirectorNowScreen(
                    snapshot = current,
                    onIntent = { intent ->
                        scope.launch { bridge.submitDirectorIntent(intent) }
                    },
                )
            }
        }
    }
}

@Composable
fun SignalCaptureScreen(
    bridge: QuorumCoreBridge = remember { PreviewQuorumCoreBridge() },
    onBack: (() -> Unit)? = null,
    inquiryThreadId: String = CaptureSessionContext.inquiryThreadId(),
) {
    val scope = rememberCoroutineScope()
    val extractor = remember { PlatformSignalExtractor() }
    val appContext = LocalContext.current.applicationContext

    var modality by remember { mutableStateOf(SignalModality.TEXT) }
    var rawCapture by remember { mutableStateOf("") }
    var draft by remember { mutableStateOf<FieldSignalDraft?>(null) }
    var appendEvent by remember { mutableStateOf<QuorumAppendEvent?>(null) }
    var persistedRecord by remember { mutableStateOf<PersistedQueueRecordSummary?>(null) }
    var durableQueue by remember { mutableStateOf<List<PersistedQueueRecordSummary>>(emptyList()) }
    var privateDrafts by remember { mutableStateOf<List<FieldSignalDraft>>(emptyList()) }
    var normalizationNote by remember { mutableStateOf<String?>(null) }
    var statusMessage by remember { mutableStateOf<String?>(null) }
    var errorMessage by remember { mutableStateOf<String?>(null) }

    LaunchedEffect(bridge) {
        durableQueue = bridge.loadPersistedQueueRecords()
    }

    Scaffold { padding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .verticalScroll(rememberScrollState())
                .padding(padding)
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp),
        ) {
            if (onBack != null) {
                TextButton(onClick = onBack) {
                    Text("Back")
                }
            }

            Text("Signal Capture", style = MaterialTheme.typography.headlineMedium)

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

            normalizationNote?.let {
                Text(it, style = MaterialTheme.typography.bodySmall)
            }

            Button(
                onClick = {
                    scope.launch {
                        appendEvent = null
                        persistedRecord = null
                        statusMessage = null
                        errorMessage = null
                        draft = null
                        runCatching {
                            val normalized = extractor.normalizeCapture(modality, rawCapture)
                            normalizationNote = if (normalized.usedPlatformAI) {
                                "Normalized with platform AI."
                            } else {
                                "Normalized locally (platform AI unavailable)."
                            }
                            draft = bridge.draftFieldSignal(
                                inquiryThreadId = inquiryThreadId,
                                modality = normalized.input.modality,
                                rawCapture = normalized.input.rawCapture,
                            )
                        }.onFailure { error ->
                            errorMessage = error.message
                        }
                    }
                },
                enabled = rawCapture.trim().isNotEmpty(),
            ) {
                Text("Create Draft")
            }

            draft?.let { currentDraft ->
                ConsentReviewScreen(
                    draft = currentDraft,
                    onOutcome = { outcome ->
                        scope.launch {
                            statusMessage = null
                            errorMessage = null
                            when (outcome) {
                                is ConsentOutcome.Accept -> {
                                    if (!outcome.decision.permitsQueue()) {
                                        errorMessage = "Consent ${outcome.decision.label()} cannot enter the queue."
                                        return@launch
                                    }
                                    runCatching {
                                        appendEvent = bridge.appendConsentedSignal(outcome.draft)
                                        persistedRecord = bridge.persistConsentedSignalToQueue(
                                            outcome.draft,
                                            outcome.decision,
                                        )
                                        durableQueue = bridge.loadPersistedQueueRecords()
                                        draft = null
                                        statusMessage = "Queued with consent: ${outcome.decision.label()}."
                                    }.onSuccess {
                                        // Best-effort: the record is already durable and the
                                        // startup flush drains stranded queue entries, so a
                                        // scheduling failure must not surface as a failed consent.
                                        runCatching { QueueBackgroundSubmit.enqueue(appContext) }
                                    }.onFailure { error ->
                                        appendEvent = null
                                        persistedRecord = null
                                        errorMessage = error.message
                                    }
                                }
                                is ConsentOutcome.SavePrivate -> {
                                    privateDrafts = privateDrafts + outcome.draft
                                    draft = null
                                    statusMessage = "Saved private — not queued for sync."
                                }
                                ConsentOutcome.Reject -> {
                                    draft = null
                                    statusMessage = "Rejected — no sync event created."
                                }
                                ConsentOutcome.Discard -> {
                                    draft = null
                                    statusMessage = "Discarded."
                                }
                            }
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
                        Text(event.eventType.label)
                        Text(event.syncState.label)
                    }
                }
            }

            persistedRecord?.let { record ->
                Card(modifier = Modifier.fillMaxWidth()) {
                    Column(
                        modifier = Modifier.padding(16.dp),
                        verticalArrangement = Arrangement.spacedBy(8.dp),
                    ) {
                        Text("Durable Queue", style = MaterialTheme.typography.titleMedium)
                        Text(record.recordId)
                        Text("${record.queueState} · ${record.updatedAt}")
                    }
                }
            }

            if (privateDrafts.isNotEmpty()) {
                Text("Private drafts (this session)", style = MaterialTheme.typography.titleMedium)
                privateDrafts.forEach { item ->
                    Text(item.summary)
                }
            }

            if (durableQueue.isNotEmpty()) {
                Text("Reloaded After Launch", style = MaterialTheme.typography.titleMedium)
                durableQueue.forEach { record ->
                    Text("${record.recordId} · ${record.queueState}")
                }
            }

            statusMessage?.let {
                Text(it, color = MaterialTheme.colorScheme.primary)
            }

            errorMessage?.let {
                Text(it, color = MaterialTheme.colorScheme.error)
            }
        }
    }
}

@Composable
@OptIn(ExperimentalMaterial3Api::class)
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
