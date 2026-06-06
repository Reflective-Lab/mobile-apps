# Android Native Shell Template

Use this structure when a product app is ready for an Android shell:

```text
android/
  app/src/main/java/.../app/
  app/src/main/java/.../capture/
  app/src/main/java/.../platformai/
  app/src/main/java/.../corebridge/
  app/src/test/
```

## Responsibilities

- `app/` owns Compose navigation and app lifecycle.
- `capture/` owns CameraX, ML Kit capture helpers, MediaPipe, and Media3 flows.
- `platformai/` owns Gemini Nano, LiteRT, ML Kit GenAI, and model delegates.
- `corebridge/` owns UniFFI-generated bindings and thin Kotlin adapters.
- `app/src/test/` includes platform tests plus fixture parity tests against Rust.

Do not put product invariants in Kotlin unless they are platform-specific.

