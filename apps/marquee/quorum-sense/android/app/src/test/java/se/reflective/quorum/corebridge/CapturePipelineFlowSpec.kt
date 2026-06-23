package se.reflective.quorum.corebridge

import app.cash.turbine.test
import io.kotest.core.spec.style.FunSpec
import io.kotest.matchers.shouldBe
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.test.runTest
import se.reflective.quorum.capture.SignalModality

/** A capture pipeline modelled as a Flow, exercised with Turbine. */
private fun capturePipeline(
    bridge: QuorumCoreBridge,
    inquiryThreadId: String,
    modality: SignalModality,
    rawCapture: String,
): Flow<String> = flow {
    val draft = bridge.draftFieldSignal(inquiryThreadId, modality, rawCapture)
    emit("draft:${draft.draftId}")
    val event = bridge.appendConsentedSignal(draft)
    emit("event:${event.eventType.wireName}")
}

class CapturePipelineFlowSpec : FunSpec({
    test("pipeline emits the draft id then the consented event, then completes") {
        runTest {
            capturePipeline(
                bridge = PreviewQuorumCoreBridge(),
                inquiryThreadId = "inq_mobile_launch_risks",
                modality = SignalModality.TEXT,
                rawCapture = "support is seeing confusion",
            ).test {
                awaitItem() shouldBe "draft:draft:inq_mobile_launch_risks:field-signal-v1"
                awaitItem() shouldBe "event:SignalDraftConsented"
                awaitComplete()
            }
        }
    }
})
