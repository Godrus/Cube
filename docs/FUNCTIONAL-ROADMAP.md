# Cube functional roadmap

Human-readable view of what Cube can do, what is being built and what is still only a plan. The complete list with stable ids is `FEATURE-REGISTRY.md` (generated from `features.json`). The technical plan, dependencies and gates stay in `production/` (ADG in `production/module-boundaries.md`); this file does not replace them and never changes them.

**Last verified state:** `dev` at `c248bce` (CI green; includes the Godot first world, world record, greedy without T-junctions); `dev` at `c87534e`, CI 42720748823 green. Gates passed: G0, G1, G2 (RG-2 still mandatory), G3, G4. Open: G5 (PS-5), G10 (PS-10). RG-1 not passed.

**Product view:** the player-facing order of work (milestones M0..M12, crosswalk to PS-0..PS-14) is `roadmap/PRODUCT-ROADMAP.md`; the vision is `vision/GAME-VISION.md`. Inventory and world are reconciled after a crash (F-GPL-002, Rust-tested, branch `prod/m1-reconcile`).

**One rule above all:** integrated Rust code is *not* a player feature. Today **no feature is player available**. One thing is a **developer demo (DEMO_AVAILABLE)**: a Godot 4 scene (`godot/`) that shows a generated, editable and savable voxel terrain; it needs a Rust build and a local Godot and is not part of CI. Everything else marked integrated is reachable only through Rust APIs and tests.

Status words: Decision (IDEA, PROPOSED, APPROVED, DEFERRED, REJECTED), Implementation (NOT_STARTED, IN_PROGRESS, IMPLEMENTED, TESTED, INTEGRATED), Player (NOT_AVAILABLE, INTERNAL_ONLY, DEMO_AVAILABLE, PLAYER_AVAILABLE). Blocked is a flag with blocker ids.

## 1. What exists, by function

| Direction | What works today (integrated, internal only) | What is missing | Product detail |
|---|---|---|---|
| Geometry Core | coordinates incl. negative space, sparse bricks with explicit absent-data states, revisions, atomic commit with three writer categories, views and content hash, bounds, world identity values, material palette (provisional) | selective micro-voxel detail (PS-9), layer mapping of palettes | approved architecture |
| Editing | transactional editing through one user commit path, conflicts, rollback, authorization port, idempotent retry also after restart (rebuilt from the journal) | undo/redo (PS-7), tools (PS-6), async handles | approved architecture |
| Geometry Backends | voxel backend (set, replace, fill, clear, copy and move with rotation, any level); faces, greedy, collision, LOD and incremental rebuild providers; **SDF backend** (union, subtraction, material, Surface Nets mesh, normals, collision) and **mesh backend** (pages, validation, add/remove/material, collision), both tested, provisional; conformance suite on all three | greedy T-junctions inside a brick; G5 (RG-1) | approved |
| Processing | snapshots with apron, provider graph, executor, cancellation, stale-result rejection, resource budgets | none planned before PS-9 | approved |
| Persistence | experimental checksummed codec, atomic snapshot save and load, **file-backed journal with write payload (acknowledged = fsynced, a damaged acknowledged record fails closed, a torn unacknowledged tail is dropped)**, replay of confirmed changes, restart idempotency, crash-point harness, killed-process test, **manifest, verified snapshot store, crash-safe compaction and world open/recovery (experimental, crash-tested at every step)** | world record (PERS-08), device durability (DG-2), retry history before the last compaction (L-COMP-1); final format (GOV-10, needs RG-1) | AOD-21 approved; format not approved |
| World Generation | **deterministic terrain generator terrain-v1** (integer only, versioned, pinned hashes; tested on the CI host; engineering baseline, not an approved world design) | world record (PERS-08), service, materialiser (WGEN-02..04, gated by GOV-21 and GOV-11), caves, biomes, structures | generator approved as engineering foundation (owner message 2026-11-10); technical assignment DEC-091 |
| Rendering / Client | **Godot 4 scene and thin adapter (developer demo, screenshot in docs/screenshots)**, **engine-neutral client runtime foundation** (active region, derived-geometry cache, dirty updates, render batches, lifecycle), tested headless | engine adapter (ADP-01, O-5), mesh upload, camera, input, a window | foundation approved (owner message 2026-11-10, DEC-091); adapter owner unassigned |
| Construction Tools | none | sculpt brush, booleans (PS-6, after G4 and G5) | PROPOSED; needs product detail |
| History | none | undo/redo (PS-7) | PROPOSED; GOV-25 open |
| First visible world (integration) | **headless scenario**: seed to generated bricks to faces to client batches, edit updates geometry, restore from seed plus journal; checked against an oracle | adapter and window: the scene is not player visible | foundation approved; not a player feature |
| Resolution / Microvoxels | level-aware keys, storage, operations and providers | selective detailing, resampling (PS-9) | PROPOSED; blocked by RG-1 |
| Composite Geometry | none | PS-8 | PROPOSED |
| Server / Multiplayer | **in-process local host of one world** (DEC-092; not the networked server) | authoritative server, replication (PS-13) | PROPOSED; owners unassigned |
| Gameplay, Characters, Inventory, Crafting, Combat, NPC / AI | **a walking player in the Godot demo**: gravity, jump, collision with the voxel world, first and third person camera that stops in front of walls, place and remove blocks, the world generated around the player as they move (finite, about 770 voxels each way), far bricks unloaded (resident bricks stay flat on a walk out and back), a hotbar and inventory with creative/survival rules (prototype), mining and placing with a target outline and placement preview, one objective (mine 8, place 12), world, player, inventory and progress saved on quit and continued on the next start (crafting and ground drops do not exist; **combat prototype, run with `--creatures`**: health, a wooden sword on `E`, slimes that chase and hurt you, loot into the inventory, kills saved; developer demo, see F-CLI-005) | everything; a minimal slice is *proposed* in `production/FIRST_PLAYABLE_SCOPE.md` | **product detail still required**; not approved |
| Content / Mods | palette model only | packages, mod loading (no Mod SDK planned yet) | PROPOSED |
| Platforms / QA | **Visual QA of Godot screenshots with TypeSafe Jev (experimental, advisory only; v2 measured on 37 real Godot pictures: deterministic code found every partial load and stale update, TypeSafe found missing geometry and wrong materials without a false positive but missed the one-block edits; counts in `tools/visual-qa/V2-RESULTS.md`). v3 adds one command (`python cube_visual_qa.py suite ...`) that generates reproducible scenarios, runs them in Godot (20 of 20 executed, identical on a second run), checks them in code, and plans batched TypeSafe questions (one live comparison of 8 requests with no errors; limits and plans in `tools/visual-qa/V3.md`)**, CI (Linux and Windows), loom, cargo-deny, architecture checks, API snapshots, fault injection model | mobile device validation (DG-1..3) | approved; needs physical devices |

