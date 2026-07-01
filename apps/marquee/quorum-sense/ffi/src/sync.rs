//! Capture submit + admission reconciliation FFI (M4.8–M4.9).

use reflective_mobile_core::director::DirectorApiConfig;
use reflective_mobile_core::persistence::PersistedQueueRecord;
use reflective_mobile_core::sync::{
    CaptureSubmitError, QueueSubmitError, begin_persisted_queue_submit, build_submit_request_json,
    reconcile_persisted_queue_record, rollback_persisted_queue_submit,
    submit_persisted_queue_record,
};
use std::sync::Mutex;

use crate::{QuorumError, observed};

static CAPTURE_API: Mutex<Option<DirectorApiConfig>> = Mutex::new(None);

/// Point the capture submit boundary at Quorum HTTP (same base as director in local dev).
pub fn quorum_configure_capture_api(base_url: String, bearer_token: String) {
    if let Ok(mut slot) = CAPTURE_API.lock() {
        *slot = Some(DirectorApiConfig::new(base_url, bearer_token));
    }
}

fn map_submit_error(error: CaptureSubmitError) -> QuorumError {
    match error {
        CaptureSubmitError::ApiNotConfigured => QuorumError::CaptureApiNotConfigured,
        CaptureSubmitError::HttpError { status, body } => {
            QuorumError::CaptureSubmitFailed { status, body }
        }
        CaptureSubmitError::InvalidReceipt(detail) => QuorumError::InvalidAdmissionReceipt {
            detail: detail.to_string(),
        },
        CaptureSubmitError::IdempotencyMismatch | CaptureSubmitError::DraftIdMismatch => {
            QuorumError::InvalidAdmissionReceipt {
                detail: error.to_string(),
            }
        }
        other => QuorumError::InvalidPersistedRecord {
            detail: other.to_string(),
        },
    }
}

fn map_queue_submit_error(error: QueueSubmitError) -> QuorumError {
    match error {
        QueueSubmitError::Persistence(detail) => QuorumError::InvalidPersistedRecord {
            detail: detail.to_string(),
        },
        QueueSubmitError::Submit(submit) => map_submit_error(submit),
    }
}

pub fn quorum_build_capture_submit_body(record_json: String) -> Result<String, QuorumError> {
    observed((|| {
        let record = PersistedQueueRecord::from_json(&record_json).map_err(|error| {
            QuorumError::InvalidPersistedRecord {
                detail: error.to_string(),
            }
        })?;
        let entry = record
            .decode()
            .map_err(|error| QuorumError::InvalidPersistedRecord {
                detail: error.to_string(),
            })?;
        let request = build_submit_request_json(&entry).map_err(map_submit_error)?;
        Ok(request)
    })())
}

pub fn quorum_begin_queue_submit(
    record_json: String,
    updated_at: String,
) -> Result<String, QuorumError> {
    observed(
        begin_persisted_queue_submit(&record_json, &updated_at).map_err(map_queue_submit_error),
    )
}

pub fn quorum_rollback_queue_submit(
    record_json: String,
    updated_at: String,
) -> Result<String, QuorumError> {
    observed(
        rollback_persisted_queue_submit(&record_json, &updated_at).map_err(map_queue_submit_error),
    )
}

pub fn quorum_reconcile_capture_admission(
    record_json: String,
    receipt_json: String,
    updated_at: String,
) -> Result<String, QuorumError> {
    observed(
        reconcile_persisted_queue_record(&record_json, &receipt_json, &updated_at)
            .map_err(map_queue_submit_error),
    )
}

pub fn quorum_submit_persisted_queue_record(
    record_json: String,
    updated_at: String,
) -> Result<String, QuorumError> {
    observed({
        let config = CAPTURE_API.lock().ok().and_then(|guard| guard.clone());
        submit_persisted_queue_record(&record_json, &updated_at, config.as_ref())
            .map_err(map_queue_submit_error)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ConsentDecision, SignalModality, quorum_build_persisted_queue_record,
        quorum_draft_field_signal,
    };

    #[test]
    fn build_submit_body_from_persisted_record() {
        let draft = quorum_draft_field_signal(
            "inq_mobile_launch_risks".to_owned(),
            SignalModality::VoiceTranscript,
            "support is seeing confusion".to_owned(),
        );
        let json = quorum_build_persisted_queue_record(
            draft,
            ConsentDecision::Accepted,
            "2026-06-06T12:01:00Z".to_owned(),
            "2026-06-06T12:02:00Z".to_owned(),
            Some("0.1.2".to_owned()),
            true,
            "ios".to_owned(),
        )
        .expect("record");
        let body = quorum_build_capture_submit_body(json).expect("body");
        assert!(body.contains("idempotency:draft:inq_mobile_launch_risks:field-signal-v1"));
    }
}
