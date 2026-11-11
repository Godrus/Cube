# M2 First Playable: scope

**Status:** **Proposed.** Every requirement, content item and criterion below is a [NP] proposal for the owner's review (labels as in `GAME_TRACK_ROADMAP.md`). Nothing here is approved, and nothing authorises implementation or the start of PS-1.
**Definition of the target.** A *minimum playable game*, not a technical demo: one complete, repeatable gameplay loop with deliberately limited content, played by a person through the real client on one desktop platform, on top of the authoritative runtime the architecture requires.

## 1. The player-experience statement

> A player starts the game, creates a new world, steps into it as a character seen from behind and above, runs and jumps across a procedurally generated landscape with two kinds of terrain, meets a hostile creature, fights it with a weapon, takes what it drops, equips a better weapon and feels the difference in the next fight, digs and builds with the terrain, saves, quits, returns, and finds the world, the character and the belongings as they were left.

## 2. The thirteen proposed requirements, evaluated

| # | Requirement | Verdict | Rationale | If deferred, what First Playable would mean |
|---|---|---|---|---|
| 1 | Launch the game | **In** | entry point of every session | nothing to play |
| 2 | Create a new world | **In, minimal**: world name and seed (text or number), no other options | deterministic generation is the core promise; a fixed test world would hide generator and persistence problems (`DEVELOPMENT_ROADMAP.md` allowed a fixture; this proposal removes that option) | a demo of a pre-made level, not a procedural game |
| 3 | Enter the world | **In** | | |
| 4 | Control a character in third person | **In** | a visible character and a follow camera carry combat readability and the identity of the game; first person would not exercise a character model | a fly camera = tech viewer |
| 5 | Move, jump, interact | **In**: walk/run, jump, one interact action (pick up / use), collision with voxel terrain | no jump = no terrain traversal; interaction opens pickup and use | a walking simulator without agency |
| 6 | Explore a small but meaningful generated region | **In, defined as**: streaming around the player; at least **two** terrain/biome variants reachable on foot; resource deposits and creatures placed by the generator | "small" is bounded by content, not by an artificial wall; the key range decision D-K gives a hard outer bound | nothing to discover |
| 7 | Encounter at least one creature type | **In: two types** (one hostile, one passive) | one hostile type is the minimum for combat; one passive type tests that creatures are not all enemies and gives a second drop source | combat has no target |
| 8 | Meaningful combat interaction | **In, minimal** (section 4.3): attack action, hit detection, damage, creature death, creature attacks back, player can be defeated and respawns | "meaningful" needs risk both ways; without the creature's attack there is no tension | an interactive sandbox, not an RPG |
| 9 | Collect a resource or item | **In**: block resources from digging, a creature drop, a weapon drop | the reward step of the loop | no reason to fight |
| 10 | Modify the voxel environment (destroy and place) | **In** through the Editing API | pillar P-3; also the first real consumer of the production edit path | a game without the Cube twist |
| 11 | Save the world | **In** | | progress is worthless |
| 12 | Exit and reload the world | **In** | proves the persistence design with real play data | |
| 13 | Continue with the saved state preserved | **In**: terrain edits, player position, inventory, equipment, stats; creatures are re-spawned by rule (section 5) | | |
| + | Added by this review (needed to close the loop): **equip an item with a stat effect**, **inventory**, **death and respawn**, **minimal HUD and menus** | In | step 6 of the loop ("improve") cannot exist without them; they are small | no reason to collect |

## 3. Minimum content (counts are scope proposals, not performance budgets)

