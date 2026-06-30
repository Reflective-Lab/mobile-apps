package se.reflective.quorum.queue

import org.json.JSONObject
import uniffi.quorum_ffi.ConsentDecision

/** Summary of a durable queue record for UI and reload (M4.6). */
data class PersistedQueueRecordSummary(
    val recordId: String,
    val queueState: String,
    val updatedAt: String,
)

fun persistedQueueRecordSummaryFromJSON(json: String): PersistedQueueRecordSummary? {
    val objectNode = runCatching { JSONObject(json) }.getOrNull() ?: return null
    val recordId = objectNode.optString("record_id").takeIf { it.isNotEmpty() } ?: return null
    val queueState = objectNode.optString("queue_state").takeIf { it.isNotEmpty() } ?: return null
    val updatedAt = objectNode.optString("updated_at").takeIf { it.isNotEmpty() } ?: return null
    return PersistedQueueRecordSummary(recordId, queueState, updatedAt)
}

fun ConsentDecision.label(): String = when (this) {
    ConsentDecision.ACCEPTED -> "Accepted"
    ConsentDecision.EDITED_AND_ACCEPTED -> "Edited and accepted"
    ConsentDecision.REJECTED -> "Rejected"
    ConsentDecision.SAVED_PRIVATE -> "Saved private"
    ConsentDecision.EXPIRED -> "Expired"
}

fun ConsentDecision.permitsQueue(): Boolean = when (this) {
    ConsentDecision.ACCEPTED,
    ConsentDecision.EDITED_AND_ACCEPTED,
    -> true
    ConsentDecision.REJECTED,
    ConsentDecision.SAVED_PRIVATE,
    ConsentDecision.EXPIRED,
    -> false
}

object QueueTimestamp {
    fun nowISO8601(): String = java.time.Instant.now().toString()
}
