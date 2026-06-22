# ADR 0003: Responsive-by-Construction — Off-Main Linearized Core, Snapshot-Projected Views

- Date: 2026-06-22
- Status: Accepted
- Related: ADR 0001 (native Swift/Kotlin + shared Rust core), ADR 0002 (platform
  boundary), `AGENTS.md` Engineering Principles, `apps/marquee/quorum-sense`
  (reference implementation),
  `docs/architecture/quorum-ios-native-ai-collaboration-report.md` (the product
  surfaces — capture / consent / live status / offline queue — that ride on
  these invariants)

## Context

Several portfolio apps are **real-time collaboration** surfaces: multiple humans,
on-device AI, and a session that spans other devices, all editing shared state
concurrently. Two properties are therefore **hard invariants**, not quality
targets:

- **Responsiveness** — the app stays interactive *no matter what else is
  happening*: a local model mid-inference, a peer's operations landing, and a
  stalled network, simultaneously.
- **Consistency** — the view always shows one coherent, current, correct
  snapshot of the world; never a torn or half-merged intermediate, never a
  flicker back to a stale state.

The system is **async and parallel by nature** (three concurrent writers: local
user, local AI, remote peers). These two invariants conflict only if the UI
reads mutable shared state. The resolution is to make the view a projection of
immutable, versioned snapshots produced by a single off-main linearization
point.

## Decision

### Invariant I — Responsiveness

> The interaction loop (input → next frame) is **never** coupled to any
> unbounded-latency operation: AI inference, network sync, disk, or merge.

### Invariant II — Snapshot consistency

> The view is a **pure projection of immutable, versioned, internally-consistent
> snapshots**, swapped atomically and applied monotonically by version. The UI
> never reads mutable shared state.

### Reference architecture

**Parallel compute, serialized commit, atomic publish.** Concurrent ops are
computed in parallel and off-main (inference, merge, network); state transitions
are committed in causal order through one core; each commit publishes one
coherent snapshot.

```text
intents (fire-and-forget) ─────────────────►  ┌──────────────────────────────┐
                                               │  Rust core actor (off-main)  │
local user ─┐                                  │  • single linearization point│
local AI  ──┼─ parallel, async producers ───►  │  • CRDT merge (local+remote) │
remote peers┘                                  │  • emits versioned snapshots │
                                               └───────────────┬──────────────┘
SwiftUI / Compose view  ◄── snapshot stream ───────────────────┘
(holds ONE snapshot, atomic swap, monotonic by version)
```

## Fixed rules (all real-time apps)

1. **No core call on the main thread/actor.** The FFI bridge is actor-isolated
   and runs off-main; UI `await`s it. A `@MainActor` bridge that calls
   synchronous FFI is forbidden.
2. **The UI reads only immutable snapshots** — swapped atomically, applied
   monotonically by version. No mutable shared state, no in-place mutation the
   view can observe.
3. **The core is the single linearization point.** All mutation (local user,
   local AI, remote peers) commits through it in causal order; each commit emits
   exactly one coherent snapshot.
4. **Intents in, events out.** UI sends fire-and-forget intents that return
   immediately; results and updates arrive *only* over the snapshot stream
   (UniFFI async functions + callback interface → Swift `AsyncStream`, Kotlin
   `Flow`). The UI never polls and never blocks on a result.
5. **Everything is cancellable and bounded.** Superseded AI/network work is
   cancelled; rapid input is coalesced/debounced; queues are bounded and shed or
   merge stale work under backpressure — they never grow until something stalls.
6. **Snapshots are coherent by type.** Domain types make inconsistent world
   states unrepresentable (per `AGENTS.md`); the core may only emit a
   type-valid snapshot. "Correct snapshot" is enforced at the boundary, not
   asserted at runtime.
7. **On-device AI runs off the interaction path** on a dedicated executor,
   streams partial results, and is cancellable. Never a blocking await of a full
   result.
8. **One pipeline for local and remote.** Remote ops (sync transport: HTTP/SSE,
   future HTTP/3; APNs/FCM only as a wake signal) feed the *same* core, the
   *same* CRDT, and the *same* snapshot stream as local ops — no special-casing.

## Optimistic echo

Local input applies instantly as a **provisional but coherent** snapshot (the
optimistic projection). When the core confirms/merges, it emits the reconciled
snapshot and the UI swaps. At every instant the view shows *a* valid snapshot —
optimistic or authoritative — never a half-reconciled one. Snapshots may carry
liveness metadata (version, peer cursors, unsynced-local-op count) so "ahead of
server" / "offline" is part of the rendered truth, not a separate blocking
check.

## Applicability

- **Invariants I and II are universal** across the fleet.
- The full CRDT / multi-writer machinery applies to **real-time collaborative**
  apps (humans + AI + multi-device sessions).
- **Capture-only** marquee apps still obey I and II (off-main core,
  snapshot-projected UI) but may run a degenerate single-writer core without
  cross-peer merge.

## Options considered

1. **Off-main linearized core + snapshot stream (chosen).** Responsive and
   consistent by construction; one model serves render and merge.
2. **`@MainActor` bridge with synchronous FFI** — rejected. `async` without
   offloading still blocks the main thread; freezes the UI the moment a heavy
   core call (embeddings, vector search, inference) runs. This is the prototype
   shape in the current Quorum bridge and must be replaced before it propagates
   to other apps.
3. **Request/response polling of mutable core state** — rejected. The UI can
   observe partially-applied mutation (torn reads) and must poll, which is both
   inconsistent and not responsive under concurrent writers.
4. **Shared mutable state behind locks** — rejected. Lock contention on the
   interaction path reintroduces stalls; torn reads reintroduce inconsistency.
   Immutable snapshots make reads lock-free and the actor serializes writes.

## Consequences

- The Quorum reference bridge is **rebuilt** from the `@MainActor` synchronous
  shape to an actor-isolated core that streams snapshots; apps #2…N inherit
  responsiveness and consistency by copying the reference, not by rediscovery.
- `mobile-core` owns the snapshot/stream and optimistic-reconciliation contract
  (extends the ADR 0002 offline-queue follow-up); individual apps do not fork it.
- The typed-domain work (illegal states unrepresentable) is load-bearing here —
  it is what makes snapshots correct by construction; weakening it weakens
  consistency.
- UniFFI's async + callback-interface support is now a **required** capability,
  not incidental — it is the mechanism for the snapshot stream on both platforms.

## Follow-up

- Replace `PreviewQuorumCoreBridge`/`QuorumCoreBridgeFFI` synchronous calls with
  an actor-isolated core exposing intents + a snapshot `AsyncStream`/`Flow`
  (supersedes the ADR 0002 "wire Quorum bridge" step).
- Define the `mobile-core` snapshot envelope: immutable payload + version
  (Lamport/vector clock) + liveness metadata; specify monotonic apply + stale
  drop.
- Specify the CRDT used for collaborative document/session state and where merge
  runs (core actor, off-main).
- Add a CI/lint check that the FFI bridge is not `@MainActor` and that core
  calls are not made on the main thread.
- Per-app: mark which apps are real-time collaborative vs capture-only in the
  ADR 0002 worksheet, since that selects the CRDT machinery.
