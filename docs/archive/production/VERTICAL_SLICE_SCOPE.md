# M3 Vertical Slice: scope

**Status:** **Proposed** [NP]. Counts and features are scope proposals for the owner, not decisions; nothing authorises implementation. Labels as in `GAME_TRACK_ROADMAP.md`. Existing criteria: `RELEASE_CRITERIA.md` section 2.2 (VS-1...VS-7, also Proposed).

## 1. Purpose

M2 shows that *one loop works with minimal content*. M3 shows that *Cube as a game* works: a small piece that is coherent and representative of what the finished game is meant to be, in a form that could be shown to people outside the project and from which the effort of the full game can be estimated.

## 2. What M3 must prove that M2 does not

| # | Question M3 answers | Why M2 cannot |
|---|---|---|
| Q-1 | Does the loop stay engaging over a **full session** (an hour or more, owner to set the length), not just the first twenty minutes? | M2 content is exhausted quickly by design |
| Q-2 | Does the world feel **varied and purposeful** (several environments, points of interest, reasons to travel)? | M2 has two terrain variants and no destinations |
| Q-3 | Does **progression** have a shape (early, middle, end of the slice) and do choices matter? | M2 has one weapon upgrade |
| Q-4 | Do **combat** and **creature variety** hold up (different threats, an elite or boss, abilities)? | M2 has one basic encounter |
| Q-5 | Does **building and destruction** become a gameplay system (structures, materials, consequences), not just place/destroy? | M2 only edits voxels |
| Q-6 | Do **persistence** and recovery cope with a realistic amount of play state (world edits, character, inventory, progress, quest state)? | M2 state is small |
| Q-7 | Is there a believable **visual and audio direction** and a content pipeline that artists and designers can feed? | M2 uses placeholders |
| Q-8 | Can the **technology scale** from the M2 slice to this content (generation features, entity counts, streaming) on the chosen desktop hardware and, if the owner includes it, on a physical Android device? | M2 measures a small case |
| Q-9 (optional) | Does **co-op** work in the same world (two players, one authoritative host)? | M2 is single-player |

## 3. Proposed scope

Counts are proposals (design volume, not performance budgets).

| Area | M3 proposal | Builds on (M2) |
|---|---|---|
| Region | a generated region of meaningful extent with **5-8 designed points of interest** (landmarks, camps, ruins, resource sites) placed by seeded rules | two terrain variants, no destinations |
| Environments | **3** biome/terrain variants including transitions, water or equivalent obstacle, vegetation or equivalent landmarks | two variants |
| Creatures | **4-5** kinds: 2 hostile melee/ranged types, 1 elite or boss at a point of interest, 1-2 passive types; group spawns | 2 kinds |
| Combat | 3-4 abilities or attack types, a dodge or block, a clear damage model, readable feedback and sound | one attack |
| Items | **about 20-30** items: block resources, creature parts, tools, consumables, weapons, armour; **3 rarity levels** | 6-8 items |
| Equipment | **3 slots** (weapon, body, one accessory) with stat effects | 1 slot |
| Progression | levels over the length of the slice; a few stats; **one meaningful choice** (two specialisations or a small talent set) | stats only |
| Gathering and crafting | gather by digging and harvesting; a **simple crafting station** with about 5-10 recipes | none |
| Building and destruction | place/remove with materials gathered; at least one **structure type or blueprint**; destruction has gameplay consequences (e.g. cover, access, hazards); undo of building (History, PS-7) if the owner wants it | place/destroy |
| Activities | one **short quest chain** (3-5 steps) and one **point of interest with a boss encounter** ("dungeon-lite") | none |
| Persistence | world edits, character, inventory, quest state, discovered points; save/load/recovery as in M2 plus migration of the M2 save | world, inventory, stats |
| Visual direction | an agreed style guide, a representative character and creature set, lighting/sky/water treatment | placeholders |
| Audio direction | music loop(s) and a sound-effect set covering combat, UI, ambience | none |
| UI | HUD, inventory, equipment, crafting, map or compass, quest tracker, settings | minimal HUD |
| Platforms | desktop; **a physical Android device checkpoint** if the owner includes mobile in M3 (otherwise desktop only, stated) | desktop |
| Multiplayer | **optional (decision D-MP):** two-player co-op via the embedded host plus an external transport (`SRV-02`, `SRV-05`, `NET-04`, GOV-12), included only if owners and technology exist; otherwise M4 | none |

