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


---

# Stage 3: partial rebuild after Edit (faces renderer)

## 13. Accepted decisions (owner, recorded here)

* **`faces` is the main experimental renderer.**
* **`cubes` is the control / reference renderer only** (kept for comparison and for the pixel-equivalence check; no further work on it).
* **No transfer to `master`.** Nothing from this branch has been merged, and the changes of this stage stay on `spike/rust-world-generation`.
* Greedy meshing is **not** implemented yet; the effect of partial rebuild is measured first.
* Real Android phone and iOS remain unverified platforms. Nothing here is mobile-ready.

## 14. What was implemented

* `WorldStreamer::take_border_touched()` (and `WorldBridge`/`CubeWorld` wrappers, plus `edit_voxel`): every voxel edit is already recorded per chunk (`take_modified`); now an edit on a chunk border column (local x or z equal to 0 or 15) also records the **Ready** neighbour across that border. A corner voxel records two neighbours, never the diagonal chunk (faces only look at the six face neighbours). Unloaded neighbours are not recorded: they are built from scratch when they load.
* Rebuild set for the faces renderer after an edit = edited chunks + `take_border_touched()`. Chunks that are added or removed still rebuild their neighbours (their whole border changes).
* Client switch `--rebuild=all` keeps the old policy (edited chunks + all four neighbours) as the "before" in the same build; `--rebuild=partial` (default) is the new one. The smoke test runs both on a fresh 5x5 world and checks that no drawn mesh is stale (`drawn[k]` equals a freshly computed face count for every chunk).

## 15. Results

### 15.1 Rust tests (97 workspace tests pass, was 88; clippy clean)

New in `crates/cube-godot-adapter/tests/chunk_mesh.rs` (7): `edit_inside_a_chunk_rebuilds_only_that_chunk`; `edit_on_a_border_rebuilds_the_neighbour_and_the_neighbour_really_changes` (removing a border voxel adds exactly one face to the neighbour, proving the neighbour rebuild is needed); `edits_on_every_side_including_negative_coordinates`; `corner_voxel_rebuilds_two_neighbours_but_not_the_diagonal`; `far_chunks_are_never_rebuilt_and_unloaded_neighbours_are_skipped` (1 of 25 chunks rebuilt); `an_edit_that_changes_nothing_reports_nothing`; `partial_rebuild_is_always_sufficient` (120 pseudo-random placements and removals, half on border columns: every chunk outside the rebuild set has an identical mesh before and after, and no edit rebuilds more than 3 chunks).

### 15.2 Edit cost, 5x5 world (25 chunks, 12 152 faces), single-voxel toggles (11 per scenario) and one 3x3x3 block

| Scenario | chunks rebuilt, before -> after | desktop headless, ms (2 runs) before -> after | desktop windowed, ms before -> after |
|---|---|---|---|
| inside a chunk | 5 -> 1 | 2.4-2.6 -> 0.48-0.55 | 1.94 -> 0.41 |
| on a border | 5 -> 2 | 2.4-2.6 -> 0.9-1.2 | 2.08 -> 0.76 |
| on a corner | 5 -> 3 | 2.4-3.0 -> 1.4 | 2.03 -> 1.23 |
| 3x3x3 block (Edit button) | 5 -> 1 | 2.6 -> 0.6-0.75 | 2.63 -> 0.60 |

"Before" is the previous behaviour (all four neighbours rebuilt on every edit); the previous measurement (section 10.2) of 2.4-3.0 ms per Edit matches it. Time includes the Rust edit, mesh building and node replacement in Godot.

Android emulator, `faces` release APKs, 2 runs each, same table (ms, before -> after; the emulator is noisy, so ranges):

| Scenario | x86_64 native | ARM64 translated |
|---|---|---|
| inside a chunk | 10-25 -> 2.2-2.4 | 17-24 -> 3.5-4.7 |
| on a border | 8.5-16 -> 4.8-5.5 | 20-49 -> 6.6-6.9 |
| on a corner | 9-22 -> 5.7-7.4 | 19-46 -> 10.2-10.3 |
| 3x3x3 block | 8.5-11 -> 2.1-4.8 | 15-41 -> 3.4-6.3 |

Faces drawn, correctness and memory:

* Number of faces after each edit sequence is identical under both policies (12 152 after toggles, 12 206 after the block); no stale meshes in any run (`stale=0`, desktop and both Android ABIs).
* Image: screenshots of the final scene after the same edits under both policies (`screenshots/world-faces-edit-all.png`, `world-faces-edit-partial.png`) are **pixel-identical** (0 of 596 736 pixels differ outside the HUD).
* Memory, desktop (Godot static memory at the end of the run, each policy in its own process): 33.8 MB (all) vs 29.5 MB (partial) headless; 37.3 vs 36.8 MB in the combined windowed run (partial measured second). The difference is allocator churn from 5x more mesh replacements, not retained geometry (face counts are equal); I did not isolate the cause further. Android process memory is unchanged by the policy (peak RSS 280-283 MB x86_64, 716-718 MB ARM64, same as section 10.3), as expected because the policies differ only in how many meshes are replaced per edit.
* Godot update time per edit is the dominant cost: it is proportional to the number of rebuilt chunks, hence the 5x / 2.5x / 1.7x drop for inside / border / corner edits.

### 15.3 Real touch on the emulator

x86_64 `faces` APK: `adb input tap` on Edit and Jump 64 worked (log "edit changed 26 voxels": one of the 27 cells already held a placed block from the scripted toggles; Jump streamed new chunks), no errors (`screenshots/world-android-faces-edit-touch.png`).

### 15.4 Audit of `Chunk::stable_hash` serialisation

The hash code uses only `u32`, `u64`, `i32`, `u16` converted with `to_le_bytes` (explicit little-endian) and FNV-1a over the bytes; no `usize`, no floats, no `to_ne_bytes`/`to_be_bytes`, no hash-map iteration, no `Hasher`. Cells are sorted by (z, y, x) before hashing, so brick storage order does not matter. Two new tests enforce this: `hash_matches_an_explicit_little_endian_reference_implementation` compares the function with an independent implementation that serialises with plain shifts (3 seeds incl. `u64::MAX` and coordinates near +-2^20), and `hash_source_uses_only_fixed_width_little_endian_integers` scans the hash functions' source for banned types and calls. Limitation: the tests run on a little-endian host; the guarantee for big-endian hosts rests on the explicit `to_le_bytes` and the shift-based reference, not on a run on such a machine. Cross-device hash comparison is still not done.

## 16. What this stage does not show

Android numbers come from an emulator (ARM64 through translation); there is no physical-device or iOS result. The time per edit is still dominated by re-meshing the whole chunk (2-5 ms on the emulator); incremental meshing of the changed region, greedy merging and off-thread meshing are open and were deliberately not started.


---

# Stage 4: incremental mesh sections

**Status:** experimental, `spike/rust-world-generation`, base `150ca51`. `master` unchanged, nothing merged. Not changed: Geometry Core API, RevisionToken, the approved undo semantics, `world-state-v0`, TCP protocol, persistence format, P0/P1 status, the terrain algorithm. Not added: greedy meshing, background meshing, biomes, multiplayer, SDF/Mesh backend. Nothing here is mobile-ready.

## 17. Mesh sections

A **mesh section** is one 16^3 brick of a chunk: `SectionCoord { chunk, brick }`, brick 0 = y 0..15, brick 1 = y 16..31. A chunk (16x32x16) therefore has two independent sections, indexed by the brick coordinate `(cx, brick, cz)` (the same brick coordinates Geometry Core uses).

