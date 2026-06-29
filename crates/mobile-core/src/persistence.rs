//! Local persistence contract for offline queue records (M4.5).
//!
//! [`PersistedQueueRecord`] is the **durable contract**: schema version, wire
//! labels, validation, and round-trip to [`QueuedCapture`]. JSON is the **record
//! encoding** (debuggable, testable, serde-native) — not the storage engine.
//!
//! Marquee governed apps (Quorum): native platforms store opaque bytes (Core Data,
//! Room, file, …) without learning product internals. Rust owns transitions;
//! native owns durability and BG scheduling (ADR 0005, M4.6+).

use crate::capture::{
    AppVersion, CaptureModality, CapturePacket, CapturePlatform, ConsentRecord, DraftPayload,
    IdempotencyKey, SourceMetadata, WorkflowVersion,
};
use crate::consent::ConsentDecision;
use crate::queue::{QueueState, QueuedCapture};
use serde::{Deserialize, Serialize};

/// Current persisted record schema. Bump when field semantics change.
pub const PERSISTENCE_RECORD_SCHEMA_VERSION: u32 = 1;

/// Stable persisted document (JSON encoding today; contract is schema-versioned fields).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PersistedQueueRecord {
    pub schema_version: u32,
    /// Primary key for native storage — the draft id.
    pub record_id: String,
    pub queue_state: String,
    pub app_slug: String,
    pub client_version: Option<String>,
    pub workflow_id: String,
    pub workflow_version: u32,
    pub idempotency_key: String,
    pub modality: String,
    pub captured_at: Option<String>,
    pub participant_session_id: Option<String>,
    pub inquiry_thread_id: Option<String>,
    pub offline: bool,
    pub platform: Option<String>,
    pub draft_id: String,
    pub raw_capture: String,
    pub summary: String,
    pub latent_need: String,
    pub contradiction: String,
    pub confidence: f32,
    pub consent_decision: Option<String>,
    pub consent_recorded_at: Option<String>,
    /// ISO-8601 timestamp of the last persistence write.
    pub updated_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PersistenceError {
    UnsupportedSchemaVersion { found: u32, expected: u32 },
    InvalidQueueState(String),
    InvalidModality(String),
    InvalidPlatform(String),
    InvalidConsentDecision(String),
    InvalidWorkflowVersion,
    RecordIdMismatch,
    ConfidenceOutOfRange,
    InvalidJson(String),
}

impl std::fmt::Display for PersistenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion { found, expected } => {
                write!(
                    f,
                    "unsupported persistence schema version {found} (expected {expected})"
                )
            }
            Self::InvalidQueueState(value) => write!(f, "invalid queue_state: {value}"),
            Self::InvalidModality(value) => write!(f, "invalid modality: {value}"),
            Self::InvalidPlatform(value) => write!(f, "invalid platform: {value}"),
            Self::InvalidConsentDecision(value) => {
                write!(f, "invalid consent_decision: {value}")
            }
            Self::InvalidWorkflowVersion => {
                f.write_str("workflow_id does not match workflow_version suffix")
            }
            Self::RecordIdMismatch => f.write_str("record_id must match draft_id"),
            Self::ConfidenceOutOfRange => f.write_str("confidence outside 0.0..=1.0"),
            Self::InvalidJson(message) => write!(f, "invalid persistence JSON: {message}"),
        }
    }
}

impl std::error::Error for PersistenceError {}

impl From<serde_json::Error> for PersistenceError {
    fn from(error: serde_json::Error) -> Self {
        Self::InvalidJson(error.to_string())
    }
}

