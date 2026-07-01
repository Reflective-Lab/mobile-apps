//! Canonical Quorum capture HTTP submission and admission reconciliation (M4.8–M4.9).
//!
//! Mobile submits through the Quorum API path — not a mobile-specific transport.
//! When the server route is unavailable locally, callers reconcile receipts from
//! tests or retry after `rollback_submission`.

use crate::capture::{CapturePacket, CapturePacketError, ConsentRecord};
use crate::consent::ConsentDecision;
use crate::director::DirectorApiConfig;
use crate::persistence::{PersistedQueueRecord, PersistenceError};
use crate::queue::{QueueState, QueuedCapture, QueuedCaptureError};
use serde::{Deserialize, Serialize};

/// Canonical Quorum field-signal submit route (marquee-apps/quorum-sense server).
pub const CAPTURE_SUBMIT_PATH: &str = "/api/capture/submit";

/// Same base URL + bearer shape as the director boundary.
pub type CaptureApiConfig = DirectorApiConfig;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdmissionOutcome {
    Admitted,
    Rejected,
    NeedsReview,
    DuplicateAdmitted,
}

impl AdmissionOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Admitted => "admitted",
            Self::Rejected => "rejected",
            Self::NeedsReview => "needs_review",
            Self::DuplicateAdmitted => "duplicate_admitted",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "admitted" => Some(Self::Admitted),
            "rejected" => Some(Self::Rejected),
            "needs_review" => Some(Self::NeedsReview),
            "duplicate_admitted" => Some(Self::DuplicateAdmitted),
            _ => None,
        }
    }
}

/// Server admission receipt — local queue state advances only through reconciliation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdmissionReceipt {
    pub idempotency_key: String,
    pub draft_id: String,
    pub outcome: AdmissionOutcome,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_receipt_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Wire body for `POST /api/capture/submit`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CaptureSubmitRequest {
    pub idempotency_key: String,
    pub workflow_id: String,
    pub workflow_version: u32,
    pub app_slug: String,
    pub client_version: Option<String>,
    pub inquiry_thread_id: Option<String>,
    pub participant_session_id: Option<String>,
    pub captured_at: Option<String>,
    pub modality: String,
    pub offline: bool,
    pub platform: Option<String>,
    pub draft_id: String,
    pub raw_capture: String,
    pub summary: String,
    pub latent_need: String,
    pub contradiction: String,
    pub confidence: f32,
    pub consent_decision: String,
    pub consent_recorded_at: String,
}

