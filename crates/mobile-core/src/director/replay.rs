//! Golden spine replay — canonical input events → `helm-client` → `DirectorSnapshot`.
//!
//! `DirectorSnapshot.version` is the upstream SSE `sequence` from the spine fixture,
//! not a mobile-local counter (M3A.9 envelope contract).

use super::{DirectorSnapshot, quorum_director_fixture_snapshot, quorum_director_replay_fixture};
use crate::quorum::director_presenter::QuorumDomainPresenter;
use director_contracts::DirectorPrompt;
use helm_client::ClientHelm;
use helm_session_contracts::finding::FindingId;
use helm_session_contracts::gate::{GateCondition, GateId, GatedDecision};
use helm_session_contracts::push::{SessionContext, SessionPush};
use helm_session_contracts::urgency::UrgencyIntent;
use serde::Deserialize;
use serde_json::Value;

/// Immutable mobile snapshot envelope — wraps the canonical Helms type; version
/// tracks the spine SSE sequence that produced the frame.
#[derive(Clone, Debug)]
pub struct MobileDirectorSnapshot {
    pub version: u64,
    pub frame: director_contracts::DirectorFrame,
    pub source_sequence: u64,
}

impl From<DirectorSnapshot> for MobileDirectorSnapshot {
    fn from(snapshot: DirectorSnapshot) -> Self {
        Self {
            version: snapshot.version,
            source_sequence: snapshot.version,
            frame: snapshot.frame,
        }
    }
}

#[derive(Debug, Deserialize)]
struct SpineFixture {
    session: SpineSession,
    events: Vec<SpineEvent>,
    #[allow(dead_code)]
    expected_projection: Option<ExpectedProjection>,
}

#[derive(Debug, Deserialize)]
struct SpineSession {
    session_id: String,
    objective: String,
}

#[derive(Debug, Deserialize)]
struct SpineEvent {
    sequence: u64,
    #[serde(rename = "type")]
    event_type: String,
    payload: Value,
}

#[derive(Debug, Deserialize)]
struct ExpectedProjection {
    snapshot_fixture: String,
    #[serde(default)]
    gate_source_sequence: Option<u64>,
}

/// Replay a committed spine input fixture through Client Helm and project a snapshot.
pub fn replay_spine_input_json(input_json: &str) -> Result<MobileDirectorSnapshot, ReplayError> {
    let fixture: SpineFixture = serde_json::from_str(input_json)?;
    let mut helm = ClientHelm::new();
    let session_id = fixture.session.session_id.clone();
    let mut last_sequence = 0_u64;

    for event in &fixture.events {
        last_sequence = event.sequence;
        match event.event_type.as_str() {
            "SessionPush" => {
                let _ = helm.handle_push(SessionPush {
                    finding_id: FindingId::from_string("find-spine-replay"),
                    urgency_intent: UrgencyIntent::Advisory,
                    payload: serde_json::json!({
                        "objective": fixture.session.objective,
                        "headline": event.payload.get("headline"),
                        "body": event.payload.get("body"),
                    }),
                    session_context: SessionContext {
                        session_id: session_id.clone(),
                        phase: "decision".into(),
                        cycle: 1,
                        timestamp_ms: event.sequence,
                    },
                });
            }
            "GateCondition" => {
                let gate_id = event
                    .payload
                    .get("gate_id")
                    .and_then(Value::as_str)
                    .unwrap_or("gate:unknown");
                helm.handle_gate(GatedDecision {
                    gate_id: GateId::from_string(gate_id),
                    condition: GateCondition::AnyParticipant,
                    payload: event.payload.clone(),
                    deadline: None,
                });
            }
            "JudgmentPrompt" => {
                // Judgment copy is presenter-owned today; spine records intent only.
            }
            other => {
                return Err(ReplayError::UnsupportedEventType(other.to_owned()));
            }
        }
    }

    let presenter = QuorumDomainPresenter;
    let snapshot = helm.director_snapshot(last_sequence, &presenter);
    Ok(MobileDirectorSnapshot {
        version: snapshot.version,
        source_sequence: last_sequence,
        frame: snapshot.frame,
    })
}

#[derive(Debug, thiserror::Error)]
pub enum ReplayError {
    #[error("invalid spine fixture JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("unsupported spine event type: {0}")]
    UnsupportedEventType(String),
}

/// Replay the committed Quorum decision-checkpoint spine and compare projectable fields.
pub fn replay_quorum_decision_checkpoint() -> Result<MobileDirectorSnapshot, ReplayError> {
    let fixture = quorum_director_replay_fixture();
    replay_spine_input_json(fixture.input_json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use director_contracts::BlockingState;

    #[test]
    fn golden_snapshot_deserializes_as_envelope() {
        let golden = quorum_director_fixture_snapshot().expect("golden snapshot");
        let envelope = MobileDirectorSnapshot::from(golden.clone());
        assert_eq!(envelope.version, 1844);
        assert_eq!(envelope.source_sequence, 1844);
        assert_eq!(
            envelope.frame.now.as_ref().map(|now| now.objective.as_str()),
            Some("Evaluate Vendor X's security claims")
        );
    }

    #[test]
    fn spine_replay_uses_sse_sequence_as_version() {
        let replayed = replay_quorum_decision_checkpoint().expect("replay spine");
        assert_eq!(replayed.version, 1844);
        assert_eq!(replayed.source_sequence, 1844);
    }

    #[test]
    fn spine_replay_gate_event_projects_blocking_gate_prompt() {
        let replayed = replay_quorum_decision_checkpoint().expect("replay spine");
        assert!(matches!(
            replayed.frame.prompt,
            Some(DirectorPrompt::Gate(_))
        ));
        assert!(matches!(
            replayed.frame.blocking,
            BlockingState::BlocksFormation
        ));
    }

    #[test]
    fn spine_push_projects_session_objective_before_gate() {
        let replayed = replay_spine_input_json(
            r#"{
              "session": {
                "session_id": "session:procurement-security-review",
                "objective": "Evaluate Vendor X's security claims"
              },
              "events": [{
                "sequence": 1842,
                "type": "SessionPush",
                "payload": { "headline": "Maria needs your opinion." }
              }]
            }"#,
        )
        .expect("push-only replay");
        assert_eq!(
            replayed.frame.now.as_ref().map(|now| now.objective.as_str()),
            Some("Evaluate Vendor X's security claims")
        );
    }
}
