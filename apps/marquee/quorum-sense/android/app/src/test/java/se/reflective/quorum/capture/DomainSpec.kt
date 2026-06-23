package se.reflective.quorum.capture

import io.kotest.core.spec.style.FunSpec
import io.kotest.matchers.nulls.shouldBeNull
import io.kotest.matchers.shouldBe
import io.kotest.property.checkAll
import uniffi.quorum_ffi.AppendEventType
import uniffi.quorum_ffi.ConsentState
import uniffi.quorum_ffi.SignalModality
import uniffi.quorum_ffi.SyncState

/**
 * Kotlin-side domain value types. The closed sets (modality/consent/event/sync)
 * are now UniFFI-generated enums, so "unknown wire value" is unrepresentable and
 * needs no Kotlin parse/reject coverage — the deep invariant coverage lives in
 * Rust. What remains host-only is `Confidence` validation (a float the UDL can't
 * constrain) and the UI affordances layered onto the generated enums.
 */
class DomainSpec : FunSpec({
    test("Confidence.of accepts inclusive bounds and mid-range") {
        Confidence.of(0f)?.value shouldBe 0f
        Confidence.of(1f)?.value shouldBe 1f
        Confidence.of(0.67f)?.value shouldBe 0.67f
    }

    test("Confidence.of rejects out-of-range and non-finite") {
        Confidence.of(-0.0001f).shouldBeNull()
        Confidence.of(1.0001f).shouldBeNull()
        Confidence.of(Float.NaN).shouldBeNull()
        Confidence.of(Float.POSITIVE_INFINITY).shouldBeNull()
        Confidence.of(Float.NEGATIVE_INFINITY).shouldBeNull()
    }

    test("property: Confidence.of is non-null iff the value is finite and within 0..1") {
        checkAll<Float> { value ->
            val expectedValid = value.isFinite() && value in 0f..1f
            (Confidence.of(value) != null) shouldBe expectedValid
        }
    }

    test("modality exposes every case to the picker") {
        SignalModality.entries shouldBe listOf(
            SignalModality.TEXT,
            SignalModality.VOICE_TRANSCRIPT,
            SignalModality.IMAGE_OCR_TEXT,
        )
    }

    test("human-facing labels are present") {
        SignalModality.VOICE_TRANSCRIPT.label shouldBe "Voice"
        SignalModality.IMAGE_OCR_TEXT.label shouldBe "Photo OCR"
        ConsentState.PENDING.label shouldBe "Pending"
        AppendEventType.SIGNAL_DRAFT_CONSENTED.label shouldBe "Signal draft consented"
        SyncState.QUEUED_FOR_SYNC.label shouldBe "Queued for sync"
    }
})