| Area | M2 proposal | Notes |
|---|---|---|
| Playable character | **1**, placeholder model, no class choice; name entry optional | class system is out |
| Environments | **2** terrain variants (for example open grassland and a rocky/sandy variant), with a visible transition; deterministic from the seed | the spike has one biome; a second is new generator work |
| Voxel materials | about **6** (the spike has 4: grass, dirt, stone, player-placed; add 2 more such as sand and a resource material) | exact set is a design choice |
| Resources | **2-3** (for example stone and dirt from digging; one ore-like material in rocks) | gathering = destroying voxels, no tools required |
| Creatures | **2**: hostile melee chaser; passive wanderer that yields a resource | simple behaviours (section 4.4) |
| Items | **about 6-8 in total**: 2-3 block resources, 1 creature drop, **2 weapons** (basic, better), **1 consumable** (restores health) | no crafting in M2 |
| Equipment | **1 slot** (weapon); optional second slot as stretch | the weapon changes the attack stat |
| Inventory | small fixed-size slot grid with stacking for resources | size is a design parameter |
| Character stats | **health**, **attack**; defence optional | placeholders tuned by play |
| Progression | **reward feedback** through loot and the better weapon; optional stretch: experience bar and level-ups that raise maximum health | not required for acceptance |
| Combat encounter | **1** basic: the hostile creature type placed in the world, groups of one to a few | no bosses |
| World interaction | pick up items, destroy and place voxels, use a consumable | |
| Save/load | **one** world save; autosave on exit optional | single local slot per world |
| UI | main menu (new world, continue, quit), pause menu (save and quit), HUD (health, target health, hotbar/inventory panel, interaction prompt) | placeholder art |
| Audio | none required; stretch: 3-5 placeholder sounds (hit, pickup, hurt) | |

**Not designed for M2:** dozens of classes, hundreds of items, complex quests, an economy.

## 4. Gameplay rules at the level needed to build and test (all tunable data, no values fixed here)

### 4.1 Character
Fixed-step movement with collision against the voxel terrain, gravity, jump; stepping up one voxel; third-person orbit camera that follows the character and does not go inside terrain; input expressed as *intents* (move, look, jump, attack, interact, use) so that keyboard/mouse now and gamepad or touch later map to the same intents (roadmap-v1 gate 6; `CLI-04`).

### 4.2 World interaction
A reach-limited ray from the camera/character selects a voxel; destroy and place requests are sent to the authoritative runtime as operations through the Editing API; the player cannot change the world locally without the server accepting it (AOD-14, AOD-15).

### 4.3 Combat (minimal)
One attack action with a short cooldown; hit detection against a creature's hit volume at the time of the attack; damage = attacker's attack stat modified by tunable data (minimum 1); creatures have health and die; the hostile creature notices the player within a radius, approaches, attacks with a short wind-up; the player has health; at zero health the player respawns at the world spawn with the inventory kept; a consumable restores health. No blocking, dodging, skills, status effects or ranged attacks in M2 (stretch: a dodge).

### 4.4 Creatures
A small state machine per creature kind: idle -> wander -> (hostile only) notice -> chase -> attack -> return; a passive kind: idle/wander/flee. Spawn rules are deterministic functions of the world seed and position, applied at chunk load; creatures are removed on unload; loot rolls come from a seeded generator derived from the world seed and an event identifier so that results can be replayed in tests (no ambient randomness).

### 4.5 Items and inventory
Item definitions are data (identifier, name, stack size, kind, stat modifiers); inventory is a server-owned list of stacks; equip changes the derived stats; items are persisted by identifier and count.

## 5. Temporary and simplified systems

Rule [NP]: a simplification is acceptable only if the *interface* that the full version needs is already the one used (so replacement does not ripple), and if its replacement condition is written down. No temporary solution may become a hidden permanent dependency; each one has an entry in a *simplification register* (integration document section 6).

