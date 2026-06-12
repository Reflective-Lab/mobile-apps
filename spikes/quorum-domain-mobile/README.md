# Spike: reuse `quorum-domain` directly in a mobile Rust core

**Status:** spike / proof. Not a product crate. **Excluded from CI** (see below).

## What this spike proves

The canonical product domain crate
`marquee-apps/quorum-sense/crates/quorum-domain` can be consumed **directly**
by a mobile-facing Rust facade — no re-implementation of DTOs or logic in the
mobile repo. The facade (`crates/spike-quorum-mobile`) exposes plain functions
over flat, serde-friendly DTO structs: exactly the shape a UniFFI `.udl`
contract could bind to Swift and Kotlin later.

## Reused domain surface

All behavior below is delegated to `quorum-domain`; the facade only maps
types:

| Facade function | Canonical source | Why chosen |
|---|---|---|
| `parse_citation`, `format_citation` | `quorum_domain::citation::Citation::{parse, format}` | The `quorum://` URI wire contract for cross-app references. Deterministic, validation-heavy, exactly what a mobile client must get right when rendering/deep-linking citations. |
| `normalize_confidence` | `quorum_domain::Confidence::new` | The domain's clamping rule ([0.0, 1.0]) for fuzzy membership scores — mobile capture sliders must produce values the platform accepts. |
| `is_probe_budget_feasible` | `quorum_domain::ProbeBudget::{new, is_feasible}` | A real domain invariant a mobile client should check before submitting a probe allocation proposal. |

Unit tests (7) call only through the facade and assert real domain behavior,
including the exact `thiserror` messages produced by `quorum-domain`'s
citation parser — proving the canonical validation runs behind the facade.

```sh
cd spikes/quorum-domain-mobile
cargo test   # 7 passed; 0 failed (verified 2026-06-11, rustc 1.96.0)
```

## Workspace mechanics (why this is standalone)

- This directory has its own `[workspace]` table, so it is **not** a member
  of the root `mobile-apps` workspace.
- `quorum-domain` is a cross-repo path dependency
  (`../../../marquee-apps/quorum-sense/crates/quorum-domain`). Sibling repos
  are not checked out in CI, so this spike **must stay out of the root
  workspace, `just ci`, and `.github/workflows/`**.
- `quorum-domain` depends on `converge-core = "3.8.1"`, which quorum-sense
  resolves via `[patch.crates-io]` to the unreleased platform head. This
  workspace mirrors quorum-sense's full patch table with paths corrected
  (one extra `../`). Only `converge-core` is load-bearing for the build;
  `converge-pack`/`converge-provider` resolve transitively as in-workspace
  path deps of the converge repo.
- Build-time note: cargo also resolves (but does not build) the
  dev-dependency closure of path deps, so `axiom-truth`/`organism-*` appear
  in `Cargo.lock`, and the mirrored-but-unneeded patch entries emit
  harmless unused-patch warnings. None of them are in the normal (shipped)
  dependency graph below.

## Dependency footprint (mobile verdict: clean)

`cargo tree -e normal --prefix none | sort -u` → **59 lines (~45 unique
crates)**, verified 2026-06-11:

- Local: `quorum-domain`, `converge-core`, `converge-pack`,
  `converge-provider` (v3.9.2 platform head).
- Everything else is lightweight and portable: `serde`/`serde_json`,
  `chrono`, `uuid`, `thiserror`, `tracing` (core only, no subscriber),
  `sha2`/`hex`/`digest`, `strum`, `typed-builder`, `async-trait`
  (proc-macro only — no runtime), `indexmap`, `getrandom`, `libc`,
  plus proc-macro plumbing (`syn`/`quote`/`proc-macro2`).
- **None of `tokio`, `reqwest`, `polars`, `hyper`, `openssl`, `axum`,
  `sqlx` appear.** No async runtime, no network stack, no TLS.

Verdict: `quorum-domain` (via `converge-core`) looks **mobile-clean** for
static-library embedding. The only platform-sensitive crates are
`getrandom`/`libc`/`iana-time-zone`/`core-foundation-sys`, all of which
support iOS and Android targets.

### Cross-compile check

Skipped — only `aarch64-apple-darwin` is installed
(`rustup target list --installed`, 2026-06-11), and the spike brief forbids
installing targets. Given the footprint above (no net/async/TLS), an
`aarch64-apple-ios` / `aarch64-linux-android` `cargo check` is expected to
pass; run it once targets are available and record the result here.

## How this maps to the planned layering

```text
quorum-domain (canonical product crate, marquee-apps)   <- reused as-is
        |
mobile-core      <- this spike's facade role: DTO mapping, deterministic
        |            wrappers, workflow fixtures (today: spike-quorum-mobile)
mobile-ffi       <- UniFFI bindings over the same flat DTOs/functions
        |            (CitationDto, ProbeBudgetDto map 1:1 onto UDL records;
        |             CitationKind onto a flat enum; FacadeError onto an
        |             error enum)
SwiftUI / Compose shells
```

The spike shows `mobile-core` does not need to own domain DTOs — it owns the
*mapping* from canonical domain types to FFI-shaped records, and the domain
crate stays single-source-of-truth in the product repo.

## Open questions for productization

- Distribution: cross-repo path deps don't work in CI. Options: publish
  `quorum-domain` (it is `publish = false` today), a git dependency, or a
  vendoring step in the mobile release pipeline. Needs an ADR.
- Version coupling: the spike builds against the *local head* of
  bedrock-platform via the replicated patch. A real mobile build needs a
  pinned, published platform floor.