## 4. Technology added beyond M2 (by ADG stage; none authorised)

| Need | ADG area | Note |
|---|---|---|
| richer generation: biome blending, features, points of interest, creature spawning as a separately seeded step | PS-11 beyond WGEN-01..04; roadmap-v1 gate 8 | generator version bump rehearsal |
| building aids and undo | PS-6 tools (partly), PS-7 History | optional |
| structures as reusable assets | PS-8 Composite | optional |
| more GPU/CPU capacity: LOD, greedy mesher | VOX-03, CLI-03 | only if measurements require it |
| gameplay layers: abilities, crafting, quests, AI beyond state machines, UI | Game Rules module and client UI (not in the plan) | owner and design needed |
| content pipeline: asset import, authoring data formats | Import/Export (unassigned O-4), data format decision | |
| device checkpoint | DEV-01, DG-1..DG-3 | requires hardware (NB-1) |
| co-op | PS-13 beyond the embedded host | D-MP |

## 5. Entry and exit

* **Entry (proposal):** M2 accepted by the owner (or the owner explicitly starts content pre-production earlier). Design work and asset production may begin before M2 acceptance, because they do not touch production code.
* **Exit:** the demonstrations of section 6 recorded, the owner's acceptance recorded; an updated estimate for M4 based on measured velocity, content costs and benchmark data (no dates are given in this repository; `DEVELOPMENT_ROADMAP.md` R-04 lists what an estimate needs).

## 6. Acceptance criteria and demonstrations (player-facing)

| ID | Criterion | Demonstration | Kind |
|---|---|---|---|
| VSG-01 | Full session | an outside tester plays a complete session without developer help; notes where they stopped, struggled or lost interest | O |
| VSG-02 | Exploration | the region contains the points of interest; a tester finds at least some without guidance; no broken terrain on the tour | M/O |
| VSG-03 | Environments | each environment is visited and recognised as different (screenshots, tester comment) | M/O |
| VSG-04 | Creatures and combat | each creature kind encountered; abilities used; the elite/boss defeated or the tester loses and retries | M |
| VSG-05 | Items and equipment | items found, equipped in all slots, stat effect visible in play | M |
| VSG-06 | Progression | levels gained over the session; the choice made; the effect observed | M |
| VSG-07 | Gathering, crafting, building | gather, craft, build the structure; destruction consequence shown | M |
| VSG-08 | Quest and point of interest | quest chain completed; boss encounter played | M |
| VSG-09 | Persistence and recovery | save, quit, reload mid-quest; crash during save recovers (crash matrix); the M2 save migrates | S/M |
| VSG-10 | Direction | style guide exists; representative assets in the build; audio present | M |
| VSG-11 | Authority and architecture | gameplay rules are server-side; client holds no authoritative state; archcheck/tests as in M2 | S |
| VSG-12 | Scalability evidence | generation, streaming, entity counts measured on named hardware; reported against the questions of Q-8; thresholds only after owner approval | S |
| VSG-13 (optional) | Co-op | two players, one world, edits and fights visible to both, recovery after a disconnect | S/M |
| VSG-14 | Declared absences | exclusions listed (what is not in the slice) | M |

## 7. Relationship to `RELEASE_CRITERIA.md` section 2.2

| Existing row | Covered by |
|---|---|
| VS-1 scenario script exists | VSG-01 and the scenario script it uses |
| VS-2 unassisted run | VSG-01 |
| VS-3 content inventory | section 3 as the slice definition; VSG-02...VSG-08 |
| VS-4 determinism | M2 FPS-02 stays; VSG-09 |
| VS-5 persistence | VSG-09 |
| VS-6 regression | all FPS criteria still pass |
| VS-7 performance | VSG-12 (measured, thresholds by the owner) |

This is an extension, not a replacement. The expansion of M2 (conflict C-1) is not repeated here.

## 8. Risks and open questions

* Content volume is the biggest cost driver and has no owner (art, audio, design). The counts in section 3 may need to shrink; the questions Q-1...Q-8 must still be answerable.
* The plan has no gameplay tasks; M3 needs a Game Rules plan, an entity model decision (REF-13), a data-format decision for content, and an owner for each.
* Whether mobile belongs in M3 (a device checkpoint) and whether co-op does (D-MP) are owner decisions.
* The GDD/TDD is absent; this scope rests on recorded constraints and the references (AOD-24 baseline BL-2 is only a candidate).
