#!/usr/bin/env bash
# Cross-compile the reflective-mobile-ffi Rust crate for Android ABIs and
# generate Kotlin bindings consumed by apps/marquee/quorum-sense/android/.
#
# Unverified by Claude — no JDK, Android SDK, NDK, or cargo-ndk on the dev
# machine where this was authored. Run on a Mac/Linux with the Android
# toolchain installed to validate.

set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
cd "$ROOT"

CRATE=reflective-mobile-ffi
UDL=crates/mobile-ffi/src/quorum_mobile.udl
ANDROID_APP=apps/marquee/quorum-sense/android/app

JNI_LIBS_DIR="$ANDROID_APP/src/main/jniLibs"
KOTLIN_OUT="$ANDROID_APP/src/main/java"

# Required tooling. cargo-ndk drives the cross-compile; it needs
# $ANDROID_NDK_HOME (or $ANDROID_NDK_ROOT) set.
if ! command -v cargo-ndk >/dev/null 2>&1; then
    echo "error: cargo-ndk not installed. Run: cargo install cargo-ndk" >&2
    exit 1
fi
if [ -z "${ANDROID_NDK_HOME:-${ANDROID_NDK_ROOT:-}}" ]; then
    echo "error: ANDROID_NDK_HOME (or ANDROID_NDK_ROOT) not set" >&2
    exit 1
fi

# Install required Rust targets if missing.
for t in aarch64-linux-android x86_64-linux-android; do
    if ! rustup target list --installed | grep -q "^$t$"; then
        echo "==> installing rustup target $t"
        rustup target add "$t"
    fi
done

# Build per ABI and place .so files under jniLibs/<abi>/.
echo "==> building $CRATE for arm64-v8a + x86_64"
rm -rf "$JNI_LIBS_DIR"
cargo ndk -t arm64-v8a -t x86_64 \
    -o "$JNI_LIBS_DIR" \
    build --release -p "$CRATE"

# Generate Kotlin bindings into app/src/main/java/uniffi/...
echo "==> generating Kotlin bindings"
mkdir -p "$KOTLIN_OUT"
cargo run -p "$CRATE" --bin uniffi-bindgen --quiet -- \
    generate "$UDL" --language kotlin --out-dir "$KOTLIN_OUT"

echo "✓ jniLibs: $JNI_LIBS_DIR"
echo "✓ Kotlin bindings: $KOTLIN_OUT/uniffi/reflective_mobile_ffi/"
