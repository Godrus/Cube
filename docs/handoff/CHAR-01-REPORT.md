# CHAR-01: first high-detail voxel character (report, 2026-11-11)

Branch `feat/voxel-character-proto` from `origin/dev` `b64485f`. Feature `F-CHR-001`. Technical readiness is **not** art approval: only the owner can approve that the model matches the target style.

## What exists

* `crates/cube-character` (new, depends on `cube-core` only): continuous sculpt of the traveller (superellipsoids, boxes, paint/carve operations, patch noise), rasterised on one shared lattice, 8 parts (torso, head, arm_l, arm_r, leg_l, leg_r, cape, sword) with stable names, pivots and parents. Stable `AssetId` `0x5452_4156_454C_0001`, content hash over LOD0 grids. Material palette (31 materials). Collision representation: the existing 0.6 x 1.8 Rust body; the voxels are never colliders.
* LODs: coarsened voxel data (not scaled meshes): LOD0 80, LOD1 40, LOD2 20 voxels per unit (thresholds 3 and 2 of 8 so sword and cape survive). Meshing: face culling plus greedy merge, flat axis-aligned normals, no smoothing.
* `CubeCharacter` (adapter) copies meshes; `godot/character_rig.gd` builds 8 `MeshInstance3D` once (meshes shared by all instances), animates by rotating nodes: idle breathing, walk and faster swing, jump/fall, attack (synchronised with the Rust `attack()` call, sword arc), hit reaction (flinch and tint on health loss). A rig-only fill light (cull mask layer 2) keeps the front readable without changing world lighting. LOD by camera distance (7 / 18 units) or `--lod`.
* Player integration: in `_update_player_and_hud` the rig replaces the old box (position = `player_feet()`, yaw = camera yaw, grounded from Rust). No Rust gameplay state was changed.

## Density comparison (release build, `cargo run --release -p cube-character --example measure`)

| res/unit | tall (voxels) | voxels LOD0 | triangles LOD0/1/2 | mesh KiB LOD0 | build ms |
|---|---|---|---|---|---|
| 32 | 57 | 4 931 | 4 742 / 1 466 / 460 | 287 | 1.7 |
| 48 | 86 | 16 396 | 8 240 / 3 034 / 1 060 | 499 | 4.0 |
| 64 | 115 | 38 759 | 13 032 / 4 748 / 1 724 | 789 | 7.7 |
| 80 (chosen) | 144 | 75 060 | 18 736 / 7 084 / 2 458 | 1 134 | 13.5 |

Reading: at 32 and 48 the eyes are 1 to 2 voxels and the face reads as a mask; at 64 the face is readable; at 80 eyes, brows, nose, mouth and hair clumps read in the close-up. Higher than 80 was not tried; 80 is not claimed to be the maximum sensible density. In the Godot debug library the same build takes about 170 ms (unoptimised Rust), a one-time cost at start.

## Godot measurements (real scene, Godot 4.7.2, opengl3 compatibility renderer, RTX 3070 Ti, vsync off, 120 Hz display; the world is the usual demo world; `--char-bench`)

| instances (+ player) | LOD | avg frame ms | worst ms | draw calls | primitives | VRAM bytes |
|---|---|---|---|---|---|---|
| 0 | - | 7.17 | 7.41 | 76 | 68 382 | 22.50 M |
| 1 | 0 | 7.15 | 7.41 | 84 | 87 388 | 22.50 M |
| 12 | 0 | 7.40 | 7.41 | 172 | 296 454 | 22.50 M |
| 48 | 0 | 7.93 | 8.33 | 460 | 980 670 | 22.50 M |
| 48 | 1 | 7.80 | 8.33 | 460 | 427 656 | 22.50 M |
| 48 | 2 | 7.92 | 8.65 | 460 | 181 284 | 22.50 M |

