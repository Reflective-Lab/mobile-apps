package se.reflective.quorum.capture

import io.kotest.core.spec.style.FunSpec
import io.kotest.matchers.nulls.shouldBeNull
import io.kotest.matchers.shouldBe
import io.kotest.property.checkAll

/**
 * Kotlin-side domain value types. The deep invariant coverage lives in the Rust
 * core; here we assert the mapping types (validation + wire-string round-tripping)
 * the FFI boundary relies on.
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

    test("modality wire values round-trip and reject unknowns") {
        SignalModality.entries.forEach { SignalModality.fromWire(it.wireName) shouldBe it }
        SignalModality.fromWire("hologram").shouldBeNull()
        SignalModality.fromWire("").shouldBeNull()
        SignalModality.fromWire("Text").shouldBeNull() // case-sensitive wire contract
    }

    test("consent / event / sync wire values round-trip and reject unknowns") {
        ConsentState.entries.forEach { ConsentState.fromWire(it.wireName) shouldBe it }
        ConsentState.fromWire("revoked").shouldBeNull()

        AppendEventType.fromWire("SignalDraftConsented") shouldBe AppendEventType.SIGNAL_DRAFT_CONSENTED
        AppendEventType.fromWire("nope").shouldBeNull()

        SyncState.fromWire("queued_for_sync") shouldBe SyncState.QUEUED_FOR_SYNC
        SyncState.fromWire("later").shouldBeNull()
    }
})
