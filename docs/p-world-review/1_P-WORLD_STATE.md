# P-WORLD: final state (experiment, for Owner Review)

**Status:** experiment **complete**, handed over for Owner Review. It is **not** production approval, **not** an approval of the current architecture, does not close PS-0, does not authorise PS-1, and no spike code has been moved into production. Development is stopped; nothing further is started without a separate permission.
**Date of this state:** 2026-11-08. **Authority:** the owner's go-ahead for the L2 experiment "P-WORLD" (Prototype A, B, C) given in this conversation; it is not a registry decision.

## 1. What exists

A runnable procedural voxel world on the existing spike code (`cube-worldgen`, `cube-godot-adapter`, Godot scene `godot-client/world.tscn`):

| Capability | State |
|---|---|
| One command opens a window with a seeded world (`--seed`, `--radius` 0..8) | works (A) |
| Deterministic terrain; same seed, same world, also across separate processes | works (A) |
| Chunk borders without cracks (480 borders, 12 seeds, negative coordinates) | works (A) |
| Third-person camera: right-drag orbit, wheel zoom, movement relative to the view | works (A) |
| A physical character: gravity, jump (Space), voxel collision, one-voxel step-up | works (B) |
| Walking across 5+ chunk borders in all four directions without falling or sticking | works (B) |
| Unloaded chunks: the character waits, never falls through or walks into a missing chunk | works (B) |
| Pick a voxel with the mouse cursor (ray with face), highlight | works (C) |
| Remove (left click) and place (right click without drag, middle click, `E`) blocks | works (C) |
| Mesh and collision change at once; edits on chunk borders update the neighbour | works (C) |
| Autosave when the window is closed; the save is loaded at the next start | works (C) |
| Edits survive a full restart (separate process); a damaged save is refused and kept as `.bad` | works (C) |

## 2. Stages and acceptance (as given by the owner, A and B "conditional")

| Stage | Result | Owner decision |
|---|---|---|
| A: reproducible start, seed and radius, seam checks, first measurements | done | accepted conditionally |
| B: voxel read, character, gravity, collision, chunk-border walking, unloaded-chunk behaviour | done | accepted conditionally |
| C: voxel picking, remove and place, collision and geometry after edits, borders, save, restart | done | experiment declared complete |

Conditions still open (from the owner's notes): no human play-through of the physics, jump and camera feel not judged, the camera can intersect the terrain, radius 6 and above exceeds the 16.7 ms frame budget, a clean-clone run of `run-world.ps1` is not verified, the effect of `opt-level=3` on the dev profile was not measured separately.

## 3. Test and check results (final run, 2026-11-08, worktree unchanged since)

| Check | Result |
|---|---|
| `cargo test --workspace` | **152 tests pass, 0 fail** (the cargo output has 33 `test result` lines adding up to 155, because 3 of them are child-process runs of one test) |
| `cargo clippy --workspace --all-targets` | 0 warnings |
| `world_test.tscn --world-smoke` (old spike smoke, headless) | `failures=0` |
| `world_seams.tscn` (determinism of 4 seeds, headless) | `failures=0` |
| `world_interactive.tscn --radius=2` (character, camera, walking, teleport; headless) | `failures=0` |
| Windowed: `world_interactive.tscn` at radius 1, 2, 4, 6 | all pass (stage B logs) |
| Windowed three-process persistence test (`scripts/test-world-persistence.ps1`) | `RESULT: PASS`, run twice |

What each kind of evidence proves, and what it does not, is in `3_P-WORLD_EVIDENCE.md` and `4_P-WORLD_LIMITATIONS.md`.

## 4. Where the code is

In a separate git worktree, **committed locally as `fea3deb`** (on the owner's instruction "коммит"; 18 files, 2119 insertions, 12 deletions; parent `820f538`), worktree HEAD **detached**, working tree **clean**, **not pushed, not merged**. It is kept by the **local branch `spike/p-world`** (created in the main repository on the owner's instruction; no upstream, no tag) and by two bundles in the permanent archive `D:\Projects\GitHub\Games\Cube-archive\p-world\`. Details, the full file list and the restore commands are in `2_P-WORLD_HANDOFF.md`. The main repository (`D:\Projects\GitHub\Games\Cube`, branch `docs/architecture-roadmap-v1`, HEAD `16db379`) was not modified by P-WORLD: its only local change is the owner's `.gitignore`. Independent Owner Review of this state: `SCRATCH\p-world-review\OWNER_REVIEW.md`.

## 5. Not done, by decision

No push, merge, tag or PR (the commit `fea3deb` and the local branch `spike/p-world` were made on the owner's instructions); no ADG, PS-0, PS-1 or production-document change; no production implementation; no multiplayer, dedicated server, SDF/mesh backend, composite geometry, sculpting, boolean operations or general voxel library; the owner's open notes (section 2) were not worked on.
