//! Portfolio-wide capture packet envelope for consent and offline queue (M2.2, M4.2).
//!
//! A [`CapturePacket`] bundles modality, source metadata, draft payload, consent
//! record, idempotency key, and app/workflow version — the unit native persistence
//! and sync will store without learning product-specific internals.

use crate::consent::ConsentDecision;

/// Client-safe idempotency token for deduplicating server submission.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct IdempotencyKey(String);

impl IdempotencyKey {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Stable v1 key: one queued submission per draft id.
    pub fn for_draft(draft_id: &str) -> Self {
        Self(format!("idempotency:{draft_id}"))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for IdempotencyKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// App identity + optional client semver (bundle / APK version).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppVersion {
    pub app_slug: String,
    pub client_version: Option<String>,
}

/// Workflow contract id and numeric version (from the `.vN` suffix).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowVersion {
    pub workflow_id: String,
    pub version: u32,
}

impl WorkflowVersion {
    pub fn parse(workflow_id: &str) -> Option<Self> {
        let suffix = workflow_id.rsplit_once(".v")?;
        let version = suffix.1.parse().ok()?;
        Some(Self {
            workflow_id: workflow_id.to_owned(),
            version,
        })
    }
}

/// How the signal was captured on device.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CaptureModality {
    Text,
    VoiceTranscript,
    ImageOcrText,
}

impl CaptureModality {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::VoiceTranscript => "voice_transcript",
            Self::ImageOcrText => "image_ocr_text",
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

/// Where and when capture happened on the device.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceMetadata {
    /// ISO-8601 timestamp when the raw signal was captured.
    pub captured_at: Option<String>,
    pub participant_session_id: Option<String>,
    pub inquiry_thread_id: Option<String>,
    pub offline: bool,
    pub platform: Option<CapturePlatform>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapturePlatform {
    Ios,
    Android,
}

impl CapturePlatform {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ios => "ios",
            Self::Android => "android",
        }
    }
}

/// Post-AI draft body carried into the queue envelope.
#[derive(Clone, Debug, PartialEq)]
pub struct DraftPayload {
    pub draft_id: String,
    pub raw_capture: String,
    pub summary: String,
    pub latent_need: String,
    pub contradiction: String,
    pub confidence: f32,
}

/// User consent applied at the review boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConsentRecord {
    pub decision: ConsentDecision,
    /// ISO-8601 timestamp when the user committed the decision.
    pub recorded_at: String,
}

/// Immutable capture unit for persistence and sync.
#[derive(Clone, Debug, PartialEq)]
pub struct CapturePacket {
    pub app: AppVersion,
    pub workflow: WorkflowVersion,
    pub idempotency_key: IdempotencyKey,
    pub modality: CaptureModality,
    pub source: SourceMetadata,
    pub payload: DraftPayload,
    pub consent: Option<ConsentRecord>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapturePacketError {
    MissingConsentRecord,
    ConsentDoesNotPermitQueue(ConsentDecision),
    InvalidWorkflowVersion,
}

impl std::fmt::Display for CapturePacketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingConsentRecord => f.write_str("capture packet has no consent record"),
            Self::ConsentDoesNotPermitQueue(decision) => {
                write!(f, "consent decision {:?} does not permit queue", decision)
            }
            Self::InvalidWorkflowVersion => f.write_str("workflow id missing .vN suffix"),
        }
    }
}

impl std::error::Error for CapturePacketError {}

impl CapturePacket {
    pub fn ready_for_queue(&self) -> Result<(), CapturePacketError> {
        match self.consent.as_ref().map(|record| record.decision) {
            Some(decision) if decision.permits_queue() => Ok(()),
            Some(decision) => Err(CapturePacketError::ConsentDoesNotPermitQueue(decision)),
            None => Err(CapturePacketError::MissingConsentRecord),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workflow_version_parses_v_suffix() {
        let workflow = WorkflowVersion::parse("quorum.field_signal_capture.v1").unwrap();
        assert_eq!(workflow.workflow_id, "quorum.field_signal_capture.v1");
        assert_eq!(workflow.version, 1);
        assert!(WorkflowVersion::parse("no-version-suffix").is_none());
    }

    #[test]
    fn idempotency_key_derives_from_draft_id() {
        assert_eq!(
            IdempotencyKey::for_draft("draft:inq:field-signal-v1").as_str(),
            "idempotency:draft:inq:field-signal-v1"
        );
    }

    #[test]
    fn ready_for_queue_requires_permitting_consent() {
        let mut packet = fixture_packet();
        packet.consent = None;
        assert!(matches!(
            packet.ready_for_queue(),
            Err(CapturePacketError::MissingConsentRecord)
        ));

        packet.consent = Some(ConsentRecord {
            decision: ConsentDecision::Rejected,
            recorded_at: "2026-06-06T12:01:00Z".into(),
        });
        assert!(matches!(
            packet.ready_for_queue(),
            Err(CapturePacketError::ConsentDoesNotPermitQueue(
                ConsentDecision::Rejected
            ))
        ));

        packet.consent = Some(ConsentRecord {
            decision: ConsentDecision::Accepted,
            recorded_at: "2026-06-06T12:01:00Z".into(),
        });
        assert!(packet.ready_for_queue().is_ok());
    }

    fn fixture_packet() -> CapturePacket {
        CapturePacket {
            app: AppVersion {
                app_slug: "quorum-sense".into(),
                client_version: Some("0.1.2".into()),
            },
            workflow: WorkflowVersion::parse("quorum.field_signal_capture.v1").unwrap(),
            idempotency_key: IdempotencyKey::for_draft(
                "draft:inq_mobile_launch_risks:field-signal-v1",
            ),
            modality: CaptureModality::VoiceTranscript,
            source: SourceMetadata {
                captured_at: Some("2026-06-06T12:00:00Z".into()),
                participant_session_id: Some("session_field_001".into()),
                inquiry_thread_id: Some("inq_mobile_launch_risks".into()),
                offline: true,
                platform: Some(CapturePlatform::Ios),
            },
            payload: DraftPayload {
                draft_id: "draft:inq_mobile_launch_risks:field-signal-v1".into(),
                raw_capture: "The sales team says rollout is fine.".into(),
                summary: "Sales reports readiness.".into(),
                latent_need: "needs earlier visibility into organizational ambiguity".into(),
                contradiction: "participants report alignment while surfacing unresolved tension"
                    .into(),
                confidence: 0.67,
            },
            consent: None,
        }
    }
}
