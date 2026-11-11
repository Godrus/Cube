# P-WORLD Owner Review (independent), commit `fea3deb`

**Reviewer:** the same agent that wrote the code, working from the commit, the logs and fresh runs; a conflict of interest the owner should weigh (findings below are checked against code and measurements, not against my earlier reports). Source code was not changed; no commit, push, merge, branch or tag was made by this review. Only artifacts outside the repository were created (`SCRATCH = ...\scratchpad`): `p-world-review\` (logs, a verified git bundle, a scratch clone with probes) and edits to the four handoff documents in `p-world-handoff\`.

## 0. Verdict: **APPROVE WITH CONDITIONS** (as an L2 experiment result; not production approval)

The experiment does what it claims: a runnable procedural world with a physical character, voxel picking and editing, and persistence across a real restart, and every checked claim reproduced. It does not demonstrate architectural correctness and must not be integrated as it is: see sections 6 and 7.

## 1. Preservation (verified)

| Check | Result |
|---|---|
| Worktree | `D:/Projects/GitHub/Games/Cube-pworld`, linked to `D:/Projects/GitHub/Games/Cube/.git/worktrees/Cube-pworld` (`git worktree list --porcelain`) |
| HEAD | `fea3deb46b49109abae8149d56fddb74610eaefb`, detached; parent `820f538c6da9b5bb9a2c2cb954d410c408b2a110` exists and is an ancestor |
| Commit contents | 18 files, 2119 insertions, 12 deletions; matches the handoff |
| Working tree | clean before, during and after all runs (`status --short` empty; ignored: `target/`, `godot-client/.godot/`) |
| Main repository | HEAD `16db379`, only the owner's ` M .gitignore`; nothing of P-WORLD there |
| Reachability | `git fsck --no-reflogs --unreachable` does not list `fea3deb`: the commit is reachable through the worktree's HEAD. Its reflog has 2 entries |
| Clonable | a local clone of the main repository (no hardlinks) checked out `fea3deb` cleanly: the objects are in the shared object store |
| Not pushed | `origin` = `github.com/Godrus/Cube`; no remote branch contains the commit; no local branch, no tag |
| Backup | `p-world-review\p-world-fea3deb.bundle`, `git bundle verify` ok (requires `820f538`); not a ref |

**Is the commit safe?** Not "safe", only "currently alive": it exists in one place (this disk), held by one detached HEAD. `git worktree remove`, `git worktree prune` (if the folder disappears), or `git checkout <other>` in the worktree would leave it reachable only through reflogs (default expiry 30 days for unreachable entries) and a later `git gc` could delete it; loss of the disk loses it. Mitigation needs the owner's permission (a branch or a tag, or moving the bundle to permanent storage). Nothing was done.

## 2. Code review (findings from reading the full diff, plus the pre-existing code it relies on)

Severity: **H** must fix before integration, **M** should fix, **L** note.

| # | Sev | Finding | Where |
|---|---|---|---|
| C-1 | **H** | **The HUD recomputes all solid voxels of all loaded chunks every frame.** `world.active_voxels()` builds a sorted vector of every solid voxel per chunk. Measured (release crates): 0.9 ms at radius 1, 1.8 ms at 2, 4.7 ms at 4, 7.8 ms at 6, 13.6 ms at 8 per call (6-18 ms in the dev profile the editor loads). This explains most of the average frame time and the radius-6/8 budget overruns; the earlier hypothesis in the handoff is now measured (not confirmed by removing the call: code was frozen). | `godot-client/world.gd:565` (call), `crates/cube-worldgen/src/streamer.rs:252` (`active_voxels`) |
| C-2 | **H** | **Two different mutation paths in one crate.** The old scene goes through `cube-editing` (`bridge.rs`, `cube_editing::Editor`); the world path goes through `WorldStreamer::set_voxel`, which commits straight to the Geometry Core with `operation: None` and keeps its own edit list. Same adapter, two write paths with different invariants. | `crates/cube-godot-adapter/src/bridge.rs:11` vs `src/world_bridge.rs:709` (`edit_voxel`), `crates/cube-worldgen/src/streamer.rs:582` (`set_voxel`) |
| C-3 | M | **`world-state-v0` is a second source of truth next to Core.** Edits live in `WorldStreamer.edits` (a per-chunk list) and are replayed over regenerated terrain; the Core asset holds the live bricks. The save contains seed, config, centre, chunk list and the edit list; nothing ties it to `GENERATOR_VERSION` (the version is only inside the chunk hash, not in the file), so after any terrain change an old save loads without error and silently shows different terrain under the old edits. | `crates/cube-worldgen/src/state_v0.rs` (layout, no generator version), `streamer.rs:582` |
| C-4 | M | **Loading trusts the content of a CRC-valid file.** CRC-32 detects accidents, not malformed or hostile files. Probes on a crafted but CRC-valid file: a centre chunk of 200,000,000 makes `cx * CHUNK_SIZE` overflow (`PANIC` in the dev profile that the Godot library uses, wrap-around in release); edits are inserted without checking that the position lies in the chunk or that `y` is in range; the epoch field drives a loop of that many iterations (4 M iterations = 2-3 ms, so a hang needs about 4 billion). In the Godot library a panic in a `#[func]` is caught and logged, the load returns false, and the client then renames the file to `.bad`. | `crates/cube-worldgen/src/state_v0.rs:244` (epoch loop), `:249` (edits inserted unchecked), `:252` (overflow) |
| C-5 | M | **Unsanitised floats in the body API.** `player_teleport(+inf, 0)` leaves the body at `[inf, 40, 0]` held forever; `player_step` with a NaN direction does not stay in place but snaps the body to the nearest wall edge (seen: x 8.5 to 1.3). `raycast` has no upper bound on `max_dist` (5e6 took 41 ms, so 1e9 would take seconds). GDScript passes finite values today; nothing enforces it. | `world_bridge.rs:575` (`player_teleport`), `player.rs:158` (`move_axis`), `world_bridge.rs:481` (`raycast`) |
| C-6 | M | **Shutdown handling is a single hook.** Autosave runs only on `NOTIFICATION_WM_CLOSE_REQUEST`. Not covered: programmatic `get_tree().quit()`, a crash, kill, mobile pause/background, Ctrl+C. A failed save at close is printed and the window closes anyway (no user-visible error). No periodic autosave. | `godot-client/world.gd` `_notification` (about line 123) |
| C-7 | M | **Load failure handling is blunt.** Any load error (checksum, an unsupported newer version, an I/O error) renames the file to `.bad`; a second damaged file silently overwrites an existing `.bad` (verified); the rename result is ignored. | `world.gd:103` |
| C-8 | L | `_save_world` and `_load_world` print `world.world_hash()` (hash of all chunks: 3.8 ms at radius 2, about 40 ms at radius 8): acceptable once, but it is test instrumentation in the client path. | `world.gd:114`, `:121` |
| C-9 | L | Test hooks live in the production client script: `aim_override`, `edit_log` (grows without bound), `persist_enabled` set to true unconditionally in `_ready`. The undo journal in the bridge also grows without bound (one batch per edit). | `world.gd:30`, `:77`, `:349`; `world_bridge.rs` `edit_voxel` |
| C-10 | L | Physics runs in `_process` with the variable frame time (sub-stepped at most 1/60 s). The jump height and distances depend slightly on the frame rate; fine for a single-player experiment, not for an authoritative simulation. | `world.gd:307` (`_process`), `player.rs:113` |
| C-11 | L | Clippy fix removed negated float comparisons; the unreachable-looking `else` branches in `raycast` (the `None` arm distinguishes three cases that all end the same way) are redundant. | `world_bridge.rs` `raycast`, `None =>` arm |
| C-12 | L | The ground probe snaps the body to the surface within 2 cm; the one-voxel step-up requires clear space for the body at the lifted position. I found no case where the body can end inside solid (tests, 230 borders, a jitter unit test, mutation checks) but the step-up and the probe are heuristics, not proofs. | `player.rs:145`, `:158` |

