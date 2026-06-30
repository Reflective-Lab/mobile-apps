package se.reflective.quorum.queue

import android.content.Context
import java.io.File

/** Native durability adapter for offline queue records (M4.6, ADR 0005). */
class FileQueueStore(
    context: Context,
    subdirectory: String = "queue",
) {
    class StoreException(message: String) : Exception(message)

    private val directory: File =
        File(context.filesDir, subdirectory).also { it.mkdirs() }

    fun save(recordId: String, json: String) {
        validateRecordId(recordId)
        val file = fileFor(recordId)
        val temp = File(directory, "$recordId.json.tmp")
        temp.writeText(json)
        if (file.exists()) file.delete()
        check(temp.renameTo(file)) { "failed to persist queue record $recordId" }
    }

    fun load(recordId: String): String? {
        validateRecordId(recordId)
        val file = fileFor(recordId)
        return if (file.exists()) file.readText() else null
    }

    fun allRecordIds(): List<String> =
        directory.listFiles()
            ?.filter { it.isFile && it.extension == "json" && !it.name.endsWith(".tmp") }
            ?.map { it.nameWithoutExtension }
            ?.sorted()
            ?: emptyList()

    fun loadAllJSON(): Map<String, String> =
        allRecordIds().mapNotNull { recordId ->
            load(recordId)?.let { recordId to it }
        }.toMap()

    fun remove(recordId: String) {
        validateRecordId(recordId)
        fileFor(recordId).delete()
    }

    private fun fileFor(recordId: String): File = File(directory, "$recordId.json")

    private fun validateRecordId(recordId: String) {
        require(recordId.isNotEmpty() && !recordId.contains('/') && !recordId.contains("..")) {
            throw StoreException("invalid record id: $recordId")
        }
    }
}
