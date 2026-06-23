package se.reflective.quorum.corebridge

import io.kotest.assertions.throwables.shouldThrow
import io.kotest.core.spec.style.FunSpec
import io.kotest.matchers.shouldBe
import io.mockk.coEvery
import io.mockk.coVerifyOrder
import io.mockk.mockk
import kotlinx.coroutines.test.runTest
import se.reflective.quorum.capture.AppendEventType
import se.reflective.quorum.capture.Confidence
import se.reflective.quorum.capture.ConsentState
import se.reflective.quorum.capture.FieldSignalDraft
import se.reflective.quorum.capture.QuorumAppendEvent
import se.reflective.quorum.capture.SignalModality
import se.reflective.quorum.capture.SyncState

/** MockK-driven tests of a consumer talking to the bridge contract. */
class BridgeConsumerSpec : FunSpec({
    val sampleDraft = FieldSignalDraft(
        workflowId = "quorum.field_signal_capture.v1",
        draftId = "draft:inq:field-signal-v1",
        inquiryThreadId = "inq",
        modality = SignalModality.TEXT,
        rawCapture = "r",
        summary = "r",
        latentNeed = "n",
        contradiction = "c",
        confidence = Confidence.literal(0.5f),
        consentState = ConsentState.PENDING,
    )
    val sampleEvent = QuorumAppendEvent(
        workflowId = "quorum.field_signal_capture.v1",
        eventType = AppendEventType.SIGNAL_DRAFT_CONSENTED,
        draftId = "draft:inq:field-signal-v1",
        inquiryThreadId = "inq",
        syncState = SyncState.QUEUED_FOR_SYNC,
    )

    test("a consumer drafts then appends, in order") {
        runTest {
            val bridge = mockk<QuorumCoreBridge>()
            coEvery { bridge.draftFieldSignal(any(), any(), any()) } returns sampleDraft
            coEvery { bridge.appendConsentedSignal(any()) } returns sampleEvent

            val draft = bridge.draftFieldSignal("inq", SignalModality.TEXT, "r")
            val event = bridge.appendConsentedSignal(draft)

            event shouldBe sampleEvent
            coVerifyOrder {
                bridge.draftFieldSignal("inq", SignalModality.TEXT, "r")
                bridge.appendConsentedSignal(sampleDraft)
            }
        }
    }

    test("bridge errors propagate to the caller") {
        runTest {
            val bridge = mockk<QuorumCoreBridge>()
            coEvery { bridge.draftFieldSignal(any(), any(), any()) } throws IllegalStateException("boom")

            shouldThrow<IllegalStateException> {
                bridge.draftFieldSignal("t", SignalModality.TEXT, "r")
            }
        }
    }
})
