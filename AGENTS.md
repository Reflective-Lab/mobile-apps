# AGENTS - mobile-apps

This workspace owns native mobile product surfaces for Reflective. Treat it as a
new product-first mobile workspace, not as a continuation of the archived
Converge placeholders.

## Start Here

- Read `README.md`.
- Read `docs/adr/0001-native-swift-kotlin-shared-rust-core.md`.
- When working on a product app, read the corresponding app source repo
  instructions under `../marquee-apps/` or `../studio-apps/` before importing
  domain assumptions.
- If entering a nested Git repo under `archive/legacy-placeholders/`, read that
  repo's nearest `AGENTS.md` and treat it as independent.

## Architecture Rules

- Native SwiftUI and Kotlin/Compose own UI and platform services.
- Platform AI stays native when Apple or Google provides the first-class runtime.
- Rust owns shared application core, orchestration, deterministic logic,
  persistence contracts, schemas, embeddings, vector search, and portable
  preprocessing.
- UniFFI is the preferred Swift/Kotlin bridge once generated bindings are needed.
- Keep third-party mobile libraries out until a platform framework cannot do the
  job.

## Commands

- `just check` or `cargo test --workspace`
- `just ci` for the local equivalent of the GitHub Actions gate
- `just fmt` for Rust formatting

## Boundaries

- Do not move or delete archived placeholder repos unless explicitly asked.
- Do not add large AI model binaries to this repo.
- Do not add network-bound dependencies just to make a scaffold look complete.
- Keep mobile app charters aligned with `../marquee-apps/README.md` and
  `../studio-apps/README.md`.
