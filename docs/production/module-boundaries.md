# Cube Module Boundaries and Authoritative Dependency Graph v2 (proposal)

**Status:** **Proposed**; nothing approved. This document owns two things: (1) the module/ownership boundaries and (2) the **Authoritative Dependency Graph (ADG)**, the only place where dependencies between modules, tasks and gates are defined. Other documents refer to it by part letter (A-D). Stage names are the canonical `PS-n` of `production-architecture-plan.md` section 2.
**Owners:** where the owner of a module is not decided, the table says `UNASSIGNED - owner decision`; this revision assigns nobody. Named agents are those of the existing QA Pack grid (Core, Processing, Main/Integration Owner, A-I, Composite, Resolution).

## Part A: modules, owners and boundaries

### A.1 Workspace (crates are created when their stage starts)

| Crate | QA-Pack layer | Owner | Created in | Why a separate crate |
|---|---|---|---|---|
| `cube-core` | L0 | Core Agent | PS-1 | foundation, zero dependencies |
| `cube-processing` | L1 | Processing Agent | PS-2 | snapshots, graph, executor |
| `cube-editing-api` | L2 | Main Agent / Integration Owner | PS-3 | contracts and ports only (ACR-002) |
| `cube-orchestration` | L3 | Agent A | PS-4 | transactions, registry, `IEditingService` implementation |
| `cube-backend-voxel` | L4 | Agent B | PS-5 | voxel ops (+ providers, ownership pending O-1) |
| `cube-backend-sdf` | L4 | Agent C | PS-5 | SDF ops (+ providers) |
| `cube-backend-mesh` | L4 | Agent D | PS-5 | mesh ops (+ providers) |
| `cube-tool-sculpt` | L5 | Agent E | PS-6 | brushes |
| `cube-tool-boolean` | L5 | Agent F | PS-6 | booleans |
| `cube-resolution` | L5 | Resolution Agent | PS-9 | resampling, refinement |
| `cube-history` | L6 | Agent G | PS-7 | undo/redo |
| `cube-composite` | L7 | Composite Agent | PS-8 | composite operations |
| `cube-persistence` | L8 | Agent H | PS-10 | storage, journal, recovery |
| `cube-worldgen` | outside AU-1.0 (ACR-004 B) | WorldGen Agent for WGEN-01 only (technical assignment, DEC-091); the rest of the module stays UNASSIGNED (WorldGen) - owner decision | PS-11 | generator, service, Materialiser |
| `cube-net` | L9 | UNASSIGNED (Networking Adapter, O-3) - owner decision | PS-13 | transport contract, protocol, transports |
| `cube-server` | outside AU-1.0 | in-process local host only: Integration Owner (DEC-092); the networked server stays UNASSIGNED (Server Runtime) - owner decision | PS-13 | authoritative runtime, two hosts |
| `cube-app` | composition root of the first world (DEC-092) | Integration Owner | PS-12 | joins the local host and the client; edges: core, processing, orchestration, persistence, backend-voxel, client, server |
| `cube-client` | outside AU-1.0 | Client Runtime Agent for CLI-00..04 (technical assignment, DEC-091 and DEC-092); the rest stays UNASSIGNED (Client Runtime) - owner decision | PS-12 | engine-independent client runtime |
| `cube-godot-adapter` | L10 | Engine Adapter Agent for ADP-01..03 (technical assignment, DEC-092) | PS-12 | GDExtension surface |
| `cube-testkit` | test-time | Agent I | PS-0 | strategies, fakes, golden runner, FaultFs |
| `cube-integration-tests` | test-time (leaf) | Agent I | PS-1 | cross-crate suites, conformance suites, benches; a leaf crate so that it may depend on everything without cycles |
| `xtask` | tooling | Main Agent / Integration Owner | PS-0 | archcheck and package checks |

20 workspace members in total; at most four exist during PS-1 (`cube-core`, `cube-testkit`, `cube-integration-tests`, `xtask`). Not created: `cube-common`, `cube-meshing`, `cube-protocol`, collision/LOD crates.
**Rule R-DEVCYCLE:** `cube-testkit` may depend (normally) on `cube-core`, `cube-processing`, `cube-editing-api` only; crates may use it as a dev-dependency (a dev-dependency cycle is permitted by Cargo); `archcheck` evaluates **normal** dependencies only, and the ADG part B grid is about normal dependencies.

### A.2 Ownership matrix

