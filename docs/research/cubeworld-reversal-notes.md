# CubeWorld-Reversal: Architecture Notes (read-only study)

**Purpose:** architectural observations for the Cube roadmap. **Not** a source of code, assets, names or data formats for Cube.
**Source:** https://github.com/qad3n/CubeWorld-Reversal, an unofficial fan project that publishes a **reconstructed (decompiled) source tree of the client and the dedicated server of an existing commercial game's early alpha**. Its README (as summarised from the repository page) says it is not compilable original code, ships no original binaries or assets, is meant for education and interoperability research, and requires a legitimate copy of the game; the repository page shows no software license. Read on 2026-11-04 through the GitHub web pages only; **nothing was cloned or downloaded** and no code was read.
**Evidence level (important):** the observations below come from the README summary and from **folder and file listings** of `cube/` (`world`, `render`, `entity`, `control`, `ui`, `db`, ...). I did not read any source file. Roles are therefore **inferred from file names and from the README**, and each statement is marked as such. `GAP_ANALYSIS.md` (`cube/GAP_ANALYSIS.md`, about 3,600 lines, 423 KB) was only partly read: its introduction, structure and statistics (the first ~100,000 of ~433,000 characters).
**Legal and clean-room policy:** the project is derived from a proprietary binary. Cube will not copy code, assets, data formats or class/function names from it, will not use it as a specification, and will not treat its contents as proven. Class and function names are deliberately **not recorded** in this document. Whether even reading such a repository is acceptable for the project is a Gate 0 license-policy question for the owner (open decision OI-5 in the roadmap).

Applicability labels: **Accepted**, **Proposed**, **Needs spike**, **Rejected** (as in `veloren-notes.md`).

## 1. What the repository contains

* Two binaries are reconstructed: a client and a dedicated server. Both have their own folders (`cube/`, `server/`).
* Folders shared by both: `world`, `entity`, `ai` (behaviour-tree style classes per the README), `db`, `game_misc` (functions without a single owning class), `include` (recovered types), `_library` (statically linked third-party code set aside).
* Client-only: `ui`, `render`, `audio`, `control`. Server-only: a `net` folder (connection and server classes) and a reduced `world`.
* The `db` folder holds a single database class which the README describes as SQLite-backed (README summary; the code was not read).
* `GAP_ANALYSIS.md` is **a best-effort semantic audit of functions that automated tooling could not name**: a table per function (address, proposed name, confidence, purpose). The statistics in its introduction: 3,429 functions analysed, with 1,578 rated high, 1,172 medium and 679 low confidence; the 1,121-row section visible in the part I read covers mostly library and runtime areas. It documents the *state of a reverse-engineering effort*, not a design. Two `.tsv` files (renames, sanitised names) and an attribution map sit next to it.

**Consequence:** the repository is a decompilation aid. It contains no design intent, no requirements and no rationale, and its own audit marks about 20% of the analysed functions as low confidence. It cannot be used as evidence that a design works well, only as a coarse picture of how one shipped game split its code.

## 2. Observations by area (inferred from folder/file names and README; not verified in code)

| Area | Observation | Applicability to Cube |
|---|---|---|
| **world/** (26 files) | The world subsystem appears split into a world container with metadata and a map; a hierarchy from large regions to zones to zone tiles; terrain chunks with a chunk buffer (likely a loading/streaming container), "fields" and landscape tiles; structures (dwellings) and dungeons as separate types; a spawn module. Generation is not visible as its own module in the listing. | **REF-20 Proposed (Gate 8/9):** a coarse spatial hierarchy above chunks (region/zone) is a plausible basis for shard/region ownership, spawn rules and map display; Cube has chunk and brick levels only. Needs a design, not a copy. **REF-31 Proposed (Gate 8):** structures, dungeons and spawning as separate modules from terrain generation. |
| **render/** (4 files listed) | A shader pair and a "smooth mesh shape" pair, which suggests smooth (not blocky) surface meshing for part of the world. | **Rejected for now:** Cube's mesh decisions are faces/greedy on voxels (AOD-10); the art direction cannot be assessed without the GDD (B-1). No evidence of how the renderer organises chunk geometry. |
| **entity/** (12 files listed) | A shared sprite base with a manager, a creature type, and a separate dialogue/quest-text group (a node structure suggests a tree). | **REF-26 Proposed (Gate 8):** dialogue/quest text as its own data structure, separate from the entity. **Not adopted:** the sprite/creature hierarchy, because nothing shows how it performs or evolves. |
| **control/** (4 files listed) | A controller and a game-controller specialisation sit between input and game state. | Consistent with Cube's input abstraction goal (Gate 6): input is mapped to intents before it reaches game logic. **Accepted as a principle** (already planned), no borrowed design. |
| **ui/** (46 files listed) | A widget set: start menu, options, system, character creation and preview, inventory, enchanting, skills, map overlay, objectives, statistics, chat, speech, world/blueprint/sprite/voxel previews. | **REF-27 Proposed:** useful only as a **feature checklist** for Gate 6/9 UI scope (inventory, skills, chat, map). Cube's UI will be Godot UI built to Cube's own spec. |
| **db/** (2 files) | A single database access class (SQLite per README). | **REF-23 Needs owner decision:** local persistence storage choice for single-player is open; Cube's persistence is seed + journal (AOD-17), not a general database. SQLite remains a candidate for accounts/characters (REF-17). |
| **client/server split** | Two separately built programs sharing world/entity/ai/db code; networking exists only on the server side of the listing (the client's connection code was not visible in the listing). | **REF-21 Accepted as a principle** (AOD-05 and AOD-13): shared domain code, client-only presentation. The listing says nothing about authority or prediction. |
| **ai/** | Behaviour-tree style classes (README). | **Needs spike (Gate 8):** AI architecture is undecided; no evidence from this source. |

## 3. What the source tells us about risk

* **Licensing and provenance risk (high):** derived from proprietary code; using its code, data layouts or names could expose Cube to claims. Policy: **REF-28 Rejected** (reverse-engineered code is not a source of truth or of code); observations only, at the level written here.
* **Accuracy risk:** the project's own audit rates a large share of functions as low confidence; structure inferred from file names may be wrong.
* **Scope mismatch:** the reconstructed game is a 2013 alpha with a different feature set; Cube's gates (mobile, server authority, deterministic generation, Editing API) are not addressed by it.

## 4. Decision

Keep this source **at the level of a feature and structure checklist** (world hierarchy idea, separate structure/dungeon/spawn modules, a dialogue data structure, the UI feature list, the client/server code split). Do not mine it further for design or code. If the owner wants any deeper study of it, record the license decision first (OI-5).

## 5. Principle register (this source)

| ID | Principle | Status |
|---|---|---|
| REF-20 | Spatial hierarchy above chunks (region/zone) | Proposed |
| REF-21 | Shared domain code, client-only presentation | Accepted (as principle) |
| REF-23 | Local SQLite database layer | Needs owner decision |
| REF-26 | Dialogue/quest text as a separate data structure | Proposed |
| REF-27 | Widget list as UI feature checklist | Proposed (checklist only) |
| REF-28 | Reverse-engineered code as source of truth | Rejected |
| REF-31 | Structures, dungeons and spawning separate from terrain generation | Proposed |
| (not numbered) | Smooth-mesh shapes, sprite/creature hierarchy | Rejected / not adopted |
