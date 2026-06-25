//! Cross-cutting tests for the Quorum field-signal-capture domain.
//!
//! Organised by intent so the suite doubles as documentation:
//!
//! * unit — happy-path behaviour on representative input
//! * negative — invalid/boundary input is rejected, not silently accepted
//! * property — invariants hold across randomised input (proptest)
//! * compile — the public API surface stays callable (this file linking is the test)
//! * stress — the pure core survives heavy concurrent load without panicking,
//!   deadlocking, or hanging
//!
//! These exercise only the public API (integration-test crate boundary), which
//! is itself the "does the published surface still compile" guarantee.

use reflective_mobile_core::quorum::{
    AppendEventType, Confidence, ConsentState, DraftId, FIELD_SIGNAL_CAPTURE_WORKFLOW_ID,
    InquiryThreadId, SignalModality, SyncState, WorkflowId, append_consented_signal,
    draft_field_signal,
};

const SAMPLE_CAPTURE: &str =
    "The sales team says rollout is fine, but support is seeing confusion in every pilot.";

// ---------------------------------------------------------------------------
// unit
// ---------------------------------------------------------------------------

#[test]
fn draft_then_append_carries_ids_through() {
    let draft = draft_field_signal(
        "inq_mobile_launch_risks",
        SignalModality::VoiceTranscript,
        SAMPLE_CAPTURE,
    );

    assert_eq!(draft.workflow_id.as_str(), FIELD_SIGNAL_CAPTURE_WORKFLOW_ID);
    assert_eq!(
        draft.draft_id.as_str(),
        "draft:inq_mobile_launch_risks:field-signal-v1"
    );
    assert_eq!(draft.inquiry_thread_id.as_str(), "inq_mobile_launch_risks");
    assert_eq!(draft.modality, SignalModality::VoiceTranscript);
    assert_eq!(draft.consent_state, ConsentState::Pending);
    // Contract update (M2.1): confidence is computed by the on-device Converge
    // refinement loop now, not the old hardcoded 0.67 fixture literal.
    assert!((0.0..=1.0).contains(&draft.confidence.value()));

    let event = append_consented_signal(&draft);
    assert_eq!(event.workflow_id, draft.workflow_id);
    assert_eq!(event.draft_id, draft.draft_id);
    assert_eq!(event.inquiry_thread_id, draft.inquiry_thread_id);
    assert_eq!(event.event_type, AppendEventType::SignalDraftConsented);
    assert_eq!(event.sync_state, SyncState::QueuedForSync);
}

#[test]
fn empty_capture_falls_back_to_a_clarification_summary() {
    let draft = draft_field_signal("inq", SignalModality::Text, "   \n\t  ");
    assert_eq!(
        draft.summary,
        "Empty capture needs participant clarification"
    );
}

#[test]
fn wire_labels_are_stable() {
    assert_eq!(SignalModality::Text.as_str(), "text");
    assert_eq!(SignalModality::VoiceTranscript.as_str(), "voice_transcript");
    assert_eq!(SignalModality::ImageOcrText.as_str(), "image_ocr_text");
    assert_eq!(ConsentState::Pending.as_str(), "pending");
    assert_eq!(ConsentState::Consented.as_str(), "consented");
    assert_eq!(
        AppendEventType::SignalDraftConsented.as_str(),
        "SignalDraftConsented"
    );
    assert_eq!(SyncState::QueuedForSync.as_str(), "queued_for_sync");
}

// ---------------------------------------------------------------------------
// negative
// ---------------------------------------------------------------------------

#[test]
fn confidence_rejects_out_of_range_and_non_finite() {
    assert!(Confidence::new(-0.000_001).is_none());
    assert!(Confidence::new(1.000_001).is_none());
    assert!(Confidence::new(f32::NAN).is_none());
    assert!(Confidence::new(f32::INFINITY).is_none());
    assert!(Confidence::new(f32::NEG_INFINITY).is_none());
}

#[test]
fn confidence_accepts_inclusive_bounds() {
    assert_eq!(Confidence::new(0.0).map(Confidence::value), Some(0.0));
    assert_eq!(Confidence::new(1.0).map(Confidence::value), Some(1.0));
}

#[test]
fn parsing_rejects_unknown_wire_values() {
    assert!(SignalModality::parse("hologram").is_none());
    assert!(SignalModality::parse("").is_none());
    assert!(SignalModality::parse("Text").is_none()); // case-sensitive wire contract
    assert!(ConsentState::parse("revoked").is_none());
    assert!(ConsentState::parse("PENDING").is_none());
}

// ---------------------------------------------------------------------------
// compile — every public entry point is named here; if a signature changes in a
// breaking way this file stops compiling, which is the intended guard.
// ---------------------------------------------------------------------------

#[test]
fn public_api_surface_is_callable() {
    let _ = WorkflowId::field_signal_capture();
    let _ = WorkflowId::new("w");
    let _ = DraftId::new("d");
    let _ = InquiryThreadId::new("t");
    let _ = Confidence::new(0.5);
    let _ = SignalModality::Text.as_str();
    let _ = SignalModality::parse("text");
    let _ = ConsentState::Pending.as_str();
    let _ = ConsentState::parse("pending");
    let _ = AppendEventType::SignalDraftConsented.as_str();
    let _ = SyncState::QueuedForSync.as_str();

    let draft = draft_field_signal("t", SignalModality::Text, "c");
    let _ = append_consented_signal(&draft);
}