| Module | Owner | Responsibilities | Provides | Depends on | Forbidden scope | Acceptance authority |
|---|---|---|---|---|---|---|
| Geometry Core | Core Agent | passive data model, commit gate, revisions, ids, error envelope, mutation boundary handles | `SharedAsset`, snapshots, handles, `OperationId`, `ActorKind`, `Error`, export views, content hash | nothing | any dependency, I/O, threads except sync primitives, clocks, randomness, engine types, formats, persistence, network, WorldGen logic, history, tools | Integration Owner (engineering); project owner (P0-alpha, PC-1 contracts) |
| Processing | Processing Agent | snapshots with apron, provider registry, dependency graph, executor, cancellation, caps, stale rejection | `Snapshot`, `Provider`, `Executor`, `CancellationToken`, `DerivedSlot` | core | editing, orchestration, backends, persistence, network, engine, async runtimes, any Core write | Integration Owner; project owner (PR-01…PR-08 after RG-2) |
| Editing API | Main Agent / Integration Owner | public types and ports | `IEditingService`, `IGeometryBackend`, `ITransactionObserver`, `IDeltaStore`, `ISnapshotStore`, `IOperationJournal`, `IBaseProvider`, `AuthorizationContext` | core, processing | behaviour; approving contracts | project owner (E-xx approval) |
| Editing Orchestration | Agent A | transactions, registry/dispatch, validation, single user commit path | `IEditingService` implementation | core, processing, editing-api | backend/tool/history/persistence implementations, network, second commit path | Integration Owner |
| Voxel / SDF / Mesh backends | Agents B / C / D | native ops on their representation (+ derived providers pending O-1) | `IGeometryBackend` implementations, providers | core, processing, editing-api | orchestration, tools, history, persistence, each other, direct commit | Integration Owner |
| Sculpting / Boolean | Agents E / F | high-level operations through capabilities | tool operations | core, processing, editing-api; `IEditingService` and capabilities as abstractions | direct commit, backend implementations, history, persistence | Integration Owner |
| Resolution | Resolution Agent | resampling, refinement, transitions | ops | as tools | direct commit, Core data changes | Integration Owner |
| History | Agent G | deltas, undo/redo as forward commits, checkpoints | `ITransactionObserver` implementation, history service | core, processing, editing-api; stores as abstractions | persistence implementation, orchestration implementation, direct commit, system ports | Integration Owner |
| Composite | Composite Agent | attach/detach, constraints, propagation | composite operations | core, processing, editing-api | owning the data model, auto-merging parts | Integration Owner |
| Persistence | Agent H | codec, manifest, snapshots, journal, compaction, recovery, migration, world record | store implementations, `recover()` | core, processing, editing-api | history, orchestration, backends, worldgen, formats before owner approval | Integration Owner; project owner (formats) |
| World Generation | **UNASSIGNED - owner decision** | pure generator, service, Materialiser, eviction | `generate()`, `WorldGenService`, `IBaseProvider` implementation | core, processing | direct commit, floats in world math, clocks, global state | Integration Owner |
| Networking Adapter | **UNASSIGNED (O-3) - owner decision** | transport contract, protocol/session types, transports | `Transport`, protocol types | core (ids/tokens), editing-api | server logic, persistence, engines | Integration Owner; project owner (transport choice) |
| Server Runtime | **UNASSIGNED - owner decision** | runtime, two hosts, sessions, authority, replication | `ServerRuntime` | all server-side crates, net | godot, UI, process calls, a second authority path, client-supplied file paths | Integration Owner |
| Client Runtime | **UNASSIGNED - owner decision** | cache, scheduler, LOD, intents, reconciliation | `ClientWorld` | core, processing, editing-api, backends (providers), net | godot, authority over world state | Integration Owner |
| Engine Adapter | **UNASSIGNED (O-5) - owner decision** | GDExtension surface, GDScript glue | engine classes | client, core read types, godot | algorithms, undo, persistence, world logic, OS.execute, panics across FFI | Integration Owner |
| Import/Export | **UNASSIGNED (O-4) - owner decision**; deferred | adapters for external formats | none yet | | | |
| Derived Data Providers | **owner decision O-1** (proposal in v1: the backend owners; not assigned) | greedy, faces, Surface Nets, collision, LOD | providers | processing | | |
| Modifier Evaluation | **owner decision O-2**; deferred | non-destructive modifier stack evaluation | none yet | | | |
| QA | Agent I | testkit, integration suites, benches, CI, device tooling | suites, harnesses | all (leaf) | production code in other crates; changing thresholds | Integration Owner |
| Architecture checks | Main Agent / Integration Owner | `xtask archcheck`, package checks | CI checks | | | project owner |

**GOV-03 record (2026-11-09, DEC-072):** the roles Core Agent, Integration Owner and Agent I (and Agents A-H, Processing, Composite, Resolution Agent for their stages) are filled by agent sessions working under the owner's delegation (DEC-001). The modules marked UNASSIGNED above are **deferred in writing**: World Generation until PS-11 (pure-generator spike work excepted), Networking Adapter and Server Runtime until PS-13 (SPK-NET), Client Runtime and Engine Adapter until PS-12, Import/Export (O-4) and Modifier Evaluation (O-2) with no stage before M3, Derived Data Providers (O-1) until PS-5 (default: the backend owners). No task of PS-1...PS-4 depends on any of them.

Candidate options for the unassigned owners are not proposed here; the owner decides (task GOV-03). Task rows owned by an unassigned module carry the marker `UNASSIGNED` and cannot start before GOV-03 names the owner (rule R-UNOWNED).

### A.3 Per-crate boundary summary

| Crate | Owned data | Threading | Error boundary | Revision semantics | Testing |
|---|---|---|---|---|---|
| core | all authoritative geometry, revisions, commit log (test-time) | lock-free readers via `Arc`, writers serialised by the gate | `CORE-xxxx` envelope | +1 per cell, layer, asset per commit; same for every `ActorKind` (R-A, Proposed) | contract, property, loom, golden hashes, compile-fail |
| processing | snapshots, derived slots, task state | thread pool per class, no async runtime | `PROC-xxxx`; cancelled/stale are results | consumes tokens, never produces; derived stamp = input stamps | loom, stress, cancellation, caps |
| editing-api | none | value-safe | public category set | results carry post tokens | mock-based contract tests, API snapshot |
| orchestration | live transactions | one writer per transaction | maps backend/Core errors | validates expected tokens before commit | conflict matrix, rollback, retry, no-bypass |
| backends | scratch only | providers pure over snapshots | `VOX-/SDF-/MSH-xxxx` | write sets with expected cell revisions | conformance suite, property tests, benches |
| tools / resolution | none | async on snapshots, validated at commit | per crate | stale snapshot gives a typed conflict | async safety, undo round trips |
| history | per-actor stacks | observer callbacks post-commit | `HIST-xxxx` | records pre/post tokens | undo/redo properties |
| composite | none | via transactions | `COMP-xxxx` | via commits | graph invariants |
| persistence | files, manifest | I/O off the main thread, single journal writer | `PERS-xxxx` | stores tokens/epochs; recovery under `Recovery` | FaultFs matrix, replay equality |
| worldgen | cache only | executor | `WGEN-xxxx` per chunk | `System` commits via the port | pinned hashes, cancellation, idempotence |
| net | none | carrier-specific; protocol core single-threaded state | `NET-xxxx` | carries tokens/ids | conformance, fuzz, loss simulation |
| server | authoritative world instance | one writer thread, executor pool | `SRV-xxxx` | server-order commits | shared suite on both hosts, authority, soak |
| client | cache | main + worker pool | `CLI-xxxx` | prediction epochs separate | reconciliation, budgets |
| adapter | none | engine threads | coded errors, `catch_unwind` | none | API snapshot, desktop smoke |

