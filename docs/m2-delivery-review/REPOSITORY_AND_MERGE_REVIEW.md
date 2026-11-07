# Repository state and readiness for integration into `master`

**Status:** review, **Proposed**. Every fact below was checked on 2026-11-07 against the live repository and, where stated, against a **clean clone** made in the scratchpad. Nothing was merged, committed, pushed or changed; `master` and the working branch were not touched.

## 1. Verified facts

| Item | Result | How verified |
|---|---|---|
| Branch / HEAD | `docs/architecture-roadmap-v1`, `362a12400935ed5976c0bc7a8ab2e150f56c735b` | `git rev-parse` |
| Working tree | one modification: ` M .gitignore` (the owner's line `AGENTS.md`); no other change, nothing staged | `git status --short` |
| Remote | `origin/docs/architecture-roadmap-v1` = HEAD (after `git fetch`) | `git rev-parse` |
| `master` | `bf60534` (2026-11-02), equals `origin/master`; 13 files, all documentation | `git rev-parse`, `git ls-tree` |
| Relation | `git rev-list --left-right --count master...HEAD` = **0 / 38**: `master` is an ancestor of HEAD, a **fast-forward** is possible | `git merge-base --is-ancestor` |
| Merge conflicts | none: `git merge-tree` (read-only form) reports 0 conflict markers; of the 13 files on `master` three are modified on the branch (`docs/README.md` +29, `docs/api-contracts/README.md` +11, `docs/decisions/audit-resolution-report.md` +46/-5), none deleted | `git merge-tree`, `git diff --name-status` |
| Size | 290 files changed, +22 737 / -5 against `master` | `git diff --shortstat` |
| Commit authorship | all 38 commits are authored under one Git identity (the owner's); **all 38 are unsigned** (`%G?` = N); agent sessions commit under that identity with a `Co-Authored-By` trailer | `git log --format` |
| CI | no `.github/`; GitHub reports **0 workflows** | `ls`, API `actions/workflows` |
| Branch protection | `master` **not protected**, no required status checks, no rulesets | GitHub API (`branches/master`: `protected: false`; `rules/branches/master`: empty) |
| Repository | **public**, default branch `master`, **no licence file** | GitHub API |
| `gh` CLI | not installed (the API was read without authentication, which is enough for these public fields) | `which gh` |

The earlier "Unknown" about GitHub settings is therefore resolved for `master`: **no protection, no CI**. Administrative settings that need authentication (secrets, environments, Actions permissions) remain unread.

## 2. What is on the branch

| Part | Commits | Files | Nature |
|---|---|---|---|
| Spike era (`c11f946` ... `820f538`) | 25 | 42 under `crates/`, 15 under `godot-client/`, 3 under `scripts/`, `Cargo.toml`, `Cargo.lock`, `.gitignore`, 170 under `docs/spike/` | experimental code and reports; roadmap-v1 section 7 and `integration-and-qa-plan.md` section 1 say the code does not go to `master` |
| Documentation era (`05d7e7e` ... `362a124`) | **13**, all touching only `docs/` (verified: `git diff --name-only 820f538 HEAD` has no path outside `docs/`) | decisions, architecture, research, contracts, production plan, status, audit, GOV-01, Game Track | proposals, records, tooling for documents |

Added files by area (excluding the three modified): `docs/spike` 170, `crates` 42, `godot-client` 15, `docs/production` 14, `docs/gov01` 11, `docs/audit` 11, `docs/decisions` 5, `docs/api-contracts` 4, `scripts` 3, `docs/research` 3, `docs/architecture` 2, four top-level status documents, `Cargo.toml`, `Cargo.lock`, `.gitignore`.

## 3. Audit scripts: results and CI readiness

Run on the live tree and on a **clean clone checked out at `362a124`** (scratchpad `ci-sim`).

| Script | Live tree | Clean clone | Assessment |
|---|---|---|---|
| `adg_check.py` | structure checks PASS; 3 policy checks FAIL (the D-01/D-02 findings, by design); exit 0 | same | usable; `--strict` is deliberately red on the committed ADG |
| `adg_check.py --proposed --strict` | PASS, exit 0 | same | |
| `gov01_check.py` | **29 checks, 1 FAILED**: "module-boundaries.md unchanged since 2c0c1f3" | same | **defect:** the assertion compares the whole `docs/production` directory with `2c0c1f3`; the Game Track documents committed there make it fail. It passed before that commit because the files were still uncommitted. My previous report ("29 of 29") was true for that moment and is not true for HEAD |
| `gametrack_check.py` | 23 checks, 0 failed | **1 FAILED**: "`.gitignore` change is still the owner's single added line" | **defect for CI use:** the check encodes a local, uncommitted state |
| `val.py` | 6 PASS | 6 PASS | usable |
| `ia.py`, `gen.py`, `m2_slice.py`, `adg-fix-simulation.py` | exit 0, print only | exit 0 | aids, not gates |

Conclusions: (1) **the scripts are not CI-ready as committed**: two assert facts about one commit or one machine. Fixing them (compare only `module-boundaries.md`; drop the `.gitignore` check or make it local-only) is a prerequisite for a documentation check; this review does not change them. (2) None of the scripts is wired into any CI; their exit codes prove nothing about required checks.

## 4. Answers to the five questions

### A. Can the current branch be integrated into `master` safely?

**No, not as it stands.** Technically a fast-forward without conflicts exists; governance and content forbid it.

### B. Concrete blockers

| # | Blocker | Evidence |
|---|---|---|
| 1 | The branch carries **spike code** (`crates`, `godot-client`, `scripts`, `Cargo.*`) | `git diff --name-only --diff-filter=A master HEAD`; roadmap-v1 section 7; GOV-04 acceptance "no spike file on master" |
| 2 | **Owner approval is required for the documentation branch itself** | roadmap-v1 section 7: the branch is "not for master without a separate owner approval"; the records are proposals "except the AODs, which the owner should confirm when merging" |
| 3 | The branch option for the docs-only baseline (GOV-01 item 15 / ODM-01) is **undecided** | `GOV-01-DECISION-REGISTER.md` item 15 |
| 4 | **No documentation check exists** and two committed scripts fail on a clean checkout (section 3) | clean-clone run |
| 5 | `master` is **not protected**; no required checks | GitHub API |
| 6 | Documents on the branch contradict each other on M2 (`DEVELOPMENT_ROADMAP.md`/`RELEASE_CRITERIA.md` M2 versus `FIRST_PLAYABLE_SCOPE.md`, conflict C-1 in `GAME_TRACK_ROADMAP.md`) and state facts that are out of date (below); merged as "stable" they would mislead | section 8 |
| 7 | Many documents reference `docs/spike/...`, `crates/...`, `scripts/...`; a docs-only baseline needs a link policy (docs/spike in or out) | `grep`: about a dozen distinct `docs/spike` targets in non-spike documents |
| 8 | There is **no confirmation mechanism** for owner decisions: every commit is unsigned and authored under the owner's identity, so "owner-signed commit" (plan 3.3 (a)) cannot be satisfied by the existing history | `%G?` = N; authorship check |

### C. One merge or several PRs?

**Several logical PRs, created tree-based from `master`** (not by cherry-picking the mixed history, not by merging the branch). Reason: reviewable units; the owner can approve decisions separately from proposals; spike code and evidence are excluded or decided separately.

| PR | Content (paths) | Files | Needs |
|---|---|---|---|
| **D1 Recorded decisions and architecture proposals** | `docs/decisions/*`, `docs/architecture/roadmap-v1.md`, `system-boundaries-v1.md`, `docs/research/*`, `docs/api-contracts/*`, `docs/README.md` (+3 modified master files) | 14 added + 3 modified | **owner review item by item**; the owner's approval of this PR is the natural confirmation source for the recorded AOD/OAM records (X-1), if the PR approval is accepted as such (ODM-10) |
| **D2 Production plan and status** | `docs/production/` (9 plan documents) + `docs/PROJECT_STATUS.md`, `DEVELOPMENT_ROADMAP.md`, `RELEASE_CRITERIA.md`, `AGENT_HANDOFF.md` | 13 | merges as **Proposed** with a banner; status documents refreshed first (section 8) |
| **D3 Audit and GOV-01 packages** | `docs/audit/`, `docs/gov01/` | 22 | after the two script defects are fixed; audit reports stay immutable evidence |
| **D4 Game Track** | five Game Track documents in `docs/production/` | 5 | as Proposed, after the owner reads the open questions |
| **D5 Spike evidence (optional, separate decision)** | `docs/spike/` (170 files, 2.4 MB: 14 reports, 45 images, logs) | 170 | owner decision; code stays on `spike/*` branches; if D5 is not taken, links must be rewritten to branch references |

Order: D1 -> D2 -> D3 and D4 (D3 and D4 may be parallel). D5 independent.

### D. Which documents are stable, need Owner Review, or stay Proposed

| Class | Documents | Reason |
|---|---|---|
| **Stable (factual records, mergeable as reference once errata are fixed)** | `docs/audit/*` reports and tools (evidence, immutable after commit); `docs/research/*` (status: research record, Proposed); `docs/spike/*` (experiment reports) | they record what was done or read; none states a decision |
| **Require Owner Review before merge** | `docs/decisions/owner-architecture-decisions-v1.md`, `owner-approval-matrix-v1.md`, `owner-decision-rust-godot.md`, `worldgen-core-boundary.md`, `acr-004-owner-review.md`, the edit to `audit-resolution-report.md`; `docs/architecture/roadmap-v1.md`, `system-boundaries-v1.md`; `docs/api-contracts/*` (all Draft/Proposed) and the edit to its README | they hold decision records awaiting confirmation or change shared contract documents on `master` |
| **Remain Proposed (merge only with a Proposed banner)** | the whole production plan (nine documents), `PROJECT_STATUS.md`, `DEVELOPMENT_ROADMAP.md`, `RELEASE_CRITERIA.md`, `AGENT_HANDOFF.md`, `docs/gov01/*`, the five Game Track documents | explicit status "Proposed" in each header |
| **Not for `master`** | `crates/`, `godot-client/`, `scripts/`, `Cargo.toml`, `Cargo.lock`, spike `.gitignore` additions | spike code |

**No document on the branch is "Owner-Approved".**

### E. Mandatory checks before a documentation merge

1. The PR contains only the paths of its package (path allow-list check).
2. Every document has a status line and no document claims an approval without a registry entry (the "approval assertion" scan of `gov01_check.py`, once fixed).
3. Task, gate and contract identifiers named in documents exist in the ADG (`val.py`).
4. Repository paths named in documents exist, or are listed as future files.
5. ADG structure checks pass (`adg_check.py` without `--strict`).
6. The two defective assertions are fixed; the checks run on a clean checkout.
7. Line endings are handled explicitly by the tools (they are); optional `.gitattributes` (approval requested separately).
8. The owner's explicit approval of the PR (and, for D1, of the recorded decisions).

## 5. Sources of truth (Part 7: missing primary sources)

Four classes are kept apart. This table is the rule for all documents of this review.

| Class | What it contains today | Where |
|---|---|---|
| **1. Confirmed owner decisions** (registry entry with a confirmation source, plan 3.3) | **none** | the registry file does not exist; no commit is signed |
| **2. Decisions recovered from conversation** (agent-transcribed, "recorded, awaiting confirmation") | AOD-01...24; OAM-01/02/03/07 (Approved) and OAM-08 (Conditionally Approved); the Rust/Godot record | `docs/decisions/owner-architecture-decisions-v1.md`, `owner-approval-matrix-v1.md`, `owner-decision-rust-godot.md`. Their primary source is the owner's chat, which is not in the repository |
| **3. Recommendations of previous agents** | OAM recommendations (OAM-09, OAM-10), ACR-004 review, GOV-01 register and recommendations, ADG patch proposal, audit findings, Production Plan v2.1, status and roadmap documents | `docs/decisions/`, `docs/gov01/`, `docs/production/`, status documents |
| **4. New proposals** | Game Track documents (`362a124`) and everything in this review | `docs/production/GAME_TRACK_ROADMAP.md` etc.; this folder |

Absent primary sources: GDD/TDD, Voxel Architecture, Geometry Processing Specification, the original AU-1.0; the reconstructed copy is "not byte-verified". Nothing in classes 2-4 is presented as a primary source in this review.

## 6. Minimum CI and protection (what the repository can support now)

| Layer | Check | Possible now? | Condition |
|---|---|---|---|
| Documents | path allow-list; identifiers exist; paths exist; status labels; ADG structure; approval-assertion scan | **yes** (Python only, ubuntu runner) | fix the two assertions; scripts need full history (`fetch-depth: 0`) |
| Formatting | rustfmt check | after the skeleton (`GOV-06`) | D-E decisions |
| Compilation | build all crates | after the skeleton | |
| Unit tests | `cargo test` | after the skeleton | |
| Integration tests | `cube-integration-tests` | after PS-1 | |
| Architecture audit | `xtask archcheck` (`XT-01/02`) | after `XT-01` | |
| Dependency-graph validation | `adg_check.py` structure + cargo-metadata crate graph against ADG Part B | ADG part now; crate part with `XT-01` | |
| Licences/advisories | `cargo deny` | after the licence policy (D-E) | |
| Document validation (again, after code exists) | same as the first row | yes | |

A check becomes *required* only after it has run green on the skeleton and red on a seeded failure (`integration-and-qa-plan.md` section 3). Enabling protection is an owner action.

## 7. Does this change the PS-0 / PS-1 status?

No. PS-0 is open (0 of 8 gate tasks), PS-1 is not allowed to start. Nothing in this review marks a task, gate or decision as done.

## 8. Findings in the existing documents (recorded, not corrected)

| ID | Finding | Evidence | Handling |
|---|---|---|---|
| F-M2-1 | `gov01_check.py` fails on HEAD (stale assertion) | section 3 | fix before D3 |
| F-M2-2 | `gametrack_check.py` fails on a clean clone (local-state assertion) | section 3 | fix before D3/D4 |
| F-M2-3 | `PROJECT_STATUS.md` names `2c0c1f3` as the verified baseline and `60d0bfc` as the status commit; it does not mention the GOV-01 and Game Track packages | text | refresh in D2 |
| F-M2-4 | `integration-and-qa-plan.md` section 1 states "31 commits ahead, 247 files" (now 38 commits, 290 files) | text vs `git rev-list`, `git diff --shortstat` | replace the numbers by a pointer to git |
| F-M2-5 | The decision registry is named `docs/production/decision-registry.md` in the status documents, audit reports, scripts and GOV-01 files, and `docs/decisions/decision-registry.md` in the plan 3.3 (F-2 of GOV-01) | `grep` | owner confirms the path; correct live documents, erratum for dated reports |
| F-M2-6 | Two M2 definitions on the branch (`DEVELOPMENT_ROADMAP.md`/`RELEASE_CRITERIA.md` versus `FIRST_PLAYABLE_SCOPE.md`) | `GAME_TRACK_ROADMAP.md` C-1 | owner decision ODM-06 |
| F-M2-7 | Roadmap-v1 gate 6 exit requires a physical Android device; the Game Track proposes desktop acceptance plus a device checkpoint | C-3 | ODM-08 |
| F-M2-8 | Repository is public with no licence file; the spike manifest says `UNLICENSED` | GitHub API, `Cargo.toml` | D-E parameter d / ODM-09 |
| F-M2-9 | No commit is signed; one identity for owner and agents | `%G?` | ODM-10 |
| F-M2-10 | `docs/audit/gen.py` no longer edits `PROJECT_STATUS.md`; the stage table there is hand-maintained and can drift | script header | note in D2 |
| F-M2-11 | `.gitignore` on the branch contains spike ignores; the owner's local line (`AGENTS.md`) is not committed; `AGENTS.md` references an absent `.agent/` | `git diff .gitignore`, `ls` | ST-1 (owner) |
