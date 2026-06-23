# Spike: reuse `quorum-domain` instead of duplicating it (Epic 4)

**Question:** can the mobile FFI depend on the canonical `quorum-domain` crate
from `Reflective-Lab/quorum-sense` instead of re-implementing domain logic in
`crates/mobile-core`, without dragging server-only dependencies into the phone
binary?

**Verdict: technically viable.** It resolves, is server-free, and cross-compiles
to iOS + Android. But it is **not a drop-in** — the canonical domain is a richer,
different model than the mobile fixture, so adopting it is a real modelling
migration (which is exactly the point of Epic 4).

## Evidence (spike branch `spike/reuse-quorum-domain`)

A throwaway `crates/spike-domain-probe` depends on `quorum-domain` pinned by rev:

```toml
quorum-domain = { git = "ssh://git@github.com/Reflective-Lab/quorum-sense", rev = "5ed4342c…" }
```

- **Resolves.** `quorum-sense` is private → use the **SSH** git URL +
  `CARGO_NET_GIT_FETCH_WITH_CLI=true`. No tags exist yet, so pin by **rev**.
- **Foundation comes from crates.io.** `quorum-domain` → `converge-core = "3.8.1"`
  (resolves to 3.9.2 on crates.io), `axiom-truth = "0.15.2"`. The local-path
  `[patch.crates-io]` entries in `quorum-sense` (`../../bedrock-platform/…`,
  `../../mosaic-extensions/…`) are **co-development only** and are NOT needed
  downstream — Cargo doesn't apply a dependency's `[patch]`, and crates.io
  satisfies the version requirements.
- **Server-free.** The closure is **60 crates** with **zero** tokio / axum /
  hyper / reqwest / tonic / openssl / sqlx. `converge-core` (despite its "Agent
  OS runtime" tagline) is lean here: serde, sha2, strum, tracing, typed-builder,
  uuid. Direct deps of `quorum-domain`: chrono, converge-core, serde, serde_json,
  thiserror, uuid.
- **Cross-compiles.** `cargo build -p spike-domain-probe --target …` succeeds for
  host, `aarch64-apple-ios`, and `aarch64-linux-android` (~12s each).

## The catch: it's the real model, not the fixture

`quorum-domain` exposes the canonical inquiry model — `InquiryId`, `SignalId(Uuid)`,
`HypothesisId`, `ProbeId`, `ContentHash`, `LamportClock`, `MerkleRoot`, citations,
content-addressed integrity. The mobile `mobile-core` is a deliberately simplified
**fixture**: `SignalModality` / `ConsentState` / `Confidence` / `draft_field_signal`
with string ids.

So reuse is **not** "delete mobile-core, import quorum-domain." It is:
- adopt canonical ids/types at the FFI boundary (e.g. `SignalId` instead of the
  `draft:…` string), and
- locate where capture-flow concepts (modality / consent / confidence) actually
  live canonically — likely `quorum-evidence` / `quorum-kernel`, not
  `quorum-domain` — and map the mobile capture workflow onto them.

The UniFFI surface (string/float `Ffi*` DTOs) can stay stable, so Swift/Kotlin are
unaffected; only `quorum-ffi`'s internal mapping changes.

## Recommended path

1. **Pin by rev now; tag later.** Ask the platform team to cut a `quorum-domain`
   tag so we pin `tag = "vX.Y.Z"` instead of a rev. Graduating `converge-core` +
   `quorum-domain` to proper published versions removes the SSH-git dance entirely.
2. **Guardrail first** (this branch): `deny.toml` bans server-only crates in the
   workspace; wire `cargo deny check bans` into CI so reuse can never silently
   pull the server in (EPIC lines 99 & 215).
3. **Incremental adoption.** Replace one seam at a time in `quorum-ffi`'s mapping
   (start with ids), keeping `Ffi*` DTOs and the fixtures as the contract. Delete
   the corresponding `mobile-core` duplication as each seam lands.
4. **CI access.** The mobile CI needs read access to the private `quorum-sense`
   repo (deploy key / org token) for the git dep, until the crates are published.

## Cleanup

`crates/spike-domain-probe` is throwaway evidence — it must not merge to `main`
(it makes the default workspace build require SSH access to the private repo).
Keep `deny.toml` and this doc; drop the probe.