| Simplification | Why acceptable | Must preserve | Must not prevent | Replace when | Full version needed at |
|---|---|---|---|---|---|
| Placeholder character model | art is not the test of the loop | a replaceable model slot, stable attachment points for equipment | later skeletal animation, equipment visuals | art direction chosen | M3 |
| Placeholder animations (idle/walk/attack as simple transforms) | readability only | event timing of the attack (hit frame) independent of the art | animation-driven combat timing | animation pipeline exists | M3 |
| Simplified creature AI (state machine) | enough for one chase-and-attack encounter | AI runs on the server, deterministic given seed and inputs | behaviour trees, navigation meshes, group behaviour | more creature kinds appear | M3 |
| Simplified combat (one attack, no skills) | proves the reward loop | server-side hit resolution, damage as data, events for the client | abilities, cooldown groups, damage types | M3 combat scope | M3 |
| Two terrain variants, no caves/rivers/structures | enough variety to test streaming and variant boundaries | biome as data in the generator, versioned generation | more layers, structures, large features | M3 content | M3 |
| Limited world size (the key range and streaming radius) | the first generator version only has to be correct | a stated world bound, no hard-coded limits in gameplay code | the final coordinate system (D-K RG-1) | after RG-1 | before M6 |
| Local-only persistence, one slot | proves save/restore | the journal/snapshot design (AOD-21), versioned records | cloud saves, multiple characters | M4+ | M5 |
| No multiplayer | outside the loop | **the embedded authoritative runtime and an in-process transport** (AOD-22), stable entity and player identifiers, intents as commands | adding sessions and remote clients | M4 | M4 |
| No mobile build | desktop first | input as intents, no desktop-only assumptions in gameplay, UI scalable; a written mobile-impact statement | touch controls, device budgets | before M3 start for a device checkpoint, M5 for support | M5 |
| One weapon slot, simple stats | the minimum power curve | item and stat data model that allows more slots | slots, sets, rarities | M3 | M3 |
| Hard-coded or file-based content tables | speed | data-driven definitions from day one (format not decided here) | designers editing content later | when the data format is approved | M3 |
| Placeholder UI | function over look | server authority for every number shown; UI reads views only | final UI, localisation | UI direction | M3 |
| Creature kills not persisted (respawn by rule) | avoids a persistence format for entities | entity identifiers and an extension point for entity records | persisted world entities | M3/M4 | M4 |
| Loot as a seeded roll on a short table | enough to show variance | determinism and replayability | drop tables, rarity, economy | M3 | M4 |

## 6. Explicit exclusions

| Item | Required for M2? | Why excluded / what must stay possible |
|---|---|---|
| 500-player logical worlds | no | load tests belong to M6; keep: single-writer authoritative tick, interest-based replication hooks |
| Production multiplayer | no | M4; keep: embedded host with in-process transport and the same semantics |
| User-hosted servers | no | M4+; keep: runtime separable from the client process |
| Full mobile support | no | M5 device gates; keep: input intents, no desktop-only gameplay code, mobile-impact statement (AOD-08) |
| Full cross-play | no | M6 |
| Practically infinite world coordinates | no | the key bound of D-K applies; keep: bound stated, key widths named constants |
| Advanced micro-voxel sculpting | no | PS-6/PS-9; the edit API must not assume a single resolution |
| All geometry representations (SDF, mesh) | no | M2 uses the voxel representation; keep: Core's closed representation enum and ports |
| Full class system | no | one character, no classes |
| Complex quests | no | M3/M4 |
| Dungeons and raids | no | M3 maybe one point of interest; M4+ |
| Full economy | no | |
| Advanced crafting | no | M2 has none; M3 may add simple crafting |
| Complete world simulation | no | creatures only near the player |
| Undo/redo of world edits | no | History (PS-7) is not needed by the loop; note that the production plan nevertheless pulls History tasks into the M2 closure (dependency document) |
| Tools such as sculpt/Boolean, composite assets, resampling | no | PS-6, PS-8, PS-9 |
| Accounts, chat, parties, guilds, moderation | no | M4+ |

Features essential to the loop are **not** excluded: inventory, equipment effect, creature attack, respawn and HUD are in.

## 7. Acceptance criteria and demonstrations

Each criterion is a *player-facing* check with a recorded demonstration. A check marked **S** is a scripted run (automatable), **M** is a manual run by a person, **O** needs an outside tester. Performance numbers are **not** criteria: they are measured and reported on named hardware; thresholds are set by the owner after measurement (`RELEASE_CRITERIA.md` R0.2).

