use crate::capture::{
    AppVersion, CaptureModality, CapturePacket, CapturePacketError, ConsentRecord, DraftPayload,
    IdempotencyKey, SourceMetadata, WorkflowVersion,
};
use crate::consent::{ConsentApplyError, ConsentDecision};
use crate::queue::{QueuedCapture, QueuedCaptureError};

pub mod director_presenter;

pub const FIELD_SIGNAL_CAPTURE_WORKFLOW_ID: &str = "quorum.field_signal_capture.v1";
pub const FIELD_SIGNAL_CAPTURE_FIXTURE_JSON: &str =
    include_str!("../../../apps/marquee/quorum-sense/fixtures/field-signal-capture.v1.json");

/// String identifier newtypes. Inside the domain these are opaque ids, never
/// interchangeable with each other or with free-form strings. The raw <-> id
/// mapping happens at the boundary (the FFI layer), not here.
macro_rules! string_id {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(Clone, Debug, Eq, PartialEq, Hash)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

string_id!(
    /// Identifies a Quorum workflow contract (e.g. field-signal-capture v1).
    WorkflowId
);
string_id!(
    /// Identifies a single drafted signal.
    DraftId
);
string_id!(
    /// Identifies the inquiry thread a signal belongs to.
    InquiryThreadId
);

impl WorkflowId {
    /// The one workflow this module implements today.
    pub fn field_signal_capture() -> Self {
        Self(FIELD_SIGNAL_CAPTURE_WORKFLOW_ID.to_owned())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SignalModality {
    Text,
    VoiceTranscript,
    ImageOcrText,
}

impl SignalModality {
    pub fn as_str(self) -> &'static str {
        match self {
            SignalModality::Text => "text",
            SignalModality::VoiceTranscript => "voice_transcript",
            SignalModality::ImageOcrText => "image_ocr_text",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "text" => Some(Self::Text),
            "voice_transcript" => Some(Self::VoiceTranscript),
            "image_ocr_text" => Some(Self::ImageOcrText),
            _ => None,
        }
    }
}

/// Whether a captured signal has cleared consent for sync.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConsentState {
    Pending,
    Consented,
}

impl ConsentState {
    pub fn as_str(self) -> &'static str {
        match self {
            ConsentState::Pending => "pending",
            ConsentState::Consented => "consented",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "pending" => Some(Self::Pending),
            "consented" => Some(Self::Consented),
            _ => None,
        }
    }
}

/// The kind of event emitted when a draft is appended to the workflow log.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AppendEventType {
    SignalDraftConsented,
}

impl AppendEventType {
    pub fn as_str(self) -> &'static str {
        match self {
            AppendEventType::SignalDraftConsented => "SignalDraftConsented",
        }
    }
}

/// Where an appended event sits in the sync pipeline.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SyncState {
    QueuedForSync,
}

impl SyncState {
    pub fn as_str(self) -> &'static str {
        match self {
            SyncState::QueuedForSync => "queued_for_sync",
        }
    }
}

/// A model confidence score, constrained to `0.0..=1.0` at construction so the
/// rest of the domain never has to re-check the range.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Confidence(f32);

impl Confidence {
    pub fn new(value: f32) -> Option<Self> {
        if value.is_finite() && (0.0..=1.0).contains(&value) {
            Some(Self(value))
        } else {
            None
        }
    }

