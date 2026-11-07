# Cube Game Track Roadmap

**Status:** **Proposed.** Nothing in this document is an owner decision, a contract approval or a change of the Production Plan. It does not authorise implementation or the start of PS-1.
**Source state:** branch `docs/architecture-roadmap-v1`, HEAD `5c9a5c5` (verified), PS-0 open (0 of 8 tasks), PS-1 not started, milestone M0 in progress (`docs/PROJECT_STATUS.md`).
**Companions:** `FIRST_PLAYABLE_SCOPE.md` (M2), `FIRST_PLAYABLE_DEPENDENCIES.md`, `VERTICAL_SLICE_SCOPE.md` (M3), `TECHNOLOGY-GAME-TRACK-INTEGRATION.md`.

**Labels used in all Game Track documents** (so that approved, proposed and new things are never mixed):
* **[REC]** a decision recorded from the owner (AOD/OAM) that still awaits the owner's countersignature (plan 3.3); treated as a constraint, not as a confirmed registry entry.
* **[EP]** an existing proposal in the repository (not approved).
* **[NP]** a new proposal made in this task. Every [NP] needs the owner's review.

## 1. Why there is a Game Track

Technology exists to enable the game. The Production Plan v2.1 (147 tasks, PS-0...PS-14) is deliberately a *geometry and runtime* plan: it states that gameplay, content and MMO scope are outside it (`production-architecture-plan.md` section 12 and the "outside this plan" row for roadmap-v1 gate 6) and `DEVELOPMENT_ROADMAP.md` lists the unplanned packages G-1...G-13. Left alone, the plan can be completed in full without a single enemy to fight. The Game Track fills that gap and is measured by what a player can do, not by task counts.

| Track | Content | Measured by |
|---|---|---|
| Technology | engine/runtime, geometry, world generation, rendering, physics, persistence, networking, server, platforms, performance | ADG gates, contract tests, benchmarks on target hardware |
| Game | character and controls, camera, exploration, combat, creatures and AI, items and equipment, RPG progression, biomes and world content, building and destruction, activities, gameplay loop, player experience | player-facing demonstrations and playtests (section 7) |

[NP] **Principle:** the tracks are independent in ownership and coordinated in time. The Technology Track provides *minimal, stable capabilities early*; the Game Track consumes them as soon as they exist and feeds back requirements; both converge at M2 (First Playable) and M3 (Vertical Slice). Neither track may declare a milestone by its own task counts.

## 2. What exists and what is missing (verified)

| Item | State |
|---|---|
| GDD/TDD 0.3 | **absent** (`docs/README.md`, blocker B-1). AOD-24 candidate baseline BL-2 would use AOD-01...24 as product constraints in its place; BL-2 is a candidate, not approved |
| Game concept in the repository | only the project requirements list (`docs/PROJECT_STATUS.md` 5.1), the reference studies (`docs/research/*`) and the roadmap-v1 gates 6 and 8; no character, combat, item or progression design |
| Production code | none; the spike has terrain generation with one biome, chunk streaming, voxel edit, a Godot client with a marker that is moved by buttons, HUD, save/load (`docs/spike/*`, `godot-client/world.gd`: no controller, no collision, no inventory) |
| Gameplay owner | **unassigned** (roadmap-v1 gate 6 "unassigned gameplay owner (OI-8)"); not among the owners GOV-03 must decide (WorldGen, Networking, Server, Client, Engine Adapter, Import/Export, O-1, O-2) |
| Content, art, audio owners | none named anywhere |
| Gameplay module in the module grid | none (`module-boundaries.md` Part A/B); the server is the composition root |

## 3. Cube's identity [NP]

**Inspirations as recorded in the references (not copied):** early Cube World (exploration, discovery, RPG progression, encounters), Veloren (open-source voxel technology, procedural world, multiplayer architecture), Trove (classes, dungeons, bosses, collections, building, long-term progression). Policy [REC] AOD-23: no code or assets are copied; GPL projects are architectural references only; CubeWorld-Reversal is research material only; ideas are re-thought and implemented independently.

**Cube's own pillars (proposal, to be confirmed or changed by the owner):**

