# GOV-01 Corrective Review (CR-01...CR-10)

**Status:** **Proposed.** Analysis of the technical review remarks and of what was corrected in the five GOV-01 documents. It decides nothing, approves nothing and changes no contract, ACR, AOD or OAM status.
**Source:** commit `74566cd975df656282d7844e98cea79a3329a18b`. **Branch:** `docs/architecture-roadmap-v1` (the name in the task, `architecure_roadmap`, does not exist in the repository; `74566cd` is HEAD of this branch and is contained in `origin/docs/architecture-roadmap-v1`; no branch was created or switched; finding F-12).
**Every remark below was re-checked against files**, not taken from the previous audit. Evidence = file and line, or a command whose output is reproduced in `GOV-01-VALIDATION-REPORT.md`.

## Summary

| # | Remark | Verdict | Corrected in |
|---|---|---|---|
| CR-01 | Owner authority vs Integration Owner | **Confirmed in part:** contract-approval statements conflict; module-acceptance statements do not (corrected, see `GOV-01-POST-REVIEW-CORRECTIONS.md`) | Register 0.2, F-10; X-2 |
| CR-02 | GOV-01 scope completeness; X-1...X-4 | **Confirmed in part:** X-1 has no home; X-2...X-4 are already tasks of G0 | Register 3, 3.1; Blockers |
| CR-03 | OD-R6 RegionStamp | **Confirmed:** contract under-specified; also a cell/brick terminology gap | Register 0.1, item 8 |
| CR-04 | D-T numeric policy | **Confirmed:** scope ambiguous | Register item 11 |
| CR-05 | D-K coordinates and Brick | **Confirmed:** a key-range/voxel-range inconsistency exists in the spike code | Register item 10 |
| CR-06 | D-E toolchain consistency | **Confirmed with a correction:** no internal contradiction in D-E; the spike profile conflicts | Register item 14 |
| CR-07 | ADG D-01/D-02 | **Confirmed on the real tables; fix refined** (rule, not rows; no automatic mandatory release stages) | ADG analysis, patch proposal |
| CR-08 | ADG reproducibility | **Confirmed:** `exec`/text-splitting parser | `docs/archive/audit/adg.py`, `adg_check.py`, refactored tools |
| CR-09 | ST-1 | **Confirmed with details** | ST-1 document |
| CR-10 | P1 scope and dependencies | **Partly confirmed:** the ADG already separates PR and E approvals; a different contradiction was found (production code against Proposed contracts) | Register 3.2 H-3, Blockers F-11, section CR-10 below |

## CR-01: Owner authority vs Integration Owner

**Evidence.**
* Project Owner decides: AOD-18 ("contracts stay Draft until the owner separately approves them", `owner-architecture-decisions-v1.md` line 28); plan 3.1 (`Owner-Approved`: "project owner only"); `integration-and-qa-plan.md` section 2 (the Integration Owner "does not approve contracts or ACRs").
* Older documents that attribute **contract approval** to the Integration Owner: `specifications/cube-agent-development-specification-v1.0.md` line 69 ("Only the Integration Owner may approve changes to shared public contracts"; partial, it concerns *changes*); `agent-tasks/geometry-editing-agent-task-pack-v1.0.md` lines 6 and 710 (direct); `decisions/audit-resolution-report.md` lines 211 and 244 (direct, historical). `api-contracts/README.md` line 3 ("recorded") is ambiguous but compatible.
* **Not conflicts:** Agent Development Specification lines 507 and 520 and Geometry Editing Specification line 860 concern **module acceptance**, which the plan keeps with the Integration Owner (the first version of this review listed them as conflicting; corrected after the post-review).
* A second inconsistency: change control of AODs accepts the owner's chat approval (`owner-architecture-decisions-v1.md` section 4); registry confirmation (plan 3.3) does not.
**Not rewritten silently.** The Register 0.2 states the rule used in this package, lists the conflicting passages with lines, and proposes the owner's ruling (fold into X-2): either declare that for contracts "approve" means the Project Owner and the Integration Owner's role is review, conformance, recording and engineering acceptance, naming the documents to amend; or delegate named classes of approval explicitly. **The Integration Owner is given no approval power anywhere in the package** (`gov01_check.py` check).
**Residual:** the ruling is the owner's.