    pub fn value(self) -> f32 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct QuorumSignalDraft {
    pub workflow_id: WorkflowId,
    pub draft_id: DraftId,
    pub inquiry_thread_id: InquiryThreadId,
    pub modality: SignalModality,
    pub raw_capture: String,
    pub summary: String,
    pub latent_need: String,
    pub contradiction: String,
    pub confidence: Confidence,
    pub consent_state: ConsentState,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuorumAppendEvent {
    pub workflow_id: WorkflowId,
    pub event_type: AppendEventType,
    pub draft_id: DraftId,
    pub inquiry_thread_id: InquiryThreadId,
    pub sync_state: SyncState,
}

pub fn draft_field_signal(
    inquiry_thread_id: &str,
    modality: SignalModality,
    raw_capture: &str,
) -> QuorumSignalDraft {
    let summary = summarize_for_fixture(raw_capture);

    QuorumSignalDraft {
        workflow_id: WorkflowId::field_signal_capture(),
        draft_id: DraftId::new(format!("draft:{inquiry_thread_id}:field-signal-v1")),
        inquiry_thread_id: InquiryThreadId::new(inquiry_thread_id),
        modality,
        raw_capture: raw_capture.to_owned(),
        summary,
        latent_need: "needs earlier visibility into organizational ambiguity".to_owned(),
        contradiction: "participants report alignment while surfacing unresolved tension"
            .to_owned(),
        // Known-valid literal: constructed directly because the private field is
        // in scope here; external callers must go through `Confidence::new`.
        confidence: Confidence(0.67),
        consent_state: ConsentState::Pending,
    }
}

/// Queue a draft after explicit consent. Only `Accepted` and
/// `EditedAndAccepted` may enter the offline queue.
pub fn append_after_consent(
    draft: &QuorumSignalDraft,
    decision: ConsentDecision,
) -> Result<QuorumAppendEvent, ConsentApplyError> {
    if !decision.permits_queue() {
        return Err(ConsentApplyError::DoesNotPermitQueue(decision));
    }
    Ok(queue_consented_event(draft))
}

/// Shorthand for [`append_after_consent`] with [`ConsentDecision::Accepted`].
pub fn append_consented_signal(draft: &QuorumSignalDraft) -> QuorumAppendEvent {
    queue_consented_event(draft)
}

fn queue_consented_event(draft: &QuorumSignalDraft) -> QuorumAppendEvent {
    QuorumAppendEvent {
        workflow_id: draft.workflow_id.clone(),
        event_type: AppendEventType::SignalDraftConsented,
        draft_id: draft.draft_id.clone(),
        inquiry_thread_id: draft.inquiry_thread_id.clone(),
        sync_state: SyncState::QueuedForSync,
    }
}

/// Build a portfolio [`CapturePacket`] from a Quorum field-signal draft.
pub fn capture_packet_from_draft(
    draft: &QuorumSignalDraft,
    app: AppVersion,
    source: SourceMetadata,
    consent: Option<ConsentRecord>,
) -> Result<CapturePacket, CapturePacketError> {
    let workflow = WorkflowVersion::parse(draft.workflow_id.as_str())
        .ok_or(CapturePacketError::InvalidWorkflowVersion)?;

    Ok(CapturePacket {
        app,
        workflow,
        idempotency_key: IdempotencyKey::for_draft(draft.draft_id.as_str()),
        modality: capture_modality_from_signal(draft.modality),
        source: SourceMetadata {
            inquiry_thread_id: Some(draft.inquiry_thread_id.as_str().to_owned()),
            ..source
        },
        payload: DraftPayload {
            draft_id: draft.draft_id.as_str().to_owned(),
            raw_capture: draft.raw_capture.clone(),
            summary: draft.summary.clone(),
            latent_need: draft.latent_need.clone(),
            contradiction: draft.contradiction.clone(),
            confidence: draft.confidence.value(),
        },
        consent,
    })
}

/// Queue from a validated capture packet (consent + workflow must match Quorum).
pub fn append_from_capture_packet(
    packet: &CapturePacket,
) -> Result<QuorumAppendEvent, CapturePacketError> {
    packet.ready_for_queue()?;

    if packet.workflow.workflow_id != FIELD_SIGNAL_CAPTURE_WORKFLOW_ID {
        return Err(CapturePacketError::InvalidWorkflowVersion);
    }

    Ok(QuorumAppendEvent {
        workflow_id: WorkflowId::new(packet.workflow.workflow_id.clone()),
        event_type: AppendEventType::SignalDraftConsented,
        draft_id: DraftId::new(packet.payload.draft_id.clone()),
        inquiry_thread_id: InquiryThreadId::new(
            packet.source.inquiry_thread_id.clone().unwrap_or_default(),
        ),
        sync_state: SyncState::QueuedForSync,
    })
}

/// Build and enqueue a [`QueuedCapture`] after explicit consent.
pub fn queue_capture_from_draft(
    draft: &QuorumSignalDraft,
    app: AppVersion,
    source: SourceMetadata,
    consent: ConsentRecord,
) -> Result<QueuedCapture, QueuedCaptureError> {
    let packet = capture_packet_from_draft(draft, app, source, Some(consent))?;
    QueuedCapture::at_review(packet).enqueue()
}

fn capture_modality_from_signal(modality: SignalModality) -> CaptureModality {
    match modality {
        SignalModality::Text => CaptureModality::Text,
        SignalModality::VoiceTranscript => CaptureModality::VoiceTranscript,
        SignalModality::ImageOcrText => CaptureModality::ImageOcrText,
    }
}

fn summarize_for_fixture(raw_capture: &str) -> String {
    let trimmed = raw_capture.trim();
    if trimmed.is_empty() {
        return "Empty capture needs participant clarification".to_owned();
    }

    trimmed.chars().take(96).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::CapturePlatform;

    #[test]
    fn fixture_declares_first_quorum_workflow() {
        assert!(FIELD_SIGNAL_CAPTURE_FIXTURE_JSON.contains(FIELD_SIGNAL_CAPTURE_WORKFLOW_ID));
        assert!(FIELD_SIGNAL_CAPTURE_FIXTURE_JSON.contains("\"consent_state\": \"pending\""));
        assert!(FIELD_SIGNAL_CAPTURE_FIXTURE_JSON.contains("\"sync_state\": \"queued_for_sync\""));
    }

    #[test]
    fn draft_requires_consent_before_append() {
        let draft = draft_field_signal(
            "inq_mobile_launch_risks",
            SignalModality::VoiceTranscript,
            "The sales team says rollout is fine, but support is seeing confusion in every pilot.",
        );

        assert_eq!(draft.workflow_id.as_str(), FIELD_SIGNAL_CAPTURE_WORKFLOW_ID);
        assert_eq!(draft.modality, SignalModality::VoiceTranscript);
        assert_eq!(draft.confidence.value(), 0.67);
        assert_eq!(draft.consent_state, ConsentState::Pending);

        let event = append_consented_signal(&draft);
        assert_eq!(event.event_type, AppendEventType::SignalDraftConsented);
        assert_eq!(event.sync_state, SyncState::QueuedForSync);

        let edited = append_after_consent(&draft, ConsentDecision::EditedAndAccepted)
            .expect("edited accept queues");
        assert_eq!(edited.draft_id, draft.draft_id);

        assert!(matches!(
            append_after_consent(&draft, ConsentDecision::Rejected),
            Err(ConsentApplyError::DoesNotPermitQueue(
                ConsentDecision::Rejected
            ))
        ));
        assert!(matches!(
            append_after_consent(&draft, ConsentDecision::SavedPrivate),
            Err(ConsentApplyError::DoesNotPermitQueue(
                ConsentDecision::SavedPrivate
            ))
        ));
    }

    #[test]
    fn confidence_rejects_out_of_range() {
        assert!(Confidence::new(0.0).is_some());
        assert!(Confidence::new(1.0).is_some());
        assert!(Confidence::new(-0.1).is_none());
        assert!(Confidence::new(1.1).is_none());
        assert!(Confidence::new(f32::NAN).is_none());
    }

    #[test]
    fn capture_packet_carries_fixture_fields() {
        let draft = draft_field_signal(
            "inq_mobile_launch_risks",
            SignalModality::VoiceTranscript,
            "The sales team says rollout is fine, but support is seeing confusion in every pilot.",
        );
        let source = SourceMetadata {
            captured_at: Some("2026-06-06T12:00:00Z".into()),
            participant_session_id: Some("session_field_001".into()),
            inquiry_thread_id: None,
            offline: true,
            platform: Some(CapturePlatform::Ios),
        };
        let consent = ConsentRecord {
            decision: ConsentDecision::Accepted,
            recorded_at: "2026-06-06T12:01:00Z".into(),
        };

        let packet = capture_packet_from_draft(
            &draft,
            AppVersion {
                app_slug: "quorum-sense".into(),
                client_version: Some("0.1.2".into()),
            },
            source,
            Some(consent),
        )
        .expect("packet builds");

        assert_eq!(packet.app.app_slug, "quorum-sense");
        assert_eq!(packet.workflow.version, 1);
        assert_eq!(
            packet.idempotency_key.as_str(),
            "idempotency:draft:inq_mobile_launch_risks:field-signal-v1"
        );
        assert_eq!(packet.modality, CaptureModality::VoiceTranscript);
        assert_eq!(
            packet.source.inquiry_thread_id.as_deref(),
            Some("inq_mobile_launch_risks")
        );
        assert_eq!(packet.payload.confidence, 0.67);

        let event = append_from_capture_packet(&packet).expect("packet queues");
        assert_eq!(event.draft_id.as_str(), draft.draft_id.as_str());
    }

    #[test]
    fn modality_and_consent_round_trip() {
        for value in ["text", "voice_transcript", "image_ocr_text"] {
            assert_eq!(SignalModality::parse(value).unwrap().as_str(), value);
        }
        assert!(SignalModality::parse("hologram").is_none());

        for value in ["pending", "consented"] {
            assert_eq!(ConsentState::parse(value).unwrap().as_str(), value);
        }
        assert!(ConsentState::parse("revoked").is_none());
    }
}
