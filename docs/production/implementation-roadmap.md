# Cube Implementation Roadmap v2 (proposal)

**Status:** **Proposed**; no stage is authorised to start by this document, and no stage is complete because documents exist (PS-0 and PS-1 are **not** treated as complete). Stage names are the canonical `PS-n` ([`production-architecture-plan.md`](production-architecture-plan.md) section 2). **Dependencies, start conditions, integration conditions and closing conditions are defined only in the ADG ([`module-boundaries.md`](module-boundaries.md), parts C and D); this document adds purpose, artifacts, tests and acceptance per stage and never restates an edge.** Task ids and owners are generated from the same data as the ADG.
**Rule:** integration is per module and only after the module's acceptance criteria and the stage gate conditions; a stage closes when its gate `Gn` is satisfied; contract statuses change only by the owner's decision-registry entry.

## 1. Stage overview

| Stage | Name | Gate | Main owners |
|---|---|---|---|
| PS-0 | Governance and Architecture Freeze | G0 | project owner, Integration Owner, Agent I |
| PS-1 | Geometry Core Foundation (P0-alpha) | G1 | Core Agent |
| PS-2 | Processing Foundation | G2 | Processing Agent |
| PS-3 | Editing API and Contracts | G3 | Integration Owner |
| PS-4 | Editing Orchestration | G4 | Agent A |
| PS-5 | Geometry Backends and Derived Providers | G5 | Agents B, C, D |
| PS-6 | Tools: Sculpting and Boolean | G6 | Agents E, F |
| PS-7 | History | G7 | Agent G |
| PS-8 | Composite Geometry | G8 | Composite Agent |
| PS-9 | Resampling and Resolution Refinement | G9 | Resolution Agent |
| PS-10 | Persistence | G10 | Agent H |
| PS-11 | World Generation | G11 | UNASSIGNED (WorldGen) |
| PS-12 | Client Runtime and Engine Adapter | G12 | UNASSIGNED (Client Runtime, Engine Adapter) |
| PS-13 | Server Runtime and Networking | G13 | UNASSIGNED (Server Runtime, Networking Adapter) |
| PS-14 | Full QA and Release Validation | G14 | Agent I, Integration Owner |

Critical paths: see [`production-architecture-plan.md`](production-architecture-plan.md) section 12 (computed chain and the owner-blocked chain through G2).

## 2. Cross-cutting validation rules (fixes to the v1 QA coupling)

1. **Platform-independent code is verified on the CI host.** No Android emulator or device run is a condition of PS-1, PS-2, PS-3, PS-4 or PS-10 exits.
2. **Device validation is a separate track:** DG-1 (renderer: faces vs greedy), DG-2 (mobile determinism and durability), DG-3 (mobile release). They need `EXT-DEV`, do not block PS-1…PS-13, and gate only the decisions and releases named in their rows (GOV-13 renderer default, PS-14).
3. **Backend conformance (G5) is independent of the renderer choice.** The renderer default is decided by GOV-13 after DG-1. `faces` stays the default and `greedy` experimental until then (AOD-10).
4. **CI specification (GOV-05) and CI implementation (GOV-06) are separate deliverables** with separate evidence.
5. **Mandatory tests by kind** (details in [`integration-and-qa-plan.md`](integration-and-qa-plan.md)): concurrency (loom and stress) from PS-1, fault injection from PS-10, negative tests for forbidden mutation paths from PS-1 (TEST-06), property-based tests per crate, golden hashes and determinism on the CI host from PS-1 and across devices only in DG-2.
6. Golden hashes and determinism claims in a stage exit are limited to the CI host unless the row is a device gate.

## 3. Stages