impl PersistedQueueRecord {
    pub fn encode(entry: &QueuedCapture, updated_at: impl Into<String>) -> Self {
        let packet = &entry.packet;
        let consent = packet.consent.as_ref();

        Self {
            schema_version: PERSISTENCE_RECORD_SCHEMA_VERSION,
            record_id: packet.payload.draft_id.clone(),
            queue_state: entry.state.as_str().to_owned(),
            app_slug: packet.app.app_slug.clone(),
            client_version: packet.app.client_version.clone(),
            workflow_id: packet.workflow.workflow_id.clone(),
            workflow_version: packet.workflow.version,
            idempotency_key: packet.idempotency_key.as_str().to_owned(),
            modality: packet.modality.as_str().to_owned(),
            captured_at: packet.source.captured_at.clone(),
            participant_session_id: packet.source.participant_session_id.clone(),
            inquiry_thread_id: packet.source.inquiry_thread_id.clone(),
            offline: packet.source.offline,
            platform: packet.source.platform.map(|p| p.as_str().to_owned()),
            draft_id: packet.payload.draft_id.clone(),
            raw_capture: packet.payload.raw_capture.clone(),
            summary: packet.payload.summary.clone(),
            latent_need: packet.payload.latent_need.clone(),
            contradiction: packet.payload.contradiction.clone(),
            confidence: packet.payload.confidence,
            consent_decision: consent.map(|record| record.decision.as_str().to_owned()),
            consent_recorded_at: consent.map(|record| record.recorded_at.clone()),
            updated_at: updated_at.into(),
        }
    }

    pub fn decode(self) -> Result<QueuedCapture, PersistenceError> {
        if self.schema_version != PERSISTENCE_RECORD_SCHEMA_VERSION {
            return Err(PersistenceError::UnsupportedSchemaVersion {
                found: self.schema_version,
                expected: PERSISTENCE_RECORD_SCHEMA_VERSION,
            });
        }

        if !self.confidence.is_finite() || !(0.0..=1.0).contains(&self.confidence) {
            return Err(PersistenceError::ConfidenceOutOfRange);
        }

        let queue_state = QueueState::parse(&self.queue_state)
            .ok_or_else(|| PersistenceError::InvalidQueueState(self.queue_state.clone()))?;

        let modality = CaptureModality::parse(&self.modality)
            .ok_or_else(|| PersistenceError::InvalidModality(self.modality.clone()))?;

        let platform = match self.platform.as_deref() {
            None => None,
            Some("ios") => Some(CapturePlatform::Ios),
            Some("android") => Some(CapturePlatform::Android),
            Some(other) => return Err(PersistenceError::InvalidPlatform(other.to_owned())),
        };

        let consent = match (
            self.consent_decision.as_deref(),
            self.consent_recorded_at.as_deref(),
        ) {
            (None, None) => None,
            (Some(decision), Some(recorded_at)) => Some(ConsentRecord {
                decision: ConsentDecision::parse(decision)
                    .ok_or_else(|| PersistenceError::InvalidConsentDecision(decision.to_owned()))?,
                recorded_at: recorded_at.to_owned(),
            }),
            _ => {
                return Err(PersistenceError::InvalidConsentDecision(
                    "consent_decision and consent_recorded_at must both be set or both absent"
                        .into(),
                ));
            }
        };

        let workflow = WorkflowVersion {
            workflow_id: self.workflow_id.clone(),
            version: self.workflow_version,
        };
        if WorkflowVersion::parse(&workflow.workflow_id)
            .is_none_or(|parsed| parsed.version != workflow.version)
        {
            return Err(PersistenceError::InvalidWorkflowVersion);
        }

        let packet = CapturePacket {
            app: AppVersion {
                app_slug: self.app_slug,
                client_version: self.client_version,
            },
            workflow,
            idempotency_key: IdempotencyKey::new(self.idempotency_key),
            modality,
            source: SourceMetadata {
                captured_at: self.captured_at,
                participant_session_id: self.participant_session_id,
                inquiry_thread_id: self.inquiry_thread_id,
                offline: self.offline,
                platform,
            },
            payload: DraftPayload {
                draft_id: self.draft_id,
                raw_capture: self.raw_capture,
                summary: self.summary,
                latent_need: self.latent_need,
                contradiction: self.contradiction,
                confidence: self.confidence,
            },
            consent,
        };

        if packet.payload.draft_id != self.record_id {
            return Err(PersistenceError::RecordIdMismatch);
        }

        Ok(QueuedCapture {
            packet,
            state: queue_state,
        })
    }

