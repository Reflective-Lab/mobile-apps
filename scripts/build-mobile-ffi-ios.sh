#!/usr/bin/env bash
# Build the quorum-ffi Rust crate for iOS device + simulator,
# generate Swift bindings, and assemble an XCFramework consumed by
# apps/marquee/quorum-sense/ios/.
#
# Unverified by Claude — no Xcode + iOS targets on the dev machine where
# this was authored. Run on a Mac with Xcode 15+ to validate.

set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
cd "$ROOT"

CRATE=quorum-ffi
LIB_BASENAME=quorum_ffi
LIB_FILE="lib${LIB_BASENAME}.a"
FRAMEWORK=QuorumFFI
UDL=apps/marquee/quorum-sense/ffi/src/quorum_mobile.udl

# Match the app's iOS deployment target (project.yml: iOS 18.0). Rust defaults the
# aarch64-apple-ios target to iOS 10.0, which mismatches C deps compiled against
# the current SDK (e.g. aws-lc-sys via reqwest/rustls) and fails to link. Pinning
# it keeps every slice consistent with the app it links into.
export IPHONEOS_DEPLOYMENT_TARGET=18.0

DEVICE=aarch64-apple-ios
SIM_ARM=aarch64-apple-ios-sim
SIM_X86=x86_64-apple-ios

# Install required rustup targets if missing.
for t in "$DEVICE" "$SIM_ARM" "$SIM_X86"; do
    if ! rustup target list --installed | grep -q "^$t$"; then
        echo "==> installing rustup target $t"
        rustup target add "$t"
    fi
done

# Build all three slices in release mode.
for t in "$DEVICE" "$SIM_ARM" "$SIM_X86"; do
    echo "==> building $CRATE for $t"
    cargo build --release --target "$t" -p "$CRATE"
done

# Combine the two simulator slices into a universal static lib.
SIM_UNIVERSAL_DIR="target/sim-universal/release"
mkdir -p "$SIM_UNIVERSAL_DIR"
lipo -create \
    "target/$SIM_ARM/release/$LIB_FILE" \
    "target/$SIM_X86/release/$LIB_FILE" \
    -output "$SIM_UNIVERSAL_DIR/$LIB_FILE"

# Generate Swift bindings.
GEN_DIR=$(mktemp -d)
trap 'rm -rf "$GEN_DIR"' EXIT
echo "==> generating Swift bindings"
cargo run -p "$CRATE" --bin uniffi-bindgen --quiet -- \
    generate "$UDL" --language swift --out-dir "$GEN_DIR"

# Package headers + modulemap. Rename to module.modulemap so clang
# auto-discovers it inside the XCFramework's Headers/.
HEADERS_DIR=$(mktemp -d)
trap 'rm -rf "$GEN_DIR" "$HEADERS_DIR"' EXIT
cp "$GEN_DIR/${LIB_BASENAME}FFI.h" "$HEADERS_DIR/"
cp "$GEN_DIR/${LIB_BASENAME}FFI.modulemap" "$HEADERS_DIR/module.modulemap"

OUT_DIR="apps/marquee/quorum-sense/ios/CoreBridge"
mkdir -p "$OUT_DIR"
rm -rf "$OUT_DIR/$FRAMEWORK.xcframework"

echo "==> building XCFramework"
xcodebuild -create-xcframework \
    -library "target/$DEVICE/release/$LIB_FILE" -headers "$HEADERS_DIR" \
    -library "$SIM_UNIVERSAL_DIR/$LIB_FILE" -headers "$HEADERS_DIR" \
    -output "$OUT_DIR/$FRAMEWORK.xcframework"

cp "$GEN_DIR/${LIB_BASENAME}.swift" "$OUT_DIR/$FRAMEWORK.swift"

echo "✓ $OUT_DIR/$FRAMEWORK.xcframework"
echo "✓ $OUT_DIR/$FRAMEWORK.swift"
