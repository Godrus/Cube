# ADG defects D-01 and D-02: analysis

**Status:** **Proposed.** The ADG (`docs/production/module-boundaries.md` Parts C, D, E, committed in `2c0c1f3`, unchanged since: `git diff 2c0c1f3 HEAD -- docs/production` is empty, *verified*) is **not modified**. Everything below was computed on an in-memory copy by `docs/audit/adg-fix-simulation.py` (uses the parser `docs/audit/ia.py`). Changing the ADG needs a separate approval and regeneration of Parts C/D (the data file `_build/data.py` of the plan is outside the repository; see section 6).

## 1. D-01 — stage gate G1 constrains nothing

**Exact problem.** In the published graph, `G1` (closing PS-1) is not an ancestor of any task of PS-2…PS-14 and not an ancestor of any gate: the gate list of ancestors is `G1:[G0]`, `G2:[G0]`, `G3:[G0,G2]`, … (table in section 4). PS-2 tasks `PROC-01`, `CORE-15`, `CORE-16` start after `CORE-10` and merge after `CORE-06`/`CORE-10`, never after `G1`.
**Affected tasks.** G1 members never required by later work: `TEST-06` (mutation-boundary compile-fail tests), `INT-01` (record that P0-alpha is Implemented), `CORE-11` (for G4), and by extension all 16 G1 members as a *set*. Later engineering tasks without any G1 ancestor: 102 of the PS-2+ engineering tasks (all but `GOV-*`, `SPK-*`, `DEV-*`).
**Affected gates.** G1 itself (dead end), and the meaning of G2…G14 (they do not imply PS-1 completion).
**Consequences.** (1) PS-1 can stay open while Processing, Editing API, Orchestration, Persistence and WorldGen merge. (2) Mutation boundary MB-1 (compile-fail tests, archcheck v1) is not a prerequisite of any merge that uses `UserCommit`/`SystemCommit`/`RecoveryCommit`, notably `PERS-*` (RecoveryCommit) and `WGEN-*`. (3) The rule "no next stage before the previous gate" is not machine-expressed. The validator cannot notice, because it checks only that G1 is isolated *from later work*.
**Note on intent.** The plan deliberately lets later stages *start* early (S edges on `CORE-10`), so the fix must be on the merge side (I edges), not the start side.

## 2. D-02 — the release gate G14 does not require most stages

**Exact problem.** Ancestors of `G14` among gates: `G0, G2, G3, G4, G10, G13`. Missing: `G1, G5, G6, G7, G8, G9, G11, G12`.
**Affected tasks/gates.** G14 members `QA-02, QA-04, QA-06, DG-1, DG-2, DG-3, GOV-13, GOV-15`. Their ancestors do not include Backends/Derived Providers (PS-5), Tools (PS-6), History (PS-7), Composite (PS-8), Resampling (PS-9), WorldGen integration (PS-11) or Client Runtime/Adapter (PS-12) as gates.
**Consequences.** "Full QA and Release Validation" can be accepted formally with those stages unbuilt; `DG-1…DG-3` depend on specific tasks only (e.g. `ADP-01`, `VOX-03`, `CLI-02`), so some stage tasks are reached but not their gates. Milestones M3…M8 of the roadmap would rest on a graph that does not enforce completeness.
**Not a defect to fix blindly:** if the owner intends a stage to be optional for release (for example PS-8/PS-9), that must be a recorded scope exclusion, not an omission.

## 3. Proposed corrections (shown as a diff, not applied)

### 3.1 Minimal variant (4 rows)

```
--- PROC-01
- | PROC-01 | PS-2 | Processing Agent | CORE-10 | CORE-06 |
+ | PROC-01 | PS-2 | Processing Agent | CORE-10 | CORE-06, G1 |
--- CORE-15
- | CORE-15 | PS-2 | Core Agent | CORE-10 | CORE-10, GOV-16 |
+ | CORE-15 | PS-2 | Core Agent | CORE-10 | CORE-10, GOV-16, G1 |
--- CORE-16
- | CORE-16 | PS-2 | Core Agent | CORE-10 | CORE-10 |
+ | CORE-16 | PS-2 | Core Agent | CORE-10 | CORE-10, G1 |
--- QA-04
- | QA-04 | PS-14 | Agent I (QA and Benchmarks) | G13 | G13 |
+ | QA-04 | PS-14 | Agent I (QA and Benchmarks) | G13 | G13, G1, G5, G6, G7, G8, G9, G11, G12 |
```
Part D text (generated) then lists these in the PS-2 and PS-14 "Merge-after" columns, and the G2 closing line gains "G1 satisfied".

