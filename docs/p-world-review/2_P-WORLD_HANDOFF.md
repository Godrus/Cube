# P-WORLD handoff: worktree, files, how to run

## 1. Worktree and its state

| Item | Value |
|---|---|
| Path | `D:\Projects\GitHub\Games\Cube-pworld` |
| Kind | separate git worktree of the repository `D:\Projects\GitHub\Games\Cube` (listed by `git worktree list`) |
| HEAD | `fea3deb46b49109abae8149d56fddb74610eaefb` ("spike: P-WORLD experiment ..."), **detached** in this worktree |
| Branch | **`spike/p-world`** in `D:\Projects\GitHub\Games\Cube` points at `fea3deb` (local only, no upstream, no tag; created on the owner's instruction; the worktree stays detached and the main repository still has `docs/architecture-roadmap-v1` checked out) |
| Parent | `820f538c6da9b5bb9a2c2cb954d410c408b2a110` ("chore: Godot uid file for world_smoke.gd", the tip of `spike/rust-world-generation`); exists, is an ancestor of `fea3deb` |
| Commit | **18 files, 2119 insertions, 12 deletions** (4 modified, 14 new; lists below), made locally on the owner's instruction; **not pushed, not merged**; no remote ref contains it |
| Working tree | **clean** (`git status --short` empty; only git-ignored `target\` and `godot-client\.godot\`) |
| Build output | `D:\Projects\GitHub\Games\Cube-pworld\target` (git-ignored) |
| Main repository | `D:\Projects\GitHub\Games\Cube`, branch `docs/architecture-roadmap-v1`, HEAD `16db379`; only the owner's ` M .gitignore`; P-WORLD wrote nothing there |
| Permanent archive | `D:\Projects\GitHub\Games\Cube-archive\p-world\` (outside the repository, outside the working trees): `p-world-fea3deb.bundle` (35,226 bytes, incremental `820f538..fea3deb`, needs `820f538` in the target repository, sha256 `592723d3021d10f1e0eaa011dfe27aa38184fa85eb19c2334efb355eb923bf87`, identical to the verified original) and `spike-p-world-full.bundle` (2,429,356 bytes, complete history of the branch `spike/p-world`, self-contained); copies of these four documents in `...\p-world\handoff\` and of `OWNER_REVIEW.md` in `...\p-world\` (the review describes the state at the time of the review, before the branch existed; it was not edited) |
| Not done | no push, merge, tag, PR, cherry-pick; no source change; no new commit; the worktree was not deleted |

**Durability:** `fea3deb` is now held by three independent things: the branch `spike/p-world`, the worktree's HEAD, and two bundles on disk. The local branch keeps the commit even if the worktree is removed. It is still on one disk and not on any remote; the archive folder is on the same drive as the repository (`D:`).

**Restore (verified on 2026-11-08, restore tests under `...\scratchpad\p-world-review\restore-test\`):**

```powershell
# self-contained, into a new directory (the bundle has no HEAD, so name the branch):
git clone -b spike/p-world D:\Projects\GitHub\Games\Cube-archive\p-world\spike-p-world-full.bundle <new-dir>
# or into an existing repository that has 820f538 (for example a clone of spike/rust-world-generation):
git fetch D:\Projects\GitHub\Games\Cube-archive\p-world\p-world-fea3deb.bundle HEAD:refs/heads/restored
```

Results: the full-bundle clone has HEAD `fea3deb46b49109abae8149d56fddb74610eaefb`, tree `6182830cddd7e3d00a80263335871a3f736c6a87` identical to the original, `git fsck --full` clean, 18 files changed in the commit, 29 commits of history, clean checkout. The incremental bundle fetched into a clone that has `820f538` gives `restored == fea3deb`, same tree, clean checkout. Into an empty repository it is refused ("lacks prerequisite commit 820f538"), as expected.

To remove the worktree later (owner's call, not done): `git worktree remove D:\Projects\GitHub\Games\Cube-pworld`; the commit stays on `spike/p-world`.

## 2. Changed and new files (complete list: exactly the 18 files of commit `fea3deb`)

Modified (4):

| File | Change |
|---|---|
| `Cargo.toml` | `opt-level = 3` for `cube-core`, `cube-worldgen`, `cube-godot-adapter` in the dev profile (the Godot editor loads the debug library) |
| `crates/cube-godot-adapter/src/lib.rs` | Godot methods: `world_hash`, `voxel_at`, `column_top`, `raycast`, `remove_block`, `place_block`, `player_spawn`, `player_teleport`, `player_step`, `player_pos`, `player_on_ground`, `player_held`; `pub mod player` |
| `crates/cube-godot-adapter/src/world_bridge.rs` | `world_hash`, voxel reads, `raycast` (DDA), `remove_block`, `place_block`, character body hooks, load repositions the character |
| `godot-client/world.gd` | args `--seed --radius --save --fresh`; orbit camera; frame times; walking character; mouse picking and edit actions; highlight; autosave on close, load on start; HUD |

New (14):

| File | Lines | Purpose |
|---|---|---|
| `crates/cube-godot-adapter/src/player.rs` | 268 | character body physics (pure Rust) + 5 unit tests |
| `crates/cube-godot-adapter/tests/edit_world.rs` | 304 | raycast, remove/place rules, collision and border edits, two-process persistence |
| `crates/cube-godot-adapter/tests/player_world.rs` | 169 | walking across chunk borders, unloaded chunks, edits and collision |
| `crates/cube-godot-adapter/tests/seams.rs` | 73 | seam check on 12 seeds |
| `crates/cube-godot-adapter/tests/world_hash.rs` | 36 | world hash determinism |
| `godot-client/world_edit.gd` / `.tscn` | 271 / 6 | stage C scripted entry (picking, editing, three phases) |
| `godot-client/world_interactive.gd` / `.tscn` | 209 / 6 | stage B scripted entry (synthetic input in a real window) |
| `godot-client/world_seams.gd` / `.tscn` / `.gd.uid` | 99 / 6 / 1 | stage A seed, hash and seam entry, frame-time measurement |
| `scripts/run-world.ps1` | 35 | build and start the world window |
| `scripts/test-world-persistence.ps1` | 81 | three-process window test: edit, close window, restart, verify |

Not part of the worktree (kept in the scratchpad): reports, logs, screenshots, and the four earlier recovery documents (`...\scratchpad\prototype-recovery\*.md`).

## 3. Start the world (one verified command, radius 2)

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File D:\Projects\GitHub\Games\Cube-pworld\scripts\run-world.ps1 -Radius 2
```

Verified on 2026-11-08: the script builds the adapter, finds the winget Godot 4.7.2, opens the window "Cube Spike Client (DEBUG)", the console prints `WORLD seed=12345 radius=2 renderer=faces`. Options: `-Seed <n>`, `-Radius 0..8`, `-Renderer faces|greedy|cubes`, `-Godot <path>`, `-NoBuild`. (Not verified on a clean clone.)

**Controls:** WASD or arrows walk (relative to the camera), Space jumps, right-drag orbits the camera, wheel zooms, left click removes the voxel under the cursor, right click (no drag), middle click or `E` places a block on the face under the cursor, buttons: Save, Load, Undo, Redo, "Jump 64" (teleport; the character waits for the chunk, then falls).

## 4. Where the save is, and how to start a fresh world

- **Default save file:** `%APPDATA%\Godot\app_userdata\Cube Spike Client\world.wst0` (`C:\Users\Godrus\AppData\Roaming\Godot\app_userdata\Cube Spike Client\world.wst0`). It does **not exist yet** (checked: P-WORLD never wrote to this path); it is created the first time the window is closed.
- **Another file:** pass `--save=<absolute path>` to Godot (the script does not forward it).
- **An existing save wins over `--seed` and `--radius`:** verified: with a save of seed 12345 and radius 1, a start with `--seed=777 --radius=2` loaded seed 12345, radius 1. The saved seed and radius define the world.
- **Start a new world with a clean seed:**
  - keep the old world: rename the file, then start with `-Seed N -Radius 2`:
    ```powershell
    Rename-Item "$env:APPDATA\Godot\app_userdata\Cube Spike Client\world.wst0" world.wst0.old
    ```
  - or start Godot directly with `--fresh` (ignores the existing save; **closing the window then overwrites the old save**). Verified: `--fresh --seed=777 --radius=2` starts `seed=777 radius=2` and does not load the file:
    ```powershell
    & "<path to Godot_v4.7.2-stable_win64.exe>" --path D:\Projects\GitHub\Games\Cube-pworld\godot-client world.tscn -- --fresh --seed=777 --radius=2
    ```
    (`run-world.ps1` has no `-Fresh` switch; adding one would be a code change and was not made.)
- A damaged save is renamed to `world.wst0.bad` and a fresh world starts.

## 5. How to end the application so that the autosave runs

- **Close the window with its close button (X) or Alt+F4.** That is the real close request; the client then writes the save. This path was tested by closing a real window from a script (`CloseMainWindow`, the same close request) in three separate processes.
- Check before closing: the HUD line shows `unsaved edits N`. The **Save** button saves at any time and resets it to 0.
- **Do not** end the process with Ctrl+C in the terminal, Task Manager "End task", `Stop-Process`/`Kill`, or by closing the terminal: no close request reaches the game and **unsaved edits are lost** (there is no periodic autosave).
- The character position is not saved; after a load it appears above the middle of the saved centre chunk and falls onto the surface.

## 6. Re-running the checks

```powershell
cd D:\Projects\GitHub\Games\Cube-pworld
cargo test --workspace                                                                   # 152 tests
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\test-world-persistence.ps1   # opens windows; edit, close, restart
```

Godot scripted entries (headless, from the `godot-client` folder): `world_test.tscn -- --world-smoke`, `world_seams.tscn`, `world_interactive.tscn -- --radius=2`; windowed with screenshots: add `--shot=<dir>`.
