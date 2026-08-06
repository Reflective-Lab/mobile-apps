# Which app in the fleet to build (see apps/registry.txt).
# Override per invocation: `just ios-build app=atlas`, `just android-sim app=vouch`.
app := "quorum"

default:
    @just --list

# Monorepo local dev: mobile-core path-deps use ../../../bedrock-platform/helms
# (sibling of mobile-apps). In standalone CI, checkout-helms-deps recreates the
# reflective/{mobile-apps,bedrock-platform/helms} layout instead.
path-deps:
    #!/usr/bin/env bash
    set -euo pipefail
    root="{{justfile_directory()}}"
    target="$(cd "$root/.." && pwd)/bedrock-platform/helms/crates/director-contracts/Cargo.toml"
    if [[ -f "$target" ]]; then
      echo "path-deps OK: $target"
    else
      echo "path-deps FAIL: expected helms at $target" >&2
      echo "  monorepo: clone Reflective-Lab/helms into ../bedrock-platform/helms" >&2
      exit 1
    fi

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all --check

lint:
    cargo clippy --workspace --all-targets -- -D warnings

# Run the workspace tests with cargo-nextest (faster, process-isolated runner).
# Install once: cargo install cargo-nextest --locked (or https://get.nexte.st).
test:
    cargo nextest run --workspace

# Fallback runner (also covers doctests, which nextest skips).
test-cargo:
    cargo test --workspace --locked

doc:
    cargo doc --workspace --no-deps --locked

scaffold-check:
    bash scripts/check-mobile-scaffold.sh

build-script-test:
    bash scripts/tests/build-mobile-ffi-ios-env-test.sh

# Fuzz the untrusted-input seam (QF-2026-06-24-06). Needs nightly + cargo-fuzz:
#   rustup toolchain install nightly && cargo install cargo-fuzz
# Targets: draft_field_signal | parse_enums | confidence_roundtrip.
# Reads the committed seed corpus (seeds/<target>) and writes discoveries to the
# scratch corpus/<target> (gitignored). Bounded by default so it doubles as a
# quick smoke; raise `secs=` to fuzz longer.
fuzz-core target="draft_field_signal" secs="60":
    cd crates/mobile-core/fuzz && mkdir -p corpus/{{target}} && cargo +nightly fuzz run {{target}} corpus/{{target}} seeds/{{target}} -- -max_total_time={{secs}}

check: fmt test scaffold-check

# Rust + iOS smoke shell + Android smoke shell. Slow. Requires Xcode, Android SDK+NDK, cargo-ndk.
check-mobile: check ios-build android-build

ci: fmt-check lint test doc scaffold-check build-script-test

# --- iOS native shell template ---

ios-uniffi:
    bash scripts/build-shell-ffi-ios.sh

_ios-gen:
    #!/usr/bin/env bash
    set -euo pipefail
    APP={{app}} source scripts/app-config.sh
    cd templates/native-shells/ios && xcodegen generate

ios-build device="iPhone 16": ios-uniffi _ios-gen
    cd templates/native-shells/ios && xcodebuild \
      -scheme App \
      -destination 'platform=iOS Simulator,name={{device}}' \
      -derivedDataPath build \
      build

ios-sim device="iPhone 16": (ios-build device)
    #!/usr/bin/env bash
    set -euo pipefail
    APP={{app}} source scripts/app-config.sh
    cd templates/native-shells/ios
    APP_PATH="build/Build/Products/Debug-iphonesimulator/App.app"
    xcrun simctl boot "{{device}}" 2>/dev/null || true
    open -a Simulator
    xcrun simctl install booted "$APP_PATH"
    xcrun simctl launch booted "$APP_BUNDLE_ID"

# --- Quorum iOS product app (apps/marquee/quorum-sense/ios) ---

# Build the quorum-ffi Rust crate into CoreBridge/QuorumFFI.xcframework +
# QuorumFFI.swift (both gitignored). Must run before generating the project.
quorum-ios-uniffi:
    bash scripts/build-mobile-ffi-ios.sh

