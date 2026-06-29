# ADR 0005: Marquee Offline Queue Durability — Native Store, Rust Contract

- Date: 2026-06-12
- Status: Accepted
- Related: ADR 0002 (platform boundary / app classes), M4.5–M4.9 in `MILESTONES.md`,
  `crates/mobile-core/src/persistence.rs`, `docs/architecture/quorum-ios-native-ai-collaboration-report.md`

## Context

M4.5 introduced `PersistedQueueRecord`: a versioned, Rust-validated document that
round-trips to `QueuedCapture`. The encoding is JSON today (serde-native, debuggable,
fixture-friendly). That raised a fair question: should durability live in Rust
(`redb`, SQLite) instead of native stores?

Two different persistence stories exist in the portfolio:

| App class | Device role | Persistence bias |
|---|---|---|
| **Marquee governed** (Quorum) | Thin client; server is authority for inquiry/admission | Platform queue store + Rust contract |
| **Studio local-first** (Inkling, etc.) | Device-authoritative vault | `runway-storage` / redb in Rust (Runway) |

Quorum's offline queue is **sync/retry state**, not the Lamport/Merkle ledger.
Embedding a Runway-style redb vault in the Marquee mobile core would conflate
those concerns.

## Decision

**For Marquee governed apps (Quorum first):**

1. **Keep M4.5 as-is.** `PersistedQueueRecord` is the durable **contract** — schema
   version, validation, round-trip — not a choice of storage engine.
2. **JSON is the record encoding**, not the storage engine. Native code may store
   opaque bytes produced by `PersistedQueueRecord::to_json()`. A more compact
   encoding (e.g. postcard) may replace JSON **without changing the contract**
   if size becomes material.
3. **Durability and background scheduling are native** (M4.6 iOS, M5.6 Android,
   M4.7 / WorkManager). Swift/Kotlin hold bytes; Rust holds schema, transitions,
   and idempotency rules.
4. **Defer `RedbQueueStore` in `mobile-core`** until at least one of:
   - a second Marquee app needs identical queue semantics,
   - iOS and Android adapters duplicate retry/idempotency/reconciliation logic,
   - a Studio app shares the same queue crate with Quorum.
5. **If redb is added later**, store the **same** `PersistedQueueRecord` bytes.
   Do not invent a parallel schema.

**Explicitly avoid:** native store **and** Rust redb both owning Quorum queue
durability.

## Layering

| Layer | Owner |
|---|---|
| State machine, transitions, idempotency rules | Rust (`reflective-mobile-core`) |
| Serialized record shape | Rust (`PersistedQueueRecord`) |
| Durability + BG scheduling | Native (M4.6–M4.7, M5.6) |
| Admission truth | Quorum server (M4.8–M4.9) |

redb in Rust remains a valid **implementation swap** behind a future `QueueStore`
trait for portfolio consolidation — not a repudiation of M4.5.

## Consequences

- M4.6 implements iOS persistence: Core Data / SwiftData / file — keyed by
  `record_id`, value = JSON blob, reload via `from_json`, call Rust for
  transitions before write.
- M5.6 mirrors the pattern on Android (Room / DataStore).
- Runway/studio local-first apps continue to use `runway-storage` (redb + FS)
  per Runtime Runway architecture; that path does not block Quorum M4.6.

## Revisit

Re-evaluate Rust `QueueStore` + redb when **N ≥ 2** Marquee apps ship queue
adapters or measured logic duplication across iOS/Android exceeds maintainability
comfort (track in `MILESTONES.md` backlog if needed).
