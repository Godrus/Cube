# World Generation Spike: Report

**Status:** experimental results. Branch `spike/rust-world-generation`, base commit `24438d6` (tip of `spike/rust-godot`). `master` is unchanged (`bf60534`). Nothing was merged or pushed.

## 1. What was built

New crate `cube-worldgen` (pure Rust, depends on `cube-core` only): `WorldSeed`, `ChunkCoord`, `BiomeId`, `Chunk`, `ChunkState`, `ChunkGenerator` (trait), `TerrainGenerator`, `WorldStreamer`, `world-state-v0`. Adapter additions: `world_bridge.rs` (pure Rust) and the `CubeWorld` GDExtension class. Client: `world.tscn`/`world.gd`. Android: two extra export presets. Description: `README.md`.

## 2. Tests

**Rust: 75 tests pass in the workspace** (24 new in `cube-worldgen`, 2 new bridge integration tests in the adapter, the dependency-rule test extended). `cargo clippy --workspace --all-targets`: no warnings.

Required tests and where they are:

| # | Requirement | Test |
|---|---|---|
| 1 | same seed + coordinate, same chunk | `same_seed_and_coordinate_give_identical_chunk` |
| 2 | different seeds differ | `different_seeds_give_different_terrain` |
| 3 | negative coordinates | `negative_coordinates_work_and_use_floor_division`, `out_of_world_chunk_is_refused` |
| 4 | no duplicate generation | `repeated_updates_never_duplicate_generation` |
| 5 | consistent borders | `neighbouring_chunks_agree_on_their_borders` (3 seeds, 4 positions incl. negatives, both axes) |
| 6 | unload removes from active set | `unloading_removes_chunks_from_the_active_set_and_core` |
| 7 | reload restores | `reloading_restores_identical_content` |
| 8 | failure isolation | `a_failing_chunk_does_not_break_the_others` |
| 9 | independent of load order | `generation_does_not_depend_on_load_order` |
| 10 | no dependency cycles | `internal_graph_has_no_cycle` (in `cube-core/tests/dependency_rules.rs`) |
| 11 | no Godot outside the adapter | `manifests_follow_allowed_edges`, `core_sources_mention_no_engine_types` |

Also covered: bounded queue and deferral, hysteresis, state-machine transition order, edits surviving unload, edit rules, world-state-v0 round trip, corruption and version rejection, restoring chunks kept by the unload margin, noise range/continuity/seed sensitivity.

**Godot (4.7.2, Windows):**

| Run | Result |
|---|---|
| `godot --headless --path godot-client world.tscn -- --world-smoke` | **exit 0**, 16/16 checks (create world; 3x3 chunks; move camera; far chunks unloaded; active count checked each time; edit; save; wander; load; counts and voxel total restored; 5x5) |
| same with the real renderer (RTX 3070 Ti), plus screenshots | exit 0, 16/16 |
| earlier smoke (`--smoke`) and UI test (`--ui-test`) | still pass (exit 0) |

## 3. Measurements

Machine: Intel i5-13600KF, Windows 11, `x86_64-pc-windows-gnu`, release build unless noted. Medians. **Desktop numbers, not mobile.**

**Rust (`cargo run --release -p cube-worldgen --example bench`)**

| Metric | Value |
|---|---|
| Generate one chunk (generator only) | 0.008 ms (debug build 0.145 ms) |
| Load one chunk (generate + Core commit) | 0.009 ms |
| Unload one chunk (Core commit) | 0.001 ms |
| 3x3 chunks (queue + generate + commit) | 0.11 ms, 32 141 voxels, 104 KB of bricks (debug 1.44 ms) |
| 5x5 chunks | 0.24 ms, 87 098 voxels, 264 KB (debug 3.87 ms) |
| 7x7 chunks | 0.45 ms, 165 570 voxels, 472 KB |
| Walk 200 chunks with a 5x5 window | 0.048 ms per step (debug 0.80 ms); window holds 30 chunks (5x5 plus the unload-margin column) |
| Peak process memory (working set) | 6.2 MB at the end of all runs, 5.0 MB at start (debug build 14.6 MB) |

Generation is far cheaper than rendering it, so a synchronous generator is adequate for this slice.

**Godot, MultiMesh update (time to fetch the buffer from Rust and build/assign the MultiMesh)**

| Environment | first 3x3 total | per chunk | first 5x5 total | per chunk |
|---|---|---|---|---|
| Desktop windowed, RTX 3070 Ti | 16.7 ms | 1.14 ms | 43.2 ms | 1.04 ms |
| Desktop headless | 14.9 ms | 0.86 ms | 37.6 ms | 0.78 ms |
| Android emulator, x86_64 (native) | 10.6 ms | 1.09 ms | 23.6 ms | 0.87 ms |
| Android emulator, ARM64 (translated) | 206.6 ms (first run, includes translation warm-up) | 17.7 ms | 30.9 ms | 1.07 ms |

