# MILESTONES — Actual Tasks

- Date: 2026-06-22
- Roadmap source: `EPIC.md`
- Architecture source: `docs/adr/0002-mobile-platform-boundary.md`
- Reference report:
  `docs/architecture/quorum-ios-native-ai-collaboration-report.md`

This file is the execution backlog. Keep strategic direction in `EPIC.md`; keep
this file concrete, testable, and task-oriented.

## Status Legend

- `[x]` done
- `[ ]` open
- `Blocked:` cannot start until the named dependency exists
- `Acceptance:` the concrete condition that makes the task done

## Standing Guardrails

- Mobile captures, drafts, asks consent, queues, syncs, and renders receipts.
- Marquee mobile never promotes facts, mutates Lamport/Merkle chains, evaluates
  rulebooks as authority, owns billing, or stores Mosaic credentials.
- Swift/Kotlin own native UX and platform services.
- Rust owns shared workflow contracts, deterministic validation, queue/sync
  state, replay fixtures, capability policy, and FFI facades.
- Product domain logic comes from canonical product repos or generated server
  contracts, not from mobile-specific forks.

## M0 — Foundation Already In Place

- [x] M0.1 Root mobile architecture documented in `README.md`.
- [x] M0.2 Native Swift/Kotlin + shared Rust decision accepted in ADR 0001.
- [x] M0.3 Device/server authority boundary accepted in ADR 0002.
- [x] M0.4 Quorum boundary responsibilities documented in
  `docs/architecture/quorum-sense-boundaries.md`.
- [x] M0.5 Quorum fixture exists at
  `apps/marquee/quorum-sense/fixtures/field-signal-capture.v1.json`.
- [x] M0.6 Canonical UniFFI schema exists at `schemas/quorum-mobile.udl`.
- [x] M0.7 Quorum product FFI crate exists at
  `apps/marquee/quorum-sense/ffi/`.
- [x] M0.8 iOS Quorum app target exists under
  `apps/marquee/quorum-sense/ios/`.
- [x] M0.9 Android Quorum Compose shell exists under
  `apps/marquee/quorum-sense/android/`.
- [x] M0.10 Local verification passed on 2026-06-22:
  `cargo test --workspace --locked`.
- [x] M0.11 Local scaffold verification passed on 2026-06-22:
  `bash scripts/check-mobile-scaffold.sh`.

## M1 — Stabilize Build, CI, And Guardrails

Goal: make the current repo state honest and keep future mobile work inside the
accepted boundaries.

Status: **Done — 2026-06-24.** CI is the live ground truth (separate Quorum
product jobs for iOS/Android plus distinct template smoke-shell jobs), the
release preflight mirrors it, and three guardrails are now enforced in CI/scripts
rather than asserted: the dependency boundary (`cargo deny check bans`, EPIC 4),
the security-advisory gate (`cargo deny check advisories`), and the authority
leakage scan (M1.6, below). Also folded in `v0.1.0` were the supply-chain and
PII-boundary hardening tracked as `QF-2026-06-24-01..05`.

- [x] M1.1 Remove stale iOS SwiftPM CI jobs that reference
  `apps/marquee/quorum-sense/ios/QuorumMobileIOS`.
  Acceptance: `rg "QuorumMobileIOS" .github MILESTONES.md` only finds this task
  until the task is removed or marked done. Verified: only this task matches.
- [x] M1.2 Add a real Quorum product iOS CI job.
  Acceptance: CI runs `just quorum-ios-gen` and `just quorum-ios-build` on
  macOS for `apps/marquee/quorum-sense/ios/`. Done: job "Quorum iOS Product App".
- [x] M1.3 Keep generic template smoke-shell CI separate from Quorum product CI.
  Acceptance: CI has distinct job names for template smoke shells and the Quorum
  product app. Done: "iOS/Android Smoke Shell" vs "Quorum iOS/Android" jobs.
- [x] M1.4 Update release preflight to match CI.
  Acceptance: release preflight validates Rust gates, scaffold checks, and the
  current native product path without stale SwiftPM references.
- [x] M1.5 Add a forbidden mobile dependency drift check.
  Acceptance: CI fails if mobile FFI/product crates depend on server-only
  crates such as `converge-kernel`, Manifold adapters, Embassy live ports, or
  Mosaic credential-bearing crates without an ADR exception. Done: `deny.toml`
  `[bans]` + CI "Dependency boundary" step.