## CR-02: GOV-01 scope completeness

**Verified structure.** The GOV-01 task row lists 15 decisions; the checklist has 15 rows; they match (`gov01_check.py`). X-1...X-4:
* **X-1** AOD/OAM confirmation: plan 3.3 assigns it to GOV-01, but the task row and the checklist omit it. Treated as a **confirmation batch in GOV-01's acceptance**, not a 16th decision. Minimal set that other items rest on: AOD-20 (items 4, 5), AOD-18 and AOD-24 (item 13).
* **X-2** status model: it is in **GOV-02** (task row text "adopt the Contract Status Model"; blocker doc section 4 maps D-REG to GOV-01/GOV-02). It now also carries the authority split (CR-01).
* **X-3** owners: **GOV-03**, an independent G0 task (S and I empty, verified); written deferral per module is explicitly allowed by the task row.
* **X-4** CI specification: **GOV-05** (S and I = GOV-01); a G0 member.
**Blocking relations** (Register 3.1): GOV-02 needs items 1, 4, 5, 6, 7, 8, 9, 11, 13 (and 10, 12 as the PC-1 basis) and X-1/X-2; GOV-03 none; GOV-04 item 15; GOV-05 items 14, 15; GOV-06 the outputs of GOV-04/05. **ADG gating is coarser:** GOV-02/04/05 wait for the whole of GOV-01. A two-batch answer inside one GOV-01 task is possible without changing the ADG.
**Written deferral:** explicit only for owners (X-3). No source allows deferring any of the 15; ACR-002/003 could slip to PS-3/PS-8 only by amending GOV-01's scope (the owner's call). The count of 15 is unchanged.

## CR-03: OD-R6 RegionStamp

**Evidence.** The contract text defines `RegionStamp` as the set of `(BrickKey, BrickRevision)` (`revision-and-snapshot-semantics.md`); the blocker text and the first Register said "(CellKey, revision)" and "per-cell revisions". `production-architecture-plan.md` 5.3 titles `CellKey (BrickKey)`, so "cell" means brick; there is no per-voxel revision. The glossary (Register 0.1) removes the ambiguity.
**Added:** a requirement table (stamped set, algorithm, digest size, canonical encoding with 33 bytes per entry, sort order, cardinality, domain separation, versioning, empty region, collision consequence, exact verification, cost) and a collision-risk policy (P-1 exact-on-equal, P-2 explicit residual risk, P-3 hybrid). BLAKE3-128 is a **candidate, not chosen**; the standard-library default hasher is excluded because its algorithm is unspecified. A hash collision never yields a silent "unchanged" in authoritative paths unless the owner accepts a stated policy. Other hash roles are separated: content hash (CORE-11, algorithm also unnamed, H-1), persistence checksums (GOV-10, H-2), `OperationId` (not a hash).
**Blocking consequence kept explicit:** no algorithm entry means C-16 cannot be approved fully; option B (approve the abstract contract, name the algorithm later) unblocks GOV-02 but blocks CORE-05, and the ADG has no task for the later entry (F-9). Not measured: cost on target devices (benchmark required).

## CR-04: D-T numeric policy

**Evidence.** D-T text "no floating point in authoritative or persisted state"; archcheck rule wording "authoritative or world-defining code"; cube-worldgen forbidden scope "floats in world math"; closed `LayerData::{Voxel, Sdf, Mesh}`; spike `voxel_size_m: f32`; the spike generator has zero `f32`/`f64`.
**Added:** the seven data classes (coordinates, `GeometryTransform`, mesh vertices, SDF/density samples, authoritative gameplay state, persisted representation, rendering-only data) with what D-T covers; the requirement of a separate numeric policy (precision, canonicalization, serialization, cross-platform determinism, comparison/tolerance, conversion/resampling) for classes 3-5; explicit statement that Mesh/SDF determinism is **not** assumed. No numeric parameter is proposed. The recommendation becomes "integer transform for classes 1-2 with a scope amendment"; the owner chooses.

## CR-05: D-K coordinates and Brick

