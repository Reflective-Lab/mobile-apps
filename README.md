# Reflective Mobile Apps

Native mobile surfaces for the Reflective stack.

This repository is the mobile product lab for Reflective. It exists to turn the
Marquee and Studio app portfolios into first-class iOS and Android experiences
without losing the governed Rust platform underneath them.

The direction is product-first and native-first:

- **SwiftUI** shells for iOS.
- **Kotlin/Compose** shells for Android.
- **Rust** below the UI line for shared application logic, replayable state,
  sync contracts, AI orchestration, storage models, embeddings, vector search,
  and portable preprocessing.
- **UniFFI** as the preferred bridge from Rust into Swift and Kotlin.
- **Platform AI first** whenever Apple or Google provides the best on-device
  runtime for camera, microphone, language, vision, and background work.

## Boundary

> Owns: native iOS (SwiftUI) + Android (Compose) capture surfaces, on-device AI preprocessing, consent gates, structured-packet handoff to the platform via UniFFI/Rust. Does NOT own: governance, fact promotion, billing semantics, product invariants — "It must not silently decide, promote facts, run product invariants, or bypass consent" (`mobile-apps/README.md:120-121`).

— Canonical claim: [Mobile Apps](https://github.com/Reflective-Lab/reflective/blob/main/KB/04-architecture/current-system-map.md#mobile-apps) in the boundary registry. Update there first; this README quotes that source.

## Why Mobile Exists

Reflective apps are not generic chat wrappers. They are governed work surfaces:
each app starts from a Job To Be Done, translates user intent into checkable
runtime contracts, runs specialist reasoning, and records what was admitted as
fact.

Mobile adds something the desktop cannot:

- field capture from voice, camera, image OCR, and quick actions
- offline local drafts and consent gates
- on-device summaries and extraction before data leaves the device
- native background queues
- presence in the moment where ambiguity, evidence, and decisions appear

The mobile app is the layer between raw human experience and governed logic. It
captures intent, normalizes it with local AI, asks for consent, then hands a
structured packet to the Rust/platform stack.

## The App Families

Mobile follows the same product split as the broader Reflective workspace.

### Marquee Apps

Marquee apps are thin, JTBD-oriented commercial proof surfaces. They are built
for specific, consequential work where the stack must show receipts.

Current Marquee candidates from `../marquee-apps/`:

| App | Mobile angle |
|---|---|
| `quorum-sense` | Field signal capture, participant input, live sensemaking, offline drafts |
| `atlas-integration` | Acquisition/integration rooms, diligence evidence, executive updates |
| `tally-escrow` | Conditional handoff, approval capture, bilateral proof packets |
| `vouch-lending` | Borrower/lender evidence capture and governed credit workflows |
| `scout-sourcing` | Supplier/site signals, field photos, qualification evidence |
| `plumb-execution` | Work execution capture, proof of completion, exception handling |
| `triage-keeper` | Incident intake, escalation signals, responder context |
| `catalyst-biz` | Business operating room, experiment and execution signals |
| `fathom-narrative` | Narrative evidence capture and corporate disclosure analysis |
| `warden-compliance` | Rule authoring, verdict capture, shadow-compliance evidence |

### Studio Apps

Studio apps are creative, research, notes, writing, and presentation products.
They own product voice, local state, capture workflows, and the user experience
around knowledge work.

Current Studio candidates from `../studio-apps/`:

| App | Mobile angle |
|---|---|
| `inkling-notes` | Local-first notes, camera OCR, speech notes, offline enrichment |
| `wolfgang-chat` | Research companion, grounded chat, reading and capture |
| `folio-editor` | Portable editorial review and information triage |
| `moosemen-writer` | Writing capture, revision prompts, voice notes |
| `wykkid-preso` | Presentation rehearsal, persuasion capture, field feedback |

## The Reflective Stack

Mobile apps consume the Reflective stack. They do not replace it.

| Layer | Responsibility | Mobile relationship |
|---|---|---|
| **Helms** | Trust transfer surface: operator control, receipts, readiness, user authority | Mobile gives operators and participants native capture/approval surfaces |
| **Axiom / Axioms** | Turns human jobs into governed runtime contracts: truths, verifier expectations, lineage, IntentPackets | Mobile helps gather the raw job clauses and evidence that Axiom can compile |
| **Organism** | Formation planning: which specialist team should reason, argue, simulate, and attempt the work | Mobile supplies structured context and human feedback, not ad-hoc agent control |
| **Converge** | Fixed-point execution, Suggestor runs, proposal promotion, facts, traces, and replay | Mobile submits consented proposals and renders admitted facts/traces |
| **Mosaic** | Reusable specialist capabilities: policy, models, ports, solvers, memory, analytics, SMT | Mobile can invoke or display Mosaic-backed capabilities through the platform path |
| **Runtime Runway** | Auth, distribution, deployment, secrets, storage, telemetry, runtime plumbing | Mobile relies on it for identity, sessions, sync, and production operations |
| **Commerce Rails** | Billing, entitlements, subscriptions, partner payouts, commercial ledger | Mobile consumes accepted entitlements; it does not invent billing semantics |

The north-star path is:

```text
human job / field signal
  -> mobile AI normalization and consent
  -> JTBD / Intent codec
  -> Axiom truth package and IntentPacket
  -> Organism formation selection
  -> Converge fixed-point execution
  -> Mosaic specialists where needed
  -> facts, traces, receipts, and replayable state
```

## The Mobile AI Layer

The mobile AI layer sits between UX and logic.

```text
SwiftUI / Compose UX
  -> native capture and platform AI
  -> consented structured draft
  -> UniFFI / Rust bridge
  -> shared Rust DTOs and orchestration
  -> platform logic and APIs
```

It is deliberately narrow. It may summarize, extract, rewrite, transcribe, and
preprocess. It must not silently decide, promote facts, run product invariants,
or bypass consent.

### iOS

iOS shells use native Apple frameworks:

- Foundation Models for local summarization, extraction, rewriting, structured
  output, and agent-style drafts.
- Core ML for custom models, embeddings, vision/audio models, and exported local
  transformers.
- AVFoundation, Vision, Speech, PhotosUI, App Intents, and background tasks for
  real mobile capture.

### Android

Android shells use native Google/Android frameworks:

- Gemini Nano and Android GenAI APIs for on-device generative workflows.
- LiteRT for custom local models and accelerator-backed inference.
- ML Kit, MediaPipe, CameraX, Media3, WorkManager, and Kotlin coroutines for
  native capture, media, OCR, realtime inference, and background sync.

### Rust

Rust owns the shared application surface:

- DTOs and replayable fixtures
- deterministic decisions
- AI routing policy
- sync contracts
- storage model
- embeddings/vector search where portable
- UniFFI-facing facade

## Current Shape

```text
apps/
  marquee/
    quorum-sense/          first mobile candidate and fixture harness
  studio/
    inkling-notes/         studio candidate
    wolfgang-chat/         studio candidate
crates/
  mobile-core/             portfolio contract and Quorum workflow fixture logic
  mobile-ai/               AI execution routing policy
  mobile-ffi/              UniFFI-facing facade
schemas/
  quorum-mobile.udl        planned UniFFI contract
templates/
  native-shells/
    ios/                   minimal SwiftUI template
    android/               minimal Compose template
docs/
  adr/                     architecture decisions
  architecture/            platform and product boundaries
```

The first concrete workflow is Quorum field signal capture:

```text
voice/text/photo input
  -> local draft
  -> user consent
  -> queued append event
  -> online sync to Quorum inquiry thread
```

See:

- `apps/marquee/quorum-sense/fixtures/field-signal-capture.v1.json`
- `schemas/quorum-mobile.udl`
- `docs/architecture/quorum-sense-boundaries.md`

## Commands

```sh
just check
just ci
```

Direct Rust:

```sh
cargo test --workspace --locked
```

Native template commands:

```sh
just ios-build
just ios-sim
just android-build
just android-sim
```

## CI/CD

GitHub Actions live in `.github/workflows/`.

- `ci.yml` runs Rust format, clippy, tests, docs, scaffold checks, and the iOS
  shell build.
- `release-preflight.yml` runs the same gates for tags and manual dispatches,
  then packages the current scaffold as an artifact.

Android CI should become a hard gate once the Android template has been verified
on a machine with JDK 17, Android SDK 35, and a working emulator.

## Repository Status

This repo is public and intentionally early. It is a product lab plus native
foundation, not a finished app store product.

The old Converge and Wolfgang mobile skeletons are preserved under
`archive/legacy-placeholders/` for reference only. They are not the current
product direction.
