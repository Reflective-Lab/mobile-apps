package se.reflective.quorum.corebridge

import androidx.test.ext.junit.runners.AndroidJUnit4
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Test
import org.junit.runner.RunWith
import se.reflective.quorum.capture.AppendEventType
import se.reflective.quorum.capture.ConsentState
import se.reflective.quorum.capture.SignalModality
import se.reflective.quorum.capture.SyncState

/**
 * On-device boundary test: the Kotlin adapter + UniFFI bindings + the Rust
 * static lib (jniLibs) all execute, mirroring the iOS `BridgeBoundaryTests`
 * FFI round-trip. Requires an emulator/device with the native lib loaded.
 */
@RunWith(AndroidJUnit4::class)
class FfiBridgeInstrumentedTest {
    @Test
    fun ffiAdapterRoundTripsThroughRealRustCore() = runBlocking {
        val bridge = QuorumCoreBridgeFFI()

        assertEquals("quorum.field_signal_capture.v1", bridge.workflowId())

        val draft = bridge.draftFieldSignal(
            inquiryThreadId = "inq_mobile_launch_risks",
            modality = SignalModality.VOICE_TRANSCRIPT,
            rawCapture = "The sales team says rollout is fine, but support is seeing confusion in every pilot.",
        )
        assertEquals("draft:inq_mobile_launch_risks:field-signal-v1", draft.draftId)
        assertEquals(SignalModality.VOICE_TRANSCRIPT, draft.modality)
        assertEquals(ConsentState.PENDING, draft.consentState)
        assertEquals(0.67f, draft.confidence.value, 0.0001f)
        assertEquals("needs earlier visibility into organizational ambiguity", draft.latentNeed)

        val event = bridge.appendConsentedSignal(draft)
        assertEquals(AppendEventType.SIGNAL_DRAFT_CONSENTED, event.eventType)
        assertEquals(SyncState.QUEUED_FOR_SYNC, event.syncState)
        assertEquals(draft.draftId, event.draftId)
    }
}
