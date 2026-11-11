# Reference Comparison: Veloren, CubeWorld-Reversal, Cube

**Status:** research record, **Proposed**. Nothing here is a decision except where a cell cites an Approved Owner Decision (AOD, [`docs/decisions/owner-architecture-decisions-v1.md`](../decisions/owner-architecture-decisions-v1.md)).
**Evidence:** Veloren: the **upstream repository** https://gitlab.com/veloren/veloren (`master`, commit `7daa95fd`, 2026-10-30, GPL-3.0), read through GitLab raw-file and API endpoints and the official book; a first draft based on an older local clone was replaced after the owner pointed out the intended source; details in [`veloren-notes.md`](veloren-notes.md). CubeWorld-Reversal: https://github.com/qad3n/CubeWorld-Reversal (`master`, `cube/` and `server/`): README, the full file tree, folder listings and four declaration-only headers, **no implementation file read**; details and legal caveats in [`cubeworld-reversal-notes.md`](cubeworld-reversal-notes.md). Cube: `master` documentation and the experimental branches `spike/rust-godot`, `spike/rust-world-generation` (all spike results are experimental and desktop/emulator-only).
**Rule:** another project's choice is never presented as proven for Cube. Every borrowed principle carries one status: **Accepted**, **Proposed**, **Needs spike**, **Rejected**.

## 1. Comparison table

| Area | Veloren | CubeWorld-Reversal | Cube (today) | Cube decision |
|---|---|---|---|---|
| **World hierarchy** | Horizontal chunks 32x32 blocks with a variable-height column (sparse vertical sub-chunks); chunk/world conversions; world-scale simulation (regions, locations, civilisations, sites) above chunks | Appears to have a world container with region, zone and zone-tile levels, plus chunks/fields; structures and dungeons as separate types (inferred from file names) | Asset, layer, brick (16^3, copy-on-write, sparse); spike chunk = 16x32x16 column of two bricks; no level above chunks | Keep bricks and the Core model (Draft Core contracts). Add chunk streaming states in Gate 3. **REF-20 Proposed:** region/zone level above chunks is designed in Gate 8/9 (sharding and content), not now. **REF-30 Proposed:** storage-independent read/write views |
| **Chunk streaming** | Server decides what each client sees (view distance with a server maximum plus a 2-chunk hysteresis margin, capped at 128 chunks); asynchronous chunk generation with a deduplicated pending map, per-chunk cooperative cancel flags, a dedicated slow-job pool with a thread budget per job name, results via channel; staggered unload (one sixteenth of chunks checked per tick), background deallocation; client meshing queued only when the 3x3 neighbourhood is loaded, closest-first, capped by core count, applied under a per-frame budget, stale results dropped | Chunk buffer declarations suggest on-demand loading with chunks grouped in vertical columns; no unload or threading routine visible | Spike: synchronous streamer, bounded nearest-first queue, hysteresis, states Unloaded/Queued/Generating/Ready/Failed, no cancellation, no threads | **REF-05 Proposed (Gate 3):** cancellable asynchronous generation, bounded queue, thread budget. **REF-18 Proposed (Gate 7):** server-side interest management. **REF-06 Proposed (Gate 4):** background meshing with stale-result dropping |
| **Generation** | 32-bit seed; heavy one-off world simulation at start (macro terrain, civilisations and sites, economy, spots) then per-chunk generation with a cancellation callback polled per column and ordered layers (caverns, caves, rocks, shrubs, trees, scatter, paths, spots) plus sites; terrain follows the seed, entity/decoration output uses a system-seeded random generator; nightly Rust (`nightly-2026-06-13`) | Not visible as its own module in the listing; no evidence | Pure function of (seed, coordinate): integer value noise, height map, one biome; stable chunk hash and pinned values; synchronous | AOD-09 deterministic Rust generation. **REF-08 Proposed:** terrain deterministic, entity spawn separate. **REF-09 Needs spike (Gate 8):** whether to add a world-scale pass for rivers/biomes (start-up and mobile cost). **REF-12 Rejected:** nightly toolchain |
| **Rendering** | Own renderer on wgpu 27 / winit 0.30.12; greedy meshing that merges faces by caller-defined metadata, with AO/light/glow stored per texel in a texture atlas (not per vertex); special case for chunk-boundary planes; T-junctions not mentioned; terrain meshing on a slow-job pool | Shader pair and a smooth-mesh shape suggest smooth surfaces (inferred) | Godot; `faces` reference renderer, `greedy` experimental (material-aware merging, T-junction free, pixel-identical to faces in compared scenes); per-section dirty rebuild; no AO, no atlas; no device benchmark | AOD-10 (faces reference, greedy candidate until a real mobile benchmark). **REF-14 Rejected:** custom renderer as first client. **REF-15 Needs spike:** AO/light/atlas. Smooth meshing: **Rejected** for the current scope |
| **Client/server** | Separate crates and binaries: shared `common`, a display-agnostic client library, headless server binary, separate network crate with an I/O-free protocol core (TCP, in-process, QUIC; streams with priorities and a bandwidth budget); ECS on the server; single-player through a `singleplayer` feature that embeds the server; authority model not examined in detail | Client and server binaries share world/entity/ai/db code; net folder only on the server side of the listing | Spike server (loopback TCP, unauthenticated), embedded runtime in the Godot adapter; edits and streaming bypass any authority model | AOD-05, AOD-13, AOD-14, AOD-22. **REF-02 Proposed:** shared domain crate. **REF-03 Accepted.** **REF-13 Needs spike:** ECS for gameplay entities. **REF-04 Needs spike:** transport with stream priorities |
| **Persistence** | Terrain: **block-level overrides** per modified chunk, one file per chunk, versioned format with newest-to-oldest fallback readers, atomic replace, unknown versions backed up, written on unload only (not durable across power loss, acknowledged in the source), LRU cache by block count; overrides re-applied over regenerated terrain, overrides equal to generated dropped; unmodified world regenerated from the seed; feature-gated; characters in SQLite with migrations | SQLite-backed database class (README summary) | Spike: `cube-persistence` (spike format), `world-state-v0` (seed + edits + CRC, temporary); persistence contracts Draft | AOD-17 seed + generator version + versioned delta journal. **REF-07 Needs spike:** per-chunk snapshots as compaction. **REF-17 Proposed (Gate 9):** relational store for accounts/characters. **REF-23 Needs owner decision:** local single-player store |
| **UI/tools** | Own UI stack (several immediate/retained UI libraries, a debug UI), in-game commands; localisation crates; sandboxed WASM plugins | A 46-file widget set for menus, character, inventory, skills, map, chat (file names) | Godot buttons and HUD in the spike; no gameplay UI; GDScript limited to UI/glue (AOD-07) | **REF-27 Proposed:** UI feature checklist only. Godot UI built to Cube's own spec in Gate 6; no tool (editor) UI decided |
| **Mobile** | Builds for Windows, macOS and Linux (ARM64 builds for macOS and Linux); wgpu/winit desktop stack; nightly toolchain; README, manifests and the book mention no Android, iOS or WebAssembly client | Windows executables; no evidence | Android built and run on emulators only (ARM64 via translation); iOS blocked (no macOS/Xcode); no physical device result; `OS.execute` banned | AOD-08, AOD-11, AOD-12: mobile is mandatory and proven only on devices. **No mobile evidence in either reference**; neither may be cited for mobile feasibility |
| **Licensing** | GPL-3.0: studying allowed, copying source or close paraphrase would bind Cube to GPL terms | No license shown; derived from a proprietary game's binary; the README restricts use to people owning the game | License policy for Cube not decided | **Gate 0 decision (OI-5):** clean-room policy: observations only, no code/assets/names; owner decides the license policy and whether studying reverse-engineered sources is acceptable at all. **REF-28 Rejected:** reverse-engineered code as a source of truth |

