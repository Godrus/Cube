# Spike: World Generation (Rust + Godot)

**Status: EXPERIMENTAL.** Branch `spike/rust-world-generation`, cut from `spike/rust-godot` at `24438d6`. World generation here is an experimental **user** of the existing Geometry Core API. It changes no contract: RevisionToken semantics, undo semantics, the approved persistence format (`cube-persistence`), the TCP protocol, ACR-001/002/003 and P0/P1 contract status are untouched. Results and measurements: `report.md`.

## Pipeline

```
WorldSeed -> cube-worldgen (TerrainGenerator, WorldStreamer) -> GeometryAsset (cube-core bricks)
          -> cube-godot-adapter (WorldBridge, CubeWorld) -> Godot (one MultiMesh per chunk)
```

`cube-worldgen` depends on `cube-core` only. Nothing but `cube-godot-adapter` depends on Godot (checked by `crates/cube-core/tests/dependency_rules.rs`).

## Coordinates

| Concept | Definition |
|---|---|
| World voxel | `(x, y, z)` integers, `y` up, 1 voxel = 1 world unit. Negative coordinates are normal |
| Chunk | a column of **16 × 32 × 16** voxels = two stacked 16³ bricks (the Geometry Core brick size) |
| `ChunkCoord{x, z}` | `floor(world / 16)` on each horizontal axis (floor division, so `-1 -> chunk -1`) |
| Bricks of chunk `(cx, cz)` | brick coordinates `(cx, 0, cz)` (y 0..15) and `(cx, 1, cz)` (y 16..31) in the single voxel layer. An all-air brick is not stored |
| Limits | `|chunk| <= 2^20` per axis (`GenError::OutOfWorld` beyond); heights `0..31` |
| Materials | `0` air, `1` grass, `2` dirt, `3` stone, `4` placed by player edit |

## Terrain and determinism

- One biome (`BiomeId::PLAINS`). Column height `h(x, z) = 6 + (n32 * 10 + n16 * 4) / 255`, range `6..19`, where `n32`, `n16` are integer value-noise samples with lattice cells of 32 and 16 voxels.
- Voxel material by `y`: `y > h` air, `y == h` grass, `h-3 <= y < h` dirt, below stone.
- **Rules that make generation deterministic:**
  1. The height is a pure function of `(seed, world x, world z)`. It never uses chunk-local state, the order of generation, time, threads, or floating point.
  2. Noise is integer-only (SplitMix64 hash, fixed-point smoothstep and interpolation), so every platform computes identical values.
  3. Nothing random is drawn from a shared generator; each lattice value is hashed from `(seed, salt, ix, iz)`.
  4. A chunk is a pure function of `(seed, coord)`; edits are stored separately and re-applied after generation.
