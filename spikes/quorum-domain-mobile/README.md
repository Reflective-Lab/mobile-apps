# Spike: quorum-domain on mobile

Status: spike, 2026-06-12. Standalone Cargo workspace, **excluded from CI**
(see "CI exclusion" below). Companion to
`docs/architecture/mobile-shared-core-study.md`, which names this spike as the
proof for the marquee-governed reuse seam.

## What this spike proves

Real domain code from the canonical product app
`marquee-apps/quorum-sense` can be consumed **directly** by a mobile-facing
Rust core — no re-implementation of DTOs or logic in the mobile repo. The
spike depends on `quorum-domain` by path:

```toml
quorum-domain = { path = "../../../../../marquee-apps/quorum-sense/crates/quorum-domain" }
```

and wraps two of its deterministic surfaces behind a small, mobile-shaped
facade (`crates/spike-quorum-mobile/src/lib.rs`): plain functions over flat,
serde-friendly DTO structs — the shape a UniFFI `.udl` could bind to
Swift/Kotlin later. Every facade function delegates to the canonical crate;
the unit tests assert behavior (clamping values, error strings, URI
round-trips) that only the real `quorum-domain` code produces.

## Reused functions

1. **Citation parse/format** (`quorum_domain::citation::Citation`):
   `parse_citation(uri)` and `format_citation(kind, id)` delegate to
   `Citation::parse` / `Citation::format`, Quorum's wire contract for
   `quorum://` cross-app references. Chosen because it is the exact thing a
   mobile capture surface must validate offline (deterministic validation
   "near the user" per the placement rule) and because mis-reimplementing the
   scheme on mobile is the canonical drift failure this reuse seam prevents.
2. **`Confidence` clamping and `ProbeBudget` feasibility**
   (`quorum_domain::{Confidence, ProbeBudget}`): `normalize_confidence(raw)`
   applies the domain's [0.0, 1.0] clamp to raw slider input, and
   `is_probe_budget_feasible(dto)` delegates to `ProbeBudget::is_feasible`.
   Chosen as the simplest value-object invariants a field-capture UI needs
   for instant local feedback.

## Verification

`cargo test` inside this directory (2026-06-12, rustc 1.96.0):

```text
running 7 tests ... test result: ok. 7 passed; 0 failed
```

`converge-core` resolved to the local
`bedrock-platform/converge/crates/core` checkout (v3.9.2) via the
`[patch.crates-io]` block in this workspace's `Cargo.toml`, which mirrors the
full patch table in `marquee-apps/quorum-sense/Cargo.toml` with relative paths
corrected for this location (`../../../` vs the product app's `../../`). Most
patch entries are unused in this narrow graph (Cargo warns harmlessly); only
`converge-core`, `converge-pack`, and `converge-provider` participate in the
normal-edge tree today.

## Dependency footprint (mobile-clean verdict)

`cargo tree -e normal --prefix none | sort -u` → **59 unique lines**
(including proc-macro/build-side entries like `syn`, `quote`,
`serde_derive`, which never ship in the binary). Workspace-relevant runtime
crates:

```text
chrono, converge-core, converge-pack, converge-provider, getrandom, hex,
indexmap, quorum-domain, serde, serde_json, sha2, strum, thiserror,
tracing (core only), typed-builder, uuid, zmij
```

Heavy-dependency scan: **no tokio, no reqwest, no polars, no hyper, no
openssl, no axum, no sqlx/rusqlite** in the normal-edge tree.

**Verdict: mobile-clean.** The tree is serde + value types + light hashing
(`sha2`) + `tracing` facade. Nothing pulls a network stack, async runtime, or
native C/C++ solver. Notes:

- `converge-provider` and `converge-pack` come in via `converge-core` but are
  themselves contract/type crates here; they bring `async-trait` (proc-macro,
  trait-shape only — no runtime) rather than an executor.
- `getrandom`/`uuid` (v4 ids) and `chrono` are fine on iOS/Android.
- The only thing that would block mobile is if `quorum-domain` later grew a
  dependency on server-side crates (manifold adapters, ferrox, mnemos). The
  footprint check in this README is the drift signal to re-run.

### Cross-compile check

Skipped: only `aarch64-apple-darwin` is installed
(`rustup target list --installed`, 2026-06-12). Per spike constraints no
targets were installed. Given the pure-Rust, no-native-deps tree above,
`aarch64-apple-ios` / `aarch64-linux-android` checks are expected to pass;
re-run `cargo check --target aarch64-apple-ios` once the target is installed
to confirm.

## Mapping to the planned mobile layering

Per `docs/architecture/mobile-shared-core-study.md`:

```text
Swift/Kotlin shells
      │ UniFFI (per-app .udl)
mobile-quorum-ffi      ← the facade in this spike is the prototype of this layer:
      │                  flat DTOs + plain functions, FacadeError as the error enum
mobile-core (shared)   ← capture workflow, consent, offline queue
      │
portable platform slice ← quorum-domain consumed AS-IS (this spike's proof)
```

- `CitationDto` / `ProbeBudgetDto` / `CitationKind` / `FacadeError` are the
  shapes that move into the per-app FFI crate and `schemas/quorum-mobile.udl`.
- `quorum-domain` itself sits in the "portable platform slice" — pulled from
  the app's canonical `domain` crate, never forked into the mobile repo.
- Authority stays server-side ("the device proposes, the server promotes"):
  everything exposed here is deterministic validation/format logic, safe to
  run near the user.

## CI exclusion

This directory is a **standalone workspace** (its own `[workspace]` table)
and deliberately **not** a member of the root `mobile-apps` workspace. It
depends on sibling repos by relative path
(`../../../marquee-apps/...`, `../../../bedrock-platform/...`) which do not
exist in CI checkouts, so `just ci` / `.github/workflows/ci.yml` never build
it. Run it manually from a full `~/dev/reflective` checkout:

```sh
cd spikes/quorum-domain-mobile
cargo test
```