#[derive(Debug, thiserror::Error)]
pub enum CaptureSubmitError {
    #[error("capture packet not ready for submit: {0}")]
    Packet(#[from] CapturePacketError),
    #[error("queue entry invalid for submit: {0}")]
    Queue(#[from] QueuedCaptureError),
    #[error("missing consent record on capture packet")]
    MissingConsent,
    #[error("capture API not configured")]
    ApiNotConfigured,
    #[error("HTTP {status}: {body}")]
    HttpError { status: u16, body: String },
    #[error("network error: {0}")]
    Transport(#[from] ureq::Error),
    #[error("invalid admission receipt JSON: {0}")]
    InvalidReceipt(#[from] serde_json::Error),
    #[error("admission receipt idempotency_key mismatch")]
    IdempotencyMismatch,
    #[error("admission receipt draft_id mismatch")]
    DraftIdMismatch,
}

#[derive(Debug, thiserror::Error)]
pub enum QueueSubmitError {
    #[error("persistence error: {0}")]
    Persistence(#[from] PersistenceError),
    #[error("submit error: {0}")]
    Submit(#[from] CaptureSubmitError),
}

#[must_use]
pub fn capture_submit_url(config: &CaptureApiConfig) -> String {
    format!(
        "{}{CAPTURE_SUBMIT_PATH}",
        config.base_url.trim_end_matches('/')
    )
}

/// Build the canonical submit body from a queued capture entry.
pub fn build_submit_request(
    entry: &QueuedCapture,
) -> Result<CaptureSubmitRequest, CaptureSubmitError> {
    entry.packet.ready_for_queue()?;
    let consent = entry
        .packet
        .consent
        .as_ref()
        .ok_or(CaptureSubmitError::MissingConsent)?;
    let packet = &entry.packet;

    Ok(CaptureSubmitRequest {
        idempotency_key: packet.idempotency_key.as_str().to_owned(),
        workflow_id: packet.workflow.workflow_id.clone(),
        workflow_version: packet.workflow.version,
        app_slug: packet.app.app_slug.clone(),
        client_version: packet.app.client_version.clone(),
        inquiry_thread_id: packet.source.inquiry_thread_id.clone(),
        participant_session_id: packet.source.participant_session_id.clone(),
        captured_at: packet.source.captured_at.clone(),
        modality: packet.modality.as_str().to_owned(),
        offline: packet.source.offline,
        platform: packet.source.platform.map(|p| p.as_str().to_owned()),
        draft_id: packet.payload.draft_id.clone(),
        raw_capture: packet.payload.raw_capture.clone(),
        summary: packet.payload.summary.clone(),
        latent_need: packet.payload.latent_need.clone(),
        contradiction: packet.payload.contradiction.clone(),
        confidence: packet.payload.confidence,
        consent_decision: consent.decision.as_str().to_owned(),
        consent_recorded_at: consent.recorded_at.clone(),
    })
}

/// Build the canonical submit body JSON from a persisted queued entry.
pub fn build_submit_request_json(entry: &QueuedCapture) -> Result<String, CaptureSubmitError> {
    let request = build_submit_request(entry)?;
    serde_json::to_string(&request).map_err(CaptureSubmitError::InvalidReceipt)
}

/// Move an eligible entry into `submitting` before HTTP dispatch.
pub fn begin_submission(entry: QueuedCapture) -> Result<QueuedCapture, QueuedCaptureError> {
    if !entry.state.permits_submit() {
        return Err(QueuedCaptureError::Transition(
            crate::queue::QueueTransitionError {
                from: entry.state,
                to: QueueState::Submitting,
            },
        ));
    }
    entry.packet.ready_for_queue()?;
    entry.transition_to(QueueState::Submitting)
}

/// Return a failed in-flight submit to the durable queue for retry.
pub fn rollback_submission(entry: QueuedCapture) -> Result<QueuedCapture, QueuedCaptureError> {
    entry.transition_to(QueueState::Queued)
}

/// Apply a server admission receipt to a `submitting` entry.
pub fn reconcile_admission_receipt(
    entry: QueuedCapture,
    receipt: &AdmissionReceipt,
) -> Result<QueuedCapture, QueuedCaptureError> {
    if entry.state != QueueState::Submitting {
        return Err(QueuedCaptureError::Transition(
            crate::queue::QueueTransitionError {
                from: entry.state,
                to: QueueState::Admitted,
            },
        ));
    }

    if receipt.idempotency_key != entry.packet.idempotency_key.as_str() {
        return Err(QueuedCaptureError::Transition(
            crate::queue::QueueTransitionError {
                from: entry.state,
                to: QueueState::Admitted,
            },
        ));
    }
    if receipt.draft_id != entry.packet.payload.draft_id {
        return Err(QueuedCaptureError::Transition(
            crate::queue::QueueTransitionError {
                from: entry.state,
                to: QueueState::Admitted,
            },
        ));
    }

    let next = match receipt.outcome {
        AdmissionOutcome::Admitted | AdmissionOutcome::DuplicateAdmitted => QueueState::Admitted,
        AdmissionOutcome::Rejected => QueueState::Rejected,
        AdmissionOutcome::NeedsReview => QueueState::NeedsReview,
    };
    entry.transition_to(next)
}

/// POST the canonical submit body; returns the parsed admission receipt.
pub fn submit_capture_request(
    config: &CaptureApiConfig,
    request: &CaptureSubmitRequest,
) -> Result<AdmissionReceipt, CaptureSubmitError> {
    let url = capture_submit_url(config);
    let response = ureq::post(&url)
        .set("Authorization", &format!("Bearer {}", config.bearer_token))
        .set("Idempotency-Key", &request.idempotency_key)
        .set("Content-Type", "application/json")
        .send_json(request)?;

    let status = response.status();
    let body = response.into_string().unwrap_or_default();
    if !(200..300).contains(&status) {
        return Err(CaptureSubmitError::HttpError { status, body });
    }

    let receipt: AdmissionReceipt = serde_json::from_str(&body)?;
    if receipt.idempotency_key != request.idempotency_key {
        return Err(CaptureSubmitError::IdempotencyMismatch);
    }
    if receipt.draft_id != request.draft_id {
        return Err(CaptureSubmitError::DraftIdMismatch);
    }
    Ok(receipt)
}

/// Decode a persisted record, submit when configured, and return updated JSON.
pub fn submit_persisted_queue_record(
    record_json: &str,
    updated_at: &str,
    config: Option<&CaptureApiConfig>,
) -> Result<String, QueueSubmitError> {
    let record = PersistedQueueRecord::from_json(record_json)?;
    let entry = record.decode()?;

    let submitting = begin_submission(entry).map_err(CaptureSubmitError::from)?;
    let request = build_submit_request(&submitting)?;

    let config = config.ok_or(CaptureSubmitError::ApiNotConfigured)?;

    match submit_capture_request(config, &request) {
        Ok(receipt) => {
            let reconciled = reconcile_admission_receipt(submitting, &receipt)
                .map_err(CaptureSubmitError::from)?;
            PersistedQueueRecord::encode(&reconciled, updated_at)
                .to_json()
                .map_err(QueueSubmitError::from)
        }
        Err(error) => {
            let _rolled = rollback_submission(submitting).map_err(CaptureSubmitError::from)?;
            Err(error.into())
        }
    }
}

/// Reconcile a receipt against persisted JSON without performing HTTP.
pub fn reconcile_persisted_queue_record(
    record_json: &str,
    receipt_json: &str,
    updated_at: &str,
) -> Result<String, QueueSubmitError> {
    let record = PersistedQueueRecord::from_json(record_json)?;
    let entry = record.decode()?;
    let receipt: AdmissionReceipt = serde_json::from_str(receipt_json)
        .map_err(|error| QueueSubmitError::Submit(CaptureSubmitError::InvalidReceipt(error)))?;

    let submitting = if entry.state == QueueState::Queued || entry.state == QueueState::NeedsReview
    {
        begin_submission(entry).map_err(CaptureSubmitError::from)?
    } else {
        entry
    };

    let reconciled =
        reconcile_admission_receipt(submitting, &receipt).map_err(CaptureSubmitError::from)?;

    PersistedQueueRecord::encode(&reconciled, updated_at)
        .to_json()
        .map_err(QueueSubmitError::from)
}

/// Transition persisted JSON to `submitting` without HTTP (BG task step 1).
pub fn begin_persisted_queue_submit(
    record_json: &str,
    updated_at: &str,
) -> Result<String, QueueSubmitError> {
    let record = PersistedQueueRecord::from_json(record_json)?;
    let entry = record.decode()?;
    let submitting = begin_submission(entry).map_err(CaptureSubmitError::from)?;
    PersistedQueueRecord::encode(&submitting, updated_at)
        .to_json()
        .map_err(QueueSubmitError::from)
}

/// Roll back persisted JSON from `submitting` to `queued`.
pub fn rollback_persisted_queue_submit(
    record_json: &str,
    updated_at: &str,
) -> Result<String, QueueSubmitError> {
    let record = PersistedQueueRecord::from_json(record_json)?;
    let entry = record.decode()?;
    let rolled = rollback_submission(entry).map_err(CaptureSubmitError::from)?;
    PersistedQueueRecord::encode(&rolled, updated_at)
        .to_json()
        .map_err(QueueSubmitError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::{
        AppVersion, CaptureModality, CapturePlatform, DraftPayload, IdempotencyKey, SourceMetadata,
        WorkflowVersion,
    };

    fn fixture_queued() -> QueuedCapture {
        QueuedCapture {
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
        }
    }

    #[test]
    fn build_submit_request_matches_packet_fields() {
        let entry = fixture_queued();
        let request = build_submit_request(&entry).expect("request");
        assert_eq!(request.draft_id, "draft:inq:field-signal-v1");
        assert_eq!(request.consent_decision, "accepted");
        assert_eq!(request.modality, "voice_transcript");
    }

    #[test]
    fn reconcile_admitted_moves_submitting_to_admitted() {
        let queued = fixture_queued();
        let submitting = begin_submission(queued).expect("begin");
        let receipt = AdmissionReceipt {
            idempotency_key: submitting.packet.idempotency_key.as_str().to_owned(),
            draft_id: submitting.packet.payload.draft_id.clone(),
            outcome: AdmissionOutcome::Admitted,
            server_receipt_id: Some("rcpt_001".into()),
            message: None,
        };
        let admitted = reconcile_admission_receipt(submitting, &receipt).expect("reconcile");
        assert_eq!(admitted.state, QueueState::Admitted);
    }

    #[test]
    fn duplicate_idempotency_reconciles_to_admitted() {
        let submitting = begin_submission(fixture_queued()).expect("begin");
        let receipt = AdmissionReceipt {
            idempotency_key: submitting.packet.idempotency_key.as_str().to_owned(),
            draft_id: submitting.packet.payload.draft_id.clone(),
            outcome: AdmissionOutcome::DuplicateAdmitted,
            server_receipt_id: None,
            message: Some("already admitted".into()),
        };
        let admitted = reconcile_admission_receipt(submitting, &receipt).expect("reconcile");
        assert_eq!(admitted.state, QueueState::Admitted);
    }

    #[test]
    fn rollback_returns_submitting_to_queued() {
        let submitting = begin_submission(fixture_queued()).expect("begin");
        let rolled = rollback_submission(submitting).expect("rollback");
        assert_eq!(rolled.state, QueueState::Queued);
    }
}