## 2. In development and next

* PS-5 remaining: the T-junction criterion for greedy inside a brick, and RG-1/EXT-VA-RG1 handling (owner exception exists only for CORE-12); SDF and Mesh are done at TESTED level.
* PS-11/12 foundation: WGEN-01 and CLI-00..02 (engine-neutral) are done; next WGEN-02..04 (gated by GOV-21, GOV-11), ADP-01 (no owner), CLI-03/04.
* PS-10: PERS-04, PERS-02, PERS-03, PERS-05 are implemented and tested (SPK-PERS report: `research/spk-pers-durability-report.md`). Remaining: PERS-06..09 (recovery orchestration, world record, migration), GOV-10 and GOV-11 (owner), device validation (DG-2).
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

Source of the proposal: `archive/production/FIRST_PLAYABLE_SCOPE.md` and `FIRST_PLAYABLE_DEPENDENCIES.md` (M2-T is not confirmed as final M2).

| # | Scenario | Exists | Missing | Depends on | How it will be checked | Status |
|---|---|---|---|---|---|---|
| 1 | Launch Cube | Godot project that starts the world (`godot --path godot`) | packaged build, menu | F-CLI-001, F-ADP-001 | the app starts on desktop | DEMO (developer machine) |
| 2 | Create a world from a seed (**world record**, generator, static area) | seed and generator version values, **generator terrain-v1** (CI-host determinism proven) | system materialiser (called directly in tests), world record | F-WGN-001, F-WGN-002, F-PER-004 | same seed gives the same world hash on CI and desktop | PARTIAL (generator only, headless) |
| 3 | See the generated region | **rendered in Godot 4.2.2** (docs/screenshots/first-world.png); faces baseline, greedy without T-junctions | streaming beyond a static area, textures | F-CLI-001, F-CLI-004, F-BKD-002 | screenshot or scene check in the client | DEMO (static area 11 x 11 x 11 bricks) |
| 4 | Move a camera or character | free-fly camera and intents (unit tested; keyboard and mouse handlers in the scene, not exercised by a person yet); collision boxes (data) | character controller | F-CLI-002, F-BKD-005 | walk and jump without falling through | PARTIAL (camera only) |
| 5 | Add and remove a block | crosshair picking, edit through the Rust chain, geometry update; **applied inside Godot by the smoke script** | mouse-driven use verified by a person | F-CLI-003, F-EDT-002, F-BKD-001 | the change is visible and authoritative | DEMO (smoke) |
| 6 | Save the world | atomic snapshot save, file journal with payload, manifest, verified snapshot, compaction; edits are restorable from seed plus journal (headless test) | world record, UI trigger | F-PER-004 | files verified by the codec | PARTIAL (library level) |
| 7 | Close the application | the journal is durable per edit; `close()` stops the session | menu and clean-quit UI | F-PER-004 | clean shutdown leaves a loadable world | PARTIAL |
| 8 | Load the world | `open_world_with_record`: record, manifest, regenerated base, journal replay, idempotency recovery (tests, real files; second Godot launch loads the saved world by design, not yet exercised by a person) | loader UI | F-PER-004 | world hash equal after load | PARTIAL (library level) |
| 9 | Changes survived | proven in tests on the model and a real directory | the same through the real app, then on a device | F-PER-004, DG-2 | hash equality after restart | PARTIAL (not on devices) |

## 5. Later RPG functions

Characters, inventory, crafting, combat, creatures, NPC behaviour, multiplayer and mods have **no approved product specification yet**. They exist in the registry only as placeholders (F-GPL-001, F-SRV-001/002, F-MOD-001). When the owner approves one, an agent creates its feature ids, dependencies and acceptance criteria first (see AGENT-HANDOFF, rule 5) and only then implements.

## 6. Known inconsistencies between sources

* `archive/PROJECT_STATUS.md` is a historical snapshot of 2026-11-05. The current state is this roadmap, `production/ps-0-exit-checklist.md` (sections 6 to 14) and `.agent/BOARD.md`.
* The old handoff files (`archive/AGENT_HANDOFF.md`, `archive/AGENT-HANDOFF.md`) are archived; their valid rules live in `handoff/NEW-CHAT-START.md`.
* Decision status of ADG-planned but not yet started features (history, tools, server) is PROPOSED here because only the engineering plan names them, not a feature-level owner decision.
