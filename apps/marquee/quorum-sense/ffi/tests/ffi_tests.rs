//! Boundary tests for the Quorum UniFFI surface.
//!
//! The inline `#[cfg(test)]` module in `src/lib.rs` covers fixture-exact unit and
//! negative cases. This integration suite adds the property, public-API-compile,
//! and concurrency-stress coverage, all through the published FFI functions —
//! the exact entry points Swift and Kotlin call.
//!
//! NB: there are no "reject unknown modality / consent" tests here. Those fields
//! are *enums* on the wire, so an invalid value cannot be constructed — the type
//! system enforces it at compile time, which is why this file no longer needs to.

use quorum_ffi::{
    AppendEventType, ConsentState, FfiDirectorIntent, FfiQuorumSignalDraft, GateVerdict, QuorumError,
    SignalModality, SyncState, ai_execution_home, mobile_portfolio, quorum_append_consented_signal,
    quorum_current_director_snapshot, quorum_draft_field_signal, quorum_field_signal_workflow_id,
    quorum_submit_director_intent,
};

const VALID_MODALITIES: [SignalModality; 3] = [
    SignalModality::Text,
    SignalModality::VoiceTranscript,
    SignalModality::ImageOcrText,
];

fn valid_draft() -> FfiQuorumSignalDraft {
    // No Result: `modality` is an enum, so drafting can't fail.
    quorum_draft_field_signal(
        "inq_mobile_launch_risks".to_owned(),
        SignalModality::VoiceTranscript,
        "support is seeing confusion in every pilot".to_owned(),
    )
}

// ---------------------------------------------------------------------------
// negative — only the fields that are still primitives can fail.
// ---------------------------------------------------------------------------

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
    let _ = quorum_current_director_snapshot();
    quorum_submit_director_intent(FfiDirectorIntent::RequestContext {
        level: quorum_ffi::ContextLevel::Session,
    });
}

#[test]
fn director_snapshot_matches_fixture_version() {
    let snapshot = quorum_current_director_snapshot();
    assert_eq!(snapshot.version, 1844);
    assert_eq!(
        snapshot
            .frame
            .now
            .as_ref()
            .map(|now| now.objective.as_str()),
        Some("Evaluate Vendor X's security claims")
    );
}

// ---------------------------------------------------------------------------
// property
// ---------------------------------------------------------------------------

use proptest::prelude::*;

proptest! {
    /// Any valid modality drafts into a coherent, pending draft — across arbitrary
    /// thread ids and capture text.
    #[test]
    fn drafting_is_coherent_for_every_modality(
        thread in ".{0,48}",
        modality in prop::sample::select(VALID_MODALITIES.to_vec()),
        raw in ".{0,200}",
    ) {
        let draft = quorum_draft_field_signal(thread.clone(), modality, raw.clone());
        prop_assert_eq!(draft.modality, modality);
        prop_assert_eq!(&draft.inquiry_thread_id, &thread);
        prop_assert_eq!(&draft.raw_capture, &raw);
        prop_assert_eq!(draft.consent_state, ConsentState::Pending);
        prop_assert!((0.0..=1.0).contains(&draft.confidence));
    }

    /// A round-tripped valid draft always appends to a consented, queued event
    /// whose ids match the draft.
    #[test]
    fn valid_draft_round_trips_through_append(
        thread in ".{0,48}",
        modality in prop::sample::select(VALID_MODALITIES.to_vec()),
        raw in ".{0,200}",
    ) {
        let draft = quorum_draft_field_signal(thread, modality, raw);
        let event = quorum_append_consented_signal(draft.clone()).expect("pending draft appends");
        prop_assert_eq!(&event.draft_id, &draft.draft_id);
        prop_assert_eq!(&event.inquiry_thread_id, &draft.inquiry_thread_id);
        prop_assert_eq!(&event.workflow_id, &draft.workflow_id);
        prop_assert_eq!(event.event_type, AppendEventType::SignalDraftConsented);
        prop_assert_eq!(event.sync_state, SyncState::QueuedForSync);
    }

    /// The boundary never panics on arbitrary *confidence* — it returns Ok or a
    /// typed error. Modality/consent can't be fuzzed any more: they're enums.
    #[test]
    fn append_is_total_over_arbitrary_confidence(
        confidence in proptest::num::f32::ANY,
    ) {
        let mut draft = valid_draft();
        draft.confidence = confidence;
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
                                modality,
                                "payload under load".to_owned(),
                            );
                            let event = quorum_append_consented_signal(draft.clone())
                                .expect("pending draft appends under load");
                            assert_eq!(event.draft_id, draft.draft_id);
                            // Also exercise the throwing AI-routing path.
                            let _ = ai_execution_home(
                                "ios".to_owned(),
                                "structured-extraction".to_owned(),
                            );
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
            panic!(
                "FFI stress scenario did not finish within {DEADLINE:?} — possible deadlock/hang"
            );
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            coordinator
                .join()
                .expect("coordinator thread panicked under load");
            panic!("FFI stress coordinator disconnected without signalling completion");
        }
    }
}