### PS-0: Governance and Architecture Freeze
* **Gate:** G0 (conditions: ADG part D). **Dependencies and start/merge conditions:** ADG part C, not repeated here.
* **Inputs:** ADG adopted; owner availability. Inputs per task: ADG part C.
* **Tasks (id, owner):** GOV-01 (Project owner); GOV-02 (Project owner); GOV-03 (Project owner); GOV-04 (Main Agent / Integration Owner); GOV-05 (Main Agent / Integration Owner); GOV-06 (Agent I (QA and Benchmarks)); TEST-01 (Agent I (QA and Benchmarks)); XT-01 (Main Agent / Integration Owner); SPK-PERS (Agent H); SPK-NET (UNASSIGNED (Networking Adapter, O-3)); DEV-01 (Project owner)
* **Output artifacts:** decision-registry entries (owner-confirmed); Owner-Approved P0-alpha; registered Provisional PC-1; ownership table for the modules needed next; docs-only baseline on master; CI specification; running CI skeleton; testkit skeleton; archcheck v0; spike reports (persistence durability, transport).
* **Contracts needed:** P0-alpha (C-01, C-02, C-03, C-04, C-06, C-16, C-17, C-18, C-19) reviewed; PC-1 (C-07, C-08, C-09) registered.
* **Tests:** CI self-run on seeded failures; archcheck seeded violations.
* **Acceptance criteria:** Every PS-1 entry criterion of the architecture plan section 14 has evidence. No contract is changed by an agent.
* **Exit gate:** G0 closes when every member is merged and accepted (ADG part D).
* **Parallel work:** GOV-06, TEST-01, XT-01, SPK-PERS, SPK-NET, DEV-01 run independently.

### PS-1: Geometry Core Foundation (P0-alpha)
* **Gate:** G1 (conditions: ADG part D). **Dependencies and start/merge conditions:** ADG part C, not repeated here.
* **Inputs:** Core model of the architecture plan section 5; spike `cube-core` as reference only.
* **Tasks (id, owner):** CORE-01 (Core Agent); CORE-02 (Core Agent); CORE-03 (Core Agent); CORE-04 (Core Agent); CORE-05 (Core Agent); CORE-07 (Core Agent); CORE-06 (Core Agent); CORE-08 (Core Agent); CORE-09 (Core Agent); CORE-11 (Core Agent); CORE-10 (Core Agent); TEST-02 (Agent I (QA and Benchmarks)); TEST-05 (Agent I (QA and Benchmarks)); XT-02 (Main Agent / Integration Owner); TEST-06 (Agent I (QA and Benchmarks)); INT-01 (Main Agent / Integration Owner)
* **Output artifacts:** `cube-core` with Layer/Representation, CellKey(level), Region, channels and cell state, Transform, revision tokens, commit gate and CommitBatch, ids/ActorKind/errors, mutation-boundary handles, system-commit and recovery ports, export views and content hash, API freeze snapshot; golden-hash harness; Core property and loom suites; archcheck v1.
* **Contracts needed:** P0-alpha Owner-Approved and Implemented; PC-1 Implemented under Provisional approval (INT-01 records the statuses); C-05, C-10…C-15 NOT in this stage.
* **Tests:** contract tests per contract; property and loom tests; TEST-06 mutation-boundary negative tests (compile-fail, commit-log completeness, archcheck seeded violations); golden hashes on the CI host.
* **Acceptance criteria:** Exit criteria of the architecture plan section 15, including: no emulator or device condition; no Frozen contract; ACR-004 part A closed.
* **Exit gate:** G1 closes when every member is merged and accepted (ADG part D).
* **Parallel work:** From CORE-10: Processing, API-01, WGEN-01, CLI-00.

### PS-2: Processing Foundation
* **Gate:** G2 (conditions: ADG part D). **Dependencies and start/merge conditions:** ADG part C, not repeated here.
* **Inputs:** Core API freeze (CORE-10).
* **Tasks (id, owner):** CORE-15 (Core Agent); PROC-01 (Processing Agent); PROC-02 (Processing Agent); PROC-03 (Processing Agent); PROC-04 (Processing Agent); PROC-05 (Processing Agent); PROC-06 (Processing Agent); TEST-08 (Agent I (QA and Benchmarks)); GOV-07 (Project owner); CORE-16 (Core Agent); GOV-16 (Project owner); GOV-24 (Project owner)
* **Output artifacts:** world identity value types (CORE-16); `cube-processing` (snapshot with apron, provider registry and dependency graph, executor and task states, cancellation, stale rejection, caps, bounds slots); C-05 bounds data.
* **Contracts needed:** PR-01…PR-08 (Proposed; Owner-Approved only after the Geometry Processing Specification, task GOV-07); C-05.
* **Tests:** loom and stress suites (TEST-08); cap enforcement against a synthetic mobile budget; cancellation at every state. No device smoke is required for the stage exit.
* **Acceptance criteria:** No stale result is ever published under the explored interleavings; caps enforced; **G2 requires GOV-07**, so the stage cannot close without the specification or an owner-recorded alternative.
* **Exit gate:** G2 closes when every member is merged and accepted (ADG part D).
* **Parallel work:** Starts at CORE-10; Editing API, WorldGen generator and client skeleton run alongside. Downstream **production integration waits for G2.**

