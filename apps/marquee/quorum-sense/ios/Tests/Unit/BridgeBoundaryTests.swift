import Testing
@testable import QuorumMobile

/// Thin boundary/integration tests around `QuorumCoreBridge`:
///  - the preview bridge behaves like the fixture,
///  - the hand-written spy records interactions and propagates errors,
///  - the real FFI adapter round-trips through the Rust core on-device.
@MainActor
@Suite("Bridge boundary")
struct BridgeBoundaryTests {
    @Test("preview bridge drafts then appends with fixture markers")
    func previewRoundTrip() async throws {
        let bridge = PreviewQuorumCoreBridge()
        let draft = try await bridge.draftFieldSignal(
            inquiryThreadId: "inq_mobile_launch_risks",
            modality: .voiceTranscript,
            rawCapture: "support is seeing confusion in every pilot"
        )
        #expect(draft.workflowId == "quorum.field_signal_capture.v1")
        #expect(draft.draftId == "draft:inq_mobile_launch_risks:field-signal-v1")
        #expect(draft.consentState == .pending)
        #expect(draft.confidence.value == 0.67)

        let event = try await bridge.appendConsentedSignal(draft)
        #expect(event.eventType == .signalDraftConsented)
        #expect(event.syncState == .queuedForSync)
        #expect(event.draftId == draft.draftId)
        #expect(event.inquiryThreadId == draft.inquiryThreadId)
    }

    @Test("preview bridge exposes the derived Director snapshot")
    func previewDirectorSnapshot() async throws {
        let bridge = PreviewQuorumCoreBridge()
        let snapshot = try await bridge.currentDirectorSnapshot()

        #expect(snapshot.version == 1844)
        #expect(snapshot.frame.now?.objective == "Evaluate Vendor X's security claims")
        #expect(snapshot.frame.secondary.contains { action in
            if case .respondGate(_, .reject) = action.intent { return true }
            return false
        })
    }

    @Test("Director actions cross the bridge as typed intents")
    func spyRecordsDirectorIntent() async throws {
        let spy = SpyQuorumCoreBridge()
        try await spy.submitDirectorIntent(.respondGate(
            gateId: "gate:procurement-security-approval",
            verdict: .approve
        ))

        #expect(spy.directorIntentCalls == [
            .respondGate(gateId: "gate:procurement-security-approval", verdict: .approve),
        ])
    }

    @Test("spy records the draft call and propagates a stubbed error")
    func spyRecordsAndThrows() async throws {
        let spy = SpyQuorumCoreBridge()
        spy.draftStub = .failure(SpyQuorumCoreBridge.TestError.boom)

        await #expect(throws: SpyQuorumCoreBridge.TestError.self) {
            _ = try await spy.draftFieldSignal(inquiryThreadId: "t", modality: .text, rawCapture: "r")
        }
        #expect(spy.draftCalls == [.init(inquiryThreadId: "t", modality: .text, rawCapture: "r")])
    }

    @Test("spy returns its stubbed success draft")
    func spyReturnsStub() async throws {
        let spy = SpyQuorumCoreBridge()
        spy.draftStub = .success(fixtureDraft())
        let draft = try await spy.draftFieldSignal(inquiryThreadId: "x", modality: .text, rawCapture: "y")
        #expect(draft.draftId == "draft:inq_mobile_launch_risks:field-signal-v1")
        #expect(spy.draftCalls.count == 1)
    }

    /// On-device integration: the Swift adapter + UniFFI bindings + Rust static
    /// lib all execute. This is the boundary contract the deep Rust suite pins.
    @Test("FFI adapter round-trips through the real Rust core")
    func ffiRoundTrip() async throws {
        let bridge = try QuorumCoreBridgeFFI()

        #expect(await bridge.workflowId() == "quorum.field_signal_capture.v1")

        let draft = try await bridge.draftFieldSignal(
            inquiryThreadId: "inq_mobile_launch_risks",
            modality: .voiceTranscript,
            rawCapture: "The sales team says rollout is fine, but support is seeing confusion in every pilot."
        )
        #expect(draft.draftId == "draft:inq_mobile_launch_risks:field-signal-v1")
        #expect(draft.modality == .voiceTranscript)
        #expect(draft.consentState == .pending)
        #expect(draft.confidence.value == 0.67)
        #expect(draft.latentNeed == "needs earlier visibility into organizational ambiguity")

        let event = try await bridge.appendConsentedSignal(draft)
        #expect(event.eventType == .signalDraftConsented)
        #expect(event.syncState == .queuedForSync)
        #expect(event.draftId == draft.draftId)
    }

    /// Every modality survives the real boundary unchanged. The single-case
    /// round-trip above can't catch a wire mix-up between two specific cases (a
    /// UDL reorder or a bridge-mapping slip) — the failure mode the enum-on-the-
    /// wire refactor could still regress into; this pins each case end to end.
    @Test("every modality round-trips through the real Rust core",
          arguments: SignalModality.allCases)
    func modalityRoundTripsThroughCore(_ modality: SignalModality) async throws {
        let bridge = try QuorumCoreBridgeFFI()
        let draft = try await bridge.draftFieldSignal(
            inquiryThreadId: "inq_modality_roundtrip",
            modality: modality,
            rawCapture: "boundary fidelity probe"
        )
        #expect(draft.modality == modality)
    }
}
