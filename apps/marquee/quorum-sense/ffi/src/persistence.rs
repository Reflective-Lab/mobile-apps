//! Offline queue persistence FFI (M4.6). Native stores JSON bytes; Rust owns
//! record shape and transition validation (ADR 0005).

use reflective_mobile_core::capture::{AppVersion, CapturePlatform, ConsentRecord, SourceMetadata};
use reflective_mobile_core::consent::ConsentDecision as DomainConsentDecision;
use reflective_mobile_core::persistence::{PersistedQueueRecord, PersistenceError};
use reflective_mobile_core::queue::QueueState as DomainQueueState;
use reflective_mobile_core::quorum::queue_capture_from_draft;

use crate::{FfiQuorumSignalDraft, QuorumError, from_ffi_draft, observed};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConsentDecision {
    Accepted,
    EditedAndAccepted,
    Rejected,
    SavedPrivate,
    Expired,
}

impl From<DomainConsentDecision> for ConsentDecision {
    fn from(decision: DomainConsentDecision) -> Self {
        match decision {
            DomainConsentDecision::Accepted => Self::Accepted,
            DomainConsentDecision::EditedAndAccepted => Self::EditedAndAccepted,
            DomainConsentDecision::Rejected => Self::Rejected,
            DomainConsentDecision::SavedPrivate => Self::SavedPrivate,
            DomainConsentDecision::Expired => Self::Expired,
        }
    }
}

