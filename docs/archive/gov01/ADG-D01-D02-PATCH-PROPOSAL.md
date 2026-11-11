# ADG D-01 / D-02: patch proposal

**Status:** **Proposed.** Nothing is applied. `docs/production/module-boundaries.md` is unchanged (`git diff 2c0c1f3 HEAD -- docs/production` empty). Applying needs the Project Owner's acceptance (ST-2) and an Integration Owner review. This document is the exact change plus how it was checked.

## 1. What changes

1. **R-MERGE-G1** (rule): each of the 102 engineering tasks of PS-2...PS-14 (not `GOV-*`, `SPK-*`, `DEV-*`) gets `G1` in its *merge-after* (I) column. No start-after (S) edge is added.
2. **Release closure:** `QA-04` (a member of G14) gets merge-after `G5, G6, G7, G8, G9, G11, G12` (the gates that are not already predecessors after rule 1; G1 comes from rule 1). An owner-approved exclusion (`REL-EXCL-n`) would remove a gate from this list; none is proposed.
3. **Part D** (generated stage table): `G1` is added to the merge-after column of PS-2...PS-14; the seven gates are added to the PS-14 row.
4. **Part E** (validation): add the checks listed in section 4.

Items 1-3 are given as an exact patch: `docs/archive/gov01/ADG-D01-D02.proposed.patch` (unified diff, `-U0`, sha256 prefix `3b7f90e1827789d2`, 245 diff lines, 102 task rows in Part C plus the Part D stage rows). The order of ids inside a generated cell may differ from what the generator would print; the content is what matters.

## 2. Reproduction (commands that were run)

```
git apply --check --unidiff-zero docs/archive/gov01/ADG-D01-D02.proposed.patch     # against the real tree; applies cleanly, nothing changed
# copy module-boundaries.md to a scratch directory, `git apply --unidiff-zero` the patch there,
# then: adg.load(<patched copy>) and adg.run_checks(...)  -> all 8 checks PASS, no cycle, longest path 25 tasks
python docs/archive/audit/adg-fix-simulation.py                                      # in-memory before/after
python docs/archive/audit/adg_check.py                 # committed ADG: structure PASS, 3 policy findings FAIL (expected)
python docs/archive/audit/adg_check.py --proposed --strict   # proposed ADG: all PASS, exit 0
python docs/archive/audit/adg_check.py --determinism   # two runs produce the same report hash
```

Output of the in-memory simulation (as run):

```
== BEFORE (committed ADG)
PASS [structure] no cycles 
PASS [structure] all referenced tasks/gates/external nodes exist 
PASS [structure] no task depends on a later stage 
PASS [structure] gate membership consistent (stage match, one gate per task) 
FAIL [policy] G14 release closure: every gate is a predecessor of G14 or an owner-approved exclusion (missing: G1, G5, G6, G7, G8, G9, G11, G12)
FAIL [policy] G1 is a predecessor of G2..G14 (missing for: G2, G3, G4, G5, G6, G7, G8, G9, G10, G11, G12, G13, G14)
FAIL [policy] G1 merge policy: every engineering task of PS-2+ has G1 as a predecessor (102 tasks without G1 ancestor, e.g. ADP-01, ADP-02, ADP-03, API-01, API-02, API-03)
PASS [structure] explicit exclusions refer to existing gates 
longest path to G14: 21 tasks
== AFTER R-MERGE-G1 only
PASS [structure] no cycles 
PASS [structure] all referenced tasks/gates/external nodes exist 
PASS [structure] no task depends on a later stage 
PASS [structure] gate membership consistent (stage match, one gate per task) 
FAIL [policy] G14 release closure: every gate is a predecessor of G14 or an owner-approved exclusion (missing: G5, G6, G7, G8, G9, G11, G12)
PASS [policy] G1 is a predecessor of G2..G14 
PASS [policy] G1 merge policy: every engineering task of PS-2+ has G1 as a predecessor 
PASS [structure] explicit exclusions refer to existing gates 
longest path to G14: 25 tasks
== AFTER R-MERGE-G1 + release closure
PASS [structure] no cycles 
PASS [structure] all referenced tasks/gates/external nodes exist 
PASS [structure] no task depends on a later stage 
PASS [structure] gate membership consistent (stage match, one gate per task) 
PASS [policy] G14 release closure: every gate is a predecessor of G14 or an owner-approved exclusion 
PASS [policy] G1 is a predecessor of G2..G14 
PASS [policy] G1 merge policy: every engineering task of PS-2+ has G1 as a predecessor 
PASS [structure] explicit exclusions refer to existing gates 
longest path to G14: 25 tasks
== task rows changed by R-MERGE-G1: 102 (every engineering task of PS-2+ gets merge-after G1)
== QA-04 merge-after before: ['G13']
== QA-04 merge-after after : ['G13', 'G1', 'G5', 'G6', 'G7', 'G8', 'G9', 'G11', 'G12']
```

## 3. Predecessor table (CR-07)

