# Decisions only the Project Owner can make

**Status:** **Proposed.** A compact list. Nothing here is decided. Details: [`GOV-01-DECISION-REGISTER.md`](GOV-01-DECISION-REGISTER.md) (item numbers), [`GOV-01-CORRECTIVE-REVIEW.md`](GOV-01-CORRECTIVE-REVIEW.md). An answer becomes effective only as a registry entry with a confirmation source (plan 3.3).

## A. The 15 GOV-01 decisions

Choose (C) or confirm (OK) the recommendation. "Needs data" means a value that no source supplies.

| # | ID | Your choice is between | Needs data from you |
|---|---|---|---|
| 1 | ACR-001 | Core owns revision/operation/dirty/error types (rec.) / `cube-foundation` crate | OK |
| 2 | ACR-002 | ports in `cube-editing-api` (rec.) / direct dependency | OK |
| 3 | ACR-003 | data in Core, operations in Composite (rec.) / all in Composite / all in Core | OK |
| 4 | ACR-004 A | approve (rec.) / amend / reject (needs a change of AOD-20) | OK |
| 5 | OAM-10 | R-A (rec.) / R-B / R-C (write `OAM-10/R-A` in the registry) | C |
| 6 | OD-R1 | `u64` value + `u32` epoch (rec.) / `u32` rollover | OK |
| 7 | OD-R2 | immutable state + commit gate (rec.) / cell CAS / keep single-writer | C |
| 8 | OD-R6 | approve now: A name algorithm, encoding, policy / B abstract contract, algorithm later. Collision policy P-1 / P-2 / P-3 (rec.) | algorithm (candidate BLAKE3-128, or another); which uses are "authoritative" |
| 9 | D-R8 | composite `(scope, sequence)` (rec.) / random 128-bit / client-supplied | dedupe retention |
| 10 | D-K | K-A restrict range / K-B `i64` voxel positions / K-C both `i64` / K-D hierarchical key; edge 16 as Provisional PC-1 | `VoxelPos` width, level direction, voxel size, whether the world bound is acceptable |
| 11 | D-T | integer transform for classes 1-2 with scope amendment (rec.) / broad wording / `f64` + epsilon | numeric policies for Mesh, SDF, gameplay state (later) |
| 12 | D-C | built-in channels with quantised density (rec.) / fixed layout / float density | quantisation widths, attribute registry |
| 13 | D-0 | P0-alpha + PC-1 (rec.) / wait for sources / full P0 on baseline (contradicts AOD-24) | C |
| 14 | D-E | ten parameters a-j | Rust version, edition, CI host, project licence, licence allow-list, panic strategy per artifact, FFI boundary, server runtime, dev-dependencies, Core dependency policy |
| 15 | branch | new docs-only branch (rec.) / path-limited branches / accept spike code | branch name; permission to push and open PRs |

## B. Governance items that are not additional GOV-01 decisions

| ID | Where | Question |
|---|---|---|
| X-1 | GOV-01 acceptance | Countersign AOD-01...24 and OAM-01/02/03/07/08 (at least AOD-18, 20, 24 now), by signed commit or an owner-written file? |
| X-2 | GOV-02 | Adopt the status model (Proposed, Provisional, Owner-Approved, Implemented, Frozen, Rejected, Superseded) **and** rule that contract approval belongs to the Project Owner (the Task Pack lines 6 and 710 and the audit-resolution report lines 211 and 244 attribute contract approval to the Integration Owner, Agent Specification line 69 does so partially; module acceptance by the Integration Owner is not affected; which documents are amended?) |
| X-3 | GOV-03 | Owners for WorldGen, Networking Adapter, Server Runtime, Client Runtime, Engine Adapter, Import/Export, O-1, O-2, or a written deferral of each |
| X-4 | GOV-05 | Approve the CI specification (after item 14) |

## C. Questions raised by the corrective review

| ID | Question | Recommendation |
|---|---|---|
| ST-1 | Canonical state model and [`AGENTS.md`](../../../AGENTS.md): A (PROJECT_STATUS canonical; a short tracked [`AGENTS.md`](../../../AGENTS.md)), B (`.agent/` canonical), C (status only, [`AGENTS.md`](../../../AGENTS.md) untracked) | A; you edit [`AGENTS.md`](../../../AGENTS.md)/`.gitignore` yourself or authorise it |
| ST-2 | Accept the D-01/D-02 corrections (rule R-MERGE-G1 + release closure) | accept after your review of the patch |
| REL-EXCL | Is any stage optional for release? G6 (sculpt/Boolean tools) and G8 (composite) are not traceable to a recorded requirement | none proposed; default is closure of all stages |
| H-1 | Hash algorithm of `content_hash` (CORE-11, PS-1) | with item 8 |
| H-3 | Register Provisional groups for PR-01...PR-08 (PS-2) and E-02...E-12 (PS-3), because production code merges before their approval | yes (option 1) |
| P1 | Rename the tier P1 into P1-Processing and P1-Editing in the contract documents | label change only, no ADG change |
| REG | Registry path: [`docs/decisions/decision-registry.md`](../../decisions/decision-registry.md) (plan 3.3) | yes |
| BR | Which branch is the target: `architecure_roadmap` does not exist; work was done on `docs/architecture-roadmap-v1` | confirm |
| ORD | May GOV-01 be answered in two batches (14-15 first)? | yes, optional |
