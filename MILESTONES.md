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

- [x] M2.1 Rename or isolate fixture-only Rust behavior.
  Done 2026-06-27: `refine.rs` Converge formation replaces hardcoded stub in
  `draft_field_signal`; fixture ideals remain in JSON for structural contract only.
- [x] M2.2 Define the product capture submission DTO.
  Done: `CapturePacket` + `CaptureSubmitRequest` in `capture.rs` / `sync.rs`
  (see ADR 0006).
- [x] M2.3 Define typed Quorum mobile errors.
  Done: `QuorumError` in UDL; modality/consent/event/sync are wire enums.
- [x] M2.4 Productize the `spikes/quorum-domain-mobile/` learning.
  Done 2026-06-27: v1 uses Converge refinement + portfolio types; incremental
  `quorum-domain` adoption tracked in ADR 0006 and spike doc.
- [x] M2.5 Decide product crate distribution.
  Done 2026-06-27: workspace path deps + `checkout-helms-deps` CI action (ADR 0006).
- [x] M2.6 Add full fixture replay tests at the FFI boundary.
  Done 2026-06-27: `ffi/tests/fixture_replay.rs` loads `field-signal-capture.v1.json`.
- [x] M2.7 Add generated-binding smoke tests where practical.
  Done: `ffi/tests/ffi_tests.rs` public surface + Android `CapturePipelineFlowSpec`.

## M3 — Quorum iOS Native Capture Slice

Goal: ship the first real iOS-native Quorum workflow: speech/text capture →
editable draft → explicit consent.

- [x] M3.1 Create iOS feature folders for `Consent/`, `OfflineQueue/`, and
  `Realtime/`.
  Done 2026-06-30: `ios/Consent/`, `ios/OfflineQueue/` (renamed from `Queue/`),
  `ios/Speech/`, `ios/Realtime/`.
- [x] M3.2 Replace hardcoded inquiry id with an injected session/context value.
  Done 2026-06-30: `CaptureSessionContext.inquiryThreadId()` from
  `QUORUM_INQUIRY_THREAD_ID` env; `SignalCaptureView` accepts injected id.
- [x] M3.3 Add speech permission request flow.
  Done 2026-06-30: `Speech/SpeechCaptureService.swift` requests Speech + mic;
  denied/restricted states surfaced in capture UI.
- [x] M3.4 Add speech transcript capture.
  Done 2026-06-30: live transcript via `SFSpeechRecognizer` + `AVAudioEngine`.
- [x] M3.5 Add text capture as a first-class path.
  Done 2026-06-30: unified capture form; text and voice share draft → review → consent flow.
- [x] M3.6 Add local draft normalization hook.
  Done 2026-06-30: `PlatformSignalExtractor.normalizeCapture` with typed trim fallback.
- [x] M3.7 Add draft review screen.
  Done 2026-06-30: `Consent/ConsentReviewView.swift` — editable summary, raw capture,
  latent need, contradiction, confidence.
- [x] M3.8 Add typed consent decisions in Swift.
  Done 2026-06-30: UniFFI `ConsentDecision` + `ConsentReviewView` actions.
- [x] M3.9 Add save-private and discard actions.
  Done 2026-06-30: save-private keeps session-local draft; reject/discard skip sync.
- [x] M3.10 Add consented packet creation through the bridge.
  Done 2026-06-30: only `Accepted` / `EditedAndAccepted` call append + durable queue persist.

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
- [x] M3A.8 Align DirectorFrame with Helms `director-contracts`.
  Done 2026-06-30: UniFFI interface-enum pass for `FfiWaitingFor`, `FfiDirectorPrompt`,
  and `FfiDirectorIntent` in `schemas/quorum-mobile.udl`; Swift/Kotlin bridge mapping
  updated for tagged wire enums; domain mirrors remain in `DirectorModels` at the
  view boundary.
- [x] M3A.9 Add mobile-core snapshot envelope and replay harness.
  Done 2026-06-30: `crates/mobile-core/src/director/replay.rs` exposes
  `MobileDirectorSnapshot` (version = upstream SSE sequence) and golden spine replay
  tests (`SessionPush` + `GateCondition` → `helm-client` → `DirectorSnapshot`).
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
  resolve through `mobile-core` using canonical `DirectorSnapshot`; Client Helm SSE
  projection via `quorum_wait_director_update`; intent submit POSTs to
  `POST /api/director/dev/intent` when configured (`LOCAL_DEV`); fixture fallback
  when unreachable; DEBUG defaults to `http://127.0.0.1:5161/quorum-sense` Bearer `dev`.

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
- [x] M4.5 Define local persistence contract.
  Acceptance: Rust exposes stable records; native code can store them without
  learning product internals. Done: `persistence.rs` with versioned
  `PersistedQueueRecord` JSON encode/decode and `persistence_round_trip`.
  Durability engine: native (ADR 0005); JSON is record encoding only.
