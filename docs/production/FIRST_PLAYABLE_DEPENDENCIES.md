# M2 First Playable: technology dependencies

**Status:** **Proposed** [NP]. The mapping is the preparer's reading of the architecture documents at HEAD `5c9a5c5`; it is not a plan of record and does not authorise any work. Task identifiers are those of the Authoritative Dependency Graph (ADG) in `module-boundaries.md`; the computation is reproducible with `python docs/audit/m2_slice.py`.
**Scope:** `FIRST_PLAYABLE_SCOPE.md`. **Principle:** M2 needs the *minimum technology subset* that makes its loop real; all other technology proceeds in parallel and does not block M2.

## 1. Game capability to technology

Legend. Owner: the module owner in `module-boundaries.md` (A.1/A.2) or **UNASSIGNED**. Temporary: whether a simplified implementation is acceptable (see `FIRST_PLAYABLE_SCOPE.md` section 5). Blocks M2: **Yes** = needed for the loop; **No** = may be deferred.

| # | Game capability | Required technology | Required interface / contract (status) | Minimum implementation for M2 | ADG tasks (stage) | Owner | Temporary possible? | Blocks M2 |
|---|---|---|---|---|---|---|---|---|
| 1 | Launch, menus, HUD | client runtime, engine adapter, game UI | client views of server state; adapter values/views (Proposed) | main menu, pause menu, HUD in the engine, reading views only | CLI-00, ADP-01, ADP-02 (PS-12) | **UNASSIGNED** (Client Runtime, Engine Adapter, game UI) | yes (placeholder UI) | **Yes** |
| 2 | Create a world by seed | world identity, pure generator, generation service, system-commit materialiser | C-16..C-19 + `WorldSeed`/`GeneratorVersion` (CORE-16); WG-01..WG-10, SC-xx (Draft, GOV-21 not decided) | seeded generator with two terrain variants; materialisation through `SystemCommit`; world record | CORE-16 (PS-2); PERS-08 (PS-10); WGEN-01..04, LEG-01 (PS-11) | **UNASSIGNED** (WorldGen); Core Agent; Agent H | generator content yes; the **boundary no** (AOD-16/20) | **Yes** |
| 3 | Exploration and rendering | chunk streaming, derived data providers (mesh), client chunk cache, mesh scheduling, rendering | PR-01..PR-08 (Proposed, need the Processing Specification); provider interface; adapter views | faces provider (reference, AOD-10) meshes, closest-first scheduling, one view radius | PROC-01..05 (PS-2); VOX-02, VOX-04 (PS-5); CLI-01, CLI-02 (PS-12) | Processing Agent; Agent B (provider ownership pending O-1); **UNASSIGNED** client | greedy mesher **not needed**; LOD not needed | **Yes** |
| 4 | Character movement, jump, collision | fixed-step simulation, kinematic controller, collision queries against a voxel snapshot | collision provider (pending O-1); gameplay entity model (open, REF-13); physics determinism (open) | AABB-vs-voxel controller with gravity, jump, one-voxel step | VOX-05 (PS-5); **no task for the controller** | collision: Agent B (pending); controller: **UNASSIGNED** (gameplay, OI-8) | yes (simple kinematic) | **Yes** |
| 5 | Camera and input | input intents, camera | `CLI-04` intents (Proposed) | third-person orbit camera in the engine, keyboard and mouse mapped to intents | CLI-04 (PS-12) | **UNASSIGNED** (Client, Engine Adapter) | yes | **Yes** |
| 6 | Interaction (pick, aim) | voxel ray queries, spatial queries | PR-08 "Spatial Hierarchy / Bounds / BVH update" (home of the query API open) | reach-limited voxel ray traversal on the server view | PROC-06 holds the PR-08 bounds/BVH slots (not selected: a ray walk over bricks needs no BVH); **no task for the query API** | Processing Agent; query home open | yes | **Yes** |
| 7 | Destroy and place voxels | Editing API, orchestration, voxel backend, user commit path | E-02..E-12 (Proposed, GOV-09); ACR-002 (undecided); ACR-004 A | `set/clear/fill region` through `IEditingService`, authorisation by origin, one user commit path | API-01..04 (PS-3), ORCH-01..05 (PS-4), VOX-01 (PS-5) | Main Agent / Integration Owner; Agent A; Agent B | no (AOD-15) | **Yes** |
| 8 | Creatures and behaviour | entity model, server tick, deterministic spawning, simple AI | none exist (gameplay is outside the contracts) | state-machine creatures simulated on the server near the player, spawned by seeded rule | SRV-03 (PS-13) for the tick; **no gameplay tasks** | **UNASSIGNED** (gameplay) | yes | **Yes** |
| 9 | Combat | hit resolution, damage rules, events to clients | none exist | server-side attack, damage, death, respawn; events replicated | **no gameplay tasks**; SRV-04 for replication | **UNASSIGNED** (gameplay) | yes | **Yes** |
| 10 | Items, inventory, equipment, stats | gameplay data model, server-owned inventory, **player state persistence** | none exist; `system-boundaries-v1.md` lists "player, inventory, characters" under a server persistence service with the database undecided | item definitions as data, inventory list, one equip slot, derived stats; saved with the world | **no gameplay tasks; no persistence task for player state** | **UNASSIGNED** (gameplay); persistence: Agent H | yes (local file in the world save) | **Yes** |
| 11 | Save, load, recovery | codec, manifest, snapshot store, journal, compaction, recovery | P-01..P-07 and WP-xx (Draft); persistence format approval GOV-10 needs the Voxel Architecture (RG-1) | AOD-21 protocol: snapshot, fsync, verify, atomic manifest, journal tail replay | PERS-01..06 (PS-10); SPK-PERS (PS-0); ITG-01 | Agent H | the format may be labelled experimental only with an owner ruling | **Yes** |
| 12 | Single-player runtime (authority) | embedded authoritative server, tick loop, in-process transport | NET-01..03, SR-01..SR-10 (Draft, GOV-22) | embedded host over the in-process transport, single writer | SRV-01, SRV-03 (PS-13); NET-01..03 | **UNASSIGNED** (Server Runtime, Networking Adapter) | the in-process transport is the minimum; **no shortcut that bypasses the server** | **Yes** |
| 13 | State to client (chunks, entities, inventory, events) | replication, interest management, client cache | NET-02 protocol and session types (Draft); **game messages are not in NET-02** | replicate chunk deltas and a small set of entity/inventory/combat events | SRV-04, NET-02 (PS-13), CLI-01, CLI-05 | **UNASSIGNED** | yes (small message set) | **Yes** |
| 14 | Authority guarantees | mutation boundary, archcheck, adapter audit | MB-1 handles, XT-02, TEST-06, TEST-10 | compile-fail and archcheck evidence that gameplay code cannot mutate geometry except through the API | CORE-07..09, TEST-06, XT-02 (PS-1), TEST-10 (PS-4), ADP-02 | Core Agent; Agent I; Main Agent | no | **Yes** (FPS-13) |
| 15 | Art, animation, audio | placeholder asset pipeline | none | one placeholder character, two creatures, UI | none | **UNASSIGNED** | yes | **Yes** (placeholders only); audio **No** |
| 16 | Multiplayer, sessions, auth | network transport, protocol conformance | OAM-05 spike required; GOV-12 | none for M2 | SRV-02, SRV-05..08, NET-04, SPK-NET, TEST-03b/07 | **UNASSIGNED** | n/a | **No** (M4) |
| 17 | Mobile | device builds, touch, budgets | OAM-06; DG-1..DG-3 | statement only | DEV-01, DG-1..3 | owner / Agent I | n/a | **No** (before M3: checkpoint) |
| 18 | Undo/redo, tools, composite, resampling, SDF/mesh backends | History, tools, composite, resolution, backends | E-09..E-10, tool contracts | none | HIST-*, SCU-*, BOO-*, COMP-*, RES-*, SDF-*, MSH-* | various | n/a | **No** |
| 19 | Dedicated server, external transport | SRV-08, NET-04 | GOV-12 | none | SRV-08, NET-04 | **UNASSIGNED** | n/a | **No** |
| 20 | Performance and device budgets | benchmarks on target hardware | `RELEASE_CRITERIA.md` B-1..B-11 | measure and report on named desktop hardware | TEST-02, TEST-05, benches | Agent I | n/a | measured, **not** thresholds |

