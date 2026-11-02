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
