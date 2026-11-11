# Cube Product Roadmap (M0 to M12), engineering crosswalk, critical path

**Status:** proposed execution order from the owner's directive of 2026-11-10. The strategic direction is the owner's; the milestone cut and order are an agent proposal that stays reversible. It changes **no** engineering gate, ADG edge or contract (those stay in `docs/production/`). Status words follow `docs/AGENT-HANDOFF.md` rule list: implementation NOT_STARTED / IN_PROGRESS / IMPLEMENTED / TESTED / INTEGRATED, availability NOT_AVAILABLE / INTERNAL_ONLY / DEMO_AVAILABLE / PLAYER_AVAILABLE. Feature ids refer to `docs/features.json` (never a second registry); statuses are copied from it, not restated here. Vision: `docs/vision/GAME-VISION.md`. Tasks: `WORK-PACKAGES.md`.

**Numbering.** `M0..M12` here are *product* milestones. The older Game Track uses `M2-T` (technical slice: walk, edit, save/load, done in the demo), `M2` (First Playable with a gameplay loop) and `M3` (Vertical Slice) in `docs/production/`. Proposed mapping: Game Track M2 = product M1 + M2 (+ minimal M3); Game Track M3 = product M3..M5 slice. The historical Phase 0..8 of GDD/TDD v0.3 are kept in section 2; PS-0..PS-14 are not renamed.

## 1. Milestones

Status = what is true in the repository on `dev` 679e8d5 plus the reconciliation commit in `NEXT-EXECUTION.md`. "Demo" means the Godot developer demo, run by hand on the developer machine, not in CI.

| M | Goal and player outcome | Required features | Engineering stages | Main risks | Acceptance (playable) | Status |
|---|---|---|---|---|---|---|
| **M0** Current playable foundation | Launch, walk, edit blocks, inventory, save/continue | F-CLI-001/002/003, F-GPL-002/003, F-SRV-003/004, F-WGN-002 | PS-1..5, 10 (part), 11 (part), 12 (demo) | Godot not in CI; steep terrain blocks walking | `python tools/visual-qa/walk_smoke.py --play` in real Godot | IMPLEMENTED, DEMO_AVAILABLE |
| **M1** Persistent world and gameplay integrity | Travel and build for long without losing progress or filling memory | evict edited bricks; inventory and world consistent (done: reconcile); worker-thread generation if measured; character data ready for a server | PS-10, PS-4, PS-12 | edited-brick revisions in replay (see WP M1-A1); CB-1 | build, leave, unload, restart, return: intact; kill between stores: no dupes | IN_PROGRESS (inventory consistency TESTED; eviction NOT_STARTED) |
| **M2** First action RPG combat loop | See an enemy, fight, win, loot, repeat (prototype loop exists) | health, damage, death/respawn, one weapon, one hostile creature with chase and attack AI, loot into inventory, combat feedback, persistence | gameplay (no module yet, see below), PS-12; PS-13 contracts later | no gameplay owner or module (OI-8/ODM-05); balance undecided | end-to-end scripted combat in real Godot | IMPLEMENTED prototype, DEMO_AVAILABLE with `--creatures` (F-GPL-004/005, F-CRT-001/002, F-CLI-005); not played by a person |
| **M3** Interesting procedural world | Exploring shows new things | 2-3 biomes with transitions, caves, ores, trees, simple structures, POIs, generator versioning, old worlds still load | PS-11 | WGEN-02..04 gated (GOV-21, GOV-11); noise cost on main thread | a person walks and sees different areas; old save regenerates identically | IMPLEMENTED prototype, DEMO_AVAILABLE: biomes, caves, ores, trees, ruins (F-WGN-003..006); not explored by a person |
| **M4** RPG progression | The character grows and plays differently | XP, levels, stats, first specialisation, several abilities, weapon types, armour, loot tiers, persistence | gameplay | class design is an owner decision (ODM-05) | level up, new ability, new weapon changes play | IN_PROGRESS: XP and levels done (slice 1, F-GPL-006, `cube-app/src/progression.rs`); first weapon type and recipe done (iron sword, G, F-GPL-007); stats, abilities, armour, more recipes not started |
| **M5** Crafting and deep building | Building is more than unit cubes | recipes, stations, tools, sculpt, boolean, undo/redo, composite, micro-voxel editing (limited) | PS-6, 7, 8, 9 | G5/RG-1 open; OPN-07 (smooth surfaces) | craft a tool, sculpt, undo, place a composite object | NOT_STARTED (engine parts TESTED, not player tools) |
| **M6** Living world | The world feels alive | several creatures, behaviour states, spawn/despawn, simple ecosystem, NPCs, settlements, day/night, weather | gameplay, PS-11 | simulation cost; owner for creatures | watch creatures act on their own | NOT_STARTED |
| **M7** Adventure content | Exploration leads to adventures | dungeons, local bosses, rewards, events, quests, unique items, collections, repeatables | gameplay, PS-11 | content volume; pipeline | clear a dungeon, beat a boss | NOT_STARTED |
| **M8** Multiplayer vertical slice | Two players travel, fight and build together | dedicated server, two clients, auth (minimal), character and world sync, shared building, combat sync, server inventory validation, reconnect, interest management | PS-13 (+10) | no Server/Networking owner (DEC-072); transport undecided | two clients in one world | NOT_STARTED |
| **M9** Android and cross-play MVP | PC and Android in one world | Android build, touch input, adaptive UI, mobile profile, compatible content, physical-device checks | PS-12, 14 | no physical devices (EXT-DEV); thermal and memory | a PC and an Android player together | NOT_STARTED (early Android spike due before M9) |
| **M10** Expanded RPG and social | Depth and community | several classes, skill growth, weapon variety, deeper crafting, trade, groups, guilds, collections, social spaces | gameplay, PS-13 | economy integrity (AI-20) | group activity with trade | NOT_STARTED |
| **M11** Large-scale infrastructure | Toward 500 players | distributed simulation, regional servers, entity handoff, load balancing, monitoring, admin, accounts, stress tests | PS-13, 14 | only after M8 works | measured load tests | NOT_STARTED |
| **M12** Production quality | Release quality | art, animation, sound, music, UX, accessibility, content, PC and mobile optimisation, mod support, user content editor, release prep | PS-14 | scope | release candidate checks | NOT_STARTED |

