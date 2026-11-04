# System Boundaries v1

**Status:** **Proposed** architecture description for the owner, derived from the Approved Owner Decisions (AOD-01…19, `docs/decisions/owner-architecture-decisions-v1.md`). It does not change AU-1.0, ACR-001/002/003, any contract (all stay Draft, AOD-18) or the dependency grid in the QA Pack. Anything marked **(proposal)** is not decided.
**Scope:** processes, trust zones, modules, write paths, data ownership and the interfaces between them, from the current Rust + Godot spike to a client/server voxel RPG. Behaviour of individual features is out of scope (see `roadmap-v1.md`).

## 1. Processes and trust zones

```
                    TRUST BOUNDARY (network / process)
  CLIENT PROCESS (untrusted)                 |            SERVER PROCESS (authoritative, headless Rust)
  ------------------------------------------+------------------------------------------------------
  Frontend: Godot 4.x (replaceable)          |  Server host: sessions, auth, permissions, tick
   - UI, input, camera, meshes (GDScript glue)|  Editing service  (user/designer operations)
  Client runtime (Rust, GDExtension)         |  WorldGen service + Materialiser (system writer)
   - cache of chunks, prediction, meshing    |  Geometry Core (authoritative asset, snapshots)
   - protocol client                          |  Persistence (seed + generator version + journal)
  Geometry Core view (read-only cache)       |  Replication, interest management
  ------------------------------------------+------------------------------------------------------
```

* The **server** is a separate headless Rust process built from the same Core crates (AOD-05); it decides and stores everything that matters (AOD-13).
* The **client** is untrusted and never the source of truth (AOD-14): it renders, predicts its own edits and is corrected.
* **Single-player (proposal, owner decision OI-6):** the same server components run in the client's process as an embedded server instance, behind the same Editing/replication interfaces, so that there is exactly one authoritative path (Veloren does this behind a `singleplayer` feature, REF-32; its authority rules were not examined). A "local mode" with client-side authority is not allowed by AOD-14.
* **Today:** the spike runs a loopback TCP server and an in-process runtime in the Godot adapter; neither implements this separation (see section 8).

## 2. Modules and layers

The module grid is the one of the Integration & QA Pack (layers L0…L10, rule DEP-01); this document adds the modules the roadmap needs and places them. Existing crates (spike): `cube-core` (L0), `cube-processing` (L1), `cube-editing` (L2/L3 spike), `cube-persistence` (L8 spike), `cube-server`, `cube-godot-adapter` (L10 spike), `cube-worldgen` (not in AU-1.0).

| Module | AU-1.0 layer / status | Depends on | Owner |
|---|---|---|---|
| Geometry Core (assets, layers, bricks, commit primitive, RevisionToken, DirtyRegion, snapshots) | L0 | nothing | Core Agent |
| Processing (derived data, bounds, validation) | L1 | Core | Processing Agent |
| Editing API and contracts (service facade, operations, results, ports) | L2 | Core, Processing | Main Agent / Integration Owner |
| Orchestration, backends, tools, history, composite, persistence integration | L3-L8 | L2 ports, as in the grid | Agents A-H, Composite, Resolution |
| Networking adapter | L9 | L2 abstractions | **unassigned (O-3)** |
| Engine integration adapters (Godot) | L10 | the adapter surface (section 6) | **unassigned (O-5)** |
| **WorldGen** (generator, streaming/lifecycle, materialiser) | **not defined in AU-1.0. Proposal:** a module that depends on Core value types only (as `cube-worldgen -> cube-core` today); its writer role uses a Core system-commit port | Core (and, for lifecycle policy, server host) | **unassigned: owner decision** |
| Server host (sessions, ticks, interest, replication) | not defined in AU-1.0 | L2 abstractions, L8, WorldGen | **unassigned (O-3 networking overlaps)** |
| Shared domain types ("common") | **REF-02 Proposed**, not placed | Core | open |

Rules (existing, unchanged): imports follow the grid; the compile-time graph is acyclic; no engine type outside the adapter (checked by `dependency_rules.rs`). Proposed additions: `cube-worldgen` and the server host may not depend on Godot (already true for worldgen); only the adapter crate may depend on `godot`.

## 3. Data ownership

| Data | Authoritative owner | Stored where | Client copy |
|---|---|---|---|
| Voxel geometry (generated + user) | Server's Core asset | derived from seed + generator version + journal; snapshots as an optimisation | read-only cache, may be stale |
| Generated terrain baseline | WorldGen (pure function of seed, version, coordinate) | not stored for untouched chunks (AOD-17); cache only | regenerated locally only when the generator version and the chunk hash match the server's, else received |
| User changes | Server (Editing service) | versioned mutation journal | prediction until acknowledged |
| Revisions (edit revision, generation revision) | Server's Core | with the world | compared for staleness and conflicts |
| Operation ids, undo/redo history | Server (History module) | journal / history store | optimistic local ids reconciled by the server |
| Player, inventory, characters, accounts | Server persistence service | database (not decided) | display copy |
| Meshes, GPU buffers, UI state | Client | none | local only |
| Settings (graphics, controls) | Client | local config | n/a |

