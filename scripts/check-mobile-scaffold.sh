#!/usr/bin/env bash
set -euo pipefail

required_files=(
  "apps/marquee/quorum-sense/fixtures/field-signal-capture.v1.json"
  "schemas/quorum-mobile.udl"
  "schemas/quorum-mobile-workflow-v1.md"
  "apps/marquee/quorum-sense/ios/QuorumMobileIOS/Package.swift"
  "apps/marquee/quorum-sense/ios/QuorumMobileIOS/Sources/QuorumMobileIOS/Views/SignalCaptureView.swift"
  "apps/marquee/quorum-sense/ios/QuorumMobileIOS/Sources/QuorumMobileIOS/CoreBridge/QuorumCoreBridge.swift"
  "apps/marquee/quorum-sense/android/settings.gradle.kts"
  "apps/marquee/quorum-sense/android/app/build.gradle.kts"
  "apps/marquee/quorum-sense/android/app/src/main/java/se/reflective/quorum/mobile/ui/QuorumMobileApp.kt"
  "apps/marquee/quorum-sense/android/app/src/main/java/se/reflective/quorum/mobile/corebridge/QuorumCoreBridge.kt"
)

for file in "${required_files[@]}"; do
  test -f "$file" || {
    echo "missing required scaffold file: $file" >&2
    exit 1
  }
done

grep -q '"id": "quorum.field_signal_capture.v1"' \
  apps/marquee/quorum-sense/fixtures/field-signal-capture.v1.json
grep -q "quorum_draft_field_signal" schemas/quorum-mobile.udl
grep -q "SignalCaptureView" \
  apps/marquee/quorum-sense/ios/QuorumMobileIOS/Sources/QuorumMobileIOS/Views/SignalCaptureView.swift
grep -q "SignalCaptureScreen" \
  apps/marquee/quorum-sense/android/app/src/main/java/se/reflective/quorum/mobile/ui/QuorumMobileApp.kt

echo "mobile scaffold check passed"

