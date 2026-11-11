# Independent Production Audit: Cube, Production Plan v2.1

**Status of this document:** audit report, **Proposed**. It approves nothing and changes nothing in the repository.
**Auditor:** independent read-only pass (new session; the earlier agent's report was treated as *Reported*, not as evidence).
**Date of check:** 2026-11-05 (system date of the session).
**Method legend:** **Verified** = observed directly in this session by a command; **Reported** = stated by a previous agent/document; **Inferred** = derived; **Unknown** = could not be confirmed.

---

## 1. Repository Baseline (Stage A)

| # | Item | Result | Class | How checked |
|---|---|---|---|---|
| 1 | Current branch | `docs/architecture-roadmap-v1` | Verified | `git rev-parse --abbrev-ref HEAD` |
| 2 | HEAD | `2c0c1f37b2ac5f56a37b0995b6b735ab0b04f3e0` | Verified | `git rev-parse HEAD` |
| 3 | Commit `2c0c1f3` exists, is the branch tip, is a commit object | yes | Verified | `git cat-file -t`, `git branch --contains` |
| 4 | Commit author / message | Godrus, `docs: production plan v2.1 (proposal): final consistency review, PS-0 exit checklist, v2.1 report` | Verified | `git show --stat` |
| 5 | Commit content | 9 files, **all** under `docs/production/` (476 insertions, 134 deletions); no file outside that directory | Verified | `git diff --name-only 2c0c1f3~1 2c0c1f3` filtered by `^docs/production/` returned nothing |
| 6 | Working tree | one modification: `.gitignore` (adds the line `AGENTS.md`); nothing else; no untracked files reported | Verified | `git status --short`, `git diff .gitignore` |
| 7 | `AGENTS.md` | present on disk, **not tracked** (`git ls-files AGENTS.md` empty) and **ignored** by the uncommitted `.gitignore` change | Verified | `git ls-files`, diff |
| 8 | Remote | `origin` = `https://github.com/Godrus/Cube.git`; `origin/docs/architecture-roadmap-v1` tip equals local HEAD (`2c0c1f3`) | Verified | `git remote -v`, `git log origin/...` |
| 9 | Branches | local: `docs/architecture-roadmap-v1`, `master`, `spike/rust-godot`, `spike/rust-world-generation`; the same four on `origin` | Verified | `git branch -a` |
| 10 | Divergence from `master` | the docs branch is **33 commits** ahead of `master`; `git diff --shortstat master HEAD` = 259 files, +19 005 / -5 | Verified | `git rev-list --count`, `git diff --shortstat` |
| 11 | Production code | **none.** There is no `xtask/`, no `crates/cube-editing-api`, no `cube-integration-tests`, no `rust-toolchain.toml`, no `deny.toml` | Verified | `ls`, `git ls-files` |
| 12 | Spike code | present on this branch: 7 crates (`cube-core`, `cube-processing`, `cube-editing`, `cube-persistence`, `cube-worldgen`, `cube-server`, `cube-godot-adapter`), ~5 900 lines of Rust in `src/*.rs`, `godot-client/`, `scripts/`, 170 files in `docs/spike/` | Verified | `Cargo.toml`, `git ls-files` |
| 13 | Tests | spike workspace: **131 passed, 0 failed** (`cargo test --workspace --offline`, cargo 1.97.1) | Verified | executed in this session (writes only to the git-ignored `target/`) |
| 14 | CI | **none.** There is no `.github/` directory at all (no workflows) | Verified | `ls .github` -> not found |
| 15 | GitHub required status checks / branch protection | **Unknown.** `gh` is not installed in the environment, so the GitHub API could not be queried; and since no workflow exists, no check can currently be produced that could be required | Unknown (Inferred: none effective) | `gh: command not found` |
| 16 | `archcheck` (production) | **does not exist as code.** The word appears only in the nine `docs/production` documents. A *spike-level* ancillary check exists: `crates/cube-core/tests/dependency_rules.rs` (3 tests) reads crate manifests and enforces a hard-coded allowed-edge table plus "only the adapter may depend on godot"; its header says it "does not close blocker B-9" | Verified | `grep`, file read |
| 17 | Can it detect mutation-boundary bypasses? | **No.** The spike check inspects `Cargo.toml` dependency edges only; there is no `use`-level scan, no commit-handle rule, no trybuild compile-fail test, no commit log | Verified (by reading the test) | file read |
| 18 | `.agent/` and `.cursor/rules/` | **do not exist** in the repository, although `AGENTS.md` instructs agents to read `.agent/README.md`, `.agent/tasks/NEXT_STEPS.md`, `.agent/history/recent.md` and to maintain `.agent/state/CURRENT_CONTEXT.md` | Verified | `ls -a` |
| 19 | GDD/TDD, Voxel Architecture, Geometry Processing Specification | **absent** (marked MISSING in `docs/archive/README.md`); the Architecture Update AU-1.0 is a reconstructed, unverified copy | Verified | `docs/archive/README.md` |
| 20 | `docs/production/decision-registry.md` (the owner-decision registry that GOV-01/02 are supposed to fill) | **does not exist** | Verified | `ls docs/production docs/decisions` |

### 1.1 Reported statements of the previous agent: verification

| Reported claim | Verdict |
|---|---|
| Production Plan v2.1 is commit `2c0c1f3` on `docs/architecture-roadmap-v1` | **Verified** |
| PS-0 is not complete | **Verified (stronger than reported):** no PS-0 gate task has any artefact in the repository: no registry, no CI, no archcheck, no baseline branch |
| "0 of 15 owner decisions confirmed" | **Verified with a clarification.** The 15 items are the decisions that GOV-01/GOV-02 must contain (`ps-0-exit-checklist.md` section 2), each marked NOT DONE and with no registry entry in the repo. It does **not** mean that no decision of the owner exists: AOD-01…24 and OAM-01/02/03/07/08 are recorded in `docs/decisions/`, but as *agent-transcribed records awaiting owner countersignature* (AOD-24 only conditionally) |
| PS-1 not allowed to start | **Verified:** `CORE-01`/`CORE-02`/`CORE-07` start after `G0`; `G0` is unsatisfied |
| Plan contains "31 commits ahead, 247 files versus 13 on master" (`integration-and-qa-plan.md` section 1) | **Stale, minor:** actual values are 33 commits and 259 files (counts moved with the later commits) |
| Reaudit 19/19 PASS, validator "errors: 0" | **Reproduced** (`_build/validate.py` printed `errors: 0`, 147 tasks, 15 gates, 3 external). See section 3 for what this does and does not prove |

## 2. Stage B: findings on the documents

Documents inspected in full or by section: `production-architecture-plan.md`, `module-boundaries.md`, `implementation-roadmap.md`, `agent-task-breakdown.md`, `architecture-blocker-decisions.md`, `integration-and-qa-plan.md`, `production-plan-revision-v2.1-report.md`, `ps-0-exit-checklist.md`, plus the superseded v2 report, `docs/archive/README.md`, `docs/decisions/owner-architecture-decisions-v1.md`.

### 2.1 The mandatory checks

| # | Check | Result | Evidence |
|---|---|---|---|
| 1 | CORE-01…CORE-11 consistent in all documents | **PASS** | every CORE id from the ADG appears in `agent-task-breakdown.md`; no task id in the breakdown is missing from the ADG and vice versa (my own parser); `CORE-10` (API freeze) covers CORE-01..07, `CORE-11` is a gate member |
| 2 | ACR-004 split into parts A-E | **PASS** | `architecture-blocker-decisions.md` section 2 has five parts with owner, contracts, dependencies, stage, acceptance; parts A/D decided in PS-0, C in PS-10, B/E in PS-11/12 |
| 3 | No hidden dependency of PS-1 on WorldGen/Persistence/History | **PASS** | independent ancestor computation: `G1` has no ancestor outside PS-0/PS-1 and no external node; PS-1 owners are only Core Agent (11), Agent I (3), Integration Owner (2) |
| 4 | Processing (Stage 2) exits before production integration of the Editing API (Stage 3) | **PASS** | `G2` is an ancestor of `API-02`, `G3`, all `ORCH-*`, `VOX-01`, `SDF-01`, `MSH-01`, `SRV-01`; `G3` has ancestors `G0,G2` |
| 5 | No Android emulator/device in PS-1 acceptance | **PASS** | `EXT-DEV` is not an ancestor of any gate G0…G13; section 15 of the plan states "no emulator or device run is part of this exit"; DG-1…DG-3 and DEV-01 stand outside |
| 6 | Owner-Approved / Provisional / Implemented / Frozen separated | **PASS (design)** | two axes in `production-architecture-plan.md` section 3.1 (approval, maturity) |
| 7 | Provisional -> Frozen forbidden without revalidation + Owner Approval | **PASS (design)** | plan section 3.1/3.3 and `GOV-15` (depends on `GOV-08` and `EXT-VA-RG1`); but see D-04: nothing enforces it mechanically yet |
| 8 | ADG complete: tasks, dependencies, cycles, gates, owners, external inputs | **PASS with defects D-01, D-02** | section 3 |
| 9 | Task dependencies match roadmap stages | **PASS** | no stage inversion found (no task depends on a task of a later stage) |
| 10 | Conditions to *start* vs conditions to *merge* separated | **PASS (design)** | edge kinds S (start-after) and I (merge-after) |
| 11 | ACR/AOD/OD/OAM/D decisions consistent with earlier architecture | **PASS with remarks** | v2.1 report section 4 maps AOD-01…24 to tasks; I found no contradiction, but AOD wording for AOD-01…19 comes from a chat transcription and the originals (AU-1.0, Voxel Architecture, GDD/TDD, Processing Spec) are absent: **Unknown** whether AOD-01…19 are complete |
| 12 | CI and archcheck are real required checks | **FAIL (reality), PASS (specification)** | specified (GOV-05/06, XT-01/02) but **nothing exists**: no workflow, no xtask, no branch protection verifiable |

### 2.2 Defects and contradictions

Severity: **H** blocks a correct reading of the plan; **M** weakens a guarantee; **L** cosmetic.

| ID | Sev | Where | Finding | Consequence | Proposed correction (needs no owner decision unless stated) |
|---|---|---|---|---|---|
| D-01 | **H** | `module-boundaries.md` Part C/D (ADG), reproduced by my independent parser | **Gate G1 is a dead end in the graph: it is not an ancestor of any task of PS-2 or later, nor of any other gate.** Later stages start after `G0` and individual Core tasks (`CORE-10`, `CORE-07`, `CORE-09`, `CORE-11` in some places) only. In particular `TEST-06` (mutation-boundary compile-fail tests), `INT-01` (the record that P0-alpha is Implemented) and, for `G4`, `CORE-11` are **not** ancestors of `G4`; `TEST-06` and `INT-01` are not ancestors of `G10` or `G11` either | PS-1 can be "open" while PS-2…PS-13 merge: the rule "do not start/merge the next stage before the previous stage's gate" is not expressed in the graph; the mutation boundary (MB-1) is not a prerequisite for Orchestration, Persistence or WorldGen merges; a stage "closing" has no consequence | Add `G1` as a merge-after (I) edge of every PS-2 task that consumes Core (or of `G2`), keep start-after on `CORE-10`; in general add an I edge from each stage's first merging tasks to the previous stage's gate where a stage dependency is intended. Re-run validation |
| D-02 | **H** | same | **Gates G5, G6, G7, G8, G9, G11, G12 are not ancestors of `G14`** (G14 has ancestor gates `G0,G2,G3,G4,G10,G13` only); G1 neither. `G14` (Full QA and Release Validation) therefore does not formally require History, Composite, Resampling, Backends, Tools, WorldGen integration (G11) or the client (G12) | the final release gate can be "satisfied" with whole stages unbuilt; contradicts the intent of a full QA stage and the Release 1.0 scope | Make `G14` (or its first task) merge-after every gate `G1…G13`, or document each omission as an explicit owner-approved scope exclusion. **Needs owner confirmation** only if a stage is meant to be optional |
| D-03 | M | `module-boundaries.md` Part E | The validator's checks test that G1 is *isolated from later work*, i.e. only the upward direction. No check tests that **earlier gates constrain later stages**. The "34 PASS" are consistent with D-01/D-02 | the automatic result gives false assurance about stage ordering | Add checks: for each i>0, `G(i-1)` or a declared substitute is an ancestor of `G(i)` (exceptions listed) |
| D-04 | M | plan section 3.1/4, GOV-15 | The "no Provisional -> Frozen" rule is enforced by documentation and by `GOV-15`'s dependencies only. There is no machine check (registry lint) planned in `GOV-02`/`XT-01` that status columns obey the model | an agent could edit a status column and CI would not notice | Add to `XT-01`/`XT-02` a docs lint: a contract with approval != Owner-Approved cannot have maturity Frozen |
| D-05 | M | `AGENTS.md`, `.gitignore` (uncommitted), `docs/` | `AGENTS.md` mandates `.agent/**` and `.agent/state/CURRENT_CONTEXT.md`; neither exists; `AGENTS.md` itself is untracked and about to be ignored. The production plan contains no task for a handoff/state document | a new agent that follows `AGENTS.md` finds nothing; two competing state sources are likely (`CURRENT_CONTEXT.md` vs `PROJECT_STATUS.md`) | **Owner decision (new, ST-1):** which file is canonical; this package proposes `docs/archive/PROJECT_STATUS.md` canonical and a <=30-line `CURRENT_CONTEXT.md` pointer if the owner keeps `AGENTS.md` |
| D-06 | M | `integration-and-qa-plan.md` sections 1/3, GOV-04/06, XT-01 | Strong dependence on a non-existent `master` baseline: PS-0 requires a docs-only `master` baseline, but `master` has 13 files and the docs branch has 259 (spike). The branch option (checklist item 15) is undecided, and the production crates are named like the spike crates (`crates/cube-core` etc.), so CORE-01 would collide with the spike `cube-core` unless the baseline excludes it | wrong branch choice would silently mix spike and production code | Resolve checklist item 15 first; add to GOV-04 an explicit acceptance "no spike file under `crates/` on the production base" (already worded; keep) |
| D-07 | L | `integration-and-qa-plan.md` section 1 | stale counts "31 commits ahead, 247 files" (actual 33 / 259) | none | replace with "see `git rev-list`" instead of numbers |
| D-08 | L | `ps-0-exit-checklist.md` section 5 | the word "READY FOR OWNER APPROVAL" differs from this audit's vocabulary ("READY FOR OWNER REVIEW"); both mean "not approved" | none | harmonise |
| D-09 | M | missing evidence | GitHub branch protection could not be inspected (`gh` absent) | Unknown whether `master` has any protection | owner or an agent with `gh`/token to export `repos/Godrus/Cube/branches/master/protection` (read-only) |
| D-10 | M | decisions | The 15 PS-0 decisions require owner action before **any** engineering task starts; `GOV-01` is the single root of the longest chain (21 tasks) | the schedule depends on a human decision, not on engineering | none; see PROJECT_STATUS next action |

### 2.3 What I could not check

* The earlier "full technical audit" and "second review" texts were never available to me; nothing here relies on them.
* The truth of AOD-01…19 against the original owner intent (only chat-transcribed records exist).
* Content of absent source documents (GDD/TDD, Voxel Architecture, Processing Spec, original AU-1.0).
* GitHub-side settings (branch protection, rulesets, Actions secrets).
* Device behaviour (no device data in the repository except an emulator report).

## 3. ADG: independent verification

**Method.** I did not reuse `data.py` or `validate.py`'s logic as proof. I wrote a second, independent parser (`ia.py`, kept in the audit evidence) that reads only the **published Part C and Part D tables of `module-boundaries.md`** (as committed in `2c0c1f3`), builds the graph and checks it. I also re-ran the original `validate.py` for reproduction.

| Check | Independent result | Original validator |
|---|---|---|
| tasks / gates / external | 147 / 15 / 3 (the 3 external nodes are named in section C.1) | 147 / 15 / 3 |
| unknown references | 0 | 0 |
| cycles (DFS on S ∪ I ∪ gate membership) | none | none |
| stage inversions | 0 | 0 |
| gate membership: every member belongs to the gate's stage; no task in two gates | yes | yes |
| tasks outside any gate | DEV-01, SPK-NET, SPK-PERS | same |
| task ids in `agent-task-breakdown.md` vs ADG | identical sets | n/a |
| G1 ancestors outside PS-0/PS-1; external ancestors of G1 | none; none | PASS |
| G2 ancestor of every ORCH task, of VOX-01/SDF-01/MSH-01/SRV-01/API-02 | true | PASS |
| `EXT-DEV` ancestor of any gate G0…G13 | no | PASS |
| unassigned-owner tasks | SPK-NET (PS-0) and PS-11…PS-13 tasks only | PASS |
| longest chain | 21 task nodes (a tie: the independent search found an equivalent chain starting `GOV-01 -> GOV-05 -> GOV-06 -> XT-01`; the plan prints `GOV-01 -> GOV-04 -> GOV-06 -> TEST-01`) | 21 |
| **gate chain (new)** | **G1 not an ancestor of any later gate; G5–G9, G11, G12 not ancestors of G14** | not checked |

**What the automatic check does not prove.** Acyclicity and consistency show the table is well formed; they do not show that the *architectural* dependencies are right (for example whether History really needs only `G4`, or whether the persistence format can precede Composite). Those are engineering judgments for the owner/Integration Owner. The new finding D-01/D-02 shows that a well-formed graph can still omit intended stage ordering.

## 4. Verdict inputs

* **Is Production Plan v2.1 a trustworthy working base?** *Yes for PS-0 planning, with two High defects (D-01, D-02) that must be corrected before PS-2+ are scheduled from the ADG.* The plan is internally consistent, acyclic, reproducible, and honest about its Proposed status. The defects concern stage ordering semantics, not PS-0/PS-1 tasks.
* **PS-0 complete?** **No.** 0 of 8 gate tasks have evidence; 0 of 15 decisions confirmed in a registry; no CI, no archcheck, no baseline, no registry file.
* **PS-1 start permitted?** **No.**
* **Next task:** `GOV-01` (owner decision), see `PROJECT_STATUS.md` section 5.7.

## 5. Evidence kept

* `ia.py` independent parser and its output (reproduced in `PROJECT-STATUS-VALIDATION.md`).
* Commands and outputs listed in section 1 (git, ls, cargo test, grep).
* The original validator output: `tasks: 147 gates: 15 external: 3 ... errors: 0`.