Cubes drawn: 32 141 (3x3), 87 098 (5x5). FPS at 5x5: desktop windowed 280 (measured over 300 frames, headless/renderer-less FPS is not reported as a result); Android emulator 60 (vsync-capped, not a measure of headroom).

**Android emulator memory and start (Android 17, 2 GB RAM, 16 KB pages; process includes the Godot engine)**

| Build | Cold start (`am start -W`) | Peak RSS (`VmHWM`) | PSS |
|---|---|---|---|
| ARM64 release (translated) | 2.90 s | 770–772 MB | 552–557 MB |
| x86_64 release (native on emulator) | 0.41–0.44 s | 334–335 MB | 163–166 MB |

**None of these numbers describes a real phone.** The emulator translates ARM64 instructions and renders through a host GPU translator.

Cross-platform determinism evidence: the voxel totals (32 141 for 3x3, 87 098 for 5x5) are identical on desktop, the ARM64 build and the x86_64 build. This supports the integer-only design but is not a proof (it compares counts, not voxel hashes across devices).

## 4. What was verified where

| | Desktop (Windows) | Android emulator, x86_64 | Android emulator, ARM64 (translated) | Real device | iOS |
|---|---|---|---|---|---|
| Rust tests | yes | no | no | no | no |
| Library builds | yes | yes | yes | n/a | **no** |
| Extension loads in Godot | yes | yes | yes | no | no |
| Scripted world check (16 checks) | yes, headless and windowed | yes, 2 runs | yes, 2 runs | no | no |
| Real input (mouse / `adb input tap`) | scripted mouse in the older UI test; keyboard not exercised | taps on Edit and "Jump 64": streaming and edit worked, no crash | not repeated | no | no |
| Crashes / engine errors | none | none | none | no | no |

## 5. Problems found (and what was done)

1. **Saved state did not restore chunks kept by the unload margin** (found by the Godot smoke after moving the camera): loading regenerated only the radius, so 3 of 12 saved chunks "did not come back". Fixed: load now re-queues the saved active set; new Rust test `world_state_v0_restores_chunks_kept_by_the_unload_margin`.
2. **My smoke scenario expected the wrong count** after a short move (hysteresis keeps 12 chunks, not 9). Scenario corrected to a move beyond radius + margin.
3. **Test-mode flag disabled streaming after the check** in the Android "worldtest" build (a tap on "Jump 64" moved the player but loaded nothing; Edit changed 0 voxels). Found from a screenshot; fixed with a separate `smoke_running` flag; re-verified with taps (30 chunks loaded after the jump, Edit changed 27 voxels).
4. **Cosmetic:** faces away from the sun rendered black (no ambient light). Added an ambient light.
5. **Design notes, not defects:** every streamed chunk is a Core commit, so the asset revision grows by one per chunk load/unload; world edits bypass the Editing API (no undo); HUD `fps` in screenshots reads 1–2 because it is `Engine.get_frames_per_second()` in a short run (the measured FPS is reported above).

## 6. Experimental / not for production

All of it: the terrain function and its constants, chunk size and column layout, `world-state-v0`, the synchronous streamer and bounded queue, the cube-per-voxel renderer, the `worldtest`/`autotest` hooks in the client, the debug-key signing of the Android builds.

## 7. Do not move to `master` without a separate owner decision

- `world-state-v0` (or any world save format) and the choice of storing terrain by seed instead of by data.
- The terrain function and chunk layout (changing either invalidates saved worlds).
- Writing world data through the Core commit primitive from outside the Editing API; the per-chunk revision churn.
- The streamer's state machine and defaults as an API for other modules (it is not a contract).
- `CubeWorld` / `WorldBridge` as the client interface.
- Anything touching RevisionToken semantics, undo semantics, the approved persistence format, the TCP protocol, ACR-001/002/003, P0/P1 status (none of these were changed in this branch).

## 8. Next steps (if the owner wants them)

Real ARM64 device run (memory and thermal over minutes of movement); face-culled or greedy-meshed rendering and a draw-call budget; decision on chunk height and view radius per device class; a world-edit path through the Editing API with history; voxel-hash (not count) determinism comparison across devices; iOS build on macOS.


---

# Follow-up stage: determinism proof, WorldGen/Core boundary, face-culled renderer

