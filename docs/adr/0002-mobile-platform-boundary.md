# ADR 0002: Mobile Platform Boundary — Device Proposes, Server Promotes

- Date: 2026-06-12
- Status: Accepted
- Supersedes: informal placement notes in `docs/architecture/mobile-shared-core-study.md`
  (study retained as evidence and spike index)
- Related: ADR 0001, `docs/architecture/native-ai-rust-core.md`,
  `docs/architecture/quorum-sense-boundaries.md`

## Decision

Reflective mobile apps share one **authority boundary** across the portfolio:

> **The device proposes; the server promotes.**

Mobile owns capture, consent, draft-stage normalization, deterministic
validation, offline queue, and rendering of server-computed traces. Mobile does
**not** own governed promotion, multi-party integrity proofs, formation assembly,
Mosaic specialist execution, credentials, or billing semantics.

This is an **authority** line, not a technology line. `converge-kernel` is
embeddable in-process; mobile still must not run the promotion-authoritative
Converge loop for governed marquee apps.

### Fixed rules (all apps)

1. **No fact promotion on device** for marquee governed apps — no round close,
   no Lamport/Merkle chain mutation, no governing rulebook evaluation as
   authority.
2. **No Mosaic credentials on device** — Manifold LLM adapters, Embassy live
   ports, Mnemos/Ferrox/Soter servers stay server-side; on-device AI uses
   native platform runtimes (Foundation Models, Gemini Nano) per ADR 0001.
3. **Reuse canonical domain crates** — per-app `domain` types and logic come
   from `marquee-apps/` or `studio-apps/` via path dependency + UniFFI facade;
   no parallel Swift/Kotlin domain forks.
4. **One shared mobile-core** — capture workflow, consent, offline queue, sync
   client, replay fixtures; one thin `mobile-<app>-ffi` per product.
5. **Transport follows the app server** — mobile calls each product's existing
   HTTP/SSE (or future gRPC) surface; mobile does not invent a second wire
   format. Portfolio-wide sync contract lives in `mobile-core` once Quorum M2
   proves capture → queue → submit.

### Per-app classification (required before scoping)

Every mobile companion must be classified into one of three app classes before
crate placement and transport design:

| App class | Where authority lives | Device may carry |
|---|---|---|
| **Marquee governed** | App server | Contract types, domain validation, drafts, queue, trace rendering |
| **Studio local-first** | Device (vault/store) | Domain + kernel + local store + optional Organism for recall |
| **Studio hybrid** | Split by surface | Capture/notes on device; panel/reasoning on server |

The rule is fixed; the **classification is per app**.

## Device vs server placement

### Near the user (Rust on device + native AI)

- `converge-pack`, `organism-pack`, `embassy_pack` contract types
- Per-app `domain` crate logic that is deterministic and mobile-clean (see
  spikes under `spikes/`)
- `converge-fuzzy-inference` for **preview** scoring only (marked draft)
- Axiom-style invariant checks and portable WASM artifacts where applicable
- Offline queue, consent state, sync protocol client
- Rendering of server-computed activated rules, receipts, lineage

### Server only (app server + Runtime Runway)

- Converge engine loop, promotion gate, `ContextState`, fact lineage, HITL
- Organism formation assembly, adversarial review, simulation
- Manifold suggestors, Embassy observations, Mnemos recall, Ferrox/Soter,
  full Prism/Crucible
- Helm workbench semantics; mobile provides capture/approval surfaces instead
- Stripe, entitlements, secrets, multi-party audit artifacts

### Converge on mobile — allowed vs forbidden

| Operation | Marquee governed | Studio local-first |
|---|---|---|
| Deserialize/render facts and traces | Yes | Yes |
| Deterministic domain validation | Yes | Yes |
| Draft / preview suggestor pass (explicitly non-authoritative) | Yes | Yes |
| Promotion gate, fact admission | **No** | Yes when device is authority |
| Integrity chain (Lamport, Merkle) mutation | **No** | Per app charter |
| Governing rulebook as authority | **No** | Per app charter |

## Layering

```text
Swift/Kotlin (UI, camera, mic, platform AI)
  -> UniFFI (per-app .udl)
mobile-<app>-ffi
  -> mobile-core (capture, consent, queue, sync)
  -> portable slice (converge-pack, organism-pack, domain crates, fuzzy preview)
  ── network boundary ──
app server (Converge, Organism, Mosaic, Helm receipts)
```

Evidence: `spikes/quorum-domain-mobile/` (marquee domain reuse, mobile-clean),
`spikes/inkling-notes-mobile/` (studio reuse; not mobile-clean until upstream
feature split), `apps/marquee/quorum-sense/ffi` (UniFFI product track for Quorum).