- [x] M1.6 Add a scaffold check for Quorum authority leakage.
  Acceptance: mobile Quorum source cannot introduce round-close, rulebook,
  Lamport, Merkle, Stripe, or entitlement-authority code without updating the
  boundary docs and an ADR. Done: `scripts/check-mobile-scaffold.sh` scans
  mobile source for those tokens and fails CI; relaxing it requires an ADR.
- [x] M1.7 Document generated artifact expectations.
  Acceptance: iOS XCFramework/Swift bindings and Android JNI/Kotlin bindings
  are clearly documented as generated and gitignored. Done: nested `.gitignore`
  in `ios/` and `android/`; `apps/marquee/quorum-sense/README.md` documents them.
- [x] M1.8 Re-run local gates after CI changes.
  Acceptance: `cargo test --workspace --locked` and
  `bash scripts/check-mobile-scaffold.sh` pass.

## M2 — Quorum FFI And Canonical Domain Seam

Goal: stop treating fixture behavior as product logic and define the real mobile
seam to canonical Quorum.

- [ ] M2.1 Rename or isolate fixture-only Rust behavior.
  Acceptance: hardcoded Quorum draft behavior in `crates/mobile-core/src/quorum.rs`
  is clearly under a fixture/test/demo module or replaced by canonical domain
  delegation.
- [ ] M2.2 Define the product capture submission DTO.
  Acceptance: DTO includes inquiry id, participant/session id, modality, raw
  capture reference, AI draft, user edits, redactions, consent decision,
  captured-at timestamp, idempotency key, and client app version.
- [ ] M2.3 Define typed Quorum mobile errors.
  Acceptance: FFI exposes typed errors for invalid modality, invalid consent,
  invalid confidence, missing inquiry, stale app contract, unavailable
  capability, and sync rejection.
- [ ] M2.4 Productize the `spikes/quorum-domain-mobile/` learning.
  Blocked: canonical `../marquee-apps/quorum-sense` checkout or published
  Quorum domain crate must be available.
  Acceptance: mobile FFI delegates deterministic validation to canonical Quorum
  domain logic or generated server contracts.
- [ ] M2.5 Decide product crate distribution.
  Acceptance: ADR or doc states whether mobile consumes Quorum domain crates by
  published crate, git dependency, vendoring, or workspace checkout.
- [ ] M2.6 Add full fixture replay tests at the FFI boundary.
  Acceptance: Rust test loads `field-signal-capture.v1.json`, runs the FFI
  workflow, and checks all expected draft/event fields.
- [ ] M2.7 Add generated-binding smoke tests where practical.
  Acceptance: iOS and Android generated bindings can call workflow id, draft,
  and append functions in a minimal build/test path.

## M3 — Quorum iOS Native Capture Slice

Goal: ship the first real iOS-native Quorum workflow: speech/text capture →
editable draft → explicit consent.

- [ ] M3.1 Create iOS feature folders for `Consent/`, `OfflineQueue/`, and
  `Realtime/`.
  Acceptance: Quorum iOS source structure matches the product surfaces named in
  the architecture report.
- [ ] M3.2 Replace hardcoded inquiry id with an injected session/context value.
  Acceptance: `SignalCaptureView` does not own a permanent fixture inquiry id.
- [ ] M3.3 Add speech permission request flow.
  Acceptance: user sees native permission request and a clear unavailable state
  when permission is denied or restricted.
- [ ] M3.4 Add speech transcript capture.
  Acceptance: iOS can capture a voice signal into a transcript draft using
  native Speech/AVFoundation APIs.
- [ ] M3.5 Add text capture as a first-class path.
  Acceptance: text capture uses the same state machine as speech capture, not a
  separate demo-only path.
- [ ] M3.6 Add local draft normalization hook.
  Acceptance: `PlatformSignalExtractor` produces a structured draft input from
  native capture output, with typed fallback when platform AI is unavailable.
- [ ] M3.7 Add draft review screen.
  Acceptance: raw capture, summary, contradiction/tension, confidence,
  uncertainty, and redactions are visible and editable before consent.
- [ ] M3.8 Add typed consent decisions in Swift.
  Acceptance: Swift models accepted, edited-and-accepted, rejected,
  saved-private, and expired without raw strings.
- [ ] M3.9 Add save-private and discard actions.
  Acceptance: user can keep a draft local or delete it without creating a sync
  event.
- [ ] M3.10 Add consented packet creation through the bridge.
  Acceptance: only a consented or edited-and-consented draft can become a queued
  packet.

## M3A — Quorum AI Director UX Slice