Simulation result: no cycle; G1 becomes an ancestor of `G2…G14`; every gate becomes an ancestor of `G14`. **Residual:** 25 PS-2+ engineering tasks still have no G1 ancestor *through their own edges* (`PROC-03`, `PROC-04`, `API-01/03/04/05`, `CORE-12/13/14`, `BOO-01`, `SCU-01`, `PERS-01…07`, `PERS-09`, `QA-P2`, `TEST-04`, `CLI-00/01/04`, `QA-02`). They are reachable only because their *gates* depend on G1; they could merge individually before G1. Hence variant 3.2 for persistence safety.

### 3.2 Rule variant (recommended): one stage-entry rule instead of 100 edits

Add to Part C a generated rule **R-MERGE-G1:** every engineering task of PS-2…PS-14 (not `GOV-*`, `SPK-*`, `DEV-*`) has implicit merge-after `G1`; and to `QA-04` the merge-after list `G5, G6, G7, G8, G9, G11, G12` (and `G1` implied). The generator expands the rule into the table (102 tasks) so Part C stays the single readable source; the validator asserts it.
Simulation: **no cycle**; every PS-2+ engineering task has `G1` as ancestor; every gate is an ancestor of `G14`; longest dependency chain grows from 21 to **25 task nodes** (PS-1 closing now sits on the critical path, which is what the plan intended: "do not start the next stage without its entry criteria").

### 3.3 What the fix does not decide (design questions for the Integration Owner / owner)

Currently-present ancestor gates (section 4): G5 lacks G4; G7 and G8 lack G5 and G6; G10 lacks G5-G9; G11 lacks G5-G9 and G12; G12 lacks G10 and G11; G13 lacks G11 and G12. These parallel branches may be intended (backends parallel with orchestration, client parallel with persistence). **The fix does not add them.** A "predecessor stage" table (which prior gates each stage must see) should be written by the Integration Owner and then asserted by the validator; that is a plan change needing owner acceptance.

## 4. Gate ancestry before the fix (simulation output, verified)

| Gate | Ancestor gates before |
|---|---|
| G1 | G0 |
| G2 | G0 |
| G3 | G0, G2 |
| G4 | G0, G2, G3 |
| G5 | G0, G2, G3 |
| G6 | G0, G2, G3, G4, G5 |
| G7 | G0, G2, G3, G4 |
| G8 | G0, G2, G3, G4 |
| G9 | G0, G2, G3, G4, G5 |
| G10 | G0, G2, G3, G4 |
| G11 | G0, G2, G3, G4, G10 |
| G12 | G0, G2, G3, G4 |
| G13 | G0, G2, G3, G4, G10 |
| G14 | G0, G2, G3, G4, G10, G13 |

## 5. Checks on the fix

| Check | Minimal variant | Rule variant |
|---|---|---|
| cycle (DFS over S ∪ I ∪ gate membership) | none | none |
| unknown references | none (G1 exists) | none |
| stage inversion (task depends on a later stage) | none (G1 is PS-1, depender PS-2+) | none |
| missing dependencies introduced | none; one *new constraint* on PS-2+ merges | none |
| G1 ancestor of G2…G14 | yes | yes |
| every gate ancestor of G14 | yes | yes |
| every PS-2+ engineering task has a G1 ancestor | no (25 residual) | yes |
| longest chain | 25 | 25 |
| contradicts a plan statement | no; plan sec. 15 says PS-1 exit is verifiable; it supports the fix | no |
| new risk | merge of PS-2 can no longer overtake PS-1 closure: if INT-01/TEST-06 are late, PS-2 merges wait (start is unaffected) | same |

## 6. Process notes

* The ADG source is the data file `_build/data.py` in the previous session's scratchpad (not in the repository). `module-boundaries.md` says Part C is "generated and machine-validated", but the generator is not committed. To change the ADG reproducibly, either commit the generator or declare the markdown tables the source. This is an additional finding (F-5 in `PS-0-REMAINING-BLOCKERS.md`).
* Validator extension needed: assert (a) `G1` is an ancestor of `G2…G14`; (b) every gate is an ancestor of `G14` unless listed in an explicit, owner-approved exception list; (c) the predecessor-stage table once written.
* Owner decision needed (ST-2 in the status package): accept D-01/D-02 as corrections, and say whether any stage is optional for release.
