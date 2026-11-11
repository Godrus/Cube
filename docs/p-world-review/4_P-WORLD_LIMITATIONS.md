# P-WORLD: known limitations and architectural deviations

**Status:** for Owner Review. Everything here is experimental spike behaviour. None of it is a decision, a contract or a production design.

## 1. Architectural deviations (must be known before any reuse)

| # | Deviation | Detail and consequence |
|---|---|---|
| D-1 | **Editing bypasses the Editing API.** | Player edits (`remove_block`, `place_block`, `edit_voxel`) write through the Geometry Core commit primitive of the spike streamer, not through `cube-editing` and not through any approved Editing API contract. No history, no `UserCommit`/`SystemCommit`/`RecoveryCommit` distinction, no revision-token semantics for edits, no authorisation. The existing spike already documented this as non-compliant; P-WORLD extends it (raycast, remove, place). It violates `docs/AGENT_HANDOFF.md` hard rule 4 as a production rule; it is permitted here only because this is an isolated L2 experiment (`TECHNOLOGY-GAME-TRACK-INTEGRATION.md` section 3, rule 5). |
| D-2 | **Temporary save format `world-state-v0`.** | The save stores the seed, streaming configuration, the centre chunk, the list of loaded chunks and the edits per chunk with a CRC-32; terrain is not stored but regenerated from the seed. No migration, no compatibility promise, not the approved `cube-persistence` format. Any change to the terrain function or `GENERATOR_VERSION` makes existing saves wrong. It must not become a product format (`TECHNOLOGY-GAME-TRACK-INTEGRATION.md` section 3, rule 3). |
| D-3 | **The terrain function and the chunk layout are experimental.** | One biome, height map only (heights 6..19), fixed world height of 32 voxels (two stacked 16-cubed bricks), chunk 16x32x16. Open decision: `VoxelPos` width, vertical extent, world bound (ODM-11). |
| D-4 | **Physics is spike-local.** | The character body (`player.rs`) is an AABB with fixed constants (width 0.6, height 1.8, gravity 28, jump 8.5, speed 6), written for feel-testing, not a contract; authority lives in the adapter/Godot process, not in a server runtime; there is no tick, no single-writer, no World Operations interface. |
| D-5 | **Authority and rules are in a client scene.** | `world.gd` owns streaming, meshing, input, picking, edit actions and the save file. Production direction (AOD-07) puts authority in Rust and keeps GDScript to UI and glue. The Rust bridge holds the logic that matters (voxel reads, raycast, rules, physics), but the call order is decided by the script. |
| D-6 | **Single process, no multiplayer.** | No sessions, no authorisation, no replication, no TCP server use. |
| D-7 | **Debug library with optimised crates.** | `Cargo.toml` raises `opt-level` of three crates in the dev profile because the Godot editor binary loads the debug adapter library. Effect on frame times was not compared with the old profile. |
| D-8 | **Spike code stays in a spike worktree.** | Committed locally as `fea3deb` (18 files, parent `820f538`); local branch `spike/p-world` points at it (no upstream, no tag); worktree HEAD detached, clean tree; two bundles in `D:\Projects\GitHub\Games\Cube-archive\p-world\`; not pushed, not merged. Moving any of it into production needs a separate owner decision (`AGENT_HANDOFF.md` rule 5). |

## 2. Functional limitations

| Area | Limitation |
|---|---|
| Autosave | Only when the window is closed normally. Process kill, crash, Ctrl+C in the terminal or closing the terminal lose unsaved edits. No periodic autosave. The HUD shows `unsaved edits N`; the Save button saves manually. |
| Save contents | The character position is not saved (it appears above the middle of the saved centre chunk and falls). Only one save slot. |
| Save versus arguments | An existing save overrides `--seed` and `--radius`. `run-world.ps1` has no `-Fresh` switch and does not forward `--save`; use the rename or direct-Godot commands in the handoff document. |
| `--fresh` | Does not delete the old save; closing the window overwrites it. |
| Damaged saves | A damaged save is renamed to `world.wst0.bad`; a second damaged file silently replaces an existing `.bad` (verified in the Owner Review), and any load failure (also an unsupported version) is treated as damage. A failed save at close is only printed; the window closes anyway. |
| Editing | One material only (placed block, material 4). No block palette, no area tools, no drag-to-dig. Undo and Redo exist (buttons, spike-local journal) but are not bound to keys. Reach is 10 voxels from the character; the cursor ray looks 80 voxels. |
| Aiming | The aim is the mouse cursor, not a screen-centre crosshair. |
| Camera | Orbit camera can intersect the terrain; no collision for the camera (owner's note). |
| Character | Ground detection, step-up, jump and collision are tested, but the feel of the controls is not judged and no human has played it (owner's note). One-voxel step-up; a two-voxel wall blocks. |
| Streaming | Synchronous, on the main thread; radius 6 and 8 exceed the 16.7 ms frame budget (owner's note); frame time grows with the radius even without loading. Cause found in the Owner Review: the HUD's per-frame `active_voxels()` rebuilds the list of all solid voxels (0.9-13.6 ms per call at radius 1-8), see `3_P-WORLD_EVIDENCE.md` section 4. Default radius is 1; the verified start command uses 2. |
| World | 32 voxels high; no caves, water, biomes; unloaded chunks around the edge of the loaded area are walls for the character (it waits, it does not cross). |
| Rendering | Flat colours, no textures, no lighting beyond one directional light, no LOD. |
| Platforms | Windows desktop only was exercised for P-WORLD. Android emulator results exist from the original spike; nothing was re-run for P-WORLD; no physical device or iOS. |

## 3. Test limitations (what the results do not show)

1. **The interactive tests use synthetic input.** Keys and mouse events are injected with `Input.parse_input_event`; a human mouse path was not exercised. In a real window a synthetic mouse-motion event does not move the viewport's cursor, so the cursor aim of the windowed test is set by `aim_override`; the clicks themselves arrive as input events and go through the real handler. (In a headless run the synthetic motion did move the cursor and the picking worked.)
2. **Persistence was verified for one scenario** (7 edits, seed 12345, radius 1) in the window test and 8 edits (seed 424242, radius 2) in the Rust two-process test. Not verified: very large edit sets, edits far from the saved centre while the chunk is unloaded at save time (covered in-process: edits survive unloading, but not through a restart), saves while edits are pending in a worker (there are none).
3. **Seam checks compare the mesh with the height function** on 12 seeds; visual checks are 12 screenshots and two were inspected by eye.
4. **Performance numbers are preliminary** (single machine, single GPU, no comparison with the previous profile).
5. **A clean-clone run** of `run-world.ps1` was not performed; the script was verified in this worktree with a populated `target`.
6. **A side effect on the machine:** the old spike smoke test (`world_test.tscn`) wrote its own files into `%APPDATA%\Godot\app_userdata\Cube Spike Client\` (`world_smoke.wst0`, `world_m_faces.wst0`, `world_m_greedy.wst0`; the older `*.cube` and `world-*.png` files there date from earlier spike work). None is named `world.wst0`, so the first real start is a fresh world. One accidental whole-desktop screenshot was taken during stage A, deleted at once and never used or stored in any report.

## 4. Open items from the owner's reviews (unchanged, not worked on)

Manual play-through of the physics; judging the feel of controls and jump; camera and terrain intersection; radius 6 and 8 performance; clean-clone start; separate measurement of `opt-level=3`.

## 5. Questions for the Owner Review (proposals only)

1. Is the experiment's finding "a visible, walkable, editable, restartable world is reachable in this small amount of code" accepted as the knowledge that flows back (rule 2 of the integration document)?
2. What flows into production as requirements and tests: voxel read and raycast requirements, collision rules at unloaded chunks, the frame-time jitter lesson, the edit-then-restart scenario, the seam and determinism checks.
3. What happens to the worktree and the local commit `fea3deb`? It is kept by the local branch `spike/p-world` and by two bundles in the archive; nothing was pushed, tagged or merged. Open: whether to keep or remove the worktree, whether a second copy on another drive or a remote is wanted, and whether the evidence (logs, screenshots, about 1.3 GB scratch build included in the temporary folder) should be archived: only the bundles and the documents were archived.
4. Should a manual play-test with the owner's own hands precede any further work, and which of the open items (section 4) matter first?

None of this authorises production work; PS-0 remains open and PS-1 is not authorised.