**Status:** experimental, same branch. `master` unchanged. `world-state-v0`, the world-generation API and the P0/P1 contracts are **not** approved or changed by this stage. The Rust workspace now has **88 tests passing**, `clippy` clean (13 new: 7 in `crates/cube-worldgen/tests/determinism.rs`, 6 in `crates/cube-godot-adapter/tests/chunk_mesh.rs`). WorldGen/Core write-path options: `worldgen-core-boundary.md`.

## 9. Hash-based determinism

`Chunk::stable_hash(seed)` (and `WorldStreamer::chunk_stable_hash`, which hashes what Geometry Core holds) is FNV-1a 64 over a fixed byte stream: magic `CUBECHK1`, `GENERATOR_VERSION` (u32), seed (u64), chunk x/z (i32), then every non-air voxel sorted by (z, y, x) as world x, y, z (i32) and material (u16), all little-endian. It is integer-only, so it is the same on every platform, and it compares full voxel content, not counts.

| Required property | Test (`determinism.rs`) | Result |
|---|---|---|
| same seed + same coordinate, same hash | `same_seed_and_coordinate_give_the_same_hash` (169 chunks, 4 repeats) | pass |
| same seed, different load order, same hash | `hash_does_not_depend_on_load_order`: generator in natural, shuffled and reverse order; three streamers reaching the same 7x7 area by different paths (direct, long jumps, 24 small steps with a 5-chunk budget); 49 chunks each | pass |
| different seeds, different hash | `different_seeds_give_different_hashes` (64 seeds x 2 coordinates, no collision) | pass |
| neighbours identical when generated alone and inside an area | `neighbouring_chunks_are_identical_alone_and_inside_an_area` (3 seeds x 3 centres incl. negatives, 25-30 chunks each: hash and voxel-by-voxel comparison against what Core stores); `border_columns_do_not_depend_on_the_generating_chunk` | pass |
| hash covers coordinate, voxels, generator version | `hash_covers_coordinate_voxels_and_generator_version` (169 distinct coordinates; a changed material, a missing voxel, version+1, another coordinate or seed each change the hash; input order does not) | pass |
| accidental generator change is detected | `pinned_hashes_detect_accidental_generator_changes` (4 pinned values incl. coordinate 1,000,000) | pass |

Limit: all of this ran on one machine (Windows x86_64). The hash was **not** compared across devices; the earlier cross-platform evidence is still voxel counts only. A device comparison needs the hash printed from the app (not done); the pinned values are the reference for it.

## 10. Renderers compared

* **cubes (control, unchanged):** one `MultiMeshInstance3D` per chunk, one box instance per non-air voxel (64 B per instance), no culling.
* **faces (experimental):** one `MeshInstance3D` per chunk with an `ArrayMesh` of only the faces whose neighbouring voxel is air. Built in Rust (`WorldBridge::chunk_mesh`): 4 vertices (position, normal, colour) and 6 indices per face. Neighbours across chunk borders are read from the neighbouring chunks (an unloaded neighbour counts as air, below the world as solid), so the four neighbours of a loaded, unloaded or edited chunk are rebuilt too. No greedy merging yet. Geometry Core and persistence are untouched. Select with `--renderer=faces`, the custom feature `faces`, or the in-app "Renderer" button.

Correctness: face counts equal an independent brute-force definition in every chunk (test); winding and normals are checked by test; the screenshots of both renderers differ in at most 38 of 596 736 pixels outside the HUD (`screenshots/world-cubes-*` vs `world-faces-*`).

### 10.1 Elements and data (5x5 area, 87 098 voxels, 25 chunks; identical on every platform)

| | cubes | faces | change |
|---|---|---|---|
| drawable elements | 87 098 cubes | 12 146 faces | 7.2x fewer |
| cube faces submitted | 522 588 (6 per cube) | 12 146 | 43x fewer |
| triangles | 1 045 176 | 24 292 | 43x fewer |
| nodes / draw calls (windowed) | 25 / 40 | 25 / 40 | same |
| in-frame primitives (windowed) | 810 014 | 18 734 | 43x fewer |
| array bytes from Rust (`render_bench`) | 5 443 KB | 2 182 KB | 2.5x smaller |
| 3x3 area, elements | 32 141 | 5 534 | 5.8x fewer |
| 7x7 area, elements | 165 570 | 21 040 | 7.9x fewer |

### 10.2 Desktop (Windows, RTX 3070 Ti, Godot 4.7.2, Rust library built in release)

Correction to section 3: the earlier "MultiMesh about 1 ms per chunk" numbers were taken with the debug-profile Rust library, which is the one the Godot editor binary loads from `cube.gdextension`. With the release library the same cube renderer needs 0.22-0.30 ms per chunk (headless). All numbers below use the release library (the `target/debug` DLL was temporarily replaced by the release build for these runs).

