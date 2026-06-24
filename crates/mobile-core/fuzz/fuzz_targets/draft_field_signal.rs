#![no_main]
//! Fuzz the capture entry point with attacker-influenceable text (transcripts,
//! OCR, free notes cross the FFI seam here). Property: drafting then appending a
//! consented signal never panics, regardless of input bytes — a panic would
//! unwind into Swift/Kotlin (UB). See QF-2026-06-24-06.
use libfuzzer_sys::fuzz_target;
use reflective_mobile_core::quorum::{
    append_consented_signal, draft_field_signal, SignalModality,
};

fuzz_target!(|data: (u8, String, String)| {
    let (modality_sel, inquiry_thread_id, raw_capture) = data;
    let modality = match modality_sel % 3 {
        0 => SignalModality::Text,
        1 => SignalModality::VoiceTranscript,
        _ => SignalModality::ImageOcrText,
    };
    let draft = draft_field_signal(&inquiry_thread_id, modality, &raw_capture);
    let _ = append_consented_signal(&draft);
});
