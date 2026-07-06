package se.reflective.quorum.corebridge

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import uniffi.quorum_ffi.AppendEventType
import uniffi.quorum_ffi.ConsentState
import uniffi.quorum_ffi.SignalModality
import uniffi.quorum_ffi.SyncState

/**
 * On-device boundary test: the Kotlin adapter + UniFFI bindings + the Rust
 * static lib (jniLibs) all execute, mirroring the iOS `BridgeBoundaryTests`
 * FFI round-trip. Requires an emulator/device with the native lib loaded.
 */
@RunWith(AndroidJUnit4::class)
class FfiBridgeInstrumentedTest {
    @Test
    fun ffiAdapterRoundTripsThroughRealRustCore() = runBlocking {
        val bridge = QuorumCoreBridgeFFI(InstrumentationRegistry.getInstrumentation().targetContext)

        assertEquals("quorum.field_signal_capture.v1", bridge.workflowId())

        val draft = bridge.draftFieldSignal(
            inquiryThreadId = "inq_mobile_launch_risks",
            modality = SignalModality.VOICE_TRANSCRIPT,
            rawCapture = "The sales team says rollout is fine, but support is seeing confusion in every pilot.",
        )
        assertEquals("draft:inq_mobile_launch_risks:field-signal-v1", draft.draftId)
        assertEquals(SignalModality.VOICE_TRANSCRIPT, draft.modality)
        assertEquals(ConsentState.PENDING, draft.consentState)
        // Confidence and latent need come from the core's refinement loop
        // (crates/mobile-core/src/refine.rs), not a fixture: the contract is a
        // valid in-range score and a capture-derived need, not magic literals.
        assertTrue(
            "confidence ${draft.confidence.value} outside (0, 1]",
            draft.confidence.value > 0f && draft.confidence.value <= 1f,
        )
        assertTrue("latent need must be derived, not blank", draft.latentNeed.isNotBlank())

        val event = bridge.appendConsentedSignal(draft)
        assertEquals(AppendEventType.SIGNAL_DRAFT_CONSENTED, event.eventType)
        assertEquals(SyncState.QUEUED_FOR_SYNC, event.syncState)
        assertEquals(draft.draftId, event.draftId)
    }

    /**
     * Every modality survives the real boundary unchanged. The round-trip above
     * only exercises one case, so it can't catch a wire mix-up between two
     * specific cases (a UDL reorder or a bridge-mapping slip) — the failure mode
     * the enum-on-the-wire refactor could still regress into; this pins each case.
     */
    @Test
    fun everyModalityRoundTripsThroughRealRustCore() = runBlocking {
        val bridge = QuorumCoreBridgeFFI(InstrumentationRegistry.getInstrumentation().targetContext)
        for (modality in SignalModality.entries) {
            val draft = bridge.draftFieldSignal(
                inquiryThreadId = "inq_modality_roundtrip",
                modality = modality,
                rawCapture = "boundary fidelity probe",
            )
            assertEquals(modality, draft.modality)
        }
    }
}