### PS-3: Editing API and Contracts
* **Gate:** G3 (conditions: ADG part D). **Dependencies and start/merge conditions:** ADG part C, not repeated here.
* **Inputs:** Core frozen; Processing interfaces frozen (API-02 merges only after G2: Processing production gate).
* **Tasks (id, owner):** API-01 (Main Agent / Integration Owner); API-02 (Main Agent / Integration Owner); API-03 (Main Agent / Integration Owner); API-04 (Main Agent / Integration Owner); API-05 (Main Agent / Integration Owner); GOV-09 (Project owner)
* **Output artifacts:** `cube-editing-api` with public types, ports (editing service, backend, observer, storage, journal, base provider), authorisation context, API-snapshot tooling.
* **Contracts needed:** E-02…E-07, E-09, E-11, E-12 and the storage ports Owner-Approved (GOV-09).
* **Tests:** mock-based contract tests; negative tests per error; API snapshot.
* **Acceptance criteria:** Backends, tools, history and persistence compile against the API without orchestration; no behaviour in the crate; API-02 merged only after G2.
* **Exit gate:** G3 closes when every member is merged and accepted (ADG part D).
* **Parallel work:** Orchestration (design), backends, persistence ports after freeze.

### PS-4: Editing Orchestration
* **Gate:** G4 (conditions: ADG part D). **Dependencies and start/merge conditions:** ADG part C, not repeated here.
* **Inputs:** API frozen (start); G2 and G3 (merge).
* **Tasks (id, owner):** ORCH-01 (Agent A); ORCH-02 (Agent A); ORCH-03 (Agent A); ORCH-04 (Agent A); ORCH-05 (Agent A); TEST-03a (Agent I (QA and Benchmarks)); TEST-10 (Agent I (QA and Benchmarks))
* **Output artifacts:** `cube-orchestration`; backend conformance suite (generic, TEST-03a); mutation-path negative tests through the real path (TEST-10).
* **Contracts needed:** E-04, E-06, E-07, E-11, E-12 Implemented.
* **Tests:** conflict matrix; rollback atomicity; retry by OperationId; observer ordering; no direct commit (archcheck, compile-fail).
* **Acceptance criteria:** All spike `cube-editing` behaviours reproduced through the new single user commit path; the three-writer audit passes.
* **Exit gate:** G4 closes when every member is merged and accepted (ADG part D).
* **Parallel work:** Backends and persistence run alongside; tools wait for G4 and G5.

### PS-5: Geometry Backends and Derived Providers
* **Gate:** G5 (conditions: ADG part D). **Dependencies and start/merge conditions:** ADG part C, not repeated here.
* **Inputs:** API frozen; Processing interfaces; (merge) G2, G3.
* **Tasks (id, owner):** CORE-12 (Core Agent); VOX-01 (Agent B); VOX-02 (Agent B); VOX-03 (Agent B); VOX-04 (Agent B); VOX-05 (Agent B); SDF-01 (Agent C); SDF-02 (Agent C); SDF-03 (Agent C); MSH-01 (Agent D); MSH-02 (Agent D); MSH-03 (Agent D); GOV-08 (Project owner); GOV-17 (Project owner)
* **Output artifacts:** three backend crates; providers (faces, greedy, Surface Nets, collision, LOD) with ownership pending O-1; palette data (CORE-12).
* **Contracts needed:** E-07 capabilities; PR-07/PR-08 registration; C-10 after RG-1.
* **Tests:** backend conformance (independent of any renderer); greedy covers the same area as faces; no T-junctions; determinism; benches on the CI host.
* **Acceptance criteria:** Conformance green per backend. **No device or renderer-default condition in this stage** (DG-1 and GOV-13 are separate).
* **Exit gate:** G5 closes when every member is merged and accepted (ADG part D).
* **Parallel work:** B, C, D in parallel with each other and with PS-4, PS-7, PS-10.

