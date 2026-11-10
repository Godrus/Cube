# Next execution (snapshot, updated at the end of each major iteration)

**Written:** 2026-11-10, after the first iteration of the product-roadmap directive. Replace the facts below when you finish yours; do not append a log.

## Current epic and tasks

* Epic **E-M1** (persistent world and gameplay integrity), see `docs/roadmap/WORK-PACKAGES.md`.
* **M1-A2** inventory/world reconciliation: **TESTED**, commit `e47a35b` on branch `prod/m1-reconcile` (from `origin/dev` 679e8d5). Integration into `dev` depends on CI; check `git log origin/dev` for whether it already landed before assuming either way.
* Next, one task: **CMB-01** (health, damage event, death and respawn as a pure Rust module `crates/cube-app/src/combat.rs`), then CMB-02, CMB-03. In parallel an independent agent may take **M1-A1** (spike first) or **WGN2-01**.

## Priority and dependencies

1. M1-A1 eviction of edited bricks (blocks the claim "long play never loses or overfills"), needs a spike because of the revision trap described in `WORK-PACKAGES.md`.
2. M2 combat (CMB-01..10): no persistence blocker; CMB-07 needs M1-A2 (done).
3. M3 world (WGN2): independent of both.

## First action for the next agent (copy)

```bash
git fetch origin
git worktree add -b prod/combat-core ../Cube-combat origin/dev     # or origin/prod/m1-reconcile if dev has not got it
cd ../Cube-combat
cargo test -p cube-app --test reconcile      # proves the base is green
```

CMB-01 acceptance: a `Health` value (`cur`, `max`) and a `Damage` event in `crates/cube-app/src/combat.rs`, no engine types; damage clamps at zero, death is reported once, respawn restores health and position by a stated rule, integer overflow cannot happen (checked or saturating, tested at the limits), unit tests in the module. Not integrated into Godot yet; mark registry entries NOT_STARTED/IN_PROGRESS honestly until CMB-08..10.

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
* `gh` was not installed on the machine that wrote this; CI results come from the GitHub web UI or API.
* The Godot smoke after the reconcile change was **not re-run** in this iteration (Rust level only); run `walk_smoke.py --play` before claiming the demo still works end to end.

## Must not change without a decision

ADG edges and crate graph; `replay_confirmed` semantics and the journal and snapshot formats (experimental until GOV-10); the three commit handles; the pinned generator v1 hashes; contract statuses (Provisional stays Provisional); owner files in the main checkout.

## Completion conditions for this handoff item

The next agent is done with CMB-01 when its tests pass, `cargo fmt`, clippy, workspace tests, archcheck and features checks are green locally and in CI, the change is in `origin/dev`, and this file names the next task.

## Last integration result

Branch `prod/m1-reconcile` holds `e47a35b` (code, tests, registry) and the documents of this directive. Local verification on Windows: `cargo test -p cube-app` (all suites incl. 4 new reconcile tests and 5 new game unit tests) passed, mutation check made the crash tests fail as intended, fmt, clippy `-D warnings`, `archcheck` and `features --check` clean. CI result: see the commit status on GitHub (not read by the author). Uncommitted work: none in the `Cube-m1` worktree after the commit of these documents.