Goal: make Quorum mobile feel like an AI Director, not a dashboard. This milestone
rides alongside M3: start with preview data if needed, then align the canonical
director contract with Helms (`director-contracts` / `helm-client`) and expose it
through the Quorum FFI. `mobile-core` consumes/re-exports the contract and owns
mobile snapshot envelopes and replay fixtures; it does not define a parallel
`DirectorFrame`.

Architecture source:
`../KB/04-architecture/2026-06-27-ai-director-mobile-ux-architecture.md`
and root epic `../KB/08-roadmap/2026-06-27-ai-director-ux-epic.md`.

- [x] M3A.1 Add Quorum DirectorFrame fixture.
  Acceptance: `apps/marquee/quorum-sense/fixtures/` contains a canonical spine
  input event fixture (SessionPush / gate / session context shape) and a derived
  DirectorFrame JSON fixture for a decision checkpoint. The DirectorFrame is
  treated as a projection, not hand-authored standalone truth.
- [x] M3A.2 Add Swift value types for Director snapshots.
  Acceptance: iOS models `DirectorFrame`, `DirectorPrompt`, `Choice`,
  `BlockingState`, `ContextLevel`, and action intent tokens as typed Swift
  values mirroring the Helms contract; no raw strings in view logic beyond
  boundary mapping.
- [x] M3A.3 Add `DirectorNowView` in SwiftUI.
  Acceptance: iOS can render the Morning Director / Single Task state from the
  fixture or preview bridge with one primary action and no dashboard navigation.
- [x] M3A.4 Add `JudgmentPromptView` in SwiftUI.
  Acceptance: iOS renders a focused question with at most three choices and a
  single submit action.
- [x] M3A.5 Add `GatePromptView` in SwiftUI.
  Acceptance: iOS renders an explicit blocking gate with consequence, deadline,
  and bounded choices derived from `GatedDecision` / `GateCondition`. No UI-only
  "later" verdict is allowed; defer must exist in the Helms contract before the
  UI can send it.
- [x] M3A.6 Add context escape affordance.
  Acceptance: task-level UI exposes local context/session/formation levels as an
  explicit escape hatch, but the default screen starts at the task.
- [x] M3A.7 Route director actions as intents, not direct mutation.
  Acceptance: UI sends typed intents such as open task, submit judgment, approve
  gate, reject gate, request context; each intent maps to Helms/client action
  vocabulary, and view state updates only through a snapshot.
- [ ] M3A.8 Align DirectorFrame with Helms `director-contracts`.
  Status: **In progress** — `mobile-core` re-exports `director-contracts`;
  Swift/Kotlin domain structs track the canonical shape; flat enums
  (`GateVerdict`, `ContextLevel`, …) now come from UniFFI (M3A.10). Tagged
  shapes (`WaitingFor`, `DirectorPrompt`, `DirectorIntent`) remain host-side
  until a future UDL interface-enum pass.
  Acceptance: provisional Swift/Kotlin mirrors track the canonical
  `DirectorSnapshot { version, frame }` shape; replace with UniFFI-generated or
  mapped types once Quorum FFI exposes the contract (M3A.10). `helm-client`
  remains the projector from ordered spine events.
- [ ] M3A.9 Add mobile-core snapshot envelope and replay harness.
  Interim: `crates/mobile-core/src/director.rs` re-exports `director-contracts`,
  deserializes the golden snapshot fixture, and exposes
  `gate_condition_wire_label` for UniFFI; live projection via `helm-client` +
  `QuorumDomainPresenter` is follow-up.
  Acceptance: `crates/mobile-core` carries an immutable versioned mobile snapshot
  envelope that includes the Helms DirectorFrame and derives its version from the
  upstream SSE sequence; golden replay tests project canonical spine fixtures
  into DirectorFrame.
- [x] M3A.10 Expose director snapshot through Quorum FFI.
  Acceptance: `apps/marquee/quorum-sense/ffi` emits the fixture-backed
  `DirectorSnapshot` through UniFFI (`quorum_current_director_snapshot`,
  `quorum_submit_director_intent`); Swift/Kotlin production bridges map wire
  DTOs at `DirectorBridgeMapping`.
- [x] M3A.11 Draft Android Compose parity screen.
  Acceptance: Android renders the same fixture as a Compose Now screen with
  semantic parity, not necessarily pixel parity.
- [x] M3A.12 Wire live Director snapshot fetch against Quorum HTTP.
  Acceptance: `quorum_configure_director_api` + `GET /api/director/snapshot`
  resolve through `mobile-core` using canonical `DirectorSnapshot`; fixture
  fallback when the route is absent or unreachable; intent submit stays local
  until Plan 2 session push; DEBUG builds default to
  `http://127.0.0.1:5161/quorum-sense` with Bearer `dev`.

