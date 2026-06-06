# Reflective Mobile Apps

This workspace is the mobile surface layer for Reflective products. The mobile
direction is now product-first and native-first:

- iOS apps use SwiftUI for UI, permissions, camera, microphone, background work,
  App Intents, Foundation Models, Core ML, Vision, Speech, PhotosUI, and
  AVFoundation.
- Android apps use Kotlin and Compose for UI, permissions, CameraX, background
  work, Gemini Nano, LiteRT, ML Kit, MediaPipe, Media3, and platform services.
- Shared Rust owns product/domain logic, sync contracts, storage models, vector
  search, embeddings, AI orchestration, deterministic replay, schemas, and
  portable preprocessing.
- UniFFI is the preferred bridge into Swift and Kotlin when the first native
  shell needs generated bindings.

The old Converge and Wolfgang mobile skeletons have been moved to
`archive/legacy-placeholders/`. They are retained for reference only and are not
the product direction.

## Layout

```text
apps/
  marquee/
    quorum-sense/       first Marquee mobile candidate
  studio/
    wolfgang-chat/      Studio mobile candidate
    inkling-notes/      Studio mobile candidate
crates/
  mobile-core/          shared product model and portfolio contract
  mobile-ai/            AI execution routing and orchestration policy
  mobile-ffi/           future UniFFI-facing facade over shared Rust
docs/
  adr/                  local mobile architecture decisions
  architecture/         implementation guidance and boundaries
templates/
  native-shells/
    ios/                SwiftUI shell contract
    android/            Kotlin/Compose shell contract
archive/
  legacy-placeholders/  preserved placeholder apps
```

## First Product Direction

Quorum is the first serious Marquee mobile candidate because mobile gives the
product a concrete job: live signal capture, voice/photo/text input, local
summaries, offline participant notes, and sensemaking while the user is in the
field.

Studio candidates are lower commitment for now:

- Wolfgang Chat: mobile research companion, reading, capture, and grounded chat.
- Inkling Notes: local-first capture with camera OCR, speech notes, and offline
  enrichment.

## Non-Goals

- Do not use Svelte/Tauri as the main mobile shell for AI-heavy camera and
  microphone surfaces.
- Do not force all AI through Rust. Use Apple and Google runtimes where they are
  first-class and keep Rust as the shared application core.
- Do not revive Converge mobile as the default direction unless a product brief
  makes it meaningful again.

## Commands

```sh
cargo test --workspace --locked
```

If `just` is installed:

```sh
just check
just ci
```

## CI/CD

The baseline workflows live in `.github/workflows/`:

- `ci.yml` runs Rust format, clippy, tests, docs, and scaffold checks on pushes
  and pull requests.
- `release-preflight.yml` runs the same gates for version tags and manual
  dispatches, then packages the current scaffold as a GitHub Actions artifact.

Native iOS and Android build jobs should be added after the first generated
UniFFI bindings are checked in.
