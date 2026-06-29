use reflective_mobile_core::capture::{AppVersion, CapturePlatform, ConsentRecord, SourceMetadata};
use reflective_mobile_core::consent::ConsentDecision;
use reflective_mobile_core::queue::{QueueState, QueuedCapture};
use reflective_mobile_core::quorum::{
    SignalModality, SyncState, draft_field_signal, queue_capture_from_draft,
};

#[test]
fn queue_state_covers_full_lifecycle_vocabulary() {
    let labels = [
        "draft_local",
        "pending_consent",
        "queued",
        "submitting",
        "admitted",
        "rejected",
        "needs_review",
        "abandoned",
    ];
    for label in labels {
        assert!(QueueState::parse(label).is_some(), "missing state: {label}");
    }
}

#[test]
fn legacy_sync_state_maps_to_queued() {
    assert_eq!(
        QueueState::from_legacy_sync_state(SyncState::QueuedForSync.as_str()),
        Some(QueueState::Queued)
    );
}

#[test]
fn quorum_draft_queues_into_queued_capture() {
    let draft = draft_field_signal(
        "inq_mobile_launch_risks",
        SignalModality::VoiceTranscript,
        "The sales team says rollout is fine.",
    );

    let queued = queue_capture_from_draft(
        &draft,
        AppVersion {
            app_slug: "quorum-sense".into(),
            client_version: None,
        },
        SourceMetadata {
            captured_at: Some("2026-06-06T12:00:00Z".into()),
            participant_session_id: None,
            inquiry_thread_id: None,
            offline: true,
            platform: Some(CapturePlatform::Android),
        },
        ConsentRecord {
            decision: ConsentDecision::Accepted,
            recorded_at: "2026-06-06T12:01:00Z".into(),
        },
    )
    .expect("queues");

    assert_eq!(queued.state, QueueState::Queued);
    assert!(queued.state.permits_submit());
    assert_eq!(
        queued.packet.idempotency_key.as_str(),
        "idempotency:draft:inq_mobile_launch_risks:field-signal-v1"
    );
}

#[test]
fn new_draft_starts_at_draft_local() {
    let draft = draft_field_signal("inq_test", SignalModality::Text, "note");
    let packet = reflective_mobile_core::quorum::capture_packet_from_draft(
        &draft,
        AppVersion {
            app_slug: "quorum-sense".into(),
            client_version: None,
        },
        SourceMetadata {
            captured_at: None,
            participant_session_id: None,
            inquiry_thread_id: None,
            offline: false,
            platform: None,
        },
        None,
    )
    .expect("packet");

    let entry = QueuedCapture::new_draft(packet);
    assert_eq!(entry.state, QueueState::DraftLocal);
    assert!(entry.state.is_active());
}

#[test]
fn milestone_illegal_transitions_are_rejected() {
    use QueueState::*;

    let illegal = [
        (PendingConsent, Submitting),
        (PendingConsent, Admitted),
        (Rejected, Admitted),
        (Rejected, Submitting),
        (Rejected, Queued),
        (DraftLocal, Queued),
        (Admitted, Queued),
        (Abandoned, PendingConsent),
    ];

    for (from, to) in illegal {
        assert!(
            !from.allows_transition_to(to),
            "expected illegal: {from:?} -> {to:?}"
        );
        assert!(from.transition_to(to).is_err());
    }
}

#[test]
fn rejected_reaches_queued_only_through_needs_review() {
    let rejected = sample_queued_at_review()
        .enqueue()
        .expect("enqueue")
        .transition_to(QueueState::Submitting)
        .expect("submitting")
        .transition_to(QueueState::Rejected)
        .expect("server rejected");

    assert!(!rejected.state.allows_transition_to(QueueState::Queued));

    let requeued = rejected
        .transition_to(QueueState::NeedsReview)
        .expect("escalate to review")
        .transition_to(QueueState::Queued)
        .expect("retry after review");
    assert_eq!(requeued.state, QueueState::Queued);
}

#[test]
fn happy_path_reaches_submitting_from_queued() {
    let entry = sample_queued_at_review()
        .enqueue()
        .expect("enqueue")
        .transition_to(QueueState::Submitting)
        .expect("submit");
    assert_eq!(entry.state, QueueState::Submitting);
}

fn sample_queued_at_review() -> QueuedCapture {
    let draft = draft_field_signal("inq_test", SignalModality::Text, "note");
    let packet = reflective_mobile_core::quorum::capture_packet_from_draft(
        &draft,
        AppVersion {
            app_slug: "quorum-sense".into(),
            client_version: None,
        },
        SourceMetadata {
            captured_at: None,
            participant_session_id: None,
            inquiry_thread_id: None,
            offline: false,
            platform: None,
        },
        Some(ConsentRecord {
            decision: ConsentDecision::Accepted,
            recorded_at: "2026-06-06T12:01:00Z".into(),
        }),
    )
    .expect("packet");
    QueuedCapture::at_review(packet)
}
