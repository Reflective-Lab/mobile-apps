//! Allowed [`QueueState`](super::QueueState) transitions (M4.4).

use super::QueueState;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QueueTransitionError {
    pub from: QueueState,
    pub to: QueueState,
}

impl std::fmt::Display for QueueTransitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "illegal queue transition: {} -> {}",
            self.from.as_str(),
            self.to.as_str()
        )
    }
}

impl std::error::Error for QueueTransitionError {}

impl QueueState {
    /// Whether `self` may move directly to `next` under the portfolio queue contract.
    pub fn allows_transition_to(self, next: QueueState) -> bool {
        use QueueState::{
            Abandoned, Admitted, DraftLocal, NeedsReview, PendingConsent, Queued, Rejected,
            Submitting,
        };

        if self == next {
            return true;
        }

        matches!(
            (self, next),
            (DraftLocal, PendingConsent)
                | (DraftLocal, Abandoned)
                | (PendingConsent, DraftLocal)
                | (PendingConsent, Queued)
                | (PendingConsent, Abandoned)
                | (Queued, Submitting)
                | (Queued, Abandoned)
                | (Submitting, Admitted)
                | (Submitting, Rejected)
                | (Submitting, NeedsReview)
                | (Submitting, Queued)
                | (Rejected, NeedsReview)
                | (Rejected, Abandoned)
                | (NeedsReview, Queued)
                | (NeedsReview, Abandoned)
        )
    }

    pub fn transition_to(self, next: QueueState) -> Result<QueueState, QueueTransitionError> {
        if self.allows_transition_to(next) {
            Ok(next)
        } else {
            Err(QueueTransitionError {
                from: self,
                to: next,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::QueueState;

    #[test]
    fn pending_consent_cannot_skip_to_submitting_or_admitted() {
        assert!(!QueueState::PendingConsent.allows_transition_to(QueueState::Submitting));
        assert!(!QueueState::PendingConsent.allows_transition_to(QueueState::Admitted));
    }

    #[test]
    fn rejected_cannot_reach_admitted_or_submit_without_review() {
        assert!(!QueueState::Rejected.allows_transition_to(QueueState::Admitted));
        assert!(!QueueState::Rejected.allows_transition_to(QueueState::Submitting));
        assert!(!QueueState::Rejected.allows_transition_to(QueueState::Queued));
        assert!(QueueState::Rejected.allows_transition_to(QueueState::NeedsReview));
    }

    #[test]
    fn terminal_states_allow_no_outbound_transitions() {
        for to in [
            QueueState::DraftLocal,
            QueueState::Queued,
            QueueState::Submitting,
        ] {
            assert!(!QueueState::Admitted.allows_transition_to(to));
            assert!(!QueueState::Abandoned.allows_transition_to(to));
        }
    }
}
