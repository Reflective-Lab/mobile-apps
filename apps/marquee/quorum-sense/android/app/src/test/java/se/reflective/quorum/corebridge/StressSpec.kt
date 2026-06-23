package se.reflective.quorum.corebridge

import io.kotest.core.spec.style.FunSpec
import io.kotest.matchers.shouldBe
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.test.runTest
import uniffi.quorum_ffi.SignalModality
import kotlin.time.Duration.Companion.seconds

/**
 * Concurrency stress: thousands of overlapping draft+append coroutines on the
 * real Default dispatcher. `runTest(timeout=…)` turns any deadlock/hang into a
 * deterministic failure instead of blocking the suite forever.
 */
class StressSpec : FunSpec({
    test("thousands of concurrent draft+append complete without deadlock/hang") {
        runTest(timeout = 60.seconds) {
            val bridge = PreviewQuorumCoreBridge()
            val taskCount = 2_000

            val results = (0 until taskCount).map { i ->
                async(Dispatchers.Default) {
                    val draft = bridge.draftFieldSignal("inq_$i", SignalModality.TEXT, "payload under load $i")
                    val event = bridge.appendConsentedSignal(draft)
                    event.draftId == draft.draftId
                }
            }.awaitAll()

            results.size shouldBe taskCount
            results.all { it } shouldBe true
        }
    }
})