## 2. Gaps: technology the game needs that no task in the plan covers

| Gap | Needed by | Proposal |
|---|---|---|
| Gameplay rules module (entities, combat, items, stats, spawning) | rows 4, 8, 9, 10 | a *Game Rules* module above the Editing API ports (`GAME_TRACK_ROADMAP.md` section 6); needs an owner, an ADG Part A/B change and an entity-model spike (REF-13) |
| Character controller and collision-based movement | row 4 | part of Game Rules; collision queries from the collision provider (VOX-05); determinism is an open decision (roadmap-v1 gate 6) |
| Voxel ray/spatial query API | row 6 | decide the home of PR-08 queries (open in the contracts) |
| Player state persistence (inventory, stats, position) | rows 10, 11 | a *player record* stored with the world save; needs a contract and a place in PS-10 (not in PERS-01..09); `system-boundaries-v1.md` leaves the storage open |
| Gameplay protocol messages | row 13 | extend the protocol scope beyond handshake and session messages (NET-02) |
| Placeholder asset and UI pipeline | rows 1, 15 | an art/UI owner and a minimal pipeline; engine-side only |
| Deterministic seeded randomness service for loot and spawning | rows 2, 8, 9 | a rule (seeded, replayable) rather than a module; to be written into the Game Rules contract |

## 3. The minimum technology subset (65 of 147 ADG tasks)

Selection (a proposal): all of gate G1 plus `CORE-16`; `PROC-01..05`, `TEST-08`; `API-01..04`, `ORCH-01..05`; `VOX-01`, `VOX-02`, `VOX-04`, `VOX-05`; `PERS-01..06`, `PERS-08`, `ITG-01`; `WGEN-01..04`, `LEG-01`, `ITG-02`; `CLI-00`, `CLI-01`, `CLI-02`, `CLI-04`, `CLI-05`, `ADP-01..03`, `LEG-02`; `SRV-01`, `SRV-03`, `SRV-04`, `NET-01..03`.

