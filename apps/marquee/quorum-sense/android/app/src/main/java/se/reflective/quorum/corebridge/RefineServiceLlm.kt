package se.reflective.quorum.corebridge

import java.net.HttpURLConnection
import java.net.URL
import org.json.JSONObject
import uniffi.quorum_ffi.LlmBackend

/**
 * Cloud-fallback LLM backend (M6 compute placement). Implements the generated
 * UniFFI [LlmBackend] callback: the Rust refinement loop calls [complete], and
 * this POSTs the prompt to the local Quorum refine-service, which holds the API
 * keys (so no key ever reaches the device). Returns `null` on any failure — the
 * Rust refiner then falls back to its deterministic heuristics, so a draft is
 * always produced.
 *
 * Blocking by design: [complete] is called synchronously from Rust, which runs
 * on `Dispatchers.Default` off the main thread, so waiting on the request here
 * never freezes the UI. `10.0.2.2` is the host loopback as seen from the Android
 * emulator; cleartext to it is allowed via `usesCleartextTraffic` in the debug
 * manifest. A device build would point this at the GC-Secrets backend instead.
 */
class RefineServiceLlm(
    private val endpoint: String = "http://10.0.2.2:8765/complete",
) : LlmBackend {
    override fun `complete`(`prompt`: String): String? {
        return try {
            val connection = (URL(endpoint).openConnection() as HttpURLConnection).apply {
                requestMethod = "POST"
                connectTimeout = 5_000
                readTimeout = 20_000
                doOutput = true
                setRequestProperty("Content-Type", "application/json")
            }
            connection.outputStream.use { out ->
                out.write(JSONObject().put("prompt", prompt).toString().toByteArray())
            }
            val text =
                if (connection.responseCode == 200) {
                    val body = connection.inputStream.bufferedReader().use { it.readText() }
                    JSONObject(body).optString("text", "")
                } else {
                    ""
                }
            connection.disconnect()
            text.ifBlank { null }
        } catch (e: Exception) {
            // Any network/parse failure -> null -> Rust heuristic fallback.
            null
        }
    }
}
