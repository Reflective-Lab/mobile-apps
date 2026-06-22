# Which app in the fleet to build (see apps/registry.txt).
# Override per invocation: `just ios-build app=atlas`, `just android-sim app=vouch`.
app := "quorum"

default:
    @just --list

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all --check

lint:
    cargo clippy --workspace --all-targets -- -D warnings

test:
    cargo test --workspace --locked

doc:
    cargo doc --workspace --no-deps --locked

scaffold-check:
    bash scripts/check-mobile-scaffold.sh

check: fmt test scaffold-check

# Rust + iOS smoke shell + Android smoke shell. Slow. Requires Xcode, Android SDK+NDK, cargo-ndk.
check-mobile: check ios-build android-build

ci: fmt-check lint test doc scaffold-check

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
