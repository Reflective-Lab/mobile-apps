# Quorum Sense — Boundaries and Responsibilities

Where `marquee-apps/quorum-sense` ends and `mobile-apps/apps/marquee/quorum-sense`
begins. Sister doc to `native-ai-rust-core.md`, scoped to Quorum.

## The product spans two repos

Quorum Sense is one product served from two workspaces. Same domain, same
inquiry kernel, different surfaces.

```
~/dev/reflective/
├── marquee-apps/quorum-sense/             ← canonical product home
│   ├── crates/                            ← Rust domain + app + truths + suggestors
│   ├── apps/desktop/  (Svelte web SPA)    ← v1 paying surface (web-first)
│   └── (Tauri shell)                      ← dormant; revive only on product brief
│
└── mobile-apps/apps/marquee/quorum-sense/ ← native mobile shells
    ├── ios/   (SwiftUI)                   ← post-v1, field-capture jobs
    ├── android/ (Kotlin/Compose)          ← post-v1, field-capture jobs
    └── fixtures/                          ← shell-only test data
```

Treat `marquee-apps/quorum-sense` as the canonical home. Mobile shells exist to
serve mobile-specific jobs the web cannot do, not to re-host what the web
already covers.

## What the marquee-apps repo owns (and mobile must not duplicate)

The Rust crates under `marquee-apps/quorum-sense/crates/` are the single source
of truth for:

- `InquiryThread`, `Signal`, `Hypothesis`, `Probe`, `Branch`, `Synthesis` and
  every other domain type. No parallel Swift or Kotlin structs.
- `InquiryContract`, `DecisionRule`, `RoundVisibility`, `ActorKind`, all
  enums — values flow over the wire, mobile renders them, mobile never invents
  new variants.
- The inquiry kernel: round admission, Lamport ordering, Merkle chain,
  EventLog, snapshot rehydration, sealed-round pseudonymization, seal reveal,
  contract amendment, decision-rule enforcement.
- `Suggestor` implementations: `ManifoldSignalExtractor` (with the EXP-001
  applicability-field pattern), `ProbeGenerationSuggestor`,
  `FuzzyEvaluationSuggestor`, `SkepticSuggestor`, synthesis suggestors.
- `quorum-truths` rulebooks (e.g. `tough-decision-v1.fuzzy.json`) and the
  Mamdani evaluation glue against `prism::fuzzy`.
- All receipt-bearing artifacts: `IntegrityProof`, `ProcessReceipt`,
  `QuorumOutcome`, `ActivatedRule` traces.
- HTTP/SSE API: `/quorum/inquiry/:id/{live,signals,rounds/next,decision,
  status,integrity,process-receipt,outcome}` plus the v1 surface (`/api/*`
  for Stripe + auth).
- v1 web SPA: landing, Stripe Checkout, host console, participant join page,
  result page. Served by the same Rust axum process for v1.
- The membership-gated admission path. Host access requires an active
  Reflective Labs entitlement, checked against `commerce-rails` on session
  start. Quorum is not the commercial authority — Stripe, subscriptions,
  and entitlement grants live in `commerce-rails`. Mobile never touches
  Stripe directly; it consumes the same entitlement check.

## What `mobile-apps/apps/marquee/quorum-sense` owns

Only the things native platforms do better than a web browser. Following
`native-ai-rust-core.md`:

iOS:

- SwiftUI participant and facilitator views.
- Foundation Models for **on-device draft normalization** of a captured signal
  before it leaves the device (the draft, not the canonical extraction —
  canonical extraction stays server-side through `ManifoldSignalExtractor`).
- Speech for live transcription of a verbal signal.
- AVFoundation and PhotosUI for camera/photo signal capture and consent UI.
- App Intents for "capture a Quorum signal" quick actions.
- Background tasks for queued offline submission.

Android:

- Compose participant and facilitator views.
- Gemini Nano / ML Kit GenAI for on-device draft normalization.
- CameraX for capture.
- ML Kit for OCR / document signal extraction.
- WorkManager for queued offline submission.

