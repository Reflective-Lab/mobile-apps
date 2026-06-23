package se.reflective.quorum.corebridge

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import se.reflective.quorum.capture.AppendEventType
import se.reflective.quorum.capture.Confidence
import se.reflective.quorum.capture.ConsentState
import se.reflective.quorum.capture.FieldSignalDraft
import se.reflective.quorum.capture.QuorumAppendEvent
import se.reflective.quorum.capture.SignalModality
import se.reflective.quorum.capture.SyncState
import uniffi.quorum_ffi.FfiQuorumAppendEvent
import uniffi.quorum_ffi.FfiQuorumSignalDraft
import uniffi.quorum_ffi.quorumAppendConsentedSignal
import uniffi.quorum_ffi.quorumDraftFieldSignal
import uniffi.quorum_ffi.quorumFieldSignalWorkflowId

/**
 * Raised when the Rust core returns a value this app build cannot map into a
 * domain type (a core/app version skew). Distinct from the generated
 * `QuorumException`, which the FFI functions themselves throw when the *core*
 * rejects input.
 */
sealed class CoreBridgeException(message: String) : Exception(message) {
    class UnrecognizedCoreValue(val field: String, val value: String) :
        CoreBridgeException("core returned unrecognized $field: $value")

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
 * This adapter is the *only* place the FFI wire DTOs (`Ffi*`, string/float-typed)
 * are translated to and from the domain types, mirroring `quorum-ffi/src/lib.rs`
 * and the iOS `QuorumCoreBridgeFFI`. The synchronous FFI runs off the main
 * dispatcher (ADR 0003) so the UI thread is never blocked.
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
            quorumDraftFieldSignal(inquiryThreadId, modality.wireName, rawCapture).toDomain()
        }

    override suspend fun appendConsentedSignal(draft: FieldSignalDraft): QuorumAppendEvent =
        withContext(Dispatchers.Default) {
            quorumAppendConsentedSignal(draft.toFfi()).toDomain()
        }

    // Boundary mapping (FFI wire DTO <-> domain) — the single home for
    // String/Float <-> domain conversion on this platform.

    private fun FfiQuorumSignalDraft.toDomain(): FieldSignalDraft {
        val parsedModality = SignalModality.fromWire(modality)
            ?: throw CoreBridgeException.UnrecognizedCoreValue("modality", modality)
        val parsedConsent = ConsentState.fromWire(consentState)
            ?: throw CoreBridgeException.UnrecognizedCoreValue("consentState", consentState)
        val parsedConfidence = Confidence.of(confidence)
            ?: throw CoreBridgeException.ConfidenceOutOfRange(confidence)
        return FieldSignalDraft(
            workflowId = workflowId,
            draftId = draftId,
            inquiryThreadId = inquiryThreadId,
            modality = parsedModality,
            rawCapture = rawCapture,
            summary = summary,
            latentNeed = latentNeed,
            contradiction = contradiction,
            confidence = parsedConfidence,
            consentState = parsedConsent,
        )
    }

    private fun FieldSignalDraft.toFfi(): FfiQuorumSignalDraft =
        FfiQuorumSignalDraft(
            workflowId = workflowId,
            draftId = draftId,
            inquiryThreadId = inquiryThreadId,
            modality = modality.wireName,
            rawCapture = rawCapture,
            summary = summary,
            latentNeed = latentNeed,
            contradiction = contradiction,
            confidence = confidence.value,
            consentState = consentState.wireName,
        )

    private fun FfiQuorumAppendEvent.toDomain(): QuorumAppendEvent {
        val parsedType = AppendEventType.fromWire(eventType)
            ?: throw CoreBridgeException.UnrecognizedCoreValue("eventType", eventType)
        val parsedSync = SyncState.fromWire(syncState)
            ?: throw CoreBridgeException.UnrecognizedCoreValue("syncState", syncState)
        return QuorumAppendEvent(
            workflowId = workflowId,
            eventType = parsedType,
            draftId = draftId,
            inquiryThreadId = inquiryThreadId,
            syncState = parsedSync,
        )
    }
}