## Part B: authoritative module-level dependency graph

`D` = direct normal dependency allowed; `A` = abstraction through an L2 port only; blank = forbidden. Row = consumer. `archcheck` (XT-01, XT-02) compares the real crate graph with this grid.

| Consumer \ Provider | core | proc | api | orch | backends | tools | resol | hist | comp | pers | worldgen | net | server | client | adapter | testkit |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| core | - | | | | | | | | | | | | | | | |
| processing | D | - | | | | | | | | | | | | | | |
| editing-api | D | D | - | | | | | | | | | | | | | |
| orchestration | D | D | D | - | A | | | A | | A (journal port only) | | | | | | |
| backends | D | D | D | | - | | | | | | | | | | | |
| tools / resolution | D | D | D | A | A | - | | | | | | | | | | |
| history | D | D | D | A | | | | - | | A | | | | | | |
| composite | D | D | D | A | | | | | - | | | | | | | |
| persistence | D | D | D | | | | | | | - | | | | | | |
| worldgen | D | D | | | | | | | | | - | | | | | |
| net | D (ids, tokens) | | D | A | | | | | | | | - | | | | |
| server (composition root) | D | D | D | D | D | D | D | D | D | D | D | D | - | | | |
| client | D | D | D | | D (providers) | | | | | | | D | | - | | |
| godot-adapter | D | D | D | | | | | | | | | | | D | - | |
| testkit | D | D | D | | | | | | | | | | | | | - |
| integration-tests (leaf) | D | D | D | D | D | D | D | D | D | D | D | D | D | D | D | D |

Notes: (1) History and Persistence never import each other; both use L2 ports (closes B-9 once the XT-02 test exists). (2) Persistence does not depend on WorldGen: recovery obtains the base through `IBaseProvider` (API-04) implemented by WorldGen and wired in the server (SRV-01). (3) Persistence does not depend on Composite operations: PERS-09 uses only the Core composite data (CORE-14). (4) The Godot adapter depends on the client runtime and read types only. (5) `godot` appears only in the adapter. (6) WorldGen uses Core's public value types and the system-commit port; it has no L2 dependency.

### B.1 Interfaces: Provider, Consumer, Owner

| Interface | Provider (implements) | Consumers | Interface owner (defines/freezes) | Frozen by |
|---|---|---|---|---|
| Core read API, snapshots, `CellState` | core | processing, backends, client, worldgen | Core Agent | CORE-10 |
| `UserCommit` / `SystemCommit` / `RecoveryCommit` handles | core | orchestration / worldgen / persistence respectively | Core Agent | CORE-08, CORE-09 |
| `Provider`, `Snapshot`, `Executor`, `CancellationToken` | processing | backends, orchestration, worldgen, client | Processing Agent | PROC-01…04 (G2 for production) |
| `IEditingService`, operations, results | orchestration | tools, history, composite, server | Integration Owner | API-02 |
| `IGeometryBackend` + capabilities | backends | orchestration, tools | Integration Owner | API-02 |
| `ITransactionObserver` | history | orchestration | Integration Owner | API-02 |
| `IDeltaStore`, `ISnapshotStore` | persistence | history, server | Integration Owner | API-04 |
| `IOperationJournal` (append of committed user operations) | persistence | orchestration (ORCH-04), server wiring | Integration Owner | API-04 |
| `IBaseProvider` | worldgen | persistence recovery, server | Integration Owner | API-04 |
| `AuthorizationContext` | server (session) | orchestration | Integration Owner | API-03 |
| `Transport` | net | server, client | Networking owner (UNASSIGNED) | NET-01 |
| Protocol and session types | net | server, client | Networking owner (UNASSIGNED) | NET-02 |
| Client surface for the engine | client | adapter | Client owner (UNASSIGNED) | CLI-01…04 |

## Part C: the Authoritative Dependency Graph (tasks)

Semantics: **S (start-after)** - the dependency's public interface is frozen; the task may begin. **I (merge-after)** - the dependency is merged and accepted; the task may merge. A task merges after S union I. `Gn` are stage gates; `EXT-…` are external inputs. The data file `_build/data.py` is the source; the table is generated and machine-validated (part E).

