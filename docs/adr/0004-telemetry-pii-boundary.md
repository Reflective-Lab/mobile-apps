# ADR 0004: Telemetry-PII Boundary — Crash Reporting Must Not Exfiltrate Capture

- Date: 2026-06-24
- Status: Accepted
- Related: ADR 0002 (device proposes; server promotes), `QF-2026-06-24-05`,
  `schemas/quorum-mobile.udl` (the FFI contract carrying `raw_capture`)

## Context

We ship crash/error telemetry to Sentry from **three** independent clients:

1. **Rust core** — `init_observability` in `apps/marquee/quorum-sense/ffi/src/lib.rs`
   (panic + error integration), reporting to the separate Rust Sentry project.
2. **iOS app** — `SentrySDK.start` in `App/QuorumMobileApp.swift`.
3. **Android app** — manifest auto-init via `io.sentry.*` meta-data.

The Quorum workflow's core payload is **field-signal capture**: `raw_capture` is a
voice transcript, OCR text, or free-form note (`SignalModality` in the UDL), plus
derived `summary` / `latent_need` / `contradiction`. This is sensitive user content.

A crash pipe is an outbound network channel we built on purpose. Left at defaults,
it can attach device hostname, user/IP context, and — if capture text ever reaches a
panic message, breadcrumb, or error string — the capture itself. That would route
sensitive data off-device through telemetry, contradicting the ADR 0002 posture that
the device owns capture.

## Decision

**Telemetry is a trust boundary. No client may emit personal/device identifiers, and
capture content must never ride a crash event off the device.**

Concretely, on every client:

- `send_default_pii = false` — set **explicitly**, not relied on as a default.
- A `before_send` hook strips `server_name` (device hostname) and `user`
  (id/username/ip/geo) before the event ships.
- Panic message + backtrace are retained (needed to debug a crash) on the contract
  that they must not embed user capture. Validation errors that echo input
  (`QuorumError::UnsupportedTask(value)`) are **not** reported (see
  `report_if_unexpected`); only unexpected internal failures are.

## Status by client

| Client  | `send_default_pii=false` | `before_send` scrub | Mechanism |
|---------|--------------------------|---------------------|-----------|
| Rust    | ✅ | ✅ | `scrub_pii` in `ffi/src/lib.rs` |
| iOS     | ✅ | ✅ | `options.beforeSend` in `QuorumMobileApp.swift` |
| Android | ✅ (manifest) | ✅ | `scrubPii` via `SentryAndroid.init` in `QuorumApplication` |

All three clients now strip `server_name`/`user` before send. The Android client
moved from manifest auto-init to a code-based `SentryAndroid.init` in a custom
`Application` (auto-init disabled via `io.sentry.auto-init=false`); `init` still reads
the `io.sentry.*` manifest meta-data so the DSN/environment/`send-default-pii` config
stays single-sourced, and the lambda only adds the `beforeSend` scrub. Covered by
`ScrubPiiSpec` (Kotest, `app/src/test`).

## Consequences

- Crash reports lose device/user attribution — acceptable; we debug by release +
  stack, not by user identity.
- A future "who hit this crash" need must be met with an explicit, consented opt-in,
  not by re-enabling default PII.
- New telemetry surfaces (analytics, logs) inherit this boundary by default.