| # | Pillar | Meaning for play | Why it is Cube's own |
|---|---|---|---|
| P-1 | A world worth walking through | a seamless, deterministic procedural world that rewards curiosity with landmarks, resources and encounters | the technology goal (endless seamless world) is the product's first promise |
| P-2 | Every fight pays | readable, short combat whose outcome visibly changes the character (loot, equipment, stats) | ties the RPG progression to concrete encounters |
| P-3 | The world remembers you | placing and destroying voxels is real gameplay, and every change persists | the Universal Geometry and persistence work becomes something the player feels, not a hidden library |
| P-4 | Together in one world (from M4) | co-op/multiplayer in the same persistent world | server-authoritative design [REC] AOD-13/14/22 |
| P-5 | Everywhere you play (long-term) | PC and mobile as peers; cross-play | [REC] AOD-08 (mobile is mandatory, not a late port) |

**Distinguishing hypotheses to test at M2/M3 (not requirements):** H-1 terrain is a tactical tool in combat (digging, walling, high ground); H-2 resources found while exploring feed both building and equipment, so exploration, fighting and building share one economy of materials; H-3 a small, fast loop is more motivating in the first hour than a deep skill system. M2 is designed to test H-1 to H-3 cheaply.

## 4. Core gameplay loop [NP]

```
Enter the world -> Explore -> Discover (location, resource, creature) -> Fight / interact
      ^                                                                      |
      |                                                                      v
 New opportunities  <- Modify / build the world <- Improve character and equipment <- Rewards
```

| Step | Player verb | Systems | M2 | M3 | M4+ |
|---|---|---|---|---|---|
| 1 Enter | start / load a world | menu, world creation, load | **mandatory** | polish | accounts, servers |
| 2 Explore | walk, jump, look around | character controller, camera, terrain streaming | **mandatory** | more biomes, points of interest | larger, richer world |
| 3 Discover | notice resources, creatures, landmarks | generation features, visibility, UI cues | **mandatory (minimal)** | points of interest, biome variants | structures, dungeons |
| 4 Fight / interact | attack, avoid, gather | combat, creature behaviour, interaction | **mandatory (1 encounter type)** | several creature types, abilities | bosses, groups |
| 5 Rewards | pick up loot, resources | drops, inventory | **mandatory** | variety, rarity | economy, collections |
| 6 Improve | equip, use, level | equipment, stats, progression | **mandatory (minimal: one slot + stat effect)** | slots, levels, build choices | classes, long-term progression |
| 7 Modify | place and destroy voxels, build | editing, persistence | **mandatory (place/destroy)** | structures, building aids | shared building, permissions |
| 8 New opportunities | go further, try again | content hooks | **implicit** (better gear lets the player reach and win more) | explicit goals | quests, activities |

**Why the loop should bring players back [NP, hypotheses to validate by playtest]:** curiosity (the next hill may show something new), risk and reward (a fight has a tangible payoff), a visible power curve (the character that returns is stronger), agency (the world can be changed and the change stays), and, later, shared play. The loop is not a promise of fun; M2 exists to find out whether the first twenty minutes are engaging with minimal content (section 7, criterion L-1).

## 5. Milestones from the player's point of view

The release milestones M0...M8 already exist in `docs/DEVELOPMENT_ROADMAP.md` and `docs/RELEASE_CRITERIA.md` (both Proposed). This section adds the game view. Differences from the existing definitions are collected in section 9 and are **proposals**.

| Milestone | Player experience | Scope document |
|---|---|---|
| **M2 First Playable** | one coherent loop with limited content: create a world, explore, fight one kind of enemy, collect, equip, change the world, save, reload, continue | `FIRST_PLAYABLE_SCOPE.md` |
| **M3 Vertical Slice** | a small, representative piece of the final game: several environments, creature types, items, progression, building, persistence, a visual and audio direction | `VERTICAL_SLICE_SCOPE.md` |
| **M4 Internal Alpha** | broader systems and content; sustained playability for hours; multiplayer on dedicated and embedded servers | existing M4 + gameplay breadth (unplanned packages G-1...G-13 get plans) |
| **M5 Closed Alpha** | controlled external testers; telemetry; update and recovery | existing M5 |
| **M6 Beta** | feature-complete or nearly; stabilisation; platforms and cross-play | existing M6 |
| **M7-M8 RC, 1.0** | scope freeze; release production | existing M7, M8 |

