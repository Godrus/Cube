> **Справка, не обновлять.** Состояние задач ведётся в `.agent/BOARD.md`, история работ в `.agent/LOG.md` (схема DOC-01, 2026-11-11). Здесь остаются подробности прошлых итераций.

# Next execution (snapshot, updated at the end of each major iteration)

**Written:** 2026-11-10, after the first iteration of the product-roadmap directive. Replace the facts below when you finish yours; do not append a log.

## Current epic and tasks

* **PRIORITY CHANGE (owner, 2026-11-11):** visual direction is now Cube World Alpha and Veloren (stylised, cohesive, playable). The micro-voxel character task (CHAR-01, branch `feat/voxel-character-proto`) is cancelled as current work and parked, not merged and not deleted; do not start high-resolution character pipelines, renderer/WorldGen/Geometry Core rewrites or micro-voxel LOD systems. Details: `docs/vision/VISUAL-STYLE-GUIDE.md`. Development continues per `docs/roadmap/PRODUCT-ROADMAP.md`: game mechanics, adventures, RPG progression, world exploration, Android compatibility; no large graphics rework without a separate owner task.
* Epic **E-M1** (persistent world and gameplay integrity), see `docs/roadmap/WORK-PACKAGES.md`.
* **M1-A2** inventory/world reconciliation: **TESTED and INTEGRATED into `origin/dev`** (commits `e47a35b`, docs `6ff18b3`; CI run 42857847038 green). Player-visible: DEMO_AVAILABLE, but the Godot smoke was not re-run after the change.
* **CMB-01..03 TESTED** (branch `prod/combat-core`, feature F-GPL-004): `combat.rs` health, damage, respawn, one melee weapon, attack resolution; health saved in `game.state`. Not player visible. **CMB-04 TESTED** (feature F-CRT-001, `creature.rs`: Slime, Idle/Chase/Attack/Dead on the player's Walker body). **CMB-05 TESTED** (F-CRT-002, `spawn.rs`, `PortSession` ticks creatures; off by default, `CUBE_CREATURES=1` enables; INTERNAL_ONLY, not run in Godot). **CMB-01..10 done at prototype level** (F-GPL-004/005, F-CRT-001/002, F-CLI-005): health, damage, weapon, slime with AI, spawn, loot, saved kills and loot, port/adapter view, Godot attack (E), health bar, creatures drawn. Verified by Rust tests and the real-Godot combat smoke `python tools/visual-qa/walk_smoke.py --combat` (scripted bot, not a person, not in CI). DEMO_AVAILABLE with `--creatures`. **M3 slice 1 done (WGN2-01, F-WGN-003):** terrain-v2 with four biomes, v1 kept, HUD biome name, `walk_smoke.py --world` flies a terrain-following tour in real Godot (fresh checkout needs the Godot import, see below). **M3 slice 2 done (WGN2-02, F-WGN-004):** terrain-v3 caves and ores, `walk_smoke.py --caves`. **M3 slice 3 done (WGN2-03, F-WGN-005):** terrain-v4 trees and forests, `walk_smoke.py --trees`. Known small defect: creatures can spawn inside a tree (spawn rule ignores trees). **M3 slice 4 done (WGN2-04, F-WGN-006):** terrain-v5 ruins with a creature den; creature spawn is now tree-aware (Land trait). M3 first vertical set (biomes, caves and ores, trees, ruins) is in `dev` with a Godot smoke each (`--world`, `--caves`, `--trees`, `--ruins`). **Next, one task:** make the ruin worth visiting (a loot chest object or an altar reward: a ground item/loot container system replaces the lost-overflow rule), then a stairway into a small dungeon below a ruin as generator version 6; OR M4 (XP and levels) if the owner prefers progression first. Earlier combat note: **Next (older):** have a person play the loop and fix what feels wrong, then creature model and sound and ground items (instead of lost overflow); after that M4 (XP, levels), M1-A1 (edited bricks) or WGN2-01 (biomes). Verification (this iteration): the earlier hang of `--combat` was the native extension not loading in a fresh git worktree (no `godot/.godot` cache, so the scene ran a placeholder: frozen picture, no frames); the runner now imports once and names the last step on a timeout; the bot ends with an explicit result. Two consecutive combat smokes, play smoke, walk smoke and a plain launch pass. Persistence honesty: only `game.state` is written atomically (kill plus loot); the whole world/journal/inventory problem is NOT solved (CB-1 open, reconciliation covers block edits only). Fresh worktrees need `godot --headless --path godot --import` once (the smoke runner does it).

**Iteration after the visual change (2026-11-11, local `dev` only, not pushed):** M4 slice 1 (XP and levels: kill gives XP, level from total XP, +4 max health per level, level-up refills health, saved in `game.state` with the kill; HUD bar) and the ruin reward (R near a ruin altar gives one deterministic cache and 10 XP, taken caches saved as `cache x z` lines; F-GPL-006). Tests and clippy/archcheck/features pass; the combat smoke passed twice in Godot (its loot check now ignores the XP text); the XP bar and the R key were not looked at on screen. **Then WGN2-05 (F-WGN-007, terrain-v6):** a stairway and a brick chamber with a pedestal under every ruin; R at the pedestal gives the deep cache (3 rolls, 25 XP); v6 is the default for new worlds, v5 worlds unchanged; ruins and play smokes pass; nobody walked down in Godot. The chamber now has a guard slime (v6 den) and was looked at in a Godot picture. Weapon types and the first recipe are done (F-GPL-007: iron sword = 3 iron ore + 2 wood, G crafts, the selected hotbar item picks the weapon; nobody pressed G on screen). Android **ran in the emulator (x86_64, 2026-11-11)**: `scripts/build-android.sh`, `godot/export_presets.cfg` (x86_64 emulator and arm64 device), Android entries in `cube.gdextension`, the mobile renderer set to compatibility, `godot/README.md` section; the machine has Android Studio, SDK, NDK 28.1, Godot 4.7.2 export templates and an x86_64 emulator `Medium_Phone` (Android 16, 16 KB pages). The owner approved the one download (`rustup target add aarch64-linux-android x86_64-linux-android`, 280 MB in the pinned 1.97.1 toolchain). Built `libcube_godot_adapter.so` (x86_64, 6 MB), exported a 37 MB debug APK, installed and started it: the Rust core loaded and the world was created in `user://world`; with the **Vulkan mobile renderer** terrain, HUD and hotbar draw (picture in `docs/screenshots/android/`). The GLES3 compatibility renderer failed in the emulator (shader link, uniforms over 261), so the project uses `mobile` on Android. **Not done:** a physical phone, the arm64 build run, multi-touch and a real phone with the new touch controls (`godot/touch_controls.gd`: floating joystick, look pad, tap to mine, action buttons, landscape; self-test `--touch-test` passes; in the emulator a tap mined a block and a joystick swipe moved the player), frame time and memory, the odd camera in the picture (feet y -36 at start was not investigated). **Then F-GPL-008 (classless stats, no class decision needed):** +1 attack damage per 4 levels, and Iron armour (second recipe: 5 iron ore + 2 coal ore; worn while carried; flat damage reduction, never below 1); Rust tests only, not seen on screen. Still open and blocked outside the code: classes, a stat screen and abilities (owner, ODM-05); the physical phone, arm64 run, multi-touch, frame time and memory (owner's device); M1 brick eviction, M5 sculpt tools, M6 to M12 (large, each its own task; M8 and M11 also lack a networking owner, DEC-072). **Next, one task:** a physical-phone check and the arm64 build (needs the owner's device) OR stats and a first ability (needs the owner's class decision, ODM-05).

## Priority and dependencies

1. M1-A1 eviction of edited bricks (blocks the claim "long play never loses or overfills"), needs a spike because of the revision trap described in `WORK-PACKAGES.md`.
2. M2 combat (CMB-01..10): no persistence blocker; CMB-07 needs M1-A2 (done).
3. M3 world (WGN2): independent of both.

## First action for the next agent (copy)

```bash
git fetch origin
git worktree add -b prod/combat-core ../Cube-combat origin/dev
cd ../Cube-combat
cargo test -p cube-app --test reconcile      # proves the base is green
```

(Done, kept for reference) CMB-01 acceptance: a `Health` value (`cur`, `max`) and a `Damage` event in `crates/cube-app/src/combat.rs`, no engine types; damage clamps at zero, death is reported once, respawn restores health and position by a stated rule, integer overflow cannot happen (checked or saturating, tested at the limits), unit tests in the module. Not integrated into Godot yet; mark registry entries NOT_STARTED/IN_PROGRESS honestly until CMB-08..10.

## Tests and how to run the game

```bash
cargo test --workspace                                       # all Rust tests
cargo test -p cube-app --test reconcile                      # the crash-between-stores tests
cargo xtask archcheck && cargo xtask features --check        # architecture and registry
python tools/visual-qa/walk_smoke.py --play                  # real Godot (see godot/README.md for the path; not in CI)
```

## Risks

* Revision expectations in the journal make unloading edited bricks unsafe unless one of the three options in `WORK-PACKAGES.md` (M1-A1) is chosen and tested; "just evict" loses edits silently.
* `cube-app` hosts prototype gameplay; a gameplay crate needs an ADG change (owner).
* `gh` was not installed on the machine that wrote this; CI results come from `curl https://api.github.com/repos/Godrus/Cube/actions/runs?branch=<branch>` (the repository is public) or the web UI.
* The Godot smoke after the reconcile change was **not re-run** in this iteration (Rust level only); run `walk_smoke.py --play` before claiming the demo still works end to end.

## Must not change without a decision

ADG edges and crate graph; `replay_confirmed` semantics and the journal and snapshot formats (experimental until GOV-10); the three commit handles; the pinned generator v1 hashes; contract statuses (Provisional stays Provisional); owner files in the main checkout.

## Completion conditions for this handoff item

The next agent is done with CMB-01 when its tests pass, `cargo fmt`, clippy, workspace tests, archcheck and features checks are green locally and in CI, the change is in `origin/dev`, and this file names the next task.

## Last integration result

`origin/dev` = the commit that contains `6ff18b3` (fast-forward from 679e8d5 by push of `prod/m1-reconcile:dev` after CI run 42857847038 finished `success`: fmt, clippy, tests on Linux and Windows, loom, deny, archcheck, features). Local checks before: `cargo test -p cube-app` all suites (new: 4 crash tests in `reconcile.rs`, 5 unit tests in `game.rs`; a mutation that disables the settle step made the crash tests fail as intended), fmt, clippy `-D warnings`, archcheck, features. Uncommitted work: none.
