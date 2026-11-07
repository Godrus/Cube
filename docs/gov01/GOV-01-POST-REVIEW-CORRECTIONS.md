# GOV-01: corrections after the Owner Decision Review

**Status:** **Proposed.** Records the corrections made to the committed GOV-01 package (`5c9a5c5`) after the Owner Decision Review. Documentation and audit tooling only. No decision, contract, ACR, AOD, OAM or Provisional status changed; the ADG and its patch were not modified; nothing was committed or pushed.
**Terminology used here (precise):** **baseline commit** = the commit a validation started from; **package commit** = the commit that contains the package; **HEAD** = the commit currently checked out. At the time of these corrections HEAD and the package commit are both `5c9a5c5`; the baseline commit of the earlier validation was `74566cd`.

## A1. OAM identifier of the renderer decision

* **Finding (confirmed):** `docs/decisions/owner-approval-matrix-v1.md` has `## 4. OAM-04 Renderer` (line 179) with options R-A `faces`, R-B `greedy`, R-C hybrid (lines 185-188), and `## 5. OAM-05 Network layer` (line 200). The Register said the renderer was "OAM-05".
* **Correction:** `docs/gov01/GOV-01-DECISION-REGISTER.md`, item 5: "OAM-05" -> "OAM-04". Meaning and status of the decision are untouched (OAM-04 stays Proposed; the registry label for the revision model remains `OAM-10/R-A`).
* **Search:** `grep OAM-05 docs/gov01` finds no other misuse. The Game Track documents use OAM-04 for the renderer and OAM-05 for the network layer.

## A2. Authority conflict: module acceptance versus contract approval

* **Finding (confirmed against the source text):**
  * Agent Development Specification line 507: "Only Integration Owner can mark a module Accepted." (status list of a module); line 520: Definition of Done item "Integration Owner approves acceptance." Geometry Editing Specification line 860: item 12 of the conditions under which "A module is accepted". All three concern **module acceptance**.
  * Only these statements concern **contract approval**: Agent Specification line 69 ("Only the Integration Owner may approve changes to shared public contracts."; it concerns *changes*); Task Pack line 6 ("Contract authority: Integration Owner") and line 710 ("Integration Owner approves contracts by tier"); audit-resolution report lines 211 and 244 ("Integration Owner approval", "approve P0, then P1 contracts; set status to Approved").
  * The Agent Specification never mentions a Project Owner, so it neither grants nor denies owner authority; it predates the owner records.