| Metric (2 headless runs unless noted) | cubes | faces |
|---|---|---|
| first 3x3: total / per chunk | 3.5-3.8 ms / 0.28-0.30 ms | 7.1-7.4 ms / 0.68-0.70 ms |
| first 5x5: total / per chunk | 7.7-8.9 ms / 0.22-0.27 ms | 15.8-15.9 ms / 0.55-0.56 ms |
| 5x5 windowed: total / per chunk | 17.6 ms / 0.63 ms | 14.0 ms / 0.48 ms |
| Edit (27 voxels, incl. rebuild), headless / windowed | 0.43-0.45 / 0.46 ms | 2.8-3.0 / 2.4 ms |
| Load (rebuild 9 chunks), headless / windowed | 17.2-17.6 / 19.9 ms | 20.3-21.5 / 19.6 ms |
| FPS at 5x5, windowed | 280 | 280 |
| Godot static memory, windowed / headless | 56.7 / 31.0 MB | 35.4 / 26.9 MB |
| video memory, windowed | 19.5 MB | 13.4 MB |

FPS is equal because both runs sit at the same cap; this scene is not GPU-bound on an RTX 3070 Ti, so desktop FPS says nothing about the GPU saving. On the CPU side faces cost about 2-3x more per chunk to build and about 6x more for an Edit (the edited chunk and its neighbours are re-meshed).

### 10.3 Android emulator (Android 17, 2 GB RAM, host GPU translator), release APKs, 2-3 runs each

| Metric | x86_64 native, cubes | x86_64 native, faces | ARM64 translated, cubes | ARM64 translated, faces |
|---|---|---|---|---|
| peak RSS (`VmHWM`) | 335 MB | 283 MB (-15.5%) | 773 MB | 717-719 MB (-7%) |
| PSS | 161-164 MB | 136 MB (-17%) | 547-551 MB | 519-529 MB (-5%) |
| video memory (Godot) | 31.3 MB | 25.2 MB | not measured | not measured |
| first 5x5 total | 40-44 ms (first run 72) | 38-47 ms | 53-54 ms | 87-94 ms |
| per chunk (5x5) | 1.5-1.65 ms | 1.4-1.6 ms | 1.8-1.9 ms | 3.2-3.5 ms |
| Edit | 1.7-1.8 ms | 8.3-8.6 ms | 35-37 ms | 37-45 ms |
| Load | 11.5-16 ms | 12.6-14.3 ms | 31-32 ms | 45-56 ms |
| FPS (vsync-capped, ~60) | 58 | 57-59 | 56-58 | 52-58 |
| cold start (`am start -W`) | 0.6-2.1 s | 0.6-1.9 s | 4.7-4.9 s | 4.7 s |
| scripted checks | 16/16, 3 runs | 17/17 (one face check added), 3 runs | 16/16, 2 runs | 17/17, 2 runs |

Real touch on the x86_64 faces build (`adb input tap`): Edit (27 voxels changed), Jump 64 (30 chunks, 13 642 faces), and the Renderer button (switch to cubes, same picture): `screenshots/world-android-faces-after-touch.png`, `world-android-cubes-after-switch.png`. Cold-start times differ by up to 3x between identical runs on the emulator, so they are not a renderer effect. Memory is the stable signal. The very first run after a fresh install is slower (dex/verification), hence the "first run" notes.

## 11. Emulator limits

The ARM64 runs execute through instruction translation on an x86_64 emulator, and rendering goes through a host GPU translator, so the ~60 FPS cap and the translator, not a phone GPU, decide the frame rate. CPU-heavy steps on ARM64 (mesh building about 3x slower than native) are translation costs. Memory includes the Godot engine and the emulator's GL layer. No physical ARM64 phone was available (`adb devices` listed none), so **none of this is mobile-ready evidence**: no real-device, thermal or long-run data exist, and iOS is still not built.

## 12. Decision for the next spike

* **Keep faces as the renderer for the next spike**, with cubes retained only as the control. Reasons: 43x fewer triangles, 7x fewer elements, 2.5x smaller arrays, 15-17% less memory on the native emulator build, and pixel-equivalent output.
* **Known costs to fix next:** CPU mesh time per chunk and per Edit is higher (Edit rebuilds neighbours). Next: incremental, dirty-region meshing; building meshes off the main thread once a job contract exists; greedy merging (the 4-vertex-per-face layout could drop to about a quarter of the vertices); byte-sized vertex colours.
* **Not decided here:** the world-generation API, `world-state-v0`, and the WorldGen/Core write path (see `worldgen-core-boundary.md`).
* A run on a physical ARM64 device is still required before any statement about mobile performance.
