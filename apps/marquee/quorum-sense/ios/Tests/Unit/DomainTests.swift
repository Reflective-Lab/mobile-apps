import Testing
@testable import QuorumMobile

/// Swift-side domain value types. These mirror the Rust domain; the deep
/// invariant coverage lives in Rust, so here we assert the Swift mapping types
/// (validation + wire-string round-tripping) the FFI boundary depends on.
@Suite("Domain value types")
struct DomainTests {
    @Test("Confidence accepts inclusive bounds and mid-range")
    func confidenceAccepts() {
        #expect(Confidence(0)?.value == 0)
        #expect(Confidence(1)?.value == 1)
        #expect(Confidence(0.67)?.value == 0.67)
    }

    @Test("Confidence rejects out-of-range and non-finite", arguments: [
        Float(-0.0001), Float(1.0001), Float.nan, Float.infinity, -Float.infinity,
    ])
    func confidenceRejects(_ value: Float) {
        #expect(Confidence(value) == nil)
    }

    @Test("modality wire values round-trip", arguments: SignalModality.allCases)
    func modalityRoundTrips(_ modality: SignalModality) {
        #expect(SignalModality(rawValue: modality.rawValue) == modality)
    }

    @Test("unknown wire strings map to nil")
    func unknownWireStrings() {
        #expect(SignalModality(rawValue: "hologram") == nil)
        #expect(ConsentState(rawValue: "revoked") == nil)
        #expect(AppendEventType(rawValue: "Nope") == nil)
        #expect(SyncState(rawValue: "later") == nil)
    }

    @Test("wire raw values are the stable contract")
    func wireRawValues() {
        #expect(SignalModality.voiceTranscript.rawValue == "voice_transcript")
        #expect(SignalModality.imageOcrText.rawValue == "image_ocr_text")
        #expect(ConsentState.pending.rawValue == "pending")
        #expect(AppendEventType.signalDraftConsented.rawValue == "SignalDraftConsented")
        #expect(SyncState.queuedForSync.rawValue == "queued_for_sync")
    }

    @Test("human-facing labels are present")
    func labels() {
        #expect(ConsentState.pending.label == "Pending")
        #expect(AppendEventType.signalDraftConsented.label == "Signal draft consented")
        #expect(SyncState.queuedForSync.label == "Queued for sync")
    }
}
