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
| Android | ✅ (manifest) | ⚠️ **gap** | manifest meta-data; beforeSend needs code init |

## Residual / follow-up

The Android client auto-initialises from manifest meta-data, which **cannot** register
a `beforeSend` scrub. Closing this requires a code-based init (a custom `Application`
or `SentryAndroid.init` with an options callback / `EventProcessor`). Tracked under
`QF-2026-06-24-05`. Until then Android relies on `send-default-pii=false` plus the
contract that capture text never enters a logged exception.

## Consequences

- Crash reports lose device/user attribution — acceptable; we debug by release +
  stack, not by user identity.
- A future "who hit this crash" need must be met with an explicit, consented opt-in,
  not by re-enabling default PII.
- New telemetry surfaces (analytics, logs) inherit this boundary by default.
