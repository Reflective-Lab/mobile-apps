//! Snapshot-stream firehose harness (ADR 0003 events-out): implement the
//! `SnapshotSink`, run the mock collaboration at max throughput, and measure the
//! stream — rate, monotonicity, and per-snapshot coherence.
//!
//!   cargo run --release -p quorum-ffi --example stream_firehose
//!
//! This measures Rust generation + sink dispatch (the upper bound). On a real
//! device the sink is Swift/Kotlin, so each snapshot also pays FFI marshalling —
//! measured per-platform with Instruments / Macrobenchmark on top of this.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use quorum_ffi::{Snapshot, SnapshotSink, run_mock_collaboration};

struct Recorder {
    count: Arc<AtomicU64>,
    last_version: Arc<AtomicU64>,
    out_of_order: Arc<AtomicU64>,
    incoherent: Arc<AtomicU64>,
}

impl SnapshotSink for Recorder {
    fn on_snapshot(&self, snapshot: Snapshot) {
        // Monotonic by version (the UI applies in order, drops stale).
        let prev = self.last_version.swap(snapshot.version, Ordering::SeqCst);
        if snapshot.version <= prev {
            self.out_of_order.fetch_add(1, Ordering::Relaxed);
        }
        // Coherent-by-type: the head draft is a real, internally consistent domain
        // object — an inconsistent world literally cannot be encoded.
        if snapshot.latest.workflow_id != "quorum.field_signal_capture.v1"
            || snapshot.latest.consent_state != "pending"
        {
            self.incoherent.fetch_add(1, Ordering::Relaxed);
        }
        self.count.fetch_add(1, Ordering::Relaxed);
    }
}

fn main() {
    let count = Arc::new(AtomicU64::new(0));
    let last = Arc::new(AtomicU64::new(0));
    let ooo = Arc::new(AtomicU64::new(0));
    let incoherent = Arc::new(AtomicU64::new(0));

    let n: u32 = 500_000;
    let peers: u32 = 64;

    println!(
        "STREAM firehose: {peers} simulated sessions (humans + AI), {n} snapshots, unthrottled"
    );
    let rec = Recorder {
        count: count.clone(),
        last_version: last.clone(),
        out_of_order: ooo.clone(),
        incoherent: incoherent.clone(),
    };
    let start = Instant::now();
    run_mock_collaboration(Box::new(rec), peers, n, 0);
    let el = start.elapsed();

    let total = count.load(Ordering::SeqCst);
    let rate = total as f64 / el.as_secs_f64();
    println!("  delivered: {total} snapshots in {el:.2?}");
    println!("  rate: {rate:.0} snapshots/sec  (Rust->sink; native adds FFI marshalling on top)");
    println!(
        "  monotonic by version: out-of-order = {} (must be 0)",
        ooo.load(Ordering::SeqCst)
    );
    println!(
        "  coherent-by-type:     incoherent   = {} (must be 0)",
        incoherent.load(Ordering::SeqCst)
    );
    assert_eq!(ooo.load(Ordering::SeqCst), 0, "snapshots must be monotonic");
    assert_eq!(
        incoherent.load(Ordering::SeqCst),
        0,
        "every snapshot must be coherent"
    );
    assert_eq!(total, n as u64, "all snapshots must be delivered");

    println!("\nSNAPSHOT STREAM PASSED");
}
