package se.reflective.quorum.queue

import androidx.work.ListenableWorker.Result
import io.kotest.core.spec.style.FunSpec
import io.kotest.matchers.shouldBe
import io.mockk.coEvery
import io.mockk.mockk
import kotlinx.coroutines.test.runTest

class QueueSubmitWorkerSpec : FunSpec({
    test("successful submit reschedules when records were admitted") {
        runTest {
            val persistence = mockk<QuorumQueuePersistence>()
            coEvery { persistence.submitEligibleRecords() } returns 2
            var rescheduleCount = 0

            val result = QueueSubmitWorker.runQueueSubmit(
                appContext = mockk(relaxed = true),
                persistence = persistence,
                attempt = 0,
                reschedule = { rescheduleCount += 1 },
            )

            result shouldBe Result.success()
            rescheduleCount shouldBe 1
        }
    }

    test("failed submit retries until max attempts") {
        runTest {
            val persistence = mockk<QuorumQueuePersistence>()
            coEvery { persistence.submitEligibleRecords() } throws IllegalStateException("offline")

            QueueSubmitWorker.runQueueSubmit(
                appContext = mockk(relaxed = true),
                persistence = persistence,
                attempt = 2,
            ) shouldBe Result.retry()

            QueueSubmitWorker.runQueueSubmit(
                appContext = mockk(relaxed = true),
                persistence = persistence,
                attempt = QueueSubmitWorker.MAX_ATTEMPTS,
            ) shouldBe Result.failure()
        }
    }
})
