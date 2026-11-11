# Work packages: epics and tasks for M1, M2, M3 (and later epics)

**Status:** proposed decomposition. Hierarchy: Vision -> pillar -> milestone -> epic -> feature (`docs/features.json`) -> work package -> task -> acceptance test. Each task names its acceptance; a task is done only by the Definition of Done in `docs/handoff/NEW-CHAT-START.md` section 6. Ids here are work ids (`E-`, `M1-A1`, `CMB-01`), not feature ids; a feature id is created only when a real new function appears (rule 5 of `docs/AGENT-HANDOFF.md`). Roles are engineering areas, not owner assignments (the ownership matrix and DEC-072 stand; nobody is assigned by this file).

## Epic E-M1: persistent world and gameplay integrity (M1)

| Task | Action | Start state | Acceptance | Status |
|---|---|---|---|---|
| **M1-A2** | Inventory and world reconciliation | `game.state` and the journal were two stores | write-ahead intent in `game.state`; at open settle against the journal; Rust tests crash between each write pair (`crates/cube-app/tests/reconcile.rs`) | **TESTED** on branch `prod/m1-reconcile` (see `NEXT-EXECUTION.md` for integration) |
| **M1-A1** | Unload edited bricks and bring them back | `evict_far` never unloads bricks in `Streamed.edited` (pinned) | build over many bricks, walk away, resident memory bounded, come back and restart: identical content; edit journal and snapshots intact | NOT_STARTED, design below |
| **M1-A3** | Measure generation cost on the main thread | `ensure_loaded` generates up to 48 bricks per frame in the frame call | a measured frame-time table (debug and release) under heavier worlds; only if it exceeds a budget, move generation to a worker with version checks, determinism and budgets | NOT_STARTED |
| **M1-A4** | Character data ready for a server | player and game state are client files | an engine-neutral `CharacterRecord` with a stable text/binary form separate from the world (AI-15) | NOT_STARTED |

### M1-A1 design notes (so nobody rediscovers the trap)

Why edited bricks are pinned: the journal records `expected` revisions of the cells an edit touched, and **every unload and reload of a brick raises its revision by one** (never resets). A restart replays the journal onto a regenerated base; `bring_to_revision` reproduces the unload/reload steps of the *first* expectation of each brick only. If an edited brick were unloaded and reloaded between two of its edits, the second edit would expect a revision that a replay from scratch does not reach, and replay would fail (or silently drop the edit as superseded). So unloading edited bricks needs one of:

1. **Journal the reload** of an edited brick (a system record the replay understands), so replay reproduces the revision. Touches the journal contract (PS-10), needs design and probably owner review.
2. **An edited-brick store**: on eviction write the brick's current content to a per-brick file (checksummed, atomic) with its revision; reload from it, with replay skipping edits already contained (revision comparison). Needs a rule for what replay does on top of a stored brick.
3. **Fold edits into a snapshot** (compaction exists and is crash-tested) and reload bricks from that snapshot by key. Needs per-brick loads from the snapshot (the store loads whole worlds today).

Recommended first step: a short spike on option 2 or 3 in a test only (no API change) to see if replay-from-seed-plus-journal equals the world after eviction cycles, with the failing case (option "just evict") reproduced first as a test. Do not change the replay semantics (`replay_confirmed`) or the journal format without the persistence owner path; both are experimental until GOV-10.

## Epic E-CMB: basic combat (M2)

Goal: one scripted loop. See an enemy, it chases and attacks, the player attacks it, Rust computes damage, it dies, loot enters the inventory, the state is saved, the loop repeats. Out of scope: classes, skills, balance, networking (shape only), art.

Placement: engine-neutral Rust in `cube-app` (`combat.rs`, `creature.rs`) until the owner decides a gameplay crate (see `PRODUCT-ROADMAP.md` section 3). No Godot types, no wall clock, no randomness without a seeded source; fixed time step input; results are plain data the adapter draws.

Shape (so that it can become server-authoritative): `Input -> client intent (Attack{target?}) -> host validates and resolves (reach, cooldown, state) -> CombatEvent list -> game state -> client feedback`. The client never decides damage.

