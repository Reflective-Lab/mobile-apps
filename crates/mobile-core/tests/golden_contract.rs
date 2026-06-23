//! Golden + contract tests for the Quorum core.
//!
//! * golden — `insta` snapshots pin the exact shape of the draft/event the
//!   core produces, so any unintended change shows up as a diff.
//! * contract — the committed fixture (`field-signal-capture.v1.json`) is the
//!   cross-language source of truth shared with the Swift and Kotlin boundary
//!   tests. Here we assert the Rust core honours the fields that fixture
//!   declares for the `rust_core.draft` / `append_event`.
//!
//! Note: the fixture's `summary` is a *curated ideal* summary, not what the
//! current deterministic stub emits, so it is intentionally not asserted here.

use reflective_mobile_core::quorum::{
    FIELD_SIGNAL_CAPTURE_FIXTURE_JSON, SignalModality, append_consented_signal, draft_field_signal,
};
use serde_json::Value;

fn fixture() -> Value {
    serde_json::from_str(FIELD_SIGNAL_CAPTURE_FIXTURE_JSON).expect("fixture is valid JSON")
}

/// Build the draft from the fixture's declared input, exactly as a client would.
fn draft_from_fixture_input() -> reflective_mobile_core::quorum::QuorumSignalDraft {
    let fx = fixture();
    let input = &fx["input"];
    let modality = SignalModality::parse(input["modality"].as_str().unwrap())
        .expect("fixture modality is a known wire value");
    draft_field_signal(
        input["inquiry_thread_id"].as_str().unwrap(),
        modality,
        input["raw_capture"].as_str().unwrap(),
    )
}

// ---------------------------------------------------------------------------
// contract
// ---------------------------------------------------------------------------

#[test]
fn core_draft_honours_the_fixture_contract() {
    let fx = fixture();
    let expected = &fx["rust_core"]["draft"];
    let draft = draft_from_fixture_input();

    assert_eq!(
        draft.workflow_id.as_str(),
        expected["workflow_id"].as_str().unwrap()
    );
    assert_eq!(
        draft.draft_id.as_str(),
        expected["draft_id"].as_str().unwrap()
    );
    assert_eq!(draft.latent_need, expected["latent_need"].as_str().unwrap());
    assert_eq!(
        draft.contradiction,
        expected["contradiction"].as_str().unwrap()
    );
    assert_eq!(
        draft.consent_state.as_str(),
        expected["consent_state"].as_str().unwrap()
    );

    let expected_confidence = expected["confidence"].as_f64().unwrap() as f32;
    assert_eq!(draft.confidence.value(), expected_confidence);
}

#[test]
fn core_append_event_honours_the_fixture_contract() {
    let fx = fixture();
    let expected = &fx["rust_core"]["append_event"];
    let event = append_consented_signal(&draft_from_fixture_input());

    assert_eq!(
        event.event_type.as_str(),
        expected["event_type"].as_str().unwrap()
    );
    assert_eq!(
        event.sync_state.as_str(),
        expected["sync_state"].as_str().unwrap()
    );
}

// ---------------------------------------------------------------------------
// golden
// ---------------------------------------------------------------------------

#[test]
fn draft_matches_golden_snapshot() {
    insta::assert_debug_snapshot!(draft_from_fixture_input());
}

#[test]
fn append_event_matches_golden_snapshot() {
    insta::assert_debug_snapshot!(append_consented_signal(&draft_from_fixture_input()));
}
