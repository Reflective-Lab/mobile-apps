use reflective_mobile_core::capture::{AppVersion, CapturePlatform, ConsentRecord, SourceMetadata};
use reflective_mobile_core::consent::ConsentDecision;
use reflective_mobile_core::persistence::{
    PERSISTENCE_RECORD_SCHEMA_VERSION, PersistedQueueRecord, persistence_round_trip,
};
use reflective_mobile_core::queue::QueueState;
use reflective_mobile_core::quorum::{
    SignalModality, draft_field_signal, queue_capture_from_draft,
};

#[test]
fn persisted_record_json_uses_wire_labels_not_rust_debug() {
    let draft = draft_field_signal(
        "inq_mobile_launch_risks",
        SignalModality::VoiceTranscript,
        "signal",
    );
    let entry = queue_capture_from_draft(
        &draft,
        AppVersion {
            app_slug: "quorum-sense".into(),
            client_version: Some("0.1.2".into()),
        },
        SourceMetadata {
            captured_at: Some("2026-06-06T12:00:00Z".into()),
            participant_session_id: Some("session_field_001".into()),
            inquiry_thread_id: None,
            offline: true,
            platform: Some(CapturePlatform::Ios),
        },
        ConsentRecord {
            decision: ConsentDecision::Accepted,
            recorded_at: "2026-06-06T12:01:00Z".into(),
        },
    )
    .expect("queue");

    let json = PersistedQueueRecord::encode(&entry, "2026-06-06T12:02:00Z")
        .to_json()
        .expect("json");

    assert!(json.contains("\"schema_version\":1"));
    assert!(json.contains("\"queue_state\":\"queued\""));
    assert!(json.contains("\"modality\":\"voice_transcript\""));
    assert!(json.contains("\"consent_decision\":\"accepted\""));
    assert!(!json.contains("VoiceTranscript"));
}

#[test]
fn quorum_queued_capture_survives_native_json_storage() {
    let draft = draft_field_signal("inq_test", SignalModality::Text, "note");
    let entry = queue_capture_from_draft(
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
        ConsentRecord {
            decision: ConsentDecision::EditedAndAccepted,
            recorded_at: "2026-06-06T12:01:00Z".into(),
        },
    )
    .expect("queue");

    let restored = persistence_round_trip(&entry, "2026-06-06T12:02:00Z").expect("round trip");
    assert_eq!(restored.state, QueueState::Queued);
    assert_eq!(
        restored.packet.payload.draft_id,
        "draft:inq_test:field-signal-v1"
    );
    assert_eq!(PERSISTENCE_RECORD_SCHEMA_VERSION, 1);
}