What I checked and found sound: the raycast (DDA) agrees with a brute-force march on 1600 rays and its face and air-before-hit invariants hold; the remove/place rules (reject inside body, unloaded chunks, out of world height, solid targets, material 0); the border handling (dirty sections include the neighbour, meshes equal a rebuild from the save); edit-list deduplication (`list.retain(|e| e.pos != p)`), so repeated edits of one voxel do not grow the save; the save is atomic (temp file, `sync_all`, rename) and a failed save keeps the old file (pre-existing test).

## 3. Evidence check

| Claim | Verdict |
|---|---|
| 152 tests pass | **Confirmed**: `--list` gives 152 unique tests, 155 executions (3 child-process runs), 0 failed, 0 ignored (`cargo-test-full.log`) |
| 1600 rays / 1235 hits / all agree; 230 borders walked; 480 seam borders and 615 faces | **Confirmed, reproduced identically** (printed statistics in the same log). Earlier these numbers existed only in terminal output, not in saved logs |
| Godot checks `failures=0` | **Confirmed** (headless, new logs) |
| Same seeds give the same hashes across processes | **Confirmed**: the four seed hashes equal the stage A values |
| Three-process persistence test passes | **Confirmed, reproduced**: new run, `expected.json` identical (hash `fd6662768be811c0`, 7 edits), PASS |
| Performance tables | **Valid for the code states they were taken on, not for the commit**: stage A at 05:41, stage B at 06:03-06:06; the commit is 06:41 (stage C added a raycast per frame and HUD lines). One re-run on the commit (radius 2: average 3.24 ms, worst 25.8 ms, 2 frames over 16.7 ms of 5149) is consistent. Run-to-run spread about 10 percent |
| "Radius 2 repeated after the last edits: failures=0" (handoff) | **Was unsupported by a saved log** (it was console output only); now backed by `windowed-interactive-r2.log`; handoff corrected |
| Stage C windowed run "aim through the cursor" | **Only through `aim_override`**: in a real window the synthetic motion event did not move the viewport cursor; stated in the handoff |

