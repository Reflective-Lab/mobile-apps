#!/usr/bin/env bash
set -euo pipefail

required_files=(
  "apps/marquee/quorum-sense/fixtures/field-signal-capture.v1.json"
  "schemas/quorum-mobile.udl"
  "apps/marquee/quorum-sense/ffi/src/quorum_mobile.udl"
  "schemas/quorum-mobile-workflow-v1.md"
  "apps/marquee/quorum-sense/ios/project.yml"
  "apps/marquee/quorum-sense/ios/App/QuorumMobileApp.swift"
  "apps/marquee/quorum-sense/ios/Views/SignalCaptureView.swift"
  "apps/marquee/quorum-sense/ios/CoreBridge/QuorumCoreBridge.swift"
  "apps/marquee/quorum-sense/android/settings.gradle.kts"
  "apps/marquee/quorum-sense/android/app/build.gradle.kts"
  "apps/marquee/quorum-sense/android/app/src/main/java/se/reflective/quorum/ui/QuorumMobileApp.kt"
  "apps/marquee/quorum-sense/android/app/src/main/java/se/reflective/quorum/corebridge/QuorumCoreBridge.kt"
)

for file in "${required_files[@]}"; do
  test -f "$file" || {
    echo "missing required scaffold file: $file" >&2
    exit 1
  }
done

# schemas/quorum-mobile.udl is the canonical contract doc; the compiled copy
# read by apps/marquee/quorum-sense/ffi/build.rs must stay byte-identical.
cmp -s schemas/quorum-mobile.udl apps/marquee/quorum-sense/ffi/src/quorum_mobile.udl || {
  echo "schemas/quorum-mobile.udl and apps/marquee/quorum-sense/ffi/src/quorum_mobile.udl have drifted apart" >&2
  exit 1
}

grep -q '"id": "quorum.field_signal_capture.v1"' \
  apps/marquee/quorum-sense/fixtures/field-signal-capture.v1.json
grep -q "quorum_draft_field_signal" schemas/quorum-mobile.udl
grep -q "SignalCaptureView" \
  apps/marquee/quorum-sense/ios/Views/SignalCaptureView.swift
grep -q "@main" \
  apps/marquee/quorum-sense/ios/App/QuorumMobileApp.swift
grep -q "SignalCaptureScreen" \
  apps/marquee/quorum-sense/android/app/src/main/java/se/reflective/quorum/ui/QuorumMobileApp.kt

# M1.6 — authority-leakage guard (MILESTONES.md). The marquee mobile client
# captures, drafts, asks consent, queues, and renders receipts; it must never
# carry server *authority*: round-close, rulebook evaluation, Lamport/Merkle
# chain mutation, Stripe billing, or entitlement decisions (ADR 0002, EPIC 4).
# Introducing any of those into mobile source requires updating the boundary
# docs + an ADR — and relaxing this guard in the same change, not silently.
authority_dirs=(
  crates/mobile-core/src
  crates/mobile-ai/src
  apps/marquee/quorum-sense/ffi/src
  apps/marquee/quorum-sense/ios/App
  apps/marquee/quorum-sense/ios/Views
  apps/marquee/quorum-sense/ios/CoreBridge
  apps/marquee/quorum-sense/android/app/src/main
)
authority_pattern='round[-_ ]?close|rulebook|lamport|merkle|stripe|entitlement'
# QuorumFFI.swift is generated; never hand-edited, so it is not a leak surface.
if leak=$(grep -rinE "$authority_pattern" "${authority_dirs[@]}" \
      --include='*.rs' --include='*.swift' --include='*.kt' 2>/dev/null \
      | grep -v 'CoreBridge/QuorumFFI.swift'); then
  echo "forbidden server-authority code in mobile source (ADR 0002 / EPIC 4):" >&2
  echo "$leak" >&2
  echo "If intentional, update the boundary docs + an ADR, then relax this guard." >&2
  exit 1
fi

echo "mobile scaffold check passed"

