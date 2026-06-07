# Mobile Apps

Mobile apps are grouped by product family.

- `marquee/` contains thin JTBD commercial proof surfaces aligned with
  `../marquee-apps/`.
- `studio/` contains creative, research, notes, writing, and presentation
  surfaces aligned with `../studio-apps/`.

Each real app should eventually contain:

```text
ios/          SwiftUI shell
android/      Kotlin/Compose shell
fixtures/     cross-platform behavior fixtures
README.md     product-specific mobile charter
```

## Starting a new mobile app

To bootstrap a new app, copy the platform shell templates and rename:

- iOS: `cp -r templates/native-shells/ios apps/<family>/<app>/ios` — see `templates/native-shells/ios/README.md` for the adoption path (rename `ReflectiveShell` → product name in `project.yml` + sources, update `applicationId`).
- Android: `cp -r templates/native-shells/android apps/<family>/<app>/android` — see `templates/native-shells/android/README.md` for the adoption path (rename `dev.reflective.shell` → product `applicationId`/`namespace`/package, update `rootProject.name`).

Both templates already include a UniFFI round-trip against the product-neutral `crates/shell-ffi/` crate. Real apps swap that out for their product's Rust crate(s) — Quorum apps point at `marquee-apps/quorum-sense/crates/` per `docs/architecture/quorum-sense-boundaries.md`.

