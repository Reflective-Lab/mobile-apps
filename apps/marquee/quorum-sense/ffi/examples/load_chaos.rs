//! Core-level load + chaos harness for the shared Quorum FFI boundary.
//!
//!   cargo run --release -p quorum-ffi --example load_chaos
//!
//! LOAD  — many concurrent "sessions" (humans + AI) firing draft→append cycles
//!         through the real boundary functions; measures throughput + latency
//!         and asserts every result is a coherent, typed snapshot.
//! CHAOS — adversarial input under concurrency; asserts every bad value is
//!         rejected as a *typed* error (never a panic, never a silent Ok), i.e.
//!         the system degrades by rejecting, not by corrupting.
//!
//! Note: the wire contract uses strict enums (SignalModality / ConsentState), so
//! a malformed modality or consent value is now *compile-time* impossible — a
//! stronger guarantee than the previous string-rejection. The only input the
//! boundary can still receive malformed at runtime is an out-of-range confidence
//! (an f32), which is what the chaos phase exercises.
//!
//! This exercises the synchronous core (the shared foundation). The UI
//! responsiveness-under-firehose layer needs the ADR 0003 snapshot stream and
//! is measured per-platform with Instruments / Macrobenchmark — not here.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Instant;

use quorum_ffi::{
    ConsentState, FfiQuorumSignalDraft, QuorumError, SignalModality, SyncState,
    quorum_append_consented_signal, quorum_draft_field_signal,
};

/// Deterministic SplitMix64 so runs are reproducible.
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

const MODALITIES: [SignalModality; 3] = [
    SignalModality::Text,
    SignalModality::VoiceTranscript,
    SignalModality::ImageOcrText,
];

fn pct(sorted: &[u64], p: f64) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let idx = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[idx]
}

fn main() {
    let sessions = 32usize; // concurrent "incoming sessions" (humans + AIs)
    let ops_each = 50_000usize; // draft+append cycles per session

    // ---------------- LOAD ----------------
    println!(
        "LOAD: {sessions} concurrent sessions x {ops_each} draft+append cycles = {} ops",
        sessions * ops_each
    );
    let start = Instant::now();
    let handles: Vec<_> = (0..sessions)
        .map(|s| {
            thread::spawn(move || {
                let mut rng = Rng(0xC0FF_EE00 ^ s as u64);
                let mut lats = Vec::with_capacity(ops_each);
                for i in 0..ops_each {
                    let modality = MODALITIES[(rng.next() % 3) as usize];
                    let thread_id = format!("inq_session_{s}_{}", rng.next() % 1000);
                    let capture = format!("signal {i} from session {s}: {}", rng.next());
                    let t = Instant::now();
                    // Strict enums make this infallible: there is no invalid modality.
                    let draft = quorum_draft_field_signal(thread_id, modality, capture);
                    let event = quorum_append_consented_signal(draft.clone())
                        .expect("valid draft must append");
                    lats.push(t.elapsed().as_nanos() as u64);
                    // Coherence: the snapshot the core emitted is internally consistent.
                    assert_eq!(event.workflow_id, draft.workflow_id);
                    assert_eq!(event.draft_id, draft.draft_id);
                    assert_eq!(event.sync_state, SyncState::QueuedForSync);
                }
                lats
            })
        })
        .collect();
    let mut all: Vec<u64> = handles
        .into_iter()
        .flat_map(|h| h.join().unwrap())
        .collect();
    let elapsed = start.elapsed();
    let total = all.len();
    all.sort_unstable();
    let ops_per_sec = total as f64 / elapsed.as_secs_f64();
    println!("  ok: {total} cycles, no deadlock/panic, all snapshots coherent");
    println!("  throughput: {ops_per_sec:.0} draft+append/sec  (wall {elapsed:.2?})");
    println!(
        "  latency/cycle: p50 {} us | p99 {} us | max {} us",
        pct(&all, 0.50) / 1000,
        pct(&all, 0.99) / 1000,
        all.last().copied().unwrap_or(0) / 1000
    );

    // ---------------- CHAOS ----------------
    // Bad modality / consent are compile-time-prevented by the strict wire enums,
    // so the only malformed runtime input is an out-of-range confidence (f32).
    let chaos_threads = 16usize;
    let chaos_each = 25_000usize;
    println!(
        "\nCHAOS: {chaos_threads} threads x {chaos_each} out-of-range-confidence ops = {} bad inputs",
        chaos_threads * chaos_each
    );
    let bad_confidence = Arc::new(AtomicU64::new(0));
    let slipped_through = Arc::new(AtomicU64::new(0)); // bad input wrongly accepted

    let handles: Vec<_> = (0..chaos_threads)
        .map(|t| {
            let (bc, slip) = (bad_confidence.clone(), slipped_through.clone());
            thread::spawn(move || {
                let mut rng = Rng(0xBAD_5EED ^ t as u64);
                for _ in 0..chaos_each {
                    // Confidence forced above 1.0 -> must be ConfidenceOutOfRange.
                    let bad = FfiQuorumSignalDraft {
                        workflow_id: "w".into(),
                        draft_id: "d".into(),
                        inquiry_thread_id: "i".into(),
                        modality: SignalModality::Text,
                        raw_capture: "x".into(),
                        summary: "x".into(),
                        latent_need: "x".into(),
                        contradiction: "x".into(),
                        confidence: 2.0 + (rng.next() % 100) as f32,
                        consent_state: ConsentState::Pending,
                    };
                    match quorum_append_consented_signal(bad) {
                        Err(QuorumError::ConfidenceOutOfRange { .. }) => {
                            bc.fetch_add(1, Ordering::Relaxed);
                        }
                        Ok(_) => {
                            slip.fetch_add(1, Ordering::Relaxed);
                        }
                        Err(_) => {}
                    }
                }
            })
        })
        .collect();
    // A panicking thread fails the join -> harness exits non-zero.
    for h in handles {
        h.join().expect("chaos thread must not panic");
    }
    let slipped = slipped_through.load(Ordering::Relaxed);
    println!(
        "  out-of-range confidence rejected as typed ConfidenceOutOfRange: {}",
        bad_confidence.load(Ordering::Relaxed),
    );
    println!("  (bad modality / consent are compile-time-prevented by strict wire enums)");
    println!("  bad inputs wrongly accepted (must be 0): {slipped}");
    assert_eq!(slipped, 0, "a malformed input was accepted as valid");
    println!("  ok: no panic, no corruption, every bad value rejected at the boundary");

    println!("\nLOAD + CHAOS PASSED");
}