| Task | Stage | Owner | Start-after (S) | Merge-after (I) |
|---|---|---|---|---|
| DEV-01 | PS-0 | Project owner | - | - |
| GOV-01 | PS-0 | Project owner | - | - |
| GOV-02 | PS-0 | Project owner | - | GOV-01 |
| GOV-03 | PS-0 | Project owner | - | - |
| GOV-04 | PS-0 | Main Agent / Integration Owner | - | GOV-01 |
| GOV-05 | PS-0 | Main Agent / Integration Owner | GOV-01 | GOV-01 |
| GOV-06 | PS-0 | Agent I (QA and Benchmarks) | - | GOV-04, GOV-05 |
| SPK-NET | PS-0 | UNASSIGNED (Networking Adapter, O-3) - owner decision | - | - |
| SPK-PERS | PS-0 | Agent H | - | - |
| TEST-01 | PS-0 | Agent I (QA and Benchmarks) | GOV-05 | GOV-06 |
| XT-01 | PS-0 | Main Agent / Integration Owner | GOV-05 | GOV-06 |
| CORE-01 | PS-1 | Core Agent | G0 | - |
| CORE-02 | PS-1 | Core Agent | G0 | - |
| CORE-03 | PS-1 | Core Agent | CORE-01, CORE-02 | - |
| CORE-04 | PS-1 | Core Agent | CORE-01 | - |
| CORE-05 | PS-1 | Core Agent | CORE-01, CORE-02 | - |
| CORE-06 | PS-1 | Core Agent | CORE-03, CORE-05, CORE-07 | - |
| CORE-07 | PS-1 | Core Agent | G0 | - |
| CORE-08 | PS-1 | Core Agent | CORE-06, CORE-07 | - |
| CORE-09 | PS-1 | Core Agent | CORE-08 | - |
| CORE-10 | PS-1 | Core Agent | CORE-01, CORE-02, CORE-03, CORE-05, CORE-06, CORE-07, CORE-04 | CORE-01, CORE-02, CORE-03, CORE-05, CORE-06, CORE-07, CORE-04 |
| CORE-11 | PS-1 | Core Agent | CORE-03, CORE-05 | CORE-06 |
| INT-01 | PS-1 | Main Agent / Integration Owner | - | CORE-01, CORE-02, CORE-03, CORE-04, CORE-05, CORE-06, CORE-07, CORE-08, CORE-09, CORE-10, CORE-11, TEST-02, TEST-05, TEST-06, XT-02 |
| TEST-02 | PS-1 | Agent I (QA and Benchmarks) | TEST-01 | CORE-11 |
| TEST-05 | PS-1 | Agent I (QA and Benchmarks) | CORE-06 | CORE-06 |
| TEST-06 | PS-1 | Agent I (QA and Benchmarks) | CORE-08 | CORE-09, XT-02 |
| XT-02 | PS-1 | Main Agent / Integration Owner | CORE-08 | CORE-09 |
| CORE-15 | PS-2 | Core Agent | CORE-10 | CORE-10, GOV-16, G1 |
| CORE-16 | PS-2 | Core Agent | CORE-10 | CORE-10, G1 |
| GOV-07 | PS-2 | Project owner | EXT-GPS | EXT-GPS, PROC-02, PROC-04 |
| GOV-16 | PS-2 | Project owner | - | - |
| GOV-24 | PS-2 | Project owner | - | - |
| PROC-01 | PS-2 | Processing Agent | CORE-10 | CORE-06, G1 |
| PROC-02 | PS-2 | Processing Agent | PROC-01 | PROC-01, G1 |
| PROC-03 | PS-2 | Processing Agent | CORE-10 | G1 |
| PROC-04 | PS-2 | Processing Agent | PROC-03 | PROC-03, G1 |
| PROC-05 | PS-2 | Processing Agent | PROC-01 | PROC-01, G1 |
| PROC-06 | PS-2 | Processing Agent | PROC-01, CORE-15 | PROC-01, GOV-24, G1 |
| TEST-08 | PS-2 | Agent I (QA and Benchmarks) | PROC-03 | PROC-05, G1 |
| API-01 | PS-3 | Main Agent / Integration Owner | CORE-10 | CORE-07, G1 |
| API-02 | PS-3 | Main Agent / Integration Owner | API-01, PROC-01, PROC-04 | API-01, G2, G1 |
| API-03 | PS-3 | Main Agent / Integration Owner | API-01 | API-01, G1 |
| API-04 | PS-3 | Main Agent / Integration Owner | API-01, CORE-09 | API-01, G1 |
| API-05 | PS-3 | Main Agent / Integration Owner | API-01 | API-01, G1 |
| GOV-09 | PS-3 | Project owner | API-02, API-03, API-04 | API-02, API-03, API-04 |
| ORCH-01 | PS-4 | Agent A | API-02 | G2, G3, G1 |
| ORCH-02 | PS-4 | Agent A | API-02 | G2, G3, G1 |
| ORCH-03 | PS-4 | Agent A | CORE-10, API-01 | G2, G3, G1 |
| ORCH-04 | PS-4 | Agent A | PROC-02, API-02, API-04 | G2, G3, G1 |
| ORCH-05 | PS-4 | Agent A | API-03, CORE-08 | G2, G3, XT-02, G1 |
| TEST-03a | PS-4 | Agent I (QA and Benchmarks) | API-02 | API-02, G1 |
| TEST-10 | PS-4 | Agent I (QA and Benchmarks) | CORE-08 | ORCH-05, XT-02, G1 |
| CORE-12 | PS-5 | Core Agent | CORE-10 | CORE-10, EXT-VA-RG1, GOV-17, G1 |
| GOV-08 | PS-5 | Project owner | EXT-VA-RG1 | EXT-VA-RG1 |
| GOV-17 | PS-5 | Project owner | GOV-08 | GOV-08 |
| MSH-01 | PS-5 | Agent D | API-02, CORE-03, TEST-03a | G2, G3, G1 |
| MSH-02 | PS-5 | Agent D | MSH-01 | MSH-01, G1 |
| MSH-03 | PS-5 | Agent D | PROC-02 | G2, G1 |
| SDF-01 | PS-5 | Agent C | API-02, TEST-03a | G2, G3, G1 |
| SDF-02 | PS-5 | Agent C | PROC-02, SDF-01 | G2, G1 |
| SDF-03 | PS-5 | Agent C | SDF-01 | SDF-01, G1 |
| VOX-01 | PS-5 | Agent B | API-02, TEST-03a | G2, G3, G1 |
| VOX-02 | PS-5 | Agent B | PROC-01, PROC-02 | G2, G1 |
| VOX-03 | PS-5 | Agent B | VOX-02 | VOX-02, G1 |
| VOX-04 | PS-5 | Agent B | VOX-03 | VOX-03, G1 |
| VOX-05 | PS-5 | Agent B | PROC-02 | G2, G1 |
| BOO-01 | PS-6 | Agent F | API-01 | API-01, GOV-20, G1 |
| BOO-02 | PS-6 | Agent F | ORCH-02, BOO-01 | G4, G5, G1 |
| BOO-03 | PS-6 | Agent F | BOO-02 | BOO-02, G1 |
| GOV-20 | PS-6 | Project owner | - | - |
| SCU-01 | PS-6 | Agent E | API-01 | API-01, G1 |
| SCU-02 | PS-6 | Agent E | ORCH-02, VOX-01, SDF-01 | G4, G5, GOV-20, G1 |
| SCU-03 | PS-6 | Agent E | SCU-02 | SCU-02, G1 |
| GOV-25 | PS-7 | Project owner | - | - |
| HIST-01 | PS-7 | Agent G | API-02, API-04 | G4, G1 |
| HIST-02 | PS-7 | Agent G | ORCH-03, HIST-01 | G4, GOV-25, G1 |
| HIST-03 | PS-7 | Agent G | API-04 | HIST-01, G1 |
| HIST-04 | PS-7 | Agent G | HIST-02 | HIST-02, G1 |
| COMP-01 | PS-8 | Composite Agent | ORCH-01, CORE-14 | G4, GOV-23, G1 |
| COMP-02 | PS-8 | Composite Agent | COMP-01 | COMP-01, G1 |
| COMP-03 | PS-8 | Composite Agent | CORE-04, COMP-01 | COMP-01, G1 |
| CORE-14 | PS-8 | Core Agent | CORE-10 | CORE-10, GOV-19, G1 |
| GOV-19 | PS-8 | Project owner | - | - |
| GOV-23 | PS-8 | Project owner | - | - |
| CORE-13 | PS-9 | Core Agent | CORE-10 | CORE-10, EXT-VA-RG1, GOV-18, G1 |
| GOV-18 | PS-9 | Project owner | GOV-08 | GOV-08 |
| RES-01 | PS-9 | Resolution Agent | CORE-13, VOX-01 | G4, G5, GOV-23, G1 |
| RES-02 | PS-9 | Resolution Agent | CORE-13 | G4, G1 |
| RES-03 | PS-9 | Resolution Agent | VOX-03, RES-02 | RES-02, G1 |
| GOV-10 | PS-10 | Project owner | SPK-PERS, PERS-05, EXT-VA-RG1 | SPK-PERS, PERS-05, EXT-VA-RG1 |
| GOV-11 | PS-10 | Project owner | - | - |
| ITG-01 | PS-10 | Agent I (QA and Benchmarks) | PERS-04, ORCH-04 | G4, PERS-04, G1 |
| PERS-01 | PS-10 | Agent H | CORE-11, API-04 | CORE-11, G1 |
| PERS-02 | PS-10 | Agent H | PERS-01, SPK-PERS | PERS-01, G1 |
| PERS-03 | PS-10 | Agent H | PERS-01, PERS-02, API-04 | PERS-02, G1 |
| PERS-04 | PS-10 | Agent H | PERS-01, API-04, CORE-07, CORE-05 | PERS-01, G1 |
| PERS-05 | PS-10 | Agent H | PERS-02, PERS-03, PERS-04, SPK-PERS, TEST-04 | PERS-03, PERS-04, G1 |
| PERS-06 | PS-10 | Agent H | PERS-05, CORE-09, API-04, GOV-11 | PERS-05, CORE-09, G1 |
| PERS-07 | PS-10 | Agent H | PERS-01 | PERS-01, G1 |
| PERS-08 | PS-10 | Agent H | PERS-01, CORE-07, CORE-16 | PERS-01, CORE-16, G1 |
| PERS-09 | PS-10 | Agent H | PERS-01, CORE-14 | PERS-01, CORE-14, G1 |
| QA-P2 | PS-10 | Agent I (QA and Benchmarks) | PERS-05, TEST-04 | PERS-06, G1 |
| TEST-04 | PS-10 | Agent I (QA and Benchmarks) | TEST-01 | TEST-01, G1 |
| GOV-21 | PS-11 | Project owner | - | - |
| ITG-02 | PS-11 | Agent I (QA and Benchmarks) | WGEN-04, PERS-06 | G10, WGEN-04, G1 |
| LEG-01 | PS-11 | UNASSIGNED (WorldGen) - owner decision | WGEN-03, GOV-11 | WGEN-04, PERS-08, ITG-02, G1 |
| TEST-09 | PS-11 | Agent I (QA and Benchmarks) | TEST-02, WGEN-01 | WGEN-01, G1 |
| WGEN-01 | PS-11 | WorldGen Agent (technical assignment, DEC-091) | CORE-10, CORE-07, CORE-16 | CORE-10, CORE-16, G1 |
| WGEN-02 | PS-11 | UNASSIGNED (WorldGen) - owner decision | WGEN-01, PROC-03, PROC-04 | PROC-04, GOV-21, G1 |
| WGEN-03 | PS-11 | UNASSIGNED (WorldGen) - owner decision | WGEN-01, CORE-09, GOV-11 | CORE-09, GOV-11, GOV-21, G1 |
| WGEN-04 | PS-11 | UNASSIGNED (WorldGen) - owner decision | WGEN-02, WGEN-03, API-04 | WGEN-03, G1 |
| WGEN-05 | PS-11 | UNASSIGNED (WorldGen) - owner decision | WGEN-01, TEST-02 | WGEN-01, G1 |
| ADP-01 | PS-12 | Engine Adapter Agent (technical assignment, DEC-092) | CLI-01 | CLI-01, CLI-02, CORE-12, G1 |
| ADP-02 | PS-12 | Engine Adapter Agent (technical assignment, DEC-092) | ADP-01 | ADP-01, G1 |
| ADP-03 | PS-12 | Engine Adapter Agent (technical assignment, DEC-092) | ADP-01 | ADP-01, G1 |
| CLI-00 | PS-12 | Client Runtime Agent (technical assignment, DEC-091) | CORE-10, API-01 | G1 |
| CLI-01 | PS-12 | Client Runtime Agent (technical assignment, DEC-091, DEC-092) | CLI-00, CORE-06 | CORE-06, G1 |
| CLI-02 | PS-12 | Client Runtime Agent (technical assignment, DEC-091, DEC-092) | CLI-00, PROC-03, VOX-02 | VOX-03, PROC-05, G1 |
| CLI-03 | PS-12 | Client Runtime Agent (technical assignment, DEC-091, DEC-092) | VOX-05 | VOX-05, G1 |
| CLI-04 | PS-12 | Client Runtime Agent (technical assignment, DEC-091, DEC-092) | CLI-00, API-01 | API-01, G1 |
| LEG-02 | PS-12 | UNASSIGNED (Engine Adapter, O-5) - owner decision | ADP-01, CLI-01, GOV-11 | ORCH-05, HIST-02, CLI-01, ADP-01, TEST-10, G1 |
| RND-01 | PS-12 | Agent I (QA and Benchmarks) | VOX-03 | VOX-03, G1 |
| CLI-05 | PS-13 | UNASSIGNED (Client Runtime) - owner decision | CLI-01, NET-02 | NET-03, SRV-04, CLI-01, G1 |
| GOV-12 | PS-13 | Project owner | SPK-NET | SPK-NET |
| GOV-22 | PS-13 | Project owner | - | - |
| NET-01 | PS-13 | UNASSIGNED (Networking Adapter, O-3) - owner decision | API-02 | API-02, G1 |
| NET-02 | PS-13 | UNASSIGNED (Networking Adapter, O-3) - owner decision | NET-01, CORE-07 | NET-01, GOV-22, G1 |
| NET-03 | PS-13 | UNASSIGNED (Networking Adapter, O-3) - owner decision | NET-01 | NET-01, G1 |
| NET-04 | PS-13 | UNASSIGNED (Networking Adapter, O-3) - owner decision | NET-02, GOV-12 | NET-02, GOV-12, G1 |
| SRV-01 | PS-13 | UNASSIGNED (Server Runtime) - owner decision | CORE-10, API-02, NET-01 | G4, G10, WGEN-03, NET-03, PERS-06, GOV-22, G1 |
| SRV-02 | PS-13 | UNASSIGNED (Server Runtime) - owner decision | NET-02, API-03, SRV-01 | NET-02, SRV-01, G1 |
| SRV-03 | PS-13 | UNASSIGNED (Server Runtime) - owner decision | SRV-01 | ORCH-05, SRV-01, G1 |
| SRV-04 | PS-13 | UNASSIGNED (Server Runtime) - owner decision | SRV-03, WGEN-04, NET-02 | SRV-03, WGEN-04, G1 |
| SRV-05 | PS-13 | UNASSIGNED (Server Runtime) - owner decision | SRV-04, ORCH-03 | SRV-04, G1 |
| SRV-06 | PS-13 | UNASSIGNED (Server Runtime) - owner decision | SRV-02 | SRV-02, G1 |
| SRV-07 | PS-13 | UNASSIGNED (Server Runtime) - owner decision | HIST-04, SRV-02 | HIST-04, SRV-02, G1 |
| SRV-08 | PS-13 | UNASSIGNED (Server Runtime) - owner decision | SRV-01, NET-04 | SRV-01, NET-04, G1 |
| TEST-03b | PS-13 | Agent I (QA and Benchmarks) | NET-01 | NET-03, NET-04, G1 |
| TEST-07 | PS-13 | Agent I (QA and Benchmarks) | SRV-02 | SRV-06, G1 |
| DG-1 | PS-14 | Agent I (QA and Benchmarks) | VOX-03, CLI-02, ADP-01, EXT-DEV, QA-06 | QA-06, EXT-DEV, G1 |
| DG-2 | PS-14 | Agent I (QA and Benchmarks) | WGEN-05, PERS-05, EXT-DEV, QA-06 | QA-06, EXT-DEV, G1 |
| DG-3 | PS-14 | Agent I (QA and Benchmarks) | G13, EXT-DEV, QA-06 | G13, EXT-DEV, QA-06, G1 |
| GOV-13 | PS-14 | Project owner | DG-1 | DG-1 |
| GOV-15 | PS-14 | Project owner | GOV-08 | GOV-08 |
| QA-02 | PS-14 | Agent I (QA and Benchmarks) | TEST-01 | TEST-01, G1 |
| QA-04 | PS-14 | Agent I (QA and Benchmarks) | G13 | G13, G1, G5, G6, G7, G8, G9, G11, G12 |
| QA-06 | PS-14 | Agent I (QA and Benchmarks) | ADP-01 | ADP-01, G1 |

