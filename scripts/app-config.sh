#!/usr/bin/env bash
# Single source of truth for per-app identity across the iOS + Android shells.
#
# SOURCE this file (don't execute it); select the app with the APP env var,
# or pass the slug as $1. Defaults to "quorum".
#
#   APP=atlas source scripts/app-config.sh
#   source scripts/app-config.sh vouch
#
# Exports:
#   APP_SLUG                     e.g. quorum
#   APP_NAME                     e.g. Quorum   (user-visible display name)
#   APP_BUNDLE_ID                e.g. se.reflective.quorum   (iOS)
#   APP_ANDROID_APPLICATION_ID   e.g. se.reflective.quorum   (Android install identity)
#   APP_ANDROID_NAMESPACE        se.reflective.shell         (shared code package, constant)
#   APP_BACKEND_URL              e.g. https://...run.app      (per-app backend, deploy-set)
#
# Identity (bundle id / applicationId) varies per app; the shared shell CODE
# lives in one constant namespace so all apps reuse the same Swift/Kotlin/Rust.
#
# Every app also has a corresponding backend server (apps/backends.txt). The
# exact Cloud Run URL is assigned at deploy time, so APP_BACKEND_URL is taken
# from the environment when set (deploy injects it) and otherwise falls back to
# the recorded default. See docs/architecture/mobile-backend-pairing.md.

_cfg_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
_root="$(dirname "$_cfg_dir")"
_registry="$_root/apps/registry.txt"
_backends="$_root/apps/backends.txt"

APP_SLUG="${APP:-${1:-quorum}}"

if ! printf '%s' "$APP_SLUG" | grep -qE '^[a-z][a-z0-9]*$'; then
    echo "app-config: invalid app slug '$APP_SLUG' (lowercase letters/digits, must start with a letter)" >&2
    return 1 2>/dev/null || exit 1
fi

# Resolve display name from the registry; fall back to a capitalized slug.
APP_NAME=""
if [ -f "$_registry" ]; then
    while IFS= read -r _line || [ -n "$_line" ]; do
        case "$_line" in ''|\#*) continue ;; esac
        if [ "${_line%%:*}" = "$APP_SLUG" ]; then
            APP_NAME="${_line#*:}"
            break
        fi
    done < "$_registry"
fi
if [ -z "$APP_NAME" ]; then
    APP_NAME="$(printf '%s' "$APP_SLUG" | awk '{print toupper(substr($0,1,1)) substr($0,2)}')"
    echo "app-config: '$APP_SLUG' not in registry; using display name '$APP_NAME'" >&2
fi

export APP_SLUG APP_NAME
export APP_BUNDLE_ID="se.reflective.${APP_SLUG}"
export APP_ANDROID_APPLICATION_ID="se.reflective.${APP_SLUG}"
export APP_ANDROID_NAMESPACE="se.reflective.shell"

# Backend service URL. A value already in the environment wins (deploy injects
# the exact Cloud Run URL); otherwise fall back to the recorded default.
_backend_default=""
if [ -f "$_backends" ]; then
    while IFS= read -r _line || [ -n "$_line" ]; do
        case "$_line" in ''|\#*) continue ;; esac
        if [ "${_line%%:*}" = "$APP_SLUG" ]; then
            _backend_default="${_line#*:}"
            break
        fi
    done < "$_backends"
fi
export APP_BACKEND_URL="${APP_BACKEND_URL:-$_backend_default}"
if [ -z "$APP_BACKEND_URL" ]; then
    echo "app-config: no backend URL for '$APP_SLUG' (set APP_BACKEND_URL or apps/backends.txt)" >&2
fi