**Evidence (verified in code).** `crates/cube-core/src/coord.rs`: `BRICK_SIZE: i32 = 16`, `div_euclid`/`rem_euclid`, and a voxel-from-brick conversion `x * BRICK_SIZE + local` computed in `i32`. Therefore a brick coordinate beyond about 1.3e8 overflows the voxel coordinate; the declared key range (`i32` bricks) and the voxel range (`i32` voxels) are inconsistent in the spike. `VoxelPos` width is not specified in any source; level direction and the mapping between levels are unspecified (C-07 "refinement addressing" open).
**Added:** coordinate spaces, floor-division rule, level gaps, range analysis, serialization implication, an explicit statement that an `i32` world is **bounded** and that "practically endless" is only defensible with a stated bound or a wider/hierarchical key (options K-A...K-D), with an assumed-size illustration clearly labelled as such. PC-1 stays **Provisional** with RG-1; a benchmark is the review's proposal, not a sourced requirement (the sources require the RG-1 diff).

## CR-06: D-E toolchain consistency

**Evidence.** D-E proposes `catch_unwind` at the FFI **and** "test `panic = abort` before adopting it": consistent. The conflict is with the spike's `[profile.release] panic = "abort"` (root `Cargo.toml`), under which `catch_unwind` cannot catch anything; godot-core 0.5.5 itself calls `std::panic::catch_unwind` (`src/init/mod.rs`, verified in the local registry source). ADP-03 (PS-12) is the task that decides and tests the panic policy.
**Corrected:** the first Register called this an internal tension of the proposal; it is between the proposal and the spike profile. **Added:** a table of ten parameters (toolchain version, edition, CI host, project licence, dependency-licence allow-list, panic strategy per artifact, FFI panic boundary, async/runtime policy, development dependencies, dependencies in Core) with sources, options and consequences. **Removed:** the earlier agent suggestion of specific licences; no licence policy and no Rust version are proposed. Deferral of f/g to ADP-03 is possible only if no earlier artifact sets `panic`.

## CR-07: ADG D-01 and D-02 on the real source

* Source: the tables in `docs/production/module-boundaries.md` (unchanged since `2c0c1f3`).
* D-01 reproduced: 102 engineering tasks of PS-2+ lack `G1`; `G1` is a predecessor of no gate. Start and merge eligibility separated: the proposal adds I edges only.
* The minimal variant (4 rows) leaves 25 tasks able to merge before G1; the **rule R-MERGE-G1** closes this. No cycle; critical path 21 -> 25 tasks.
* D-02 reproduced: `G1, G5, G6, G7, G8, G9, G11, G12` are not predecessors of `G14`. **Refinement:** release closure is the *default*, but the proposal does not declare every stage mandatory: a traceability table separates stages required by recorded requirements from G6 and G8 (scope not confirmed), and an exclusion mechanism (`REL-EXCL-n`, owner decision, validator input) is defined. No exclusion is proposed.
* The patch is a real unified diff, checked with `git apply --check` on the real tree (nothing applied) and applied to a **copy**, whose graph then passes all checks. Table "Task/Gate | Current | Required | Reason | Proposed change": `ADG-D01-D02-PATCH-PROPOSAL.md` section 3.

## CR-08: Reproducibility

**Found:** `ia.py`/`gen.py`/`val.py`/`adg-fix-simulation.py` used `exec(open(...).read().split("print('tasks'")[0])` (fragile text splitting, breaks when a print line moves) and were not importable.
**Done:** `docs/archive/audit/adg.py` (importable: `load`, `Graph`, `run_checks`, `apply_g1_merge_rule`, `apply_release_closure`, `render_report`; explicit `AdgFormatError` when the markdown layout changes), `docs/archive/audit/adg_check.py` (CLI: structure/policy checks, `--proposed`, `--strict`, `--exclude`, `--table`, `--determinism`), the four older scripts reduced to thin wrappers without `exec`, and `docs/archive/audit/gov01_check.py`. **Not a CI gate:** the repository has no CI; exit codes prove nothing about required checks.
**Checks present:** no cycles; references exist; stage monotonicity; gate membership; predecessor closure (G1); G1 merge policy; G14 release closure with exclusions; exclusions valid; deterministic output. **Pending:** a per-stage required-predecessor table (does not exist yet).