* **Faces are stored per section.** `WorldBridge::section_mesh(cx, brick, cz)` builds only the faces of the section's own voxels, reading neighbours from the other brick of the chunk, the neighbouring chunks, and the world limits (below y = 0 counts as solid, above y = 31 as air, an unloaded neighbour chunk as air). `chunk_mesh` (the previous whole-chunk builder, kept as the independent reference) equals section 0 appended with section 1: same faces, same order.
* **Godot:** each chunk is a `Node3D` container with up to two `MeshInstance3D` children (one per non-empty section). A rebuild replaces only that section's node. Draw calls go up (48 instead of 40 in the 5x5 scene, because 50 sections instead of 25 chunk meshes).
* **Rebuild policies (client):** `partial` (default) rebuilds only the dirty sections reported by Rust; `chunk` rebuilds all sections of the edited chunks plus neighbours across an edited border (the previous stage); `all` rebuilds all sections of the edited chunks and of their four neighbours (full rebuild, the original fallback). Select with `--rebuild=<policy>` (Android: `setprop debug.cube.rebuild <policy>`). Added or removed chunks still rebuild all sections of their four neighbours in every policy.

### Dirty-region rule

After an edit at voxel `p`, the dirty region is `p` expanded by one voxel in every direction (the 3x3x3 box). A face of voxel `q` depends only on the six face neighbours of `q`, so only the sections that contain `p` or one of its six face neighbours can change; the 20 diagonal cells of the box cannot change any face and are not used for section selection (otherwise a corner edit would needlessly rebuild the diagonal chunk). Only sections of Ready chunks and inside y 0..31 are reported. `WorldStreamer::take_dirty_sections()` returns them, deduplicated, in order of first occurrence.

Sections rebuilt per type of edit (single voxel; verified by Rust tests and by the Godot smoke test on all three platforms):

| Edit | Sections rebuilt (`partial`) | `chunk` | `all` |
|---|---|---|---|
| inside a section (8,10,8) | 1: own | 2 (1 chunk) | 10 (5 chunks) |
| on a brick boundary (8,15,8) or (8,16,8) | 2: both bricks of the chunk | 2 | 10 |
| on a chunk border (15,10,8) | 2: own + neighbour chunk, same brick | 4 | 10 |
| on a horizontal corner (15,10,15) | 3: own + the two neighbours, never the diagonal chunk | 6 | 10 |
| corner on a brick boundary (15,15,15) | 4: own two bricks + the two neighbours' brick 0 | 6 | 10 |
| chunk border + brick boundary above (0,16,0) | 4: own brick 1 and 0, west neighbour brick 1, north neighbour brick 1 | n/a | n/a |
| y = 0 or y = 31 | 1: nothing outside the world | | |
| 3x3x3 block inside a chunk | 2 (measured) | 2 | 10 |
| no change (same material, air above the world, unloaded chunk) | 0 | 0 | 0 |

Undo/redo: the adapter has a **spike-local inverse-edit journal** (`WorldBridge::undo/redo`, used only by the smoke test and the Undo/Redo buttons). It is not the Editing API undo, does not touch RevisionToken or the undo semantics, is cleared on load and is lost with the process. Three edits undone then redone rebuild 5 sections (`partial`), 8 (`chunk`), 30 (`all`) in each direction.

## 18. Tests

**Rust: 108 workspace tests pass (was 97), `clippy` clean.** New in `crates/cube-godot-adapter/tests/sections.rs` (11): one section for an edit inside a section (both bricks); two sections on a brick boundary; section of the neighbour chunk on every chunk border incl. negative coordinates; corner edits (3 and 4 sections, diagonal never); world limits; unloaded neighbours skipped; empty operation dirties nothing and creates no undo step; far sections never change; `chunk_mesh` equals its two sections appended (faces, order, hash); **incremental equals full rebuild** after 80 random edits (client-style cache refreshed only from dirty sections compared with a full rebuild of all sections after every edit, plus face lists and `content_hash` per chunk); undo/redo restore every mesh exactly and report the same sections.

**Godot smoke** (`--world-smoke --renderer=faces`; all checks pass, `failures=0`, desktop headless, desktop windowed, Android x86_64, Android ARM64): edit inside a section, on a brick boundary, on a chunk border, on a corner, corner on a brick boundary; each checks the number of sections rebuilt and that no drawn mesh is stale (per section, the drawn face count equals a freshly computed one). Then three edits are undone and redone: no stale mesh, face count back to the state before the edits after undo and equal to the edited state after redo, equal voxel counts. Finally the picture is compared across policies. The cubes renderer smoke still passes.

## 19. Pixel comparison

Scene after the same sequence (5 scenarios x 11 toggles, a 3x3x3 block, three edits undone and redone) under `all`, `chunk` and `partial`, captured with the same camera and compared inside the script (3D area only: the HUD strip shows a live fps number, which made the first comparison report 237 differing pixels that are all digits of the fps counter, bounding box x 837-864, y 10-22).

| Platform | `all` vs `partial` | `chunk` vs `partial` |
|---|---|---|
| Windows desktop, RTX 3070 Ti (596 736 px) | 0 differing | 0 differing |
| Android x86_64 emulator (1 977 600 px) | 0 differing | 0 differing |
| Android ARM64 emulator, translated (1 977 600 px) | 0 differing | 0 differing |

The pictures are pixel-identical. Edits are visible in the pictures (the picture after the edits differs from the one before them, checked on desktop). Desktop images: `screenshots/world-faces-edit-all.png`, `-chunk.png`, `-partial.png`.

## 20. Measurements: incremental (`partial`) vs full rebuild (`all`) vs whole chunks (`chunk`)

Same 5x5 world (25 chunks, 12 146-12 212 faces; the face count is identical under all policies after the same edits). Time per edit includes the Rust edit, mesh building (`rust`) and node replacement (`node`) in Godot; 11 toggles per scenario averaged. "Rebuilt" counts sections.

**Windows desktop** (Rust library built in release, 2 headless runs in separate processes, ms; windowed runs agree within 10%):

| Metric | `all` | `chunk` | `partial` |
|---|---|---|---|
| Edit inside a section (sections rebuilt) | 1.34-1.40 (10) | 0.28-0.38 (2) | **0.21-0.24 (1)** |
| Edit on a brick boundary | 1.24-1.43 (10) | 0.29-0.31 (2) | 0.30-0.32 (2) |
| Edit on a chunk border | 1.34-1.36 (10) | 0.52-0.58 (4) | **0.42-0.43 (2)** |
| Edit on a corner | 1.36-1.39 (10) | 0.78-0.82 (6) | **0.63-0.64 (3)** |
| Corner on a brick boundary | 1.30-1.31 (10) | 0.79-0.83 (6) | **0.67-0.70 (4)** |
| 3x3x3 block | 1.66-1.69 (10) | 0.43-0.58 (2) | 0.50-0.51 (2) |
| Undo avg / redo avg (3 edits) | 1.30-1.34 / 1.46-1.49 | 0.32-0.37 / 0.41-0.46 | **0.23-0.24 / 0.35-0.37** |
| of which Godot node update (inside a section) | 0.07-0.08 | 0.03 | 0.01-0.02 |
| Godot static memory (end of the run, separate processes) | 27.2 MB | 27.2 MB | 27.2 MB |
| faces | 12 146-12 212 | same | same |

Rust micro-benchmark (`cargo run --release -p cube-godot-adapter --example render_bench`, 5x5): whole chunk 0.145 ms, **section 0 (y 0-15) 0.121 ms, section 1 (y 16-31) 0.026 ms**. Almost all terrain lies in brick 0 (heights 6-19), so splitting a chunk into two bricks barely helps an edit in brick 0: `partial` vs `chunk` is 0.21-0.24 vs 0.28-0.38 ms inside a section, and equal (0.30 vs 0.29 ms) on a brick boundary. The large gain came from not rebuilding neighbours (`all` -> `chunk` -> `partial`); sections add a modest further gain on borders and corners, and a big one only for edits in the nearly empty upper brick (0.026 ms vs 0.145 ms for the whole chunk).

