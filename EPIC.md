> **Archived 2026-07-02** — epics now live as Projects in Linear (Reflective team).
> This file is kept for historical context only. E11 = Helm Coordination,
> E12 = AI Director UX (resolved 2026-07-02).

# EPIC — Native Mobile Portfolio Roadmap

- Date: 2026-06-22
- Status: Active direction
- Scope: Reflective native mobile apps across iOS, Android, and shared Rust.
- Reference app: `apps/marquee/quorum-sense/`

## Product Thesis

Reflective mobile apps should be native, field-ready surfaces for capture,
consent, local AI preprocessing, offline resilience, realtime collaboration, and
proof rendering. They should not become thin chat wrappers or mobile clones of
desktop workbenches.

The durable rule:

```text
the device proposes
the user consents
the server promotes
the app renders receipts
```

For Quorum, this means the iOS app should help people capture live organizational
signals, preserve ambiguity, edit AI-generated drafts, explicitly consent, and
submit to the canonical Quorum server. The server remains authoritative for
fact promotion, inquiry chain mutation, rulebooks, receipts, entitlements, and
Mosaic-backed specialist work.

## Architecture Principles

- Native SwiftUI owns iOS UI, platform permissions, sensors, App Intents,
  background tasks, and Apple AI/runtime integration.
- Native Kotlin/Compose owns Android UI, permissions, sensors, WorkManager, and
  Google/Android AI/runtime integration.
- Rust owns shared workflow contracts, deterministic validation, queue/sync
  state machines, capability policy, replay fixtures, portable preprocessing,
  and UniFFI-facing facades.
- UniFFI is the default bridge for shared Rust into Swift and Kotlin.
- Platform AI stays native when camera, microphone, OS integration, acceleration,
  or vendor on-device models are the important capability.
- Canonical product domain logic comes from `../marquee-apps/` or
  `../studio-apps/`; mobile does not fork domain models.
- Marquee governed apps obey "device proposes; server promotes."
- Studio local-first apps may put authority on device only when the app charter
  explicitly says the local vault/store is authoritative.

## Long-Term Roadmap

### Epic 1 — Foundation And Build Pipeline

Build and maintain a repeatable native mobile pipeline:

- Rust workspace gates: format, clippy, tests, docs, scaffold checks.
- iOS XcodeGen product apps and smoke templates.
- Android Gradle product apps and smoke templates.
- UniFFI generation for iOS XCFrameworks and Android JNI/Kotlin bindings.
- CI jobs that validate both generic templates and real product apps.

Success means a new app can start with native shells, a Rust FFI crate, fixtures,
and build commands without inventing a repo shape.

### Epic 2 — Quorum Field Signal Capture

Make Quorum iOS the reference product implementation:

- Capture text, speech, and photo/OCR signals natively.
- Use local AI to normalize a draft before data leaves the device.
- Show raw capture and AI draft side by side.
- Let the user edit, redact, discard, save private, or consent.
- Queue consented packets offline.
- Submit through the canonical Quorum server API.
- Reconcile server admission receipts and live inquiry state.

Success means the mobile app improves signal capture without moving Quorum
authority onto the phone.

### Epic 3 — Shared Consent And Offline Queue Core

Extract a portfolio-wide Rust core for consent and offline operation:

- Typed consent decisions.
- Durable queue state machine.
- Idempotency and retry contracts.
- Server admission reconciliation.
- Replay fixtures for Swift, Kotlin, and Rust.
- Local/server state separation.

Success means every app gets the same consent and queue semantics while keeping
its product-specific domain separate.

### Epic 4 — Canonical Product Domain Reuse

Move from fixture-level DTOs to real product seams:

- Quorum mobile FFI delegates deterministic validation to canonical Quorum
  domain crates or generated server API types.
- Studio app mobile facades reuse mobile-clean product crates.
- Dependency audits prevent server-only crates from entering mobile builds.
- Published, pinned, or vendored product crate strategy replaces ad hoc
  cross-repo path assumptions.

Success means mobile shells do not duplicate domain logic in Swift, Kotlin, or
mobile-local Rust.

### Epic 5 — Capability-Aware Compute Placement

