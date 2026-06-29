//! Portfolio-wide offline queue state (M4.3–M4.4).
//!
//! [`QueueState`] labels where a [`QueuedCapture`] sits between local draft,
//! consent review, durable queue, server submission, and admission outcome.
//! Allowed transitions live in [`transitions`].

mod transitions;

pub use transitions::QueueTransitionError;

use crate::capture::{CapturePacket, CapturePacketError};

/// Lifecycle state for a capture packet in the offline queue pipeline.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueueState {
    /// Draft exists only on device; not yet at the consent review boundary.
    DraftLocal,
    /// User is reviewing / editing; no queue or submit yet.
    PendingConsent,
    /// Consented and durably queued for background submit.
    Queued,
    /// HTTP submission in flight to the server.
    Submitting,
    /// Server accepted the packet (admission receipt reconciled).
    Admitted,
    /// Server rejected the packet; may move to needs-review or abandoned.
    Rejected,
    /// Human review required before retry or abandon.
    NeedsReview,
    /// User or policy abandoned the packet; terminal.
    Abandoned,
}

impl QueueState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DraftLocal => "draft_local",
            Self::PendingConsent => "pending_consent",
            Self::Queued => "queued",
            Self::Submitting => "submitting",
            Self::Admitted => "admitted",
            Self::Rejected => "rejected",
            Self::NeedsReview => "needs_review",
            Self::Abandoned => "abandoned",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "draft_local" => Some(Self::DraftLocal),
            "pending_consent" => Some(Self::PendingConsent),
            "queued" => Some(Self::Queued),
            "submitting" => Some(Self::Submitting),
            "admitted" => Some(Self::Admitted),
            "rejected" => Some(Self::Rejected),
            "needs_review" => Some(Self::NeedsReview),
            "abandoned" => Some(Self::Abandoned),
            _ => None,
        }
    }

    /// Terminal states — no further automatic progression without explicit user action.
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Admitted | Self::Abandoned)
    }

    /// States where the packet may still be submitted or retried.
    pub fn is_active(self) -> bool {
        matches!(
            self,
            Self::DraftLocal
                | Self::PendingConsent
                | Self::Queued
                | Self::Submitting
                | Self::Rejected
                | Self::NeedsReview
        )
    }

    /// Whether the packet is eligible to enter or re-enter the submit path.
    pub fn permits_submit(self) -> bool {
        matches!(self, Self::Queued | Self::NeedsReview)
    }

    /// Map the legacy Quorum append-event sync label to queue state.
    pub fn from_legacy_sync_state(value: &str) -> Option<Self> {
        match value {
            "queued_for_sync" => Some(Self::Queued),
            _ => None,
        }
    }
}

/// Applying a queue operation to a capture entry failed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueuedCaptureError {
    Packet(CapturePacketError),
    Transition(QueueTransitionError),
}

impl std::fmt::Display for QueuedCaptureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Packet(error) => write!(f, "{error}"),
            Self::Transition(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for QueuedCaptureError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Packet(error) => Some(error),
            Self::Transition(error) => Some(error),
        }
    }
}

impl From<CapturePacketError> for QueuedCaptureError {
    fn from(error: CapturePacketError) -> Self {
        Self::Packet(error)
    }
}

impl From<QueueTransitionError> for QueuedCaptureError {
    fn from(error: QueueTransitionError) -> Self {
        Self::Transition(error)
    }
}

/// A capture packet bound to its queue lifecycle state.
#[derive(Clone, Debug, PartialEq)]
pub struct QueuedCapture {
    pub packet: CapturePacket,
    pub state: QueueState,
}

impl QueuedCapture {
    pub fn new_draft(packet: CapturePacket) -> Self {
        Self {
            packet,
            state: QueueState::DraftLocal,
        }
    }

    pub fn at_review(packet: CapturePacket) -> Self {
        Self {
            packet,
            state: QueueState::PendingConsent,
        }
    }

    pub fn transition_to(mut self, next: QueueState) -> Result<Self, QueuedCaptureError> {
        self.state = self.state.transition_to(next)?;
        Ok(self)
    }

    /// Move to `Queued` after consent validation and transition check.
    pub fn enqueue(self) -> Result<Self, QueuedCaptureError> {
        self.packet.ready_for_queue()?;
        self.transition_to(QueueState::Queued)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::{
        AppVersion, CaptureModality, ConsentRecord, DraftPayload, IdempotencyKey, SourceMetadata,
        WorkflowVersion,
    };
    use crate::consent::ConsentDecision;

    #[test]
    fn all_queue_states_have_stable_wire_labels() {
        let states = [
            QueueState::DraftLocal,
            QueueState::PendingConsent,
            QueueState::Queued,
            QueueState::Submitting,
            QueueState::Admitted,
            QueueState::Rejected,
            QueueState::NeedsReview,
            QueueState::Abandoned,
        ];
        for state in states {
            assert_eq!(QueueState::parse(state.as_str()), Some(state));
        }
    }

    #[test]
    fn parse_rejects_legacy_sync_state_string() {
        assert!(QueueState::parse("queued_for_sync").is_none());
    }

    #[test]
    fn terminal_and_active_flags() {
        assert!(QueueState::Admitted.is_terminal());
        assert!(QueueState::Abandoned.is_terminal());
        assert!(!QueueState::Queued.is_terminal());
        assert!(QueueState::Queued.is_active());
        assert!(!QueueState::Admitted.is_active());
    }

    #[test]
    fn enqueue_requires_permitting_consent() {
        let packet = fixture_packet(None);
        let entry = QueuedCapture::at_review(packet);
        assert!(matches!(
            entry.enqueue(),
            Err(QueuedCaptureError::Packet(
                CapturePacketError::MissingConsentRecord
            ))
        ));

        let packet = fixture_packet(Some(ConsentRecord {
            decision: ConsentDecision::Accepted,
            recorded_at: "2026-06-06T12:01:00Z".into(),
        }));
        let queued = QueuedCapture::at_review(packet).enqueue().expect("queues");
        assert_eq!(queued.state, QueueState::Queued);
    }

    #[test]
    fn queued_capture_rejects_illegal_transition() {
        let packet = fixture_packet(Some(ConsentRecord {
            decision: ConsentDecision::Accepted,
            recorded_at: "2026-06-06T12:01:00Z".into(),
        }));
        let at_review = QueuedCapture::at_review(packet);
        assert!(matches!(
            at_review.transition_to(QueueState::Submitting),
            Err(QueuedCaptureError::Transition(_))
        ));
    }

    fn fixture_packet(consent: Option<ConsentRecord>) -> CapturePacket {
        CapturePacket {
            app: AppVersion {
                app_slug: "quorum-sense".into(),
                client_version: None,
            },
            workflow: WorkflowVersion::parse("quorum.field_signal_capture.v1").unwrap(),
            idempotency_key: IdempotencyKey::for_draft("draft:inq:field-signal-v1"),
            modality: CaptureModality::VoiceTranscript,
            source: SourceMetadata {
                captured_at: None,
                participant_session_id: None,
                inquiry_thread_id: Some("inq_mobile_launch_risks".into()),
                offline: true,
                platform: None,
            },
            payload: DraftPayload {
                draft_id: "draft:inq:field-signal-v1".into(),
                raw_capture: "signal".into(),
                summary: "signal".into(),
                latent_need: "need".into(),
                contradiction: "tension".into(),
                confidence: 0.67,
            },
            consent,
        }
    }
}
