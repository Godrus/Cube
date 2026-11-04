# Reference Comparison: Veloren, CubeWorld-Reversal, Cube

**Status:** research record, **Proposed**. Nothing here is a decision except where a cell cites an Approved Owner Decision (AOD, `docs/decisions/owner-architecture-decisions-v1.md`).
**Evidence:** Veloren: a read-only local clone (commit `b4af357b`, 2025-03-17, GPL-3.0), directory listings and the heads of a few files; details in `veloren-notes.md`. CubeWorld-Reversal: the repository page and **file/folder listings only**, no code read; details and legal caveats in `cubeworld-reversal-notes.md`. Cube: `master` documentation and the experimental branches `spike/rust-godot`, `spike/rust-world-generation` (all spike results are experimental and desktop/emulator-only).
**Rule:** another project's choice is never presented as proven for Cube. Every borrowed principle carries one status: **Accepted**, **Proposed**, **Needs spike**, **Rejected**.

## 1. Comparison table

| Area | Veloren | CubeWorld-Reversal | Cube (today) | Cube decision |
|---|---|---|---|---|
| **World hierarchy** | Horizontal chunks 32x32 blocks with a variable-height column (sparse vertical sub-chunks); chunk/world conversions; world-scale simulation (regions, locations, civilisations, sites) above chunks | Appears to have a world container with region, zone and zone-tile levels, plus chunks/fields; structures and dungeons as separate types (inferred from file names) | Asset, layer, brick (16^3, copy-on-write, sparse); spike chunk = 16x32x16 column of two bricks; no level above chunks | Keep bricks and the Core model (Draft Core contracts). Add chunk streaming states in Gate 3. **REF-20 Proposed:** region/zone level above chunks is designed in Gate 8/9 (sharding and content), not now. **REF-30 Proposed:** storage-independent read/write views |
| **Chunk streaming** | Server decides what each client sees (view distance with a server maximum, subscriptions); asynchronous chunk generation with per-chunk cancel flags, a thread budget per job type, results via channel; meshing on worker threads with neighbour re-mesh | Chunk buffer type suggests a streaming container; nothing verifiable | Spike: synchronous streamer, bounded nearest-first queue, hysteresis, states Unloaded/Queued/Generating/Ready/Failed, no cancellation, no threads | **REF-05 Proposed (Gate 3):** cancellable asynchronous generation, bounded queue, thread budget. **REF-18 Proposed (Gate 7):** server-side interest management. **REF-06 Proposed (Gate 4):** background meshing with stale-result dropping |
| **Generation** | Seed-based; heavy one-off world simulation at start (erosion, rivers, civilisations) then per-chunk generation with layers (caves, rocks, vegetation, wildlife, sites); terrain deterministic, entity supplements use random sources; nightly Rust features | Not visible as its own module in the listing; no evidence | Pure function of (seed, coordinate): integer value noise, height map, one biome; stable chunk hash and pinned values; synchronous | AOD-09 deterministic Rust generation. **REF-08 Proposed:** terrain deterministic, entity spawn separate. **REF-09 Needs spike (Gate 8):** whether to add a world-scale pass for rivers/biomes (start-up and mobile cost). **REF-12 Rejected:** nightly toolchain |
| **Rendering** | Own wgpu/winit renderer; greedy meshing with ambient occlusion, light and texture atlas baked in; special case for chunk-boundary faces; terrain meshing on workers | Shader pair and a smooth-mesh shape suggest smooth surfaces (inferred) | Godot; `faces` reference renderer, `greedy` experimental (material-aware merging, T-junction free, pixel-identical to faces in compared scenes); per-section dirty rebuild; no AO, no atlas; no device benchmark | AOD-10 (faces reference, greedy candidate until a real mobile benchmark). **REF-14 Rejected:** custom renderer as first client. **REF-15 Needs spike:** AO/light/atlas. Smooth meshing: **Rejected** for the current scope |
| **Client/server** | Separate crates and binaries: shared `common`, client library, headless server binary, separate network crate; ECS state on both sides; server authoritative (the architecture implies it; authority model not examined in detail) | Client and server binaries share world/entity/ai/db code; net folder only on the server side of the listing | Spike server (loopback TCP, unauthenticated), embedded runtime in the Godot adapter; edits and streaming bypass any authority model | AOD-05, AOD-13, AOD-14. **REF-02 Proposed:** shared domain crate. **REF-03 Accepted.** **REF-13 Needs spike:** ECS for gameplay entities. **REF-04 Needs spike:** transport with stream priorities |
| **Persistence** | Modified chunks persisted per chunk with atomic writes and an LRU cache; unmodified world regenerated from the seed; characters in SQLite with migrations | SQLite-backed database class (README summary) | Spike: `cube-persistence` (spike format), `world-state-v0` (seed + edits + CRC, temporary); persistence contracts Draft | AOD-17 seed + generator version + versioned delta journal. **REF-07 Needs spike:** per-chunk snapshots as compaction. **REF-17 Proposed (Gate 9):** relational store for accounts/characters. **REF-23 Needs owner decision:** local single-player store |
| **UI/tools** | Own UI stack (several immediate/retained UI libraries, a debug UI), in-game commands; localisation crates | A 46-file widget set for menus, character, inventory, skills, map, chat (file names) | Godot buttons and HUD in the spike; no gameplay UI; GDScript limited to UI/glue (AOD-07) | **REF-27 Proposed:** UI feature checklist only. Godot UI built to Cube's own spec in Gate 6; no tool (editor) UI decided |
| **Mobile** | Desktop builds (Windows/macOS/Linux per README); wgpu/winit desktop stack; nightly toolchain; no Android/iOS target found in the manifests and documents searched | Windows executables; no evidence | Android built and run on emulators only (ARM64 via translation); iOS blocked (no macOS/Xcode); no physical device result; `OS.execute` banned | AOD-08, AOD-11, AOD-12: mobile is mandatory and proven only on devices. **No mobile evidence in either reference**; neither may be cited for mobile feasibility |
| **Licensing** | GPL-3.0: studying allowed, copying source or close paraphrase would bind Cube to GPL terms | No license shown; derived from a proprietary game's binary; the README restricts use to people owning the game | License policy for Cube not decided | **Gate 0 decision (OI-5):** clean-room policy: observations only, no code/assets/names; owner decides the license policy and whether studying reverse-engineered sources is acceptable at all. **REF-28 Rejected:** reverse-engineered code as a source of truth |

