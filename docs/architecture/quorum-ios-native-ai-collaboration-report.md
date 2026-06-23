# Quorum iOS Native AI Collaboration Report

- Date: 2026-06-22
- Scope: Quorum iOS app direction, with a reusable mobile pattern for the
  Reflective app portfolio.
- Inputs reviewed: `README.md`, `AGENTS.md`, `KB/`, ADR 0001, ADR 0002,
  `docs/architecture/`, Quorum iOS/Android shells, shared Rust crates, FFI
  schema, fixtures, spikes, CI, and local Rust/scaffold checks.
- Limitation: sibling `../marquee-apps/` and `../studio-apps/` repositories were
  not present in this checkout, so canonical Quorum product claims are taken
  from this repo's boundary documents and spike notes.
- See also: ADR 0003 (`docs/adr/0003-responsiveness-and-snapshot-consistency.md`)
  is the system contract beneath this report. The "Live Inquiry Status", consent,
  and offline-queue surfaces below assume its invariants — off-main linearized
  core, immutable versioned snapshots, optimistic echo, one local+remote
  pipeline. Read it before implementing any of them.

## Executive Decision

Quorum iOS should not become a mobile version of the desktop/web workbench. It
should become the native **field signal and consent surface** for governed group
sensemaking: capture what happened, preserve ambiguity, let the human correct
the AI draft, obtain explicit consent, queue safely offline, and submit to the
canonical Quorum server for promotion.

The architectural line should stay firm:

```text
Native iOS / Android:
  capture, local AI draft normalization, permissions, consent UX, offline queue,
  realtime rendering, notifications, app intents

Shared Rust on device:
  typed workflow DTOs, capability policy, deterministic validation, queue
  contracts, sync state machine, replay fixtures, FFI facade

Quorum server:
  canonical domain authority, inquiry kernel, promotion, Lamport/Merkle chain,
  rulebooks, Mosaic/Manifold specialists, receipts, entitlements, billing
```

The product posture is simple: **the phone proposes; the server promotes; the
user remains visibly in control.**

## Current State

The repo is directionally sound. The core decisions already match the product
need:

- Native-first shells are explicit: SwiftUI for iOS, Compose for Android, Rust
  below the UI line, UniFFI at the bridge, and platform AI first where vendor
  runtimes are best.
- ADR 0002 establishes the right portfolio authority rule: device proposes,
  server promotes.
- Quorum boundary docs correctly prohibit mobile from owning the inquiry kernel,
  promotion, rulebooks, Stripe, admission, or receipt authority.
- The iOS app now injects `QuorumCoreBridgeFFI` at the root and keeps
  `PreviewQuorumCoreBridge` available for previews.
- The Rust FFI layer has the right shape: raw strings/floats cross UniFFI, then
  get parsed into typed Rust domain values at the boundary.
- Local verification passed: `cargo test --workspace --locked` and
  `bash scripts/check-mobile-scaffold.sh`.

The important caveat: most of the current Quorum mobile behavior is still a
fixture harness. That is acceptable for M1, but it must not become accidental
product architecture.

## Material Gaps

### 1. Rust Quorum behavior is still fixture logic

`crates/mobile-core/src/quorum.rs` currently hardcodes the workflow id, draft id
shape, summary behavior, latent need, contradiction, confidence, and append
event. That is fine for replay scaffolding, but it is not the product seam.

The spike in `spikes/quorum-domain-mobile/` says the better path is viable:
consume canonical `quorum-domain` behind a mobile-shaped facade and expose only
flat DTOs/functions through UniFFI. That should become the M2 bridge direction.

Recommendation:

- Keep fixture helpers only under a fixture/test module.
- Introduce a Quorum mobile facade that delegates all real Quorum validation and
  DTO semantics to canonical Quorum crates or to generated server API types.
- Treat any hand-written Swift/Kotlin/Rust Quorum domain mirror as temporary
  scaffolding with an explicit deletion path.

### 2. Android is behind iOS on bridge and type safety

iOS maps FFI strings into Swift enums and a bounded `Confidence` value. Android
currently leaves `consentState`, `eventType`, `syncState`, and `confidence` as
raw primitives and still uses `PreviewQuorumCoreBridge`.

Recommendation:

- Add Kotlin sealed/value/domain wrappers equivalent to Swift:
  `ConsentState`, `AppendEventType`, `SyncState`, and `Confidence`.