## M4 — Shared Consent, Offline Queue, And Sync Core

Goal: make offline operation durable and reusable across the portfolio.

- [x] M4.1 Define Rust `ConsentDecision`.
  Acceptance: consent states are explicit typed variants, not booleans or
  strings. Done: `crates/mobile-core/src/consent.rs` with
  `Accepted` / `EditedAndAccepted` / `Rejected` / `SavedPrivate` / `Expired`;
  `quorum::append_after_consent` gates queue on `permits_queue()`.
- [x] M4.2 Define Rust `CapturePacket`.
  Acceptance: packet captures modality, source metadata, draft payload,
  consent record, idempotency key, and app/workflow version. Done:
  `crates/mobile-core/src/capture.rs`; Quorum builders in
  `quorum::capture_packet_from_draft` / `append_from_capture_packet`.
- [x] M4.3 Define Rust `QueueState`.
  Acceptance: state machine covers draft-local, pending-consent, queued,
  submitting, admitted, rejected, needs-review, and abandoned. Done:
  `crates/mobile-core/src/queue.rs` with `QueueState`, `QueuedCapture`, and
  `quorum::queue_capture_from_draft`.
- [x] M4.4 Define allowed queue transitions.
  Acceptance: tests reject illegal transitions such as pending-consent →
  submitting or rejected → admitted without explicit retry/review. Done:
  `queue/transitions.rs` with `allows_transition_to` / `transition_to` and
  `QueuedCapture::transition_to`.
- [ ] M4.5 Define local persistence contract.
  Acceptance: Rust exposes stable records; native code can store them without
  learning product internals.
- [ ] M4.6 Implement iOS durable queue adapter.
  Acceptance: queued packets survive app termination and relaunch.
- [ ] M4.7 Add BGTaskScheduler submission hook.
  Acceptance: queued packets can be retried in background when iOS allows it.
- [ ] M4.8 Define HTTP submission client boundary.
  Blocked: canonical Quorum server API contract must be available in this
  checkout or generated into this repo.
  Acceptance: mobile submits through the canonical Quorum API, not a new
  mobile-specific transport.
- [ ] M4.9 Add admission receipt reconciliation.
  Acceptance: local queued/submitting state changes only after server response
  or receipt reconciliation.
- [ ] M4.10 Add replay tests for queue behavior.
  Acceptance: Rust tests cover offline, retry, duplicate idempotency key, server
  rejection, and needs-review flows.

## M5 — Android Parity

Goal: make Android equal in architecture, even if individual native APIs differ.

- [ ] M5.1 Add Kotlin value/domain wrappers.
  Acceptance: `ConsentState`, `AppendEventType`, `SyncState`, and `Confidence`
  are typed; raw strings/floats are mapped only at the boundary.
- [ ] M5.2 Add Android `QuorumCoreBridgeFFI`.
  Acceptance: product app can use generated UniFFI Kotlin bindings instead of
  `PreviewQuorumCoreBridge`.
- [ ] M5.3 Add Android product FFI build command.
  Acceptance: a `just` command builds Quorum Android JNI libraries and Kotlin
  bindings for the product app.
- [ ] M5.4 Add Android product build CI.
  Acceptance: CI assembles the Quorum Android product app, not only the generic
  template.
- [ ] M5.5 Add Compose consent review screen.
  Acceptance: Android supports the same consent decisions as iOS.
- [ ] M5.6 Add WorkManager queue submission hook.
  Acceptance: Android queued packets survive process death and retry in
  background.
- [ ] M5.7 Add native capture path selection.
  Acceptance: text, voice transcript, and image/OCR capture flow into the same
  bridge contract.

## M6 — Capability-Aware Compute Placement

Goal: make local/server AI placement explicit, inspectable, and policy-driven.

- [ ] M6.1 Define Rust `CapabilitySnapshot`.
  Acceptance: snapshot includes platform, OS version, local model availability,
  permissions, network, battery, thermal state, storage, privacy mode, plan, and
  workspace policy inputs.
- [ ] M6.2 Define Rust `ComputePlacement`.
  Acceptance: placement variants cover local-required, local-preferred,
  server-required, server-preferred, unavailable, and ask-user.
- [ ] M6.3 Replace static `mobile-ai` routing.
  Acceptance: routing considers task, capability snapshot, privacy, and plan
  instead of only platform/task.
- [ ] M6.4 Add iOS capability probes.
  Acceptance: Swift probes platform AI availability, permissions, network,
  battery/thermal state where available, and passes typed values to Rust.