impl From<ConsentDecision> for DomainConsentDecision {
    fn from(decision: ConsentDecision) -> Self {
        match decision {
            ConsentDecision::Accepted => Self::Accepted,
            ConsentDecision::EditedAndAccepted => Self::EditedAndAccepted,
            ConsentDecision::Rejected => Self::Rejected,
            ConsentDecision::SavedPrivate => Self::SavedPrivate,
            ConsentDecision::Expired => Self::Expired,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueueState {
    DraftLocal,
    PendingConsent,
    Queued,
    Submitting,
    Admitted,
    Rejected,
    NeedsReview,
    Abandoned,
}

impl From<DomainQueueState> for QueueState {
    fn from(state: DomainQueueState) -> Self {
        match state {
            DomainQueueState::DraftLocal => Self::DraftLocal,
            DomainQueueState::PendingConsent => Self::PendingConsent,
            DomainQueueState::Queued => Self::Queued,
            DomainQueueState::Submitting => Self::Submitting,
            DomainQueueState::Admitted => Self::Admitted,
            DomainQueueState::Rejected => Self::Rejected,
            DomainQueueState::NeedsReview => Self::NeedsReview,
            DomainQueueState::Abandoned => Self::Abandoned,
        }
    }
}

impl From<QueueState> for DomainQueueState {
    fn from(state: QueueState) -> Self {
        match state {
            QueueState::DraftLocal => Self::DraftLocal,
            QueueState::PendingConsent => Self::PendingConsent,
            QueueState::Queued => Self::Queued,
            QueueState::Submitting => Self::Submitting,
            QueueState::Admitted => Self::Admitted,
            QueueState::Rejected => Self::Rejected,
            QueueState::NeedsReview => Self::NeedsReview,
            QueueState::Abandoned => Self::Abandoned,
        }
    }
}

fn persistence_error(error: PersistenceError) -> QuorumError {
    QuorumError::InvalidPersistedRecord {
        detail: error.to_string(),
    }
}

fn parse_capture_platform(value: &str) -> Result<CapturePlatform, QuorumError> {
    match value {
        "ios" => Ok(CapturePlatform::Ios),
        "android" => Ok(CapturePlatform::Android),
        other => Err(QuorumError::UnsupportedPlatform {
            value: other.to_owned(),
        }),
    }
}

pub fn quorum_build_persisted_queue_record(
    draft: FfiQuorumSignalDraft,
    consent_decision: ConsentDecision,
    consent_recorded_at: String,
    updated_at: String,
    client_version: Option<String>,
    offline: bool,
    capture_platform: String,
) -> Result<String, QuorumError> {
    observed((|| {
        let domain_draft = from_ffi_draft(draft)?;
        let decision: DomainConsentDecision = consent_decision.into();
        if !decision.permits_queue() {
            return Err(QuorumError::ConsentDoesNotPermitQueue {
                decision: decision.as_str().to_owned(),
            });
        }
        let platform = parse_capture_platform(&capture_platform)?;
        let entry = queue_capture_from_draft(
            &domain_draft,
            AppVersion {
                app_slug: "quorum-sense".into(),
                client_version,
            },
            SourceMetadata {
                captured_at: Some(updated_at.clone()),
                participant_session_id: None,
                inquiry_thread_id: None,
                offline,
                platform: Some(platform),
            },
            ConsentRecord {
                decision,
                recorded_at: consent_recorded_at,
            },
        )
        .map_err(|error| QuorumError::InvalidPersistedRecord {
            detail: error.to_string(),
        })?;
        PersistedQueueRecord::encode(&entry, updated_at)
            .to_json()
            .map_err(persistence_error)
    })())
}

pub fn quorum_validate_persisted_queue_record(record_json: String) -> Result<(), QuorumError> {
    observed((|| {
        let record = PersistedQueueRecord::from_json(&record_json).map_err(persistence_error)?;
        record.decode().map_err(persistence_error)?;
        Ok(())
    })())
}

pub fn quorum_apply_queue_transition(
    record_json: String,
    next_state: QueueState,
    updated_at: String,
) -> Result<String, QuorumError> {
    observed((|| {
        let record = PersistedQueueRecord::from_json(&record_json).map_err(persistence_error)?;
        let from = record.queue_state.clone();
        let entry = record.decode().map_err(persistence_error)?;
        let next: DomainQueueState = next_state.into();
        let updated =
            entry
                .transition_to(next)
                .map_err(|_| QuorumError::IllegalQueueTransition {
                    from,
                    to: next.as_str().to_owned(),
                })?;
        PersistedQueueRecord::encode(&updated, updated_at)
            .to_json()
            .map_err(persistence_error)
    })())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SignalModality, quorum_draft_field_signal};

    const FIXTURE_INQUIRY_THREAD_ID: &str = "inq_mobile_launch_risks";
    const FIXTURE_RAW_CAPTURE: &str =
        "The sales team says rollout is fine, but support is seeing confusion in every pilot.";

    fn fixture_draft() -> FfiQuorumSignalDraft {
        quorum_draft_field_signal(
            FIXTURE_INQUIRY_THREAD_ID.to_owned(),
            SignalModality::VoiceTranscript,
            FIXTURE_RAW_CAPTURE.to_owned(),
        )
    }

    #[test]
    fn build_persisted_record_json_is_queued() {
        let json = quorum_build_persisted_queue_record(
            fixture_draft(),
            ConsentDecision::Accepted,
            "2026-06-06T12:01:00Z".into(),
            "2026-06-06T12:02:00Z".into(),
            Some("0.1.2".into()),
            true,
            "ios".into(),
        )
        .expect("build");

        assert!(json.contains("\"queue_state\":\"queued\""));
        assert!(json.contains("\"platform\":\"ios\""));
        quorum_validate_persisted_queue_record(json).expect("valid");
    }

    #[test]
    fn rejected_consent_does_not_build_record() {
        assert!(matches!(
            quorum_build_persisted_queue_record(
                fixture_draft(),
                ConsentDecision::Rejected,
                "2026-06-06T12:01:00Z".into(),
                "2026-06-06T12:02:00Z".into(),
                None,
                false,
                "ios".into(),
            ),
            Err(QuorumError::ConsentDoesNotPermitQueue { decision }) if decision == "rejected"
        ));
    }

    #[test]
    fn apply_transition_rejects_illegal_move() {
        let json = quorum_build_persisted_queue_record(
            fixture_draft(),
            ConsentDecision::Accepted,
            "2026-06-06T12:01:00Z".into(),
            "2026-06-06T12:02:00Z".into(),
            None,
            false,
            "ios".into(),
        )
        .expect("build");

        assert!(matches!(
            quorum_apply_queue_transition(
                json,
                QueueState::Admitted,
                "2026-06-06T12:03:00Z".into(),
            ),
            Err(QuorumError::IllegalQueueTransition { from, to })
                if from == "queued" && to == "admitted"
        ));
    }

    #[test]
    fn apply_queued_to_submitting_succeeds() {
        let json = quorum_build_persisted_queue_record(
            fixture_draft(),
            ConsentDecision::Accepted,
            "2026-06-06T12:01:00Z".into(),
            "2026-06-06T12:02:00Z".into(),
            None,
            false,
            "ios".into(),
        )
        .expect("build");

        let updated = quorum_apply_queue_transition(
            json,
            QueueState::Submitting,
            "2026-06-06T12:03:00Z".into(),
        )
        .expect("transition");

        assert!(updated.contains("\"queue_state\":\"submitting\""));
    }
}
