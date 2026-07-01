//! Queue lifecycle replay scenarios (M4.10).

use reflective_mobile_core::capture::{
    AppVersion, CaptureModality, CapturePacket, CapturePlatform, ConsentRecord, DraftPayload,
    IdempotencyKey, SourceMetadata, WorkflowVersion,
};
use reflective_mobile_core::consent::ConsentDecision;
use reflective_mobile_core::persistence::{PersistedQueueRecord, persistence_round_trip};
use reflective_mobile_core::queue::{QueueState, QueuedCapture};
use reflective_mobile_core::sync::{
    AdmissionOutcome, AdmissionReceipt, begin_persisted_queue_submit, begin_submission,
    build_submit_request, reconcile_admission_receipt, reconcile_persisted_queue_record,
    rollback_persisted_queue_submit,
};

fn fixture_packet(consent: ConsentDecision) -> CapturePacket {
    CapturePacket {
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
            raw_capture: "The sales team says rollout is fine.".into(),
            summary: "Sales reports readiness.".into(),
            latent_need: "need".into(),
            contradiction: "tension".into(),
            confidence: 0.67,
        },
        consent: Some(ConsentRecord {
            decision: consent,
            recorded_at: "2026-06-06T12:01:00Z".into(),
        }),
    }
}

fn persist_queued(consent: ConsentDecision) -> String {
    let queued = QueuedCapture::at_review(fixture_packet(consent))
        .enqueue()
        .expect("enqueue");
    PersistedQueueRecord::encode(&queued, "2026-06-06T12:02:00Z")
        .to_json()
        .expect("json")
}

#[test]
fn offline_enqueue_survives_persistence_round_trip() {
    let json = persist_queued(ConsentDecision::Accepted);
    let record = PersistedQueueRecord::from_json(&json).expect("parse");
    let entry = record.decode().expect("decode");
    assert_eq!(entry.state, QueueState::Queued);
    let restored = persistence_round_trip(&entry, "2026-06-06T12:03:00Z").expect("round trip");
    assert_eq!(restored.state, QueueState::Queued);
}

#[test]
fn submit_path_admits_after_receipt_reconciliation() {
    let json = persist_queued(ConsentDecision::Accepted);
    let submitting_json =
        begin_persisted_queue_submit(&json, "2026-06-06T12:04:00Z").expect("begin");
    let submitting = PersistedQueueRecord::from_json(&submitting_json)
        .expect("parse")
        .decode()
        .expect("decode");
    assert_eq!(submitting.state, QueueState::Submitting);

    let receipt = AdmissionReceipt {
        idempotency_key: submitting.packet.idempotency_key.as_str().to_owned(),
        draft_id: submitting.packet.payload.draft_id.clone(),
        outcome: AdmissionOutcome::Admitted,
        server_receipt_id: Some("rcpt_offline_001".into()),
        message: None,
    };
    let receipt_json = serde_json::to_string(&receipt).expect("receipt json");
    let admitted_json =
        reconcile_persisted_queue_record(&submitting_json, &receipt_json, "2026-06-06T12:05:00Z")
            .expect("reconcile");

    let admitted = PersistedQueueRecord::from_json(&admitted_json)
        .expect("parse")
        .decode()
        .expect("decode");
    assert_eq!(admitted.state, QueueState::Admitted);
}

#[test]
fn network_failure_rolls_submitting_back_to_queued() {
    let json = persist_queued(ConsentDecision::Accepted);
    let submitting_json =
        begin_persisted_queue_submit(&json, "2026-06-06T12:04:00Z").expect("begin");
    let rolled_json = rollback_persisted_queue_submit(&submitting_json, "2026-06-06T12:05:00Z")
        .expect("rollback");
    let rolled = PersistedQueueRecord::from_json(&rolled_json)
        .expect("parse")
        .decode()
        .expect("decode");
    assert_eq!(rolled.state, QueueState::Queued);
}

#[test]
fn duplicate_idempotency_key_reconciles_to_admitted() {
    let queued = fixture_queued_capture();
    let submitting = begin_submission(queued).expect("begin");
    let receipt = AdmissionReceipt {
        idempotency_key: submitting.packet.idempotency_key.as_str().to_owned(),
        draft_id: submitting.packet.payload.draft_id.clone(),
        outcome: AdmissionOutcome::DuplicateAdmitted,
        server_receipt_id: Some("rcpt_dup".into()),
        message: Some("duplicate".into()),
    };
    let admitted = reconcile_admission_receipt(submitting, &receipt).expect("reconcile");
    assert_eq!(admitted.state, QueueState::Admitted);
}

#[test]
fn server_rejection_moves_to_needs_review_on_retry_path() {
    let queued = fixture_queued_capture();
    let submitting = begin_submission(queued).expect("begin");
    let receipt = AdmissionReceipt {
        idempotency_key: submitting.packet.idempotency_key.as_str().to_owned(),
        draft_id: submitting.packet.payload.draft_id.clone(),
        outcome: AdmissionOutcome::Rejected,
        server_receipt_id: None,
        message: Some("policy".into()),
    };
    let rejected = reconcile_admission_receipt(submitting, &receipt).expect("reconcile");
    assert_eq!(rejected.state, QueueState::Rejected);

    let review = rejected
        .transition_to(QueueState::NeedsReview)
        .expect("needs review");
    let requeued = review
        .transition_to(QueueState::Queued)
        .expect("retry queue");
    assert_eq!(requeued.state, QueueState::Queued);
}

#[test]
fn idempotency_key_is_stable_per_draft() {
    let entry = fixture_queued_capture();
    let request = build_submit_request(&entry).expect("request");
    assert_eq!(
        request.idempotency_key,
        "idempotency:draft:inq:field-signal-v1"
    );
}

fn fixture_queued_capture() -> QueuedCapture {
    QueuedCapture::at_review(fixture_packet(ConsentDecision::Accepted))
        .enqueue()
        .expect("enqueue")
}
