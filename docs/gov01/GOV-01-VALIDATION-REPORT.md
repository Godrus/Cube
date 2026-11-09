# GOV-01 Validation Report

**Status:** **Proposed.** Records the checks that were actually run for the corrective pass. A check that was not run is listed as not run. None of these tools is a CI gate (the repository has no CI, no workflow and no verified required status check).

## 1. Repository state

* **Baseline (source) commit of this validation:** `74566cd975df656282d7844e98cea79a3329a18b`. The corrective package was validated as uncommitted working-tree changes on top of this baseline and was committed afterwards as the **package commit** `5c9a5c5`; the HEAD of the repository when this correction was made is also `5c9a5c5`. Branch: `docs/architecture-roadmap-v1` (requested name `architecure_roadmap` does not exist; no branch created or switched). `git merge-base --is-ancestor 74566cd HEAD`: true (HEAD equals `74566cd`). Upstream `origin/docs/architecture-roadmap-v1` contains it.
* The `git rev-parse HEAD` output printed when the checks ran (this is the baseline commit, not the package commit): `74566cd975df656282d7844e98cea79a3329a18b`.
* Working tree after the pass (`git status --short`):

```
M .gitignore
 M docs/audit/adg-fix-simulation.py
 M docs/audit/gen.py
 M docs/audit/ia.py
 M docs/audit/val.py
 M docs/gov01/ADG-D01-D02-ANALYSIS.md
 M docs/gov01/GOV-01-DECISION-REGISTER.md
 M docs/gov01/GOV-01-TECHNICAL-RECOMMENDATIONS.md
 M docs/gov01/PS-0-REMAINING-BLOCKERS.md
 M docs/gov01/ST-1-CANONICAL-STATE-RECOMMENDATION.md
?? docs/audit/adg.py
?? docs/audit/adg_check.py
?? docs/audit/gov01_check.py
?? docs/gov01/ADG-D01-D02-PATCH-PROPOSAL.md
?? docs/gov01/ADG-D01-D02.proposed.patch
?? docs/gov01/GOV-01-CORRECTIVE-REVIEW.md
?? docs/gov01/OWNER-DECISION-REQUIRED.md
```
(` M .gitignore` is the owner's own change and was not touched; nothing is staged or committed.)

* Scope check: no file under `crates/`, `godot-client/`, `scripts/`, `docs/api-contracts/`, `docs/decisions/`, `docs/production/` changed (`gov01_check.py` verifies tracked changes; `git diff 2c0c1f3 HEAD -- docs/production` is empty). Modified existing files are limited to `docs/gov01/*` and the tools in `docs/audit/`; the dated audit reports in `docs/audit/*.md` are untouched.

```
.gitignore                                        |   1 +
 docs/audit/adg-fix-simulation.py                  |  89 ++--------
 docs/audit/gen.py                                 |  46 ++---
 docs/audit/ia.py                                  | 120 +++----------
 docs/audit/val.py                                 |   6 +-
 docs/gov01/ADG-D01-D02-ANALYSIS.md                | 136 +++++++-------
 docs/gov01/GOV-01-DECISION-REGISTER.md            | 205 +++++++++++++++++-----
 docs/gov01/GOV-01-TECHNICAL-RECOMMENDATIONS.md    |  34 ++--
 docs/gov01/PS-0-REMAINING-BLOCKERS.md             |  36 ++--
 docs/gov01/ST-1-CANONICAL-STATE-RECOMMENDATION.md |  40 +++--
 10 files changed, 359 insertions(+), 354 deletions(-)
warning: in the working copy of '.gitignore', LF will be replaced by CRLF the next time Git touches it
warning: in the working copy of 'docs/audit/adg-fix-simulation.py', LF will be replaced by CRLF the next time Git touches it
warning: in the working copy of 'docs/audit/gen.py', LF will be replaced by CRLF the next time Git touches it
warning: in the working copy of 'docs/audit/ia.py', LF will be replaced by CRLF the next time Git touches it
warning: in the working copy of 'docs/audit/val.py', LF will be replaced by CRLF the next time Git touches it
warning: in the working copy of 'docs/gov01/ADG-D01-D02-ANALYSIS.md', LF will be replaced by CRLF the next time Git touches it
warning: in the working copy of 'docs/gov01/GOV-01-DECISION-REGISTER.md', LF will be replaced by CRLF the next time Git touches it
warning: in the working copy of 'docs/gov01/GOV-01-TECHNICAL-RECOMMENDATIONS.md', LF will be replaced by CRLF the next time Git touches it
warning: in the working copy of 'docs/gov01/PS-0-REMAINING-BLOCKERS.md', LF will be replaced by CRLF the next time Git touches it
warning: in the working copy of 'docs/gov01/ST-1-CANONICAL-STATE-RECOMMENDATION.md', LF will be replaced by CRLF the next time Git touches it
```

## 2. Checks run

### 2.1 `python docs/audit/gov01_check.py` (28 checks)

```
PASS register summary table has 15 numbered decisions 
PASS classification counts 11 / 2 / 2 
PASS personal choice 9 / confirm-only 6 
PASS register has 15 decision sections 
PASS GOV-01 task row names all 15 decisions (13 tokens, ACR-001/002/003 counts 3) 
PASS PS-0 exit checklist section 2 has rows 1-15 
PASS X-1 appears in register, blockers and corrective documents 
PASS X-2 appears in register, blockers and corrective documents 
PASS X-3 appears in register, blockers and corrective documents 
PASS X-4 appears in register, blockers and corrective documents 
PASS X-2/X-3/X-4 homes GOV-02, GOV-03, GOV-05 exist in the ADG and are G0 members 
PASS GOV-03 has no dependencies (X-3 independent) 
PASS repository paths named in the GOV-01 documents exist (or are declared future files) 
PASS no package document asserts an approval status as fact 
PASS every GOV-01 document is labelled Proposed 
PASS no contract, decision, production-plan or code file changed in the working tree 
PASS Integration Owner is not given approval authority in this package (quotes of older documents excepted) 
PASS ADG structure checks pass on the committed ADG 
PASS ADG with the proposal: all checks pass, no cycle 
PASS proposal adds no start-after edge 
PASS report is deterministic (two renders identical) 
PASS patch applies cleanly to the committed module-boundaries.md (git apply --check, nothing applied) 
PASS module-boundaries.md unchanged since 2c0c1f3 
PASS no Processing task has GOV-07 or GOV-09 as ancestor (their development does not wait for contract approval) 
PASS GOV-07 is merge-after PROC-02 and PROC-04 (approval follows implementation) 
PASS GOV-09 is merge-after API-02/03/04 (approval follows implementation) 
PASS API-02 has GOV-07 and G2 as ancestors (Editing production waits for Processing approval) 
PASS processing-contracts.md contains no E-xx reference 
28 checks, 0 failed
```

What it proves: the decision count (15), classification counts (11 / 2 / 2), the 9/6 split of personal choices, agreement with the GOV-01 task row and checklist rows 1-15, X-1...X-4 traceability and homes in the ADG, that paths named in backticks exist (or are declared future files), that no document in the package asserts an approval status as fact, that no contract/decision/production-plan/code file changed, that the Integration Owner is not given approval power (quotations of older documents excepted; heuristic), ADG checks, patch applicability, and the CR-10 dependency facts. What it does not prove: the truth of prose statements, architecture soundness, or anything about CI.

### 2.2 ADG checks on the committed tables (`docs/audit/adg_check.py`)

```
tasks 147 gates 15
PASS [structure] no cycles 
PASS [structure] all referenced tasks/gates/external nodes exist 
PASS [structure] no task depends on a later stage 
PASS [structure] gate membership consistent (stage match, one gate per task) 
FAIL [policy] G14 release closure: every gate is a predecessor of G14 or an owner-approved exclusion (missing: G1, G5, G6, G7, G8, G9, G11, G12)
FAIL [policy] G1 is a predecessor of G2..G14 (missing for: G2, G3, G4, G5, G6, G7, G8, G9, G10, G11, G12, G13, G14)
FAIL [policy] G1 merge policy: every engineering task of PS-2+ has G1 as a predecessor (102 tasks without G1 ancestor, e.g. ADP-01, ADP-02, ADP-03, API-01, API-02, API-03)
PASS [structure] explicit exclusions refer to existing gates 
longest path to G14: 21 tasks
G0 predecessor gates: 
G1 predecessor gates: G0
G2 predecessor gates: G0
G3 predecessor gates: G0 G2
G4 predecessor gates: G0 G2 G3
G5 predecessor gates: G0 G2 G3
G6 predecessor gates: G0 G2 G3 G4 G5
G7 predecessor gates: G0 G2 G3 G4
G8 predecessor gates: G0 G2 G3 G4
G9 predecessor gates: G0 G2 G3 G4 G5
G10 predecessor gates: G0 G2 G3 G4
G11 predecessor gates: G0 G2 G3 G4 G10
G12 predecessor gates: G0 G2 G3 G4
G13 predecessor gates: G0 G2 G3 G4 G10
G14 predecessor gates: G0 G2 G3 G4 G10 G13
```
Structure checks pass; the three policy findings fail by design (they are the D-01/D-02 defects). With the proposed rules applied in memory (`--proposed --strict`):

```
tasks 147 gates 15
PASS [structure] no cycles 
PASS [structure] all referenced tasks/gates/external nodes exist 
PASS [structure] no task depends on a later stage 
PASS [structure] gate membership consistent (stage match, one gate per task) 
PASS [policy] G14 release closure: every gate is a predecessor of G14 or an owner-approved exclusion 
PASS [policy] G1 is a predecessor of G2..G14 
PASS [policy] G1 merge policy: every engineering task of PS-2+ has G1 as a predecessor 
PASS [structure] explicit exclusions refer to existing gates 
longest path to G14: 25 tasks
G0 predecessor gates: 
G1 predecessor gates: G0
G2 predecessor gates: G0 G1
G3 predecessor gates: G0 G1 G2
G4 predecessor gates: G0 G1 G2 G3
G5 predecessor gates: G0 G1 G2 G3
G6 predecessor gates: G0 G1 G2 G3 G4 G5
G7 predecessor gates: G0 G1 G2 G3 G4
G8 predecessor gates: G0 G1 G2 G3 G4
G9 predecessor gates: G0 G1 G2 G3 G4 G5
G10 predecessor gates: G0 G1 G2 G3 G4
G11 predecessor gates: G0 G1 G2 G3 G4 G10
G12 predecessor gates: G0 G1 G2 G3 G4
G13 predecessor gates: G0 G1 G2 G3 G4 G10
G14 predecessor gates: G0 G1 G2 G3 G4 G5 G6 G7 G8 G9 G10 G11 G12 G13
```

### 2.3 Patch

* `git apply --check --unidiff-zero docs/gov01/ADG-D01-D02.proposed.patch`: clean on the real tree, nothing applied.
* The patch was applied to a **copy** of `module-boundaries.md` in a scratch directory; `adg.load` of the copy: all 8 checks PASS, no cycle, longest path 25 tasks.
* Cycle check: none (in memory and on the patched copy). G1 ancestry: G1 is a predecessor of G2...G14 after the patch. G14 closure: every gate is a predecessor after the patch.

### 2.4 Status documents against the ADG (`docs/audit/val.py`)

```
PASS PROJECT_STATUS.md [] [] []
PASS DEVELOPMENT_ROADMAP.md [] [] []
PASS RELEASE_CRITERIA.md [] [] []
PASS AGENT_HANDOFF.md [] [] []
PASS audit/Independent-Production-Audit.md [] [] []
PASS audit/PROJECT-STATUS-VALIDATION.md [] [] []
```

### 2.5 Exit codes of the tools (run from the repository root)

| Command | Exit code |
|---|---|
| `python docs/audit/adg_check.py` | 0 |
| `python docs/audit/adg_check.py --strict` | 1 |
| `python docs/audit/adg_check.py --proposed --strict` | 0 |
| `python docs/audit/adg_check.py --determinism` | 0 |
| `python docs/audit/gov01_check.py` | 0 |
| `python docs/audit/val.py` | 0 |
| `python docs/audit/ia.py` | 0 |
| `python docs/audit/gen.py` | 0 |
| `python docs/audit/adg-fix-simulation.py` | 0 |

`adg_check.py` without `--strict` returns 0 on the committed ADG although the policy findings fail; with `--strict` it returns 1. `ia.py`, `gen.py` and `adg-fix-simulation.py` only print.

### 2.6 Reproducibility
`adg_check.py --determinism` renders the report twice and compares: identical (hash printed in 2.5 run). The tools no longer use `exec` or text splitting; `adg.py` raises `AdgFormatError` if the markdown layout changes.

## 3. Checks not run or not possible

| Check | Why not |
|---|---|
| Markdown rendering and anchor links | no markdown linter or renderer was run; only repository paths in backticks were checked |
| Cross-document consistency by a tool beyond the targeted checks of 2.1 | no general consistency tool exists; the review of CR-01...CR-10 was manual, with file and line citations |
| Regeneration of ADG Part C/D with the real generator | the generator (`_build/data.py` etc.) is not in the repository (F-5) |
| CI execution of any check | the repository has no CI; nothing is registered as a required status check; GitHub settings could not be read (`gh` not installed) |
| `cargo` build/tests | not relevant to this documentation pass; the earlier spike result (131 passing tests) was not re-run |
| Independent confirmation of AOD-01...19 wording | the original sources are absent |

## 4. Result of the specific validations requested

| Validation | Result |
|---|---|
| Path validation | pass (2.1) |
| Cross-document consistency | pass for the targeted checks; prose reviewed manually |
| GOV-01 decision count | 15; 11 / 2 / 2; 9 / 6 |
| X-1...X-4 traceability | pass; X-1 in GOV-01 acceptance, X-2 GOV-02, X-3 GOV-03, X-4 GOV-05 |
| Contract status consistency | no status changed; no contract file touched |
| Owner authority consistency | pass (heuristic) with the older-document conflicts listed in Register 0.2 |
| ADG simulation and graph validation | pass on the committed graph (structure) and on the proposed graph (all) |
| Dependency cycle check | none |
| G1 ancestry analysis | committed: G1 predecessor of no later gate; proposed: of G2...G14 |
| G14 predecessor closure | committed: missing G1, G5-G9, G11, G12; proposed: complete |
| P1 consumer/dependency analysis | in `GOV-01-CORRECTIVE-REVIEW.md` CR-10; facts re-checked by `gov01_check.py` |
| Git diff within the allowed scope | yes (section 1) |

## 5. Not started / not changed

PS-1 not started; no production code; no contract, ACR, AOD, OAM or status changed; no ADG source changed; no commit, push or PR; `.gitignore` and `AGENTS.md` untouched.
