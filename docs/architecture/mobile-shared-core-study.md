# Mobile Shared Core — Portfolio Study (Evidence)

Status: evidence index, 2026-06-12. **Canonical decision:** ADR 0002
(`docs/adr/0002-mobile-platform-boundary.md`). This document retains spike
results, dependency findings, and wave ordering — not the placement rule
itself.

## Summary

Every app in `../marquee-apps/` and `../studio-apps/` eventually gets a mobile
companion — a governed capture surface (voice/text/photo/approval → structured
draft → consent → queued sync) with app-specific domain types. That pattern
motivated ADR 0002: **the device proposes; the server promotes.**

See ADR 0002 for:

- Fixed portfolio rules and Converge allow/forbid matrix
- Three app classes (marquee governed, studio local-first, studio hybrid)
- Per-app placement worksheet (authority, device crates, transport)
- Layering diagram and follow-ups

## Wave ordering

| Wave | Apps | Mobile job class |
|---|---|---|
| Now | quorum-sense | field signal capture |
| Next | inkling-notes, wolfgang-chat | OCR/speech capture, research companion |
| Later | catalyst-biz, scout-sourcing, fathom-narrative, plumb-execution, triage-keeper, vouch-lending, tally-escrow, atlas-integration, warden-compliance, folio-editor, moosemen-writer, wykkid-preso | evidence capture, approvals, incident intake, editorial triage |

## Mobile-portable crates (2026-06-12 spike evidence)

| Crate / layer | Verdict | Evidence |
|---|---|---|
| `converge-pack`, `organism-pack`, `embassy_pack` | Portable | Serde contract types |
| `converge-fuzzy-inference` | Portable | Preview scoring only |
| `axiom-truth` WASM artifacts | Portable | Invariant validation |
| Per-app `domain` (quorum) | **Mobile-clean** | `spikes/quorum-domain-mobile/` — 7 tests, no tokio/reqwest |
| `notes` (inkling) | **Not mobile-clean yet** | `spikes/inkling-notes-mobile/` — reqwest/tokio via hard-coded `sources-web` |
| Manifold, Embassy live, Ferrox, Soter, Mnemos, full Prism, Helm | Server only | ADR 0002 |

## Spikes

Standalone workspaces under `spikes/` (excluded from CI — cross-repo path deps):

- `spikes/quorum-domain-mobile/` — reuses `quorum-domain` citation parse/format,
  confidence clamp, probe-budget feasibility.
- `spikes/inkling-notes-mobile/` — reuses vault capture + navigation index;
  documents upstream fixes needed in `studio-apps/inkling-notes`.

Product UniFFI pattern: `schemas/quorum-mobile.udl` → `crates/mobile-ffi`.

## Cross-references

- `docs/adr/0001-native-swift-kotlin-shared-rust-core.md` — shells + UniFFI
- `docs/adr/0002-mobile-platform-boundary.md` — **placement decision**
- `docs/architecture/native-ai-rust-core.md` — native AI vs Rust
- `docs/architecture/quorum-sense-boundaries.md` — Quorum reference instance
- `../marquee-apps/kb/pattern/directory-shape.md` — domain crate reuse seam
