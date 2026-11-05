# PS-0: remaining blockers and the path to closing G0

**Status:** **Proposed.** State as verified at HEAD `64e8a76` (2026-11-05): PS-0 open, 0 of 8 gate tasks done, nothing implemented, no registry, no CI.

## 1. G0 tasks: what each needs (dependencies from ADG Part C, verified)

| Task | Owner (role) | Start-after (S) | Merge-after (I) | State | What is missing |
|---|---|---|---|---|---|
| GOV-01 | Project owner | - | - | Ready, not done | 15 decisions in the registry with confirmation sources (+ X-1/X-2 if folded in) |
| GOV-02 | Project owner | - | GOV-01 | not started | P0-alpha approval entry; PC-1 Provisional register entry (five mandatory fields); status-model adoption (X-2) |
| GOV-03 | Project owner | - | - | Ready, not done | owners for WorldGen, Networking Adapter, Server Runtime, Client Runtime, Engine Adapter, Import/Export, O-1, O-2 **or a written deferral of each** |
| GOV-04 | Integration Owner | - | GOV-01 | blocked by branch option (#15) | docs-only baseline on the chosen base; acceptance: no spike file under `crates/` |
| GOV-05 | Integration Owner | GOV-01 | GOV-01 | blocked by D-E (#14) | CI specification document Owner-Approved (checks, required statuses, tool pins, pass conditions) |
| GOV-06 | Agent I | - | GOV-04, GOV-05 | blocked | skeleton PR: workspace, pinned toolchain, running fmt/clippy/test/deny/archcheck v0; workflow-run URL green on skeleton and red on seeded failures; branch protection listing the required checks |
| TEST-01 | Agent I | GOV-05 | GOV-06 | blocked | testkit/harness skeleton |
| XT-01 | Integration Owner | GOV-05 | GOV-06 | blocked | archcheck v0 with seeded-violation self-tests, registered as a required check |

Outside the gate (not required to close G0): DEV-01 devices, SPK-PERS, SPK-NET.

## 2. Remaining blockers

| ID | Blocker | Type | Blocks | Who | Evidence to close |
|---|---|---|---|---|---|
| PB-1 | 15 GOV-01 decisions unconfirmed (register `GOV-01-DECISION-REGISTER.md`) | owner decision | GOV-02, GOV-04, GOV-05 and all later | project owner | 15 registry entries with confirmation source |
| PB-2 | Adjacent decisions X-1 (countersign AOD-01…24), X-2 (status model), X-3 (owners or deferral), X-4 (CI spec approval) | owner decision | GOV-02, GOV-03, GOV-05 | project owner | registry entries |
| PB-3 | Branch option for the baseline | owner decision | GOV-04, GOV-06 | project owner | choice recorded; baseline branch/PR |
| PB-4 | Missing source data inside decisions: hash algorithm (OD-R6), Rust version, licence allow-list, `panic` policy, own-code licence, CI host (D-E), mesh/SDF representation scope (D-T), dedupe window (D-R8), channel widths (D-C) | missing data | the respective G0 approvals | project owner (supply) | values written in the entry or an explicit "delegate to a later entry" |
| PB-5 | No CI exists; no GitHub-side settings can be read (`gh` not installed) | environment / permission | GOV-06, XT-01 evidence | owner (admin on GitHub), Agent I | workflow-run URL; export of branch protection or ruleset |
| PB-6 | ADG defects D-01/D-02 (`ADG-D01-D02-ANALYSIS.md`) | plan defect | correctness of later scheduling; not G0 tasks | Integration Owner, owner acceptance (ST-2) | corrected Parts C/D, extended validator |
| PB-7 | State-source conflict ST-1 | owner decision | agent handoff reliability | project owner | choice recorded; `AGENTS.md` fixed |
| PB-8 | External inputs absent: Voxel Architecture, Geometry Processing Spec, GDD/TDD | external | not G0; constrain PC-1, G2, M3+ | project owner | documents supplied (separate track) |

## 3. Findings of this preparation (inconsistencies in the repository documents)

| ID | Finding | Evidence | Proposed fix (owner/Integration Owner; none applied) |
|---|---|---|---|
| F-1 | The checklist's 15 items omit decisions that G0 tasks need (X-1…X-4); plan 3.3 says GOV-01 covers the AOD countersignature | `production-architecture-plan.md` 3.3 vs `ps-0-exit-checklist.md` section 2 | add rows 16-19 to the checklist or fold them into GOV-01 |
| F-2 | **Registry path inconsistency.** The plan names `docs/decisions/decision-registry.md`; the task breakdown names only `decision-registry.md`; the status package I wrote earlier (`PROJECT_STATUS.md` 5.7, the audit report, `val.py` future-file list) says `docs/production/decision-registry.md` | `grep` across `docs/` | use the plan's path; correct the status package (my error) once the owner confirms the path |
| F-3 | `ps-0-exit-checklist.md` section 5 says "READY FOR OWNER APPROVAL" while the status package uses another label | text | harmonise (cosmetic) |
| F-4 | GOV-06's "skeleton PR" adds a Rust workspace to `master`; GOV-04 requires a docs-only baseline first. The order is right in the ADG (GOV-06 merge-after GOV-04) but the branch option decides which base the skeleton targets | ADG, integration plan section 1 | decide #15 before GOV-06 |
| F-5 | The ADG tables are called "generated"; the generator (`_build/data.py`, `validate.py`, `build.py`) is not in the repository | `ls`, plan Part C text | commit the generator, or declare the markdown tables the source |
| F-6 | The ADG gate-chain gaps D-01/D-02 | analysis doc | see `ADG-D01-D02-ANALYSIS.md` |
| F-7 | `integration-and-qa-plan.md` section 1 states stale counts (31 commits, 247 files; now 35 commits, 268 files) | `git rev-list`, `git diff --shortstat` | replace numbers by "see git" |
| F-8 | Plan text "OD-R8" in some contract tables vs "D-R8" in the checklist for the same decision (OperationId) | `api-contracts/README.md` C-17 vs checklist | one label in the registry |

## 4. Path to close PS-0 (ordered by the ADG, nothing started)

1. **Owner decides ST-1 and ST-2** (small, unblocks the shape of the status files and the graph). Optional but cheap.
2. **GOV-01:** the owner answers the 15 decisions (+ X-1, X-2). Nine need a personal choice, six are confirm-only (`GOV-01-TECHNICAL-RECOMMENDATIONS.md`). Each answer becomes a registry entry with a confirmation source (owner-signed commit or owner-written file).
3. In parallel after GOV-01: **GOV-02** (approve P0-alpha, register PC-1, adopt status model), **GOV-03** (owners or written deferral), **GOV-04** (baseline per the chosen option), **GOV-05** (CI specification, then owner approval).
4. After GOV-04 and GOV-05: **GOV-06** (CI skeleton PR; needs the owner to allow pushing/PR and to enable branch protection), then **TEST-01** and **XT-01**.
5. **Close G0:** every member merged and accepted; evidence: registry entries, workflow-run URL (green on skeleton, red on seeded failures), branch-protection export listing the required checks, archcheck self-tests.
6. Then record G0 in `PROJECT_STATUS.md`; PS-1 may start only after that, and, if D-01 is accepted, PS-2 merges only after G1.

**Not required to close G0:** devices, the Processing Specification, the Voxel Architecture, owners of later modules (a written deferral suffices), GDD/TDD.

## 5. Actions only the owner can take

* Provide the confirmation sources (signed commit/PR approval or owner-written entries).
* Choose the branch option and authorise push/PR/merge for the baseline and the CI skeleton.
* Enable branch protection / required checks on GitHub (or give an agent the token/`gh` access for a read-only export).
* Supply: Rust version, licence allow-list, CI host, own-code licence.
* Decide ST-1, ST-2, the registry path, and what happens to `AGENTS.md` and `.gitignore`.