- **Chunk borders:** adjacent chunks sample the same height function at the border columns, so there is no seam by construction. Tests also bound the step across a border to <= 3 (the terrain's own slope).

### Stable chunk hash

`Chunk::stable_hash(seed)` hashes seed, chunk coordinate, `GENERATOR_VERSION` and every voxel (FNV-1a 64, fixed byte order). Tests in `crates/cube-worldgen/tests/determinism.rs` compare hashes (not counts) across repeats, load orders, seeds and alone-vs-in-area generation. An intentional terrain change must bump `GENERATOR_VERSION` and the pinned values. The WorldGen/Core write-path options are analysed in `worldgen-core-boundary.md`.

## Streaming

`WorldStreamer` keeps the chunks around a centre in a `GeometryAsset`. Synchronous: generation happens in `pump(budget)`; there is no scheduler or thread (the async/job contracts are Draft).

- `set_center_world(x, z)`: unloads chunks farther than `load_radius + unload_margin` (Chebyshev, in chunks; default 1 + 1), then queues missing chunks within `load_radius`, nearest first, up to `max_queue` (default 64; the rest are *deferred* to the next update).
- Duplicate protection: a chunk has at most one entry in the state map, so it can neither be queued twice nor generated twice.
- Chunk commit: one `GeometryAsset::commit` per chunk (all its bricks). Unload: one commit removing them.
- Failures: a generator or commit error marks only that chunk `Failed`; others continue. Failed chunks are not retried automatically; `retry_failed()` re-queues them.
- Edits: `set_voxel` writes through the Core commit primitive and records the edit; edits survive unload and are re-applied when the chunk is regenerated. There is no undo for world edits.

### State machine

```
              set_center (in radius)         pump                 pump ok
 Unloaded ---------------------> Queued ---------> Generating ----------> Ready
     ^                              |                   |                    |
     |      out of range            |                   | error              | out of range
     +------------------------------+                   v                    |
     +--------------------------------------------- Failed                   |
     +----------------------------------------------------------------------+
   Failed --retry_failed--> Queued
```

`Unloaded` means "no entry". `Generating` exists only inside `pump` (it is visible in the trace used by the tests).

## world-state-v0 (temporary save format)

`WorldStreamer::save_state` / `state_v0::load`. **Experimental, temporary, not for production, no compatibility promise**; separate from the approved `cube-persistence` format. Contents: format version (0), seed, streaming config, asset epoch and revision (informational), centre chunk, list of Ready chunks, player edits per chunk, CRC-32. Terrain is **not** stored; it is regenerated from the seed. Loading starts a new asset epoch (old tokens become `Incomparable`) and restores the whole saved active set, including chunks kept by the unload margin. Written atomically (temp file + rename).

## Godot side

- `CubeWorld` (GDExtension class, thin): `create(seed, radius)`, `set_player(x, y, z)`, `pump(budget)`, `take_added()`, `take_removed()`, `take_modified()`, `active_chunks()`, `chunk_voxels(cx, cz)`, `chunk_instance_buffer(cx, cz)`, `edit_around(...)`, `save(path)`, `load(path)`, counts, `last_error()`. Everything behind it is the pure-Rust `WorldBridge`, tested without Godot.
- `godot-client/world.tscn` + `world.gd`: one `MultiMeshInstance3D` per chunk (buffer built in Rust, assigned in one call), a yellow player marker, camera behind the player, HUD (seed, active chunks, nodes, voxels, queue, position, fps), keys WASD/arrows or on-screen buttons, Edit / Save / Load / Jump 64.
- Two renderers: **cubes** (control: every occupied voxel is a cube instance) and **faces** (experimental: only faces whose neighbour voxel is air, one mesh per chunk, no greedy merge, no LOD). Select with `--renderer=faces`, the custom feature `faces`, or the in-app button. Comparison: `report.md` sections 10-12.

## Run

```bash
cargo test --workspace
cargo run --release -p cube-worldgen --example bench
godot --path godot-client world.tscn                                     # interactive
godot --headless --path godot-client world.tscn -- --world-smoke         # scripted check, exit 0/1
godot --path godot-client world.tscn -- --world-smoke --shot=<dir>       # with a renderer, writes screenshots
```

Android (emulator verified, see report): `scripts/build-android.sh arm64 x86_64`, then export the presets "Android ARM64 World Validation" / "Android x86_64 World Validation" (cubes), or the same names with " (faces)" (face-culled renderer, extra feature `faces`) (they set the custom feature `worldtest`, which starts `world.tscn` and runs the scripted check).

## Limitations of this spike

- Synchronous generation, no threads, no priority beyond nearest-first, bounded queue only.
- One biome, height-map terrain only: no caves, rivers, structures, NPCs, biome blending, vertical streaming (a chunk is always 32 high).
- The control renderer draws interior voxels; the faces renderer has no greedy merge and rebuilds neighbours on every change.
- World edits bypass `cube-editing` (no history, no undo) and use the Core commit primitive directly because the Editing API contracts are Draft.
- Each chunk load or unload is a Core commit, so the asset revision grows by one per streamed chunk.
- `world-state-v0` has no migration, no compatibility, no dedup.
- Streaming trigger is the player's chunk only; no prediction of movement, no distance-based priority between several players.
- No multiplayer, authorization or server replication.

## Not production-ready

Everything above. In particular: the world-state-v0 format, the terrain function (any change alters every existing world), the chunk size and layout, the rendering path, and all mobile numbers.

## Needed for mobile validation (not done)

A physical ARM64 Android device (the ARM64 results here come from an emulator with instruction translation), an iOS build on macOS with Xcode, memory and thermal measurements over minutes of movement, a face-culled or meshed renderer (draw-call and memory budgets on target phones), and a decision on chunk size and view radius for the target hardware.
