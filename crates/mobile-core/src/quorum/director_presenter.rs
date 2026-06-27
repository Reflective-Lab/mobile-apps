use director_contracts::NowTask;
use helm_client::director::{DomainPresenter, GateCopy};
use helm_client::formation::LocalFormationIntent;
use helm_session_contracts::gate::GatedDecision;

/// Quorum-specific human copy for `helm-client`'s domain-agnostic projection.
///
/// Reads reason/consequence from the gate's opaque payload when present; falls
/// back to conservative defaults so projection never panics on missing fields.
pub struct QuorumDomainPresenter;

impl DomainPresenter for QuorumDomainPresenter {
    fn now_task(&self, intent: &LocalFormationIntent) -> NowTask {
        NowTask {
            objective: intent.description.clone(),
            needed_from_user: Some("Review the encryption section".into()),
            estimated_minutes: Some(4),
        }
    }

    fn gate_copy(&self, gate: &GatedDecision) -> GateCopy {
        GateCopy {
            reason: gate
                .payload
                .get("reason")
                .and_then(|value| value.as_str())
                .unwrap_or("Security approval required.")
                .to_owned(),
            consequence: gate
                .payload
                .get("consequence")
                .and_then(|value| value.as_str())
                .unwrap_or("Without approval the recommendation remains blocked.")
                .to_owned(),
        }
    }

    fn idle_title(&self) -> String {
        "Good morning. Nothing needs you right now.".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use director_contracts::{BlockingState, DirectorPrompt};
    use helm_client::director::{ProjectionInputs, project};
    use helm_session_contracts::gate::{GateCondition, GateId, GatedDecision};

    #[test]
    fn quorum_presenter_projects_gate_copy_from_opaque_payload() {
        let gate = GatedDecision {
            gate_id: GateId::from_string("gate:procurement-security-approval"),
            condition: GateCondition::AnyParticipant,
            payload: serde_json::json!({
                "reason": "Security approval is required before publish.",
                "consequence": "Recommendation remains blocked."
            }),
            deadline: Some(1_718_000_000_000),
        };

        let snapshot = project(
            1844,
            ProjectionInputs {
                running_intent: None,
                pending_gate: Some(&gate),
            },
            &QuorumDomainPresenter,
        );

        assert_eq!(snapshot.version, 1844);
        assert!(matches!(
            snapshot.frame.blocking,
            BlockingState::BlocksFormation
        ));
        assert!(matches!(
            snapshot.frame.prompt,
            Some(DirectorPrompt::Gate(_))
        ));
    }
}