- Add `QuorumCoreBridgeFFI` on Android using generated UniFFI Kotlin bindings.
- Make Android fail on unrecognized core values instead of displaying unknown
  strings as if they were valid product states.

### 3. CI has stale iOS jobs

`.github/workflows/ci.yml` and `release-preflight.yml` still contain
`apps/marquee/quorum-sense/ios/QuorumMobileIOS` `swift build` jobs. The iOS app
has been flattened into an XcodeGen app target, so those jobs are stale and will
fail or stop validating the real product path.

Recommendation:

- Remove the old SwiftPM jobs.
- Add a Quorum product iOS CI job that runs `just quorum-ios-gen` and
  `just quorum-ios-build` on macOS.
- Keep the generic template smoke shell, but do not confuse it with the Quorum
  product app gate.

### 4. Native capture is not yet native capture

The current iOS surface is a `TextEditor` plus a modality picker. The imports
for Foundation Models, Speech, and Vision are guarded, but no AVFoundation,
Speech, PhotosUI, Vision, or App Intents flow is implemented yet.

Recommendation:

- Build one real iOS capture path first: speech transcript to local draft.
- Then add PhotosUI/Vision OCR capture.
- Then add App Intent quick capture.
- Keep all local AI output visibly editable and labeled as draft, never truth.

### 5. Consent is a button, not a consent model

The current button says "Consent And Queue" and immediately asks Rust for an
append event. Product consent needs more structure than that.

Recommendation:

- Model `ConsentDecision` as a typed value: accepted, edited-and-accepted,
  rejected, saved-private, expired.
- Persist the pre-consent draft, post-edit draft, timestamp, actor/session, and
  redaction state.
- Submit only the post-consent packet.
- Keep "save private" separate from "submit later"; offline does not imply
  consent.

### 6. Offline queue and realtime reconciliation are absent

The current append event has `queued_for_sync`, but there is no durable local
queue, retry policy, idempotency key, admission receipt, SSE reconciliation, or
conflict status.

Recommendation:

- Put a typed queue state machine in `mobile-core`.
- Store queue records in native persistence through a narrow Rust contract:
  pending consent, queued, submitting, admitted, rejected, needs review.
- Use server admission receipts to reconcile local optimistic state.
- Render server state distinctly from local device state.

### 7. Capability and subscription placement is not modeled yet

The architecture says some Bedrock/Mosaic/Rust capabilities will run locally,
some on the server, and some based on capability, load, use case, and plan. The
current `mobile-ai` crate returns a static recommended home by platform/task.

Recommendation:

- Replace static task routing with a typed `CapabilitySnapshot`:
  device platform, OS version, model availability, network status, battery,
  thermal state, privacy mode, workspace policy, plan entitlement, server load
  hint, and data sensitivity.
- Return a typed `ComputePlacement`: local required, local preferred, server
  required, server preferred, unavailable, or ask user.
- Keep platform probes native; keep policy evaluation in Rust so iOS and Android
  remain consistent.

## Target iOS Product Shape

Quorum iOS should be built around four native surfaces.

### 1. Participant Capture

The participant surface should be fast, non-workbench, and ambiguity-preserving:

- One primary action: capture a signal.
- Inputs: text, speech, photo/OCR, later camera/video-derived observations.
- Local AI creates a structured draft with summary, source excerpt, latent need,
  contradiction/tension, confidence, uncertainty, and suggested redactions.
- The user edits before consent.
- The UI labels draft AI work clearly and avoids implying admission into the
  inquiry.

### 2. Consent Review

The consent screen is the trust transfer point:

- Show raw capture beside AI draft.
- Show what will be sent and what stays local.
- Show whether the signal is anonymous, attributed, or pseudonymous for this
  inquiry.
- Provide explicit actions: submit, edit, redact, save private, discard.
- Never auto-submit because a model completed extraction.

### 3. Live Inquiry Status

Realtime should render the server's governed state, not simulate it locally:

- Local draft: "on this device".
- Queued event: "waiting to submit".
- Server-admitted signal: "received by inquiry".
- Server-computed trace/receipt: "admitted by rule".
- Round/status changes arrive through the canonical HTTP/SSE surface.

