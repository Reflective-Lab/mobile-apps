use reflective_mobile_core::capture::{
    AppVersion, CaptureModality, CapturePacket, CapturePlatform, ConsentRecord, DraftPayload,
    IdempotencyKey, SourceMetadata, WorkflowVersion,
};
use reflective_mobile_core::consent::ConsentDecision;
use reflective_mobile_core::quorum::{
    FIELD_SIGNAL_CAPTURE_WORKFLOW_ID, SignalModality, append_from_capture_packet,
    capture_packet_from_draft, draft_field_signal,
};

#[test]
fn capture_packet_matches_field_signal_fixture_shape() {
    let draft = draft_field_signal(
        "inq_mobile_launch_risks",
        SignalModality::VoiceTranscript,
        "The sales team says rollout is fine, but support is seeing confusion in every pilot.",
    );

    let packet = capture_packet_from_draft(
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
        Some(ConsentRecord {
            decision: ConsentDecision::EditedAndAccepted,
            recorded_at: "2026-06-06T12:01:00Z".into(),
        }),
    )
    .expect("packet builds");

    assert_eq!(
        packet.workflow.workflow_id,
        FIELD_SIGNAL_CAPTURE_WORKFLOW_ID
    );
    assert_eq!(packet.modality, CaptureModality::VoiceTranscript);
    assert_eq!(
        packet.payload.draft_id,
        "draft:inq_mobile_launch_risks:field-signal-v1"
    );
    assert!(packet.ready_for_queue().is_ok());

    let event = append_from_capture_packet(&packet).expect("queues from packet");
    assert_eq!(event.draft_id.as_str(), packet.payload.draft_id);
}

#[test]
fn capture_packet_without_consent_cannot_queue() {
    let packet = CapturePacket {
        app: AppVersion {
            app_slug: "quorum-sense".into(),
            client_version: None,
        },
        workflow: WorkflowVersion::parse(FIELD_SIGNAL_CAPTURE_WORKFLOW_ID).unwrap(),
        idempotency_key: IdempotencyKey::for_draft("draft:test:field-signal-v1"),
        modality: CaptureModality::Text,
        source: SourceMetadata {
            captured_at: None,
            participant_session_id: None,
            inquiry_thread_id: Some("inq_test".into()),
            offline: false,
            platform: None,
        },
        payload: DraftPayload {
            draft_id: "draft:test:field-signal-v1".into(),
            raw_capture: "signal".into(),
            summary: "signal".into(),
            latent_need: "need".into(),
            contradiction: "tension".into(),
            confidence: 0.5,
        },
        consent: None,
    };

    assert!(packet.ready_for_queue().is_err());
    assert!(append_from_capture_packet(&packet).is_err());
}