**Android x86_64 emulator** (native on the emulator, release APK, 2 runs per policy in separate processes, ms):

| Metric | `all` | `chunk` | `partial` |
|---|---|---|---|
| Edit inside a section | 4.3-4.8 | 1.2-1.3 | **0.8-0.9** |
| Edit on a brick boundary | 3.8-4.3 | 1.0 | 1.1-1.2 |
| Edit on a chunk border | 3.9-4.4 | 1.6-1.9 | **1.3-1.4** |
| Edit on a corner | 3.9-4.8 | 2.6-2.7 | **1.8-2.1** |
| Undo avg / redo avg | 3.4-3.9 / 3.9-4.0 | 1.0-1.1 / 1.0-1.1 | **0.72-0.74 / 0.74-0.88** |
| Peak RSS (`VmHWM`) / PSS | 286 MB / 131 MB | 287 MB / 133 MB | 287 MB / 134-137 MB |

**Android ARM64 emulator, translated** (release APK; ranges over all completed runs, 2-4 per policy; absolute values are dominated by the translator, not representative of a phone):

| Metric | `all` | `chunk` | `partial` |
|---|---|---|---|
| Edit inside a section | 10.7-28 | 2.1-11.6 | 1.4-3.5 |
| Edit on a brick boundary | 9.3-18 | 1.9-4.2 | 1.9-2.0 |
| Edit on a chunk border | 10.4-20 | 3.1-9.8 | 2.2-4.4 |
| Undo avg / redo avg | 7.2-26 / 7.5-16.6 | 2.3-4.0 / 2.2-4.6 | 1.3-1.4 / 1.8-3.2 |
| Peak RSS (`VmHWM`) / PSS | 701-754 MB / 523-529 MB | 702-722 MB / 522-524 MB | 701 MB / 527 MB |

Reading the numbers: (1) the ordering `all` > `chunk` >= `partial` holds on every platform; (2) process memory does not depend on the policy (the policies only differ in how many meshes are replaced per edit, and the number of faces is equal); (3) emulator timings vary 2-5x between identical runs (host load, first run after install), so only the ordering and rough ratios (about 3x to 6x fewer ms for `partial` than `all`) can be trusted, not absolute milliseconds; (4) real touch input was already verified in the previous stage and was not repeated here.

## 21. Limitations, problems found, readiness

* **Sections help little in this terrain.** 85% of the meshing time is in brick 0. `section_mesh` still scans the whole 16^3 brick for one edited voxel; a finer local patch (only the faces around the edited voxel) is the next lever, not more sections.
* **Neighbour handling is coarse for added/removed chunks:** both bricks of the four neighbours are rebuilt even though only the border column changed.
* **A section costs a node per brick** (48 draw calls instead of 40 in 5x5): acceptable here, to be revisited with a draw-call budget.
* **Undo/redo is a spike-local journal**, not the Editing API undo; world edits still bypass the Editing API (see `worldgen-core-boundary.md`).
* **Emulator only.** ARM64 runs are translated, timings are noisy, no physical ARM64 phone and no iOS. Memory and timing say nothing about real devices, thermals or long sessions.
* **Problems found during the stage:** (1) the first full-frame picture comparison reported 237 differing pixels, all in the HUD fps text; comparison now excludes the HUD. (2) Several ARM64 runs (first run after install, runs with three policies in one process) did not finish within the initial 4-minute wait, so the first ARM64 table was incomplete; the wait was extended and the missing runs repeated, one policy combination (`partial`, last run) was stopped by hand, so its ARM64 range comes from the earlier completed runs. (3) A shell helper for killing processes (`pkill`) does not exist in the Git Bash used here, so earlier clean-up attempts of the emulator and wait loops silently did nothing and the emulator stayed running after the runs. Clean-up is now done with `adb emu kill` and `taskkill` and was checked (no emulator, qemu or adb process left). (4) A first version of the smoke test contained a check that could not fail (`... or true`); it was replaced by a real comparison and the run was repeated.

**Readiness for greedy meshing:** yes, with conditions. The section is a sound unit (independent faces, an exact dirty rule, proven incremental == full rebuild, identical pictures). Greedy merging inside a section keeps that contract: an edit still rebuilds the dirty sections, only the section builder changes, and the existing equivalence tests (face-area coverage and picture comparison instead of face lists) can validate it. Conditions: measure the section build cost first (greedy adds work per rebuild, while reducing triangles and memory), decide how colours and the shared border faces are merged, and keep the cubes renderer and `--rebuild=all` as references. A physical ARM64 device run is still required before any statement about mobile performance.


---

# Stage 5: greedy meshing and ARM64 crash isolation

**Status:** experimental, `spike/rust-world-generation`, base `5665b62`. `master` unchanged, nothing merged. Not changed: Geometry Core API, RevisionToken, undo semantics, `world-state-v0`, persistence format, TCP protocol, P0/P1 status, the terrain algorithm. No threads, no new biomes, no SDF/Mesh backend. **ARM64 validation status: `inconclusive`** (section 26). **`greedy` is not fully visually equivalent to `faces`** (section 25).

## 22. Greedy meshing

**Modes:** `cubes` (base control, unchanged), `faces` (reference renderer for correctness: one quad per visible unit face), `greedy` (new). The rebuild policy is unchanged (`partial` by default; `chunk` and `all` remain). `faces` and `greedy` use the same dirty-section rule, the same section nodes and the same neighbour data; only the section builder differs. The Renderer button cycles `cubes -> faces -> greedy` in the running app.

**Algorithm (per section, `WorldBridge::section_quads`):**

1. The section's own voxels and a one-voxel padding (other brick, neighbouring chunks, world limits) are loaded into an 18^3 grid, exactly as for `faces`.
2. For each of the six face directions and each of the 16 slices along the normal, a 16x16 mask is filled: the cell holds the material of the voxel if the voxel is solid and its neighbour in the normal direction is empty, otherwise 0. Only the section's own voxels enter the mask.
3. The mask is covered with rectangles: scan `v` outer, `u` inner; at the first unmerged cell, extend along `u` while the cells have the same material, then extend along `v` while the whole row of that width has the same material; clear the rectangle and emit one quad `{normal, voxel (smallest corner), size [w, h], material}`.
4. A quad becomes 4 vertices (same layout and winding as a `faces` face) and 6 indices.

**Material rule:** quads merge only cells of the same material. Different materials are never merged, even when coplanar and adjacent (colour is a function of material; the test slab below checks it).

**Boundary rule:** the mask is the section's own 16x16 slice, so a quad can never leave its section: not across a brick boundary inside a chunk and not across a chunk border. This is by construction, not by a proof of merge correctness across sections, and it is checked by test. Cost: faces on the boundary between two sections are not merged across it (more quads). Neighbouring voxels across the border are read for visibility only (a face towards an occupied neighbour is hidden; towards an unloaded chunk it is drawn). Empty sections produce no quads and no node. Negative coordinates work because everything is expressed in world integer coordinates (floor division).

**Geometric tests (11 new, Rust; 119 workspace tests pass, clippy clean):** `crates/cube-godot-adapter/tests/greedy.rs`. The reference is the list of visible unit faces `(voxel, normal, material)` computed directly from the voxels (`section_unit_faces`) and the `faces` mesh; the greedy quads are expanded to unit faces and compared as sorted lists (position, normal, size, material), so gaps, overlaps (duplicates) and wrong materials fail.