Honest reading: 8 draw calls per character; on this PC the cost of 48 extra characters is about 0.8 ms, well inside 60 FPS. Frame time seems to be capped near 7 to 8 ms by something other than the characters (the debug Rust library, the display); the table shows the character cost relative to the baseline, not the true GPU limit. **Not measured:** GPU time (no timers in the compatibility renderer), true CPU time per frame (the engine's process monitor gave an implausible value and was dropped), LOD switch cost beyond about 0.01 ms per instance (mesh swap on the main thread), memory on a phone, anything on Android. The VRAM figure is the whole scene; meshes are shared, so it does not grow with instances (it did before sharing: 98.7 M with 48 unshared rigs).

## Screenshots (`docs/screenshots/character-v1/`, taken from the running Godot scene, not generated images)

`front`, `side`, `back`, `face`, `armor`, `lod0`, `lod1`, `lod2` (studio placement on a flat patch of real terrain; the LOD views use the same distance), `game` (third person, real HUD), `move` (walking, real HUD), `attack` (mid-swing, real HUD). `bench.txt` has the table above as printed by the game.

## Comparison with the concept (honest)

Achieved: human proportions and a readable silhouette at all three LODs, small flat voxels (not Minecraft blocks), face with eyes, brows, nose, mouth and cheeks, curly multi-tone hair, tunic with colour patches, gold-edged tabard, belt with buckle, chest strap, pouch, big pauldron with gold trim, red shawl and ragged cape, gauntlets, buckled boots, a two-tone sword.

Not achieved: the concept has roughly twice the voxel density on the face and clothes, much more varied shading, folds and wrinkles in cloth, individually modelled fingers, a more dynamic pose and a diagonal blade; here the pose is a rig of stiff parts (no bending elbows or knees, no spine), the cape is a stepped slab, hands are simple, shading is flat vertex colours with no ambient occlusion, and the legs look slightly short. Colours were tuned by eye only. This is a technically working first prototype, not the target art.

## Checks run

`cargo fmt`, `clippy --workspace --all-targets -D warnings`, `cargo test --workspace` (new: 11 tests in `cube-character`: stable id and hash, determinism, three LODs cheaper, human bounds, no missing faces at every LOD, palette and flat normals, thin parts survive, shared lattice), `cargo xtask archcheck`, `cargo xtask features --check`: all pass locally. Real Godot: `walk_smoke.py` (walk, jump, third person, dig, far streaming, quit and continue), `--combat` (two runs), `--play`, headless `--smoke`: all PASS with the rig as the player body. CI result: see the handoff file and the pull request.

## Android and iOS

Not built, not measured. Present: nothing for Android on `dev` (no export preset, no build script; `rustup target list --installed` shows only `x86_64-pc-windows-gnu`). Missing: Android Rust targets and NDK linking for the GDExtension (`aarch64-linux-android`, `.so` per ABI in `cube.gdextension`), export preset, renderer check (Compatibility/GLES3 is what the preview used; vertex colours and a single unlit-free `StandardMaterial3D` are mobile-safe in principle, the fill light with a cull mask and the optional sun shadow are untested there), asset loading (the character is generated in memory, no files), touch controls, memory budget and performance instrumentation on a device. Decisions taken with mobile in mind: three LODs, 8 draw calls per instance, shared meshes, no per-voxel nodes or colliders, no per-frame rebuild; a budget phone target (30 FPS) is untested. Do not state Android support.

## Reused and created

Reused: the Rust player body and collision, `player_feet()`, `is_grounded()`, `attack()`, the third-person camera, the scene and HUD, the world and the Godot smoke runner, the adapter's index-order convention, `AssetId`. Created: crate `cube-character`, adapter class `CubeCharacter`, `godot/character_rig.gd`, preview and bench options in `main.gd`, an `archcheck` entry, feature `F-CHR-001`. The existing brick greedy mesher (`cube-backend-voxel`) was not reused because it is tied to 16^3 world bricks and cell keys; the character mesher is a small separate module (about 200 lines) and could later be unified if the owner wants one mesher.

## Open

Art approval (owner); equipment slots and a skeleton with bending joints; better cape and cloth; a LOD generator and mobile LOD policy (e.g. forced LOD1 on phones); merging parts of distant instances to cut draw calls; GPU timing; Android build and device test; Godot in CI.

Developer commands: `godot --path godot -- --char-shot <front|side|back|face|armor|lod0|lod1|lod2|game|move|attack> <out.png> --world <dir>`, `-- --char-bench <n> --lod <0..2>`, `--char-res <n>`.