- [x] M4.6 Implement iOS durable queue adapter.
  Acceptance: queued packets survive app termination and relaunch. Store
  `PersistedQueueRecord::to_json()` by `record_id`; reload via `from_json`;
  call Rust for transitions before write (ADR 0005). Done: UniFFI persistence
  helpers, `FileQueueStore` + `QuorumQueuePersistence`, bridge wiring, and
  relaunch tests in `QueuePersistenceTests`.
- [x] M4.7 Add BGTaskScheduler submission hook.
  Done 2026-06-30: `OfflineQueue/QueueBackgroundSubmit.swift` registers
  `se.reflective.quorum.queue-submit`; schedules after consent queue persist;
  bridge `submitEligibleQueueRecords()` drives Rust HTTP submit.
- [x] M4.8 Define HTTP submission client boundary.
  Done 2026-06-30: `mobile-core/src/sync.rs` defines `POST /api/capture/submit`,
  `CaptureSubmitRequest`, UniFFI `quorum_configure_capture_api` +
  `quorum_submit_persisted_queue_record`. Server handler shipped 2026-06-27 in
  `marquee-apps/quorum-sense` (`capture_submit.rs`, `runway.app.json`); E2E
  submit path is live when the Quorum server is running.
- [x] M4.9 Add admission receipt reconciliation.
  Done 2026-06-30: `AdmissionReceipt` + `reconcile_admission_receipt` /
  `quorum_reconcile_capture_admission`; local state advances only after receipt.
- [x] M4.10 Add replay tests for queue behavior.
  Done 2026-06-30: `crates/mobile-core/tests/queue_replay_tests.rs` covers offline
  persist, submit/admit, rollback, duplicate idempotency, rejection→review→retry.

## M5 — Android Parity

Goal: make Android equal in architecture, even if individual native APIs differ.

- [x] M5.1 Add Kotlin value/domain wrappers.
  Done: typed `Confidence`, UniFFI enums at boundary, domain wrappers in `capture/`.
- [x] M5.2 Add Android `QuorumCoreBridgeFFI`.
  Done: production bridge maps FFI DTOs in `QuorumCoreBridgeFFI.kt`.
- [x] M5.3 Add Android product FFI build command.
  Done: `just quorum-android-uniffi` / `just quorum-android-build`.
- [x] M5.4 Add Android product build CI.
  Done 2026-06-30: `android-product` job runs `just quorum-android-build`.
- [x] M5.5 Add Compose consent review screen.
  Done 2026-06-30: `consent/ConsentReviewScreen.kt` mirrors iOS consent actions.
- [x] M5.6 Add WorkManager queue submission hook.
  Done 2026-06-27: `QueueSubmitWorker` + `QueueBackgroundSubmit` enqueue on consent
  persist and app startup; network-constrained unique work with exponential backoff.
- [x] M5.7 Add native capture path selection.
  Done 2026-06-30: `PlatformSignalExtractor.normalizeCapture` + modality picker;
  text/voice/OCR share one bridge contract.

## M6 — Capability-Aware Compute Placement

**Epic:** M6

Goal: make local/server AI placement explicit, inspectable, and policy-driven.

Status: **In progress — 2026-06-27.** Refinement backend seam (M6.7–M6.10) is on
`main`; capability snapshot, placement policy, probes, and user-facing fallback
copy (M6.1–M6.6) remain open.

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
- [x] M6.7 Add `RefineBackend` + heuristic fallback in mobile-core.
  Done 2026-06-27: `refine.rs` — Converge formation with `HeuristicBackend`;
  `draft_field_signal_with_backend` for pluggable language work.
- [x] M6.8 Expose UniFFI `LlmBackend` callback for draft refinement.
  Done 2026-06-27: `schemas/quorum-mobile.udl` callback interface;
  `quorum_draft_field_signal_with_llm` in `quorum-ffi`; `LlmRefineBackend` per-field
  fallback to heuristic when the model returns nothing.
- [x] M6.9 Add local `quorum-refine-service` dev tool.
  Done 2026-06-27: `tools/quorum-refine-service/` — HTTP `POST /complete` cloud-fallback
  tier for simulator/emulator when no on-device model is available.
- [x] M6.10 Wire native cloud-fallback LLM on iOS and Android.
  Done 2026-06-27: `RefineServiceLlm.swift` / `RefineServiceLlm.kt` implement
  `LlmBackend`; production bridges pass into `quorum_draft_field_signal_with_llm`.
  Android `network_security_config.xml` permits localhost in debug.

## M7 — Realtime Collaboration UX

**Epic:** M6

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

**Epic:** M8

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

**Epic:** M10

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

**Epic:** M1

- [ ] Add PhotosUI + Vision OCR path on iOS after speech/text path works.
- [ ] Add CameraX + ML Kit OCR path on Android after FFI parity works.
- [ ] Add App Intents quick capture after consent model is implemented.
- [ ] Add cross-platform fixture replay in Swift and Kotlin test targets.
- [ ] Add local vector/search memory only after a real workflow requires it.
- [ ] Evaluate portable embeddings only after model size, privacy, and device
  support are specified.
- [ ] Add Studio local-first queue/sync variant after Inkling/Wolfgang worksheet
  is accepted.