### C.1 External inputs

| Node | Meaning |
|---|---|
| EXT-GPS | Geometry Processing Specification supplied to the repository (blocker B-1) |
| EXT-VA-RG1 | Voxel Architecture v1.0 supplied and revalidation gate RG-1 passed (blocker B-1) |
| EXT-DEV | Physical Android ARM64 device(s) and, for iOS, macOS/Xcode available (NB-1, NB-2) |

## Part D: gates, start conditions, integration conditions, closing conditions

A stage **closes** when its gate node `Gn` is satisfied: every member task is merged and accepted and the gate's extra conditions hold. Membership is generated:

| Gate | Stage | Member tasks (all must be merged and accepted) |
|---|---|---|
| G0 | PS-0 | GOV-01, GOV-02, GOV-03, GOV-04, GOV-05, GOV-06, TEST-01, XT-01 |
| G1 | PS-1 | CORE-01, CORE-02, CORE-03, CORE-04, CORE-05, CORE-06, CORE-07, CORE-08, CORE-09, CORE-10, CORE-11, TEST-02, TEST-05, XT-02, TEST-06, INT-01 |
| G2 | PS-2 | CORE-15, PROC-01, PROC-02, PROC-03, PROC-04, PROC-05, PROC-06, TEST-08, GOV-07, CORE-16, GOV-16, GOV-24 |
| G3 | PS-3 | API-01, API-02, API-03, API-04, API-05, GOV-09 |
| G4 | PS-4 | ORCH-01, ORCH-02, ORCH-03, ORCH-04, ORCH-05, TEST-03a, TEST-10 |
| G5 | PS-5 | CORE-12, VOX-01, VOX-02, VOX-03, VOX-04, VOX-05, SDF-01, SDF-02, SDF-03, MSH-01, MSH-02, MSH-03, GOV-08, GOV-17 |
| G6 | PS-6 | SCU-01, SCU-02, SCU-03, BOO-01, BOO-02, BOO-03, GOV-20 |
| G7 | PS-7 | HIST-01, HIST-02, HIST-03, HIST-04, GOV-25 |
| G8 | PS-8 | CORE-14, COMP-01, COMP-02, COMP-03, GOV-19, GOV-23 |
| G9 | PS-9 | CORE-13, RES-01, RES-02, RES-03, GOV-18 |
| G10 | PS-10 | TEST-04, PERS-01, PERS-02, PERS-03, PERS-04, PERS-05, PERS-06, PERS-07, PERS-08, PERS-09, ITG-01, QA-P2, GOV-10, GOV-11 |
| G11 | PS-11 | WGEN-01, TEST-09, WGEN-02, WGEN-03, WGEN-04, WGEN-05, ITG-02, LEG-01, GOV-21 |
| G12 | PS-12 | CLI-00, CLI-01, CLI-02, CLI-03, CLI-04, ADP-01, ADP-02, ADP-03, LEG-02, RND-01 |
| G13 | PS-13 | NET-01, NET-02, NET-03, GOV-12, NET-04, SRV-01, SRV-02, SRV-03, SRV-04, SRV-05, SRV-06, SRV-07, CLI-05, TEST-03b, TEST-07, GOV-22, SRV-08 |
| G14 | PS-14 | QA-02, QA-04, QA-06, DG-1, GOV-13, DG-2, DG-3, GOV-15 |

