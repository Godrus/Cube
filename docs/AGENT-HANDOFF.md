# Agent handoff (current state, short)

Read this, then `FUNCTIONAL-ROADMAP.md`. The long procedural guide is `AGENT_HANDOFF.md`; the full plan is `production/`. State comes from the repository, tests and CI, never from chat memory.

## State (`dev` 01519c6, CI green)

* **Integration branch:** `dev`. `master` is release only and must not be changed. Work on short `prod/*` branches from the latest `dev`; merge only commits whose required CI checks are green (fast-forward push is allowed by the ruleset). No force push.
* **Gates:** G0 to G4 passed (G2 with mandatory RG-2 revalidation). G5 and G10 open. RG-1 not passed. Evidence: `production/ps-0-exit-checklist.md` sections 6 to 14, registry entries DEC-076..DEC-090.
* **Works (Rust level, tested, internal only):** Core (coordinates, bricks, revisions, atomic commit, views, bounds, palette), Processing (snapshots, executor, stale rejection, budgets), Editing API, Orchestration (one user commit path), voxel backend with faces/greedy/collision/LOD providers, experimental persistence codec with atomic snapshots, file-backed journal with payload (acknowledged = fsynced; a damaged acknowledged record fails closed through acknowledgement marks), replay and restart idempotency, crash-point harness, manifest, verified snapshot store, crash-safe compaction and `open_world` recovery (all experimental).
* **Also exists (`dev` 01519c6, tested, provisional):** persisted retry floors (L-COMP-1, requirement RS-1 in `api-contracts/editing-contracts.md`), SDF backend (`cube-backend-sdf`), mesh backend (`cube-backend-mesh`, Core page store), deterministic generator terrain-v1 (`cube-worldgen`, technical assignment DEC-091), engine-neutral client foundation (`cube-client`), headless first-world test (`first_world.rs`). **Also (`dev` c248bce):** greedy mesh without T-junctions (`triangulate`), world record (PERS-08), client intents/camera/picking/buffers, `cube-server` (in-process local host), `cube-app` (composition root, `open_first_world`), `cube-godot-adapter` (GDExtension, thin) and the Godot project `godot/` (`godot/README.md`: run, headless smoke, screenshot). The Godot scene was run with Godot 4.2.2 (found at D:/Downloads/Projects/Engine/...); it is a developer demo, not in CI. DEC-092 records the technical assignments. **Visual QA (experimental):** `tools/visual-qa/` asks TypeSafe Jev 1.14 about Godot screenshots; dry run by default, live only with `--live` and `TYPESAFE_API_KEY`; not in CI, not required; first live run on 5 images: gross failures caught, 2 of 2 good screenshots wrongly flagged (see `tools/visual-qa/results/2026-11-10-first-live/SUMMARY.md`); v2 (`tools/visual-qa/V2.md`): questionnaires, 37 real Godot fault pictures with pre-registered labels, deterministic checks; measured once on calibration and evaluation splits (`tools/visual-qa/V2-RESULTS.md`): useful only as an advisory check for missing geometry and wrong materials; the evaluation set is used up, so any rework needs a new calibration set; do not use it as a gate (F-QA-001).
* **Does not exist:** world service and materialiser, engine adapter, renderer, any gameplay, a world record (PERS-08), a final save format. Known limit L-COMP-1: retry history before the last compaction is not remembered. No feature is player available.

## Architecture in five lines

1. Crates: `cube-core` (no dependencies) <- `cube-processing` <- `cube-editing-api` <- `cube-orchestration`; backends, persistence depend on the same base; `cube-integration-tests` is the only place that may build a live asset with all handles.
2. All authoritative change goes through Core's commit gate via three handles: user (only Orchestration, only in `commit_path.rs`), system (WorldGen), recovery (Persistence). Enforced by `cargo xtask archcheck` and compile-fail tests.
3. Backends return write sets; they never commit. Derived data (meshes, collision, LOD) is computed by Processing on immutable snapshots and rejected if stale.
4. Persistence: untouched world = seed + generator version; changes = write-ahead journal (Intent, Confirmed, Aborted) with payload; snapshots speed loading. Formats are experimental until GOV-10.
5. Unknown or unloaded data is never reported as empty; overflow is an error; there is no world-size limit.

## Hard constraints

* Never present a provisional contract as Frozen. Provisional: PC-1 (C-07..C-09), C-10 palette part, PR-01..PR-08 (RG-2), editing contracts (not Frozen).
* Do not claim crash durability on devices (DG-2) or crash-atomicity of commit plus journal (CB-1 is open).
* CI-host timings are not device measurements. Greedy is not the default renderer (GOV-13).
* Do not move spike code (P-WORLD) into production without an owner decision. Do not touch the owner's uncommitted files (`.gitignore`, `docs/decisions/owner-decision-extensibility-v1.md` in the main checkout).

## Open work you may continue independently

PERS-06..09; WGEN-02..04 (gated by GOV-21, GOV-11), PERS-06..09, CLI-03/04, ADP-01 (no owner); TD-2 (validation port needs a layer, additive API change with snapshot update); TD-1 (observer delivery queue, only with loom models); greedy T-junctions. Check ADG start and merge conditions before each task.

## Decisions the owner still has to make

RG-1 handling beyond CORE-12 (GOV-10 format, CORE-13, G5), GOV-11, owners for WorldGen, Client Runtime, Engine Adapter, Server Runtime, Networking, a gameplay owner and model, GOV-13 renderer default, GOV-25 (history).

## Rules for features (mandatory)

1. Before significant work read this file and `FUNCTIONAL-ROADMAP.md`.
2. The registry source is `docs/features.json`; `FEATURE-REGISTRY.md` is generated (`cargo xtask features --write`) and checked in CI (`cargo xtask features --check`).
3. After adding, changing or finishing functionality update the matching registry entries in the same change as the code: implementation status, evidence (commit, CI, tests), player availability, verified revision, and this file's state if it changed. No IMPLEMENTED or beyond without evidence; no PLAYER_AVAILABLE for code only reachable through tests.
4. Never reuse a feature id. Never mark a feature APPROVED unless the owner approved it (cite the source).
5. When the owner approves a new feature: check the registry, create the id, record the approved behaviour and its source, dependencies, acceptance criteria and the technical tasks, check them against the ADG, update the roadmap, and do not call it implemented before it is.
6. End every report with a line: `Functional Roadmap: updated / unchanged (reason). Features added: ... Features completed: ... Player-visible functionality: ...`.

## Links

`production/implementation-roadmap.md`, `production/module-boundaries.md` (ADG), `production/ps-0-exit-checklist.md`, `decisions/decision-registry.md`, `api-contracts/README.md`, `production/FIRST_PLAYABLE_SCOPE.md`, `production/processing-foundation-contract.md`, `production/editing-api-contract.md`.
