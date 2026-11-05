# PS-0: remaining blockers and the path to closing G0

**Status:** **Proposed.** Corrective revision. State verified at source HEAD `74566cd` (2026-11-05): PS-0 open, 0 of 8 gate tasks done, nothing implemented, no registry, no CI, PS-1 not started.

## 1. G0 tasks: what each needs (dependencies from ADG Part C, verified)

| Task | Owner (role) | Start-after (S) | Merge-after (I) | State | What is missing |
|---|---|---|---|---|---|
| GOV-01 | Project owner | - | - | Ready, not done | 15 decisions in the registry with confirmation sources; plus, in its acceptance, the confirmation batch X-1 (AOD/OAM records) |
| GOV-02 | Project owner | - | GOV-01 | not started | P0-alpha approval entry; PC-1 Provisional register entry (five mandatory fields); status-model adoption **and authority split** (X-2); proposal H-3 (Provisional group for PR/E contracts) if accepted |
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
| PB-2 | Governance items outside the 15, each in its own task: X-1 confirmation of AOD/OAM records (GOV-01 acceptance), X-2 status model and authority split (GOV-02), X-3 owners or written deferral (GOV-03), X-4 CI specification approval (GOV-05). **Not** new GOV-01 decisions | owner decision | GOV-02 (X-1, X-2), GOV-05 (X-4); X-3 blocks no G0 task | project owner | registry entries |
| PB-3 | Branch option for the baseline | owner decision | GOV-04, GOV-06 | project owner | choice recorded; baseline branch/PR |
| PB-4 | Missing source data inside decisions: RegionStamp algorithm, collision policy and content-hash algorithm (OD-R6, CORE-11), `VoxelPos` width, level direction and voxel size (D-K), numeric policies for Mesh/SDF/gameplay (D-T), Rust version and edition, CI host, project licence, licence allow-list, panic strategy per artifact (D-E), dedupe window (D-R8), channel widths (D-C) | missing data | the respective G0 approvals | project owner (supply) | values written in the entry or an explicit "delegate to a later entry" |
| PB-5 | No CI exists; no GitHub-side settings can be read (`gh` not installed) | environment / permission | GOV-06, XT-01 evidence | owner (admin on GitHub), Agent I | workflow-run URL; export of branch protection or ruleset |
| PB-6 | ADG defects D-01/D-02 (`ADG-D01-D02-ANALYSIS.md`) | plan defect | correctness of later scheduling; not G0 tasks | Integration Owner, owner acceptance (ST-2) | corrected Parts C/D, extended validator |
| PB-7 | State-source conflict ST-1 | owner decision | agent handoff reliability | project owner | choice recorded; `AGENTS.md` fixed |
| PB-8 | External inputs absent: Voxel Architecture, Geometry Processing Spec, GDD/TDD | external | not G0; constrain PC-1, G2, M3+ | project owner | documents supplied (separate track) |

## 3. Findings (inconsistencies in the repository documents; none applied)

| ID | Finding | Evidence | Proposed fix (owner / Integration Owner) |
|---|---|---|---|
| F-1 | The checklist's 15 rows omit the confirmation of the recorded AOD/OAM decisions, although plan 3.3 says "decision GOV-01 covers this" | `production-architecture-plan.md` 3.3 vs `ps-0-exit-checklist.md` section 2 and the GOV-01 task row | record X-1 as a confirmation batch in GOV-01's acceptance (Register section 3); no 16th decision |
| F-2 | **Registry path inconsistency**: the plan says `docs/decisions/decision-registry.md`; the task breakdown says only `decision-registry.md`; `PROJECT_STATUS.md`, the first audit reports and `val.py` say `docs/production/decision-registry.md` (my error in the earlier package) | `grep` over `docs/` | owner confirms the plan's path; correct `PROJECT_STATUS.md` and `val.py`; for the dated audit reports add an erratum instead of editing them |
| F-3 | `ps-0-exit-checklist.md` section 5 says "READY FOR OWNER APPROVAL" while the status package uses other labels | text | harmonise (cosmetic) |
| F-4 | GOV-06's skeleton PR adds a Rust workspace to the base; GOV-04 requires a docs-only baseline first. The order is right in the ADG, but item 15 decides which base the skeleton targets | ADG, `integration-and-qa-plan.md` section 1 | decide item 15 before GOV-06 |
| F-5 | The ADG tables are called "generated"; the generator is not in the repository | plan Part C text; `ls` | commit the generator, or declare the markdown the source |
| F-6 | Gate-chain gaps D-01/D-02 | `ADG-D01-D02-ANALYSIS.md` | patch proposal with an in-memory and a patched-copy verification |
| F-7 | `integration-and-qa-plan.md` section 1 states stale counts (31 commits, 247 files) | `git rev-list --count master..HEAD`, `git diff --shortstat master HEAD` | replace the numbers by "see git" |
| F-8 | Plan tables use "OD-R8" in some places and "D-R8" in the checklist for the same decision (OperationId) | `api-contracts/README.md` C-17 vs checklist | one label in the registry |
| F-9 | If OD-R6 is approved in its abstract form (Register item 8, option B), no ADG task owns the later algorithm entry that CORE-05 needs; likewise for the `content_hash` algorithm needed by CORE-11 (a PS-1 task) | ADG Part C (no such task) | add the entries to GOV-02, or add tasks; owner decides |
| F-10 | **Authority conflict**: the Agent Development Specification (lines 69, 507, 520), the Task Pack (lines 6, 710), the audit-resolution report (211, 244) and the Geometry Editing Specification (860) give the Integration Owner approval authority over contracts; AOD-18 and plan 3.1 give it to the Project Owner. Also: chat is a sufficient change instruction in `owner-architecture-decisions-v1.md` section 4 but not a confirmation source in plan 3.3 | Register 0.2 | owner ruling inside X-2 (the Register states the rule used) |
| F-11 | **Production code against Proposed contracts (CR-10):** plan 3.1 allows Proposed contracts only for "design, review, mocks, spikes", but PS-2 (Processing, against PR-01...PR-08) and PS-3 (Editing API, against E-02...E-12) merge production code before GOV-07/GOV-09 approve those contracts; `acr-004-owner-review.md` line 62 says "no implementation starts against a Draft" | `module-boundaries.md` Part C (GOV-07 I: PROC-02, PROC-04; GOV-09 I: API-02/03/04), `implementation-roadmap.md` lines 66, 68 | owner entry registering Provisional groups at PS-2/PS-3 start, or change the ADG so merges wait (the latter conflicts with the I edges of GOV-07/GOV-09, which need the implementation) |
| F-12 | The requested branch name `architecure_roadmap` does not exist; work was done on `docs/architecture-roadmap-v1`, the only branch containing `74566cd` | `git branch -a`, `git branch --contains` | owner to confirm the branch (no branch was created or switched) |

## 4. Path to close PS-0 (ordered by the ADG, nothing started)

1. **Owner decides ST-1 and ST-2** (small, unblocks the shape of the status files and the graph). Optional but cheap.
2. **GOV-01:** the owner answers the 15 decisions and the confirmation batch X-1. Nine need a personal choice, six are confirm-only (`GOV-01-TECHNICAL-RECOMMENDATIONS.md`). Each answer becomes a registry entry with a confirmation source (owner-signed commit or owner-written file).
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
