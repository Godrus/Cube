# GOV-01 Technical Recommendations

**Status:** **Proposed.** These are the agent's recommendations to the owner. They are not decisions and change no document. Facts are marked *(verified)* when checked in the repository at HEAD `64e8a76`; otherwise they come from the cited documents.

## 1. Recommendations with technical justification

| # | ID | Recommended option | Technical justification | Risk if the owner chooses otherwise |
|---|---|---|---|---|
| 1 | ACR-001 | **A** Core owns C-16…C-19 | Only Core's commit creates revisions, dirty regions and operation ids; a lower crate would add a layer with one client. Already true in the spike *(verified)* | Option B adds a crate and an archcheck edge for no dependency benefit |
| 2 | ACR-002 | **A** ports in `cube-editing-api` | Keeps Orchestration and backends mutually independent; backends compile and test without Orchestration; no cycle | Direct dependency recreates the cycle risk the QA Pack rejected |
| 3 | ACR-003 | **A** data in Core, operations in Composite | Persistence can serialize composites without depending on Composite; behaviour stays out of Core | B adds a Persistence→Composite edge; C puts operations in Core |
| 4 | ACR-004 A | **Approve** | It is the structural form of AOD-20 (Approved as recorded): system generation is not an ordinary Editing operation, and capability handles make the boundary a compile-time fact (TEST-06 trybuild) | Rejecting makes system writes ordinary undoable edits, contradicting AOD-20 |
| 5 | OAM-10 | **R-A** | Reuses the existing revision scopes and the `Brick ⇒ Layer ⇒ Asset` invariant; no new token kind; journal records only user commits. Requires OD-R6 (item 8) so that user-edit conflicts are checked by region stamps | R-B/R-C change C-16 and persisted state before WorldGen exists; premature without proven need |
| 6 | OD-R1 | **`u64` value / `u32` epoch** | Overflow is unreachable in practice; matches the spike *(verified)*; rollover would break monotonic comparison used by validation and replay | `u32` rollover invalidates snapshot isolation after wrap |
| 7 | OD-R2 | **A** immutable `Arc` state + per-asset commit gate | Keeps validate-then-apply one atomic step across bricks (transactional editing), gives lock-free immutable snapshot readers (server threads, Processing), and wraps the spike algorithm unchanged *(verified: `commit(&mut self, WriteSet)`)*; fine-grained gating can be added behind the same API | Cell-level CAS makes multi-brick atomicity and its tests much harder |
| 8 | OD-R6 | **128-bit hash over sorted (CellKey, revision) + count**, exact per-cell revisions kept in snapshots; **the owner must name the algorithm** (agent suggestion: a non-cryptographic 128-bit hash with a fixed seed and a versioned name; cryptographic strength is not required since the stamp is not a security boundary) | O(1) comparison, no false conflicts besides collisions (≈2⁻¹²⁸); needed because R-A makes asset-level tokens too coarse | Coarse stamps cause spurious conflicts; sets cost memory |
| 9 | D-R8 | **Composite `(scope u64, sequence u64)`** + separate `ActorKind`, schema version | Deterministic and ordered per actor, supports idempotent retry and journal replay; avoids trusting client-chosen ids (client is not the source of truth, AOD) | Random ids lose ordering; client ids need remapping |
| 10 | D-K | **As proposed, recorded as Provisional PC-1** | Equals what the spike was built and measured on *(verified)*; the Voxel Architecture is absent so nothing better is evidenced; RG-1 limits the risk. State the per-axis range as an explicit limit | `i64` or another edge without data is a guess in the opposite direction |
| 11 | D-T | **Approve integer transform, with the scope clause (variant C)** | Determinism across platforms is a server-authoritative and cross-play requirement; the clause removes the ambiguity with Mesh/SDF layers and avoids an archcheck rule that would forbid float mesh data | Unamended A may be unimplementable for Mesh layers; B reintroduces epsilon policies |
| 12 | D-C | **As proposed, Provisional PC-1** | Minimal slice that stores cells; quantised density keeps hashes stable; extensible attribute channel | Float density conflicts with hash stability |
| 13 | D-0 | **A** P0-alpha + PC-1 | The only option consistent with AOD-18 (contracts stay Draft until approved) and AOD-24 (baseline never substitutes for missing sources) that still lets work start | Waiting has no end date; full P0 on baseline contradicts AOD-24 |
| 14 | D-E | **A** with these specifics requested from the owner: Rust version to pin; licence allow-list (agent suggestion: permissive licences only, e.g. MIT, Apache-2.0, BSD-2/3-Clause, ISC, Zlib, Unicode; copyleft excluded in line with AOD-23); decide `panic` policy (**unwind** in crates that are loaded behind `catch_unwind`, or abort without `catch_unwind` claims) | A lower layer without an async runtime keeps loom/deterministic tests tractable; the allow-list follows the owner's licence policy | Async in lower layers couples the pure core to a runtime; undecided panic policy leaves the FFI safety claim false under `panic = "abort"` |
| 15 | branch | **A** new docs-only branch from `master` | Only option meeting GOV-04's acceptance and roadmap-v1 section 7; spike branches stay untouched | Option C collides with production crate paths and the "spike never merges" rule |

