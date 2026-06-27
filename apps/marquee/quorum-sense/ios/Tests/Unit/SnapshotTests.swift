import SnapshotTesting
import Testing
@testable import QuorumMobile

/// Golden snapshot of the Swift domain draft, mirroring the Rust `insta` golden.
/// Uses the deterministic `.dump` strategy (no UI rendering), so the reference
/// is stable across machines and OS versions. Run once to record the reference
/// under `__Snapshots__/`, then commit it.
@MainActor
@Suite("Snapshot golden")
struct SnapshotTests {
    @Test func draftDumpMatchesGolden() {
        assertSnapshot(of: fixtureDraft(), as: .dump)
    }

    @Test func appendEventDumpMatchesGolden() {
        let event = QuorumAppendEvent(
            workflowId: "quorum.field_signal_capture.v1",
            eventType: .signalDraftConsented,
            draftId: "draft:inq_mobile_launch_risks:field-signal-v1",
            inquiryThreadId: "inq_mobile_launch_risks",
            syncState: .queuedForSync
        )
        assertSnapshot(of: event, as: .dump)
    }

    @Test func directorSnapshotMatchesFixtureContract() {
        let snapshot = DirectorFixture.quorumDecisionCheckpoint

        #expect(snapshot.version == 1844)
        #expect(snapshot.frame.now?.objective == "Evaluate Vendor X's security claims")
        #expect(snapshot.frame.blocking == .blocksFormation)
    }
}
