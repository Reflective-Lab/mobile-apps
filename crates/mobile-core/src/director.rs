//! Mobile consumption of the Helms director contract.
//!
//! `director-contracts` owns `DirectorFrame`; this module re-exports it and
//! carries fixture replay wiring only — no parallel frame definition.

pub mod live;

pub use live::{
    DEFAULT_LOCAL_DEV_BEARER, DEFAULT_LOCAL_QUORUM_BASE_URL, DIRECTOR_SNAPSHOT_PATH,
    DirectorApiConfig, DirectorSnapshotSource, LiveDirectorError, ResolveDirectorError,
    director_snapshot_url, fetch_live_director_snapshot, resolve_director_snapshot,
};

pub use director_contracts::{
    BlockingState, Choice, ContextLevel, DirectorFrame, DirectorIntent, DirectorPrompt,
    DirectorSnapshot, GatePrompt, GateVerdict, JudgmentPrompt, NowTask, PresenceHint,
    PrimaryAction, ReviewPrompt, ReviewStance, SecondaryAction, WaitingFor,
};
pub use helm_session_contracts::gate::GateId;

/// Stable wire label for a gate condition (UniFFI cannot carry `GateCondition`).
#[must_use]
pub fn gate_condition_wire_label(
    condition: &helm_session_contracts::gate::GateCondition,
) -> &'static str {
    use helm_session_contracts::gate::GateCondition;
    match condition {
        GateCondition::QuorumOfRoles { .. } => "quorum_of_roles",
        GateCondition::SpecificAuthority { .. } => "specific_authority",
        GateCondition::AnyParticipant => "any_participant",
        GateCondition::Unanimous => "unanimous",
    }
}

pub const QUORUM_DIRECTOR_INPUT_FIXTURE_JSON: &str = include_str!(
    "../../../apps/marquee/quorum-sense/fixtures/quorum-director-decision-checkpoint.input.v1.json"
);
pub const QUORUM_DIRECTOR_SNAPSHOT_FIXTURE_JSON: &str = include_str!(
    "../../../apps/marquee/quorum-sense/fixtures/quorum-director-decision-checkpoint.snapshot.v1.json"
);

/// Fixture metadata the replay harness is allowed to own alongside the canonical
/// Helms snapshot envelope.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DirectorReplayFixture {
    pub name: &'static str,
    pub input_json: &'static str,
    pub snapshot_json: &'static str,
}

pub fn quorum_director_replay_fixture() -> DirectorReplayFixture {
    DirectorReplayFixture {
        name: "quorum.director.decision_checkpoint.v1",
        input_json: QUORUM_DIRECTOR_INPUT_FIXTURE_JSON,
        snapshot_json: QUORUM_DIRECTOR_SNAPSHOT_FIXTURE_JSON,
    }
}

/// Parse the committed golden `DirectorSnapshot` fixture.
pub fn quorum_director_fixture_snapshot() -> Result<DirectorSnapshot, serde_json::Error> {
    serde_json::from_str(QUORUM_DIRECTOR_SNAPSHOT_FIXTURE_JSON)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_deserializes_as_canonical_director_snapshot() {
        let snapshot = quorum_director_fixture_snapshot().expect("fixture must parse");
        assert_eq!(snapshot.version, 1844);
        assert_eq!(
            snapshot
                .frame
                .now
                .as_ref()
                .map(|now| now.objective.as_str()),
            Some("Evaluate Vendor X's security claims")
        );
    }

    #[test]
    fn director_fixture_is_derived_from_ordered_spine_input() {
        let fixture = quorum_director_replay_fixture();

        assert!(fixture.input_json.contains("\"sequence\": 1844"));
        assert!(fixture.snapshot_json.contains("\"version\": 1844"));
    }

    #[test]
    fn director_gate_has_no_ui_only_defer_choice() {
        let json = QUORUM_DIRECTOR_SNAPSHOT_FIXTURE_JSON;

        assert!(json.contains("\"verdict\": \"reject\""));
        assert!(!json.contains("Later"));
        assert!(!json.contains("Defer"));
    }

    #[test]
    fn mobile_core_reexports_director_contracts_not_a_fork() {
        let _: DirectorSnapshot = quorum_director_fixture_snapshot().expect("parse");
    }
}