| Requirement | Test |
|---|---|
| greedy and faces give the same visible surface | `greedy_and_faces_represent_the_same_visible_surface_in_every_section` (3 seeds, negative coordinates) |
| same material per quad | `every_quad_has_one_material_and_covers_only_voxels_of_that_material` |
| different materials not merged | `different_materials_are_not_merged` (3x3 slab -> one quad 3x3; centre replaced by dirt -> no quad mixes, dirt 1x1, area 9) |
| chunk borders lose no faces, no cross-section quads | `quads_do_not_cross_section_boundaries_and_chunks_keep_their_border_faces` |
| removing a voxel removes its face | `removing_a_voxel_removes_its_face_and_exposes_the_neighbours` |
| a neighbour chunk change updates the shared border | `a_change_in_the_neighbouring_chunk_updates_the_shared_border` |
| empty sections | `empty_sections_produce_no_quads` |
| deterministic, independent of generation order | `greedy_output_is_deterministic_and_independent_of_generation_order` (different paths and queue budgets) |
| full rebuild == partial rebuild | `partial_rebuild_equals_full_rebuild_of_the_greedy_mesh` (60 random edits, client-style cache vs full rebuild after each edit) |
| undo/redo restore the same mesh | `undo_and_redo_restore_the_same_greedy_mesh` (single edit, 3x3x3, 7x3x7 fill) |
| fewer quads, same covered area | `greedy_reduces_quads_and_keeps_the_covered_area` |

A mutation check confirmed the tests have teeth: merging cells of different materials makes 8 of the 11 fail.

**Godot smoke (all modes pass, `failures=0`):** Fill (`fill_box` across two chunk borders), Edit inside a section, on a brick boundary, on a chunk border, on a corner, 3x3x3 Edit, Undo, Redo, Jump, Save, Load and switching the renderer in the running app (`faces -> greedy -> cubes -> faces`), each with a stale-mesh check (per section, drawn elements equal freshly computed ones) and a check that faces and greedy cover the same number of unit faces (12 428 after the session). Run on desktop (headless and windowed), Android x86_64 and Android ARM64 emulators.

## 23. faces vs greedy

5x5 area (25 chunks, 50 sections, 87 098 voxels, 12 146 visible faces); identical on every platform.

| | cubes | faces | greedy |
|---|---|---|---|
| drawable elements | 87 098 cubes | 12 146 faces | **1 710 quads** (7.1x fewer than faces) |
| triangles | 1 045 176 | 24 292 | **3 420** (7.1x fewer) |
| 3x3 / 7x7 elements | 32 141 / 165 570 | 5 534 / 21 040 | 825 / 3 032 |
| mesh data (Rust arrays, Godot formula 40 B/vertex + 4 B/index) | 5 443 KB | 2 182 KB | **307 KB** (7.1x smaller) |
| draw calls (windowed, 25 chunks) | 40 | 48 | 48 (one node per non-empty section) |
| primitives in frame (windowed) | 810 014 | 18 562 | 3 798 |
| video memory (windowed desktop) | 19.5 MB | 13.4 MB | 12.3 MB |

**Desktop** (Windows, release Rust library, 2 headless runs each; windowed runs agree within 10%):

| Metric | faces | greedy |
|---|---|---|
| full build of 5x5, Godot (total / per chunk) | 10.9-11.1 ms / 0.17-0.18 ms | **8.2-8.5 ms / 0.12 ms** |
| Rust-only build of one chunk (2 sections, `render_bench`) | 0.148 ms | 0.173 ms (+17%) |
| Edit inside a section (`partial`) | 0.21-0.22 ms | 0.16-0.17 ms |
| Edit on a chunk border (`partial`) | 0.43-0.44 ms | 0.35-0.40 ms |
| Fill 9x3x9 across 4 chunks (`partial` / `all`) | 0.76-1.06 / 3.96-4.09 ms | 0.75 / 3.40-3.53 ms |
| Undo avg / redo avg (3 edits, `partial`) | 0.24-0.29 / 0.32 ms | 0.22 / 0.33-0.37 ms |
| Load (rebuild 9 chunks) | 18.3-19.5 ms | 17.7-18.0 ms |
| Godot static memory (end of run) | 27.2 MB | 26.2 MB |

Greedy costs about 17% more Rust time per chunk but hands Godot 7x fewer vertices, so the total Godot-side build is about 30% faster and edits are 5-25% faster on the desktop. The effect is small in absolute terms (sub-millisecond).

**Full vs partial rebuild cost (greedy, desktop):** `all` 1.20-1.26 ms (10 sections) vs `chunk` 0.25 ms (2) vs `partial` 0.16-0.17 ms (1) for an Edit inside a section; the same ordering as for `faces` (section 20).

**Android emulators**, noexec release APKs (section 26), run 1 / run 2, ms:

| Metric | x86_64 faces | x86_64 greedy | ARM64 (translated) faces | ARM64 (translated) greedy |
|---|---|---|---|---|
| full build of 5x5 (total) | 41.3 / 50.8 | 50.9 / 49.8 | 135.0 / 57.1 | 96.1 / 96.3 |
| Edit inside a section (`partial`) | 1.51 / 1.53 | 1.62 / 1.48 | 3.14 / 1.70 | 2.33 / 2.59 |
| Edit on a chunk border (`partial`) | 3.06 / 3.35 | 2.62 / 4.41 | 7.40 / 3.07 | 4.87 / 4.38 |
| Fill (`partial`) | 5.27 / 5.92 | 4.63 / 7.73 | 12.8 / 5.2 | 7.8 / 12.6 |
| Undo avg / redo avg | 1.79 / 1.74, 2.40 / 2.76 | 1.56 / 1.58, 1.70 / 2.07 | 3.3 / 3.6, 1.7 / 1.8 | 2.4 / 2.8, 3.2 / 3.1 |
| Load | 13.9 / 14.5 | 16.9 / 14.7 | 60.0 / 28.4 | 41.6 / 52.4 |
| Godot video memory | 27.2 / 25.2 MB | 26.1 / 24.1 MB | 25.2 / 25.2 MB | 24.1 / 24.1 MB |

On the emulators there is **no consistent CPU-time difference** between faces and greedy: run-to-run variation (2x) is larger than the difference. The consistent effects are geometric: 7x fewer triangles and quads, 3 798 vs 18 562 primitives in a frame.

**Process memory** (`memrun`, 1 run per cell, end of the scripted run; the cubes build does not run the edit/renderer-matrix phases, so the peak is **not** comparable between cubes and the mesh modes; faces and greedy ran the same phases):

| ABI | mode | peak RSS (`VmHWM`) | PSS |
|---|---|---|---|
| x86_64 native | cubes / faces / greedy | 338 / 385 / 383 MB | 161 / 142 / 138 MB |
| ARM64 translated | cubes / faces / greedy | 771 / 823 / 822 MB | 553 / 531 / 532 MB |

Peak RSS is dominated by the engine and by the test's transient allocations (screenshots, both renderers in one process), not by mesh size; faces and greedy differ by less than 1% (the mesh arrays are only 2.2 MB vs 0.3 MB). PSS at the end is 14-17% lower for the mesh modes than for cubes on x86_64.

## 24. Pixel and geometry results

* **Geometry:** exact (section 22 tests): the sorted unit-face lists of faces and greedy are equal in every section, in 3 seeds, after 60 random edits, after undo/redo and independent of generation order.
* **Pictures, same scene and session, faces vs greedy** (3D area below the HUD, per-view): desktop windowed 596 736 px per view; Android 1 977 600 px per view, the 5-view counts below are from run 1 of each ABI and are identical on x86_64 and on ARM64 (translated); the default-view counts are identical in all 10 faces/greedy smoke runs (6 on ARM64, 4 on x86_64).

| View | Desktop: differ / visible (> 16 levels) | Android (both ABIs): differ / visible |
|---|---|---|
| 0 (default camera) | 15 / 2 | 21 / 7 |
| 1 | 55 / 20 | 26 / 6 |
| 2 | 26 / 11 | 22 / 17 |
| 3 | 10 / 4 | 9 / 4 |
| 4 | 38 / 13 | 67 / 43 |
| **total** | **144 of 2 983 680 px (0.0048%), 50 visible (0.0017%)** | **145 of 9 888 000 px (0.0015%), 77 visible (0.0008%)** |