# Generate the (uncommitted) QuorumMobile.xcodeproj from project.yml. Depends on
# the xcframework existing, so it chains through quorum-ios-uniffi.
quorum-ios-gen: quorum-ios-uniffi
    cd apps/marquee/quorum-sense/ios && xcodegen generate

# Build the product app for the simulator (runs against the real Rust core via
# QuorumCoreBridgeFFI).
quorum-ios-build device="iPhone 16": quorum-ios-gen
    cd apps/marquee/quorum-sense/ios && xcodebuild \
      -scheme QuorumMobile \
      -destination 'platform=iOS Simulator,name={{device}}' \
      -derivedDataPath build \
      build

# Archive for distribution. Signing is configured in project.yml
# (DEVELOPMENT_TEAM + automatic); -allowProvisioningUpdates lets Xcode create or
# download the distribution profile. Requires the App ID se.reflective.quorum and
# a Distribution certificate in the login keychain / Apple account.
quorum-ios-archive: quorum-ios-gen
    cd apps/marquee/quorum-sense/ios && xcodebuild \
      -scheme QuorumMobile \
      -destination 'generic/platform=iOS' \
      -archivePath build/QuorumMobile.xcarchive \
      -allowProvisioningUpdates \
      archive

# --- Quorum Android product app (apps/marquee/quorum-sense/android) ---

# Build the quorum-ffi Rust crate into the app's jniLibs (per ABI) and generate
# Kotlin bindings under uniffi/quorum_ffi/ (both gitignored). Must run before the
# Gradle build. Requires ANDROID_NDK_HOME + cargo-ndk.
quorum-android-uniffi:
    bash scripts/build-mobile-ffi-android.sh

# Build the product app (runs against the real Rust core via QuorumCoreBridgeFFI).
quorum-android-build: quorum-android-uniffi
    cd apps/marquee/quorum-sense/android && ./gradlew :app:assembleDebug

# Regenerate the sha256 dependency-verification metadata (QF-2026-06-24-04).
# Resolves every classpath CI touches so the committed file stays complete; needs
# the UniFFI jniLibs present first (Gradle configures the FFI task). Re-run after any
# Gradle dependency or plugin bump, then review the diff before committing.
quorum-android-verify-metadata: quorum-android-uniffi
    cd apps/marquee/quorum-sense/android && ./gradlew --write-verification-metadata sha256 \
        assembleDebug assembleRelease assembleDebugAndroidTest testDebugUnitTest

# --- Android native shell template ---

android-uniffi:
    bash scripts/build-shell-ffi-android.sh

android-build: android-uniffi
    #!/usr/bin/env bash
    set -euo pipefail
    APP={{app}} source scripts/app-config.sh
    cd templates/native-shells/android
    ./gradlew :app:assembleDebug -PappSlug="$APP_SLUG" -PappName="$APP_NAME"

android-sim avd="Pixel_8_API_35": android-build
    #!/usr/bin/env bash
    set -euo pipefail
    APP={{app}} source scripts/app-config.sh
    cd templates/native-shells/android
    APK="app/build/outputs/apk/debug/app-debug.apk"
    APP_ID="$APP_ANDROID_APPLICATION_ID"
    if ! adb devices | awk 'NR>1 && $2=="device"{found=1} END{exit !found}'; then
        echo "booting emulator: {{avd}}"
        emulator -avd "{{avd}}" -no-snapshot-load >/dev/null 2>&1 &
        adb wait-for-device
        until [ "$(adb shell getprop sys.boot_completed 2>/dev/null | tr -d '\r')" = "1" ]; do
            sleep 2
        done
    fi
    adb install -r "$APK"
    # Activity class lives in the constant namespace (se.reflective.shell);
    # it is installed under the per-app applicationId.
    adb shell am start -n "${APP_ID}/se.reflective.shell.MainActivity"
