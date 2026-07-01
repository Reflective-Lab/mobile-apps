# ADR 0006: Mobile-Core External Dependencies and Domain Seam

- Date: 2026-06-27
- Status: Accepted
- Related: M2 in `MILESTONES.md`, ADR 0002 (device/server boundary),
  `docs/architecture/spike-quorum-domain-reuse.md`, `crates/mobile-core/src/refine.rs`

## Context

M2 requires the mobile FFI to stop treating fixture stubs as product logic and to
define a real seam to canonical Quorum. Three distribution questions were open:

1. Where does on-device draft refinement live?
2. What is the product capture submission DTO?
3. How do CI and fresh clones resolve bedrock crates (`converge-core`, `helms`)?

## Decision

### M2.1 — On-device refinement replaces the fixture stub

`draft_field_signal` delegates to `refine::refine_capture`, a Converge fixed-point
formation over the participant's raw capture. The device proposes a structured
draft; the server promotes facts (ADR 0002). Hardcoded fixture literals
(`latent_need`, `contradiction`, `confidence = 0.67`) are removed from product
code.

### M2.2 — `CapturePacket` is the product capture submission DTO

`crates/mobile-core/src/capture.rs` defines the portfolio envelope:
inquiry thread id, participant session id, modality, raw capture, AI draft
fields (`DraftPayload`), consent record, captured-at timestamp, idempotency key,
and client app version. `CaptureSubmitRequest` in `sync.rs` is the wire body for
server admission (M4.8).

User edits and redactions are applied in native consent review UI before the
packet is built; the consented draft crosses the FFI as `FfiQuorumSignalDraft`.

### M2.3 — Typed errors at the FFI boundary

`schemas/quorum-mobile.udl` exposes `QuorumError` for runtime failures
(confidence range, queue transitions, capture submit HTTP, admission receipt
parse). Modality, consent decision, event type, and sync state are **enums** on
the wire — invalid values are compile-time errors in Swift/Kotlin, not runtime
strings.

### M2.4 — Incremental path to canonical Quorum domain

Full `quorum-domain` adoption is deferred per
`docs/architecture/spike-quorum-domain-reuse.md` (modelling migration, not
drop-in). v1 product logic uses:

- Converge formation for deterministic on-device refinement
- Portfolio capture/queue/sync types in `reflective-mobile-core`
- Server contracts for admission (`POST /api/capture/submit`)

Replace seams incrementally in `quorum-ffi` mapping as canonical crates publish
versioned releases.

### M2.5 — Workspace path deps + CI sibling checkout

`reflective-mobile-core` consumes bedrock crates via **workspace path deps**:

- `bedrock-platform/converge/crates/core`
- `bedrock-platform/helms/crates/{director-contracts,helm-client,helm-session-contracts}`

Hosted CI uses `.github/actions/checkout-helms-deps` to restore the sibling
layout under `GITHUB_WORKSPACE/reflective/`. Converge is pinned by git ref
`84649f872cb15ebd6b2a31386dbce96ec5beec04` in that action.

Published crates.io versions are the target once helms/converge cut stable tags;
until then, path + CI checkout is the distribution model.

## Consequences

- Local dev expects `reflective/mobile-apps` beside `reflective/bedrock-platform/{helms,converge}`.
- Golden/fixture tests assert **structural** contract fields; AI-derived fields
  (summary, latent need, contradiction, confidence) assert presence and validity,
  not byte-equality with curated fixture ideals.
- M6 may add `LlmRefineBackend` behind the same `RefineBackend` seam without
  changing the FFI DTO shape.

## Revisit

Re-evaluate git/crates.io pinning when `converge-core` and `quorum-domain` ship
semver tags consumable without SSH checkout (see spike doc).