The UX goal is to preserve collective intelligence under uncertainty: users need
to see whether the group is converging, which tensions remain unresolved, and
whether minority signals are still visible.

### 4. Facilitator Field Mode

The facilitator mobile flow should not duplicate the host console. It should
cover work that happens away from a desk:

- scan the room for unresolved tension,
- capture stakeholder signals,
- review queued participant submissions,
- see inquiry health and next missing evidence,
- approve mobile-safe actions only when the server is authority.

## Reusable Portfolio Pattern

Every Reflective mobile app should follow the same app skeleton:

```text
apps/<family>/<app>/
  ios/
    App/
    Capture/
    Consent/
    Realtime/
    OfflineQueue/
    CoreBridge/
    PlatformAI/
  android/
    app/src/main/java/.../capture
    app/src/main/java/.../consent
    app/src/main/java/.../realtime
    app/src/main/java/.../offlinequeue
    app/src/main/java/.../corebridge
    app/src/main/java/.../platformai
  ffi/
  fixtures/
```

Rust should separate portfolio-wide capabilities from app-specific surfaces:

```text
crates/mobile-core/
  capture workflow primitives
  consent ledger contract
  offline queue state machine
  sync envelope and idempotency
  capability snapshot and compute placement
  replay fixture harness

crates/mobile-ai/
  policy over task + capabilities + plan + privacy

apps/<family>/<app>/ffi/
  thin product FFI over canonical app domain/API types
```

Do not create shared SwiftUI or Compose UI frameworks prematurely. Share design
language, state-machine contracts, generated bindings, and fixture behavior
first. Native UI reuse should emerge from proven repetition, not from a forced
cross-app abstraction.

## Boundary Rules Going Forward

For Quorum and other marquee governed apps:

- No on-device fact promotion.
- No Lamport/Merkle mutation.
- No rulebook authority.
- No Mosaic credentials or live specialist ports on device.
- No direct Stripe or entitlement semantics on device.
- No Swift/Kotlin domain fork for server-owned entities.
- No generic chat surface as the primary mobile UX.

For studio local-first apps:

- Device authority is allowed only when the app charter says the device owns the
  vault/store.
- Local-first still needs typed sync, conflict, privacy, and export semantics.
- Native AI can draft and enrich, but user-owned knowledge must remain
  inspectable and recoverable.

## Tactical Plan

### P0: Stabilize the foundation

- Fix stale iOS CI jobs and add a real Quorum product iOS build gate.
- Bring Android to FFI parity with iOS.
- Add scaffold/CI checks for forbidden mobile dependencies named in ADR 0002.
- Move fixture-only Quorum logic behind explicit fixture/test naming.

### P1: Ship the first real iOS slice

- Implement speech capture with native permission flow.
- Normalize transcript locally into a draft.
- Add a consent review screen with edit/redact/save-private/submit actions.
- Persist queued consented packets durably.
- Submit through the canonical Quorum HTTP API when available.

### P2: Add collaboration and resilience

- Add SSE live inquiry rendering.
- Reconcile queued submissions with server admission receipts.
- Add PhotosUI/Vision OCR capture.
- Add App Intent quick capture.
- Add capability-aware AI routing and user-visible fallback behavior.

### P3: Turn the pattern into a portfolio platform

- Extract reusable consent and queue contracts in `mobile-core`.
- Add app-class worksheet enforcement for every new mobile app.
- Standardize generated binding scripts and CI jobs per app.
- Add fixture replay across Rust, Swift, and Kotlin for each workflow.

## Non-Negotiable UX Principles

- Capture-first, not chat-first.
- Drafts are editable and visibly non-authoritative.
- Consent is explicit, typed, and revocable before submission.
- Minority and ambiguous signals must be preserved, not averaged away.
- Local and server state must be visually distinct.
- Offline mode must never blur "saved locally" with "submitted".
- Receipts and traces should be rendered as user-understandable provenance, not
  hidden diagnostics.

## Bottom Line

The right Quorum iOS app is a native, trustworthy, field-ready capture and
collaboration surface. Its advantage is not that it embeds more server logic on
the phone. Its advantage is that it uses the phone's sensors, permissions,
local AI, offline storage, and notification surfaces to capture better human
signals before the governed Reflective stack decides anything.

Keep the app small, native, typed, consent-forward, and server-authoritative.
Then reuse that shape across the portfolio.
