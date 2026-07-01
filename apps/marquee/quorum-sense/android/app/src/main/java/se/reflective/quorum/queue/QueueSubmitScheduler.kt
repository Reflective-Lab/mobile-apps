package se.reflective.quorum.queue

import android.content.Context
import androidx.work.BackoffPolicy
import androidx.work.Constraints
import androidx.work.ExistingWorkPolicy
import androidx.work.NetworkType
import androidx.work.OneTimeWorkRequestBuilder
import androidx.work.WorkManager
import androidx.work.WorkRequest
import java.util.concurrent.TimeUnit

/**
 * WorkManager hook for durable queue submission (M5.6).
 *
 * Call [enqueue] after consent persists a record, and on app startup to flush
 * any stranded queue entries from prior sessions.
 */
object QueueBackgroundSubmit {
    const val WORK_NAME = "se.reflective.quorum.queue-submit"

    fun enqueue(context: Context) {
        val appContext = context.applicationContext
        val constraints = Constraints.Builder()
            .setRequiredNetworkType(NetworkType.CONNECTED)
            .build()

        val request = OneTimeWorkRequestBuilder<QueueSubmitWorker>()
            .setConstraints(constraints)
            .setBackoffCriteria(
                BackoffPolicy.EXPONENTIAL,
                WorkRequest.MIN_BACKOFF_MILLIS,
                TimeUnit.MILLISECONDS,
            )
            .build()

        WorkManager.getInstance(appContext).enqueueUniqueWork(
            WORK_NAME,
            ExistingWorkPolicy.KEEP,
            request,
        )
    }
}

/**
 * Schedules background queue submission. Production uses [QueueBackgroundSubmit];
 * tests may substitute [NoOpQueueSubmitScheduler].
 */
fun interface QueueSubmitScheduler {
    fun enqueuePendingRecords()
}

class WorkManagerQueueSubmitScheduler(
    private val appContext: Context,
) : QueueSubmitScheduler {
    override fun enqueuePendingRecords() {
        QueueBackgroundSubmit.enqueue(appContext)
    }
}

object NoOpQueueSubmitScheduler : QueueSubmitScheduler {
    override fun enqueuePendingRecords() = Unit
}
