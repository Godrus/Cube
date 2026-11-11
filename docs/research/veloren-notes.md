# Veloren: Architecture Notes (read-only study of the upstream repository)

**Purpose:** architectural observations for the Cube roadmap. **Not** a source of code, assets or names for Cube.
**Source:** the upstream repository https://gitlab.com/veloren/veloren, branch `master`, read through the GitLab web interface and raw-file/API endpoints on 2026-11-04. Latest commit at the time: `7daa95fd53ae62aa185d31ad316d0cabfe2d70ac` (2026-10-30, "Merge branch 'neura/flakefixixixi' into 'master'"). Official book: https://book.veloren.net (page `contributors/developers/codebase-structure`). License: GNU General Public License v3 or later.
**Correction of an earlier draft:** the first version of this document was written from an older **local clone** (commit `b4af357b`, 2025-03-17). The owner pointed out that the upstream repository is the intended source. Re-reading upstream changed several facts: the pinned toolchain is now `nightly-2026-06-13` (the clone had `nightly-2025-02-14`), the graphics stack is `wgpu` 27 with `winit` 0.30.12 (the clone had 0.18 and 0.28), the workspace has 25 members (an extra credential-store crate), and terrain persistence stores block-level overrides (my first note described whole-chunk persistence). All statements below come from upstream.
**License handling:** Cube must not copy, translate or closely paraphrase Veloren source. This document records structure and design ideas in our own words, with file-level references so the owner can verify them. Veloren identifiers are used only where a file path needs them.
**What was read (upstream, read-only, nothing built or run):** `Cargo.toml` (workspace, profiles), `rust-toolchain`, `README.md`, `voxygen/Cargo.toml`, `world/src/lib.rs`, `server/src/chunk_generator.rs`, `server/src/terrain_persistence.rs`, `server/src/sys/terrain.rs`, `common/src/slowjob.rs`, `common/src/terrain/mod.rs`, `voxygen/src/scene/terrain/mod.rs`, `voxygen/src/mesh/greedy.rs`, `network/protocol/src/lib.rs`, `rtsim/src/lib.rs`, the commit and `world/src` tree via the GitLab API, and the book's codebase-structure page. This is **not** a code review; the files were summarised, not audited. Anything not read is marked "not examined".

Applicability labels: **Accepted** (consistent with an Approved Owner Decision or already true in Cube), **Proposed** (suggested for a gate, needs owner/gate approval), **Needs spike** (evidence required), **Rejected** (not applicable or contrary to a decision).

## 1. Workspace structure

* 25 workspace members: `common` with sub-crates (`assets`, `base`, `dynlib`, `ecs`, `i18n`, `net`, `state`, `systems`, `frontend`, `query_server`), `client` (+ `i18n`), `server` (+ `agent`), `server-cli`, `voxygen` (+ `anim`, `credentialstore`, `egui`, `i18n-helpers`), `world`, `network` (+ `protocol`), `rtsim`; plugin crates are separate.
* The book classifies the crates as **frontends** (`voxygen`, `server-cli`), **backends** (`server`, `client`, `world`: headless libraries that implement gameplay; `client` is display-agnostic with chat-only, spectator and character modes; `world` is usable without client or server) and **utility crates** (`common` for behaviour and definitions shared by client and server, `network`). Stated goals: modularity, performance, no lock-in; the split also speeds up compilation. Frontends are first-class: any frontend can sit on the generic client library.
* The graphical client has a `singleplayer` feature that pulls in the server library (built with world generation) so that the game runs a server in the same program.
* Observations for Cube: this is the separation AOD-04, AOD-05 and AOD-06 require. **REF-01 Accepted** (world generation without client/server: already true for `cube-worldgen`), **REF-03 Accepted** (headless server binary, AOD-05), **REF-02 Proposed** (a shared domain crate used by server and client runtime; its place in the AU-1.0 layer grid is not decided). **REF-32 Accepted (consistent with AOD-22, which Cube decided independently):** single-player as an embedded server instance (Veloren does it behind a feature flag; whether its authority rules are the same as for a remote server was not examined).