## 6. Organisation and governance constraints [REC]/[EP]

* Authoritative state lives on the server, the client is never the source of truth (AOD-13/14); single-player runs the same authoritative runtime in-process (AOD-22). **Gameplay rules (combat, inventory, creatures) are therefore authoritative Rust logic, not engine scripts** (AOD-07: GDScript is UI and glue only).
* User changes to geometry go through the Editing API (AOD-15); procedural generation goes through the WorldGen/System boundary (AOD-16/20).
* Mobile impact must be stated for every gate that touches runtime (AOD-08); an emulator never replaces a device (AOD-11).
* Contracts stay Draft until the owner approves them (AOD-18); no production module may rely on a Proposed contract (plan 3.1), see the open problem H-3 in `docs/gov01/GOV-01-DECISION-REGISTER.md`.
* **Owners needed for the Game Track [NP]:** gameplay/rules owner (OI-8, unassigned), content/design owner, art and audio owner, game-UI owner. They should be added to the GOV-03 list or deferred in writing.
* **Placement of gameplay in the architecture [NP]:** the module grid has no gameplay module. Proposal: a server-side *Game Rules* module (name to be chosen) above the Editing API ports and below the server composition root: it owns entities, combat, inventory, items and progression rules; reads the world through Core/Processing read views and changes terrain only through `IEditingService`. Adding it changes ADG Parts A/B and needs the owner and the Integration Owner (not done here).
* The entity model (ECS or explicit domain model, REF-13 "Needs spike") is open (roadmap-v1 gate 6, open decisions). It must be decided by a spike before the Game Rules module is built.

## 7. Product progress controls [NP]

Progress is **not** measured by task counts, contract counts, test counts, module counts or documents. The Game Track reports on **player-facing evidence**, each backed by a recorded demonstration.

| Question | Evidence (recorded demonstration) | Milestone |
|---|---|---|
| Can the player enter and leave a world? | scripted run: menu -> new world -> in world -> exit to menu -> exit | M2 |
| Can the player explore without broken terrain? | walk a defined route across chunk borders and the generator's biome boundary; screenshots every N steps; no holes, no seams | M2 |
| Can the player fight an enemy and does it pay? | scripted + manual: engage, win, receive the drop; engage, lose, respawn | M2 |
| Can the player improve the character? | pick up an item, equip it, observe the stat change and a changed outcome of the next fight | M2 |
| Can the player change the world and does it persist? | destroy and place voxels, save, quit, reload, same voxels | M2 |
| Can the player return and continue? | reload the save, inventory, position and stats preserved | M2 |
| Is the loop understandable and repeatable? | an outside tester plays without explanation; the facilitator records where they stopped, what they said, how many loops they repeated by choice | M2 (first reading), M3 (full) |
| Does the world feel varied and purposeful? | per-environment tour with points of interest; checklist from the Vertical Slice scope | M3 |

**Lightweight reporting rule:** each month-independent increment ends with a *demo build* and a one-page *playtest note* (what the player could do, what broke, what they said). The status document records, for each player-facing question, one of: *not demonstrable*, *demonstrated by developers*, *demonstrated by an outside tester*. A milestone is reached only when the owner records acceptance (`RELEASE_CRITERIA.md` section 4). **No performance budgets are invented:** timing, frame rate and memory are *measured and reported* on target hardware (benchmarks B-5, B-6, B-10 of `RELEASE_CRITERIA.md`) and become thresholds only by the owner's approval.

## 8. Mapping to the unplanned packages of the development roadmap [NP]

