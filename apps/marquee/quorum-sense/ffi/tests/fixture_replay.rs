//! M2.6 — full fixture replay at the published UniFFI boundary.
//!
//! Loads `field-signal-capture.v1.json`, runs the same entry points Swift and
//! Kotlin call, and checks structural draft/event fields plus expected lifecycle.

use quorum_ffi::{
    AppendEventType, ConsentState, FfiQuorumSignalDraft, SignalModality, SyncState,
    quorum_append_consented_signal, quorum_draft_field_signal, quorum_field_signal_workflow_id,
};
use serde_json::Value;

const FIXTURE_JSON: &str = include_str!("../../fixtures/field-signal-capture.v1.json");

fn fixture() -> Value {
    serde_json::from_str(FIXTURE_JSON).expect("fixture JSON parses")
}

fn modality_from_fixture(value: &str) -> SignalModality {
    match value {
        "text" => SignalModality::Text,
        "voice_transcript" => SignalModality::VoiceTranscript,
        "image_ocr_text" => SignalModality::ImageOcrText,
        other => panic!("fixture modality {other:?} is not a known wire value"),
    }
}

fn draft_from_fixture_input() -> FfiQuorumSignalDraft {
    let input = &fixture()["input"];
    quorum_draft_field_signal(
        input["inquiry_thread_id"]
            .as_str()
            .expect("fixture inquiry_thread_id")
            .to_owned(),
        modality_from_fixture(input["modality"].as_str().expect("fixture modality")),
        input["raw_capture"]
            .as_str()
            .expect("fixture raw_capture")
            .to_owned(),
    )
}

#[test]
fn field_signal_capture_fixture_replays_through_ffi() {
    let fx = fixture();
    let expected_draft = &fx["rust_core"]["draft"];
    let expected_event = &fx["rust_core"]["append_event"];
    let input = &fx["input"];

    assert_eq!(
        quorum_field_signal_workflow_id(),
        expected_draft["workflow_id"].as_str().unwrap()
    );

    let draft = draft_from_fixture_input();

    assert_eq!(
        draft.workflow_id,
        expected_draft["workflow_id"].as_str().unwrap()
    );
    assert_eq!(draft.draft_id, expected_draft["draft_id"].as_str().unwrap());
    assert_eq!(
        draft.inquiry_thread_id,
        input["inquiry_thread_id"].as_str().unwrap()
    );
    assert_eq!(
        draft.modality,
        modality_from_fixture(input["modality"].as_str().unwrap())
    );
    assert_eq!(draft.raw_capture, input["raw_capture"].as_str().unwrap());
    assert_eq!(
        draft.consent_state,
        ConsentState::Pending,
        "draft starts pending per fixture contract"
    );

    // Refinement-derived fields: presence + validity, not curated-ideal literals.
    assert!(!draft.summary.is_empty());
    assert!(!draft.latent_need.is_empty());
    assert!(!draft.contradiction.is_empty());
    assert!((0.0..=1.0).contains(&draft.confidence));

    let event = quorum_append_consented_signal(draft.clone())
        .expect("pending draft with valid confidence appends");

    assert_eq!(
        event.event_type,
        AppendEventType::SignalDraftConsented,
        "{}",
        expected_event["event_type"].as_str().unwrap()
    );
    assert_eq!(
        event.sync_state,
        SyncState::QueuedForSync,
        "{}",
        expected_event["sync_state"].as_str().unwrap()
    );
    assert_eq!(event.draft_id, draft.draft_id);
    assert_eq!(event.inquiry_thread_id, draft.inquiry_thread_id);
    assert_eq!(event.workflow_id, draft.workflow_id);
}

#[test]
fn fixture_declares_after_consent_expectation() {
    assert_eq!(
        fixture()["expected"]["after_consent"].as_str().unwrap(),
        "event_queued_for_sync"
    );
}