**Synthetic:** every interactive test (input injected with `Input.parse_input_event`); the window closing (`CloseMainWindow` is a genuine close request but sent by a script); the frame loop timing of a single GPU.
**Not verified:** a human at the mouse and keyboard; clean-clone start of `run-world.ps1`; another machine, another GPU, Android or iOS after P-WORLD; saves from a different build or terrain version; a world edited far from the centre across a restart; disk-full or read-only save paths; crash during save; two instances sharing one save; the effect of `opt-level=3` alone.
**Reproducibility:** all deterministic results reproduce exactly (hashes, counts); timings vary about 10 percent.
**Limits on integration:** C-2 and C-3 (write path and format), C-1 (per-frame cost), the fixed 32-voxel world, no authority/tick structure, GDScript owning the call order.

## 4. Architectural assessment (deviations; nothing was fixed)

| # | Requirement violated (source) | Where | Consequence | OK for L2? | Needed for production |
|---|---|---|---|---|---|
| A-1 | Never bypass the Editing API and the authoritative mutation boundary; no Core writes outside UserCommit/SystemCommit/RecoveryCommit (`docs/AGENT_HANDOFF.md` hard rule 4) | `streamer.rs:582` (`set_voxel`, `operation: None`), `world_bridge.rs:535-560` (`remove_block`, `place_block`), `:709` (`edit_voxel`) | Player edits have no history, validation, permission, transaction or revision-token semantics; invisible to anything above Core; no undo through the Editing API (only the spike-local journal) | Yes (`TECHNOLOGY-GAME-TRACK-INTEGRATION.md` section 3 rule 5: must be declared, and is) | Route edits through the Editing API with typed operations; define world-generation loading as a system commit or a generated layer (the analysis `docs/spike/rust-world-generation/worldgen-core-boundary.md` lists options A and B; no decision) |
| A-2 | A saved format must not be produced by an experiment (`TECHNOLOGY-GAME-TRACK-INTEGRATION.md` section 3 rule 3; approved persistence is `cube-persistence`) | `state_v0.rs` whole file; `world_bridge.rs:1158` `load` | Temporary format, no migration, no generator version in the file (C-3), second source of truth next to Core | Yes, if never treated as the format | Use the approved persistence format; version the generator inside the saved data; validate on load (C-4) |
| A-3 | Authority lives in Rust; GDScript is UI and glue (AOD-07, cited in the integration document section 3 rule 3) | `world.gd` (`_walk`, `_do_remove`, `_do_place`, `_load_world`, `_save_world`) | The scene script owns streaming order, edit orchestration, the save file path and the shutdown hook; Rust holds rules and physics | Yes | A runtime object in Rust (SRV-01/SRV-03 style, proposed) owning tick, edits, saving and lifecycle; GDScript only sends intents |
| A-4 | Single-writer authoritative tick and a World Operations interface (proposed SR/TC-11 in `docs/owner-decision-m2t-optimization/`) | `player_step`, `edit_voxel`, called from `_process` | No tick, no message-shaped interface, a local call chain; physics depends on the display frame time | Yes | Fixed-step simulation, message-shaped operations |
| A-5 | Threading and job contract (proposed TC-10) | `streamer.rs` (header: synchronous), `world.gd:_stream` | Generation and meshing on the main thread; C-1 shows the real cost is elsewhere, but no job model exists | Yes | A job contract before any background work |
| A-6 | Coordinates and world bounds are an open decision (ODM-11) | `CHUNK_HEIGHT = 32`, `VoxelPos` use, `cx * CHUNK_SIZE` (C-4) | 32 voxels high, overflow at extreme chunk coordinates, no bound check on load | Yes | Decide width, vertical extent and world bound; checked arithmetic |
| A-7 | Validate external data (general; security checklists of the plan) | `state_v0.rs:244-252` (C-4), `world_bridge.rs:575`, `player.rs:158` (C-5), `raycast` (C-5) | Crafted save or non-finite input is mishandled | Yes (no hostile input in a local experiment) | Validate at every boundary |
| A-8 | Spike code is not production (`AGENT_HANDOFF.md` hard rule 5) | whole commit | Test hooks and instrumentation inside the client script (C-8, C-9) | Yes | Separate test entry points; no test hooks in runtime classes |