**Visual direction (owner, 2026-11-11):** Cube World Alpha / Veloren style; micro-voxel character work is deferred and changes no milestone. See `docs/vision/VISUAL-STYLE-GUIDE.md`. Focus: mechanics, adventures, RPG progression, exploration, Android compatibility.

**Do not** build the M11 infrastructure before M8 runs; **do not** wait for M8 to design the network contracts: gameplay state in M2 must be engine-neutral and server-authoritative in shape (intent in, validated result out).

## 2. Historical Phase 0..8 of GDD/TDD v0.3 mapped to today

The v0.3 text is not in the repository; this list is the owner's directive's own summary. Order of execution does not follow the old numbering.

| Phase | Content | Where it lives now |
|---|---|---|
| 0 Technical research | engines, mobile rendering, chunks, meshing, networking, memory | engine chosen (Rust + Godot, AOD-01/02); spike done; **Android spike and memory profiling on devices not done** (DG-1..3) |
| 1 Voxel engine prototype | chunks, terrain, place/break, persistence, streaming, PC and Android builds | M0 (PC); Android in M9 |
| 2 Gameplay prototype | controller, camera, inventory, items, basic combat, creatures, AI, equipment | controller, camera, inventory: M0; the rest M2, M4 |
| 3 World generation | biomes, caves, structures, resources, vegetation, weather, POIs | M3, M6 |
| 4 Multiplayer prototype | server, auth, sessions, cross-play, sync, shared building | M8, M9 |
| 5 Persistence and infrastructure | accounts, characters, world, regions, private server, local world, recovery | M1 and M8/M11; world and recovery exist at engine level (PS-10) |
| 6 RPG and content | classes, skills, progression, crafting, dungeons, bosses, collections, social | M4, M5, M7, M10 |
| 7 Scaling | regions, load balancing, handoff, 500-player tests, DB, monitoring | M11 |
| 8 Production | content, UX, mobile and PC optimisation, QA, certification, release | M12 |

## 3. Engineering crosswalk

`Product capability -> original phase -> engineering stage -> modules -> status -> availability -> evidence`. Verified against `docs/features.json` on `dev` 679e8d5. Dependencies are the ADG's (`production/module-boundaries.md`), not new ones.