### PS-6: Tools: Sculpting and Boolean
* **Gate:** G6 (conditions: ADG part D). **Dependencies and start/merge conditions:** ADG part C, not repeated here.
* **Inputs:** ORCH-02, VOX-01, SDF-01 (start); G4, G5 (merge). Tool-level contracts must exist first (B-7).
* **Tasks (id, owner):** SCU-01 (Agent E); SCU-02 (Agent E); SCU-03 (Agent E); BOO-01 (Agent F); BOO-02 (Agent F); BOO-03 (Agent F); GOV-20 (Project owner)
* **Output artifacts:** `cube-tool-sculpt`, `cube-tool-boolean`.
* **Contracts needed:** BooleanPolicy and sculpt contracts Owner-Approved before implementation merges.
* **Tests:** transaction integration; explicit-conversion tests; async revision safety; undo round trips against a History fake.
* **Acceptance criteria:** No direct commits; capability queries only; no hidden conversions.
* **Exit gate:** G6 closes when every member is merged and accepted (ADG part D).
* **Parallel work:** E and F in parallel.

### PS-7: History
* **Gate:** G7 (conditions: ADG part D). **Dependencies and start/merge conditions:** ADG part C, not repeated here.
* **Inputs:** API ports; (merge) G4.
* **Tasks (id, owner):** HIST-01 (Agent G); HIST-02 (Agent G); HIST-03 (Agent G); HIST-04 (Agent G); GOV-25 (Project owner)
* **Output artifacts:** `cube-history`.
* **Contracts needed:** E-09, E-10, E-12 consumers.
* **Tests:** undo/redo properties; divergence conflicts; memory limits; history and persistence independence (archcheck).
* **Acceptance criteria:** System and Recovery commits never enter history; undo is a forward commit.
* **Exit gate:** G7 closes when every member is merged and accepted (ADG part D).
* **Parallel work:** Tools, Composite, Persistence.

### PS-8: Composite Geometry
* **Gate:** G8 (conditions: ADG part D). **Dependencies and start/merge conditions:** ADG part C, not repeated here.
* **Inputs:** ACR-003 decided; ORCH-01 (start); G4 (merge).
* **Tasks (id, owner):** CORE-14 (Core Agent); COMP-01 (Composite Agent); COMP-02 (Composite Agent); COMP-03 (Composite Agent); GOV-19 (Project owner); GOV-23 (Project owner)
* **Output artifacts:** Composite data in Core (CORE-14) and `cube-composite` operations.
* **Contracts needed:** C-13, C-14, C-15 Owner-Approved.
* **Tests:** graph invariants; transform propagation determinism.
* **Acceptance criteria:** No operations in Core; no cycles.
* **Exit gate:** G8 closes when every member is merged and accepted (ADG part D).
* **Parallel work:** Tools, History.

### PS-9: Resampling and Resolution Refinement
* **Gate:** G9 (conditions: ADG part D). **Dependencies and start/merge conditions:** ADG part C, not repeated here.
* **Inputs:** RG-1 passed for C-11/C-12; VOX-01.
* **Tasks (id, owner):** CORE-13 (Core Agent); RES-01 (Resolution Agent); RES-02 (Resolution Agent); RES-03 (Resolution Agent); GOV-18 (Project owner)
* **Output artifacts:** resolution data (CORE-13) and `cube-resolution`.
* **Contracts needed:** C-11, C-12 Owner-Approved after RG-1.
* **Tests:** mixed-resolution tests; seam tests between levels; round trips.
* **Acceptance criteria:** No cracks between levels; undoable.
* **Exit gate:** G9 closes when every member is merged and accepted (ADG part D).
* **Parallel work:** Tools.