Shell-owned state is allowed only when it is **shell-local** (UI selection,
draft buffer, pending offline queue). Anything that would change the inquiry
chain goes through the canonical Rust API.

## What mobile must not own

- Domain types. Use UniFFI-generated bindings from `mobile-apps/crates/`
  (post-v1) or, until then, code-generated DTOs from the OpenAPI surface.
- The inquiry kernel. Mobile never decides a round close, applies a decision
  rule, advances Lamport, computes a Merkle root, or evaluates the fuzzy
  rulebook locally.
- Stripe. Payment is web-only for v1; if mobile ever sells, it goes through
  Apple/Google IAP and a server-side reconciliation — not direct Stripe.
- Fuzzy rule logic. `tough-decision-v1.fuzzy.json` and any future rulebook
  live in `quorum-truths`. Mobile renders the activated-rule trace; it does
  not recompute it.
- Synthesis, decision application, contract amendment. Server-only.

## How the surfaces share state

For v1, mobile is not in scope. When it is added:

- Transport: same HTTP/SSE the web uses. SSE for `live`, REST for everything
  else. Mobile reuses the SSE event vocabulary
  (`round.started`, `signal.received`, `hypothesis.formed`, …) verbatim.
- Auth: same Firebase ID-token + `commerce-rails` entitlement-check
  pattern the web uses. Native shells sign in via the Firebase SDK
  (anonymous for participants, member sign-in for host); tokens flow as
  bearer strings; `runway-auth` verifies them server-side. Mobile stores
  tokens in Keychain / EncryptedSharedPreferences.
- DTOs: generated from the server's contract. Preferred path is UniFFI over
  the shared Rust crates once `mobile-ffi` exposes the relevant surface. Until
  then, generated from OpenAPI / typed JSON.
- Offline: mobile queues signal drafts locally; submits when network returns;
  optimistic admission receipts reconcile the chain on re-sync. The chain
  itself is never branched on the device.

## v1 implication

For the first paying v1 (per the v1 design doc in
`marquee-apps/quorum-sense/docs/superpowers/specs/`), mobile product delivery is
deliberately out of scope. The web SPA is the entire paying surface. The mobile
workspace may still carry shell scaffolds, fixtures, bridge contracts, and smoke
apps so the native pipeline is ready. Those files are not a second Quorum
implementation.

SwiftUI/Compose may exist here only for shell proof work: capture, local draft,
consent, queue, and bridge round trips. No inquiry kernel, rule evaluation,
billing, admission, or server-owned domain semantics may move into mobile until
the paid mobile-specific job is named and reviewed.

When that brief lands, the first mobile feature is the field-capture path
described in `apps/marquee/quorum-sense/README.md`: "capture a field signal
from text, voice, or photo, create a local structured draft, let the user
consent, then append it to a Quorum inquiry thread when online." That feature
fits cleanly inside the boundaries above — it adds capture surfaces, not new
domain.

## Versioning

- Rust domain crates in `marquee-apps/quorum-sense/crates/` are the floor.
  When their public surface changes, mobile shells (when they exist) pin to
  the new version explicitly. No mobile-only domain forks.
- `mobile-apps/Cargo.toml` patches the Quorum crates to local paths during
  development, the same way `marquee-apps/quorum-sense/Cargo.toml` patches
  the platform crates today. CI rebuilds both workspaces against the
  patched-in floor.

## Cross-references

- `native-ai-rust-core.md` — the general native vs Rust boundary rule.
- `mobile-apps/docs/adr/0001-native-swift-kotlin-shared-rust-core.md` — the
  ADR that established the native-first direction.
- `mobile-apps/docs/adr/0002-mobile-platform-boundary.md` — portfolio placement
  rule; this doc is the Quorum reference instance.
- `marquee-apps/quorum-sense/CAPABILITIES.md` — what Quorum owns vs consumes
  from the platform; this doc extends that table to the mobile surface.
- `marquee-apps/quorum-sense/MILESTONES.md` — current milestone state; v1
  scope and the M4 participant-surface item drive what mobile cannot
  duplicate.
