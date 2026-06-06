# iOS Native Shell Template

Use this structure when a product app is ready for an iOS shell:

```text
ios/
  App/
  Capture/
  PlatformAI/
  CoreBridge/
  Resources/
  Tests/
```

## Responsibilities

- `App/` owns SwiftUI navigation and app lifecycle.
- `Capture/` owns AVFoundation, PhotosUI, Vision, and Speech flows.
- `PlatformAI/` owns Foundation Models and Core ML adapters.
- `CoreBridge/` owns UniFFI-generated bindings and thin Swift adapters.
- `Tests/` includes platform tests plus fixture parity tests against Rust.

Do not put product invariants in Swift unless they are platform-specific.

