# ADR 0001: Native Swift/Kotlin Shells With Shared Rust Core

- Date: 2026-06-06
- Status: Accepted

## Decision

Reflective mobile apps will use native SwiftUI shells on iOS, native
Kotlin/Compose shells on Android, and shared Rust below the UI line.

Rust owns:

- business and product logic
- sync contracts
- deterministic replay and validation
- storage models
- vector search
- embeddings where portable local inference is practical
- AI orchestration
- schemas and FFI-facing DTOs

iOS owns:

- SwiftUI
- Foundation Models
- Core ML
- AVFoundation
- Vision
- Speech
- PhotosUI
- App Intents
- permissions and background tasks

Android owns:

- Kotlin and Compose
- CameraX
- Gemini Nano and Android GenAI APIs
- LiteRT
- ML Kit
- MediaPipe
- Media3
- permissions and background work

UniFFI is the preferred bridge from Rust into Swift and Kotlin. `swift-bridge`
can be evaluated for iOS-only cases, but the default should preserve one Rust
binding path across both mobile platforms.

## Options Considered

1. Native Swift/Kotlin with shared Rust core.
2. Flutter with shared Rust core.
3. Svelte/Tauri mobile with shared Rust core.

## Rationale

The target apps are AI-heavy and involve camera, microphone, offline operation,
local inference, and fast-changing Apple and Google AI stacks. Those are exactly
the surfaces where native platform APIs matter most.

Flutter remains a viable compromise if one UI codebase becomes more valuable
than first-class access to native AI and media APIs. Tauri mobile is not the
right primary shell for these apps because the hard work would still require
Swift and Kotlin plugins.

## Consequences

- Product apps will have separate iOS and Android shells.
- Shared behavior must move into Rust early to avoid duplicating product logic.
- Native apps call platform AI directly when it is the strongest implementation.
- Rust AI crates are added only when they support a concrete shared capability.
- Mobile app charters are organized by product family: Marquee and Studio.

## Follow-Up

- Build the first UniFFI bridge when Quorum mobile needs Swift/Kotlin calls into
  `mobile-core`.
- Add native iOS and Android shell projects only after the first Quorum mobile
  workflow is specified.
- Add Candle, tokenizers, FastEmbed, SQLite/SQLx, Tokio, and vector-search
  dependencies when an executable feature needs them.

