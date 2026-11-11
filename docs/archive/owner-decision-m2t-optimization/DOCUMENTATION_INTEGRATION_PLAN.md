# Documentation integration plan (corrected, proposed)

**Status:** **Proposed** / `PENDING_OWNER_DECISION` (ODM-01). Nothing was merged, committed, pushed, branched or changed in the repository. Corrects the D1-D4 plan of `docs/archive/m2-delivery-review/REPOSITORY_AND_MERGE_REVIEW.md` using facts re-verified on 2026-11-08.

## 1. Facts (verified)

| Item | Result |
|---|---|
| Branch / HEAD | `docs/architecture-roadmap-v1`, `a8584cdd8a6377fbec1a5e8d2d8a6d06c54d4609`, equal to `origin`; working tree: only ` M .gitignore` (the owner's local additions: `AGENTS.md` and a list of `docs/archive/m2-delivery-review/_build/...` paths) |
| `master` | `bf60534`, 13 files; the branch is **39 commits ahead, 0 behind**: fast-forward possible, no conflicts |
| New since the previous review | commit `a8584cd` adds `docs/archive/m2-delivery-review/` (the ten documents of the previous package, identical to the scratchpad originals); the generator `_build/` is **not** committed (ignored locally) |
| CI / protection | no `.github/`; `master` unprotected (GitHub API, 2026-11-07) |
| Signed commits | none of the 39 |
| Decision registry | absent |

## 2. Packages (exact files)

Packages are created tree-based from `master` (not by cherry-picking the mixed history). Files identical to `master` are not part of a PR. `docs/spike` (170 files) and all code are outside D1-D4.

**D1 Recorded decisions, architecture proposals, research, contract drafts**: 24 files in the package; 14 new, 3 modified on `master`, 7 identical to `master` (not part of the PR).

Modified: `docs/archive/README.md`, `docs/api-contracts/README.md`, `docs/decisions/audit-resolution-report.md`

New: `api-contracts/server-runtime-contracts.md`, `api-contracts/system-commit-contracts.md`, `api-contracts/world-persistence-contracts.md`, `api-contracts/worldgen-contracts.md`, `architecture/roadmap-v1.md`, `architecture/system-boundaries-v1.md`, `decisions/acr-004-owner-review.md`, `decisions/owner-approval-matrix-v1.md`, `decisions/owner-architecture-decisions-v1.md`, `decisions/owner-decision-rust-godot.md`, `decisions/worldgen-core-boundary.md`, `research/cubeworld-reversal-notes.md`, `research/reference-comparison.md`, `research/veloren-notes.md`

**D2 Production plan and status documents**: 13 files in the package; 13 new, 0 modified on `master`, 0 identical to `master` (not part of the PR).

New: `AGENT_HANDOFF.md`, `DEVELOPMENT_ROADMAP.md`, `PROJECT_STATUS.md`, `RELEASE_CRITERIA.md`, `production/agent-task-breakdown.md`, `production/architecture-blocker-decisions.md`, `production/implementation-roadmap.md`, `production/integration-and-qa-plan.md`, `production/module-boundaries.md`, `production/production-architecture-plan.md`, `production/production-plan-revision-v2-report.md`, `production/production-plan-revision-v2.1-report.md`, `production/ps-0-exit-checklist.md`

**D3 Audit package and GOV-01 package**: 22 files in the package; 22 new, 0 modified on `master`, 0 identical to `master` (not part of the PR).

New: `audit/Independent-Production-Audit.md`, `audit/PROJECT-STATUS-VALIDATION.md`, `audit/adg-fix-simulation.py`, `audit/adg.py`, `audit/adg_check.py`, `audit/gametrack_check.py`, `audit/gen.py`, `audit/gov01_check.py`, `audit/ia.py`, `audit/m2_slice.py`, `audit/val.py`, `gov01/ADG-D01-D02-ANALYSIS.md`, `gov01/ADG-D01-D02-PATCH-PROPOSAL.md`, `gov01/ADG-D01-D02.proposed.patch`, `gov01/GOV-01-CORRECTIVE-REVIEW.md`, `gov01/GOV-01-DECISION-REGISTER.md`, `gov01/GOV-01-POST-REVIEW-CORRECTIONS.md`, `gov01/GOV-01-TECHNICAL-RECOMMENDATIONS.md`, `gov01/GOV-01-VALIDATION-REPORT.md`, `gov01/OWNER-DECISION-REQUIRED.md`, `gov01/PS-0-REMAINING-BLOCKERS.md`, `gov01/ST-1-CANONICAL-STATE-RECOMMENDATION.md`

**D4 Game Track and M2 delivery review**: 15 files in the package; 15 new, 0 modified on `master`, 0 identical to `master` (not part of the PR).

New: `m2-delivery-review/DEVELOPMENT_BRANCH_STRATEGY.md`, `m2-delivery-review/M2_DELIVERY_PLAN.md`, `m2-delivery-review/M2_MINIMUM_CONTRACTS.md`, `m2-delivery-review/M2_OWNERSHIP_MATRIX.md`, `m2-delivery-review/M2_SCOPE_REVIEW.md`, `m2-delivery-review/M2_TASK_GRAPH.md`, `m2-delivery-review/M3_SCOPE_REVIEW.md`, `m2-delivery-review/OWNER_DECISIONS_REQUIRED.md`, `m2-delivery-review/README.md`, `m2-delivery-review/REPOSITORY_AND_MERGE_REVIEW.md`, `production/FIRST_PLAYABLE_DEPENDENCIES.md`, `production/FIRST_PLAYABLE_SCOPE.md`, `production/GAME_TRACK_ROADMAP.md`, `production/TECHNOLOGY-GAME-TRACK-INTEGRATION.md`, `production/VERTICAL_SLICE_SCOPE.md`


D5 (optional, separate decision): `docs/spike/` (170 files, 2.4 MB).

## 3. Dependencies between packages and independent mergeability

Counts are references in backticks from a package to a file of another package (re-computed; `_build/docscan.py`).

| From | To | References | Consequence |
|---|---|---|---|
| D2 | D1 | 10 | D2 cites decisions and contracts; merging D2 first leaves 10 dangling references |
| D3 | D2 | 14 | the ADG tools read `docs/production/module-boundaries.md`; they cannot run without D2 |
| D3 | D1 | 8 | |
| D4 | D1 / D2 / D3 | 7 / 5 / 4 | |
| D1 | (none) | 0 | **D1 merges independently** (after the spike-link policy below) |
| D1, D2, D3, D4 | spike code or `docs/spike` | 21, 3, 10, 8 | link policy needed |

Order: **D1 -> D2 -> D3 -> D4**; no cycle. Each package after D1 is independently *reviewable* but is *mergeable* only after its predecessors. D3 and D4 could be reordered only if the references from D4 to D3 (4) are marked "forthcoming".

References to missing documents: the only targets that do not exist are the two spellings of the registry (`docs/decisions/decision-registry.md`, `docs/production/decision-registry.md`; future files), the future crate `crates/cube-editing-api` named in the first audit report, and the token `docs/architecture-roadmap-v1` which is a **branch name** (appears in 16 documents; not a file).

## 4. Consistency of the status documents (findings, none corrected)

| # | Finding | Evidence | Needed before merge |
|---|---|---|---|
| I-1 | `PROJECT_STATUS.md` names `2c0c1f3` as the verified baseline and `60d0bfc` as the documentation commit; HEAD is `a8584cd` and the GOV-01, Game Track and M2-delivery packages are not mentioned | text | refresh in D2 (a dated baseline note, the "never write HEAD" rule stays) |
| I-2 | the decision registry is named `docs/production/decision-registry.md` in `PROJECT_STATUS.md`, the first audit reports, `val.py` and GOV-01 files, and `docs/decisions/decision-registry.md` in plan 3.3 | `grep` | owner confirms the path (ODM-10); live documents corrected, dated reports get an erratum |
| I-3 | two M2 definitions (`DEVELOPMENT_ROADMAP.md`, `RELEASE_CRITERIA.md` versus `FIRST_PLAYABLE_SCOPE.md` and the M2-T/M2 split) | `docs/archive/production/GAME_TRACK_ROADMAP.md` C-1 | the owner decides ODM-06 before D2 and D4 are labelled consistent |
| I-4 | `integration-and-qa-plan.md` section 1 states "31 commits ahead, 247 files" (now 39 commits) | text | replace by a pointer to `git` |
| I-5 | the commit hashes cited in documents (`2c0c1f3` in 5 documents, `74566cd` in 6, `5c9a5c5` in 4, `362a124` in 6, `60d0bfc` in 3, `64e8a76` in 2, `22c70c7`, `05d7e7e` and others) refer to **commits of this branch**; tree-based PRs create new commits, so on `master` these hashes do not exist | `git rev-list`, `docscan.py` | **keep the branch** (and tag its tip, for example `archive/docs-roadmap-v1`, an owner action) and add one sentence to the packages: "commit hashes refer to the history of `docs/architecture-roadmap-v1`" |
| I-6 | `owner-architecture-decisions-v1.md` is headed "Approved Owner Decisions" while plan 3.3 says transcribed decisions are "recorded, awaiting confirmation" | text | in D1 the owner's approval of the PR is the natural confirmation if ODM-10 accepts it; otherwise add a status banner "recorded, awaiting confirmation" |
| I-7 | `docs/architecture/geometry-editing-specification-v1.0.md` on `master` says "Architecture approved"; origin of that approval is not in the repository | text | unchanged (already on `master`) |
| I-8 | `AGENTS.md` (untracked, ignored) references `.agent/` which does not exist | `ls` | ST-1 (owner) |
| I-9 | terminology: a scan of the branch documents found no document asserting a completed PS-0, an allowed PS-1, or a status "Owner-Approved" as a fact (`gov01_check.py`, `gametrack_check.py` scans pass in a clean checkout after the fixes below) | executed | none |

Proposed-versus-approved terminology rule for every package: each document keeps its status line; no package PR description says "approved"; the PR description says "records and proposals; not decisions".

## 5. The two failing audit scripts

Re-run on a **fresh clone checked out at `a8584cd`** (scratchpad), then on patched **copies** in that clone (the repository scripts were not changed).

| Script | As committed (clean clone) | Cause | Portable correction (proposed) | Patched copy, clean clone |
|---|---|---|---|---|
| `gov01_check.py` | 29 checks, **1 failed**: "module-boundaries.md unchanged since 2c0c1f3" | the assertion compares the **whole `docs/production` directory** with `2c0c1f3`; documents added there later (Game Track) make it fail. It also **passes falsely in a shallow clone** because the reference commit is missing and `git diff` prints nothing | compare only `docs/production/module-boundaries.md`; **fail closed** when `2c0c1f3` is not available (`git cat-file -e 2c0c1f3^{commit}`) | **29 checks, 0 failed** (full clone); in a shallow clone (`--depth 1`) it fails closed with that check, as intended |
| `gametrack_check.py` | 23 checks, **1 failed**: ".gitignore change is still the owner's single added line" | the assertion encodes a **local, uncommitted state** (the owner's one added line); in a clean clone the diff is empty, and the owner's local list has since grown | assert only that `.gitignore` is unmodified **or** has only added lines | **23 checks, 0 failed** (full clone and shallow clone) |

Diff of the proposed corrections (applied nowhere in the repository):

```diff
--- a/docs/audit/gov01_check.py	2026-11-08 00:26:25.131713700 +0300
+++ b/docs/audit/gov01_check.py	2026-11-08 00:26:25.175710700 +0300
@@ -160,7 +160,8 @@
 ok_p, det_p = patch_reproduction(patch, mb_rel)
 check('patch applies to copies of the baseline with LF and with CRLF line endings, under core.autocrlf true and false, and the patched graph passes all checks (no --ignore-whitespace)', ok_p, det_p)
 check('the repository copy of module-boundaries.md was not modified by the check', hashlib.sha256(open(os.path.join(R, mb_rel), 'rb').read()).hexdigest() == before)
-check('module-boundaries.md unchanged since 2c0c1f3', git('diff', '2c0c1f3', 'HEAD', '--', 'docs/production').strip() == '' and 'docs/production/module-boundaries.md' not in changed)
+history_ok = subprocess.run(['git', '-C', R, 'cat-file', '-e', '2c0c1f3^{commit}'], capture_output=True).returncode == 0   # fail closed when the reference commit is missing (shallow clone)
+check('module-boundaries.md unchanged since 2c0c1f3', history_ok and git('diff', '2c0c1f3', 'HEAD', '--', 'docs/production/module-boundaries.md').strip() == '' and 'docs/production/module-boundaries.md' not in changed)
 
 # 7. P1 consumer analysis (CR-10)
 proc = [t for t in cur.tasks if t.startswith('PROC') or t == 'TEST-08']
--- a/docs/audit/gametrack_check.py	2026-11-08 00:26:25.317452600 +0300
+++ b/docs/audit/gametrack_check.py	2026-11-08 00:26:25.361274300 +0300
@@ -114,7 +114,7 @@
 outside = [p for p in changed + untracked if not p.startswith(allowed) and p != '.gitignore']
 check('no tracked file under docs/production/ was modified (new Game Track files only)', not tracked_prod, str(tracked_prod))
 check('all changes are inside docs/archive/gov01, docs/archive/audit, docs/production (plus the owner\'s .gitignore)', not outside, str(outside))
-check('.gitignore change is still the owner\'s single added line', git('diff', '.gitignore').count('\n+') == 2 and 'AGENTS.md' in git('diff', '.gitignore'))
+check(".gitignore is unmodified, or has only added lines", all(not l.startswith("-") for l in git("diff", ".gitignore").splitlines() if l.startswith(("+", "-")) and not l.startswith(("+++", "---"))))
 check('AGENTS.md is untouched (untracked and ignored)', git('ls-files', 'AGENTS.md').strip() == '' and 'AGENTS.md' not in ' '.join(changed + untracked))
 check('code, contracts and decision records unchanged',
       not [p for p in changed + untracked if p.startswith(('crates/', 'godot-client/', 'scripts/', 'docs/api-contracts/', 'docs/decisions/', 'Cargo'))])
```

Conditions for using them in CI: the workflow must fetch full history (`fetch-depth: 0`), Python available, no network; the scripts are audit aids, they become a *required check* only after they run red on a seeded failure and green on a clean checkout, and after the owner registers them. Other scripts (`adg_check.py` without `--strict`, `val.py`, `ia.py`, `gen.py`, `m2_slice.py`, `adg-fix-simulation.py`) exit 0 in a clean clone; the policy failures printed by `adg_check.py` are the intended D-01/D-02 findings, not defects of the tool.

## 6. Corrected integration plan

| Step | Action | Precondition | Evidence |
|---|---|---|---|
| 0 | decide ODM-10 (confirmation), ODM-01 (this plan), ODM-09 (protection, Actions) | owner | registry entries |
| 1 | fix the two scripts (portable corrections above) and the status documents (I-1...I-4) **on the branch**, in one docs-only commit, with the owner's permission | step 0 | clean-clone run: 29/0 and 23/0 |
| 2 | decide whether to commit the graph generator (`docs/archive/m2-delivery-review/_build`, now ignored locally) so that the M2 graph is reproducible from the repository, or to declare the graph documents the source | owner | |
| 3 | tag the branch tip (`archive/docs-roadmap-v1`) so hashes keep resolving | owner action | tag exists |
| 4 | PR **D1** from `master` (new branch, tree-based); owner approval; link policy for `docs/spike` (include D5, or replace links by "see branch `spike/...`") | steps 0-3 | merged D1 |
| 5 | PR **D2**, then **D3**, then **D4**, each approved by the owner; PR descriptions use the terminology rule | D1 merged | merged |
| 6 | add the `docs-validate` workflow (the stable scripts) and protect `master` with it | D3 merged, ODM-09 | API shows `protected: true` with the required check |

A package can be merged independently only in the order above. D1 can be merged alone; D2 requires D1; D3 requires D2; D4 requires D1-D3.

## 7. What this plan does not do

No PR is opened, no branch created or tagged, no file in the repository is changed, no check is declared green except those actually executed in the scratch clone and reported above.