| Group | Tasks | Owner situation |
|---|---|---|
| Core | 17 | Core Agent (12), Agent I (3), Integration Owner (2) |
| Processing | 6 | Processing Agent, Agent I |
| Editing API and orchestration | 9 | Integration Owner, Agent A |
| Voxel backend and providers | 4 | Agent B (provider ownership pending O-1) |
| Persistence | 8 | Agent H, Agent I |
| World generation | 6 | **UNASSIGNED** |
| Client runtime and adapter | 9 | **UNASSIGNED** |
| Embedded server and in-process transport | 6 | **UNASSIGNED** |

**20 of the 65 selected tasks (31 %) have no owner**: Engine Adapter 4, Client Runtime 5, WorldGen 5, Networking Adapter 3, Server Runtime 3 (`m2_slice.py`). None of the *gameplay* work is in the 65 because it is not in the plan at all.

Deferred without harming M2: History (PS-7), tools (PS-6), composite (PS-8), resampling (PS-9), SDF and mesh backends, greedy mesher, LOD, dedicated host, external transport, session/authentication integration, reconnect, permissions, device gates, full QA (PS-14), cross-device hash harness.

## 4. What the ADG as published does to this subset (verified by computation)

| Measure | Result |
|---|---|
| Tasks selected | 65 |
| Closure when gate edges are followed as published (stage gates are "all members merged") | **100 of 147** tasks: 35 more than selected (9 PS-0, 5 PS-2, 4 PS-5, 6 PS-10, 3 PS-7, 2 PS-8, 2 PS-3, 2 PS-4, 1 PS-11, 1 PS-13) |
| Closure over task-to-task edges only | 84 tasks (19 more) |
| Gates needed | G0, G2, G3, G4, G10 |
| **External inputs needed** | **`EXT-GPS` (Geometry Processing Specification) and `EXT-VA-RG1` (Voxel Architecture revalidation)**: both documents are absent |
| Owner-decision tasks in the closure | 18 (GOV-01..11, 16, 17, 19, 21, 22, 24, 25) |

Consequences:
1. **Under the published plan the First Playable cannot start its production path without the Processing Specification** (EXT-GPS -> GOV-07 -> G2 -> API-02 -> G3 -> ORCH -> G4 ...); AOD-24 forbids replacing it with a general assumption before PR-01..PR-08 are approved. This is the single largest external risk to M2, and it is a document, not code.
2. The Voxel Architecture enters through `ADP-01` (start-after `CORE-12`), whose approval `GOV-17` needs `GOV-08` (the RG-1 record) and through the format approval `GOV-10`.
3. Several pulled-in tasks are not needed by the loop. `VOX-03` (greedy) is required by `CLI-02` and `VOX-04`; `HIST-02` is required only by `LEG-02` (adapter removal of undo); `CORE-12` (MaterialMapping, C-10) only by `ADP-01`.

**Proposals to avoid waiting for everything (each needs the owner or the Integration Owner, none applied):**
* P-A Define **milestone slices** in the ADG (roadmap gap R-01): an M2 slice with task-level, not gate-level, preconditions; validate that the slice is a closed subgraph.
* P-B Remove the over-constraints for M2: greedy mesher not a precondition of the client mesh scheduler (faces is the reference, AOD-10); the adapter's "no undo" rule is satisfied by not offering undo, without the History module; `CORE-12` not a precondition of `ADP-01`.
* P-C Register **Provisional groups** for PR-01..PR-08 and E-02..E-12 (open problem H-3) so that production code can merge against them.
* P-D Ask the owner to **supply** the Processing Specification and the Voxel Architecture, or to record, per AOD-24, exactly which alternative is acceptable for M2 (scope and expiry) — this is the highest-leverage decision for time-to-playable.
* P-E Assign owners (GOV-03) for the five unassigned areas and add gameplay/content/art owners; until then the work can only be prepared as design.

## 5. Ordering of capabilities (what the Game Track can consume, and when)

| Capability becomes available (ADG) | Earliest Game Track use |
|---|---|
| Design documents only (now) | content tables, rules, creature and item definitions as data, UI sketches, test scenarios |
| G1 (Core: assets, bricks, commit, snapshots) | headless scenario tests of the world (no graphics) |
| G2 + G3 + G4 slices (Processing, Editing API, orchestration) | programmatic edit of the world through the production path; scripted "player" actions |
| VOX-01/02/05 | meshes and collision for the viewer and the controller |
| PS-10 slice | save/load of world and player records |
| PS-11 slice | generated worlds, spawning rules |
| PS-12 + SRV-01 slice | a character in a window |
| Game Rules module | creatures, combat, items on top of all of the above |

## 6. Findings that change the plan (summary)

1. 31 % of the M2 subset has no owner and 100 % of the gameplay layer has no task.
2. M2 on the published ADG depends on two absent external documents.
3. The ADG has no milestone slices, so any slice pulls whole stages.
4. Player-state persistence and gameplay protocol messages are not covered by any task.