| Roadmap gap package | Game Track workstream | First needed |
|---|---|---|
| G-1 character and controller | GT-1 character, controls, camera | M2 |
| G-2 combat | GT-2 combat | M2 (minimal) |
| G-3 creatures/AI | GT-3 creatures and behaviour | M2 (minimal) |
| G-4 items/equipment | GT-4 items, inventory, equipment | M2 (minimal) |
| G-5 RPG progression/classes | GT-5 stats and progression | M2 (minimal), M3 |
| G-6 world content | GT-6 biomes, resources, points of interest | M2 (two biome variants), M3 |
| G-7 UI/HUD | GT-7 menus, HUD, inventory UI | M2 |
| G-9 audio/VFX/art pipeline | GT-8 placeholder art/audio pipeline, then direction | M2 (placeholders), M3 (direction) |
| G-8 online services | out of Game Track until M4 | M4 |
| G-10...G-13 telemetry, legal, localisation, anti-cheat | later | M5+ |

## 9. Consistency with existing documents (Part H)

Differences are listed so that nothing is silently rewritten. "Existing" = the cited document as committed; "New" = a proposal of this task.

| # | Existing statement | New proposal | Impact / handling |
|---|---|---|---|
| C-1 | `DEVELOPMENT_ROADMAP.md` M2: launch, create **or load a test world**, move a character, interact with voxels, edit, save/load; "what is still missing": combat, creatures, items, progression | M2 additionally requires a third-person character with jump, one hostile creature type with combat, loot, one equipment effect, inventory, new-world creation by seed | M2 becomes larger; the old M2 acceptance rows FP-1...FP-9 (`RELEASE_CRITERIA.md`) remain a subset; FP-10 ("missing items listed") is replaced by the exclusion list of `FIRST_PLAYABLE_SCOPE.md`; the owner decides whether to keep the old M2 as an internal checkpoint **M2-T** (technical slice, walk and edit, no combat) |
| C-2 | `DEVELOPMENT_ROADMAP.md` M2 test-world source: PS-11 slice **or** a fixed fixture | New: world creation uses generation, so a WGEN slice is mandatory | fixture alternative removed for M2 |
| C-3 | roadmap-v1 gate 6 exit: playable local loop **on desktop and a physical Android device** | New: M2 acceptance on desktop; a physical-device checkpoint is scheduled before M3 and is non-blocking for M2 | needs the owner's ruling (AOD-08 makes mobile mandatory, AOD-11 forbids claiming it without a device) |
| C-4 | `DEVELOPMENT_ROADMAP.md` R-02: no gameplay stages; "until GDD/TDD is supplied M3+ are scope Unknown" | New: a scope proposal for M2 and M3 built from the recorded constraints and the references | a proposal only; the missing GDD/TDD remains a blocker for final scope approval |
| C-5 | `RELEASE_CRITERIA.md` M3 rows VS-1...VS-7 | `VERTICAL_SLICE_SCOPE.md` section 8 maps them and adds player-facing demonstrations | extension, not a replacement |
| C-6 | ADG has no milestone slices (R-01) | `FIRST_PLAYABLE_DEPENDENCIES.md` shows that under the published ADG the M2 slice pulls 100 of 147 tasks and two external inputs | needs a slice definition (proposal there) |
| C-7 | module grid has no gameplay module; GOV-03 has no gameplay/content owner | proposals in section 6 | ADG and owner decisions needed |
| C-8 | AOD-07 GDScript is UI and glue only | prototypes in engine scripts are allowed only as labelled, non-production experiments (integration document, level 2) | no conflict if the rule is kept |
| C-9 | AOD-24 BL-2 (AOD-01...24 stand in for the GDD/TDD) is a candidate baseline | the Game Track relies on it for scope; the reliance is written here | the candidate needs approval before any scope document is called approved |
| C-10 | `PROJECT_STATUS.md` M0 in progress, no milestone reached | unchanged | no change |

## 10. Open questions for the owner (Game Track)

1. Confirm or change the pillars of section 3 and the loop of section 4.
2. Accept the expanded M2 (C-1) and decide on the internal checkpoint M2-T.
3. Desktop-only M2 acceptance with a device checkpoint before M3 (C-3), or a device in M2.
4. Owners: gameplay, content, art/audio, game UI; add them to GOV-03 or defer in writing.
5. Entity model spike (REF-13) and the Game Rules module placement (section 6).
6. Supply or replace the GDD/TDD, the Voxel Architecture and the Geometry Processing Specification: M2 on the published ADG depends on the last two (`FIRST_PLAYABLE_DEPENDENCIES.md` section 4).