## 2. Terrain model

* Horizontal chunks of 32 x 32 blocks (log2 constant 5; the source notes that larger values would break packing of chunk coordinates into 5 bits in the frontend). The vertical structure is a variable-height column with sparse sub-chunks (a "chonk" type), held in a 2D grid of chunks. Integer world-to-chunk conversion uses Euclidean division, so negative coordinates map correctly; chunk-relative offsets use the Euclidean remainder (`common/src/terrain/mod.rs`). Chunk metadata carries biome, altitude, temperature, humidity, river and water flags, rockiness, cliff height, site information.
* Cube: three-dimensional 16^3 copy-on-write bricks; chunk = a 16 x 32 x 16 column of two bricks in the spike; Euclidean division for negative coordinates (the same convention).
* **REF-30 Proposed (Gate 2):** read/write access through volume traits independent of the storage layout lets generation, meshing and persistence share one interface (matches the intent of the Draft Core contracts C-07/C-08). Not a reason to change the brick decision.

## 3. World generation

* Entry point: a `World` is generated from a **32-bit seed** with a thread pool, in phases: (1) a world simulation (macro terrain and per-chunk data), (2) civilisations and sites, (3) an economy simulation, (4) "spots". It runs once at start. `world/src/lib.rs` also exposes map data, LOD zones, position search and location names.
* Per-chunk generation takes a shared reference to the world, a chunk position and a **cancellation callback** and returns a terrain chunk plus a **supplement** (entity spawns, waypoint, wildlife, site supplements). The cancellation callback is polled per column. Steps: sample columns with a border, fill base stone, apply feature-gated layers in order (trains, caverns, caves, rocks, shrubs, trees, scatter, paths, spots), render the sites intersecting the chunk, assemble the supplement, defragment, resolve rtsim resource blocks.
* **Determinism:** the world structure and the terrain sampling follow the seed; dynamic content (entity spawns, decoration, wildlife, resource thinning) uses a random generator seeded from the system, which the source says is only for dynamic content.
* Observations for Cube:
  * **REF-09 Needs spike (Gate 8):** rivers, biomes and large-scale structure need a world-scale model; Veloren pays a long one-off simulation at start. Cube's per-column pure functions are cheap and local but limited for rivers and large features. A spike must measure start-up time, memory and determinism on a mobile-class CPU before anything is adopted.
  * **REF-08 Proposed (Gate 8):** terrain deterministic and entity/spawn output separate with its own seeding rule, so gameplay randomness never touches terrain.
  * **REF-37 Proposed (Gate 3):** a cooperative cancellation check inside generation (cheap, per column or per brick) on top of job-level cancellation.
  * **REF-12 Rejected:** the workspace pins a nightly toolchain (`nightly-2026-06-13`). Cube stays on stable Rust.

## 4. Server model