// ---------------------------------------------------------------------------
// property
// ---------------------------------------------------------------------------

use proptest::prelude::*;

proptest! {
    /// `Confidence::new` accepts a value iff it is finite and within `0.0..=1.0`.
    #[test]
    fn confidence_new_matches_its_contract(value in proptest::num::f32::ANY) {
        let in_range = value.is_finite() && (0.0..=1.0).contains(&value);
        let parsed = Confidence::new(value);
        prop_assert_eq!(parsed.is_some(), in_range);
        if let Some(c) = parsed {
            prop_assert_eq!(c.value(), value);
        }
    }

    /// Modality parsing round-trips for valid wire strings and only those.
    #[test]
    fn modality_parse_round_trips(s in ".*") {
        match SignalModality::parse(&s) {
            Some(m) => prop_assert_eq!(m.as_str(), s.as_str()),
            None => prop_assert!(!["text", "voice_transcript", "image_ocr_text"].contains(&s.as_str())),
        }
    }

    /// Consent parsing round-trips for valid wire strings and only those.
    #[test]
    fn consent_parse_round_trips(s in ".*") {
        match ConsentState::parse(&s) {
            Some(c) => prop_assert_eq!(c.as_str(), s.as_str()),
            None => prop_assert!(!["pending", "consented"].contains(&s.as_str())),
        }
    }

    /// Drafting is total over arbitrary input and preserves the invariants the
    /// rest of the pipeline relies on, regardless of thread id / capture text.
    #[test]
    fn draft_invariants_hold_for_any_input(
        thread in ".{0,64}",
        raw in ".{0,500}",
        modality in prop_oneof![
            Just(SignalModality::Text),
            Just(SignalModality::VoiceTranscript),
            Just(SignalModality::ImageOcrText),
        ],
    ) {
        let draft = draft_field_signal(&thread, modality, &raw);

        prop_assert_eq!(draft.workflow_id.as_str(), FIELD_SIGNAL_CAPTURE_WORKFLOW_ID);
        prop_assert_eq!(draft.draft_id.as_str(), format!("draft:{thread}:field-signal-v1"));
        prop_assert_eq!(draft.inquiry_thread_id.as_str(), thread.as_str());
        prop_assert_eq!(draft.modality, modality);
        prop_assert_eq!(&draft.raw_capture, &raw);
        prop_assert_eq!(draft.consent_state, ConsentState::Pending);
        // Summary is bounded and never empty.
        prop_assert!(draft.summary.chars().count() <= 96);
        prop_assert!(!draft.summary.is_empty());
        // Confidence is always within range.
        prop_assert!((0.0..=1.0).contains(&draft.confidence.value()));

        // Append preserves identity and emits the fixed event/sync markers.
        let event = append_consented_signal(&draft);
        prop_assert_eq!(event.draft_id, draft.draft_id);
        prop_assert_eq!(event.inquiry_thread_id, draft.inquiry_thread_id);
        prop_assert_eq!(event.workflow_id, draft.workflow_id);
        prop_assert_eq!(event.event_type, AppendEventType::SignalDraftConsented);
        prop_assert_eq!(event.sync_state, SyncState::QueuedForSync);
    }
}

// ---------------------------------------------------------------------------
// stress — the domain is pure and stateless, so heavy concurrent use must never
// deadlock, hang, or panic. A coordinator thread joins all workers and signals a
// channel; the test thread converts a missed deadline into a hard failure
// instead of an indefinite hang.
// ---------------------------------------------------------------------------

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[test]
fn concurrent_draft_append_does_not_deadlock_or_panic() {
    const WORKERS: u64 = 16;
    const ITERS: u64 = 25_000;
    const DEADLINE: Duration = Duration::from_secs(60);

    let completed = Arc::new(AtomicU64::new(0));
    let (done_tx, done_rx) = mpsc::channel();

    let coordinator = {
        let completed = Arc::clone(&completed);
        thread::spawn(move || {
            let handles: Vec<_> = (0..WORKERS)
                .map(|w| {
                    let completed = Arc::clone(&completed);
                    thread::spawn(move || {
                        for i in 0..ITERS {
                            let modality = match i % 3 {
                                0 => SignalModality::Text,
                                1 => SignalModality::VoiceTranscript,
                                _ => SignalModality::ImageOcrText,
                            };
                            let draft = draft_field_signal(
                                &format!("inq_{w}_{i}"),
                                modality,
                                SAMPLE_CAPTURE,
                            );
                            let event = append_consented_signal(&draft);
                            // Invariant under contention: ids stay paired.
                            assert_eq!(event.draft_id, draft.draft_id);
                            completed.fetch_add(1, Ordering::Relaxed);
                        }
                    })
                })
                .collect();

            for h in handles {
                // Propagate a worker panic into the coordinator (then the test).
                h.join().expect("worker thread panicked under load");
            }
            let _ = done_tx.send(());
        })
    };

    match done_rx.recv_timeout(DEADLINE) {
        Ok(()) => {
            coordinator.join().expect("coordinator thread panicked");
            assert_eq!(completed.load(Ordering::Relaxed), WORKERS * ITERS);
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            panic!("stress scenario did not finish within {DEADLINE:?} — possible deadlock/hang");
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            // Coordinator died (a worker panicked); surface it.
            coordinator
                .join()
                .expect("coordinator thread panicked under load");
            panic!("stress coordinator disconnected without signalling completion");
        }
    }
}
