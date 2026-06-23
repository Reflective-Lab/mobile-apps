package se.reflective.quorum.corebridge

import io.kotest.core.spec.style.FunSpec
import io.kotest.matchers.shouldBe
import kotlinx.coroutines.test.runTest
import se.reflective.quorum.capture.AppendEventType
import se.reflective.quorum.capture.Confidence
import se.reflective.quorum.capture.ConsentState
import se.reflective.quorum.capture.SignalModality
import se.reflective.quorum.capture.SyncState

class PreviewQuorumCoreBridgeSpec : FunSpec({
    test("preview bridge drafts then appends with fixture markers") {
        runTest {
            val bridge = PreviewQuorumCoreBridge()

            val draft = bridge.draftFieldSignal(
                inquiryThreadId = "inq_mobile_launch_risks",
                modality = SignalModality.VOICE_TRANSCRIPT,
                rawCapture = "support is seeing confusion in every pilot",
            )
            draft.workflowId shouldBe "quorum.field_signal_capture.v1"
            draft.draftId shouldBe "draft:inq_mobile_launch_risks:field-signal-v1"
            draft.consentState shouldBe ConsentState.PENDING
            draft.confidence shouldBe Confidence.literal(0.67f)

            val event = bridge.appendConsentedSignal(draft)
            event.eventType shouldBe AppendEventType.SIGNAL_DRAFT_CONSENTED
            event.syncState shouldBe SyncState.QUEUED_FOR_SYNC
            event.draftId shouldBe draft.draftId
            event.inquiryThreadId shouldBe draft.inquiryThreadId
        }
    }
})
