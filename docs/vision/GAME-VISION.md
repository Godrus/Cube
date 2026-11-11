# Cube: Game Vision (canonical, product level)

**Status of this document:** the owner's production directive of 2026-11-10 (E-CHAT, "Master Game Vision, Product Roadmap and Autonomous Production Execution") restated for the repository so that no agent depends on chat history. It adds no approved contract and no frozen game rule. Where it says *approved* it means "stated by the owner in that directive"; numbers and formulas are prototype values until the owner approves them.

**Source note (verified):** the original GDD/TDD v0.3 and "Voxel Architecture" documents are **not in the repository** (blocker B-1, [`docs/archive/README.md`](../archive/README.md), [`product-vision/production-plan-alignment-report.md`](../product-vision/production-plan-alignment-report.md) section 8). The only sources of this vision are the owner's directive above, the earlier owner statements recorded in [`docs/decisions/decision-registry.md`](../decisions/decision-registry.md) and [`docs/product-vision/cube-product-vision.md`](../product-vision/cube-product-vision.md) (CPR-xxx, the confirmed product requirements, which stay valid and are not repeated here), and the reference notes in `docs/research/`. Nothing here was reconstructed from documents that cannot be read.

Related: [`docs/roadmap/PRODUCT-ROADMAP.md`](../roadmap/PRODUCT-ROADMAP.md) (what is built in which order), `docs/production/` (engineering plan, ADG), [`docs/handoff/NEW-CHAT-START.md`](../handoff/NEW-CHAT-START.md) (start here as an agent).

## 1. What Cube is

A cross-platform **voxel open-world action RPG and sandbox with multiplayer**: a practically endless, seamless procedural world to explore; real-time combat; a character that grows (levels, skills, classes or specialisations, equipment, loot, collections); resource gathering and crafting; building and terrain change that go beyond placing unit cubes (detailed geometry, sculpting, boolean tools, micro-voxels, composite objects, undo/redo); living creatures and settlements; dungeons and bosses; play alone or with others on dedicated servers; PC and mobile on the same rules; a world that remembers what players did to it.

It is **not** only a building sandbox, and **not** a technology demonstration. The engine work (Rust, Godot 4) exists to deliver this game. Quality is measured by what a player can do and how stable it is, not by the number of declared features.

**Target scale (orientation, not a first-version requirement):** up to 500 concurrent players per logical world, through a distributed server design; not one process, and nothing of that infrastructure is built before a two-client prototype works.

## 2. Reference games (influence, not copying)

| Reference | What matters for Cube |
|---|---|
| Cube World (Alpha) | the feeling of travel; unknown procedural territory; varied enemies; classes, levels, skills, loot; adventure as the reason to explore. The spirit of the early Alpha, not the regional progression of the 2019 release. |
| Veloren | a systemic world; open, data-driven game definitions; procedural generation and simulation; creatures and NPCs; varied weapons; multiplayer; a scalable technical base. GPL: architectural reference only (AOD-23). |
| Trove | breadth of content; class development; dungeons and bosses; collections; building; co-op; long-term progression; reasons to return. |

Rule: Cube aims at **comparable depth of systems and its own identity**, not at copying mechanics, content, assets, code or worlds (policy AOD-23).

### 2.1 Capability comparison (qualitative, documented features only)

This is **not a quality ranking**. Entries for the references come from [`docs/research/reference-comparison.md`](../research/reference-comparison.md), [`cubeworld-reversal-notes.md`](../research/cubeworld-reversal-notes.md), [`veloren-notes.md`](../research/veloren-notes.md) (written earlier; the references change and were **not re-verified in this session**). "Cube" is today's honest state. Legend: `+` documented in the reference or exists in Cube; `~` partial; `-` absent; `?` not verified.

| Category | Cube World Alpha | Veloren | Trove | Cube today |
|---|---|---|---|---|
| Exploration | + | + | + | ~ walk around a finite generated terrain |
| World generation | + | + | ~ | ~ one terrain generator, no biomes or caves |
| Combat | + | + | + | - |
| Character progression | + | + | + | - |
| Classes | + | ~ (weapon-based skill sets) | + | - |
| Skills | + | + | + | - |
| Equipment, loot | + | + | + | - (block items only) |
| Crafting | ~ | + | + | - |
| Building | ~ | ~ | + | + place/mine blocks; deep tools planned |
| Geometry editing | - | - | - | ~ SDF, mesh, micro-voxel levels exist as tested engine code, not player tools |
| NPC / AI | + | + | ~ | - |
| Dungeons, bosses | + | + | + | - |
| Quests | ~ | ~ | + | ~ one hard-coded objective |
| Multiplayer | ? | + | + | - (in-process local host only) |
| Social systems | ? | ~ | + | - |
| Modding | - | + | ~ | - (palette model only) |
| Cross-platform | ? | ? | + | - (Windows desktop demo; Android unverified) |
| Performance, accessibility | ? | ? | ? | ~ measured on one PC; nothing on devices |
| Content pipeline | ? | + (data-driven RON) | ? | - |

