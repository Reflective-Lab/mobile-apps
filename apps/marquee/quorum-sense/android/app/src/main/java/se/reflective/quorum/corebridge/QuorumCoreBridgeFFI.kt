package se.reflective.quorum.corebridge

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import se.reflective.quorum.capture.Confidence
import se.reflective.quorum.capture.FieldSignalDraft
import se.reflective.quorum.capture.QuorumAppendEvent
import uniffi.quorum_ffi.FfiQuorumAppendEvent
import uniffi.quorum_ffi.FfiQuorumSignalDraft
import uniffi.quorum_ffi.SignalModality
import uniffi.quorum_ffi.quorumAppendConsentedSignal
import uniffi.quorum_ffi.quorumDraftFieldSignal
import uniffi.quorum_ffi.quorumFieldSignalWorkflowId

/**
 * Raised when the Rust core returns a value this app build cannot map into a
 * domain type. With modality/consent/event/sync now enums on the wire, the only
 * remaining unmappable value is a confidence outside `0..1` (a float the UDL
 * can't constrain). Distinct from the generated `QuorumException`, which the FFI
 * functions themselves throw when the *core* rejects input.
 */
sealed class CoreBridgeException(message: String) : Exception(message) {
    class ConfidenceOutOfRange(val value: Float) :
        CoreBridgeException("core returned confidence $value outside 0..1")
}

/**
 * Production [QuorumCoreBridge] backed by the Rust core through UniFFI.
 *
 * The generated bindings (`quorumDraftFieldSignal`, `FfiQuorumSignalDraft`,
 * `QuorumException`, …) live under `uniffi.quorum_ffi` and link against the
 * `jniLibs/<abi>/libquorum_ffi.so` produced by `scripts/build-mobile-ffi-android.sh`.
 * Both are generated and gitignored, so this file does not compile until that
 * script has run.
 *
 * This adapter is the *only* place the FFI wire DTOs (`Ffi*`) are translated to
 * and from the domain types, mirroring `quorum-ffi/src/lib.rs` and the iOS
 * `QuorumCoreBridgeFFI`. The closed-set fields are enums on the wire now, so they
 * cross unchanged; only `confidence` (a float) is validated. The synchronous FFI
 * runs off the main dispatcher (ADR 0003) so the UI thread is never blocked.
 */
class QuorumCoreBridgeFFI : QuorumCoreBridge {
    override suspend fun workflowId(): String =
        withContext(Dispatchers.Default) { quorumFieldSignalWorkflowId() }

    override suspend fun draftFieldSignal(
        inquiryThreadId: String,
        modality: SignalModality,
        rawCapture: String,
    ): FieldSignalDraft =
        withContext(Dispatchers.Default) {
            quorumDraftFieldSignal(inquiryThreadId, modality, rawCapture).toDomain()
        }

    override suspend fun appendConsentedSignal(draft: FieldSignalDraft): QuorumAppendEvent =
        withContext(Dispatchers.Default) {
            quorumAppendConsentedSignal(draft.toFfi()).toDomain()
        }

    // Boundary mapping (FFI wire DTO <-> domain). The closed sets are enums on the
    // wire and cross as-is; only the float `confidence` still needs validation.

    private fun FfiQuorumSignalDraft.toDomain(): FieldSignalDraft {
        val parsedConfidence = Confidence.of(confidence)
            ?: throw CoreBridgeException.ConfidenceOutOfRange(confidence)
        return FieldSignalDraft(
            workflowId = workflowId,
            draftId = draftId,
            inquiryThreadId = inquiryThreadId,
            modality = modality,
            rawCapture = rawCapture,
            summary = summary,
            latentNeed = latentNeed,
            contradiction = contradiction,
            confidence = parsedConfidence,
            consentState = consentState,
        )
    }

    private fun FieldSignalDraft.toFfi(): FfiQuorumSignalDraft =
        FfiQuorumSignalDraft(
            workflowId = workflowId,
            draftId = draftId,
            inquiryThreadId = inquiryThreadId,
            modality = modality,
            rawCapture = rawCapture,
            summary = summary,
            latentNeed = latentNeed,
            contradiction = contradiction,
            confidence = confidence.value,
            consentState = consentState,
        )

    private fun FfiQuorumAppendEvent.toDomain(): QuorumAppendEvent =
        QuorumAppendEvent(
            workflowId = workflowId,
            eventType = eventType,
            draftId = draftId,
            inquiryThreadId = inquiryThreadId,
            syncState = syncState,
        )
}
