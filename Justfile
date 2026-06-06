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

ci: fmt-check lint test doc scaffold-check

# --- iOS native shell template ---

_ios-gen:
    cd templates/native-shells/ios && xcodegen generate

ios-build device="iPhone 16": _ios-gen
    cd templates/native-shells/ios && xcodebuild \
      -scheme ReflectiveShell \
      -destination 'platform=iOS Simulator,name={{device}}' \
      -derivedDataPath build \
      build

ios-sim device="iPhone 16": (ios-build device)
    #!/usr/bin/env bash
    set -euo pipefail
    cd templates/native-shells/ios
    APP_PATH="build/Build/Products/Debug-iphonesimulator/ReflectiveShell.app"
    BUNDLE_ID="dev.reflective.shell"
    xcrun simctl boot "{{device}}" 2>/dev/null || true
    open -a Simulator
    xcrun simctl install booted "$APP_PATH"
    xcrun simctl launch booted "$BUNDLE_ID"

# --- Android native shell template ---

android-build:
    cd templates/native-shells/android && ./gradlew :app:assembleDebug

android-sim avd="Pixel_8_API_35": android-build
    #!/usr/bin/env bash
    set -euo pipefail
    cd templates/native-shells/android
    APK="app/build/outputs/apk/debug/app-debug.apk"
    APP_ID="dev.reflective.shell"
    if ! adb devices | awk 'NR>1 && $2=="device"{found=1} END{exit !found}'; then
        echo "booting emulator: {{avd}}"
        emulator -avd "{{avd}}" -no-snapshot-load >/dev/null 2>&1 &
        adb wait-for-device
        until [ "$(adb shell getprop sys.boot_completed 2>/dev/null | tr -d '\r')" = "1" ]; do
            sleep 2
        done
    fi
    adb install -r "$APK"
    adb shell am start -n "${APP_ID}/${APP_ID}.MainActivity"
