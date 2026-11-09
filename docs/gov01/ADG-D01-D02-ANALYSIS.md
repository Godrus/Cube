# ADG defects D-01 and D-02: analysis (corrective revision)

**Status:** **Proposed.** The ADG (`docs/production/module-boundaries.md` Parts C, D, E) is **not modified**. `git diff 2c0c1f3 HEAD -- docs/production` is empty (verified), so the committed tables are the same as when the plan was produced.
**What changed since the first version of this document:** the analysis now runs on the **real committed tables** through an importable module (`docs/audit/adg.py`, CLI `docs/audit/adg_check.py`), not through an `exec`/text-splitting copy of the parser; the exact change is given as a patch that was applied to a **copy** and re-checked (`ADG-D01-D02-PATCH-PROPOSAL.md`, `ADG-D01-D02.proposed.patch`); start eligibility and merge eligibility are separated; release closure no longer makes every stage mandatory automatically.
**Authority:** correcting the ADG changes the plan; it needs the Project Owner's acceptance (ST-2). The Integration Owner prepares and validates the change.

## 1. Edge semantics (verified in Part C and in `adg.py`)

* **S (start-after):** the dependency's interface is frozen; the task may *begin*.
* **I (merge-after):** the dependency is merged and accepted; the task may *merge*.
* A **gate** `Gn` is satisfied when all its member tasks are merged and accepted.
* The proposal below adds **only I edges**. It adds no S edge, so no task is prevented from *starting* earlier than today; it only prevents the task from *merging* before the stage it builds on is closed. (Checked in code: `apply_g1_merge_rule` appends to `merge` only.)

## 2. D-01: G1 does not constrain later stages

**Exact problem (reproduced on the real tables).** `G1` (closing PS-1) is not a predecessor of any task of PS-2…PS-14 and not a predecessor of any other gate. `python docs/audit/adg_check.py` prints: *G1 is a predecessor of G2..G14: FAIL (missing for G2…G14)* and *G1 merge policy: FAIL (102 tasks without G1 ancestor)*.
**Affected tasks.** The 102 engineering tasks of PS-2…PS-14 (every task except `GOV-*`, `SPK-*`, `DEV-*`), for example `PROC-01` (merge-after `CORE-06` only), `API-02`, `ORCH-*`, `PERS-*`, `WGEN-*`. G1 members never required by any later task: `TEST-06` (mutation-boundary compile-fail tests), `INT-01` (the record that P0-alpha is Implemented); `CORE-11` is required only by the persistence chain.
**Affected gates.** G1 (dead end); the meaning of G2…G14.
**Consequences.** PS-1 can be open while later stages merge; the mutation boundary (MB-1: `UserCommit`/`SystemCommit`/`RecoveryCommit`, compile-fail tests, archcheck v1) is not a precondition of the merges that use those handles, notably `PERS-*` (RecoveryCommit) and `WGEN-*` (SystemCommit). The validator did not notice because it only checked that G1 is isolated from later work.
**Proposed rule R-MERGE-G1:** every engineering task of PS-2…PS-14 has an implicit I edge to `G1`. Exempt: `GOV-*` (decisions), `SPK-*` (spikes), `DEV-*` (device acquisition).
**Why a rule and not four edited rows:** a minimal variant (I edge on `PROC-01`, `CORE-15`, `CORE-16` only) makes G1 an ancestor of all later *gates* but leaves 25 tasks (`PERS-01…07`, `PERS-09`, `PROC-03/04`, `API-01/03/04/05`, `CORE-12/13/14`, `BOO-01`, `SCU-01`, `CLI-00/01/04`, `QA-02`, `QA-P2`, `TEST-04`) able to merge before G1 individually. Persistence is the important case. The rule variant closes this; the cost is 102 changed rows in the generated Part C table (done by the generator, not by hand).
**Checks on the rule (all run on the real data):**

