# mobile-core fuzz harness

Coverage-guided fuzzing of the untrusted-input seam (`QF-2026-06-24-06`) — the
text and numbers that cross the FFI from the native shells into
`crates/mobile-core/src/quorum.rs`. The property under test is the one
`QF-2026-06-24-02` forbids violating: **no input may panic**, because a panic
here unwinds into Swift/Kotlin (UB), not a catchable error.

## Targets

| Target | Input | Property |
|---|---|---|
| `draft_field_signal` | `(u8, String, String)` | draft → append a consented signal never panics, any bytes |
| `parse_enums` | `String` | enum parse never panics; an accepted value round-trips to the same string |
| `confidence_roundtrip` | `f32` | `Confidence::new` accepts iff finite ∈ [0, 1], never panics, preserves the value |

## Layout

- `fuzz_targets/<target>.rs` — the libFuzzer entry points.
- `seeds/<target>/` — **committed** seed corpus: curated starting inputs (valid
  enum strings, boundary confidence values, realistic capture text incl.
  unicode/empty). Tracked, so every run and every machine starts warm.
- `corpus/<target>/` — **gitignored** scratch: where libFuzzer writes newly
  discovered units during a run. Promote anything interesting into `seeds/`.
- `artifacts/<target>/` — gitignored; a crash reproducer lands here (and uploads
  as a CI artifact on failure).

## Running

```sh
rustup toolchain install nightly
cargo install cargo-fuzz
just fuzz-core                       # draft_field_signal, 60s, seed corpus
just fuzz-core parse_enums 120       # a specific target, longer
```

CI (`.github/workflows/fuzz.yml`): a paths-scoped **PR smoke** runs the hottest
target (`draft_field_signal`) for 30s on PRs that touch the core/FFI/harness; the
**nightly** schedule + manual dispatch run the full matrix longer. All read
`seeds/<target>`.
