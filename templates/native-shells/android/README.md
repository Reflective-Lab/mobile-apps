# Android Native Shell Template

A minimal, buildable Compose app that proves the Android build + emulator + Rust-via-UniFFI pipeline. **Not a product app** — it lives here so any of the marquee/studio mobile apps can copy this structure and inherit a working build.

## What's here

```text
android/
  settings.gradle.kts
  build.gradle.kts                   # top-level plugin versions
  gradle.properties
  gradle/wrapper/                    # Gradle 8.10.2 wrapper
  gradlew, gradlew.bat
  app/
    build.gradle.kts                 # AGP 8.7.3, Kotlin 2.0.21, Compose BOM 2025.01.00, JNA 5.14.0
    src/main/
      AndroidManifest.xml
      java/dev/reflective/shell/MainActivity.kt  # calls greet() from Rust
      java/uniffi/                              # generated Kotlin bindings (not committed)
      jniLibs/<abi>/libreflective_shell_ffi.so  # cross-compiled Rust (not committed)
      res/values/{strings,themes}.xml
  .gitignore
  README.md
```

`app/build/`, `.gradle/`, `app/src/main/jniLibs/`, and `app/src/main/java/uniffi/` are not committed — Gradle and `just android-uniffi` regenerate them.

The Rust side lives at `crates/shell-ffi/` (product-neutral; just `greet()` and `coreVersion()` to prove the round-trip).

## Prerequisites

- JDK 17 (`brew install --cask temurin@17` or any JDK 17 distribution)
- Android SDK with: platform-35, build-tools 35.x, platform-tools, emulator, at least one system image (e.g. `system-images;android-35;google_apis;arm64-v8a` on Apple Silicon)
- Android NDK (install via `sdkmanager "ndk;27.0.12077973"` or any 26+); set `$ANDROID_NDK_HOME` (or `$ANDROID_NDK_ROOT`) to its path
- `$ANDROID_HOME` set (typically `~/Library/Android/sdk`); `$ANDROID_HOME/platform-tools` and `$ANDROID_HOME/emulator` on `PATH`
- At least one AVD created, e.g. `Pixel_8_API_35`. Create via `avdmanager` or Android Studio.
- `cargo-ndk`: `cargo install cargo-ndk` (cross-compiles Rust → `.so` per ABI)
- Rust toolchain (already required by the workspace). Android rustup targets (`aarch64-linux-android`, `x86_64-linux-android`) are auto-installed by `scripts/build-shell-ffi-android.sh` the first time.
- `just`: `brew install just`

## Build and run

From the repo root:

```sh
just android-uniffi              # cross-compile Rust → jniLibs + Kotlin bindings
just android-build               # android-uniffi + assemble debug APK
just android-sim                 # android-build + boot AVD + install + launch
just android-sim "Pixel_8_API_35"  # override AVD name
```

`android-build` and `android-sim` chain through `android-uniffi` automatically.

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

**Verified by Claude on host:**
- `crates/shell-ffi` compiles, `cargo test -p reflective-shell-ffi` passes
- `uniffi-bindgen` generates valid Kotlin bindings from the host dylib (Kotlin API: `uniffi.reflective_shell_ffi.greet(name)`, `coreVersion()`)

**Not verified by Claude** — the dev machine has no JDK, no Android SDK on PATH, no NDK, no cargo-ndk, no emulator binary:
- Android cross-compile via cargo-ndk to `.so` per ABI
- AGP 8.7.3 + Kotlin 2.0.21 + Compose BOM 2025.01.00 build (matches the existing `apps/marquee/quorum-sense/android/` scaffold so should be known-good)
- JNA loading `libreflective_shell_ffi.so` from `jniLibs/` at runtime
- Gradle 8.10.2 wrapper run (required floor for AGP 8.7.x)
- `adb install` + `am start` flow

First run on a machine with the full Android toolchain should be treated as the actual verification step. Expected failure modes: wrong JDK on PATH (must be 17), missing platform-35, missing NDK or `$ANDROID_NDK_HOME` unset, `cargo install cargo-ndk` not run, AVD name mismatch (`emulator -list-avds` to confirm), emulator binary not on PATH, JNA failing to find the `.so` (check ABI in `jniLibs/` matches AVD architecture — Apple Silicon emulators are arm64-v8a).

The Gradle wrapper jar was copied from `archive/legacy-placeholders/converge-android/` — its bytes were not authored by Claude.
