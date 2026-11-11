# CubeWorld-Reversal: Architecture Notes (read-only study)

**Purpose:** architectural observations for the Cube roadmap. **Not** a source of code, assets, names or data formats for Cube.
**Source:** https://github.com/qad3n/CubeWorld-Reversal, branch `master`, folder `cube/` (client) and `server/` (dedicated server), an unofficial fan project that publishes a **reconstructed source tree of the 2013 alpha client and dedicated server of an existing commercial game**. Read on 2026-11-04 through the GitHub web pages, raw-file URLs and the GitHub tree API. **Nothing was cloned or downloaded.**
**What was read:** the full README; the complete file tree (196 files); the folder listings of `cube/` subfolders (`world`, `render`, `entity`, `control`, `ui`, `db`, `ai`, `audio`, `game_misc`, `include`, `misc`, `_library`) and of `server/` (`net`, `world`); four class declaration headers from `cube/world/` (the world object, the chunk, the chunk buffer, the zone); and the introduction, structure and statistics of `cube/GAP_ANALYSIS.md` (about 3,600 lines, 423 KB; only the first ~100,000 of ~433,000 characters were read). **No `.cpp` implementation file was read.**
**Evidence level (important):** the README and the tree are factual. The four headers contain **declarations only** (signatures without member layouts), so they show shape, not data structures; their summaries below are limited to what the declarations imply and are marked "inferred". Everything about roles of the other folders is inferred from file names. This repository cannot answer questions about sizes, memory layout, threading or streaming policy.
**Legal and clean-room policy:** the project is derived from a proprietary binary. Cube will not copy code, assets, data formats or class/function names from it, will not use it as a specification, and will not treat its contents as proven. Class and function names are deliberately **not recorded** here. Whether studying such a repository is acceptable for the project is a Gate 0 license-policy question for the owner (open item OI-5 in the roadmap).

Applicability labels: **Accepted**, **Proposed**, **Needs spike**, **Rejected** (as in [`veloren-notes.md`](veloren-notes.md)).

## 1. What the repository is (from the README)

* **Purpose and method:** a recovered class and file layout of the 2013 alpha client and dedicated server, produced by static analysis in a disassembler/decompiler using run-time type information and call-graph analysis to group functions into classes and subsystems. It does **not** build; the function bodies are decompiler pseudo-C of optimised, stripped code and may be imperfect.
* **Layout:** one folder per binary. Shared folders: `world`, `entity`, `ai` (behaviour-tree classes), `db` (SQLite glue), `game_misc` (functions without a single owning class). Client only: `ui`, `render`, `audio`, `control`. Server only: `net` (connection and server classes), plus a reduced `world`. `_library` holds statically linked third-party code (SQLite, C/C++ runtime, FreeType), `include` the recovered types.
* **Scale (README and tree):** 196 files; the client has about 12,983 functions of which about 1,370 are game code, the server about 6,152 of which about 288 are game code; the rest is library code. The tree: client `world` 26 files, `ui` 46, `ai` 16, `entity` 12, `render` 4, `control` 4, `audio` 4, `db` 2; server `world` 14, `net` 4, `ai` 16, `entity` 12, `db` 2.
* **Index files:** `attribution.tsv` (function address, name, kind, owning class) and `GAP_ANALYSIS.md` next to each binary; the README does not describe the two other `.tsv` files present.
* **Provenance and stack (README):** the binaries carry developer build paths pointing to the original author; the engine code lives in separate namespaces for an engine and a reflection layer; third-party components include SQLite, zlib, an image library, an audio API and the MSVC 2012 runtime.
* **Legal notice (README, in substance):** the game, its code, assets, names and trademarks belong to its creators; the project is an unofficial, non-affiliated fan effort for education and interoperability research, contains no original binaries, assets or data files, and asks users to own a legitimate copy; rights holders may request changes or removal. The repository page shows no software license.
* **`GAP_ANALYSIS.md`:** a best-effort semantic audit of functions that automated tooling could not name: per function an address, a proposed name, a confidence rating and a purpose. Statistics in its introduction: 3,429 functions analysed; 1,578 rated high, 1,172 medium and 679 low confidence (about 20% low). The part I read is mostly library and runtime code. It documents the state of a reverse-engineering effort, not a design.

**Consequence:** the repository is a decompilation aid. It contains no design intent, requirements or rationale and cannot show that a design works well; it only gives a coarse picture of how one shipped game split its code.

## 2. Observations by area (inferred; see the evidence level above)

