# Veloren: Architecture Notes (read-only study)

**Purpose:** architectural observations for the Cube roadmap. **Not** a source of code, assets or names for Cube.
**Source:** Veloren, a multiplayer voxel RPG in Rust. Upstream: https://gitlab.com/veloren/veloren (official book: https://book.veloren.net). Read from a **local clone on the development machine** (`D:\Projects\GitHub\Games\Veloren`, remote `origin` = the upstream URL, `HEAD` `b4af357bec3521783cff411d8c0fa22efe61d681`, commit date 2025-03-17, working tree clean). Nothing was modified, built or run.
**License:** GNU General Public License v3 (`LICENSE` in the clone). Cube must not copy, translate or closely paraphrase Veloren source; this document records structure and design ideas only, in our own words, with file-level references so the owner can verify them.
**What was read:** workspace `Cargo.toml` (members, profiles), `README.md`, `rust-toolchain`, directory listings of every top-level crate, and the head of a small number of files (`world/src/lib.rs`, `server/src/chunk_generator.rs`, `server/src/terrain_persistence.rs`, `common/src/slowjob.rs`, `common/src/terrain/mod.rs`, `voxygen/src/mesh/greedy.rs`, `voxygen/src/scene/terrain/mod.rs`, `rtsim/src/lib.rs`, `voxygen/Cargo.toml`). It was **not** a code review; statements below are limited to what those files show. Anything not read is marked "not examined".

Applicability labels used below: **Accepted** (consistent with an Approved Owner Decision or already true in Cube), **Proposed** (suggested for a gate, needs owner/gate approval), **Needs spike** (evidence required before a decision), **Rejected** (not applicable or contrary to a decision).

## 1. Workspace structure

* The workspace has about 24 members: `common` with sub-crates (`assets`, `base`, `dynlib`, `ecs`, `i18n`, `net`, `state`, `systems`, `frontend`, `query_server`), `client`, `server` (+ `agent`), `server-cli`, `voxygen` (+ `anim`, `i18n-helpers`, `egui`), `world`, `network` (+ `protocol`), `rtsim`; plugins live outside the workspace (`plugin/`).
* Division of labour (from names, `README.md` and file heads): `common` = behaviour and data shared by client and server (terrain types, volumes, components, combat, recipes, time, weather, slow-job pool); `world` = world generation and simulation, designed so that a world can be generated and viewed without client or server; `network` = a separate networking crate with its own protocol sub-crate; `client` = the client-side library (connection, state mirror); `voxygen` = the graphical frontend, which the book describes as having no inherent coupling with the client library; `server`/`server-cli` = the server library and its headless binary; `rtsim` = a simulation layer (section 8).
* Observation for Cube: the same separation (shared simulation code, world generation usable alone, headless server binary, frontend decoupled from the client library) is what AOD-04, AOD-05 and AOD-06 require. `cube-worldgen` already depends on Core only. **REF-01 Accepted** (already true), **REF-03 Accepted** (AOD-05), **REF-02 Proposed**: a shared "common" crate for domain types used by both server and client runtime; the Cube layer grid (AU-1.0, QA Pack) must decide where it sits (not decided).

## 2. Terrain model

* Terrain is stored in horizontal chunks of 32 x 32 blocks (a log2 constant of 5 in `common/src/terrain/mod.rs`) with a variable-height column per chunk; the volume layer is built on read/write volume traits and 2D/3D grids of chunks (`common/src/volumes/`, `common/src/vol.rs`). A chunk type with sparse vertical sub-chunks is used (`terrain/chonk.rs`). Coordinates convert between chunk and world positions with floor-division helpers.
* Cube differs: three-dimensional 16^3 bricks (copy-on-write, sparse), chunk = a 16 x 32 x 16 column of two bricks in the spike, negative coordinates handled with Euclidean division. Both are column-ish worlds with a vertical sparse structure.
* Observation: abstracting read/write access behind volume traits lets world generation, meshing and persistence share one interface. **REF-30 Proposed** (Gate 2): define Core read/write view contracts independently of storage layout (already the intent of the Draft Core contracts C-07/C-08). Not a reason to change the 16^3 brick decision.

## 3. World generation