## 2. Architecture compatibility check

Legend: **OK** consistent; **Note** consistent with a remark; **Flag** needs the owner's attention. Principles: P1 universal hybrid Geometry System; P2 Core / Processing / Editing / Persistence separation; P3 one-directional dependencies; P4 server-authoritative multiplayer; P5 transactional editing; P6 immutable snapshots and revision validation; P7 runtime vs serialized storage separation.

| # | P1 | P2 | P3 | P4 | P5 | P6 | P7 | Comment |
|---|---|---|---|---|---|---|---|---|
| 1 ACR-001 | OK | OK | OK | OK | OK | OK | OK | foundation types stay at the bottom |
| 2 ACR-002 | OK | OK | OK | OK | OK | OK | OK | ports in L2 strengthen P3 |
| 3 ACR-003 | OK | Note | OK | OK | OK | OK | OK | passive data in Core, behaviour outside (P2) |
| 4 ACR-004 A | OK | OK | OK | OK | OK | OK | OK | handles make P4/P5 compile-time; Part A must not decide handle ABI |
| 5 OAM-10 R-A | OK | OK | OK | Note | OK | Note | OK | replication must filter by `ActorKind`; validation must use region stamps (item 8) |
| 6 OD-R1 | OK | OK | OK | OK | OK | OK | Note | persisted width must be a versioned constant independent of the in-memory type |
| 7 OD-R2 | OK | OK | OK | OK | OK | OK | OK | directly implements P5 and P6 |
| 8 OD-R6 | OK | OK | OK | OK | OK | OK | Note | hash algorithm becomes a persisted/wire contract (P7); name it |
| 9 D-R8 | OK | OK | OK | OK | OK | OK | Note | persisted in the journal; schema version required |
| 10 D-K | Note | OK | OK | OK | OK | OK | Note | i32 bounds the world per axis; "practically endless" holds only if the bound is accepted and stated |
| 11 D-T | **Flag** | OK | OK | OK | OK | OK | OK | unamended wording may clash with Mesh/SDF layers (P1) |
| 12 D-C | Note | OK | OK | OK | OK | OK | Note | SDF density quantisation must stay compatible with P1 backends; widths unspecified |
| 13 D-0 | OK | OK | OK | OK | OK | OK | OK | honest Provisional handling |
| 14 D-E | OK | OK | OK | OK | OK | OK | Note | `panic` policy vs FFI `catch_unwind` (spike uses `panic = "abort"`, *verified*) |
| 15 branch | OK | OK | OK | OK | OK | OK | OK | no architecture effect |

Cross-checks against ADG, module boundaries and contracts: ACR-001/002/003 are exactly the edges used by `module-boundaries.md` Part B (L2 ports; Core data); ACR-004 A corresponds to SC-01/02/05/10 which P0-alpha includes; D-K/D-C appear only in C-07…C-09 (PC-1); D-T only in C-04 (P0-alpha); OD-R1/R2/R6/D-R8 only in C-01/C-16/C-17 (P0-alpha). No recommendation requires a change to an approved AOD. Items needing an AOD interpretation: #4 (reject path contradicts AOD-20) and #13 (option C contradicts AOD-24).