| Area | Observation | Applicability to Cube |
|---|---|---|
| **world/** (26 client files, 14 server files) | Declarations suggest: a world object that holds a map of chunks keyed by position with an "active" subset updated each tick, lists of nearby entities, and terrain queries (walkability, block sampling); naming support for regions; separate types for regions, zones and zone tiles, terrain "fields", landscape tiles, dwellings, dungeons and a spawn module. The chunk buffer declarations suggest chunks grouped in **vertical columns**, on-demand loading with a notification, wide integer coordinates with fixed-point positions, a per-chunk quad-mesh routine, per-vertex ambient occlusion by neighbour sampling and a sunlight propagation routine that crosses chunk boundaries; no unload or threading routine is visible. The server keeps a subset (world, zone, region, field, dwelling, dungeon, spawn) without chunk or rendering classes. Generation does not appear as its own module. | **REF-20 Proposed (Gate 8/9):** a coarse spatial hierarchy above chunks (region/zone) as a basis for sharding, spawn rules and map display; Cube has chunk and brick levels only. **REF-31 Proposed (Gate 8):** structures, dungeons and spawning as separate modules from terrain generation. The server holding only a world subset is consistent with AOD-13 but proves nothing about authority. |
| **render/** (4 files) | A shader pair and a smooth-surface mesh shape pair; meshing and lighting seem to live in the world/chunk classes rather than in the render folder. | **Rejected for now:** Cube's mesh decisions are faces/greedy on voxels (AOD-10); art direction cannot be judged without the GDD (B-1). |
| **entity/** (12 files) | A sprite base with a manager, a creature type, and a separate dialogue/quest-text group (a node structure suggests a tree). | **REF-26 Proposed (Gate 8):** dialogue/quest text as its own data structure. **Not adopted:** the sprite/creature hierarchy; nothing shows how it performs or evolves. |
| **ai/** (16 files, 8 behaviour classes) | A behaviour-tree style: leaf behaviours (combat, companion, look at the player, random interaction, random walk, spawn-location, path walking) and a sequence composite; present in both client and server folders (README: shared). | **Needs spike (Gate 8):** AI architecture is undecided (REF-13 area). A behaviour tree is a standard pattern; this source adds no evidence of fitness. |
| **control/** (4 files) | A controller and a game-controller specialisation between input and game state. | Consistent with Cube's input abstraction goal (Gate 6): input becomes intents before it reaches game logic. **Accepted as a principle** (already planned); no design borrowed. |
| **ui/** (46 files) | A widget set: start menu, options, system, character creation and preview, inventory, enchanting, skills, map overlay, objectives, statistics, chat, speech, world/blueprint/sprite/voxel previews. | **REF-27 Proposed:** useful only as a **feature checklist** for Gate 6/9 UI scope. Cube's UI will be Godot UI to Cube's own spec. |
| **db/** (2 files) | One database access class; the README calls it SQLite glue. | **REF-23 Needs owner decision:** local single-player store is open; Cube's persistence is seed + journal (AOD-17). SQLite remains a candidate for accounts/characters (REF-17). |
| **client/server split** | Two separately built programs sharing world, entity, ai and db code; networking exists only in the server folder (two classes: connection and server); the client's network code was not visible in the tree. | **REF-21 Accepted as a principle** (AOD-05, AOD-13): shared domain code, client-only presentation. The tree says nothing about authority or prediction. |
| **audio/** (4 files) | Music and an audio-engine wrapper (client only). | Not in the current gates. |

## 3. What the source tells us about risk

* **Licensing and provenance risk (high):** derived from proprietary code; using its code, data layouts or names could expose Cube to claims. Policy: **REF-28 Rejected** (reverse-engineered code is not a source of truth or of code); observations only, at the level written here.
* **Accuracy risk:** the project's own audit rates about 20% of the analysed functions as low confidence, and the README says bodies may be imperfect; structure inferred from declarations and names may be wrong.
* **Scope mismatch:** a 2013 alpha with a different feature set; Cube's gates (mobile, server authority, deterministic generation, Editing API) are not addressed by it.

## 4. Decision

Keep this source **at the level of a feature and structure checklist** (world hierarchy idea, separate structure/dungeon/spawn modules, a dialogue data structure, the UI feature list, a behaviour-tree style for AI, the client/server code split). Do not mine it further for design or code. A deeper study (reading implementation files) would need the license decision first (OI-5).

## 5. Principle register (this source)

| ID | Principle | Status |
|---|---|---|
| REF-20 | Spatial hierarchy above chunks (region/zone) | Proposed |
| REF-21 | Shared domain code, client-only presentation | Accepted (as principle) |
| REF-23 | Local SQLite database for single-player | Needs owner decision |
| REF-26 | Dialogue/quest text as a separate data structure | Proposed |
| REF-27 | Widget list as UI feature checklist | Proposed (checklist only) |
| REF-28 | Reverse-engineered code as source of truth | Rejected |
| REF-31 | Structures, dungeons and spawning separate from terrain generation | Proposed |
| REF-38 | Behaviour-tree style AI | Needs spike (Gate 8) |
| (not numbered) | Smooth-mesh shapes, sprite/creature hierarchy | Rejected / not adopted |

## 6. Separation of statement types

Evidence here is weak (README, file tree, folder listings, four declaration-only headers; no implementation file read). Nothing below is evidence of fitness for Cube or for mobile platforms.

**Facts found in upstream (Fact (upstream), GitHub `master`)**
* The repository is an unofficial reconstructed source tree of a 2013 alpha client and dedicated server of a commercial game; it does not build; the README states original rights belong to its creators; no software licence is shown on the repository page.
* 196 files; the folder layout and file counts listed in section 1; `GAP_ANALYSIS.md` reports 3,429 analysed functions with 1,578 high, 1,172 medium and 679 low confidence.
* The four headers read contain declarations only.

**Agent inferences (may be wrong)**
* The file names suggest a world container with region/zone/field levels, chunk columns, separate structure/dungeon/spawn modules, a behaviour-tree style AI and a 46-widget UI set.
* The server folder containing only a world subset is consistent with a client/server split; it says nothing about authority.

**Recommendations (Proposed, not decisions)**
* Use the source only as a feature and structure checklist (section 4); REF statuses in section 5.

**Owner decisions**
* None. Whether studying a reverse-engineered repository is acceptable at all is an open owner decision (OAM-07 in [`docs/decisions/owner-approval-matrix-v1.md`](../decisions/owner-approval-matrix-v1.md)). No class, function or data-format name is recorded in Cube documents.

**Unconfirmed hypotheses**
* Any claim about memory layout, streaming policy, threading, performance, gameplay design intent or correctness of the reconstruction.
