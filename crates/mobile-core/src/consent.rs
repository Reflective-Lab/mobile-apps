//! Portfolio-wide consent decisions at the capture review boundary.
//!
//! `ConsentDecision` is the user's explicit outcome (accept, edit, reject, …).
//! Product drafts carry a separate lifecycle flag — e.g. `quorum::ConsentState`
//! (`Pending` / `Consented`) — that tracks whether a packet cleared review.

/// Explicit user consent outcome. Not a boolean gate and not a free-form string.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConsentDecision {
    /// Submit the draft unchanged.
    Accepted,
    /// User edited the draft, then accepted for queue/submit.
    EditedAndAccepted,
    /// User rejected — discard; must not enter the offline queue.
    Rejected,
    /// Keep locally only; distinct from "submit later" or offline queue.
    SavedPrivate,
    /// Review window closed before the user acted (session timeout, expiry).
    Expired,
}

impl ConsentDecision {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::EditedAndAccepted => "edited_and_accepted",
            Self::Rejected => "rejected",
            Self::SavedPrivate => "saved_private",
            Self::Expired => "expired",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "accepted" => Some(Self::Accepted),
            "edited_and_accepted" => Some(Self::EditedAndAccepted),
            "rejected" => Some(Self::Rejected),
            "saved_private" => Some(Self::SavedPrivate),
            "expired" => Some(Self::Expired),
            _ => None,
        }
    }

    /// Whether this decision authorizes moving the draft into the offline queue.
    pub fn permits_queue(self) -> bool {
        matches!(self, Self::Accepted | Self::EditedAndAccepted)
    }

    /// Whether the user changed the draft before consenting.
    pub fn user_edited(self) -> bool {
        matches!(self, Self::EditedAndAccepted)
    }
}

/// Applying a consent decision to the queue pipeline failed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConsentApplyError {
    DoesNotPermitQueue(ConsentDecision),
}

impl std::fmt::Display for ConsentApplyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DoesNotPermitQueue(decision) => {
                write!(f, "consent decision {:?} does not permit queue", decision)
            }
        }
    }
}

impl std::error::Error for ConsentApplyError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_variants_have_stable_wire_labels() {
        let variants = [
            ConsentDecision::Accepted,
            ConsentDecision::EditedAndAccepted,
            ConsentDecision::Rejected,
            ConsentDecision::SavedPrivate,
            ConsentDecision::Expired,
        ];
        for decision in variants {
            assert_eq!(ConsentDecision::parse(decision.as_str()), Some(decision));
        }
    }

    #[test]
    fn parse_rejects_unknown_and_legacy_booleans() {
        assert!(ConsentDecision::parse("true").is_none());
        assert!(ConsentDecision::parse("false").is_none());
        assert!(ConsentDecision::parse("consented").is_none());
        assert!(ConsentDecision::parse("ACCEPTED").is_none());
    }

    #[test]
    fn only_accept_variants_permit_queue() {
        assert!(ConsentDecision::Accepted.permits_queue());
        assert!(ConsentDecision::EditedAndAccepted.permits_queue());
        assert!(!ConsentDecision::Rejected.permits_queue());
        assert!(!ConsentDecision::SavedPrivate.permits_queue());
        assert!(!ConsentDecision::Expired.permits_queue());
    }

    #[test]
    fn edited_flag_matches_edited_and_accepted_only() {
        assert!(!ConsentDecision::Accepted.user_edited());
        assert!(ConsentDecision::EditedAndAccepted.user_edited());
        assert!(!ConsentDecision::Rejected.user_edited());
    }
}