* The server is an entity-component-system program (systems under `server/src/sys/`: terrain, terrain sync, chunk send and serialisation, entity sync, subscription, persistence, and more).
* **Chunk generation requests:** a chunk generator keeps a map of pending chunk positions; a duplicate request for a pending chunk is ignored; each pending chunk has a shared cancellation flag; work runs as a named job on a dedicated slow-job pool; results come back through an unbounded channel and are drained on the main tick; results whose key is no longer pending (cancelled) are discarded. Cancellation is cooperative (the worker polls the flag). The source itself notes a limitation: only the first client to cancel a chunk is notified, so others may wait about three seconds before re-requesting (`server/src/chunk_generator.rs`).
* **Slow-job pool** (`common/src/slowjob.rs`): a dedicated pool, separate from the tick's pool so that work stealing cannot pull slow jobs into time-critical work; low-priority threads; a global thread limit and a per-job-name local limit; each job name has a FIFO queue and is guaranteed at least one slot when capacity remains, the rest shared in proportion; only queued (not yet started) jobs can be cancelled; I/O-blocking work is forbidden; metrics per job name.
* **Chunk lifecycle on the server** (`server/src/sys/terrain.rs`): requests are drained and handed to the generator; finished chunks get saved modifications applied before insertion (when persistence is enabled); the effective view distance is the player's request clamped to the server maximum **plus a hysteresis margin of 2 chunks**, capped at 128 chunks; chunks are sent to every client whose view circle covers them; **unloading is staggered: only one sixteenth of the loaded and pending chunks is checked per tick** (bucket by coordinate and tick); a chunk is kept while any presence (including non-client entities) covers it; otherwise persistence is notified, a pending generation is cancelled and the chunk is removed; removed chunks are dropped on a background job so the tick is not blocked by deallocation.
* Observations for Cube:
  * **REF-05 Proposed (Gate 3):** a pending map with deduplication and cancel flags, a dedicated slow-job pool with per-class thread budgets, FIFO queues per class, and a channel back to the main tick; Cube's streamer has a bounded queue and state machine but is synchronous and cannot cancel.
  * **REF-34 Proposed (Gate 3):** hysteresis margin plus staggered unload checks and background deallocation (Cube already has load radius plus unload margin; staggering and background drop are new ideas).
  * **REF-18 Proposed (Gate 7):** per-client interest sets computed by the server from view distance and presence.
  * **REF-19 Proposed (Gate 7):** chunk serialisation and compression as separate stages (`chunk_serialize`, `chunk_send`; compression in `common/net/src/msg/`; not examined in detail).
  * **REF-13 Needs spike (Gate 6/7):** an ECS for gameplay entities; Cube's AU-1.0 is an explicit domain model over geometry assets and AOD does not imply an ECS.

## 5. Persistence

