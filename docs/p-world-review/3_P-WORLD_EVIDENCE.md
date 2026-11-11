# P-WORLD evidence: logs, screenshots, results

All paths are under `SCRATCH = C:\Users\Godrus\AppData\Local\Temp\claude\9a9f99d73db6a0d5\fe08ec09-37a6-45d2-8d52-168209f62bea\scratchpad` (a temporary session folder: copy what you want to keep; the repository does not contain these files). Evidence lives outside the worktree on purpose so that the worktree code state stays as it was.

## 1. Locations

| What | Path |
|---|---|
| Four handoff documents | `SCRATCH\p-world-handoff\1_..4_*.md` |
| Four earlier recovery documents | `SCRATCH\prototype-recovery\PROTOTYPE_RECOVERY.md`, `TECHNICAL_ASSESSMENT.md`, `FIRST_PLAYABLE_BACKLOG.md`, `NEXT_ACTIONS.md` |
| Recovery run of the original spike (before P-WORLD) | `SCRATCH\prototype-recovery\_run\` : `clean-build.log`, `smoke-headless.log`, `smoke-windowed.log`, `shots\` (18 screenshots) |
| Final regression (this handoff) | `SCRATCH\prototype-recovery\_run\final-cargo-test.log` |
| Stage A (seed, seams, first measurements) | `_run\stageA\`: `windowed.log` (determinism hashes, frame times per radius), `cargo-test.log`, `seam-seed{1,7,12345,424242}-view{0,1,2}.png` (12 screenshots) |
| Stage B (character, camera, walking) | `_run\stageB\`: `interactive-r1.log`, `-r2.log`, `-r4.log`, `-r6.log`, `cargo-test.log`, `walk-1-spawn.png` ... `walk-5-teleport.png` |
| Stage C (editing, three-process restart) | `_run\stageC-final\` (last run) and `_run\stageC-windowed2\` (earlier passing run): `write.result`, `verify.result`, `corrupt.result`, `expected.json`, `edit-1-start.png` ... `edit-4-after-restart.png`, `world.wst0.bad` (the deliberately damaged save); `_run\stageC\cargo-test.log` |
| Earlier, failing stage C run (kept for honesty) | `_run\stageC-windowed\` (aim through the synthetic mouse did not work in a real window; fixed by `aim_override`; see limitations) |
| Handoff checks (save wins, `--fresh`) | `_run\handoff-check\world.wst0` (a copy of a test save) |
| Scratch build directory | `_run\target` (about 1.3 GB, may be deleted) |

## 2. Test results

| Run | Result |
|---|---|
| `cargo test --workspace` (final) | 152 tests pass (155 result lines include 3 child-process runs); 0 fail |
| of which new in P-WORLD | `world_hash` 3, `seams` 1, `player` unit 5, `player_world` 4, `edit_world` 8 (one of them re-runs itself as 3 child processes) |
| `cargo clippy --workspace --all-targets` | 0 warnings |
| `world_test.tscn --world-smoke` | `WORLDTEST SUMMARY failures=0` |
| `world_seams.tscn` | `SEAMS SUMMARY failures=0` (seeds 1, 7, 12345, 424242: same seed same hash, different seeds differ) |
| `world_interactive.tscn --radius=2` (headless) | `WALK SUMMARY failures=0` |
| `world_interactive.tscn` windowed, radius 1, 2, 4, 6 | `failures=0` each (`stageB\interactive-r*.log`, written at 06:03-06:06, i.e. **before** the stage C code changes); on the committed code only radius 2 was re-run windowed by the Owner Review: `p-world-review\windowed-interactive-r2.log`, `failures=0` |
| `scripts\test-world-persistence.ps1` (windowed, 3 processes) | `RESULT: PASS`; write phase 0 failures, verify 0 failures, corrupt 0 failures; run twice |

Checks of the checks: the world-persistence Rust test fails if the save is written without the edits; the character tests fail when the unloaded-chunk rule or the landing is broken; a unit test with frame times from 0.2 ms to 60 ms reproduces the ground-flicker defect found in stage B.

## 3. What was verified how (separation requested by the owner)

| Kind | Items |
|---|---|
| **Automated** | determinism and hashes; seam faces on 480 borders; 1600 rays against a brute-force march; remove/place rules; collision after edits; walking across 230 chunk borders (20 runs); unloaded-chunk hold; border edits and mesh equality after a rebuild from the save; two-process save and restore in Rust (with a damaged-file control) |
| **Interactive in a real Godot window, synthetic input** (not a person) | camera orbit and zoom; movement relative to the view; jump; 100 voxels walked across 6 chunk borders; teleport onto unloaded chunks; cursor aim, highlight, left click remove, right click place, drag orbits without placing; reach limit; wall blocks walking; geometry consistent with Rust after each edit |
| **Within one process only** | Save/Load buttons in one window (old spike smoke test); all in-process Rust tests; edits before the window is closed |
| **After a full restart** | `test-world-persistence.ps1`: window closed through the OS close request, a new Godot process loaded the save at start: same seed, world hash `fd6662768be811c0` equal, 7 of 7 edits present, geometry equals stored voxels, the character landed in the hole dug before the restart; third process: damaged save kept as `.bad`, fresh world |
| **Not verified** | a human playing; a mouse really moved by a person over the window; a clean-clone start; behaviour on other machines, GPUs or Android/iOS after P-WORLD; an effect of `opt-level=3` alone |

## 4. Measurements (preliminary; RTX 3070 Ti, Godot 4.7.2, dev profile with optimised hot crates)

**Stage A, 400-voxel walk, fixed step, frame time in ms** (`stageA\windowed.log`):

| Radius | Chunks | Average | Worst |
|---|---|---|---|
| 1 | 12 | 2.16 | 6.41 |
| 2 | 30 | 3.10 | 8.53 |
| 4 | 90 | 7.12 | 14.90 |
| 6 | 182 | 12.68 | 21.26 |
| 8 | 306 | 20.19 | 29.65 |

**Stage B, real frames while the character walks 100 voxels** (`stageB\interactive-r*.log`):

| Radius | Average ms | p95 | Worst | Frames over 16.7 ms |
|---|---|---|---|---|
| 1 | 2.28 | 2.08 | 27.0 | 2 of 7319 |
| 2 | 2.93 | 7.58 | 21.4 | 1 of 5693 |
| 4 | 6.79 | 12.12 | 32.5 | 2 of 2467 |
| 6 | 13.54 | 19.44 | 36.2 | **120 of 1241** |

Teleport onto unloaded chunks: the character waits 1.34-1.39 s, then lands on the surface (worst frame 9.5-29.9 ms depending on the run). Radius 6 and 8 exceed the 16.7 ms budget. These numbers are not final, and they were taken on **earlier code states** (stage A numbers at 05:41, stage B numbers at 06:03-06:06; the commit is from 06:41), not on the committed code; run-to-run spread is visible (radius 2 walk average 2.93 ms in stage B, 3.06 ms in the final stage C run, 3.24 ms in the Owner Review re-run).

**Cause of the growth with the radius (measured by the Owner Review, see `p-world-review\OWNER_REVIEW.md`, probe log `p-world-review\probe-output-release.log`):** the HUD calls `world.active_voxels()` every frame; it builds the sorted list of all solid voxels of every loaded chunk (`WorldStreamer::active_voxels` in `crates/cube-worldgen/src/streamer.rs:252`, via `chunk_cells`). One call costs 0.9-1.8 ms at radius 1-2, 4.7 ms at radius 4, 7.8 ms at radius 6 and 13.6 ms at radius 8 (release-built crates; 6-18 ms in the dev profile the Godot editor loads). That alone explains most of the average frame time at every radius. It was not confirmed by removing the call (the code was not to be changed).

## 5. Re-verification by the Owner Review (2026-11-08, on `fea3deb`, clean worktree)

| Item | Result | Log |
|---|---|---|
| `cargo test --workspace -- --nocapture` | 152 unique tests (`--list`), 155 executions, 0 failed; printed statistics reproduce exactly (1600 rays / 1235 hits / 1600 agree; 230 chunk borders walked; 480 seam borders / 615 border faces) | `p-world-review\cargo-test-full.log` |
| Godot headless: smoke, seams, interactive r2 | `failures=0`; the seed hashes equal the earlier ones (`9bdb2952a9bcc6aa`, `192015189f619f44`, `1c6e670cbf4db1bf`, `88cca42ade3f5429`) | `p-world-review\headless-*.log` |
| Three-process window test | `RESULT: PASS`; `expected.json` identical to the earlier run (hash `fd6662768be811c0`, same 7 edits) | `p-world-review\persist-rerun.log`, `persist-rerun\` |
| Worktree after all of it | still clean | |
