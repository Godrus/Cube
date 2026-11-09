# Cube functional roadmap

Human-readable view of what Cube can do, what is being built and what is still only a plan. The complete list with stable ids is `FEATURE-REGISTRY.md` (generated from `features.json`). The technical plan, dependencies and gates stay in `production/` (ADG in `production/module-boundaries.md`); this file does not replace them and never changes them.

**Last verified state:** `dev` at `f994c83` (CI green). Gates passed: G0, G1, G2 (RG-2 still mandatory), G3, G4. Open: G5 (PS-5), G10 (PS-10). RG-1 not passed.

**One rule above all:** integrated Rust code is *not* a player feature. Today **no feature is player available**: there is no game window, no generator and no client. Everything marked integrated is reachable only through Rust APIs and tests.

Status words: Decision (IDEA, PROPOSED, APPROVED, DEFERRED, REJECTED), Implementation (NOT_STARTED, IN_PROGRESS, IMPLEMENTED, TESTED, INTEGRATED), Player (NOT_AVAILABLE, INTERNAL_ONLY, DEMO_AVAILABLE, PLAYER_AVAILABLE). Blocked is a flag with blocker ids.

## 1. What exists, by function

| Direction | What works today (integrated, internal only) | What is missing | Product detail |
|---|---|---|---|
| Geometry Core | coordinates incl. negative space, sparse bricks with explicit absent-data states, revisions, atomic commit with three writer categories, views and content hash, bounds, world identity values, material palette (provisional) | selective micro-voxel detail (PS-9), layer mapping of palettes | approved architecture |
| Editing | transactional editing through one user commit path, conflicts, rollback, authorization port, idempotent retry also after restart (rebuilt from the journal) | undo/redo (PS-7), tools (PS-6), async handles | approved architecture |
| Geometry Backends | voxel backend (set, replace, fill, clear, copy and move with rotation, any level); faces, greedy, collision, LOD and incremental rebuild providers; conformance suite | SDF backend, mesh backend (PS-5, not started); greedy T-junctions | approved |
| Processing | snapshots with apron, provider graph, executor, cancellation, stale-result rejection, resource budgets | none planned before PS-9 | approved |
| Persistence | experimental checksummed codec, atomic snapshot save and load, write-ahead journal with write payload and replay of confirmed changes, crash-point harness | file journal, manifest, snapshot store, compaction (PERS-02..05); final format (GOV-10, needs RG-1) | AOD-21 approved; format not approved |
| World Generation | only the seed and generator version values | the generator itself, system materialiser | PROPOSED; owner unassigned |
| Rendering / Client | none (derived face meshes exist as data) | client runtime, engine adapter, mesh upload, camera | PROPOSED; owners unassigned |
| Construction Tools | none | sculpt brush, booleans (PS-6, after G4 and G5) | PROPOSED; needs product detail |
| History | none | undo/redo (PS-7) | PROPOSED; GOV-25 open |
| Resolution / Microvoxels | level-aware keys, storage, operations and providers | selective detailing, resampling (PS-9) | PROPOSED; blocked by RG-1 |
| Composite Geometry | none | PS-8 | PROPOSED |
| Server / Multiplayer | none | authoritative server, replication (PS-13) | PROPOSED; owners unassigned |
| Gameplay, Characters, Inventory, Crafting, Combat, NPC / AI | none | everything; a minimal slice is *proposed* in `production/FIRST_PLAYABLE_SCOPE.md` | **product detail still required**; not approved |
| Content / Mods | palette model only | packages, mod loading (no Mod SDK planned yet) | PROPOSED |
| Platforms / QA | CI (Linux and Windows), loom, cargo-deny, architecture checks, API snapshots, fault injection model | mobile device validation (DG-1..3) | approved; needs physical devices |

## 2. In development and next

* PS-5 remaining: SDF-01..03 and MSH-01..03 (independent of open decisions), then the T-junction criterion for greedy; G5 also needs RG-1/EXT-VA-RG1 handling (owner exception exists only for CORE-12).
* PS-10 start: PERS-02..05 (world record, manifest, snapshot store, file journal, compaction), then crash-recovery tests and GOV-10/GOV-11.
* Independent anytime: TD-2 (validation port needs a layer), TD-1 (observer queue, needs loom models), more backend conformance.