Pictures after the same edit sequence under the three rebuild policies of the same renderer are bit-identical (0 differing pixels, desktop and both ABIs), i.e. incremental rebuild does not change the picture. `faces` vs `cubes` differed in at most 38 of 596 736 pixels in section 10 (the same kind of edge effect was not analysed there).

## 25. Rendering defect: background visible through greedy meshes

The "visible" pixels in section 24 (differ by more than 16 levels) are **pinholes where the background shows through the surface**. They are a **separate visual error of the greedy renderer, not an acceptable difference**: the geometry is equal, the rasterisation is not. Cause (identified from the pixel locations, not proven by a rasteriser trace): a merged quad has T-junctions, i.e. vertices of its neighbours lie in the middle of its edges, and the GPU does not guarantee that two triangles meeting at a T-junction cover exactly the same pixels along the shared line. `faces` has every vertex on the unit lattice and is watertight; the remaining 1-level colour differences are shading rounding.

Fix not implemented. The standard fix (emit every lattice vertex on a quad's boundary and fan-triangulate, so no T-junction remains) was estimated in `render_bench` for the 5x5 area: 17 374 vertices and **13 954 triangles** instead of 3 420 (faces: 24 292), i.e. it would give up about 75% of the triangle saving (greedy would be 1.7x smaller than faces instead of 7.1x). Other options (crack-hiding skirts, slight quad overlap, shared-vertex indexing along long edges) were not tried. **Greedy must therefore not be called fully visually equivalent to faces.** `faces` remains the visual reference.

Smoke thresholds (so the test documents rather than hides the defect): over all views at most 0.01% of the pixels may differ; for the default view also at most 0.001% may differ visibly (view 4 on Android has 43 visible pixels of 1 977 600, i.e. above 0.001%, and is only reported). The actual counts are printed on every run.

## 26. ARM64 emulator crash: isolation (status: inconclusive)

**What happened:** after the greedy commit, every launch of the ARM64 APK on the translated emulator ended in `SIGSEGV` in the render thread (`GLThread`, in `libndk_translation.so`, `berberis_HandleNoExec`), after which the process hung and Android showed "not responding" (ANR). These crashes were first treated as an emulator artefact; the isolation below shows that was premature in the sense that **our test code triggers it**, although the failing component is the emulator's translator. Raw files: `docs/spike/rust-world-generation/arm64-isolation/` (`runs/*.runs.txt` verdict per run, `runs/*.tombstone.txt` crash stack, `logs/` selected logcat, `scripts/iso_run.sh` harness, `minimal-godot/` project, `runs/native-rust-tests.txt`).

**Method:** each launch is one counted run (force-stop, clear logcat, set properties, start, wait for the ready marker, a fatal signal, an ANR, or 10 minutes). No run was retried or hidden; the table gives every launch. "Exit code": an Android app has none, so the table records whether the process was alive at the end, plus signal and ANR flags; native test binaries have real exit codes.

| # | Isolation step | Runs | OK | Failed | Signals |
|---|---|---|---|---|---|
| 1 | Godot headless without a graphics context on ARM64 | not possible | | | The release export ignores command-line intent extras (`--esa command_line_params` shows `parameters []`); no headless Godot on Android here. Replaced by step 2 |
| 2 | **Our Rust code only, no Godot, no graphics**: all test binaries natively under the ARM64 translation (and on x86_64), 3 repetitions | ARM64: 3 x 10 binaries (92 tests per round); x86_64: same | **30 of 30 binaries on each ABI (92 tests per round)** | 0 (the `dependency_rules` binary fails identically on both ABIs, 6 of 6, because it reads the repository sources that do not exist on the device) | none |
| 3a | Minimal Godot scene (a Label), no GDExtension, ARM64, one `OS.execute("getprop")` | 4 | 3 | 1 | SIGSEGV 1 |
| 3b | Minimal 3D scene (meshes, MultiMesh, light), no GDExtension, ARM64, one `OS.execute` | 4 + 10 = 14 | 11 | 3 | SIGSEGV 1, SIGABRT 2 |
| 3c | Minimal 3D scene, **no `OS.execute`**, ARM64 | 10 | **10** | **0** | none |
| 3d | Minimal 3D scene, **four `OS.execute` calls**, ARM64 | 9 (run 10 discarded: another harness overlapped) | 0 | **9** | SIGSEGV 9 (all in `berberis_HandleNoExec`) |
| 4 | Our app, `cubes`, ARM64, `OS.execute` (2 calls) | 4 | 0 | 4 | SIGSEGV 4 (3 in `berberis_HandleNoExec`) |
| 4 | Our app, `faces`, same | 4 | 0 | 4 | SIGSEGV 4 (3) |
| 4 | Our app, `greedy`, same | 4 | 0 | 4 | SIGSEGV 4 (3) |
| 4b | Earlier launches of the same APKs (greedy commit, with retry logic) | 15 | 0 | 15 | SIGSEGV/ANR detected |
| 5 | Our app, **no `OS.execute`** ("noexec" presets), ARM64: `cubes` / `faces` / `greedy` | 3 / 3 / 3 | **9** | **0** | none; every run finished the full smoke, `failures=0` |
| 6 | Same noexec APKs on **x86_64** (no translation): `cubes` / `faces` / `greedy` | 2 / 2 / 2 | **6** | 0 | none; `failures=0` |
| 6b | Earlier x86_64 launches of the `OS.execute` APK (greedy stage) | 5 | 5 | 0 | none |

**Reading:** the crash rate follows the number of `OS.execute` calls (child processes forked by Godot) in the translated process, in a project that contains no Rust and no GDExtension at all: 0 of 10 with none, 4 of 18 with one, 9 of 9 with four; our client made two (renderer and rebuild-policy properties) and failed 27 of 27 times; with the calls removed the same code passes 9 of 9. The same `OS.execute` APK never crashed on x86_64 without translation. So: (a) the cause is **not our Rust code and not our scene**, and the crash is **not caused by greedy** (cubes crashes identically); (b) the trigger is **our test harness code** (`OS.execute("getprop")` to select the renderer and the rebuild policy through system properties), which forks a child while the render thread runs; (c) the component that fails is the emulator's ARM-to-x86 translator, whose handling of fork in a multi-threaded process is the plausible cause (not proven: no root cause inside `libndk_translation` was established, and the crash types vary, SIGSEGV and SIGABRT). Consequence for earlier stages: some of the ARM64 runs that "did not finish" in sections 20-21 were most likely such crashes, which makes the single-run ARM64 numbers there less reliable than stated.

**What was changed:** the renderer and the policy can now also be chosen by custom feature ("faces", "greedy") and the "noexec" validation presets never call `OS.execute`; the property mechanism remains for the x86_64 emulator and is documented in `world.gd`. The six "noexec" presets were added to `export_presets.cfg`.

**What this does not show:** this is still only an emulator. The noexec ARM64 runs (9 of 9) are the first complete ARM64 smoke results with several repetitions since this stage began, but they run under instruction translation on an x86_64 host, rendering through a host GPU translator; they say nothing about a real ARM64 phone. **ARM64 validation status: `inconclusive`.** A physical ARM64 device is required.

## 27. Limitations, remaining problems, readiness

* **Visual defect** (section 25): pinholes through greedy meshes at T-junctions; not fixed.
* **Greedy does not merge across sections:** more quads than a chunk-wide merge (1 710 quads for 12 146 faces); quads on the brick boundary stay separate.
* **Greedy build cost in Rust is +17%;** on the emulators its CPU-time effect is within noise.
* **Peak memory** comparison across modes is confounded by the test phases (section 23).
* **Single-voxel local patching** of a section is still not done: every Edit rebuilds a whole section (0.1 ms desktop, a few ms on the emulators).
* **Test harness** forked processes on Android; fixed for the validation presets, but the property mechanism still exists.
* **Emulator only;** no physical ARM64 phone and no iOS; nothing here is mobile-ready.

**Readiness of greedy as the renderer for the next stage:** geometrically yes (exact, deterministic, incremental-safe, 7x fewer triangles and 7x less mesh data), **visually no** until the T-junction pinholes are fixed or accepted by the owner: the choice is between about 7x fewer triangles with a few visible pixels per frame and a T-junction-free mesh with about 1.7x fewer triangles. Recommendation: keep `faces` as the reference and the safe default, keep `greedy` experimental, decide the T-junction trade-off explicitly (owner decision) and repeat the check on a physical ARM64 phone before any mobile claim.

## 28. Not for `master` without a separate owner decision

Everything in this branch remains experimental. In particular from this stage: the greedy mesher and its `Quad` format; the spike-local inverse-edit journal (`undo`/`redo`) and `fill_box` in the adapter (writes through the Core commit primitive outside the Editing API); `world-state-v0` and the terrain/chunk layout (unchanged but still unapproved); the world-generation API, `CubeWorld`/`WorldBridge`; the Android validation presets (debug-key signing) and the harness scripts; the `OS.execute`-based property mechanism. Nothing touching RevisionToken, undo semantics, persistence format, TCP protocol, ACR-001/002/003 or P0/P1 status was changed.


---

# Stage 6: T-junction fix, greedy equals faces pixel for pixel

**Status:** experimental, `spike/rust-world-generation`, base `239aea8`. `master` unchanged, nothing merged; **greedy is not promoted to the main renderer** (that stays `faces`) and must not go to `master` without a separate owner decision. Not changed: Geometry Core API, RevisionToken, undo semantics, `world-state-v0`, persistence format, TCP protocol, P0/P1 status, the terrain algorithm.

## 29. Decisions recorded (owner)

* `OS.execute` makes the ARM64 translated emulator unstable (section 26). Runs without `OS.execute` had no crashes in the tests performed (ARM64 9/9 in stage 5, 9/9 again in this stage). ARM64 emulator validation stays **`inconclusive`**.
* The `noexec` presets are used for every further automated run. `OS.execute` must not reach any production runtime (it is a test-harness mechanism only; the property lookup is now skipped under the `noexec` feature and must be removed before any non-spike build).
* `greedy` is not the main renderer; `faces` and `cubes` are the reference modes.

## 30. Options evaluated

The pinholes of section 25 are T-junctions: vertices of neighbouring quads lying in the interior of a quad's edge. Costs are triangles of the 5x5 area (87 098 voxels, 12 146 visible faces); `faces` has 24 292.

| # | Option | Result |
|---|---|---|
| 1 | Compatible vertex grid: every lattice point on every quad edge | estimated 13 954 triangles (stage 5), not built; most of the saving is lost |
| 2 | Forced subdivision only where a neighbour has a vertex (minimal conforming: corners of other quads on the edge, plus every lattice point on section-boundary planes) | **8 482 triangles**, implemented as `SeamRule::Minimal`; holes gone, but 25 single-pixel differences remained in 4 extra camera views |
| 3 | Seam strips along section boundaries | not needed: on the six boundary planes of a section every edge carries every lattice point on **both** sides, so adjacent sections match without looking at each other (this also keeps the partial rebuild rule: a section's mesh depends only on its own quads) |
| 4 | Option 2 plus every lattice point along edge stretches that border a surface of another colour or plane | **10 858 triangles**, implemented as `SeamRule::VisibleEdges` (default); **0 differing pixels in all views** |

The edge stretches between two coplanar quads of the same material keep only the corner vertices (they are invisible: same colour, same plane). A stretch borders "another surface" unless the coplanar cell across it holds a visible face of the same material (cells outside the section count as different, which is also what makes the section boundary rule consistent).

Triangulation: each quad rectangle is split along its diagonal `c0 c2`; for a triangle `A B C` with extra points `a_1..a_m` on `A..B` and `b_1..b_n` on `B..C`, a fan from `C` over `A..a_m` plus a fan from `a_m` over `B..b_n..C` cover it exactly with `m + n + 1` triangles and use every boundary vertex (a first attempt with ear clipping left collinear boundary points unused and was dropped).

## 31. What it took to reach 0 pixel differences (found by measurement)

Pixel differences between `faces` and `greedy` had three independent causes; each was identified from the pixel positions and removed:

1. **T-junction pinholes**: background shows through at some edges. Fixed by sections 30 option 2 (holes: 77 visible pixels per 9.9 M before, 0 after).
2. **±1 colour level inside surfaces**: the default Godot material (Burley diffuse, specular) computes lighting from the interpolated fragment position, which differs by float noise between two triangulations of the same plane. The `faces`/`greedy` material now uses Lambert diffuse and no specular, so a flat face has one colour per normal. (The `cubes` material is unchanged; cubes and faces/greedy now differ slightly in shading, which is a rendering choice, not a geometry difference.)
3. **Single-pixel swaps along edges where two different surfaces meet** (25 pixels per 2.98 M with option 2): the GPU snaps vertices to a sub-pixel grid, and an edge subdivided differently in the two meshes lands up to a fraction of a pixel apart. Fixed by option 4: wherever two different-coloured surfaces meet, the edge has the same vertices as in the one-face-per-voxel mesh.

## 32. Tests (128 workspace tests pass, was 119; clippy clean)

New `crates/cube-godot-adapter/tests/tjunction.rs` (9), plus the 11 existing greedy tests (all pass unchanged on quads; mesh-level tests updated to the new vertex counts):

| Requirement | Test |
|---|---|
| geometric comparison of normalised quads | `greedy.rs`: unit-face expansion of the quads equals the reference (position, normal, size, material), in every section, after random edits, undo/redo |
| no T-junction anywhere | `the_whole_greedy_surface_has_no_t_junctions` (3 seeds incl. negative coordinates, both seam rules): no vertex of any section lies strictly inside any triangle edge of any section |
| each quad triangulated exactly | `every_quad_is_triangulated_exactly_without_degenerate_triangles`: `n - 2` triangles, all non-degenerate and clockwise from outside, areas sum to `w*h`, all vertices on the quad rectangle, distinct |
| boundary vertices | `boundary_vertices_follow_the_rule`: a point on a quad edge is a vertex iff the edge lies on a section boundary plane, or the point is a corner of another quad, or (VisibleEdges) it touches a stretch that borders another surface; the rule is re-implemented independently from the reference unit faces |
| two greedy sections joined | `two_greedy_sections_conform_on_a_brick_joint_and_on_a_chunk_border` (busy fills straddling y = 16 and x = 16; brick joint, chunk border, four chunks) |
| neighbouring chunks | same test and `the_whole_greedy_surface...` over all chunk borders |
| different materials | `different_materials_keep_conforming_seams` (material changes next to borders) |
| Edit/Undo/Redo after the fix | `edit_undo_redo_keep_the_mesh_conforming_and_restore_it` (meshes restored exactly, conforming after each step), `partial_rebuild_equals_full_rebuild_and_stays_conforming` (40 random edits, cache refreshed from dirty sections equals full rebuild) |
| triangle saving kept | `conforming_greedy_keeps_most_of_the_triangle_saving` (more than 2x fewer triangles than faces) |

Mutation checks: dropping the corner rule or the boundary-lattice rule fails 6 of 8 tests; disabling the visible-edge rule fails `boundary_vertices_follow_the_rule`.

**Godot smoke** (all modes, `failures=0`; desktop headless x6 runs, desktop windowed x2, Android x86_64 x6, Android ARM64 x9): everything of section 22 plus the pixel comparison below.

## 33. Pixel comparison faces vs greedy: 0 differences

3D area below the HUD, same scene after the same Fill/Edit/Undo/Redo/Save/Jump/Load session, switched in the running app, in 5 camera views:

| Platform | Pixels compared | Differing | Background holes |
|---|---|---|---|
| Windows desktop, windowed (x2 runs, both renderer start modes) | 2 983 680 | **0** | 0 |
| Android x86_64 emulator, noexec (x4 faces/greedy runs) | 9 888 000 each | **0** | 0 |
| Android ARM64 emulator, translated, noexec (x6 faces/greedy runs) | 9 888 000 each | **0** | 0 |

The desktop result was cross-checked outside Godot with an independent image difference (Python/Pillow on the saved PNGs of all 5 views and of the session view): 0 differing pixels. Pictures under the three rebuild policies of one renderer remain bit-identical (section 24). The check is sensitive: the same comparison reported 77 and 25 differing pixels before the fixes of section 31.

Limit of this claim: bit-identity was reached for these scenes, resolutions, views and GPUs (RTX 3070 Ti, emulator GL translator). Two different triangulations are not guaranteed to rasterise identically in general; the guarantee is structural (no T-junction, identical vertices on every edge between different surfaces, flat lighting), not a proof for every camera.

## 34. Costs: faces vs greedy (T-junction free)

5x5 area; the same on every platform:

| | faces | greedy before (stage 5) | **greedy now** |
|---|---|---|---|
| drawable elements | 12 146 faces | 1 710 quads | 1 710 quads |
| vertices | 48 584 | 6 840 | **14 278** |
| triangles | 24 292 | 3 420 | **10 858** (2.2x fewer than faces; 3x more than before) |
| mesh data (Godot formula) | 2 182 KB | 307 KB | **701 KB** (3.2x smaller than faces) |
| primitives in a frame (windowed) | 18 562 | 3 798 | **9 074** |
| video memory (windowed desktop) | 13.4 MB | 12.3 MB | 12.6 MB |
| draw calls | 48 | 48 | 48 |

The fix gives back part of the geometric saving (3 420 -> 10 858 triangles) in exchange for visual equality; 3x3 / 7x7 areas: 4 980 / 18 652 triangles (faces 11 068 / 42 080).

CPU time (the conforming build needs hash sets and a face map per section): Rust per chunk (2 sections) `render_bench`: faces 0.156 ms, greedy **0.261 ms** (+67%; stage 5 greedy: 0.173 ms). Desktop headless, Godot, 2 runs (release Rust library):

| | faces | greedy |
|---|---|---|
| full build 5x5 (total / per chunk) | 10.3-10.4 ms / 0.17 ms | 10.9-11.4 ms / 0.18-0.19 ms |
| Edit inside a section / on a chunk border (`partial`) | 0.21-0.23 / 0.43-0.44 ms | 0.29 / 0.54-0.57 ms |
| Fill 9x3x9 across 4 chunks (`partial`) | 0.75-1.21 ms | 0.90-0.93 ms |
| Undo avg / redo avg | 0.25-0.30 / 0.39-0.40 ms | 0.29-0.31 / 0.34-0.35 ms |
| Load | 4.4 ms | 4.4-4.9 ms |
| Godot static memory | 27.3 MB | 26.4 MB |

`all` / `chunk` / `partial` policies keep their ordering for greedy (Edit inside a section, desktop: 1.78 / 0.37 / 0.29 ms; faces 1.46 / 0.28 / 0.21 ms). Android noexec, run 1 (ms):

| | x86_64 faces | x86_64 greedy | ARM64 faces | ARM64 greedy |
|---|---|---|---|---|
| full build 5x5 | 45.3 | 56.3 | 90.3 | 166.3 |
| Edit inside a section (`partial`) | 2.51 | 1.30 | 2.19 | 2.85 |
| Edit on a chunk border | 2.52 | 2.48 | 4.10 | 5.22 |
| Fill (`partial`) | 4.73 | 6.34 | 7.09 | 7.92 |
| Undo avg / redo avg | 1.53 / 1.34 | 1.69 / 2.72 | 2.37 / 2.77 | 2.84 / 3.06 |
| Load | 15.6 | 17.3 | 42.3 | 73.5 |
| Godot video memory | 25.2 MB | 24.3 MB | 25.2 MB | 24.3 MB |

Run-to-run variation on the emulators is 2x, so only coarse conclusions hold: **greedy is no longer faster than faces in CPU time** (up to 1.5-2x slower in the build on the translated ARM64), and its advantage is in GPU data (3.2x less mesh data, 2x fewer primitives, 2.2x fewer triangles). Process memory (1 run per cell, end of the run; cubes does not run the edit/renderer-matrix phases, so its peak is not comparable): x86_64 peak RSS 335 / 387 / 384 MB (cubes / faces / greedy), PSS 163 / 142 / 137 MB; ARM64 767 / 818 / 818 MB, PSS 545 / 532 / 532 MB.

## 35. Android emulator runs (all without `OS.execute`)

15 launches, every launch counted: **ARM64 translated: 9 of 9 completed the full smoke with `failures=0`** (cubes 3, faces 3, greedy 3); **x86_64: 6 of 6** (2 per mode). No SIGSEGV or ANR in any run; raw verdicts in `tjunction-runs/`. Had the translated emulator crashed it would have been recorded as an emulator limitation, not as an application result; it did not. These results are still emulator results: ARM64 validation remains `inconclusive` until a physical ARM64 device is tested.

## 36. Limitations and decision

* Visual equality is bit-exact only for the tested scenes; it rests on identical vertices along edges between different surfaces, which cost 3x the triangles of the stage 5 mesh.
* Greedy now costs more CPU than faces to build (the saving is GPU data and primitives); on mobile the trade may still be right, but the emulator cannot show it. Possible next steps (not started): cheaper seam bookkeeping (no hash maps), fewer extra vertices on edges whose both sides are the same colour from the camera's viewpoint (not safe), measuring on a real GPU.
* Section-local meshes and brick-sized quads remain: quads do not merge across sections.
* The faces/greedy material changed to Lambert diffuse without specular; a production material would need to decide this explicitly.
* No physical ARM64 phone, no iOS: nothing is mobile-ready.

**Readiness:** `greedy` meets the stated bar (no background through it, same visible surface as `faces`, 0 differing pixels in the compared views on all three platforms, partial rebuild, Edit/Undo/Redo and section/chunk seams correct). It is **not** promoted: it remains an experimental option behind `faces`, and a decision to make it the default belongs to the owner after (a) a real-device run and (b) a review of the CPU cost above. Not for `master`: the greedy mesher, `Quad`/`QuadMesh`/`SeamRule`, the spike-local undo journal and `fill_box`, the `noexec` presets and harness scripts, and everything listed in section 28.


---

# Stage 7: runtime cleanup (no child processes) and Android device validation

**Status:** experimental, `spike/rust-world-generation`, base `379b5b6`. `master` unchanged, nothing merged. **Android ARM64 physical-device validation: `Blocked: no physical ARM64 device`. iOS: `iOS validation: blocked — no macOS/Xcode`. Mobile readiness: not established.** Not for `master`: the greedy renderer, `world-state-v0`, the world-generation API, the spike undo journal, the spike test hooks and entry point, the Android debug signing, the experimental export presets.

## 37. `OS.execute` removed from the runtime

**Where it was:** only in `godot-client/world.gd`, in `_renderer_arg()` (`getprop debug.cube.renderer`) and `_rebuild_arg()` (`getprop debug.cube.rebuild`), added in stages 4 and 5 to select the renderer and rebuild policy by `adb setprop`. Both called `OS.execute("getprop", ...)`, which forks a child process (the cause of the ARM64 emulator crashes, section 26).

**Now:** no `.gd` file in `godot-client/` contains `OS.execute`, `OS.execute_with_pipe`, `OS.create_process`, `OS.create_instance`, `OS.shell_open`, `OS.shell_show_in_file_manager` or `getprop`.

* `world.gd` (the runtime client) contains no test code at all. The scripted smoke test, benchmarks, renderer matrix, soak loop and the button-position printout moved to `godot-client/world_smoke.gd`, a subclass of the client script, started only through the separate scene `world_test.tscn` (`main.gd` starts it for the custom feature `worldtest`; the client export starts `world.tscn` for `worldclient`). `world.gd` keeps only a generic `auto_stream` flag that tests switch off.
* **How the renderer is chosen without a child process:** (1) the Godot user argument `--renderer=<cubes|faces|greedy>` (`godot ... -- --renderer=greedy`), (2) the export preset's custom feature `cubes` / `faces` / `greedy`, (3) default `faces` (the reference renderer). The rebuild policy comes from `--rebuild=<partial|chunk|all>`. The in-app Renderer button cycles the modes at run time. On Android the mode is therefore fixed per APK by the preset. Nothing shells out, reads a system property or starts a process.
* **Export presets** were rebuilt: `Test <abi> <mode>` (feature `worldtest`, includes the test scripts), `Soak <abi> <mode>` (plus a 10-minute loop), `Client <abi> <mode>` (feature `worldclient`; `exclude_filter` removes `world_smoke.gd` and `world_test.tscn`). Scripts are exported as text so packages can be inspected.
* **Guards:** `crates/cube-godot-adapter/tests/no_child_processes.rs` (part of `cargo test`, 131 workspace tests pass, clippy clean) fails when any GDScript uses a process or `getprop` call (comments ignored), when `world.gd` contains test hooks (`WORLDTEST`, `worldtest`, `--world-smoke`, `_smoke`, `soak`, `world_test`), or when a client preset has test features or does not exclude the test scripts. Checked by mutation: appending an `OS.execute` call to `world.gd` makes it fail; a comment mentioning it does not. `scripts/check-apk-no-exec.py` inspects exported APKs: all 14 exported APKs (test, soak, client; both ABIs) pass (no packaged script contains `execute` / `getprop` / `create_process` / `shell_open`, the client APKs contain no `world_smoke` / `world_test`); verified to fail on an APK built from a `world.gd` with an `OS.execute` call. The engine and the Rust binding libraries contain the *names* of OS methods (for example `create_process`) as part of their API tables; no script calls them.
* Host-side scripts (`scripts/device-validation.sh`, the emulator harnesses) call `adb shell getprop`/`dumpsys` **on the host**; that is test tooling outside the app.

## 38. Android validation

**Physical device: not available.** `adb devices` listed no device and no phone is connected to this machine; no result in this branch is from a phone, and none is simulated. Prepared for the owner (details and commands in `build-and-targets.md`): the APKs (`test-*`, `soak-*`, `client-*` for arm64, SHA-256 in that file), the in-app 10-minute soak mode, and `scripts/device-validation.sh`, which refuses to run (`Blocked: no physical ARM64 device`, exit 2) unless exactly one physical ARM64 device is attached and, per mode (`faces`, `greedy`): 3 launches of the scripted test (Fill, Edit, Undo, Redo, Jump, Save, Load, renderer switch, faces-vs-greedy picture comparison), start time (`am start -W`), app timings (first build, Edit, Fill, Undo/Redo, Load), FPS, peak memory (`VmHWM`, `VmRSS`, PSS), battery temperature and thermal status, logcat error lines, SIGSEGV/ANR detection, real touch events with `adb input tap` on Edit / Undo / Redo / Save / Load / Jump / Renderer, and a 10-minute soak with a memory and temperature sample every 60 s.

**What was run instead (emulators only, labelled; not device results):**

| Check | Result |
|---|---|
| ARM64 translated emulator, final `test-faces-arm64` / `test-greedy-arm64`, no `OS.execute` | 3 of 3 and 3 of 3 launches completed the full smoke, `failures=0`, no SIGSEGV, no ANR |
| ARM64 translated emulator, final `client-*-arm64` (no test code) | 3 of 3 and 3 of 3 launches alive after 25 s, no `Fatal signal` in logcat |
| x86_64 emulator, `device-validation.sh --allow-emulator`, faces and greedy | smoke `failures=0`; faces-vs-greedy comparison 0 differing pixels of 9 888 000, 0 holes; **real touch**: 8 `adb input tap` events per mode, 7 of them logged by the app (the Renderer tap has no log line): (`edit changed 26 voxels`, `undo restored 26 voxels`, `redo applied 26 voxels`, `save ok=true`, `load ok=true`, `jump to (72, 8)`, `edit changed 27 voxels`) |
| x86_64 emulator, 10-minute soak of the faces build (in-app loop: walk, Edit, Undo, Redo, Fill every 3 s) | completed (`SOAK DONE`, 199 edits), no crash or ANR, 0 stale meshes, FPS 50-60 (vsync cap), Edit avg 0.95-1.4 ms and max 11 ms, **process RSS flat at 266-269 MB over the 10 minutes, PSS 130-132 MB**, battery temperature 25 C and thermal status 0 throughout (an emulator reports no real thermal behaviour). The greedy soak ran 7.5 minutes in an earlier check (same result, no crash, 0 stale meshes); the full 10 minutes was run for faces only |

Script bugs found and fixed while testing the tooling: the in-app soak length is fixed at 10 minutes (it first waited for a length given on the command line); the button positions printed by the app were viewport pixels, while `adb input tap` needs screen pixels, so the first run sent 8 taps that reached nothing (the log lines came out empty and the script says so); the script now adds the system-bar offset `(screen - viewport) / 2` taken from `wm size`.

## 39. faces vs greedy: what is known and what is not

| Quantity | Known value | From |
|---|---|---|
| triangles (5x5 area) | faces 24 292, greedy 10 858 | exact, any platform |
| draw calls | 48 and 48 | emulator / desktop GL |
| primitives in a frame | 18 562 and 9 074 | emulator / desktop GL |
| mesh data | 2 182 KB and 701 KB | exact |
| memory | PSS 142 / 137 MB (x86_64 emulator), 532 / 532 MB (ARM64 translated) | emulator, section 34 |
| Edit latency (`partial`) | 0.2-0.3 ms desktop, 1.3-2.9 ms emulators | desktop / emulator |
| Load latency | 4.4 ms desktop, 16-73 ms emulators | desktop / emulator |
| visual difference | 0 differing pixels in all compared views, 3 platforms | sections 33 and 38 |
| stability after 10 minutes | faces: 10 min on the x86_64 emulator (stable); greedy: 7.5 min on the x86_64 emulator | emulator only |
| FPS | vsync-capped 60 on emulators | not a measure |

None of these is a phone measurement: GPU, driver, thermal behaviour, memory pressure and the real ARM CPU are unmeasured. FPS on the emulator cannot rank the renderers (both sit on the 60 Hz cap), which is why the comparison above relies on geometry, memory and latency instead.

## 40. Status and decisions

* **`faces` stays the reference renderer and the default; `greedy` stays experimental.** Reasons that remain open: no device measurement of either renderer; greedy's CPU cost is higher (section 34) and its GPU advantage (2.2x fewer triangles, 3.2x less mesh data) is unquantified on a mobile GPU; the 10-minute stability run exists only on an emulator. Greedy is visually identical to faces in all compared scenes.
* **Android ARM64:** `Blocked: no physical ARM64 device`. The ARM64 translated emulator results are real runs of the real ARM64 binaries but do not describe a phone.
* **iOS:** `iOS validation: blocked — no macOS/Xcode.` No iOS build exists and none is claimed.
* **Mobile readiness: not established** (no device, no iOS, no thermal data, no long-run data on a device).
* **Remaining limits:** the soak and the touch checks run on emulators; the per-device numbers the owner will produce with `scripts/device-validation.sh` replace them. The client still contains logging `print` calls (`edit changed ...`) used by the touch check; they are plain output, not process or shell calls. Emulator timings vary by about 2x between identical runs.
