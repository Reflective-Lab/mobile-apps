#!/usr/bin/env bash

set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
tmp_parent="$repo_root/target/script-tests"
mkdir -p "$tmp_parent"
test_root=$(mktemp -d "$tmp_parent/build-mobile-ffi-ios-env.XXXXXX")
trap 'rm -rf "$test_root"' EXIT
export TMPDIR="$test_root/tmp"
mkdir -p "$TMPDIR"

fixture_repo="$test_root/repo"
fake_bin="$test_root/bin"
mkdir -p "$fixture_repo/scripts" "$fake_bin"
cp "$repo_root/scripts/build-mobile-ffi-ios.sh" "$fixture_repo/scripts/"

fake_tool="$repo_root/scripts/tests/fixtures/fake-ios-build-tool.sh"
for tool in rustup cargo lipo xcodebuild sed mktemp; do
  ln -s "$fake_tool" "$fake_bin/$tool"
done

export TEST_COMMAND_LOG="$test_root/commands.log"
export TEST_TEMP_ROOT="$test_root/generated"
mkdir -p "$TEST_TEMP_ROOT"
: > "$TEST_COMMAND_LOG"

PATH="$fake_bin:$PATH" bash "$fixture_repo/scripts/build-mobile-ffi-ios.sh"

target_builds=$(grep -c '^build:.*:18\.0$' "$TEST_COMMAND_LOG" || true)
host_bindgen_runs=$(grep -c '^bindgen:unset$' "$TEST_COMMAND_LOG" || true)

if [[ "$target_builds" -ne 3 ]]; then
  echo "expected 3 iOS target builds with deployment target 18.0, got $target_builds" >&2
  exit 90
fi
if [[ "$host_bindgen_runs" -ne 1 ]]; then
  echo "expected 1 host bindgen run with deployment target unset, got $host_bindgen_runs" >&2
  exit 91
fi

echo "build-mobile-ffi-ios environment boundary: ok"
