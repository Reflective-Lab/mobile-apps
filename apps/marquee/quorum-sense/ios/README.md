# Quorum iOS

The Quorum Mobile SwiftUI app. This is an **archivable iOS app target** generated
with XcodeGen — the `.xcodeproj` is not committed.

## Structure

```text
ios/
  project.yml          # XcodeGen spec → generates QuorumMobile.xcodeproj
  App/                 # @main entry (QuorumMobileApp) + root navigation
  Views/               # SwiftUI screens (SignalCaptureView)
  Capture/             # Field signal models (FieldSignalDraft, modalities)
  PlatformAI/          # Native extraction (FoundationModels, Speech, Vision)
  CoreBridge/          # QuorumCoreBridge protocol + UniFFI bindings (generated)
  .gitignore
  README.md
```

Identity:

| | Value |
|---|---|
| Bundle ID | `se.reflective.quorum` (matches the registered App ID) |
| Display name | Quorum |
| Deployment target | iOS 18.0 |
| Scheme / target | `QuorumMobile` |

> **Restructured from a SwiftPM placeholder.** This directory used to hold a
> nested `QuorumMobileIOS/` Swift package (a library, not a runnable app). The
> sources were flattened into the contract folders above and an `@main` app
> entry was added so the app can be built, archived, and submitted to the App
> Store. `scripts/check-mobile-scaffold.sh` tracks the new paths.

## Prerequisites

- Xcode 16+ (`xcode-select -p` must point to `/Applications/Xcode.app/...`)
- `brew install xcodegen just`

## Build and run

From the repo root:

```sh
just quorum-ios-gen        # generate QuorumMobile.xcodeproj
just quorum-ios-build      # build for iPhone 16 simulator
just quorum-ios-build "iPhone 15 Pro"   # override device
just quorum-ios-archive    # archive for App Store distribution (needs signing)
```

Or open it in Xcode after generating:

```sh
cd apps/marquee/quorum-sense/ios && xcodegen generate && open QuorumMobile.xcodeproj
```

## CoreBridge / UniFFI

The running app talks to the **real Rust core** through UniFFI:

- `QuorumCoreBridgeFFI` (`CoreBridge/QuorumCoreBridgeFFI.swift`) implements the
  `QuorumCoreBridge` protocol by calling the generated bindings and is injected
  by `RootView`.
- `PreviewQuorumCoreBridge` remains the default argument of `SignalCaptureView`
  so SwiftUI previews and tests stay dependency-free.

The bindings are generated, not committed:

1. `bash scripts/build-mobile-ffi-ios.sh` (or `just quorum-ios-uniffi`) emits
   `CoreBridge/QuorumFFI.xcframework` + `CoreBridge/QuorumFFI.swift` (gitignored).
2. `project.yml` links the static-library xcframework (`embed: false`).
3. `just quorum-ios-build` chains the two steps, so a fresh checkout is one command.

Because the bindings are gitignored, `QuorumCoreBridgeFFI.swift` does not compile
until step 1 has run — that is expected.

## Publishing

See the signing/distribution steps: Apple Distribution cert in the login
keychain, App ID `se.reflective.quorum` registered, app record created in App
Store Connect, then `just quorum-ios-archive` → Xcode Organizer → Distribute,
or upload with the App Store Connect API key (`AuthKey_*.p8`).
