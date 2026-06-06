# Milestones

Two tracks. Product logic for Quorum stays canonical in
`../marquee-apps/quorum-sense` through marquee-apps v1. The mobile workspace may
carry Quorum shell scaffolds, fixtures, and bridge contracts so the platform
foundation can advance without forking the Quorum domain. Real product scope is
still gated by `docs/architecture/quorum-sense-boundaries.md`.

## M1 — Platform foundation: build + CI + simulators (due 2026-06-30)

Smoke shells live under `templates/native-shells/{ios,android}/`. Product
candidate scaffolds may live under `apps/`, but they must remain shell-only:
capture, draft, consent, queue, and bridge tests. No Quorum kernel logic lives in
mobile.

### iOS pipeline
- [ ] `templates/native-shells/ios/` builds a minimal SwiftUI app from CLI (`xcodebuild`)
- [ ] Runs on iOS simulator from a single `just ios-sim` (or equivalent) command
- [ ] UniFFI: Rust crate → Swift bindings → linked into the smoke shell, one round-trip call proves it
- [ ] GitHub Actions macOS runner builds the smoke shell on every push

### Android pipeline
- [ ] `templates/native-shells/android/` builds a minimal Compose app from CLI (Gradle)
- [ ] Launches on an AVD from a single `just android-sim` (or equivalent) command
- [ ] UniFFI: Rust crate → Kotlin bindings → linked into the smoke shell, one round-trip call proves it
- [ ] GitHub Actions Linux runner builds the smoke shell on every push

### Shared
- [ ] `just check` runs Rust + both native builds locally
- [ ] Document the per-app adoption path: "to start a new mobile app, copy `templates/native-shells/{ios,android}` and wire UniFFI to your Rust crates"
- [ ] Quorum boundary stays enforced: mobile Swift/Kotlin source may exist only as shell/fixture/bridge code until the paid mobile job is named; no domain/kernel/business logic under mobile

## M2 — Quorum field-capture shell, iOS first (unlocked by trigger, not by date)

Starts when a paying Quorum team names a mobile-specific job. Inherits the M1
pipeline and promotes today's scaffold into product code.

- [ ] iOS SwiftUI shell: capture text/voice/photo signal → local draft → submit via canonical HTTP API
- [ ] UniFFI bindings from marquee-apps Quorum crates expose only the DTOs needed for capture + submission
- [ ] Speech + AVFoundation + PhotosUI wired; Foundation Models draft normalization on-device
- [ ] Offline queue (BGTaskScheduler); reconciles via optimistic admission receipts
- [ ] No domain types, no kernel logic, no Stripe — enforced by `quorum-sense-boundaries.md`