## 2. Principle register

| ID | Principle | Source | Status | Gate |
|---|---|---|---|---|
| REF-01 | World generation usable without client or server | Veloren | Accepted (already true: `cube-worldgen` depends on Core only) | 3 |
| REF-02 | Shared domain/"common" crate for client and server | Veloren | Proposed | 1/2 |
| REF-03 | Headless server binary | Veloren | Accepted (AOD-05) | 7 |
| REF-04 | Separate transport crate with prioritised streams | Veloren | Needs spike | 7 |
| REF-05 | Cancellable asynchronous chunk generation, thread budget per job class | Veloren | Proposed | 3 |
| REF-06 | Background meshing; re-mesh neighbours on load; drop stale results | Veloren | Proposed | 4 |
| REF-07 | Persist modified chunks only, atomic writes, LRU cache | Veloren | Needs spike (Cube uses a journal) | 5 |
| REF-08 | Deterministic terrain, separate entity spawn step | Veloren | Proposed | 8 |
| REF-09 | World-scale simulation, then per-chunk generation | Veloren | Needs spike | 8 |
| REF-10 | Whole-world coarse simulation for NPCs/factions | Veloren | Needs spike (deferred) | 8/9 |
| REF-11 | Purpose-built Cargo profiles (worldgen-heavy, profiling, release with line tables) | Veloren | Proposed | 1 |
| REF-12 | Nightly Rust toolchain | Veloren | Rejected | 1 |
| REF-13 | ECS for gameplay entities | Veloren | Needs spike | 6/7 |
| REF-14 | Custom wgpu renderer as first client | Veloren | Rejected | 4 |
| REF-15 | Baked AO/light and texture atlas in greedy meshing | Veloren | Needs spike | 4 |
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

## 3. Where Cube is already ahead of, or deliberately different from, the references

* Determinism is tested with pinned content hashes and a rule that forbids floating point in world-defining math; neither reference was examined for an equivalent (not examined, so no comparison claim).
* Mobile is a stated requirement with a no-child-process guard; neither reference offers evidence.
* The Editing API / WorldGen split (AOD-15/16) and the seed + versioned journal (AOD-17) are Cube-specific decisions; Veloren confirms the "seed + modified data" idea at chunk granularity only.

## 4. Limits of this study

Veloren was inspected structurally, not functionally; CubeWorld-Reversal only by names. Performance, scalability, security and correctness of either project were not assessed. All "Proposed" and "Needs spike" items must be validated against Cube's constraints before adoption.