## Per-app placement worksheet

Fill or update this table when scoping a mobile companion. **Transport** must
match the canonical app repo's server API unless an ADR amends it.

| App | Class | Authority | Device carries (initial) | Transport (canonical) | Converge preview on device |
|---|---|---|---|---|---|
| quorum-sense | Marquee governed | Server | `quorum-domain`, capture draft, queue | HTTP + SSE (same as web SPA) | Validation only; no kernel |
| atlas-integration | Marquee governed | Server | Domain types, evidence capture, queue | App server HTTP (TBD at M2+) | Validation only |
| tally-escrow | Marquee governed | Server | Agreement/custody DTOs, approval capture | App server HTTP (TBD) | Validation only |
| vouch-lending | Marquee governed | Server | Applicant evidence DTOs, capture | App server HTTP (TBD) | Validation only |
| scout-sourcing | Marquee governed | Server | RFP/vendor evidence, photos | App server HTTP (TBD) | Validation only |
| plumb-execution | Marquee governed | Server | Drift/proof capture | App server HTTP (TBD) | Validation only |
| triage-keeper | Marquee governed | Server | Incident intake DTOs | App server HTTP (TBD) | Validation only |
| catalyst-biz | Marquee governed | Server | Operating-room signals | App server HTTP (TBD) | Validation only |
| fathom-narrative | Marquee governed | Server | Narrative evidence capture | App server HTTP (TBD) | Validation only |
| warden-compliance | Marquee governed | Server | Verdict/evidence capture | App server HTTP (TBD) | Validation only |
| inkling-notes | Studio local-first | Device | `notes` navigation/capture, vault, local index | Optional sync API (TBD) | Local vault authority |
| wolfgang-chat | Studio hybrid | Split | Capture/notes on device; panel on server | Web/API for panel; local for notes | Panel server-side |
| folio-editor | Studio local-first | Device (edition) | Editorial triage, beat tags | App server HTTP (TBD) | Per charter |
| moosemen-writer | Studio local-first | Device | Writing/voice capture | Local-first (TBD) | Per charter |
| wykkid-preso | Studio local-first | Device | Rehearsal capture | Local-first (TBD) | Per charter |

Quorum row is the reference implementation — see
`docs/architecture/quorum-sense-boundaries.md`.

## Options considered

1. **Portfolio authority boundary (chosen)** — fixed rule, per-app classification
   and transport.
2. **Embed full Converge kernel on device** — rejected for marquee governed
   apps: duplicates authority, breaks multi-party integrity, expands binary
   with tokio/Mosaic deps.
3. **Mobile-specific REST/gRPC layer** — rejected as default; duplicates each
   app server's contract. Accept only if a future ADR defines a shared sync
   envelope in `mobile-core`.
4. **Per-app boundary with no portfolio rule** — rejected; spikes showed
   fifteen apps share the same capture surface pattern.

## Rationale

Marquee desktop + web already split "near user" vs "backend with all
capabilities." Mobile adds offline capture and native AI; without a portfolio
rule, each shell would re-implement domain types and drift on promotion
authority.

Spikes (2026-06-12) showed marquee `domain` crates can ship mobile-clean;
studio `notes` requires upstream feature discipline. The boundary must be
stable before M2 wires Quorum shells to generated UniFFI bindings.

## Consequences

- Product mobile work starts with **worksheet classification**, then domain
  crate footprint audit (`cargo tree`), then UniFFI facade — not with Converge
  engine embedding.
- `mobile-core` owns sync/queue; individual apps do not fork offline protocols.
- Studio local-first apps may carry more Organism on device; marquee apps never
  embed Manifold/Embassy live ports.
- CI drift check: new `mobile-<app>-ffi` crates must not depend on
  `converge-kernel`, `converge-manifold-adapters`, or Embassy live crates
  without an ADR exception.
- Architecture reviews for new mobile apps cite this ADR and update the
  worksheet row.

## Follow-up

- Wire Quorum `PreviewQuorumCoreBridge` to generated `quorum_ffi`
  bindings (M2 technical step).
- Define `mobile-core` offline queue + optimistic reconciliation contract when
  Quorum field capture ships.
- Promote worksheet **Transport** cells from TBD as each app names a mobile job.
- Studio-apps: optional `sources-web` on `inkling-notes` `notes` crate so
  mobile path stays network-free (tracked in inkling spike README).
- Add scaffold-check or CI lint for forbidden mobile dependencies (follow ADR
  drift check above).