Generated from the task edges (so that it cannot diverge from part C). "Outside the stage" means a dependency that belongs to another stage, gate or external input. Hand-written text is limited to the extra closing conditions.

| Stage | Start-after (engineering tasks) | Merge-after (engineering tasks) | Inputs of the owner-decision tasks GOV-xx (they delay only those tasks and whatever lists them in the other columns) | Closing condition |
|---|---|---|---|---|
| PS-0 | - | - | - | G0: all 8 member tasks merged and accepted; CI specification Owner-Approved and, as a separate condition, CI running (workflow-run URL, required-status-check settings); registry entries owner-confirmed |
| PS-1 | G0, TEST-01 | - | - | G1: all 16 member tasks merged and accepted; P0-alpha Owner-Approved (done in PS-0) and recorded Implemented by INT-01; PC-1 stays under Provisional approval; no contract Frozen; CI-host evidence only - no device, no emulator condition |
| PS-2 | CORE-10 | CORE-10, CORE-06, G1 | EXT-GPS | G2: all 12 member tasks merged and accepted; includes GOV-07: PR-01…PR-08 Owner-Approved after RG-2; development may precede it, production merges that depend on Processing may not |
| PS-3 | CORE-10, PROC-01, PROC-04, CORE-09 | CORE-07, G2, G1 | - | G3: all 6 member tasks merged and accepted; E-02…E-12 and the storage ports Owner-Approved (GOV-09); API-02 merged only after G2 |
| PS-4 | API-02, CORE-10, API-01, PROC-02, API-04, API-03, CORE-08 | G2, G3, XT-02, API-02, G1 | - | G4: all 7 member tasks merged and accepted; Orchestration merged only after G2 and G3 |
| PS-5 | CORE-10, API-02, TEST-03a, PROC-01, PROC-02, CORE-03 | CORE-10, EXT-VA-RG1, G2, G3, G1 | EXT-VA-RG1 | G5: all 14 member tasks merged and accepted; backend conformance only; renderer validation is DG-1 and the renderer default is GOV-13, both outside this stage |
| PS-6 | API-01, ORCH-02, VOX-01, SDF-01 | API-01, G4, G5, G1 | - | G6: all 7 member tasks merged and accepted; tool-level contracts Owner-Approved (GOV-20) |
| PS-7 | API-02, API-04, ORCH-03 | G4, G1 | - | G7: all 5 member tasks merged and accepted; OD-R5 decided (GOV-25); system and recovery commits absent from history (tests) |
| PS-8 | CORE-10, ORCH-01, CORE-04 | CORE-10, G4, G1 | - | G8: all 6 member tasks merged and accepted; C-13…C-15 and composite-operation contracts Owner-Approved |
| PS-9 | CORE-10, VOX-01, VOX-03 | CORE-10, EXT-VA-RG1, G4, G5, GOV-23, G1 | GOV-08 | G9: all 5 member tasks merged and accepted; C-11, C-12 Owner-Approved after RG-1 (GOV-18) |
| PS-10 | TEST-01, CORE-11, API-04, SPK-PERS, CORE-07, CORE-05, CORE-09, CORE-16, CORE-14, ORCH-04 | TEST-01, CORE-11, CORE-09, CORE-16, CORE-14, G4, G1 | SPK-PERS, EXT-VA-RG1 | G10: all 14 member tasks merged and accepted; persistence format Owner-Approved (GOV-10, needs RG-1); desktop crash matrix green; device durability is DG-2, not a condition |
| PS-11 | CORE-10, CORE-07, CORE-16, TEST-02, PROC-03, PROC-04, CORE-09, GOV-11, API-04, PERS-06 | CORE-10, CORE-16, PROC-04, CORE-09, GOV-11, G10, PERS-08, G1 | - | G11: all 9 member tasks merged and accepted; WG-01…WG-10 Owner-Approved (GOV-21); end-to-end test ITG-02; no direct commit left in worldgen (archcheck) |
| PS-12 | CORE-10, API-01, CORE-06, PROC-03, VOX-02, VOX-05, GOV-11, VOX-03 | CORE-06, VOX-03, PROC-05, VOX-05, API-01, CORE-12, ORCH-05, HIST-02, TEST-10, G1 | - | G12: all 10 member tasks merged and accepted; desktop evidence only; no write path or history in the adapter (archcheck, TEST-10) |
| PS-13 | API-02, CORE-07, CORE-10, API-03, WGEN-04, ORCH-03, HIST-04, CLI-01 | API-02, G4, G10, WGEN-03, PERS-06, ORCH-05, WGEN-04, HIST-04, CLI-01, G1 | SPK-NET | G13: all 17 member tasks merged and accepted; protocol, transport and server contracts Owner-Approved (GOV-12, GOV-22); conformance on both transports and both hosts |
| PS-14 | TEST-01, G13, ADP-01, VOX-03, CLI-02, EXT-DEV, WGEN-05, PERS-05 | TEST-01, G13, ADP-01, EXT-DEV, G1, G5, G6, G7, G8, G9, G11, G12 | GOV-08 | G14: all 8 member tasks merged and accepted; DG-1…DG-3 reports; renderer default recorded (GOV-13); PC-1 frozen only through GOV-15 after GOV-08 |

