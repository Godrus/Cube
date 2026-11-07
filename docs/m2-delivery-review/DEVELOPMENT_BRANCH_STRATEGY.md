# Development branch strategy (proposed)

**Status:** **Proposed** / **Pending Owner Approval**. No branch is created, renamed, merged or protected by this document; commands are given for the owner or for an authorised agent and have **not** been run. Facts verified 2026-11-07 (see `REPOSITORY_AND_MERGE_REVIEW.md`).

## 1. Starting facts

* `master` (`bf60534`, 13 documentation files) is not protected, has no required status checks and no workflow (GitHub API, unauthenticated read; repository public, no licence file).
* `docs/architecture-roadmap-v1` is a fast-forward descendant of `master` (38 commits ahead, 0 behind); it carries documentation **and** spike code. Spike branches exist separately (`spike/rust-godot`, `spike/rust-world-generation`).
* Existing rules: spike code never merges to production (roadmap-v1 section 7, `integration-and-qa-plan.md` section 1); production branches are `prod/ps<stage>-<crate>[-topic]`, a PR changes one crate; merge order follows the ADG; the docs branch itself needs the owner's explicit approval to enter `master`.

## 2. Requirements to satisfy

Stable `master`; isolation of unfinished work; Technology and Game Tracks that progress independently; integration only through verifiable PRs; controlled changes of API contracts; the ability to assemble M2 from both tracks; no permanent branches without a need.

## 3. Options