| Check | Result |
|---|---|
| cycle | none (G1's members are PS-0/PS-1 tasks; the new edges point only from PS-2+ to G1) |
| unknown references | none |
| stage inversion | none |
| every engineering task of PS-2+ has G1 as predecessor | yes after the rule |
| G1 predecessor of G2…G14 | yes |
| longest chain | grows from 21 to **25** tasks: PS-1 closing is now on the critical path, which the plan intends ("do not start the next stage without its entry criteria") |
| start eligibility changed | no |
| risk | a late `TEST-06`/`INT-01` now delays merges of PS-2+ (not starts) |

## 3. D-02: G14 does not require most stages

**Exact problem.** Predecessor gates of `G14` today: `G0, G2, G3, G4, G10, G13`. Not predecessors: `G1, G5, G6, G7, G8, G9, G11, G12`.
**Affected tasks/gates.** All G14 members (`QA-02`, `QA-04`, `QA-06`, `DG-1…DG-3`, `GOV-13`, `GOV-15`).
**Consequences.** "Full QA and Release Validation" can be accepted formally while backends (G5), tools (G6), history (G7), composite (G8), resampling (G9), world-generation integration (G11) and the client/adapter (G12) are unbuilt.

### 3.1 Which stages are required for release? (separation, not a decision)

The release scope is **the owner's decision**; the proposal therefore **does not make every stage mandatory silently**. It makes release closure the *default* and gives a mechanism for **explicit owner-approved exclusions**. The table shows what each stage traces to in the recorded requirements; "traceable" is this review's reading, to be confirmed.

| Gate | Stage | Traceable to | Reading |
|---|---|---|---|
| G1-G4 | Core, Processing, Editing API, Orchestration | transactional editing (AOD-15), Universal Geometry System | required |
| G5 | Backends, derived providers | pillar "Universal Geometry System: voxels, micro-voxels, SDF, polygon mesh" | required |
| G6 | Tools: sculpting and Boolean | pillar "modification, destruction and building"; sculpt/Boolean tools are **not named** | scope not confirmed |
| G7 | History | AOD-15 ("history semantics" of user changes) | required unless the owner narrows AOD-15 |
| G8 | Composite geometry | ACR-003; no pillar names it | scope not confirmed |
| G9 | Resampling and refinement | pillar "micro-voxels" | required |
| G10 | Persistence | AOD-21 | required |
| G11 | World generation integration | procedural endless world, AOD-20 | required |
| G12 | Client runtime and adapter | PC/Android/iOS clients, AOD-02/03 | required |
| G13 | Server runtime and networking | AOD-05/13/14/22, multiplayer | required |

### 3.2 Mechanism for explicit exclusions

* Default: `QA-04` merges after every gate G1…G13 that is not already a predecessor.
* An exclusion is a registry entry (`REL-EXCL-n`): gate id, justification, the owner's decision id, and its consequences for milestones and criteria. Only gates with such an entry may be left out of `QA-04`'s edges.
* The validator takes the exclusions as input (`--exclude Gn`; in a final form read from a table in Part D) and **fails** if a gate is neither a predecessor of G14 nor excluded. No exclusion is proposed by this review; the default list is empty.

### 3.3 What the proposal does not decide

* Whether G6 or G8 are in the release scope (section 3.1).
* A table of *required predecessor gates per stage* beyond G1 (for example whether G5 must follow G4, or G12 must follow G10 or G11). The present graph has parallel branches (verified: G5 lacks G4; G7 and G8 lack G5 and G6; G10 lacks G5-G9; G11 lacks G5-G9 and G12; G12 lacks G10 and G11; G13 lacks G11 and G12). They may be intentional (backends parallel with orchestration, client parallel with persistence). The Integration Owner should write the table and the owner should confirm it; the validator would then assert it.

## 4. Predecessor table (generated from the real tables)

Produced by `python docs/audit/adg_check.py --table`. "Current" and "required" list predecessor **gates** (task rows show S and M edges). The table is reproduced in `ADG-D01-D02-PATCH-PROPOSAL.md` section 3.

## 5. Process notes

* The ADG tables are called "generated", but the generator (`_build/data.py`, `validate.py`, `build.py`) is not in the repository (finding F-5). Until it is committed, the patch is applied to the markdown tables (the patch file does that) and regenerated parts must be kept equal by hand.
* The checker is an **audit aid**: `adg_check.py` has exit codes, but nothing runs it in CI and no required status check exists (there is no CI). It is not a gate.
* Owner decisions needed: ST-2 (accept D-01/D-02 as plan corrections) and whether any stage is optional for release (REL-EXCL entries).
