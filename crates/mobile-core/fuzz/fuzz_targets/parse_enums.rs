#![no_main]
//! The wire enums are parsed from untrusted strings at the seam. Property: parse
//! never panics on arbitrary input, and any value it accepts round-trips back to
//! the same string. See QF-2026-06-24-06.
use libfuzzer_sys::fuzz_target;
use reflective_mobile_core::quorum::{ConsentState, SignalModality};

fuzz_target!(|value: String| {
    if let Some(m) = SignalModality::parse(&value) {
        assert_eq!(m.as_str(), value);
    }
    if let Some(c) = ConsentState::parse(&value) {
        assert_eq!(c.as_str(), value);
    }
});
