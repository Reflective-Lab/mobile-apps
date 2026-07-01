package se.reflective.quorum.queue

import android.content.Context
import androidx.work.CoroutineWorker
import androidx.work.WorkerParameters
import se.reflective.quorum.app.clientVersion

/**
 * Background worker that submits eligible offline queue records through the Rust
 * HTTP boundary (M5.6). Mirrors iOS `QueueBackgroundSubmit` / BGTaskScheduler.
 */
class QueueSubmitWorker(
    appContext: Context,
    params: WorkerParameters,
) : CoroutineWorker(appContext, params) {

    override suspend fun doWork(): Result =
        runQueueSubmit(
            appContext = applicationContext,
            persistence = QuorumQueuePersistence.production(
                context = applicationContext,
                clientVersion = applicationContext.clientVersion(),
            ),
            attempt = runAttemptCount,
        )

    internal companion object {
        const val MAX_ATTEMPTS = 5

        internal suspend fun runQueueSubmit(
            appContext: Context,
            persistence: QuorumQueuePersistence,
            attempt: Int,
            reschedule: (Context) -> Unit = { QueueBackgroundSubmit.enqueue(it) },
        ): Result =
            try {
                val submitted = persistence.submitEligibleRecords()
                if (submitted > 0) {
                    reschedule(appContext)
                }
                Result.success()
            } catch (_: Exception) {
                if (attempt < MAX_ATTEMPTS) {
                    Result.retry()
                } else {
                    Result.failure()
                }
            }
    }
}
