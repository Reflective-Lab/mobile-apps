#![no_main]
//! `confidence` is the one numeric that crosses the seam and is range-checked in
//! Rust. Property: `Confidence::new` accepts a value iff it is finite and in
//! [0, 1], never panics, and preserves an accepted value. See QF-2026-06-24-06.
use libfuzzer_sys::fuzz_target;
use reflective_mobile_core::quorum::Confidence;

fuzz_target!(|raw: f32| {
    match Confidence::new(raw) {
        Some(c) => {
            assert!((0.0..=1.0).contains(&c.value()));
            assert_eq!(c.value(), raw);
        }
        None => assert!(!(0.0..=1.0).contains(&raw)),
    }
});