| Option | Description | For | Against |
|---|---|---|---|
| **A. Trunk with short-lived branches (recommended)** | `master` is the only permanent branch; every change is a short-lived branch plus a PR with required checks; unfinished features live in crates/modules that are not wired into the composition root | one place of truth; no divergence between tracks; integration is continuous (the brief's goal); least process for a project with one owner and agent workers | needs required checks to protect `master` (none exist yet); half-done work must be hidden by structure, not by branch |
| B. Two permanent branches `development/technology` and `development/game` | each track merges into its branch, both into `master` at milestones | tracks isolated | the tracks share ports and contracts (world port, Game Rules ports), so divergence is certain; the merge point becomes a large risky event; two protection setups and two CI configurations; contradicts "integration continuously" |
| C. One permanent `develop` between feature branches and `master` | `master` receives only milestone merges | `master` stays release-like | an extra permanent branch with no extra safety once `master` itself is protected; delays feedback |
| D. Short-lived integration branch for a milestone (`integration/m2`) | created when both tracks are nearly ready, deleted after M2 | a place to stabilise the assembly | only needed if `master` protection forbids unfinished milestone work; may never be needed |

**Recommendation: A, with D as an optional, temporary stabilisation branch (`release/m2`) created only at the end of M2.** Separation of the tracks is by **ownership and paths**, not by branches:

* `crates/` for Technology modules and the Game Rules crate are separate crates; a PR touches one crate (existing rule).
* Labels `track:technology`, `track:game`, `track:integration` and a path-based review requirement (CODEOWNERS, section 6) route reviews.
* A Game Track change cannot silently change a technology contract: contract files and API snapshots are owned by the Integration Owner and need the owner's review (section 6).

## 4. Branch kinds and names

| Kind | Name | Lifetime | Rule |
|---|---|---|---|
| Stable | `master` | permanent | protected; only PRs with required checks; never rewritten |
| Documentation | `docs/<topic>` | short | docs-only PR; the documentation check must pass |
| Technology feature | `prod/ps<stage>-<crate>[-topic]` (existing convention) | short (one task/PR chain) | one crate per PR; ADG merge-after conditions satisfied before merge |
| Game feature | `game/<area>-<topic>` (new, same spirit) | short | one module per PR (Game Rules, UI, content) |
| Experiment | `spike/<topic>` | until the finding is written | never merged; outputs are knowledge, not code (`TECHNOLOGY-GAME-TRACK-INTEGRATION.md` section 3) |
| Stabilisation | `release/m2` | only if needed, then deleted | only fixes, no features |
| Milestone marker | tags `m2-t`, `m2`, `m3` | permanent | annotated tags made by the Integration Owner after the owner's acceptance entry |

No other permanent branch is proposed.

## 5. How unfinished work stays out of the way on `master`

1. A new crate that is not yet referenced by the composition root (the server host / client build) cannot affect the product.
2. Temporary contracts are registered with expiry (`M2_MINIMUM_CONTRACTS.md`); tests tagged `provisional`.
3. Anything experimental stays on `spike/*`.
4. Feature flags are not proposed: they add configuration surface without need at this size.

## 6. Control of API contracts

* `docs/api-contracts/` and the committed API snapshots (`CORE-10`, `API-05`) are changed only by PRs that the Integration Owner prepares and the Project Owner reviews; a CODEOWNERS file listing these paths is proposed (a repository file; **not created here**; it would assign review to the owner's account).
* The ADG tables and the audit scripts are in the same protected set.
* The decision registry is changed only by the owner or by an agent quoting an owner statement with a confirmation source.

## 7. Required checks and protection (what the repository can actually support)

Real situation: **no workflow exists**; GitHub Actions is available for a public repository; branch protection on a public repository needs the owner's administrative action. A check that is only described is not a gate (`integration-and-qa-plan.md` section 3).

| Stage | Required status checks (proposal) | Notes |
|---|---|---|
| Documentation only (first) | `docs-validate`: runs the audit scripts that are stable (see `REPOSITORY_AND_MERGE_REVIEW.md` section 6) on a clean checkout | needs two of the committed scripts fixed first (they encode state of one commit) |
| After the CI skeleton (`GOV-06`) | `fmt` (rustfmt check), `build` (compile all crates), `unit-tests`, `integration-tests` (`cube-integration-tests`), `clippy -D warnings`, `deny` (licence and advisories), `archcheck` (`XT-01/02`), `adg-validate` (structure checks of `adg_check.py`, plus the cargo-metadata crate graph against Part B) | each is added as a required check only after it has run red on a seeded failure and green on the skeleton |
| Later | benchmarks within recorded baselines (desktop), API snapshot test, device runs as non-required | |

Protection settings proposed for `master`: require a PR; require the checks above; require review by the owner for the paths of section 6; dismiss stale reviews; disallow force-push and deletion; linear history is optional. All are **owner actions**.

## 8. Order of work (steps 1-6 of the brief)

| # | Step | Content | Needs |
|---|---|---|---|
| 1 | **Documentation integration** | new docs-only branches from `master`, tree-based (not cherry-pick), as the PR set of `REPOSITORY_AND_MERGE_REVIEW.md` section 5; the spike code and (by default) `docs/spike` stay out | owner decision ODM-01; owner approval of each PR; fixes listed in that document |
| 2 | **CI and protection** | `docs-validate` workflow first; protect `master` with it; then the Rust skeleton PR (`GOV-06`) with the full check set | `GOV-04`, `GOV-05` (CI specification Owner-Approved), D-E parameters, owner action for settings |
| 3 | **M2-T preparation** | closure of PS-0 (`GOV-01..06`, `TEST-01`, `XT-01`), owners, temporary contracts registered, M2-T/M2 scope accepted, ADG change proposals (slice, Game Rules, D-01/D-02) accepted | ODM-02..ODM-08 |
| 4 | **Technology Track implementation** | `prod/ps...` branches by dependency level (`M2_TASK_GRAPH.md` section 7) | steps 2-3 |
| 5 | **Game Track implementation** | `game/...` branches for Game Rules, UI, content; L1/L2 work (design, scenarios, spikes) may start before step 3, L3 only after it | owners; Game Rules module in the ADG |
| 6 | **First playable integration** | `XI-01`, `XI-02` on `master` through PRs; tag `m2-t`; later `XI-03`, tag `m2` | all above |

## 9. Commands for step 1 (documented, **not executed**)

```bash
# one branch per PR, tree-based; the base is master, nothing is merged or rewritten
git switch -c docs/baseline-decisions master
git checkout 362a124 -- docs/decisions docs/architecture docs/research docs/api-contracts docs/README.md
# review links to docs/spike, edit docs/README.md accordingly, commit, open the PR for owner review
```
The same pattern applies to the other PRs. Creating the branches and opening the PRs needs the owner's permission.

## 10. What not to create

No `development/*` pair, no per-module long-lived branches, no feature-flag framework, no release branch before M2 stabilisation, no mirrored branches for each stage.