Make local/server compute placement explicit and typed:

- Native probes report platform, OS version, available local models, battery,
  thermal state, network, permissions, and storage conditions.
- Server policy contributes entitlement, subscription plan, workspace policy,
  privacy mode, data sensitivity, and load hints.
- Rust evaluates a typed `ComputePlacement`: local required, local preferred,
  server required, server preferred, unavailable, or ask user.
- UX explains fallback behavior when local AI is unavailable.

Success means Reflective uses local compute when appropriate without leaking
credentials, duplicating authority, or hiding plan/privacy constraints.

### Epic 6 — Realtime Collaborative Intelligence UX

Build mobile UX around collective intelligence, not generic chat:

- Preserve minority and ambiguous signals.
- Distinguish local drafts, queued events, server-admitted signals, and receipts.
- Render live inquiry status through server SSE or equivalent canonical transport.
- Add facilitator field mode for missing evidence, unresolved tensions, and
  mobile-safe approvals.
- Use notifications and App Intents for timely capture and review.

Success means mobile improves group sensemaking in the moments where evidence,
ambiguity, and disagreement appear.

### Epic 7 — Android Parity

Bring Android to the same architectural standard as iOS:

- Compose surfaces mirror product behavior, not necessarily pixel layout.
- Kotlin maps FFI output into typed values and fails on unknown core values.
- UniFFI bindings are generated and used by the product app.
- CameraX, Media3, ML Kit, Gemini Nano, and WorkManager are introduced only for
  concrete executable workflows.

Success means iOS and Android share Rust contracts and product semantics while
remaining idiomatic native apps.

### Epic 8 — Portfolio App Pattern

Turn Quorum learnings into a repeatable structure for all Reflective apps:

```text
apps/<family>/<app>/
  ios/
  android/
  ffi/
  fixtures/
  README.md
```

Reusable platform pieces live in `crates/mobile-core` and `crates/mobile-ai`.
Per-app authority, transport, and domain placement are recorded before coding.

Success means adding Atlas, Tally, Vouch, Scout, Inkling, Wolfgang, or another
mobile app starts from classification and domain audit, not from copy-pasted UI.

### Epic 9 — Studio Local-First Mobile

Apply the same native/Rust pattern to studio apps where device authority may be
correct:

- Inkling Notes: local-first vault capture, OCR, speech notes, local index.
- Wolfgang Chat: hybrid research capture on device with server-side panels where
  needed.
- Writing/presentation apps: voice capture, rehearsal review, local drafts, and
  optional cloud enrichment.

Success means studio apps feel native and local-first without pulling heavy
server/network dependencies into mobile builds by accident.

### Epic 10 — Release, Privacy, And Operations

Make mobile production-grade:

- Keychain and EncryptedSharedPreferences for tokens and secrets.
- Privacy manifests, permission copy, redaction, data-retention rules.
- TestFlight and Play internal testing pipelines.
- Crash, telemetry, and diagnostics that do not leak sensitive captures.
- App Store / Play policy compliance for AI, privacy, subscriptions, and account
  deletion.

Success means mobile can ship safely, not just demo locally.

## Non-Goals

- Do not embed the promotion-authoritative Converge loop in marquee mobile apps.
- Do not put Mosaic credentials or live specialist adapters on device.
- Do not create mobile-specific Quorum domain forks.
- Do not add large model binaries to this repo.
- Do not add third-party mobile libraries before a platform framework is proven
  insufficient.
- Do not create shared SwiftUI/Compose frameworks before real repetition proves
  the abstraction.

## Portfolio Definition Of Done

The portfolio pattern is mature when:

- Quorum iOS ships a real native field-capture workflow.
- Android reaches FFI and queue parity for the same workflow.
- `mobile-core` owns reusable consent, queue, sync, replay, and capability
  contracts.
- Each new app starts with app classification, domain footprint audit, and a
  thin product FFI.
- CI blocks forbidden server-only dependencies in mobile FFI crates.
- Swift, Kotlin, and Rust replay the same workflow fixtures.
- Local AI, consent, offline state, server admission, and receipts are visibly
  distinct in the UX.
