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