### PS-10: Persistence
* **Gate:** G10 (conditions: ADG part D). **Dependencies and start/merge conditions:** ADG part C, not repeated here.
* **Inputs:** export views and hash (CORE-11), storage ports (API-04); durability spike result for compaction.
* **Tasks (id, owner):** TEST-04 (Agent I (QA and Benchmarks)); PERS-01 (Agent H); PERS-02 (Agent H); PERS-03 (Agent H); PERS-04 (Agent H); PERS-05 (Agent H); PERS-06 (Agent H); PERS-07 (Agent H); PERS-08 (Agent H); PERS-09 (Agent H); ITG-01 (Agent I (QA and Benchmarks)); QA-P2 (Agent I (QA and Benchmarks)); GOV-10 (Project owner); GOV-11 (Project owner)
* **Output artifacts:** `cube-persistence`: codec, manifest, snapshot store, journal and delta store, compaction, recovery of the user layer, migration framework, world record, composite persistence; FaultFs; integration tests.
* **Contracts needed:** P-01…P-07 reconciled with WP-01…WP-10; the persistence format is Owner-Approved by GOV-10 (conditions: ADG).
* **Tests:** fault injection at every numbered step; truncated tail; corrupted manifest/snapshot; version mismatch; double recovery; replay equals live state (ITG-01).
* **Acceptance criteria:** AOD-21 rules hold; no format is frozen before GOV-10; recovery needs neither History nor WorldGen. Device durability is DG-2, not a stage condition.
* **Exit gate:** G10 closes when every member is merged and accepted (ADG part D).
* **Parallel work:** PS-4…PS-9.

### PS-11: World Generation
* **Gate:** G11 (conditions: ADG part D). **Dependencies and start/merge conditions:** ADG part C, not repeated here.
* **Inputs:** 11A: Core read-only API frozen. 11B: system-commit port and ACR-004 part B decision.
* **Tasks (id, owner):** WGEN-01 (UNASSIGNED (WorldGen)); TEST-09 (Agent I (QA and Benchmarks)); WGEN-02 (UNASSIGNED (WorldGen)); WGEN-03 (UNASSIGNED (WorldGen)); WGEN-04 (UNASSIGNED (WorldGen)); WGEN-05 (UNASSIGNED (WorldGen)); ITG-02 (Agent I (QA and Benchmarks)); LEG-01 (UNASSIGNED (WorldGen)); GOV-21 (Project owner)
* **Output artifacts:** 11A pure generator and version registry (uses CORE-16 types); 11B service, Materialiser, eviction; cross-device harness (software); retirement of the spike streamer and `world-state-v0`.
* **Contracts needed:** WG-01…WG-10 Owner-Approved before 11B merges; SC-xx system-commit contracts.
* **Tests:** pinned hashes per version on the CI host; cancellation; border consistency; materialise/evict idempotence; authority; generation + journal + recovery end to end (ITG-02); archcheck: no direct commit left.
* **Acceptance criteria:** Only the Materialiser writes generated data; user data is never evicted before it is durable.
* **Exit gate:** G11 closes when every member is merged and accepted (ADG part D).
* **Parallel work:** 11A beside PS-2/PS-3; PS-12.

### PS-12: Client Runtime and Engine Adapter
* **Gate:** G12 (conditions: ADG part D). **Dependencies and start/merge conditions:** ADG part C, not repeated here.
* **Inputs:** Core frozen (skeleton); backends providers and Processing executor for the scheduler.
* **Tasks (id, owner):** CLI-00 (UNASSIGNED (Client Runtime)); CLI-01 (UNASSIGNED (Client Runtime)); CLI-02 (UNASSIGNED (Client Runtime)); CLI-03 (UNASSIGNED (Client Runtime)); CLI-04 (UNASSIGNED (Client Runtime)); ADP-01 (UNASSIGNED (Engine Adapter, O-5)); ADP-02 (UNASSIGNED (Engine Adapter, O-5)); ADP-03 (UNASSIGNED (Engine Adapter, O-5)); LEG-02 (UNASSIGNED (Engine Adapter, O-5)); RND-01 (Agent I (QA and Benchmarks))
* **Output artifacts:** `cube-client`, shrunk adapter, GDScript glue only, render regression tests, retirement of WorldBridge write paths.
* **Contracts needed:** client and adapter surfaces (PR-09 boundary).
* **Tests:** adapter API snapshot; `no_child_processes`; desktop smoke; render regression on the CI host.
* **Acceptance criteria:** No algorithm, no history and no write path in the adapter; no device condition (DG-1 is separate).
* **Exit gate:** G12 closes when every member is merged and accepted (ADG part D).
* **Parallel work:** PS-11.

