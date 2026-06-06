# Quorum Mobile Workflow Fixture V1

The first fixture shape describes a mobile capture workflow that can be replayed
across SwiftUI, Compose, and Rust.

Required fields:

- `id` - stable workflow identifier.
- `version` - integer fixture version.
- `input` - mobile capture context.
- `native_ai` - platform runtimes expected to produce the local draft.
- `rust_core` - shared Rust behavior and event contract.
- `expected` - replay expectations and forbidden behavior.

The current fixture is:

- `apps/marquee/quorum-sense/fixtures/field-signal-capture.v1.json`