| ID | Criterion | Demonstration | Kind |
|---|---|---|---|
| FPS-01 | Launch and menus | start the build; reach the main menu; start, quit | S/M |
| FPS-02 | New world by seed | create a world with seed X twice; the generated terrain is identical (chunk hashes equal) and differs for seed Y | S |
| FPS-03 | Control | walk, run, jump, bump into a wall, climb a one-voxel step, fall; the camera follows without entering terrain | M |
| FPS-04 | Exploration | a defined route crosses chunk borders and the border between the two terrain variants; no holes, seams or missing chunks | S+M |
| FPS-05 | Encounter | find a hostile and a passive creature on the route | M |
| FPS-06 | Combat | win a fight; lose a fight and respawn; creature reacts to the player | M |
| FPS-07 | Loot and inventory | the defeated creature drops an item; pick it up; it appears in the inventory; stack resources | S/M |
| FPS-08 | Equipment effect | equip the better weapon; the stat changes; the same fight is measurably shorter (recorded) | S/M |
| FPS-09 | World modification | destroy and place voxels; every modification is an Editing API operation (commit log shows `UserCommit`; archcheck/compile-fail evidence when the production tests exist) | S |
| FPS-10 | Save | save through the menu | S |
| FPS-11 | Reload and continue | quit, relaunch, continue: edits, position, inventory, equipment, stats are as before; play on | S/M |
| FPS-12 | Crash during save | kill the process at each step of the AOD-21 compaction protocol; the next launch recovers a verified state, never an unverified snapshot | S |
| FPS-13 | Authority | no gameplay rule lives in GDScript; the client cannot change world or inventory state locally (adapter audit, archcheck) | S |
| FPS-14 | Understandable and repeatable loop | an outside tester plays 20+ minutes without instructions; the facilitator records: could they discover the controls, did they fight, did they equip, did they build, did they voluntarily start a second fight | O |
| FPS-15 | Declared absences | the milestone report lists exclusions of section 6 and the simplification register | M |
| FPS-16 | Measured, not budgeted | frame time, edit latency, save and load time, memory are measured on the named desktop hardware and reported (B-5, B-6, B-10) | S |
| FPS-17 | Mobile impact | a written statement per Game Track workstream of what would block a mobile build | M |

**Definition of done for M2:** all FPS criteria demonstrated with recorded evidence, the owner records acceptance in the decision registry, and the milestone is then (and only then) marked reached in `PROJECT_STATUS.md`.

## 8. Mobile impact statement (AOD-08)

M2 is accepted on desktop. Every M2 system must be written so that a mobile client is a client-side change: authority and rules in Rust on the server side, input through intents, no OS process calls (AOD-12), no assumption of a large display or a mouse in rules. A physical-device smoke run is proposed as a checkpoint before M3 (conflict C-3 in `GAME_TRACK_ROADMAP.md`).

## 9. Assumptions and open questions

* The recorded decisions Rust and Godot 4.x (AOD-01/02/03) await the owner's countersignature (X-1). The scope above is engine-neutral except where the architecture already fixes the split (rules in Rust, client for presentation).
* The M2 build depends on owners that do not exist yet (client, engine adapter, server, WorldGen, networking, gameplay) and on two missing external documents (dependency document section 4).
* Tuning values (damage, health, cooldowns, spawn density, drop chances) are not set; they are data to be tuned in play.
* The numbers of content items are proposals; the owner may raise or lower them without changing the structure.
* The expansion of M2 versus its current definition is conflict C-1; the owner decides whether the old M2 survives as an internal checkpoint (M2-T).

## 10. Changes this proposal makes to existing milestone text (none applied)

| Document | Existing | Proposed |
|---|---|---|
| `DEVELOPMENT_ROADMAP.md` M2 | six verbs, test world, "what is missing" lists combat, creatures, items | the scope of sections 1-7 |
| `RELEASE_CRITERIA.md` 2.1 | FP-1...FP-10 | FP-1...FP-9 map to FPS-01...FPS-13; FP-10 becomes FPS-15; FPS-14, FPS-16, FPS-17 added |
| roadmap-v1 gate 6 exit | desktop and a physical Android device | desktop for M2; device checkpoint before M3 |