* **Terrain modifications are persisted as block-level overrides**, not as whole chunks: each chunk keeps a map from in-chunk block position to block value, only chunks marked modified are written, one file per chunk named by its coordinates, in a terrain directory (overridable by an environment variable). The file has a 64-bit version header (a magic constant plus the format version); the current format is the third; older versions stay readable (the reader tries versions from newest to oldest); saving always uses the current format. **Unknown, corrupt or newer files are renamed to a numbered backup** and the chunk starts empty (the player's changes in that chunk are effectively lost, the file is kept for manual repair; downgrading triggers this). Files are written atomically. A chunk is written **only when it is unloaded**; the periodic maintenance hook does nothing, and the source says that this is not durable across power loss. A modified chunk with no remaining overrides has its file deleted. An in-memory LRU cache of recently unloaded chunks is limited by a total block count (about 64 million), not by chunk count.
* **Overlay semantics:** when a chunk is generated, its stored overrides are applied on top of the generated terrain; an override equal to the generated block is dropped as a size optimisation; new edits go through one setter that converts world coordinates to chunk key and local offset and marks the chunk modified only if the value actually changed; clearing a chunk empties its overlay so generated terrain shows through again. Persistence is feature-gated (`persistent_world`) and optional (`server/src/terrain_persistence.rs`, `sys/terrain.rs`).
* Character data is stored in SQLite through separate loader/updater components with migrations (`server/src/persistence/`, `server/src/migrations/`; the file list shows a migration from Diesel to rusqlite). Not examined further.
* Observations for Cube:
  * This is real-world evidence for AOD-17's "seed plus modified data" and a real-world instance of the C1 idea (one layer with block-level overrides, as named in [`docs/decisions/worldgen-core-boundary.md`](../decisions/worldgen-core-boundary.md)) at per-chunk override granularity. Cube's approved architecture is C3 with a durable journal and snapshots (AOD-20, AOD-21); C1 (block-level overrides in one layer) is the recommended first baseline of the layer model but persisted through that journal, not through per-chunk files written on unload.
  * **REF-07 Needs spike (Gate 5):** Cube's choice is a versioned delta journal (AOD-17). The Veloren design suggests comparing "journal replay" with "per-chunk override snapshot" as the compaction format, and shows the cost of writing only on unload (not durable) which Cube's journal avoids by design.
  * **REF-35 Proposed (Gate 5):** versioned files with newest-to-oldest fallback readers, backup instead of overwrite for unknown versions, atomic replacement, an "override equals generated: drop it" optimisation, and deletion of empty overlays.
  * **REF-17 Proposed (Gate 9):** a relational store (SQLite) for accounts and characters with migrations; Cube has not chosen one.
  * Not examined: how Veloren handles world or generator changes between releases (relevant to [`worldgen-core-boundary.md`](../decisions/worldgen-core-boundary.md) section 8): the version header covers the file format, not the generator.

## 6. Networking

* `network` is a separate crate with a `protocol` crate described in its documentation as an **I/O-free protocol** layer: protocol logic is separate from sockets so that it can run over different backends (TCP, in-process channels for tests, QUIC; a pure UDP protocol is planned). A channel pairs a read and a write protocol. Streams are opened and closed through the send side, bound to one channel (taking over streams after a channel break is planned), carry priorities, and are flushed with a per-second bandwidth budget supplied by the caller. A handshake exchanges participant ids and a secret; a network version constant exists (negotiation not examined). Application messages are large enums per direction in `common/net/src/msg/` (not examined in detail).
* Observation: a transport-independent protocol core with prioritised, budgeted streams fits mixed terrain and entity traffic. **REF-04 Needs spike (Gate 7)**, with the sans-I/O idea recorded as **REF-33 Proposed** (testability of the protocol without sockets). The Cube spike has only a loopback TCP protocol (Draft, unauthenticated). Transport choice and mobile behaviour (NAT, radio wake-ups, reconnect) need a spike before any protocol is approved. Not examined: encryption and the authentication flow beyond the README's mention of an authentication server.

## 7. Rendering and meshing (client)

* The client has its own renderer on `wgpu` 27 and `winit` 0.30.12 (`voxygen/Cargo.toml`), with several UI libraries (an immediate-mode UI behind the default `egui-ui` feature, plus retained UI crates from patched forks).
* **Greedy mesher** (`voxygen/src/mesh/greedy.rs`): merges adjacent faces of the same orientation and the same caller-supplied metadata (for terrain, for example, whether the face is under water); the mesher itself has no idea of colour or material. It scans each axis as 2D slices, extends runs in one direction and then in the other while full rows still match, and emits one rectangle per run. **AO, light and glow are not vertex attributes**: for each texel of a face's rectangle the mesher samples neighbouring voxels and stores light, glow and an AO bit in a **texture atlas** (a guillotine allocator, one extra border texel per rectangle; the atlas grows by doubling to a maximum). Quads have four vertices carrying atlas coordinates. Chunk-boundary handling: for chunk rendering the number of vertical planes is one less than the chunk size so that faces on shared chunk edges are not emitted twice. **T-junctions are not mentioned in the source.**
* **Terrain meshing** (`voxygen/src/scene/terrain/mod.rs`): once per tick on the main thread, new or modified chunks are queued together with their 3 x 3 neighbourhood **only when all neighbours are loaded**; modified blocks are classified as colour-only, lighting-only or full changes (lighting changes requeue neighbours within the light propagation distance; colour-only changes can skip a remesh); dispatch picks the pending chunk **closest to the focus point** (ties by request age) with the number of concurrent jobs capped by core count; jobs run on the shared slow-job pool, sample the chunk plus a one-block border and stay queued if the sample is incomplete; results return over a channel and are applied under a **per-frame budget** (a chunks-per-second rate scaled by frame time); a result is dropped if its start tick is older than the chunk's current request or the chunk was unloaded; GPU models are created for opaque and fluid geometry; frustum culling with a plane cache, a small radius always drawn to avoid popping, spiral draw order; a removed chunk may linger briefly so its shadow does not pop.
* Observations for Cube: the neighbour dependency and the boundary double-draw problem are the ones Cube handles with section-level dirty rules and boundary-plane vertices; Veloren's design differs because quads there are plain rectangles whose detail lives in the atlas. **REF-06 Proposed (Gate 4):** background meshing with closest-first dispatch, a per-frame application budget, stale-result dropping by request tick, and meshing only when neighbours are available (Cube meshes synchronously today). **REF-15 Needs spike (Gate 4):** an atlas-based AO/light scheme is a quality feature Cube does not have; it changes the vertex and texture format and interacts with T-junction-free merging. **REF-14 Rejected:** a custom wgpu renderer is not Cube's first client (AOD-02); it stays a possible future replacement behind the adapter (AOD-06).

## 8. Simulation of the whole world (rtsim)

* A coarse, event-driven model of the whole world, including areas outside loaded chunks, for NPC goals, factions, settlements, economy, wildlife, climate and events; reduced tick rates for distant entities; flat keyed tables rather than ECS iteration, chosen to ease persistence and schema migration; a migration rule runs at start (`rtsim/src/lib.rs`). Seed-based generation of its initial state was not confirmed from the excerpt.
* **REF-10 Needs spike (deferred):** relevant only for Gate 8/9 and large; the GDD (B-1) must say what the game needs first.

## 9. Build, profiles and platforms

* Cargo profiles per purpose (`Cargo.toml`): a development profile with optimisation level 2, overflow checks and debug assertions on, abort on panic, and dependencies at level 3; a debug-info variant; a `no_overflow` profile (checks off, with the world and animation crates at level 3) for maths-heavy world generation; a profiling variant of it; `release` with link-time optimisation, abort on panic and line tables for selected crates; a thin-LTO variant; a release-with-debug-info profile for profiling; a bench profile.
* The workspace pins **`nightly-2026-06-13`**.
* README: builds for Windows, macOS and Linux (including ARM64 builds for macOS and Linux); **no mobile, Android, iOS or WebAssembly client is mentioned**; `voxygen/Cargo.toml` has platform sections for macOS and Windows only. Veloren provides **no mobile evidence**.
* Observations: **REF-11 Proposed (Gate 1):** named profiles per purpose (generation-heavy, profiling, release with line tables) are cheap and useful; abort-on-panic interacts with GDExtension and Android and must be tested before adoption. **REF-12 Rejected:** nightly toolchain.

## 10. Other

* Plugins: experimental, sandboxed WebAssembly plugins that are sent to clients when they connect (book). **REF-16 Rejected for the roadmap** (modding is out of scope until the GDD says otherwise).
* Authentication: an official authentication service issues accounts used by most servers (README). Relevant for Gate 9; design not examined.

## 11. Summary of applicability

| ID | Idea | Status |
|---|---|---|
| REF-01 | World generation usable without client/server | Accepted (already true) |
| REF-02 | Shared "common" simulation crate | Proposed |
| REF-03 | Headless server binary | Accepted (AOD-05) |
| REF-04 | Transport crate with prioritised, budgeted streams | Needs spike |
| REF-05 | Cancellable asynchronous chunk generation, dedupe, per-job thread budget | Proposed |
| REF-06 | Background meshing: closest-first, per-frame budget, stale drop, neighbours loaded | Proposed |
| REF-07 | Persist modified data only, per chunk, atomic writes, LRU cache | Needs spike (Cube uses a journal) |
| REF-08 | Deterministic terrain, separate entity spawn step | Proposed |
| REF-09 | World-scale simulation, then per-chunk generation | Needs spike |
| REF-10 | Whole-world coarse simulation for NPCs/factions | Needs spike (deferred) |
| REF-11 | Purpose-built Cargo profiles | Proposed |
| REF-12 | Nightly Rust toolchain | Rejected |
| REF-13 | ECS for gameplay entities | Needs spike |
| REF-14 | Custom wgpu renderer as first client | Rejected |
| REF-15 | Atlas-baked AO/light in greedy meshing | Needs spike |
| REF-16 | WASM plugins | Rejected (out of scope) |
| REF-17 | SQLite for accounts and characters | Proposed |
| REF-18 | Server-side interest management | Proposed |
| REF-19 | Chunk serialisation and compression stage | Proposed |
| REF-30 | Volume read/write traits independent of storage | Proposed |
| REF-32 | Single-player as an embedded server instance | Accepted (consistent with AOD-22; not evidence for it) |
| REF-33 | I/O-free protocol core, testable without sockets | Proposed |
| REF-34 | Hysteresis margin, staggered unload checks, background deallocation | Proposed |
| REF-35 | Versioned override files with fallback readers, backup on unknown version, atomic writes, drop-if-equal optimisation | Proposed |
| REF-37 | Cooperative cancellation polled inside generation | Proposed |

**Not evidence of fitness for Cube:** Veloren is a desktop game with a different art style, scale and team; its persistence is feature-gated and has acknowledged durability gaps; every "Proposed" or "Needs spike" item must be validated against Cube's constraints (mobile first, Godot frontend, 16^3 bricks, deterministic integer generation, server authority) before adoption.

## 12. Separation of statement types

The sections above mix observation and interpretation by necessity. This section separates them. Nothing in Veloren is evidence that a design suits Cube or a mobile platform.

**Facts found in upstream (Fact (upstream), master `7daa95fd`, 2026-10-30)**
* The workspace has 25 members and pins `nightly-2026-06-13`; graphics use `wgpu` 27 and `winit` 0.30.12; the README lists Windows, macOS and Linux builds and no mobile target.
* A `singleplayer` feature of the graphical client embeds the server library.
* Chunks are 32 x 32 blocks with variable-height columns; coordinate conversion uses Euclidean division.
* Terrain generation takes a 32-bit seed and a cancellation callback; dynamic content uses a system-seeded random generator.
* The server keeps a deduplicated pending map with cancel flags, a dedicated slow-job pool with per-job-name limits, a 2-chunk hysteresis margin, staggered unload checks (one sixteenth per tick) and background deallocation.
* Terrain persistence stores block-level overrides per modified chunk (versioned files, newest-to-oldest fallback readers, atomic writes, backup on unknown version), written on unload only; the source states this is not durable across power loss; persistence is feature-gated.
* The greedy mesher merges by caller-supplied metadata and stores AO/light/glow per texel in an atlas; T-junctions are not mentioned in the file read.
* The network crate has an I/O-free protocol core with TCP, in-process and QUIC backends, prioritised streams and a bandwidth budget.

**Agent inferences (may be wrong)**
* Veloren's override store matches the "seed plus modified data" idea at chunk granularity (an instance of variant C1; Cube's approved architecture is C3 with the AOD-21 journal, and C1 is only recommended as the layer baseline, see [`owner-approval-matrix-v1.md`](../decisions/owner-approval-matrix-v1.md)).
* Cancellation, pools and hysteresis are cheap ideas that map to Cube's streamer; the mapping to a mobile CPU budget is untested.
* The write-on-unload policy leaves a durability gap that a journal-first design avoids.

**Recommendations (Proposed, not decisions)**
* The REF-xx statuses in section 11. "Accepted" in this document means only "consistent with an Approved Owner Decision or already true in Cube" (REF-01, REF-03); it is not an owner decision about Veloren.

**Owner decisions**
* None are derived from Veloren. The relevant approved records are AOD-01…24 (AOD-23 sets the licence policy for using Veloren as a reference).

**Unconfirmed hypotheses**
* That Veloren's embedded single-player mode uses the same authority rules as a remote server (not examined).
* That any Veloren technique performs acceptably on Android or iOS (no mobile evidence exists).
* How Veloren handles generator changes between releases (not examined).
* That its meshing, persistence or networking choices are good for Cube's scale, art style or team.
