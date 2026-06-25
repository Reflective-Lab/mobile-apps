# Changelog

All notable changes to **Quorum Mobile** are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and the project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
`QF-*` identifiers refer to the quality ledger in the umbrella repo's
`QUALITY_BACKLOG.md`.

## [Unreleased]

### Security
- Gradle dependency verification is now **strict**: a tampered, swapped, or unlisted
  dependency **fails** the build (previously lenient — warn-only). The
  `verification-metadata.xml` is a macOS+Linux superset (581 components), regenerated
  via a new `workflow_dispatch` job (`android-verify-metadata.yml`) that runs on a
  Linux CI runner so the build-tooling artifacts the runner resolves are included.
  (`QF-2026-06-24-04`)

## [0.1.1] - 2026-06-24

First stable release after the 2026-06-24 mobile security audit and the Android
toolchain modernization.

### Added
- Android: code-based Sentry initialization (`QuorumApplication`) with a `beforeSend`
  PII scrub — completes the telemetry-PII boundary across all three crash clients
  (Rust, iOS, Android). (`QF-2026-06-24-05`, ADR 0004)
- Fuzzing: committed seed corpus + PR-time smoke for the libFuzzer harness.
  (`QF-2026-06-24-06`)

### Changed
- **Android toolchain upgrade** — one coordinated change superseding five Dependabot
  PRs: AGP 8.7.3 → 9.2.1 (built-in Kotlin), Gradle 8.14.2 → 9.4.1, Kotlin 2.0.21 →
  2.4.0, Kotest 5.9.1 → 6.2.1, mockk 1.13.13 → 1.14.11; `buildToolsVersion` 36.0.0.
- Version: Android `versionName` 0.1.1 (`versionCode` 2); iOS `MARKETING_VERSION`
  0.1.1. Sentry release tag `quorum@0.1.1`.

### Security
- M1 milestone: authority-leakage guard enforced.

## [0.1.0] - 2026-06-24

Initial release — security-hardening foundation from the 2026-06-24 boundary audit.

### Added
- libFuzzer harness over the untrusted-input seam. (`QF-2026-06-24-06`)
- Gradle dependency verification (sha256 `verification-metadata.xml`, lenient
  rollout) + Gradle distribution `distributionSha256Sum` pin + wrapper-jar
  validation in CI.
- Strict-enum FFI contract — modality/consent/event/sync travel as wire enums, so an
  unknown value is unrepresentable across the UniFFI seam.

### Security
- RUSTSEC advisory gate (`cargo deny check advisories`) enforced in CI.
  (`QF-2026-06-24-01`)
- FFI panic-safety: `forbid(unsafe_code)` + `deny(unwrap/expect/panic)` on the pure
  domain crates that run under the UniFFI seam. (`QF-2026-06-24-02`)
- All GitHub Actions SHA-pinned + Dependabot for actions/cargo/gradle.
  (`QF-2026-06-24-03`)
- Telemetry-PII scrub (`beforeSend` + `send_default_pii=false`) on the Rust and iOS
  crash clients. (`QF-2026-06-24-05`, ADR 0004)
- `sentry` (Rust) 0.34 → 0.48 — drops the vulnerable `rustls-webpki`.

[Unreleased]: https://github.com/Reflective-Lab/mobile-apps/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/Reflective-Lab/mobile-apps/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/Reflective-Lab/mobile-apps/releases/tag/v0.1.0
