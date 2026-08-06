#!/usr/bin/env bash

set -euo pipefail

: "${TEST_COMMAND_LOG:?TEST_COMMAND_LOG must be set}"
: "${TEST_TEMP_ROOT:?TEST_TEMP_ROOT must be set}"

tool=$(basename "$0")

case "$tool" in
  rustup)
    if [[ "${1:-} ${2:-} ${3:-}" != "target list --installed" ]]; then
      echo "unexpected rustup invocation: $*" >&2
      exit 80
    fi
    printf '%s\n' aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios
    ;;

  cargo)
    command=${1:-}
    shift || true

    case "$command" in
      build)
        target=""
        while (( $# > 0 )); do
          if [[ "$1" == "--target" ]]; then
            target=${2:-}
            shift 2
          else
            shift
          fi
        done

        if [[ -z "$target" ]]; then
          echo "cargo build did not specify --target" >&2
          exit 81
        fi
        if [[ "${IPHONEOS_DEPLOYMENT_TARGET:-}" != "18.0" ]]; then
          echo "target build $target lost IPHONEOS_DEPLOYMENT_TARGET=18.0" >&2
          exit 82
        fi

        mkdir -p "target/$target/release"
        : > "target/$target/release/libquorum_ffi.a"
        printf 'build:%s:%s\n' "$target" "$IPHONEOS_DEPLOYMENT_TARGET" >> "$TEST_COMMAND_LOG"
        ;;

      run)
        if [[ -n "${IPHONEOS_DEPLOYMENT_TARGET:-}" ]]; then
          echo "host cargo run inherited IPHONEOS_DEPLOYMENT_TARGET=$IPHONEOS_DEPLOYMENT_TARGET" >&2
          exit 86
        fi

        out_dir=""
        while (( $# > 0 )); do
          if [[ "$1" == "--out-dir" ]]; then
            out_dir=${2:-}
            shift 2
          else
            shift
          fi
        done

        if [[ -z "$out_dir" ]]; then
          echo "cargo run did not specify --out-dir" >&2
          exit 83
        fi

        mkdir -p "$out_dir"
        printf '%s\n' '#pragma once' > "$out_dir/quorum_ffiFFI.h"
        printf '%s\n' 'module quorum_ffi {}' > "$out_dir/quorum_ffiFFI.modulemap"
        printf '%s\n' 'public enum Generated {}' > "$out_dir/quorum_ffi.swift"
        printf '%s\n' 'bindgen:unset' >> "$TEST_COMMAND_LOG"
        ;;

      *)
        echo "unexpected cargo invocation: $command $*" >&2
        exit 84
        ;;
    esac
    ;;

  lipo)
    output=""
    while (( $# > 0 )); do
      if [[ "$1" == "-output" ]]; then
        output=${2:-}
        shift 2
      else
        shift
      fi
    done
    [[ -n "$output" ]] || { echo "lipo did not specify -output" >&2; exit 87; }
    mkdir -p "$(dirname "$output")"
    : > "$output"
    ;;

  xcodebuild)
    output=""
    while (( $# > 0 )); do
      if [[ "$1" == "-output" ]]; then
        output=${2:-}
        shift 2
      else
        shift
      fi
    done
    [[ -n "$output" ]] || { echo "xcodebuild did not specify -output" >&2; exit 88; }
    mkdir -p "$output"
    ;;

  sed)
    # The test exercises environment propagation, not generated Swift rewriting.
    ;;

  mktemp)
    generated="$TEST_TEMP_ROOT/generated-$RANDOM-$$"
    mkdir -p "$generated"
    printf '%s\n' "$generated"
    ;;

  *)
    echo "unexpected fake tool: $tool" >&2
    exit 89
    ;;
esac
