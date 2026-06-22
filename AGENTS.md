# AGENTS - mobile-apps

This workspace owns native mobile product surfaces for Reflective. Treat it as a
new product-first mobile workspace, not as a continuation of the archived
Converge placeholders.

## Start Here

- Read `README.md`.
- Read `docs/adr/0001-native-swift-kotlin-shared-rust-core.md`.
- Read `docs/adr/0002-mobile-platform-boundary.md` before scoping device vs
  server placement for any product mobile companion.
- Read `docs/adr/0003-responsiveness-and-snapshot-consistency.md` before
  designing any view/bridge/core interaction in a real-time app.
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

## Engineering Principles

Write idiomatic, type-safe, reliable code in Rust, Swift, and Kotlin. Spend
effort up front — at compile time — to remove whole classes of runtime failure.
Reserve abstraction and late binding for genuine domain variation; do not spend
it working around the language. Make illegal states unrepresentable rather than
validating them at runtime.

- Push correctness to the type system and the compiler. Prefer precise types,
  enums/sealed hierarchies, and newtype wrappers over primitives and strings
  ("stringly-typed" is a smell). Handle every case explicitly (exhaustive
  `match` / `switch` / `when`); avoid catch-all defaults that hide new variants.
- Never pass anonymous numbers or semantics-bearing strings through domain code.
  A bare `String`/`Int`/`Bool` standing for a status, kind, unit, or id is
  banned inside the domain — model it as an enum or newtype. Do the raw↔domain
  mapping **only at the boundaries** (FFI, persistence, network, parse layer):
  decode once at the edge into a domain type, then trust it everywhere inside.
  Domain functions take and return typed values and never re-parse strings.
  Parse, don't validate.
- Model errors as typed values, not panics or strings. **Rust:** `Result` +
  `thiserror`, no `unwrap`/`expect`/`panic!` outside tests; lean on the borrow
  checker, avoid `dyn`/trait objects unless polymorphism is real. **Swift:**
  value types, non-optional by default, `throws`/`Result`, no force-unwrap (`!`)
  or force-cast; honor strict concurrency (`Sendable`); avoid `Any`/`AnyObject`.
  **Kotlin:** `val` over `var`, non-null types, `data`/`sealed` classes, no `!!`;
  pin nullability of platform types at the boundary.
- The UniFFI seam is the highest-risk boundary — keep it strongly typed. Model
  failures as typed UniFFI errors (never error strings), validate at the edge so
  native code only ever receives well-typed values, and let the generated
  bindings carry the types through rather than re-casting on the native side.
- Keep the build strict and loud: `cargo clippy -- -D warnings` is the gate
  (see `just ci`); fix warnings, don't silence them. A failure caught by the
  compiler or CI is one fewer failure in a user's hands.

## Responsiveness & Consistency

Several apps are real-time collaboration (humans + on-device AI + a session
across other devices), so two properties are hard invariants, not goals. See
`docs/adr/0003-responsiveness-and-snapshot-consistency.md` for the full contract.

- **Responsiveness:** the interaction loop (input → next frame) is *never*
  coupled to unbounded-latency work — AI inference, network, disk, or merge.
  The Rust core runs as an off-main, actor-isolated component; the UI `await`s
  it. A `@MainActor` bridge calling synchronous FFI is forbidden — `async`
  without offloading still blocks the main thread and freezes the UI.
- **Snapshot consistency:** the view is a pure projection of immutable,
  versioned, internally-consistent snapshots — swapped atomically, applied
  monotonically by version. The UI never reads mutable shared state and can
  never observe a torn or half-merged world.
- **One linearization point:** all mutation (local user, local AI, remote peers)
  commits through the core in causal order; compute is parallel/off-main, the
  commit is serialized, each commit emits one coherent snapshot.
- **Intents in, events out:** UI sends fire-and-forget intents that return
  immediately; updates arrive only over a snapshot stream (UniFFI async +
  callback interface → Swift `AsyncStream`, Kotlin `Flow`). Never poll, never
  block on a result.
- **Cancellable and bounded:** superseded AI/network work is cancelled, input is
  coalesced, queues are bounded — degrade by shedding/merging stale work, never
  by stalling. On-device AI runs off the interaction path, streams partial
  results, and is cancellable.
- **Correct by type:** snapshots are coherent because the domain types make
  inconsistent states unrepresentable (above) — the core may only emit a
  type-valid snapshot. This is why the type-safety rules are load-bearing, not
  cosmetic.

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
