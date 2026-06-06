# iOS Native Shell Template

A minimal, buildable SwiftUI app that proves the iOS build + simulator pipeline. **Not a product app** — it lives here so any of the marquee/studio mobile apps can copy this structure and inherit a working build.

## What's here

```text
ios/
  project.yml                  # XcodeGen spec — generates ReflectiveShell.xcodeproj
  App/
    ReflectiveShellApp.swift   # @main SwiftUI entry point
    ContentView.swift          # Hello screen
  .gitignore                   # Excludes generated .xcodeproj and build/
  README.md                    # This file
```

The `.xcodeproj` is **not committed** — XcodeGen regenerates it from `project.yml`.

## Prerequisites

- Xcode 15+ installed at `/Applications/Xcode.app` (Command Line Tools alone are not enough)
- `xcode-select -p` points to the full Xcode (`sudo xcode-select -s /Applications/Xcode.app/Contents/Developer`)
- XcodeGen: `brew install xcodegen`
- `just`: `brew install just`

## Build and run

From the repo root:

```sh
just ios-build              # build for iPhone 16 simulator
just ios-sim                # build + boot simulator + install + launch
just ios-sim "iPhone 15 Pro"  # override device
```

## Adoption path

To start a real iOS app (post-v1, post-trigger):

1. Copy this directory to your product app, e.g. `apps/marquee/<app>/ios/`.
2. Rename `ReflectiveShell` → your product name in `project.yml` and Swift sources.
3. Adopt the contract structure as the app grows:

   ```text
   App/         — SwiftUI navigation and lifecycle
   Capture/     — AVFoundation, PhotosUI, Vision, Speech
   PlatformAI/  — Foundation Models, Core ML
   CoreBridge/  — UniFFI-generated bindings + thin Swift adapters
   Resources/
   Tests/       — platform tests + fixture parity vs Rust
   ```

4. Wire UniFFI bindings from your shared Rust crate into `CoreBridge/`.

Do not put product invariants in Swift unless they are platform-specific.

## Verification status

This scaffold has **not been built or run** by Claude in this session — the dev machine has Command Line Tools only, no Xcode.app. Files are written blind based on:

- XcodeGen `project.yml` schema (current as of XcodeGen 2.x)
- Standard SwiftUI `@main` app structure
- `xcodebuild` + `xcrun simctl` invocation patterns

First run on a machine with Xcode installed should be treated as the actual verification step. Expected failure modes: XcodeGen schema drift, simulator device name mismatch (use `xcrun simctl list devices available` to confirm), Xcode license not accepted (`sudo xcodebuild -license accept`).
