# Next execution (snapshot, updated at the end of each major iteration)

**Written:** 2026-11-10, after the first iteration of the product-roadmap directive. Replace the facts below when you finish yours; do not append a log.

## Current epic and tasks

* Epic **E-M1** (persistent world and gameplay integrity), see `docs/roadmap/WORK-PACKAGES.md`.
* **M1-A2** inventory/world reconciliation: **TESTED and INTEGRATED into `origin/dev`** (commits `e47a35b`, docs `6ff18b3`; CI run 42857847038 green). Player-visible: DEMO_AVAILABLE, but the Godot smoke was not re-run after the change.
* **CMB-01..03 TESTED** (branch `prod/combat-core`, feature F-GPL-004): `combat.rs` health, damage, respawn, one melee weapon, attack resolution; health saved in `game.state`. Not player visible. **CMB-04 TESTED** (feature F-CRT-001, `creature.rs`: Slime, Idle/Chase/Attack/Dead on the player's Walker body). **CMB-05 TESTED** (F-CRT-002, `spawn.rs`, `PortSession` ticks creatures; off by default, `CUBE_CREATURES=1` enables; INTERNAL_ONLY, not run in Godot). Next, one task: **CMB-06** (loot: a drop table rolled with a seeded source on a kill via `Population::player_attack`, item into the inventory by the existing operation, full-inventory rule), then CMB-07 (persist kills and loot once: reconcile pattern), CMB-08 (view fields in `GameView`), CMB-09 (Godot attack input, creature mesh, health bar). In parallel an independent agent may take **M1-A1** (spike first) or **WGN2-01**.

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