* A world is generated from a numeric seed (the generation entry point in `world/src/lib.rs` takes a 32-bit seed). Generation has two phases: a large one-off world simulation at start (terrain/erosion/rivers/locations, civilisations and sites, using a thread pool; module names `sim/` with erosion and diffusion, `civ/` with economy) and per-chunk generation from that simulation (a per-chunk function in the same file), with layers such as caves, rocks, scatter, shrubs, trees, wildlife and "spots" (`world/src/layer/`) and settlement/dungeon sites (`world/src/site*/`).
* Entity spawn information is produced next to terrain as a separate "supplement" and uses a non-deterministic random source, while terrain columns are deterministic functions of the seed and simulation (seen in the per-chunk generation function of `world/src/lib.rs`).
* Observations for Cube:
  * **REF-09 Needs spike:** biomes, rivers and large-scale structure (Gate 8) need some form of world-scale model; Veloren pays for it with a long server-start simulation. Cube must decide whether to keep the current per-column pure functions (cheap, local, mobile-friendly, but limited for rivers and large features) or add a coarse world-scale pass computed once per seed and stored/derived deterministically. A spike must measure start-up time, memory and determinism on a mobile-class CPU.
  * **REF-08 Proposed (Gate 8):** keep terrain deterministic and produce entity/spawn data in a separate step with its own seeding rules, so terrain determinism is never polluted by gameplay randomness.
  * Veloren's seed is 32-bit; Cube's world seed is 64-bit. No action.
  * Veloren's world crate uses language features that need a nightly toolchain (see section 9). **REF-12 Rejected** (Cube stays on stable Rust).

## 4. Server model

* The server is an entity-component-system application (the `specs` crate, per `common/ecs`, `server/src/sys/`): systems run per tick for terrain, terrain synchronisation, chunk sending and serialisation, entity synchronisation, subscriptions (what each client sees), persistence, loot, waypoints and more. A `presence` and `subscription` pair decides which entities and chunks a client receives; view distance is a server setting with a server-side maximum (`server/src/sys/terrain.rs`).
* Chunk generation is asynchronous: the chunk generator keeps a map of pending chunk positions each with an atomic cancel flag, runs the work on a slow-job pool and returns results through a channel; the pool limits how many threads each named job type may use and warns that I/O-blocking jobs do not belong in it (`server/src/chunk_generator.rs`, `common/src/slowjob.rs`).
* Chunks are serialised and compressed in separate systems before sending (`chunk_serialize`, `chunk_send`; `common/net/src/msg/compression.rs`).
* Observations for Cube:
  * **REF-05 Proposed (Gate 3):** generation queue with per-job cancellation and a thread budget per job class, results delivered to the main tick through a channel; the Cube streamer already has a bounded queue and state machine, but is synchronous and has no cancellation.
  * **REF-18 Proposed (Gate 7):** interest management decided by the server per client (view distance plus permissions).
  * **REF-19 Proposed (Gate 7):** chunk serialisation and compression as an explicit stage.
  * **REF-13 Needs spike (Gate 6/7):** an ECS for entities. Cube's AU-1.0 model is an explicit domain model over geometry assets; whether gameplay entities use an ECS is not decided and is not implied by AOD.

## 5. Persistence

* Terrain modifications are persisted per chunk: only chunks that were modified are saved, with atomic file replacement and an in-memory cache of recently unloaded chunks limited by a block budget (`server/src/terrain_persistence.rs`). The unmodified world is regenerated from the seed.
* Character data is stored in SQLite through separate loader/updater components with migrations (`server/src/persistence/`, `server/src/migrations/`; a migration from Diesel to rusqlite is visible in the file list).
* Observations for Cube: this is a real-world confirmation of the "seed plus modified data" model of AOD-17, at chunk-snapshot granularity instead of an operation journal.
  * **REF-07 Needs spike (Gate 5):** Cube's choice is a versioned delta journal (AOD-17). Chunk-level snapshots of modified chunks are a candidate compaction format; a spike should compare journal replay time, file size and recovery on a mobile device.
  * **REF-17 Proposed (Gate 9):** a relational store (SQLite) for accounts and characters, with migrations as a first-class feature; the exact store is not decided.
  * Not examined: how Veloren handles generator changes between releases or world upgrades (relevant to `worldgen-core-boundary.md` section 8).

## 6. Networking

* A dedicated `network` crate with a `protocol` sub-crate implements participants, channels and independent streams with priorities, over TCP, UDP, QUIC and in-process channels, with handshake and metrics (`network/protocol/src/`: `handshake`, `prio`, `tcp`, `udp`, `quic`, `mpsc`). Application messages are two large message enums, one per direction (in `common/net/src/msg/`), plus component synchronisation helpers (`common/net/src/sync`).
* Observation: separating the transport crate from game messages and giving streams priorities matches the needs of mixed traffic (terrain bulk vs. entity updates). **REF-04 Needs spike (Gate 7):** the Cube spike has a loopback TCP protocol only (Draft, unauthenticated). Transport choice, prioritisation and mobile behaviour (NAT, radio wake-ups, reconnect) need a spike before any protocol is approved. Not examined: encryption and authentication flows beyond the README's mention of an authentication server.

## 7. Rendering and meshing