| Task/Gate | Current predecessors (gates) | Required predecessors (gates) | Reason | Proposed change |
|---|---|---|---|---|
| G0 | - | - | start of the chain | none |
| G1 | G0 | G0 | closes PS-1; becomes a predecessor of every later gate through the proposed rules | none for G1 itself |
| G2 | G0 | G0, G1 | mutation-boundary tests (TEST-06) and INT-01 sit in G1; later merges must not overtake them | rule R-MERGE-G1 (engineering tasks of this stage merge after G1) |
| G3 | G0, G2 | G0, G1, G2 | mutation-boundary tests (TEST-06) and INT-01 sit in G1; later merges must not overtake them | rule R-MERGE-G1 (engineering tasks of this stage merge after G1) |
| G4 | G0, G2, G3 | G0, G1, G2, G3 | mutation-boundary tests (TEST-06) and INT-01 sit in G1; later merges must not overtake them | rule R-MERGE-G1 (engineering tasks of this stage merge after G1) |
| G5 | G0, G2, G3 | G0, G1, G2, G3 | mutation-boundary tests (TEST-06) and INT-01 sit in G1; later merges must not overtake them | rule R-MERGE-G1 (engineering tasks of this stage merge after G1) |
| G6 | G0, G2, G3, G4, G5 | G0, G1, G2, G3, G4, G5 | mutation-boundary tests (TEST-06) and INT-01 sit in G1; later merges must not overtake them | rule R-MERGE-G1 (engineering tasks of this stage merge after G1) |
| G7 | G0, G2, G3, G4 | G0, G1, G2, G3, G4 | mutation-boundary tests (TEST-06) and INT-01 sit in G1; later merges must not overtake them | rule R-MERGE-G1 (engineering tasks of this stage merge after G1) |
| G8 | G0, G2, G3, G4 | G0, G1, G2, G3, G4 | mutation-boundary tests (TEST-06) and INT-01 sit in G1; later merges must not overtake them | rule R-MERGE-G1 (engineering tasks of this stage merge after G1) |
| G9 | G0, G2, G3, G4, G5 | G0, G1, G2, G3, G4, G5 | mutation-boundary tests (TEST-06) and INT-01 sit in G1; later merges must not overtake them | rule R-MERGE-G1 (engineering tasks of this stage merge after G1) |
| G10 | G0, G2, G3, G4 | G0, G1, G2, G3, G4 | mutation-boundary tests (TEST-06) and INT-01 sit in G1; later merges must not overtake them | rule R-MERGE-G1 (engineering tasks of this stage merge after G1) |
| G11 | G0, G2, G3, G4, G10 | G0, G1, G2, G3, G4, G10 | mutation-boundary tests (TEST-06) and INT-01 sit in G1; later merges must not overtake them | rule R-MERGE-G1 (engineering tasks of this stage merge after G1) |
| G12 | G0, G2, G3, G4 | G0, G1, G2, G3, G4 | mutation-boundary tests (TEST-06) and INT-01 sit in G1; later merges must not overtake them | rule R-MERGE-G1 (engineering tasks of this stage merge after G1) |
| G13 | G0, G2, G3, G4, G10 | G0, G1, G2, G3, G4, G10 | mutation-boundary tests (TEST-06) and INT-01 sit in G1; later merges must not overtake them | rule R-MERGE-G1 (engineering tasks of this stage merge after G1) |
| G14 | G0, G2, G3, G4, G10, G13 | G0, G1, G2, G3, G4, G5, G6, G7, G8, G9, G10, G11, G12, G13 | release validation must follow every in-scope stage; owner-approved exclusions: none | QA-04 merge-after += G1, G2, G3, G4, G5, G6, G7, G8, G9, G10, G11, G12 |
| PROC-01 | S: CORE-10; M: CORE-06 | S: CORE-10; M: CORE-06, G1 | representative task of the rule | merge-after += G1 |
| CORE-15 | S: CORE-10; M: CORE-10, GOV-16 | S: CORE-10; M: CORE-10, GOV-16, G1 | representative task of the rule | merge-after += G1 |
| CORE-16 | S: CORE-10; M: CORE-10 | S: CORE-10; M: CORE-10, G1 | representative task of the rule | merge-after += G1 |
| PERS-01 | S: CORE-11, API-04; M: CORE-11 | S: CORE-11, API-04; M: CORE-11, G1 | representative task of the rule | merge-after += G1 |
| API-01 | S: CORE-10; M: CORE-07 | S: CORE-10; M: CORE-07, G1 | representative task of the rule | merge-after += G1 |
| QA-04 | S: G13; M: G13 | S: G13; M: G13, G1, G5, G6, G7, G8, G9, G11, G12 | release closure | merge-after += G1 |

## 4. Validator checks required after the change (implemented in `docs/archive/audit/adg.py`, not wired into CI)

| Check | Implemented | Kind |
|---|---|---|
| no cycles | yes | structure |
| all referenced tasks/gates/external nodes exist | yes | structure |
| no task depends on a later stage | yes | structure |
| gate membership consistent (stage match, one gate per task) | yes | structure |
| predecessor closure: G1 is a predecessor of G2...G14 | yes | policy |
| G1 merge policy: every engineering task of PS-2+ has G1 as predecessor | yes | policy |
| G14 release closure: every gate is a predecessor of G14 or an owner-approved exclusion | yes | policy |
| explicit exclusions refer to existing gates | yes | structure |
| deterministic output (sorted, no timestamps; two runs identical) | yes (`--determinism`) | tooling |
| required gate ancestry per stage (the "predecessor stage" table, ANALYSIS 3.3) | **no**: the table does not exist yet | policy, pending |

`structure` checks must pass; `policy` checks are findings against the proposed rules (they fail on the committed ADG, which is the point). **None of this is a CI gate:** the repository has no CI and no required status check. Wiring `adg_check.py --strict` into CI is a GOV-05/GOV-06 matter.

## 5. Not covered / open

* The generator of the tables is not in the repository (F-5): after acceptance either commit it with the rule, or declare the markdown the source and apply the patch.
* Which stages are optional for release (owner decision; no exclusion proposed).
* The per-stage required-predecessor table (ANALYSIS 3.3).
* Part E text in `module-boundaries.md` is not patched (the new checks are listed above, not written into the markdown).