- [ ] M6.5 Add Android capability probes.
  Acceptance: Kotlin probes Gemini/ML Kit availability, permissions, network,
  battery/thermal state where available, and passes typed values to Rust.
- [ ] M6.6 Add user-visible fallback copy.
  Acceptance: UI explains when local AI, server AI, or manual-only mode is being
  used.

## M7 — Realtime Collaboration UX

Goal: render collective intelligence state without simulating server authority.

- [ ] M7.1 Define local vs server state labels.
  Acceptance: UI has distinct visual states for local draft, queued, submitting,
  server-admitted, rejected, and receipt-rendered.
- [ ] M7.2 Add Quorum live event model.
  Blocked: canonical SSE event contract must be available.
  Acceptance: mobile models server events such as round started, signal
  received, hypothesis formed, receipt updated, and outcome ready.
- [ ] M7.3 Add iOS live inquiry renderer.
  Acceptance: iOS can render server-computed status without local promotion.
- [ ] M7.4 Add facilitator field mode.
  Acceptance: facilitator can see missing evidence, unresolved tensions, queued
  participant signals, and mobile-safe review actions.
- [ ] M7.5 Preserve minority/ambiguous signals in UI.
  Acceptance: UX does not collapse or hide unresolved contradictions after draft
  creation or server sync.
- [ ] M7.6 Add notification/app intent plan.
  Acceptance: document and scaffold quick capture, review reminders, and queued
  submission status without bypassing consent.

## M8 — Portfolio App Pattern

Goal: make the Quorum pattern reusable for every Reflective mobile app.

- [ ] M8.1 Add app classification worksheet template.
  Acceptance: new apps must declare marquee governed, studio local-first, or
  studio hybrid before implementation.
- [ ] M8.2 Add app README template.
  Acceptance: template includes mobile job, authority, domain source, transport,
  native platform APIs, Rust crates, fixtures, and non-goals.
- [ ] M8.3 Standardize fixture naming.
  Acceptance: fixtures use `<workflow>.v<version>.json` and include input,
  native AI expectations, Rust behavior, consent, queue, sync, and forbidden
  behavior.
- [ ] M8.4 Align `apps/registry.txt` with Rust portfolio metadata.
  Acceptance: app registry and `reflective-mobile-core` cannot drift silently.
- [ ] M8.5 Add new-app bootstrap checklist.
  Acceptance: checklist covers source repo instructions, domain footprint audit,
  FFI crate, native shells, fixtures, CI, and boundary review.
- [ ] M8.6 Add first non-Quorum worksheet.
  Acceptance: Inkling Notes or Wolfgang Chat has a filled mobile classification
  and dependency-footprint note before product code is added.

## M9 — Release, Privacy, And Operations

Goal: prepare mobile apps for real users and app-store distribution.

- [ ] M9.1 Add token storage plan.
  Acceptance: iOS uses Keychain; Android uses EncryptedSharedPreferences or an
  equivalent platform-backed secure store.
- [ ] M9.2 Add capture data retention policy.
  Acceptance: docs say what remains local, what syncs, what is deleted, and how
  rejected/private drafts are handled.
- [ ] M9.3 Add privacy manifest and permission copy review.
  Acceptance: iOS permission strings and privacy metadata match actual capture
  behavior.
- [ ] M9.4 Add crash/diagnostics policy.
  Acceptance: telemetry cannot include raw captures, transcripts, photos, or
  private drafts by default.
- [ ] M9.5 Add TestFlight release checklist.
  Acceptance: signing, bundle id, archive, upload, tester notes, and rollback
  steps are documented.
- [ ] M9.6 Add Play internal testing checklist.
  Acceptance: signing, application id, release build, upload, tester notes, and
  rollback steps are documented.
- [ ] M9.7 Add billing/entitlement boundary note for mobile.
  Acceptance: docs state mobile consumes server-accepted entitlements and does
  not implement direct Stripe semantics; any Apple/Google IAP work requires
  server reconciliation.

## Backlog — Not Yet Scheduled

- [ ] Add PhotosUI + Vision OCR path on iOS after speech/text path works.
- [ ] Add CameraX + ML Kit OCR path on Android after FFI parity works.
- [ ] Add App Intents quick capture after consent model is implemented.
- [ ] Add cross-platform fixture replay in Swift and Kotlin test targets.
- [ ] Add local vector/search memory only after a real workflow requires it.
- [ ] Evaluate portable embeddings only after model size, privacy, and device
  support are specified.
- [ ] Add Studio local-first queue/sync variant after Inkling/Wolfgang worksheet
  is accepted.
