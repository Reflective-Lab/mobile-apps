# Android Native Shell Template

A minimal, buildable Compose app that proves the Android build + emulator pipeline. **Not a product app** — it lives here so any of the marquee/studio mobile apps can copy this structure and inherit a working build.

## What's here

```text
android/
  settings.gradle.kts
  build.gradle.kts                   # top-level plugin versions
  gradle.properties
  gradle/wrapper/                    # Gradle 8.10.2 wrapper
  gradlew, gradlew.bat
  app/
    build.gradle.kts                 # AGP 8.7.3, Kotlin 2.0.21, Compose BOM 2025.01.00
    src/main/
      AndroidManifest.xml
      java/dev/reflective/shell/MainActivity.kt
      res/values/{strings,themes}.xml
  .gitignore
  README.md
```

`app/build/` and `.gradle/` are not committed — Gradle regenerates them.

## Prerequisites

- JDK 17 (`brew install --cask temurin@17` or any JDK 17 distribution)
- Android SDK with: platform-35, build-tools 35.x, platform-tools, emulator, at least one system image (e.g. `system-images;android-35;google_apis;arm64-v8a` on Apple Silicon)
- `$ANDROID_HOME` set (typically `~/Library/Android/sdk`); `$ANDROID_HOME/platform-tools` and `$ANDROID_HOME/emulator` on `PATH`
- At least one AVD created, e.g. `Pixel_8_API_35`. Create via `avdmanager` or Android Studio.
- `just`: `brew install just`

## Build and run

From the repo root:

```sh
just android-build               # assemble debug APK
just android-sim                 # build + boot AVD + install + launch
just android-sim "Pixel_8_API_35"  # override AVD name
```

## Adoption path

To start a real Android app (post-v1, post-trigger):

1. Copy this directory to your product app, e.g. `apps/marquee/<app>/android/`.
2. Rename `dev.reflective.shell` → your product's `applicationId`/`namespace`/package across `build.gradle.kts`, `MainActivity.kt`, and the source folder path.
3. Update `rootProject.name` in `settings.gradle.kts`.
4. Adopt the contract structure as the app grows (mirrors the iOS contract):

   ```text
   app/src/main/java/<pkg>/
     ui/         — Compose navigation and screens
     capture/    — CameraX, MediaRecorder, Speech
     platformai/ — Gemini Nano, ML Kit GenAI, LiteRT adapters
     corebridge/ — UniFFI-generated bindings + thin Kotlin adapters
   ```

5. Wire UniFFI bindings from your shared Rust crate into `corebridge/`.

Do not put product invariants in Kotlin unless they are platform-specific.

## Verification status

This scaffold has **not been built or run** by Claude in this session — the dev machine has no JDK, no Android SDK on PATH, and no emulator binary. Files are written based on:

- AGP 8.7.3 + Kotlin 2.0.21 + Compose BOM 2025.01.00 (matches the existing `apps/marquee/quorum-sense/android/` scaffold)
- Standard `ComponentActivity` + `setContent` Compose entry point
- Gradle 8.10.2 wrapper (required floor for AGP 8.7.x)

First run on a machine with the Android toolchain should be treated as the actual verification step. Expected failure modes: wrong JDK on PATH (must be 17), missing platform-35, AVD name mismatch (`emulator -list-avds` to confirm), emulator binary not on PATH.

The Gradle wrapper jar was copied from `archive/legacy-placeholders/converge-android/` — its bytes were not authored by Claude.
