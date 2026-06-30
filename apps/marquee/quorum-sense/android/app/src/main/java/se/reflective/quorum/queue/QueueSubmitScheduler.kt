package se.reflective.quorum.queue

/**
 * M5.6 hook — WorkManager wiring lands once server submit exists. Call sites enqueue
 * durable records through this interface so background retry can plug in without
 * rewriting capture UI.
 */
fun interface QueueSubmitScheduler {
    fun enqueuePendingRecords()
}

object NoOpQueueSubmitScheduler : QueueSubmitScheduler {
    override fun enqueuePendingRecords() = Unit
}
