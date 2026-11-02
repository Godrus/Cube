# Interactive Godot Client Check (spike)

**Status:** experimental result for branch `spike/rust-godot`. Desktop Windows only. **No mobile readiness is claimed:** Android and iOS are not built.

## Setup

| Item | Value |
|---|---|
| Godot | 4.7.2 stable, windowed (not headless), 1280×720, Compatibility renderer (OpenGL 3.3) |
| GPU | NVIDIA GeForce RTX 3070 Ti (desktop) |
| Extension | debug `cube_godot_adapter.dll` (x86_64-pc-windows-gnu), godot-rust API 4.6 on runtime 4.7.2 |
| Command | `godot --path godot-client --resolution 1280x720 -- --ui-test --shot=<dir>` |
| Method | `godot-client/main.gd` `--ui-test` builds the real scene, then **injects real mouse events** over each button (move away, move onto the button, press, release) and asserts on the display state. Exit code 0 means all checks passed |

The test drives the same button handlers a user would. It cannot judge looks; screenshots are in `screenshots/` and were viewed manually.

## Results (final run plus two earlier identical runs: 19/19 checks pass each, exit 0)

| Check | Result |
|---|---|
| Scene appears, camera is current, `MultiMeshInstance3D` present, six buttons visible, starts with 0 cubes | pass |
| **Fill** (button) → 32 cubes shown, MultiMesh count equals Rust result | pass |
| Second Fill → 80 cubes | pass |
| **Undo** → 32; **Redo** → 80 (MultiMesh updated each time) | pass |
| **Save** (file written under `user://`) | pass |
| **Clear** → 0 cubes | pass |
| **Load** → 80 cubes back, epoch 1 (new epoch after load) | pass |
| Large fill 32×16×32 = **16 384 cubes**, undo (80), redo (16 384), save+load (16 384) | pass |
| Camera outside the filled volume and background visible in the frame (checks added in this round, see Finding 1) | pass |
| Engine `ERROR`/`WARNING`/`SCRIPT ERROR` lines in logs, crashes | none in 4 runs |
| Real mouse clicks reached the buttons | 0 fallbacks in the 4 runs after the hover fix (see Finding 2) |

Screenshots: `screenshots/interactive-1-fill.png` (32 cubes), `-2-cleared.png` (empty), `-3-loaded.png` (80 cubes after Load), `-4-big.png` (16 384 cubes).

## Numbers (Windows, RTX 3070 Ti, vsync disabled for the measurement)

FPS is wall-clock over 300 rendered frames after 30 warm-up frames (`Engine.get_frames_per_second()` was rejected: it reports ~1 in a short run). Ranges are over three runs.

| Measure | Value |
|---|---|
| FPS, 32 cubes | 2 680–2 750 (frame ≈ 0.37 ms) |
| FPS, 16 384 cubes | 2 670–2 760 (frame ≈ 0.37 ms) |
| Cubes displayed | 32, 80, 0, 80 (button scenario); 16 384 (large scenario) |

Operation time (median of three runs; **total** = Rust call + GDScript mirror update + MultiMesh rebuild; **Rust** = the extension call only):

| Operation | 80-cube scenario, total | Rust part | 16 384-cube scenario, total |
|---|---|---|---|
| Fill | 1.5 ms | 0.29 ms | 12.2 ms |
| Undo | 0.9 ms | 0.10 ms | 3.1 ms |
| Redo | 0.9 ms | 0.10 ms | 10.9 ms |
| Clear | 0.9 ms | 0.19 ms | — |
| Save | 4.9 ms | 4.9 ms | — |
| Load | 4.8 ms | 3.5 ms | 11.1 ms |

Reading: Rust is a small part of the cost; at 16 384 cubes the time is dominated by the GDScript mirror loop and `MultiMesh` per-instance updates. The cube field is a spike renderer (no culling or meshing).

**These numbers are for a desktop GPU only.** They say nothing about phones.

## Findings

1. **Defect found and fixed:** after a large fill the camera was inside the voxel volume and the screen was solid green (visible in the first screenshot of the large case). The client now frames the camera around the bounding box of the mirror (`_frame_camera()`). New checks: camera is outside the volume, and a corner pixel of the frame equals the clear colour. Verified that both checks **fail** when framing is disabled (2 failures) and pass with it.
2. **Test flake found and fixed:** in one run a real click on a button did not reach it (hover state was stale; the test then fell back to `pressed.emit()` and logged `NOTE`). Clicks now leave and re-enter the button first. Four consecutive runs afterwards had no fallback.
3. **Display limitation:** adjacent voxels of the same material render as one block (no per-voxel edges), so 16 384 cubes look like one box. A visual, not a data problem.
4. **Not covered:** manual human clicking, window resize, touch input, export, release DLL inside Godot, MSVC build, any mobile device or emulator, sustained load, memory use of the client.

## Not claimed

Mobile readiness (Android/iOS not built), production rendering quality, performance on target devices.

## Not to be moved to `master` without an owner decision

Persistence file format, TCP protocol, the undo-expectation rule, the repository layout (`crates/`, `godot-client/`, root Cargo files). The UI test script in `main.gd` is spike code.