## 2. Principle register

| ID | Principle | Source | Status | Gate |
|---|---|---|---|---|
| REF-01 | World generation usable without client or server | Veloren | Accepted (already true: `cube-worldgen` depends on Core only) | 3 |
| REF-02 | Shared domain/"common" crate for client and server | Veloren | Proposed | 1/2 |
| REF-03 | Headless server binary | Veloren | Accepted (AOD-05) | 7 |
| REF-04 | Separate transport crate with prioritised, budgeted streams | Veloren | Needs spike | 7 |
| REF-05 | Cancellable asynchronous chunk generation: deduplicated pending map, cancel flags, dedicated pool with per-class thread budget | Veloren | Proposed | 3 |
| REF-06 | Background meshing: closest-first, per-frame budget, stale drop, neighbours required | Veloren | Proposed | 4 |
| REF-07 | Persist block-level overrides of modified chunks only, re-applied over regenerated terrain (variant C1) | Veloren | Needs spike as a compaction/snapshot format (Cube's model is the AOD-21 journal) | 5 |
| REF-08 | Deterministic terrain, separate entity spawn step | Veloren | Proposed | 8 |
| REF-09 | World-scale simulation, then per-chunk generation | Veloren | Needs spike | 8 |
| REF-10 | Whole-world coarse simulation for NPCs/factions | Veloren | Needs spike (deferred) | 8/9 |
| REF-11 | Purpose-built Cargo profiles (worldgen-heavy, profiling, release with line tables) | Veloren | Proposed | 1 |
| REF-12 | Nightly Rust toolchain | Veloren | Rejected | 1 |
| REF-13 | ECS for gameplay entities | Veloren | Needs spike | 6/7 |
| REF-14 | Custom wgpu renderer as first client | Veloren | Rejected | 4 |
| REF-15 | Atlas-baked AO/light in greedy meshing | Veloren | Needs spike | 4 |
| REF-16 | WASM plugin interface | Veloren | Rejected (out of scope) | - |
| REF-17 | Relational store (SQLite) for accounts/characters with migrations | Veloren, CubeWorld-Reversal | Proposed | 9 |
| REF-18 | Server-side interest management | Veloren | Proposed | 7 |
| REF-19 | Chunk serialisation and compression stage | Veloren | Proposed | 7 |
| REF-20 | Spatial hierarchy above chunks (region/zone) | CubeWorld-Reversal | Proposed | 8/9 |
| REF-21 | Shared domain code, client-only presentation | CubeWorld-Reversal | Accepted (as principle) | 1 |
| REF-23 | Local SQLite database for single-player | CubeWorld-Reversal | Needs owner decision | 5 |
| REF-26 | Dialogue/quest text as a separate data structure | CubeWorld-Reversal | Proposed | 8 |
| REF-27 | Widget inventory as a UI feature checklist | CubeWorld-Reversal | Proposed (checklist only) | 6 |
| REF-28 | Reverse-engineered code as a source of truth | CubeWorld-Reversal | Rejected | 0 |
| REF-30 | Storage-independent volume read/write views | Veloren | Proposed | 2 |
| REF-31 | Structures, dungeons and spawning separate from terrain generation | CubeWorld-Reversal | Proposed | 8 |
| REF-32 | Single-player as an embedded server instance | Veloren | Accepted (consistent with AOD-22; Veloren is not evidence for it) | 6 |
| REF-33 | I/O-free protocol core, testable without sockets | Veloren | Proposed | 7 |
| REF-34 | Hysteresis margin, staggered unload checks, background deallocation | Veloren | Proposed | 3 |
| REF-35 | Versioned override files, fallback readers, backup on unknown version, atomic writes, drop-if-equal | Veloren | Proposed | 5 |
| REF-37 | Cooperative cancellation polled inside generation | Veloren | Proposed | 3 |
| REF-38 | Behaviour-tree style AI | CubeWorld-Reversal | Needs spike | 8 |

## 3. Where Cube is already ahead of, or deliberately different from, the references

* Determinism is tested with pinned content hashes and a rule that forbids floating point in world-defining math; neither reference was examined for an equivalent (not examined, so no comparison claim).
* Mobile is a stated requirement with a no-child-process guard; neither reference offers evidence.
* The Editing API / WorldGen split (AOD-15/16) and the seed + versioned journal (AOD-17) are Cube-specific decisions; Veloren confirms the "seed + modified data" idea at chunk granularity only.

## 4. Limits of this study

Veloren was inspected through summaries of selected upstream files, not functionally and not as a code review; CubeWorld-Reversal through the README, the file tree and four declaration-only headers (no implementation file read). Performance, scalability, security and correctness of either project were not assessed. All "Proposed" and "Needs spike" items must be validated against Cube's constraints before adoption.

## 5. Separation of statement types

The table in section 1 and the register in section 2 combine facts and interpretation. To keep them apart:

| Type | Where it lives | Rule |
|---|---|---|
| **Fact (upstream)** | the "Veloren" and "CubeWorld-Reversal" columns of section 1; section 2 of [`veloren-notes.md`](veloren-notes.md) onwards; [`cubeworld-reversal-notes.md`](cubeworld-reversal-notes.md) section 1 | observed in the remote repositories named in the headers; says nothing about Cube |
| **Fact (Cube)** | the "Cube (today)" column | measured or observed in the Cube repository; spike data is experimental, desktop or emulator only |
| **Agent inference** | "Observation" columns, "Agent inference" notes, section 3 | a conclusion by the agent; may be wrong |
| **Recommendation** | principle statuses Proposed / Needs spike / Rejected in section 2 | a suggestion, never a decision |
| **Owner decision** | cells that cite an AOD (for example AOD-05, AOD-10, AOD-17) and statuses "Accepted" justified by an AOD | only AOD-01…24 are Approved (AOD-24 conditionally); "Accepted" means consistent with an AOD or already true in Cube |
| **Unconfirmed hypothesis** | statements marked "Hypothesis", "not examined" or "inferred" | not to be used in a decision |

Veloren data are not evidence of suitability for Cube or for mobile platforms; neither reference contains mobile evidence. Owner-facing options and their consequences are in [`docs/decisions/owner-approval-matrix-v1.md`](../decisions/owner-approval-matrix-v1.md).