| Task | Action | Public data | Acceptance (Rust test unless noted) |
|---|---|---|---|
| CMB-01 | `Health` component and `Damage` event; death state; respawn rule (position, health) | `Health{cur,max}`, `Life{Alive,Dead}` | damage clamps at zero, death once, respawn restores, overflow-safe integers |
| CMB-02 | Attack resolution: cooldown, reach, one melee weapon definition (data) | `WeaponDef`, `AttackIntent`, `CombatEvent` | out-of-reach and on-cooldown refused with no state change; deterministic for equal inputs |
| CMB-03 | Persist player health and weapon in the game state (versioned text, like `game.state`, reading old files) | `GameState` extension | old `cube-game 1` files still load; round trip; damaged file rejected |
| CMB-04 | One hostile creature definition (data) and its state machine: idle, chase, attack, dead | `CreatureDef`, `CreatureState` | chases within sight, attacks in reach, stops when dead; no walking through solid voxels (uses the same collision query as the player) |
| CMB-05 | Creature spawn in the loaded world (deterministic from seed + brick, not saved while untouched) and despawn when far | spawn rule | same seed same spawns; far creatures removed; resident creatures bounded |
| CMB-06 | Loot: a drop table (data) rolled with a seeded source; the item enters the inventory by the existing inventory operation, refused when full without losing it | `LootTable` | drop equals table for a seeded roll; full inventory leaves the loot on the ground (or defers) per a stated rule |
| CMB-07 | Combat through the reconcile pattern: loot and damage taken are effects of confirmed host events, not of client claims | event ids | restart after a kill does not drop loot twice (same intent/confirm pattern as M1-A2) |
| CMB-08 | Adapter view: `GameView`/`SessionPort` carry health, target, hit feedback (no logic) | port fields | headless: fields equal the model |
| CMB-09 | Godot: attack input, health bar, creature mesh (simple), hit flash, sound hooks | GDScript glue only | real Godot smoke (`walk_smoke.py --combat`, to be written): kill one creature, loot appears, restart keeps health |
| CMB-10 | End-to-end combat test headless and in Godot | | the loop of section above passes twice in a row; features registry updated (new ids F-GPL-xxx, F-CRT-xxx created at that time, availability DEMO_AVAILABLE at most) |

Dependencies: CMB-02 needs CMB-01; CMB-04 needs CMB-01 and the collision query (exists in `cube-client::walk`); CMB-06 needs the inventory (exists); CMB-07 needs M1-A2 (done); CMB-09 needs CMB-08. CMB-01..03 start now.

## Epic E-WGN2: first biomes and caves (M3)

| Task | Action | Acceptance |
|---|---|---|
| WGN2-03 (DONE as terrain-v4, F-WGN-005) | trees and forests | see the registry |
| WGN2-02 (DONE as terrain-v3, F-WGN-004) | caves and ores | see the registry |
| WGN2-01 (DONE as terrain-v2, F-WGN-003; later slices are versions 3, 4, 5, never edits of v2) | generator version 2 in the existing `cube-worldgen` registry (never edit v1 in place): climate fields (integer noise), 2-3 biomes with blended borders | pinned hashes; v1 worlds still open and regenerate identically; order and thread independence tests like v1 |
| WGN2-02 | caves (3D integer noise carve), ores | same as above; ground stays continuous across brick borders; no open holes at the surface by accident |
| WGN2-03 | trees and simple vegetation as deterministic features crossing brick borders | oracle equality across borders |
| WGN2-04 | POIs: camp or ruin as a structure placed deterministically | same |
| WGN2-05 | new worlds use v2, existing worlds keep their version (record) | a v1 world opens unchanged |
| WGN2-06 | Godot: biome colours for materials, look at it | person or picture check; TypeSafe optional, advisory only |

WGEN-02..04 (service, materialiser) remain gated by GOV-21/GOV-11; this epic needs neither because it only adds a generator version to the registry.

## Later epics (epic level only; product decisions pending)

E-RPG (M4): XP and levels, stats, one specialisation, ability data, weapon types, armour slots, loot tiers. Needs ODM-05.
E-CRAFT-BUILD (M5): recipes and stations; sculpt/boolean tools over the Editing API (PS-6), history (PS-7, GOV-25), composite (PS-8), micro-voxel editing (PS-9).
E-LIVING (M6): creature set, behaviour, spawn rules, NPCs, settlements, day/night, weather.
E-ADVENTURE (M7): dungeons, bosses, quests, collections.
E-NET (M8): protocol and authority, dedicated host, replication, reconnect, interest management; start with the intent/validated-result shape already used by E-CMB.
E-MOBILE (M9): Android build and spike, touch input, adaptive UI, budgets; start the spike as soon as a device exists.
E-SOCIAL, E-SCALE, E-POLISH (M10..M12).

## Parallel workstreams and areas of ownership

| Stream | Area | Next tasks | Conflict rule |
|---|---|---|---|
| World and persistence | `cube-server`, `cube-persistence` | M1-A1 spike, M1-A3 measurement | touches journal contract only with the persistence path |
| Gameplay and combat | `cube-app/src/{game,combat,creature}.rs` | CMB-01..03 | does not touch `cube-server` internals |
| WorldGen content | `cube-worldgen` | WGN2-01.. | adds versions, never edits a pinned one |
| Client and Godot | `cube-client`, `cube-godot-adapter`, `godot/` | CMB-08/09 after the model | GDScript glue only, no logic |
| Geometry and building | backends, editing | none before M5 planning decisions | Editing API only |
| Networking, mobile | none yet | contracts, spike when a device exists | |