## CR-09: ST-1

All paths checked (ST-1 document section 3 with an "exists now" column): `docs/decisions/decision-registry.md` is **absent** (to be created); the other named files exist. `AGENTS.md`: exists, untracked, ignored by `.gitignore` line 6; it references the absent `.agent/`. **Not created, not edited, `.gitignore` not changed.** The earlier documents that name `docs/production/decision-registry.md` are listed for correction after the owner confirms the path; dated audit reports are treated as immutable (erratum, not edit).

## CR-10: P1 scope and dependency precision

**Facts (verified).**
* Contract tiers: `api-contracts/README.md` row P1 = PR-01...PR-08 + E-02...E-07, E-09, E-11, E-12 (17 contracts) in one tier for "Stage 2 (Processing), Stages 3-5". The QA Pack (lines 160-161, 173) gates stage 2 on "P1 approved".
* The v2.1 ADG **already separates** them: `GOV-07` approves PR-01...PR-08 (PS-2, after EXT-GPS), `GOV-09` approves E-02...E-12 (PS-3). No Processing task has `GOV-07` or `GOV-09` as an ancestor (so Processing is not blocked by Editing contracts); `API-02` has `GOV-07` and `G2` as ancestors.
* Dependency direction: `editing-contracts.md` line 9 "Layer 2. Depends on Core and Processing" and uses Processing outcomes (`ProcessingCancelled`, `ProcessingFailed`); `processing-contracts.md` contains no `E-xx` reference (verified). So P1-Processing before P1-Editing is the consistent order.
**Consumer table (from `api-contracts/README.md`):** PR-01 -> Orchestration, backends, Boolean, Persistence; PR-02 -> Processing, Orchestration; PR-03 -> Orchestration, Tools, Persistence; PR-04 -> Orchestration; PR-05 -> all async; PR-06 -> submitters; PR-07 -> derived-data owners; PR-08 -> queries, collision. E-02 -> Orchestration, Networking; E-03 -> Orchestration, Tools, History; E-04 -> Orchestration, Tools, backends, History; E-05 -> all; E-06 -> Orchestration, Networking; E-07 -> Orchestration, backends, Tools; E-09 -> History, Persistence; E-11 -> Tools, Composite, History, Networking; E-12 -> Orchestration, History. No PR contract has a consumer among the Editing contracts' approval tasks, and no Processing task consumes an E-contract.
**Conclusion.** A formal **split of the tier into P1-Processing and P1-Editing is justified as a labelling change** (it matches the existing GOV-07/GOV-09 and removes the stale single-tier gating in README/QA Pack/Task Pack), but **it changes no ADG edge and no stage scope**; it needs the owner's acceptance because it amends contract-tier documents. It is **not** introduced here.
**A different, real problem found (F-11, H-3).** Plan 3.1 permits a Proposed contract only for "design, review, mocks, spikes" ("no production module may claim it"), and `acr-004-owner-review.md` line 62 says "no implementation starts against a Draft". But PS-2 (Processing, against PR-01...PR-08, `implementation-roadmap.md` line 66 "Proposed") and PS-3 (Editing API, against E-02...E-12) merge **production** code while the contracts are still Proposed, because `GOV-07` is merge-after `PROC-02`/`PROC-04` and `GOV-09` is merge-after `API-02/03/04`. Options (owner): (1) register **Provisional** groups at the start of PS-2 and PS-3 (the plan's mechanism for P0 and PC-1; each needs the five mandatory fields and an RG), keeping the ADG; (2) change the ADG so production merges wait for approval, which needs GOV-07/GOV-09 to stop depending on the implementation (a different design); (3) treat PS-2/PS-3 implementation as spikes until approval (contradicts the "merge" semantics). Recommendation: (1). The stage scope is unchanged by the finding.

## Not done on purpose

No production code; no PS-1 start; no contract, ACR, AOD, OAM or Provisional status changed; no ADG source change; no new governance decision made on the owner's behalf; no commit, push, PR; `.gitignore` and `AGENTS.md` untouched; no branch created.