## 5. Documents

The four handoff documents were checked against the repository and logs and **updated only for**: HEAD `fea3deb`; 18 files (2119 insertions, 12 deletions); clean worktree; detached HEAD; no push, merge, branch, tag; the backup bundle and the reachability caveat; the stale performance statements (now marked as taken on earlier code states) and the measured cause of the frame-time growth; the unsupported "radius 2 repeated" claim; the `.bad` overwrite behaviour; a re-verification table. No production document, ADG, PS-0 or PS-1 file was touched.

## 6. Owner Review summary

1. **Critical findings:** C-1 (per-frame HUD cost explains the radius-6/8 overrun and is trivial in cause but not fixed); C-2 (two mutation paths in one crate). Neither blocks the experiment, both block integration.
2. **Architectural findings:** A-1 to A-8 above; the decisive ones are A-1 (Editing API bypass), A-2/C-3 (temporary format without generator version), A-3 (authority in the client script).
3. **Confirmed results:** 152 tests pass and reproduce exactly; seams on 12 seeds; determinism across processes; walking across 230 borders; edit and collision behaviour; the three-process restart in a real window; the commit is intact, clean and consistent with the handoff.
4. **Unconfirmed claims:** performance on the committed code (only one re-run); human play; clean-clone start; behaviour on other hardware or platforms; failure paths of the save (disk full, crash during save); "the controls feel right".
5. **Preservation risks:** one copy, one detached HEAD, no branch, no tag, not pushed; removal of the worktree or a checkout in it makes the commit reachable only through reflogs. A verified bundle exists in the scratchpad (temporary folder).
6. **Mandatory before any integration:** (1) give the commit a durable name and a second copy (owner's permission); (2) a decision on A-1, the write path (Editing API route versus a defined system commit for generation); (3) the approved persistence format with the generator version inside the data and validated loading (C-3, C-4); (4) remove the per-frame `active_voxels()` and re-measure on the committed code at radius 4, 6 and 8 (C-1); (5) sanitise inputs and cap `raycast` distance (C-5); (6) a lifecycle design for shutdown and failed saves (C-6, C-7); (7) separate test hooks from runtime classes (C-9); (8) a human play-through of the controls and the camera; (9) a clean-clone run of the start script; (10) PS-0/PS-1 conditions and owner decisions for the affected contracts, none of which this experiment satisfies.
7. **Verdict: APPROVE WITH CONDITIONS** as an L2 experiment result and as knowledge (requirements, scenarios, measurements) for production. **Not approved:** merging or reusing this code in production, the current architecture, or the temporary save format.