## 4. Write paths (who may change authoritative state)

Exactly three writers may commit to the authoritative asset:

1. **Editing service** - every user or designer operation (AOD-15): validated, id-tagged, revision-counted, recorded in the journal and in history.
2. **WorldGen Materialiser** - generated data only, through a narrow Core system-commit port (AOD-16); never recorded in history; carries the generation revision, not the edit revision (revision rule needs the B-6 decision).
3. **Persistence loader** - restoring a saved state at start (seed + version, then journal replay).

Everything else, including the renderer, the client runtime, tools and the adapter, reads snapshots or views. A client message is a *request* (an operation proposal); the server may reject or alter it.

Current non-compliant spike code (documented, experimental): the streamer and `edit_voxel`/`fill_box`/undo journal in `cube-worldgen` and `cube-godot-adapter` commit directly through the Core primitive; `cube-server` has no authentication; `world-state-v0` is a client-side file. None of it is a pattern to copy.

## 5. Interfaces

| Interface | Direction | Content | State |
|---|---|---|---|
| Editing API (`IEditingService`, E-series) | client request to server; server-internal | operations, results, errors with operation ids | Draft contracts; tool-level contracts missing (B-7) |
| WorldGen port | server host to WorldGen | generate/cancel/status for chunk coordinates; stable chunk hash | **new, not written** |
| System-commit (materialise/evict) | WorldGen to Core | generated chunk values | **new, not written** (touches P0, ACR-001 area) |
| Persistence ports (`IDeltaStore`, `ISnapshotStore`, P-series) | server-internal | world record, journal, snapshots | Draft; must be aligned with AOD-17 |
| Replication protocol | server to client (and requests back) | chunk baselines or seed + version, journal deltas, revisions, interest changes | **not designed**; spike TCP protocol is Draft and unauthenticated |
| Runtime-to-frontend adapter | client runtime to Godot | read views of chunks, dirty sections, mesh arrays, input intents | spike: `CubeRuntime`/`CubeWorld` (experimental) |

## 6. Frontend boundary (Godot replaceable, AOD-06/07)

* The adapter exposes **values and views**, never engine types, into the Rust side; it owns no authoritative geometry.
* GDScript is limited to UI, input forwarding and glue (mirroring Rust state into nodes). No game rules, persistence, generation, networking or process calls (AOD-07, AOD-12; the process-call ban is tested).
* A replacement frontend must be able to implement: render a chunk/section view, forward input intents, display UI, run on the target platforms. Nothing else may live in the frontend.
* Mesh construction (faces/greedy) currently sits on the Rust side of the adapter and the node management in GDScript; whether meshing belongs to the client runtime or the adapter is a Gate 4 decision.

## 7. Threading and scheduling (proposal)

* One writer for the authoritative asset (commit under the existing short lock rule); readers use snapshots.
* Generation, meshing and serialisation are background jobs with cancellation, bounded queues and per-job-class thread budgets (REF-05/06); results are applied on the owning thread and stale results are dropped.
* Mobile: no long main-thread work (budget in the roadmap), no process calls, low thread counts, memory bounded by view radius.

## 8. Mobile constraints that shape the boundaries (AOD-08/11)

Memory and thermal budgets must be measured on a physical device (none available yet); the view radius, chunk size, mesh format and background thread budget are parameters of the boundary, not constants. An emulator result never closes a mobile gate.

## 9. Enforcement

| Rule | Check today | Needed |
|---|---|---|
| No engine types in Core, allowed crate edges | `crates/cube-core/tests/dependency_rules.rs` | extend to WorldGen/server host crates |
| No child processes, no `getprop`, no test hooks in the runtime client, test scripts not in client packages | `no_child_processes.rs`, `scripts/check-apk-no-exec.py` | run both in CI (Gate 1) |
| Only the three writers commit | none | call-site audit test (Gate 2/3) |
| Deterministic generation | pinned hashes, reference LE hash implementation | cross-device hash test (Gate 3) |
| Frontend replaceability | adapter boundary document | adapter API snapshot test (Gate 4) |
| Server authority | none | authority tests (client cannot commit) (Gate 7) |

## 10. Open decisions for the owner

OI-6 embedded server for single-player; placement and ownership of WorldGen and the server host in the layer grid; the Core system-commit port and generation-revision rule (B-6, possible new ACR); transport and replication design; where meshing lives; the shared "common" crate (REF-02); owners for networking (O-3) and engine adapters (O-5).
