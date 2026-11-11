> **Вход для агентов: `AGENTS.md`.** Текущие задачи: `.agent/BOARD.md`; вместо `NEXT-EXECUTION.md` и handoff-обновлений в разделах 6 и 7 теперь BOARD, LOG и `cargo xtask say/chat`. Остальные правила ниже действуют.

# Cube: start here (new agent chat)

You do not have the previous chat. Everything you need is in the repository. Dynamic facts (HEAD, gates, CI) must be read from the files and commands named below, not from this page. Last written against `dev` 679e8d5 plus branch `prod/m1-reconcile` (see `NEXT-EXECUTION.md`).

## 1. Identity

**Cube** is a cross-platform voxel open-world action RPG and sandbox with multiplayer. Rust core and game logic, Godot 4 client. Influences (not copying): early Cube World Alpha, Veloren, Trove. It is not a building sandbox and not an architecture exercise: the end goal is a complete game. Full vision and pillars: `docs/vision/GAME-VISION.md`. Pillars in short: exploration first; persistent mutable world; deep action RPG; creative freedom (Universal Geometry: voxel, SDF, mesh, composite, micro-voxels); systemic world; multiplayer by design (target up to 500 players per logical world, distributed, much later); cross-platform (PC and Android in the MVP); extensible geometry and content.

## 2. Architecture (read, do not re-derive)

* Crates: `cube-core` <- `cube-processing` <- `cube-editing-api` <- `cube-orchestration`; backends (`cube-backend-voxel/sdf/mesh`), `cube-persistence`, `cube-worldgen`, `cube-client` (engine-neutral), `cube-server` (in-process local host), `cube-app` (composition root, prototype gameplay), `cube-godot-adapter` (thin GDExtension), `godot/` (scene, GDScript glue).
* All authoritative world change goes through Core's commit gate by three handles (user: only Orchestration; system: WorldGen/materialise; recovery: Persistence). Enforced by `cargo xtask archcheck` (graph must equal ADG Part B) and compile-fail tests. Never bypass the Editing API; never write Core from Godot.
* Persistence: untouched world = seed + generator version; changes = write-ahead journal with payload; snapshots speed loading; formats are experimental until GOV-10. Unknown data is never reported as empty.
* Canonical: ADG `docs/production/module-boundaries.md`; plan `docs/production/production-architecture-plan.md`; decisions `docs/decisions/decision-registry.md` (only the registry decides); contracts `docs/api-contracts/`.

## 3. Roadmaps (three levels, never mix them)

1. **Product:** `docs/roadmap/PRODUCT-ROADMAP.md` (M0..M12, crosswalk, critical path), tasks in `docs/roadmap/WORK-PACKAGES.md`.
2. **Features and status:** `docs/features.json` -> `docs/FEATURE-REGISTRY.md` (generated) and `docs/FUNCTIONAL-ROADMAP.md`. Commands: `cargo xtask features --write`, `cargo xtask features --check`. Do not create a second registry.
3. **Engineering:** `docs/production/` PS-0..PS-14 and gates G0..G12. Do not close a gate automatically.

Older state and rules: `docs/AGENT-HANDOFF.md` (short current state and feature rules), `docs/AGENT_HANDOFF.md` (long procedure, older: its "never commit without permission" rule is superseded by the owner's standing delegation, DEC-001, and by the owner's directive of 2026-11-10 to integrate completed tasks through the normal workflow).

## 4. Project state (read it, do not trust this page)

```bash
git fetch origin
git log --oneline -5 origin/dev
git worktree list
git status --short
cargo xtask features --check
cargo xtask archcheck
```