* The client has its own renderer (`voxygen/src/render`, `wgpu` 0.18 and `winit` 0.28 per `voxygen/Cargo.toml`) and a mesh module (`voxygen/src/mesh/`: `greedy`, `terrain`, `segment`). The greedy mesher works on a configuration of volume access callbacks and bakes ambient occlusion, light and glow per voxel into the result and uses a texture atlas; the documentation in `greedy.rs` explains special handling at chunk boundaries (vertical faces on a chunk border would otherwise be drawn twice).
* Terrain meshing runs on worker threads (a named terrain-meshing job and a mesh-worker response channel in `voxygen/src/scene/terrain/mod.rs`); a comment in that file shows that neighbouring chunks are re-meshed when a chunk appears, because border faces depend on neighbours.
* Observations for Cube: the neighbour-dependency and the boundary double-draw problems are the same ones Cube solved with section-level dirty rules and boundary-plane vertices. **REF-06 Proposed (Gate 4):** background mesh generation with results applied on the main thread, dropping stale results (Cube currently meshes synchronously). **REF-15 Needs spike (Gate 4):** baked AO/light and atlases are quality features Cube does not have; they interact with T-junction-free greedy meshing and are not needed for the mobile benchmark. **REF-14 Rejected:** a custom wgpu renderer is not Cube's first client (AOD-02); it remains a possible future replacement behind the adapter (AOD-06).

## 8. Simulation of the whole world

* `rtsim` is a "real-time simulation": a low-resolution simulation of the whole world, including areas outside loaded chunks, for NPC behaviour, factions and long-running processes; it runs alongside the game (`rtsim/src/lib.rs`, with `ai`, `data`, `gen`, `rule`).
* Observation: relevant only for Gate 8/9 (NPC population, economy) and large. **REF-10 Needs spike (deferred):** an off-screen coarse simulation is a design option for NPCs; it should not be decided before the GDD (B-1) states what the game needs.

## 9. Build, profiles and platforms

* Cargo profiles are tuned per use (`Cargo.toml`): a fast development profile with optimisation level 2 and dependencies at 3; a `no_overflow` variant for the maths-heavy world crate; profiling profiles with debug info; a release profile with link-time optimisation, abort-on-panic and line tables for selected crates.
* The workspace pins a **nightly** Rust toolchain (`rust-toolchain`: `nightly-2025-02-14`) and the world crate uses nightly features.
* Platform support in the README is Windows, macOS and Linux; no Android or iOS target appears in the manifests or documents searched (a text search for Android and aarch64 in `*.toml` and `*.md` found nothing). The graphics stack (wgpu/winit with several patched forks) is a desktop stack.
* Observations: **REF-11 Proposed (Gate 1):** named build profiles per purpose (worldgen-heavy test, profiling, release with line tables) are cheap and useful; note that abort-on-panic interacts with GDExtension (a panic must not unwind through the engine) and has to be tested on Android. **REF-12 Rejected:** nightly toolchain. **Mobile: no evidence.** Veloren cannot be used as a mobile feasibility reference.

## 10. Other

* Plugins: a WebAssembly plugin interface (`plugin/wit`). **REF-16 Rejected for the roadmap** (modding is out of scope until the GDD says otherwise).
* Authentication: an official authentication service issues accounts used by most servers (README). Relevant for Gate 9; design not examined.
* Internationalisation, accessibility and tooling crates exist; not relevant to the current gates.

## 11. Summary of applicability

| ID | Idea | Status |
|---|---|---|
| REF-01 | World generation usable without client/server | Accepted (already true) |
| REF-02 | Shared "common" simulation crate | Proposed |
| REF-03 | Headless server binary | Accepted (AOD-05) |
| REF-04 | Separate transport crate with prioritised streams | Needs spike |
| REF-05 | Cancellable async chunk generation with per-job thread budget | Proposed |
| REF-06 | Background meshing, neighbour re-mesh | Proposed |
| REF-07 | Persist modified chunks only, atomic writes, LRU cache | Needs spike (Cube uses journal) |
| REF-08 | Deterministic terrain vs separate entity supplement | Proposed |
| REF-09 | World-scale simulation then per-chunk generation | Needs spike |
| REF-10 | Whole-world coarse simulation (rtsim) | Needs spike, deferred |
| REF-11 | Purpose-built Cargo profiles | Proposed |
| REF-12 | Nightly toolchain | Rejected |
| REF-13 | ECS for entities | Needs spike |
| REF-14 | Custom wgpu renderer as first client | Rejected |
| REF-15 | Baked AO/light with atlas in greedy meshing | Needs spike |
| REF-16 | WASM plugins | Rejected (out of scope) |
| REF-17 | SQLite for accounts and characters | Proposed |
| REF-18 | Server-side interest management | Proposed |
| REF-19 | Chunk serialisation and compression stage | Proposed |
| REF-30 | Volume read/write traits independent of storage | Proposed |

**Not evidence of fitness for Cube:** Veloren is a desktop game with a different art style, a different world scale and a different team; every "Proposed" or "Needs spike" item above must be validated against Cube's constraints (mobile first, Godot frontend, 16^3 bricks, deterministic integer generation) before adoption.