## 3. Prioritisation

### 3.1 Classes

* **Blocking PS-0** — G0 includes GOV-02 (Owner-Approve P0-alpha), GOV-04/05/06 and XT-01. The decision must exist before the named G0 task can finish.
* **Blocking PS-1** — must exist before the named PS-1 task starts, but no G0 task needs it.
* **Deferred** — no ADG dependency before a later stage; the plan nevertheless lists it in GOV-01.

| # | ID | Class | Reason (task that needs it) |
|---|---|---|---|
| 1 | ACR-001 | Blocking PS-0 | GOV-02 approves C-16…C-19 |
| 4 | ACR-004 A | Blocking PS-0 | P0-alpha approval includes SC-01/02/05/10 (plan section 4.1) |
| 5 | OAM-10 | Blocking PS-0 | plan: principle decided in PS-0; C-16 semantics |
| 6 | OD-R1 | Blocking PS-0 | C-16 in P0-alpha |
| 7 | OD-R2 | Blocking PS-0 | C-01/C-16 in P0-alpha |
| 8 | OD-R6 | Blocking PS-0 | C-16 in P0-alpha |
| 9 | D-R8 | Blocking PS-0 | C-17 in P0-alpha |
| 11 | D-T | Blocking PS-0 | C-04 in P0-alpha |
| 13 | D-0 | Blocking PS-0 | defines what GOV-02 approves |
| 14 | D-E | Blocking PS-0 | GOV-05 (CI specification), GOV-06, XT-01 |
| 15 | branch | Blocking PS-0 | GOV-04 |
| 10 | D-K | Blocking PS-1 | assumption of Provisional PC-1 (C-07); needed before CORE-03; GOV-02 only has to cite it as basis |
| 12 | D-C | Blocking PS-1 | assumption of PC-1 (C-08/C-09) |
| 2 | ACR-002 | Deferred (conditional) | first needed by API-02 (PS-3); becomes Blocking PS-0 if the owner wants archcheck v0 (XT-01) to encode the L2 edges at PS-0 |
| 3 | ACR-003 | Deferred | first needed by CORE-14 / PS-8 |

**Counts: Blocking PS-0 = 11; Blocking PS-1 = 2; Deferred = 2.**

### 3.2 Caveat on classification

The plan's literal rule is that GOV-01 contains all 15 and is a G0 member. Treating #2, #3, #10, #12 as later decisions is **a change to GOV-01's scope** and therefore the owner's call. Even if kept in GOV-01, deciding #2 and #3 now is cheap and keeps the module grid and archcheck table unambiguous. Recommended: decide all 15 in GOV-01 as planned; classification above only says what can slip if the owner wants to start earlier.

## 4. Decisions that need the owner's personal choice (9) and confirm-only (6)

**Personal choice:** #5 OAM-10 (R-A/R-B/R-C), #7 OD-R2, #8 OD-R6 (name the algorithm), #10 D-K, #11 D-T (accept the scope clause?), #12 D-C, #13 D-0, #14 D-E (version, allow-list, panic policy, CI host), #15 branch option.
**Confirm the recommendation (low risk):** #1 ACR-001, #2 ACR-002, #3 ACR-003, #4 ACR-004 A, #6 OD-R1, #9 D-R8.
Plus the adjacent items X-1…X-4 of the Register (AOD countersignature, status-model adoption, owners or written deferral, CI specification approval).

## 5. What is missing as source data (not invented here)

* Voxel Architecture 1.0, Geometry Processing Specification, GDD/TDD, original AU-1.0: absent *(verified, `docs/README.md`)*.
* Hash algorithm for OD-R6; dedupe retention for D-R8; quantisation widths and attribute registry for D-C; representation of mesh/SDF samples (D-T); Rust version, licence allow-list, CI host, own-code licence (D-E).
* Benchmarks comparing brick edges (D-K).
