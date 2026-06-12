# Mobile Shared Core — Portfolio Study and Placement Rule

Status: study, 2026-06-11. Sister doc to `native-ai-rust-core.md` and
`quorum-sense-boundaries.md`. Generalizes those rules from Quorum to the whole
marquee + studio portfolio, and answers the placement question: how much
Converge / Organism runs near the user, and how much stays in the backend.

## The portfolio shape

Every app in `../marquee-apps/` and `../studio-apps/` eventually gets a mobile
companion. Not a port — a mobile-specific, AI-first, Jobs-to-be-Done surface.
The recurring observation: **almost every mobile companion is the same app** —
a governed capture surface (voice/text/photo/approval → structured draft →
consent → queued sync) with app-specific domain types. That is what makes the
shared-Rust core pay off across fifteen products.

| Wave | Apps | Mobile job class |
|---|---|---|
| Now | quorum-sense | field signal capture |
| Next | inkling-notes, wolfgang-chat | OCR/speech capture, research companion |
| Later | catalyst-biz, scout-sourcing, fathom-narrative, plumb-execution, triage-keeper, vouch-lending, tally-escrow, atlas-integration, warden-compliance, folio-editor, moosemen-writer, wykkid-preso | evidence capture, approvals, incident intake, editorial triage |

## What is mobile-portable today

Reusable in a mobile Rust binary (serde-only or near-serde-only):

- `converge-pack` — fact/context/suggestor contract types.
- `organism-pack` — `IntentPacket` and intent-pipeline types.
- `embassy_pack` — `Observation<T>`, provenance, `CallContext`.
- `converge-fuzzy-inference` (prism-analytics) — deliberately extracted with
  only `converge-pack` + serde; the cleanest "math on device" candidate.
- `axiom-truth` invariant artifacts — already compiled to WASM; portable
  validation is in its DNA.
- Per-app `domain` crates from the six-crate shape
  (`marquee-apps/kb/pattern/directory-shape.md`) — see the spikes below.

Server-side only — never embedded on mobile:

- **Manifold adapters** — cloud HTTP clients (tokio full + reqwest, 15+ LLM
  vendors). On-device AI is native (Foundation Models / Gemini Nano) per
  `native-ai-rust-core.md`; cloud LLMs are reached through the app server.
- **Embassy live ports** — network, credentials, sanctions screening.
- **Ferrox / Soter / Crucible / Mnemos / Prism (full)** — native C++ solvers
  (OR-Tools, HiGHS, CVC5), Polars/Burn, gRPC servers.
- **Helm** — `publish = false`, SurrealDB + wasmtime + Tauri; desktop
  workbench. Mobile gets its own trust-transfer capture/approval surfaces.

Nothing outside this repo knows about UniFFI, and nothing in the ecosystem is
`no_std`. The mobile core is assembled deliberately, not inherited.

## Placement rule: suggest locally, promote centrally

This is the same question marquee-apps answered for desktop + web: how much
Converge / Organism sits near the user versus in the backend with all
capabilities. The line is not a technology line — `converge-kernel` is
in-process embeddable and would run on a phone. The line is an **authority**
line:

**Anything that creates governed, multi-party, or audited state runs exactly
once, server-side. Anything that is draft-stage, single-user, deterministic,
or latency-sensitive runs near the user.**

Near the user (device — and equally the Tauri desktop shell):

- Contract types and DTOs (converge-pack / organism-pack / app domain crates).
- Deterministic validation and invariants (axiom-style checks, citation and
  capture validation) — instant feedback without a round trip.
- Draft-stage suggestion: on-device AI normalization of a captured signal,
  local fuzzy scoring for *preview*, explicitly marked as draft.
- Offline queue, consent state, sync protocol, replayable fixtures.
- Rendering of server-computed traces (activated rules, receipts, lineage).

Backend only (single authority):

- The Converge engine loop, promotion gate, `ContextState`, fact lineage,
  HITL pauses, integrity proofs (Lamport / Merkle — never branched on a
  device).
- Organism formation assembly, adversarial review, simulation.
- Canonical extraction (Manifold suggestors), Embassy observations, Mnemos
  recall, Ferrox/Soter computation.
- Anything holding credentials, entitlements, or billing state.

Mnemonic: **the device proposes, the server promotes.** A phone may run a
read-only or draft Converge pass; it never closes a round, promotes a fact, or
evaluates the governing rulebook as authority.

### The studio inversion

Studio local-first apps invert the default. For `inkling-notes` the vault on
the device IS canonical; the cloud is optional enrichment and sync. There the
mobile core legitimately carries more Organism (notes/intelligence crates,
local recall) — and the placement rule still holds, because the single
authority happens to live on the device. Classify each app before splitting
it:

| App class | Authority | Device carries |
|---|---|---|
| Marquee governed (quorum, scout, vouch, …) | server | contracts + drafts + queue |
| Studio local-first (inkling, moosemen) | device | domain + kernel + local store |
| Studio hybrid (wolfgang) | server (panel) + device (notes) | split per surface |

## Layering

```
Swift/Kotlin shells (UI, camera, mic, Foundation Models / Gemini Nano)
        │ UniFFI (per-app .udl)
mobile-<app>-ffi        thin facade per app
        │
mobile-core (shared)    capture workflow, consent, offline queue, sync,
                        embeddings/vector search (later), replay fixtures
        │
portable platform slice converge-pack · organism-pack · embassy_pack ·
                        converge-fuzzy-inference · per-app domain crates
        │  ── network boundary ──
app server              Converge engine + governance · Organism formations ·
                        Manifold (LLMs) · Embassy (services) · Mnemos/Prism/
                        Ferrox/Soter · Helm receipts
```

One shared `mobile-core`, one thin FFI crate per app, app domain types pulled
from each app's canonical `domain` crate. Everything heavy stays server-side
where it already lives.

## Spikes

Proof that real app code crosses into the mobile core unmodified lives under
`spikes/` (standalone workspaces, excluded from CI because they use cross-repo
path dependencies):

- `spikes/quorum-domain-mobile/` — reuses functions from
  `marquee-apps/quorum-sense/crates/quorum-domain` behind a mobile-shaped
  facade.
- `spikes/inkling-notes-mobile/` — reuses capture logic from
  `studio-apps/inkling-notes/crates/notes` with mobile-clean features.

Each spike README records the transitive dependency footprint and a
mobile-clean verdict. The product UniFFI track (`schemas/quorum-mobile.udl` →
`crates/mobile-ffi`) is the binding pattern the spikes feed into.

## Cross-references

- `docs/adr/0001-native-swift-kotlin-shared-rust-core.md` — native-first ADR.
- `docs/architecture/native-ai-rust-core.md` — native AI vs Rust boundary.
- `docs/architecture/quorum-sense-boundaries.md` — the Quorum instance of the
  placement rule; this doc generalizes it.
- `../marquee-apps/kb/pattern/directory-shape.md` — six-crate app shape whose
  `domain` crates are the mobile reuse seam.
