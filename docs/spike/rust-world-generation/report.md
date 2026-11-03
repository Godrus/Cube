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