## Part E: validation of this graph

* Tasks: 147; gates: 15; external inputs: 3; tasks outside any gate (spikes and device acquisition): SPK-PERS, SPK-NET, DEV-01.
* Method: `_build/validate.py` loads the single data file and checks ids, owners, dependency existence, stage monotonicity, gate membership, acyclicity (Kahn) on the union of S and I edges plus gate membership edges, and one owner per crate directory.

| Check | Result |
|---|---|
| graph has no unknown dependency, no cycle, no stage inversion, one owner per crate | PASS |
| G2 (Processing production gate) is an ancestor of every ORCH task | PASS |
| G2 is an ancestor of VOX-01, SDF-01, MSH-01 and SRV-01 | PASS |
| EXT-GPS is an ancestor of G2 (specification gates Processing production) | PASS |
| PS-1 gate G1 does not depend on any device, emulator, WorldGen, Persistence, Processing or networking node | PASS |
| G2 does not depend on EXT-DEV | PASS |
| G5 (backend conformance) does not depend on DG-1 or GOV-13 (renderer) | PASS |
| G10 does not depend on History (HIST-*) or WorldGen (WGEN-*) or devices | PASS |
| PERS-06 depends on PERS-04, PERS-05, CORE-09, API-04, GOV-11 (directly or via S/I) | PASS |
| PERS-04 depends on API-04, CORE-07, CORE-05, PERS-01 | PASS |
| WGEN-02 depends on WGEN-01, PROC-03, PROC-04 and not on Persistence | PASS |
| WGEN-03 depends on CORE-09 and GOV-11 | PASS |
| NET-02 (protocol types) does not depend on any SRV task; SRV-02 depends on NET-02 | PASS |
| CORE-11 is in gate G1 and is a dependency of PERS-01 | PASS |
| PERS-09 depends on CORE-14 and does not depend on COMP-* or HIST-* | PASS |
| every unassigned-owner task is outside PS-0..PS-10 except the networking spike SPK-NET (R-UNOWNED never blocks PS-1) | PASS |
| CORE-10 (API freeze) depends on CORE-01..CORE-07 including CORE-04 | PASS |
| G1 members are owned only by Core Agent, Agent I, Integration Owner (no WorldGen, Persistence, History, Processing owner) | PASS |
| G1 has no ancestor in WorldGen, Persistence, History, Processing, Editing API, Orchestration, backends, network, client, adapter or device tasks | PASS |
| G1 contains CORE-11, TEST-02, TEST-05, TEST-06, XT-02, INT-01 and CORE-07..09 | PASS |
| WorldSeed/GeneratorVersion task CORE-16 is outside PS-1 (not in G1) | PASS |
| API-02 (Editing API ports) merges after G2 (Processing production gate) | PASS |
| G3 depends on G2 | PASS |
| no device/emulator input (EXT-DEV) is an ancestor of any gate G0..G13 | PASS |
| GOV-15 (Freeze of PC-1) depends on GOV-08 and EXT-VA-RG1 | PASS |
| CORE-12, CORE-13 depend on the RG-1 revalidation record GOV-08 | PASS |
| every deferred Core contract task has an owner-approval ancestor (CORE-12:GOV-17, CORE-13:GOV-18, CORE-14:GOV-19, CORE-15:GOV-16) | PASS |
| PROC-06 depends on GOV-24 (OD-R4) and HIST-02 on GOV-25 (OD-R5) | PASS |
| tool/composite/resolution/WorldGen/server merges depend on their contract approvals (GOV-20, GOV-23, GOV-21, GOV-22) | PASS |
| ORCH-04 depends on API-04 (journal port); PERS-04 does not depend on API-02 or on G2 | PASS |
| dedicated host SRV-08 depends on NET-04; transport suite TEST-03b depends on NET-03 and NET-04 | PASS |
| CI and archcheck acceptance require registration as required status checks | PASS |
| GOV-04 and GOV-05 depend on the owner decision GOV-01 | PASS |
| every gate member task belongs to the gate stage and no task is in two gates | PASS |

* Longest dependency chain to G14 (21 task nodes): `GOV-01 -> GOV-04 -> GOV-06 -> TEST-01 -> G0 -> CORE-01 -> CORE-03 -> CORE-06 -> CORE-10 -> PROC-01 -> PROC-05 -> TEST-08 -> G2 -> API-02 -> GOV-09 -> G3 -> ORCH-05 -> TEST-10 -> G4 -> ITG-01 -> G10 -> SRV-01 -> SRV-03 -> SRV-04 -> SRV-05 -> G13 -> QA-04 -> G14`
