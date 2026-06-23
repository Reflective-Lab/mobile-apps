# Mobile App ↔ Backend Pairing

**Every mobile app in the fleet has a corresponding backend server.** The mobile
app is the client; the backend is authoritative — "device proposes; server
promotes" (see [`docs/adr/0002-mobile-platform-boundary.md`](../adr/0002-mobile-platform-boundary.md)
and `EPIC.md`).

The backends are **not** in this repo. They are owned and deployed from
`../marquee-apps/<product>/` to **Google Cloud Run** (region `europe-west`).
This repo only needs to know *which* backend each app talks to and *where* it is.

## The fleet

| App (slug) | Backend (marquee-apps product) | Cloud Run service URL | API base (`<service>/<product>`) | Status |
|---|---|---|---|---|
| `quorum` | `quorum-sense` | `https://quorum-sense-backend-goak2gcj7a-ew.a.run.app` | `…/quorum-sense` | live |
| `atlas` | _TBD_ | _TBD_ | _TBD_ | planned |
| `vouch` | _TBD_ | _TBD_ | _TBD_ | planned |

Recorded in [`apps/backends.txt`](../../apps/backends.txt), alongside the
identity registry [`apps/registry.txt`](../../apps/registry.txt).

## URL contract

The backend repo emits two values (via `just cloud-url`):

```sh
QUORUM_SERVICE_URL=https://quorum-sense-backend-goak2gcj7a-ew.a.run.app
ATLAS_QUORUM_BASE_URL=https://quorum-sense-backend-goak2gcj7a-ew.a.run.app/quorum-sense
```

- **Service URL** — the Cloud Run service root.
- **API base** — `<service URL>/<product>`. A mobile app calls its backend at the
  API base (the `…/quorum-sense` path), not the bare root. Other apps reaching
  the same backend use the same base (hence `ATLAS_QUORUM_BASE_URL`).

## Deploy-time resolution

The exact Cloud Run URL is **assigned at deploy time**, so it must not be
hard-baked. The flow is:

1. The backend is deployed from `../marquee-apps/<product>/`; Cloud Run assigns a URL.
2. The URL is read back with `just cloud-url` in that repo.
3. It is injected into the mobile build by exporting `APP_BACKEND_URL` before
   sourcing `scripts/app-config.sh` — an environment value **wins** over the
   default recorded in `apps/backends.txt`:

   ```sh
   export APP_BACKEND_URL=$(cd ../marquee-apps/quorum-sense \
     && just cloud-url | sed -n 's/^QUORUM_SERVICE_URL=//p')
   APP=quorum source scripts/app-config.sh   # APP_BACKEND_URL now set
   ```

`scripts/app-config.sh` exports `APP_BACKEND_URL` for the selected app — the same
single-source-of-truth mechanism that already provides `APP_BUNDLE_ID` etc.

## What is wired today vs. next

**Wired now (this change):** the per-app backend URL is a first-class, deploy-
overridable config value (`apps/backends.txt` + `APP_BACKEND_URL`). Nothing in
the app consumes it yet — the mobile apps are currently offline/local through the
Rust core.

**Next, when the networking layer lands:** thread `APP_BACKEND_URL` from the
build into the app so the API client can read it. Sketch:

- **iOS** — add an Info.plist key in the generated project and let `xcodebuild`
  override it at deploy:
  ```yaml
  # project.yml → target settings.base
  INFOPLIST_KEY_ReflectiveBackendURL: ${APP_BACKEND_URL}
  ```
  ```sh
  xcodebuild … INFOPLIST_KEY_ReflectiveBackendURL="$APP_BACKEND_URL"
  ```
- **Android** — expose it as a `BuildConfig` field from a Gradle property:
  ```kotlin
  // app/build.gradle.kts → defaultConfig
  buildConfigField("String", "BACKEND_URL", "\"${project.findProperty("appBackendUrl") ?: ""}\"")
  ```
  ```sh
  ./gradlew :app:assembleRelease -PappBackendUrl="$APP_BACKEND_URL"
  ```

Keep the API base (`/<product>` path) construction in one place (the Rust core or
a thin platform client), mirroring how identity and observability are centralised.