| Capability | Phase | Stage | Modules | Status | Availability | Evidence / feature ids |
|---|---|---|---|---|---|---|
| Walk, camera, place/mine blocks | 1, 2 | PS-12 (+4, 5) | client, app, server, orchestration, voxel backend | INTEGRATED | DEMO_AVAILABLE | F-CLI-002/003 |
| Streaming with unloading (unedited bricks) | 1 | PS-12 | server, core | INTEGRATED | DEMO_AVAILABLE | F-SRV-004 |
| Unloading of edited bricks | 1, 5 | PS-10 + PS-12 | server, persistence | NOT_STARTED | NOT_AVAILABLE | limit in F-SRV-004 |
| Inventory, objective (prototype rules) | 2 | PS-12 (gameplay: no module) | app | INTEGRATED | DEMO_AVAILABLE | F-GPL-002/003 |
| Inventory/world consistency | 5 | PS-10 design (CB-1) | app, server | TESTED (reconcile, Rust tests) | DEMO_AVAILABLE | F-GPL-002, `tests/reconcile.rs` |
| Combat, creatures, AI | 2, 6 | gameplay; PS-12/13 for views and authority | `cube-app` (no gameplay module yet) | prototype INTEGRATED | DEMO_AVAILABLE (`--creatures`) | F-GPL-004/005, F-CRT-001/002, F-CLI-005 |
| Biomes, caves, structures, POIs | 3 | PS-11 | worldgen | NOT_STARTED (terrain-v1 only) | NOT_AVAILABLE | F-WGN-002 |
| Save/load, recovery, world record | 5 | PS-10 | persistence | INTEGRATED (experimental format) | DEMO_AVAILABLE | F-PER-003/007/008/009 |
| Sculpt, boolean | 6 | PS-6 | tools over editing | NOT_STARTED | NOT_AVAILABLE | F-TLS-001/002 |
| Undo/redo | 6 | PS-7 | history | NOT_STARTED, blocked GOV-25 | NOT_AVAILABLE | F-HST-001 |
| Micro-voxel editing | 6 | PS-9 | resolution, voxel backend | level-aware ops INTEGRATED, selective detail NOT_STARTED | NOT_AVAILABLE | F-RES-001/002 |
| Composite objects (modular gear) | 6 | PS-8 | composite | NOT_STARTED, blocked GOV-19/23 | NOT_AVAILABLE | F-CMP-001 |
| Shared building, multiplayer, cross-play | 4 | PS-13, 14 (+3, 4, 10) | server, net | NOT_STARTED, owners unassigned | NOT_AVAILABLE | F-SRV-001/002 |
| Android | 1, 8 | PS-12, 14 | adapter, client | NOT_STARTED | NOT_AVAILABLE | F-PLT-002 |
| Content packages and mods | 6, 8 | PS-14 / content | content (proposed) | NOT_STARTED | NOT_AVAILABLE | F-MOD-001 |

**Gameplay has no module in the ADG.** `cube-app/src/game.rs` holds the prototype gameplay in the composition root. A new crate (for example `cube-game`, engine-neutral) needs an ADG Part A/B edit, which `cargo xtask archcheck` enforces equal to the code graph: that is an owner/architecture decision (ODM-05, OI-8). Until then new engine-neutral gameplay code lives in `cube-app` as a pure module (no Godot, no I/O in the rule code), shaped so that it can be moved without change.

## 4. Dependencies and critical path

Hard technical (T), product (P), owner decision (O), external (E):

```
M1 (T) eviction of edited bricks ......................... independent of M2
M2 combat: needs (T) inventory ops (exist), (O) module location, no persistence blocker
   -> M4 progression (O: class design)  -> M10
M3 world: (T) PS-11 service WGEN-02..04 (gated O: GOV-21, GOV-11) or a version-2 generator inside the existing registry
M5 tools: (T) G5/RG-1 for micro-voxel and composite (O); sculpt/boolean/history PS-6/7 after G4 (done)
M8 multiplayer: (T) M2 intent/authority shape, PS-10 recovery, (O) Server/Networking owners, transport; (E) none
M9 Android: (E) physical devices, (T) M8 for cross-play; feasibility spike has no dependency but a device
M11: after M8 only
```

**Critical path to the MVP (section 6a of the vision):** M2 (combat) -> M3 (world) -> M4 (progression) -> M8 (two clients) -> M9 (Android). M1 integrity runs in parallel and must finish before any claim that edits persist for long play. M5 and M6 enter the MVP in minimal form.

**Independent work available now** (no owner decision needed): M1-A1 design and implementation; M1-A3 measurement of main-thread generation; M2 engine-neutral combat in `cube-app`; M3 caves and a second biome as a new generator version in `cube-worldgen` (registry-based, pinned hashes); Godot HUD and animation for M2.

**Blocked, with who unblocks:** gameplay module and owner (owner: ODM-05/OI-8); Server, Networking, Client Runtime, Engine Adapter, WorldGen owners (owner: DEC-072 deferral); final save format GOV-10/11 and Frozen contracts (owner, after RG-1); Android validation (owner: devices).

## 5. Owner decisions that really block

| Id | Question | Blocks | Meanwhile |
|---|---|---|---|
| OI-8 / ODM-05 | gameplay owner and module location | clean M2/M4 structure | pure module in `cube-app` |
| GOV-10/11 | final save format | stable saves, device claims | experimental formats, no promises |
| OPN-19 | Game Track pillars as written | M2 acceptance wording | this document's pillars as working text |
| DEC-072 | owners of Server/Networking/Client/Adapter/WorldGen | M8 start | contracts and shapes only |
| rules | inventory, death, balance numbers | final M2/M4 | prototype values, configurable |
| OPN-07 | smooth surfaces in the building scope | M5 planning (SDF/mesh tools) | engine parts stay as built |