**Where Cube is behind:** nearly everything the player meets besides terrain and blocks (combat, creatures, items, progression, multiplayer). **Where Cube can differ:** the Universal Geometry System (voxel + SDF + mesh + composite, micro-voxel detail inside a blocky world) and a persistent, mutable, server-authoritative world as one design (pillars 2, 4, 6, 8). **No system without game proof counts as an advantage:** the geometry advantage is real only when a player can use it (milestone M5).

## 3. Design pillars

Eight stable principles. For each: what the player gets, what contradicts it, how it is checked.

| # | Pillar | Player gets | Contradicts it | Checked by |
|---|---|---|---|---|
| 1 | **Exploration first** | new biomes, caves, rivers, vegetation, resources, ruins, settlements, dungeons, creatures, events; travelling is the main reason to play | one noise terrain repeated; discovery only by hash tests | a person walks 30 minutes and meets different areas (M3) |
| 2 | **Persistent mutable world** | the world keeps what players change; untouched terrain is recomputed from seed + generator version; user edits are stored separately and survive unloading, restart, compaction | loss or duplication of edits; claims of durability without tests | build, leave, unload, restart, return: build intact (M1) |
| 3 | **Deep action RPG** | levels, stats, active and passive skills, classes or specialisations, weapons, armour, loot tiers, collections | an inventory and a building task presented as an RPG | a combat and growth loop playable end to end (M2, M4) |
| 4 | **Creative freedom** | detailed building, sculpting, boolean tools, micro-voxels, composite objects, custom models, undo/redo | building limited to unit cubes; tools that bypass the Editing API | a player builds something a unit-cube game cannot (M5) |
| 5 | **Systemic world** | creatures with behaviour, spawning, a basic ecosystem, NPCs, settlements, day/night, weather, events | scripted-only content; a static decorative world | observe creatures acting without the player (M6) |
| 6 | **Multiplayer by design** | solo, co-op, dedicated servers, shared building, server authority, later horizontal scaling | gameplay state that exists only in the client | two clients fight and build in one world (M8) |
| 7 | **Cross-platform** | PC and mobile with the same rules, a different UI, controls and budgets | a reduced rule set on mobile; mobile checked only at the end | a PC and an Android player in one world; early Android feasibility spike (M9) |
| 8 | **Extensible geometry and content** | new content and geometry types without rewriting formats; data-driven definitions; later mods | hard-coded content in the engine; a format change per feature | a new item or creature added as data (from M2) |

## 3a. Visual direction

Cube World Alpha and Veloren: stylised, cohesive, playable voxel RPG; not maximum voxel complexity. Canonical text and the explicit priority change (micro-voxel character work deferred) are in [`VISUAL-STYLE-GUIDE.md`](VISUAL-STYLE-GUIDE.md).

## 4. Gameplay loops

* **Adventure:** explore, find an activity, face a challenge, earn a reward, grow, reach new territory.
* **RPG:** experience, level, skills, specialisation, new abilities, harder challenges.
* **Combat:** notice an enemy, choose actions, attack/dodge/defend, win or lose, loot.
* **Building:** resources, tools, design, build, improve the base.
* **Crafting:** gather components, recipe, craft, use, improve.
* **Social:** meet, group, adventure together, trade or build together, stay.

Numbers (stack sizes, damage, drop rates, curves) are **not** decided; see section 6.

## 5. Universal Geometry System in player terms

Hybrid geometry: voxel, SDF, polygon mesh, composite geometry, procedural representations. The base world stays mostly cubic; SDF and mesh must not silently become the format of the whole world. Brick 16^3, adaptive dense/sparse/constant, hash index, compression as direction; material palette; independent layers with explicit operations and attribute channels. Frozen status of each item comes from the decision registry, not from this list.

| Technology | Player-facing use |
|---|---|
| Micro-voxels | detailed building, small architectural elements, props without raising global density |
| Composite geometry | modular weapons and armour, furniture, vehicles and moving constructions later |
| Sculpting and boolean | unique objects, terrain shaping, creative tools |
| History | undo/redo, safe editing, rollback in shared building |
| Resolution (per-layer) | complex local detail without a heavy world |

Each engine stage has a player use or it is not scheduled ahead of one that has (see crosswalk).

## 6. What is *not* decided

Owner approval is still required for: final inventory rules (27 slots, stack 64, one block = one item are prototype values); the class system and its depth; combat model details and balance; death penalty; world rules per server; economy and trading; the Game Rules module and the gameplay owner (ODM-05, OI-8); the Game Track pillars as written in `production/GAME_TRACK_ROADMAP.md` (OPN-19); the final save format (GOV-10/11); smooth surfaces as a building feature (CPR-BLD-04 versus the geometry pillar, OPN-07); mod code runtime (DF-03). An agent may proceed on reversible engineering details; it must not present any of these as decided.

## 6a. Minimum viable product (MVP) versus the demo today

The MVP is the first state in which a stranger can play for an hour and want to continue: the world looks different in different places (M3), you fight and win rewards (M2) and grow (M4), you build with more than cubes (M5, minimal), the world is alive (M6, minimal), two players can share it (M8) including one on Android (M9). Today (M0) there is a walking player in a generated finite terrain with blocks, an inventory and one objective, as a developer demo.