### PS-13: Server Runtime and Networking
* **Gate:** G13 (conditions: ADG part D). **Dependencies and start/merge conditions:** ADG part C, not repeated here.
* **Inputs:** see ADG; transport spike; owner transport decision.
* **Tasks (id, owner):** NET-01 (UNASSIGNED (Networking Adapter, O-3)); NET-02 (UNASSIGNED (Networking Adapter, O-3)); NET-03 (UNASSIGNED (Networking Adapter, O-3)); GOV-12 (Project owner); NET-04 (UNASSIGNED (Networking Adapter, O-3)); SRV-01 (UNASSIGNED (Server Runtime)); SRV-02 (UNASSIGNED (Server Runtime)); SRV-03 (UNASSIGNED (Server Runtime)); SRV-04 (UNASSIGNED (Server Runtime)); SRV-05 (UNASSIGNED (Server Runtime)); SRV-06 (UNASSIGNED (Server Runtime)); SRV-07 (UNASSIGNED (Server Runtime)); CLI-05 (UNASSIGNED (Client Runtime)); TEST-03b (Agent I (QA and Benchmarks)); TEST-07 (Agent I (QA and Benchmarks)); GOV-22 (Project owner); SRV-08 (UNASSIGNED (Server Runtime))
* **Output artifacts:** transport contract, protocol/session types, in-process and external transports, server runtime with embedded and dedicated hosts, sessions and authentication integration, tick and single writer, replication, reconnect, permissions, server history integration, client network integration.
* **Contracts needed:** SR-01…SR-10, NET contracts Owner-Approved; protocol and transport approved by the owner.
* **Tests:** transport and host conformance on both transports; authority and security tests; fuzzing; bot soak.
* **Acceptance criteria:** one runtime, two hosts; embedded has no special rights; a client cannot commit or reach system ports.
* **Exit gate:** G13 closes when every member is merged and accepted (ADG part D).
* **Parallel work:** QA infrastructure.

### PS-14: Full QA and Release Validation
* **Gate:** G14 (conditions: ADG part D). **Dependencies and start/merge conditions:** ADG part C, not repeated here.
* **Inputs:** G13 for the soak; physical devices for DG-1…DG-3.
* **Tasks (id, owner):** QA-02 (Agent I (QA and Benchmarks)); QA-04 (Agent I (QA and Benchmarks)); QA-06 (Agent I (QA and Benchmarks)); DG-1 (Agent I (QA and Benchmarks)); GOV-13 (Project owner); DG-2 (Agent I (QA and Benchmarks)); DG-3 (Agent I (QA and Benchmarks)); GOV-15 (Project owner)
* **Output artifacts:** benches with baselines, full regression/soak, device tooling, DG-1…DG-3 reports, renderer default decision record.
* **Contracts needed:** none new.
* **Tests:** full suites; device matrix (low/mid/high Android ARM64, iOS); determinism matrix; thermal, battery, memory, cold start, 4 KB/16 KB pages.
* **Acceptance criteria:** every matrix device passes; no "mobile-ready" statement without DG-3 (AOD-11).
* **Exit gate:** G14 closes when every member is merged and accepted (ADG part D).
* **Parallel work:** -


## 4. Device gates (separate track)

| Gate | Purpose | Needs | Produces | Consumed by |
|---|---|---|---|---|
| DG-1 | faces vs greedy on physical devices (triangles, draw calls, memory, edit and load latency, visual differences, 10-minute stability) | `EXT-DEV`, QA-06, VOX-03, CLI-02, ADP-01 | report | GOV-13 (owner records the renderer default) |
| DG-2 | cross-device chunk-hash equality; directory-fsync and atomic-publish behaviour on device storage | `EXT-DEV`, QA-06, WGEN-05, PERS-05 | report | owner decisions on determinism claims and persistence format risk |
| DG-3 | memory, cold start, thermal soak, battery, 4 KB and 16 KB page configurations, signing, store build, network behaviour | `EXT-DEV`, G13, QA-06 | report | release decision |

An emulator never satisfies a device gate (AOD-11).

## 5. Parallel-work summary

| Window | Independent work (ownership permitting) |
|---|---|
| PS-0 | CI implementation, testkit, archcheck v0, SPK-PERS, SPK-NET, device acquisition |
| After CORE-10 | PROC-01…05, API-01, WGEN-01, CLI-00, tool contract drafting |
| PS-4…PS-9 | backends B/C/D, tools E/F, History, Composite, Resolution (disjoint crates; each waits for its own merge conditions) |
| PS-10 | Persistence beside PS-4…PS-9 |
| PS-11…PS-13 | client runtime and adapter beside WorldGen; networking beside server runtime |
