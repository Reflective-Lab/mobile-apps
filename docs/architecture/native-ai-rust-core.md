# Native AI And Rust Core Boundary

## Boundary Rule

Use native platform AI when the task depends on vendor runtimes, camera,
microphone, permissions, device acceleration, or app OS integration. Use Rust
when the task is shared product logic, deterministic orchestration, portable
preprocessing, sync, storage, embeddings, vector search, or replayable state.

## iOS

Native iOS code owns:

- Foundation Models for on-device generative workflows, structured extraction,
  rewriting, summarization, tool-style flows, and agent experiences.
- Core ML for custom models, embeddings, vision/audio models, and exported local
  transformers.
- Vision for OCR, barcode scanning, object detection, face detection, and visual
  feature extraction.
- Speech for live transcription and speech-to-text.
- AVFoundation and PhotosUI for camera, microphone, capture sessions, and media
  library access.
- App Intents and background tasks for OS-level integration.

## Android

Native Android code owns:

- Gemini Nano and Android GenAI APIs for on-device generative workflows.
- LiteRT for custom local models and accelerator-backed inference.
- ML Kit for OCR, translation, barcode scanning, face detection, and GenAI
  helper APIs.
- MediaPipe for pose, face landmarks, hands, and realtime camera inference.
- CameraX for camera.
- Media3 for media and audio pipelines.
- WorkManager and platform services for background work.

## Rust

Rust owns:

- product/domain models
- deterministic decisions
- validation and invariants
- sync protocols
- storage model
- local memory index contracts
- vector search orchestration
- embeddings where a portable runtime is practical
- AI provider routing
- offline replay
- shared fixtures and tests

## Dependency Timing

Do not add AI dependencies for optics. Add them when the first feature needs an
executable API and tests:

- `tokio` for async Rust services and background orchestration.
- `serde` for FFI-safe serialization and fixtures.
- `sqlx` or SQLite bindings for local persistence.
- `candle` for portable Rust inference.
- `tokenizers` for model tokenization.
- `fastembed` for local embeddings.
- Qdrant client or an embedded vector alternative when the memory architecture
  is selected.