* **Correction:** `GOV-01-DECISION-REGISTER.md` section 0.2 now has four separate kinds of authority (Project Owner authority; contract approval authority; module acceptance authority; Integration Owner responsibilities) and a table of **only** the supported conflicts with a strength column (partial: Agent Specification 69; direct: Task Pack 6 and 710; direct and historical: audit-resolution 211 and 244). Lines 507, 520 and 860 are listed as *not conflicts*. The ambiguous README line 3 is listed as compatible-but-ambiguous. A possible overlap ("release acceptance" in Agent Specification 3.1 versus the owner's acceptance of milestones) is marked *not asserted*.
* **Also corrected:** `GOV-01-CORRECTIVE-REVIEW.md` (summary row and CR-01 evidence), `PS-0-REMAINING-BLOCKERS.md` finding F-10, `OWNER-DECISION-REQUIRED.md` question X-2.
* **Unchanged:** the rule used in the package (Project Owner approves; Integration Owner prepares, reviews, validates, integrates, records, gives module acceptance) and the owner's pending ruling inside X-2.

## A3. Validation report baseline

* **Finding (confirmed):** `GOV-01-VALIDATION-REPORT.md` called `74566cd` "HEAD at the time of the checks". `74566cd` was the **baseline commit** from which the checks started; the corrective package was validated as uncommitted working-tree changes and committed afterwards as `5c9a5c5`.
* **Correction:** section 1 of the report now states the baseline commit, the package commit and the HEAD separately, and the printed `git rev-parse HEAD` is labelled as the baseline.

## A4. Patch-check portability (LF / CRLF, `core.autocrlf`)

* **Finding (reproduced):** the committed check ran `git apply --check` on the working tree. It passed with `core.autocrlf=true` and failed with `core.autocrlf=false`, because the Windows checkout holds CRLF files and the patch holds LF lines.
* **Cause:** no `.gitattributes`; line endings depend on each machine's Git configuration.
* **Correction in `docs/audit/gov01_check.py` (tool only; the patch and the graph are untouched):**
  1. every `git` call carries explicit `-c core.autocrlf=true -c core.safecrlf=false`, so the repository-state checks no longer depend on the global setting;
  2. the patch check copies the baseline and the patch into a temporary directory, writes **both with the same explicit ending, once LF and once CRLF**, and runs `git apply --check` and then `git apply` under **both** `core.autocrlf=false` and `core.autocrlf=true`; it loads the patched graph and requires all graph checks to pass and no cycle; it requires the normalised results of the two variants to be **identical**; it requires that the working-tree baseline equals the committed blob after line-ending normalisation; it verifies that the repository copy was not modified (hash before and after);
  3. `--ignore-whitespace` is **not** used, so the validation is not weakened.
* **Test (run):** `python docs/audit/gov01_check.py` -> 29 checks, 0 failed; with a global setting of `core.autocrlf=true` -> 29/0; with `core.autocrlf=false` -> 29/0 (the setting was injected through `GIT_CONFIG_COUNT`/`GIT_CONFIG_KEY_0`/`GIT_CONFIG_VALUE_0`). For comparison the raw command `git -c core.autocrlf=false apply --check --unidiff-zero docs/gov01/ADG-D01-D02.proposed.patch` still fails on the Windows tree, which is why the checker no longer uses it.
* **`.gitattributes`:** not created. It is **not needed** for the checker. It would help people who apply the patch directly with `git apply`; if wanted, the proposal is a file with the line `*.patch text eol=lf` (optionally `*.md text eol=lf`). It changes how every checkout writes those files, so **approval is requested before it is added**.

## A5. Source-level verification of AOD-01...19: limitation

The previous review did not verify AOD-01...19 against primary sources and this correction does not change that. The facts:

* The **primary source** of AOD-01...19 is the owner's chat instruction of 2026-11-04 (`owner-architecture-decisions-v1.md` header); it is not in the repository and was not available to the preparer. The repository text is an agent transcription ("recorded, awaiting owner confirmation", plan 3.3). AOD-20...24 are recorded the same way (2026-11-05).
* The only independent repository text that overlaps is `docs/decisions/owner-decision-rust-godot.md` (itself "recorded from the owner's instruction (chat, 2026-11-02)").

| AOD | Subject | Cross-check made now | Result | Verified against a primary source? |
|---|---|---|---|---|
| 01 | Rust | `owner-decision-rust-godot.md` decision table | consistent | **No** (that file is also a chat transcription) |
| 02 | Godot 4.x first client | same | consistent | No |
| 03 | GDExtension only | same (binding: GDExtension); the "no sidecar process" consequence is not in that file | partial | No |
| 04 | Core pure Rust, no Godot | same, constraints 1-2 | consistent | No |
| 05 | Separate headless server process | not in that file | not cross-checked | No |
| 06 | Godot replaceable | same, constraint 4 | partial | No |
| 07 | GDScript UI and glue only | same | consistent | No |
| 08 | Android and iOS mandatory | same lists the target platforms; "mandatory" wording absent | partial | No |
| 09 | WorldGen deterministic, no floats | no source; implementation evidence: spike generator contains no `f32`/`f64` | implementation evidence only | No |
| 10 | `faces` reference, `greedy` experimental | consistent with OAM-04 (Proposed) | consistent, status nuance | No |
| 11 | Emulator never replaces a device | no source; mobile-validation report wording | not cross-checked | No |
| 12 | No `OS.execute` | no source; guard test `no_child_processes.rs` exists | implementation evidence only | No |
| 13, 14 | Server authority, client not truth | `system-boundaries-v1.md` (Proposed) | consistent | No |
| 15 | User changes through the Editing API | none | not cross-checked | No |
| 16 | WorldGen is a separate boundary | `worldgen-core-boundary.md` (analysis, Proposed) | consistent | No |
| 17 | Persistence seed + journal | consistent with AOD-21 | consistent | No |
| 18 | Contracts stay Draft | `api-contracts/README.md` states all Draft | consistent | No |
| 19 | AU-1.0 and ACR not rewritten | AU-1.0 and ACR text unchanged | consistent | No |

**Therefore:** no statement in this package or in the Game Track documents may present AOD-01...24 as audited or confirmed. They are used as **[REC] constraints** (recorded, awaiting countersignature, X-1). Items that need source-level verification before any contract approval: all of AOD-01...19 (the owner's chat record or the owner's own confirmation), every statement that cites the original AU-1.0 (including "section 23" and "section 2"), and every statement about the absent GDD/TDD, Voxel Architecture and Geometry Processing Specification.

## Files changed by this correction pass (working tree, uncommitted)

* `docs/gov01/GOV-01-DECISION-REGISTER.md`, `GOV-01-CORRECTIVE-REVIEW.md`, `GOV-01-VALIDATION-REPORT.md`, `OWNER-DECISION-REQUIRED.md`, `PS-0-REMAINING-BLOCKERS.md` (wording corrections A1-A3);
* `docs/audit/gov01_check.py` (A4);
* new: this document.
