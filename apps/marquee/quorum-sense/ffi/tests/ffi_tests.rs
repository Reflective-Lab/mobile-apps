//! Boundary tests for the Quorum UniFFI surface.
//!
//! The inline `#[cfg(test)]` module in `src/lib.rs` covers fixture-exact unit and
//! negative cases. This integration suite adds the property, public-API-compile,
//! and concurrency-stress coverage, all through the published FFI functions —
//! the exact entry points Swift and Kotlin call.

use quorum_ffi::{
    FfiQuorumSignalDraft, QuorumError, ai_execution_home, mobile_portfolio,
    quorum_append_consented_signal, quorum_draft_field_signal, quorum_field_signal_workflow_id,
};

const VALID_MODALITIES: [&str; 3] = ["text", "voice_transcript", "image_ocr_text"];

fn valid_draft() -> FfiQuorumSignalDraft {
    quorum_draft_field_signal(
        "inq_mobile_launch_risks".to_owned(),
        "voice_transcript".to_owned(),
        "support is seeing confusion in every pilot".to_owned(),
    )
    .expect("fixture modality is valid")
}

// ---------------------------------------------------------------------------
// negative
// ---------------------------------------------------------------------------

#[test]
fn draft_rejects_unknown_modality_with_the_offending_value() {
    let err = quorum_draft_field_signal("inq".to_owned(), "hologram".to_owned(), "x".to_owned())
        .expect_err("unknown modality must error");
    assert!(matches!(err, QuorumError::InvalidModality { value } if value == "hologram"));
}

#[test]
fn append_rejects_out_of_range_confidence() {
    let mut draft = valid_draft();
    draft.confidence = 2.0;
    assert!(matches!(
        quorum_append_consented_signal(draft),
        Err(QuorumError::ConfidenceOutOfRange { value }) if value == 2.0
    ));
}

#[test]
fn append_rejects_unknown_consent_state() {
    let mut draft = valid_draft();
    draft.consent_state = "revoked".to_owned();
    assert!(matches!(
        quorum_append_consented_signal(draft),
        Err(QuorumError::InvalidConsentState { value }) if value == "revoked"
    ));
}

#[test]
fn ai_execution_home_rejects_unknown_platform_and_task() {
    assert!(matches!(
        ai_execution_home("web".to_owned(), "embeddings".to_owned()),
        Err(QuorumError::UnsupportedPlatform { .. })
    ));
    assert!(matches!(
        ai_execution_home("ios".to_owned(), "telepathy".to_owned()),
        Err(QuorumError::UnsupportedTask { .. })
    ));
}

// ---------------------------------------------------------------------------
// compile — public FFI surface stays callable.
// ---------------------------------------------------------------------------

#[test]
fn public_ffi_surface_is_callable() {
    let _ = quorum_field_signal_workflow_id();
    let _ = mobile_portfolio();
    let _ = ai_execution_home("ios".to_owned(), "structured-extraction".to_owned());
    let draft = valid_draft();
    let _ = quorum_append_consented_signal(draft);
}

// ---------------------------------------------------------------------------
// property
// ---------------------------------------------------------------------------

use proptest::prelude::*;

proptest! {
    /// Drafting succeeds iff the modality string is a known wire value; the error
    /// always carries the exact offending input back to the caller.
    #[test]
    fn draft_accepts_exactly_the_valid_modalities(
        thread in ".{0,48}",
        modality in ".{0,32}",
        raw in ".{0,200}",
    ) {
        let result = quorum_draft_field_signal(thread.clone(), modality.clone(), raw.clone());
        if VALID_MODALITIES.contains(&modality.as_str()) {
            let draft = result.expect("valid modality should draft");
            prop_assert_eq!(&draft.modality, &modality);
            prop_assert_eq!(&draft.inquiry_thread_id, &thread);
            prop_assert_eq!(&draft.raw_capture, &raw);
            prop_assert_eq!(&draft.consent_state, "pending");
            prop_assert!((0.0..=1.0).contains(&draft.confidence));
        } else {
            match result {
                Err(QuorumError::InvalidModality { value }) => prop_assert_eq!(value, modality),
                other => prop_assert!(false, "expected InvalidModality, got {:?}", other),
            }
        }
    }

    /// A round-tripped valid draft always appends to a consented, queued event
    /// whose ids match the draft.
    #[test]
    fn valid_draft_round_trips_through_append(
        thread in ".{0,48}",
        modality in prop::sample::select(VALID_MODALITIES.to_vec()),
        raw in ".{0,200}",
    ) {
        let draft = quorum_draft_field_signal(thread, modality.to_owned(), raw)
            .expect("valid modality should draft");
        let event = quorum_append_consented_signal(draft.clone()).expect("pending draft appends");
        prop_assert_eq!(&event.draft_id, &draft.draft_id);
        prop_assert_eq!(&event.inquiry_thread_id, &draft.inquiry_thread_id);
        prop_assert_eq!(&event.workflow_id, &draft.workflow_id);
        prop_assert_eq!(&event.event_type, "SignalDraftConsented");
        prop_assert_eq!(&event.sync_state, "queued_for_sync");
    }

    /// The boundary never panics on arbitrary append input — it returns Ok or a
    /// typed error, mirroring how the native shells must be able to trust it.
    #[test]
    fn append_is_total_over_arbitrary_drafts(
        modality in ".{0,16}",
        confidence in proptest::num::f32::ANY,
        consent in ".{0,16}",
    ) {
        let mut draft = valid_draft();
        draft.modality = modality;
        draft.confidence = confidence;
        draft.consent_state = consent;
        // Must not panic; either outcome is acceptable here.
        let _ = quorum_append_consented_signal(draft);
    }
}

// ---------------------------------------------------------------------------
// stress — many threads hammering the boundary concurrently must not deadlock,
// hang, or panic. A watchdog turns a hang into a deterministic failure.
// ---------------------------------------------------------------------------

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[test]
fn concurrent_ffi_calls_do_not_deadlock_or_panic() {
    const WORKERS: u64 = 16;
    const ITERS: u64 = 15_000;
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
                            let modality = VALID_MODALITIES[(i % 3) as usize];
                            let draft = quorum_draft_field_signal(
                                format!("inq_{w}_{i}"),
                                modality.to_owned(),
                                "payload under load".to_owned(),
                            )
                            .expect("valid modality drafts under load");
                            let event = quorum_append_consented_signal(draft.clone())
                                .expect("pending draft appends under load");
                            assert_eq!(event.draft_id, draft.draft_id);
                            // Also exercise the throwing AI-routing path.
                            let _ = ai_execution_home("ios".to_owned(), "structured-extraction".to_owned());
                            completed.fetch_add(1, Ordering::Relaxed);
                        }
                    })
                })
                .collect();
            for h in handles {
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
            panic!("FFI stress scenario did not finish within {DEADLINE:?} — possible deadlock/hang");
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            coordinator.join().expect("coordinator thread panicked under load");
            panic!("FFI stress coordinator disconnected without signalling completion");
        }
    }
}