Facts to know: `origin/dev` is the integration branch. A *local* `dev` branch may be stale: use `origin/dev`. Work in a fresh worktree from `origin/dev` (`git worktree add -b prod/<name> ../Cube-<name> origin/dev`); do not touch other worktrees (`Cube-pworld` is the spike, never imported automatically; the owner's main checkout holds uncommitted owner files: `.gitignore` and `docs/decisions/owner-decision-extensibility-v1.md`). `master` is release only.

Gates: G0..G4 passed (G2 with mandatory RG-2 revalidation); G5, G10, G11, G12 open; RG-1 not passed. Details: `docs/production/ps-0-exit-checklist.md` sections 6..14.

## 5. What exists for a player (honest)

Only the **Godot developer demo** (not in CI, run by hand): walking player, first and third person, a finite generated terrain streamed around the player (far unedited bricks unloaded), place and mine blocks with a hotbar, 27-slot inventory (prototype rules), creative/survival, one objective, save and continue. No combat, creatures, items beyond blocks, classes, skills, crafting, biomes, caves, multiplayer, Android. Run: `godot/README.md`; smoke: `python tools/visual-qa/walk_smoke.py [--journey|--play]` (Godot 4.2.2 path is in that README).

Known limits: edited bricks never unload (M1-A1, design in `WORK-PACKAGES.md`); generation runs on the main thread (not yet measured as a problem, M1-A3); CB-1 (Core commit and journal not one atomic action) is open at the engine level; no device durability claims (DG-2).

## 6. Rules of work

**Autonomy.** You may decide reversible engineering details alone (names, internal structure, tests, small modules, new generator versions, pure gameplay modules in `cube-app`). You must ask the owner for: contract status changes (Provisional -> Frozen), anything in `docs/decisions/` as a decision, ADG edges and new crates (archcheck enforces them), save-format approval, owner assignments, game balance and class design as final rules, spending, force-push or deleting others' work. Present prototype values as prototype values. Do not invent approvals.

**Do not:** write another planning cycle instead of code; open new governance questions for settled matters; report a feature done because Rust tests pass (it needs real integration, and Godot verification for client features); make a second inventory, WorldGen or editing pipeline; copy reference games' code or assets; build 500-player infrastructure before two clients work; start Visual QA v4 or new image-model research (TypeSafe tooling exists: use it only by concrete need, never automatically paid).

**Definition of done for a game feature:** implemented; real dependencies done; contracts kept; tests pass; integrated into the app; verified in real Godot if client-visible; errors explicit; persistence works as far as required; registry and roadmap updated; CI green; code in `origin/dev`; availability classified honestly. Implementation words: NOT_STARTED, IN_PROGRESS, IMPLEMENTED, TESTED, INTEGRATED. Availability: NOT_AVAILABLE, INTERNAL_ONLY, DEMO_AVAILABLE, PLAYER_AVAILABLE. CI green is not game readiness.

**Git and CI.** Small commits on `prod/*` from the latest `origin/dev`. Required checks: fmt, clippy (`-D warnings`), tests on Linux and Windows, deny, archcheck, features check. Integration into `dev` is a fast-forward push of a commit whose required checks are green (the ruleset allows it); no force push, no destructive reset, no bypassing protection, never change `master`. Check CI with `gh run list --branch <branch>` / `gh run view`. Local verify: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo xtask archcheck`, `cargo xtask features --check`. Commits end with the Co-Authored-By line the harness gives you.

**Handoff upkeep.** At the end of a major iteration update `NEXT-EXECUTION.md` (current epic, tasks, result of the last integration, what must not change), the registry entries you touched, and `docs/FUNCTIONAL-ROADMAP.md` / `docs/AGENT-HANDOFF.md` if the player-visible state changed. If you find code and these documents disagree, the code wins: fix the documents and say so. If the repository has `AGENTS.md` pointing to `.agent/state/CURRENT_CONTEXT.md`, keep that short pointer current as well. End every report with `Functional Roadmap: updated / unchanged (reason). Features added: ... completed: ... Player-visible functionality: ...`.

## 7. Next action

See `docs/handoff/NEXT-EXECUTION.md`. Then continue from `docs/roadmap/WORK-PACKAGES.md`; do not stop at planning.
