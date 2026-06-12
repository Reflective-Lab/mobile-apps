# Spike: inkling-notes on mobile

Status: spike, 2026-06-12. Standalone Cargo workspace, **excluded from CI**
(see "CI exclusion" below). Companion to
`docs/architecture/mobile-shared-core-study.md`, which names this spike as the
proof for the studio local-first reuse seam.

## What this spike proves

Real application logic from the canonical product app
`studio-apps/inkling-notes/crates/notes` — plus the vault engine it writes
through — can be consumed **directly** by a mobile-facing Rust core. The spike
depends on `notes` by path:

```toml
notes = { path = "../../../../../studio-apps/inkling-notes/crates/notes", default-features = false, features = ["web"] }
```

and on `organism-notes` (patched to the local platform head) for the capture
write path. A small facade (`crates/spike-inkling-mobile/src/lib.rs`) wraps
two deterministic surfaces behind flat, serde-friendly DTOs — the shape a
UniFFI `.udl` could bind to Swift/Kotlin later. Integration tests in
`crates/spike-inkling-mobile/tests/facade.rs` assert behavior (filename
sanitization, wiki-link resolution, orphan counts, serde round-trips) that only
the real product/platform code produces.

## Reused functions

1. **`notes::navigation::build_navigation_index`** — delegated by
   `build_vault_navigation(vault_root)`. Chosen because it is the canonical
   offline vault indexer: tag extraction (`extract_tags`), wiki/markdown link
   parsing (`extract_links`), backlink resolution, orphan detection, and
   persisted `navigation-index.json`. This is exactly the "device-canonical
   vault" navigation job for inkling mobile (studio local-first authority on
   device). Default `NavigationIndexOptions` keeps image OCR and external-link
   fetching off, so tests run fully offline.

2. **`organism_notes::vault::ObsidianVault` write path** — used by
   `capture_text_note(vault_root, title, text)` via `allocate_note_path`,
   `save_note`, and `extract_frontmatter_value`. Chosen as the capture seam
   because `notes::capture::capture` is the smart URL/file pipeline (detect →
   fetch → extract → format) and is **not** mobile-clean: see feature findings
   below. Mobile capture is "normalized text draft → vault note"; that write
   path is what `capture.rs` / `capture/format.rs` ultimately call into via
   `ObsidianVault`. The facade does not re-implement title sanitization or
   freshness frontmatter — tests assert canonical `Inbox/Field Capture-Pier 7-.md`
   stems and `vault_created_at` stamps from organism-notes.

## Feature-flag findings

| Configuration | Compiles? | Notes |
|---|---|---|
| `notes` with `default-features = false` only | **No** | `capture.rs:62` — `#[cfg(not(feature = "social"))]` arm calls `web::capture_web` but the `web` module is gated on `feature = "web"`. Compile error: `cannot find module web`. |
| `notes` with `default-features = false, features = ["web"]` | **Yes** | Minimum feature set used by this spike. Drops default `social` but keeps the `web` capture module. |
| `notes` with desktop defaults (`web`, `social`) | Yes | Pulls social capture + web fetch; heavier than needed for mobile proof. |

Additional hard coupling in the **product** crate (not overridden by spike):

- `notes/Cargo.toml` always enables `organism-notes` with `features = ["sources-web"]`, which pulls `organism-intelligence/web` → **reqwest + tokio + hyper** even when the spike never calls URL capture at runtime.
- A truly mobile-clean `notes` dependency would need either a `notes` feature to disable `sources-web`, or splitting capture/navigation into separate crates. That is tracked as a product-side follow-up, not fixed in this spike (studio-apps is read-only here).

Runtime behavior of this spike: **no network I/O in tests** — navigation uses default options; capture writes local markdown only.

## Verification

`cargo test` inside this directory (2026-06-12):

```text
running 6 tests ... test result: ok. 6 passed; 0 failed
```

`organism-intelligence` and `organism-notes` resolve to the local
`bedrock-platform/organism/crates/*` checkout (v1.9.3) via the
`[patch.crates-io]` entry in this workspace's `Cargo.toml`, mirroring
`studio-apps/inkling-notes/Cargo.toml` with paths corrected for this location.

## Dependency footprint (mobile-clean verdict)

`cargo tree -e normal --prefix none | sort -u` → **174 unique lines**
(including proc-macro/build-side entries).

Heavy-dependency scan on the spike crate's normal edge:

```text
h2, hyper, hyper-rustls, hyper-tls, hyper-util, reqwest (0.12 + 0.13),
tokio, tokio-native-tls, tokio-rustls, tokio-util, native-tls, openssl-sys
```

By contrast, **`organism-notes` with its own `default = []` features** (no
`sources-web`) is only `chrono`, `serde`, `serde_json`, `thiserror` — but in
this spike's unified build, `notes` enables `organism-notes/sources-web`, so
`cargo tree -p organism-notes` still shows reqwest/tokio/hyper here too.

**Verdict: not mobile-clean as wired today.** The spike proves reuse is
*technically* feasible, but the current `notes` crate dependency graph pulls a
full HTTP/TLS/async stack through:

1. `notes` → `organism-notes` with hard-coded `sources-web`, and
2. `notes` requiring `feature = "web"` to compile at all.

To ship on device, the product-side seam likely needs one of:

- a `notes` feature flag to drop `sources-web` for mobile builds, or
- mobile depending on `organism-notes` + a slim `notes-navigation` crate, with
  URL/file smart capture replaced by native AI on iOS/Android per
  `docs/architecture/native-ai-rust-core.md`.

The footprint recorded here is the drift signal — re-run `cargo tree` after any
`notes` feature work.

### Cross-compile check

Skipped: only `aarch64-apple-darwin` is installed
(`rustup target list --installed`, 2026-06-12). Per spike constraints no
targets were installed. `organism-notes`-only is pure Rust; the full spike
tree adds native TLS (`openssl-sys`, `aws-lc-sys`) via reqwest, which is the
main iOS/Android cross-compile risk. Re-run `cargo check --target aarch64-apple-ios`
once the target is installed.

## Mapping to the planned mobile layering

Per `docs/architecture/mobile-shared-core-study.md`:

```text
Swift/Kotlin shells (camera, mic, Foundation Models / Gemini Nano)
      │ UniFFI (per-app .udl)
mobile-inkling-ffi     ← this spike's facade: CapturedNoteDto, VaultNavigationDto,
      │                  capture_text_note, build_vault_navigation
mobile-core (shared)   ← capture workflow, consent, offline queue (later)
      │
portable platform slice ← organism-notes (vault) + notes (navigation index)
      │  ── optional cloud enrichment boundary ──
app server / cloud      ← smart URL capture, Mistral OCR, remote PDF (when enabled)
```

- DTOs and plain functions here are the prototype of a per-app FFI crate
  (`schemas/inkling-mobile.udl` — not created yet).
- `build_vault_navigation` maps to local-first studio authority on device.
- Smart capture (`notes::capture`) stays behind native AI or server enrichment;
  the mobile job is vault write + navigation index, which this spike covers.

## CI exclusion

This directory is a **standalone workspace** (its own `[workspace]` table) and
deliberately **not** a member of the root `mobile-apps` workspace. It depends
on sibling repos by relative path
(`../../../studio-apps/...`, `../../../bedrock-platform/...`) which do not
exist in CI checkouts, so `just ci` / `.github/workflows/ci.yml` never build
it. Run it manually from a full `~/dev/reflective` checkout:

```sh
cd spikes/inkling-notes-mobile
cargo test
```
