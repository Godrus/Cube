# Processing Foundation Contract (TC-01 implementation record, PROVISIONAL)

**Status:** Provisional. Not an approval of PR-01...PR-08 (GOV-07), not the Geometry Processing Specification (EXT-GPS, absent), and not Frozen. Written for PS-2 under the owner's permission to use TC-01 (DEC-077). It records what `crates/cube-processing` actually implements, so that the later diff against the real specification (gate RG-2, migration step of TC-01) has a fixed left-hand side.

**Scope rule (TC-01):** only snapshot, dependency revision, task, result and cancellation (PR-01...PR-05), executor classes minimal (PR-03/PR-06), per-brick stamps, pin-only snapshots, tunable caps. No content of the absent specification is assumed.

## 1. The ten points

| # | Topic | What the code does | Where |
|---|---|---|---|
| 1 | **Input and snapshot semantics** | A snapshot is built from one committed `AssetState` (one asset revision), covers a core brick box plus an apron of N bricks, and pins the `Arc` payload handles of those bricks only. It never copies voxel data and never retains the whole state. Each brick keeps the five Core read states; absent data is never turned into "empty". Reading outside the captured area is an error. | `snapshot.rs` |
| 2 | **Dependencies** | A provider declares the change types it reacts to, an apron (at most 8 bricks) and derived kinds it needs (same brick). Derived inputs are per brick. A result is stamped with the revision of every captured brick, the provider version and parameters, and the publication sequence of each derived input. | `provider.rs`, `snapshot.rs` |
| 3 | **Execution order** | The registry freezes into a graph with one deterministic topological order (ties: smaller kind id). Missing dependencies, duplicate kinds, self or repeated dependencies, oversized aprons and cycles are all reported at freeze time. A task waits for the tasks of its derived inputs; scheduling is by class, then weight, then task id, with aging (promotion by one class every `aging_ticks` scheduling passes). | `graph.rs`, `executor.rs` |
| 4 | **Cancellation** | Cooperative tokens (one-way latch, first reason wins, parent/child, callbacks exactly once, deadlines through a `Clock` port). A queued task ends at once; a running task ends at the provider's next check; a task is also checked before capture and before publication. Cancelling a task cancels its queued dependents. A commit that changes the inputs of a running task cancels it as `Superseded`. | `cancel.rs`, `task.rs`, `executor.rs` |
| 5 | **Stale results** | A result enters a slot only if, under the slot lock, its input stamp equals the newest live stamp the slot has seen. The expectation only moves forward by `(epoch, asset revision, derived sequences)`. The live stamp is re-read after the computation and applied in the same critical section, so a missed notification cannot publish a stale value. Stale results end the task as `Stale`; nothing is stored. A stale value is never returned as current. | `slot.rs`, `executor.rs` |
| 6 | **Resource limits** | `ExecutorConfig`: running tasks (total and per class), queue length (a full queue rejects, it never drops), aging. Snapshot size is bounded by a hard safety limit (2^22 bricks) and by a configured pin budget (PROC-06, pending GOV-24). These are machine limits; the world is unbounded. | `executor.rs`, `snapshot.rs` |
| 7 | **Responsibility boundary** | Processing reads `SharedAsset` only and cannot name a commit handle (archcheck rules ARCH-HANDLE and the crate graph: `cube-processing -> cube-core`). It has no game logic, no async runtime, no clock of its own, no I/O. Providers are supplied by their owners; Processing knows only opaque payloads. | `cargo xtask archcheck` |
| 8 | **Errors and states** | Task states `Created, Queued, Running, Completed, Failed, Cancelled, Stale` with a checked transition table; terminal states are final. `Completed` means published. A failing or panicking provider is `Failed` (code 3007) and publishes nothing. Error codes 3001...3009. | `task.rs`, `error.rs` |
| 9 | **Determinism** | Registry order, invalidation targets and published values are deterministic for equal state and provider outputs, for any worker count (tested with 1, 4 and 8 workers). The execution order of independent tasks is fixed only for a single `run_one` caller. | tests |
| 10 | **Safe publication** | One writer path into a slot (`DerivedSlot::publish`); results are `Arc` values replaced atomically; readers get either the current value or an explicitly flagged older one (`last_known`); there is no window in which a stale value is stored and corrected later. No automatic recomputation: the owner of the data resubmits. | `slot.rs` |

## 2. Status of each statement

**Confirmed by existing decisions:** layer 1 above Core, depends only on Core (AU-1.0 section 5.3, ADG Part B); Processing never commits geometry (PR-04); snapshots are immutable and ref-counted (PR-01); cycles are rejected at registration (PR-07); the task state machine and its terminal states (PR-03); cooperative cancellation with a one-way latch (PR-05); class names of priority (PR-06); revision tokens, epochs and the incomparable-is-stale rule (C-16, DEC-059...062); per-brick tokens (TC-01).

**Provisional (decided here, reversible, tunable):** pin-only snapshots; per-brick stamps including provider version, parameters and derived sequences; payloads as `Arc<dyn Any + Send + Sync>` (PR-04 "payload typing" open); derived inputs limited to the same brick; `MAX_APRON = 8`; the aging rule; default limits (4 running, per-class limits, 4096 queued); the hard snapshot limit; deadlines as ports; no automatic recomputation.

**Undefined (the absent specification, not assumed):** providers for Surface Mesh, Collision, Hitbox and LOD (O-1); granularity beyond per brick; copy-versus-pin threshold; BVH / spatial hierarchy and its query API (PR-08); weighting and per-platform policy tables (PR-06); engine ports (PR-09); task kind extension and exposure of handles to tools; memory-cap policy and snapshot lifetime (OD-R4, GOV-24).

**Can be deferred without blocking PS-2:** BVH and bounds slots beyond the brick-level bounds provider; per-platform priority tables; handle exposure to tools; derived inputs of other bricks.

**Needed for final approval (GOV-07):** the Geometry Processing Specification (EXT-GPS); an RG-2 diff of PR-01...PR-08 against it; a decision on OD-R4; the owner's explicit reading of ODM-02 (whether a Provisional registration satisfies AOD-24's condition).

## 3. Expiry and migration

Expiry is the earlier of GOV-07 accepted and the start of M3 content work (TC-01). At that point diff this file against the specification; a budgeted rework task is expected, not estimated. No other crate may import internals: everything public is re-exported from the crate root.