## 3. Blockers (owner or external)

| Blocker | Blocks | Needed |
|---|---|---|
| RG-1 / EXT-VA-RG1 (Voxel Architecture) | Frozen status of PC-1 and C-10, GOV-10 save format, CORE-13, G5 | the document, or a further owner-recorded alternative |
| GOV-10, GOV-11 | G10, final formats | owner decisions after RG-1 |
| Unassigned owners: WorldGen, Client Runtime, Engine Adapter, Server Runtime, Networking | PS-11, PS-12, PS-13 | owner assignment (DEC-072 deferral) |
| Gameplay owner and gameplay model (OI-8, REF-13) | every gameplay feature | owner |
| Physical devices (EXT-DEV) | DG-1..3, mobile claims | devices |
| CB-1 (Core commit and journal not one atomic action) | crash-atomicity claims | safe boundary exists; full fix is PS-10 design |

## 4. First playable vertical slice (proposed orientation, not an approved specification)

Source of the proposal: `production/FIRST_PLAYABLE_SCOPE.md` and `FIRST_PLAYABLE_DEPENDENCIES.md` (M2-T is not confirmed as final M2).

| # | Scenario | Exists | Missing | Depends on | How it will be checked | Status |
|---|---|---|---|---|---|---|
| 1 | Launch Cube | nothing | client runtime, engine adapter, entry point | F-CLI-001 | the app starts on desktop | NOT_STARTED |
| 2 | Create a world from a seed | seed and generator version values | generator, system materialiser, world record | F-WGN-001, F-PER-004 | same seed gives the same world hash on CI and desktop | NOT_STARTED |
| 3 | See the generated region | face, greedy, LOD providers (data) | renderer, mesh scheduling, chunk streaming | F-CLI-001, F-BKD-002 | screenshot or scene check in the client | NOT_STARTED |
| 4 | Move a camera or character | collision boxes (data) | controller, camera, input | F-CLI-002, F-BKD-005 | walk and jump without falling through | NOT_STARTED |
| 5 | Add and remove a block | editing chain end to end in tests | input, picking, client calls the chain | F-CLI-003, F-EDT-002, F-BKD-001 | the change is visible and authoritative | chain INTEGRATED, scenario NOT_STARTED |
| 6 | Save the world | atomic snapshot save, journal with payload | file journal, manifest, UI trigger | F-PER-004 | files verified by the codec | PARTIAL (library level) |
| 7 | Close the application | - | lifecycle handling | F-PER-004 | clean shutdown leaves a loadable world | NOT_STARTED |
| 8 | Load the world | snapshot load, journal replay, idempotency recovery (tests) | world record, manifest, loader UI | F-PER-004 | world hash equal after load | PARTIAL (library level) |
| 9 | Changes survived | proven in tests on the model and a real directory | the same through the real app, then on a device | F-PER-004, DG-2 | hash equality after restart | PARTIAL (not on devices) |

## 5. Later RPG functions

Characters, inventory, crafting, combat, creatures, NPC behaviour, multiplayer and mods have **no approved product specification yet**. They exist in the registry only as placeholders (F-GPL-001, F-SRV-001/002, F-MOD-001). When the owner approves one, an agent creates its feature ids, dependencies and acceptance criteria first (see AGENT-HANDOFF, rule 5) and only then implements.

## 6. Known inconsistencies between sources

* `PROJECT_STATUS.md` sections 4 and 5.2 still describe the 2026-11-05 documentation baseline; the stage rows were updated, the baseline text was not. This roadmap and `production/ps-0-exit-checklist.md` (sections 6 to 14) carry the current state.
* `AGENT_HANDOFF.md` (older, contains rules written before DEC-001 delegation, e.g. rule 8) and the new `AGENT-HANDOFF.md` coexist; the new file is the current state snapshot, the old one the long procedural guide.
* Decision status of ADG-planned but not yet started features (history, tools, server) is PROPOSED here because only the engineering plan names them, not a feature-level owner decision.