    pub fn to_json(&self) -> Result<String, PersistenceError> {
        Ok(serde_json::to_string(self)?)
    }

    pub fn from_json(json: &str) -> Result<Self, PersistenceError> {
        serde_json::from_str(json).map_err(PersistenceError::from)
    }
}

/// Encode, serialize, parse, and decode — the native storage round-trip contract.
pub fn persistence_round_trip(
    entry: &QueuedCapture,
    updated_at: &str,
) -> Result<QueuedCapture, PersistenceError> {
    let json = PersistedQueueRecord::encode(entry, updated_at).to_json()?;
    PersistedQueueRecord::from_json(&json)?.decode()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::{
        AppVersion, CaptureModality, DraftPayload, IdempotencyKey, SourceMetadata, WorkflowVersion,
    };
    use crate::consent::ConsentDecision;

    #[test]
    fn encode_decode_round_trips_queued_capture() {
        let entry = QueuedCapture {
            packet: CapturePacket {
                app: AppVersion {
                    app_slug: "quorum-sense".into(),
                    client_version: Some("0.1.2".into()),
                },
                workflow: WorkflowVersion::parse("quorum.field_signal_capture.v1").unwrap(),
                idempotency_key: IdempotencyKey::for_draft("draft:inq:field-signal-v1"),
                modality: CaptureModality::VoiceTranscript,
                source: SourceMetadata {
                    captured_at: Some("2026-06-06T12:00:00Z".into()),
                    participant_session_id: Some("session_field_001".into()),
                    inquiry_thread_id: Some("inq_mobile_launch_risks".into()),
                    offline: true,
                    platform: Some(CapturePlatform::Ios),
                },
                payload: DraftPayload {
                    draft_id: "draft:inq:field-signal-v1".into(),
                    raw_capture: "signal".into(),
                    summary: "summary".into(),
                    latent_need: "need".into(),
                    contradiction: "tension".into(),
                    confidence: 0.67,
                },
                consent: Some(ConsentRecord {
                    decision: ConsentDecision::Accepted,
                    recorded_at: "2026-06-06T12:01:00Z".into(),
                }),
            },
            state: QueueState::Queued,
        };

        let restored = persistence_round_trip(&entry, "2026-06-06T12:02:00Z").expect("round trip");
        assert_eq!(restored, entry);
    }

    #[test]
    fn rejects_unsupported_schema_version() {
        let mut record = PersistedQueueRecord::encode(
            &QueuedCapture::new_draft(CapturePacket {
                app: AppVersion {
                    app_slug: "quorum-sense".into(),
                    client_version: None,
                },
                workflow: WorkflowVersion::parse("quorum.field_signal_capture.v1").unwrap(),
                idempotency_key: IdempotencyKey::for_draft("draft:x"),
                modality: CaptureModality::Text,
                source: SourceMetadata {
                    captured_at: None,
                    participant_session_id: None,
                    inquiry_thread_id: None,
                    offline: false,
                    platform: None,
                },
                payload: DraftPayload {
                    draft_id: "draft:x".into(),
                    raw_capture: "x".into(),
                    summary: "x".into(),
                    latent_need: "x".into(),
                    contradiction: "x".into(),
                    confidence: 0.5,
                },
                consent: None,
            }),
            "2026-06-06T12:00:00Z",
        );
        record.schema_version = 99;
        assert!(matches!(
            record.decode(),
            Err(PersistenceError::UnsupportedSchemaVersion {
                found: 99,
                expected: 1
            })
        ));
    }
}
