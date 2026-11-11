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
| 8 | OD-R6 | **Approve the abstract `RegionStamp` contract (Register item 8, option B) now; the owner chooses the algorithm and the collision policy.** Technical candidate: BLAKE3 truncated to 128 bits, policy P-3 (exact verification on equality for authoritative uses, accepted residual probability only for cache hints). Not chosen by this review | An abstract contract fixes what the system relies on (versioned, canonical, never "unchanged" across versions) without prejudging the algorithm; exact per-brick revisions are kept by snapshots, so exact verification is possible; BLAKE3 offers domain separation and strong collision behaviour but adds an external dependency in Core (D-E parameter j) | A stamp that silently reports a changed region as unchanged is a data-integrity bug in conflict detection; coarse stamps or sets cost conflicts or memory; one hash algorithm must not be assumed for stamp, content hash and persistence checksums |
| 9 | D-R8 | **Composite `(scope u64, sequence u64)`** + separate `ActorKind`, schema version | Deterministic and ordered per actor, supports idempotent retry and journal replay; avoids trusting client-chosen ids (client is not the source of truth, AOD) | Random ids lose ordering; client ids need remapping |
| 10 | D-K | **Record `CellKey` (= brick key), edge 16 and Euclidean division as Provisional PC-1 with RG-1; decide `VoxelPos` width together with the key width; state the world bound explicitly.** Between K-A (restrict range so voxels fit `i32`), K-B (`i64` voxel positions) and further options (Register item 10) the review recommends **K-B or K-D for a "practically endless" world**, and K-A only if the owner accepts a finite bound; this is the owner's scope decision | The spike multiplies `i32` brick coordinates by 16 in `i32`, so the declared `i32` key range does not fit the voxel coordinate; fixing only the key width would leave an inconsistent pair. Nothing evidences a better edge than 16 (no Voxel Architecture, no benchmark) | An `i32` world is bounded; an unstated bound contradicts the project requirement; changing widths after persistence exists is a migration |
| 11 | D-T | **Approve integer transform for coordinates and `GeometryTransform` only (classes 1-2); require a separate numeric policy for Mesh, SDF, gameplay state before those contracts are approved (Register item 11)** | Determinism across platforms is required for server-authority and cross-play; floating-point determinism is not automatic, so Mesh/SDF cannot be declared deterministic by a broad wording; rendering-only data stays free to use float | The broad wording may forbid what Mesh layers need or leave gameplay state undefined; option C reintroduces epsilon policies everywhere |
| 12 | D-C | **As proposed, Provisional PC-1** | Minimal slice that stores cells; quantised density keeps hashes stable; extensible attribute channel | Float density conflicts with hash stability |
| 13 | D-0 | **A** P0-alpha + PC-1 | The only option consistent with AOD-18 (contracts stay Draft until approved) and AOD-24 (baseline never substitutes for missing sources) that still lets work start | Waiting has no end date; full P0 on baseline contradicts AOD-24 |
| 14 | D-E | **Decide the ten parameters (Register item 14) as one policy.** Technical leanings only where a source supports them: no async runtime in Core/Processing/Backends; `unwind` for the GDExtension artifact if FFI `catch_unwind` is kept; exact toolchain pin. **No Rust version, edition, CI host, project licence or licence allow-list is recommended: they need the owner's input** | A lower layer without an async runtime keeps loom and deterministic tests tractable; `catch_unwind` is inert under `panic = "abort"` (spike profile) | An undecided panic policy leaves the FFI safety claim false; an unset licence policy leaves `cargo deny` unconfigurable |
| 15 | branch | **A** new docs-only branch from `master` | Only option meeting GOV-04's acceptance and roadmap-v1 section 7; spike branches stay untouched | Option C collides with production crate paths and the "spike never merges" rule |

## 1.1 Authority

Every row above is a recommendation of the technical preparer. Approval belongs to the Project Owner; the Integration Owner reviews and records (Register 0.2). Where the Agent Development Specification or the Task Pack say the Integration Owner "approves" contracts, that wording conflicts with AOD-18 and plan 3.1 and is listed for the owner's ruling (Register 0.2).

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
| 8 OD-R6 | OK | OK | OK | OK | OK | Note | Note | the algorithm and encoding become a wire/persisted contract (P7); a collision must not yield a false "unchanged" in authoritative paths (P6); policy needed |
| 9 D-R8 | OK | OK | OK | OK | OK | OK | Note | persisted in the journal; schema version required |
| 10 D-K | **Flag** | OK | OK | OK | OK | OK | Note | the key range and the voxel range are inconsistent in the spike; an `i32` world is bounded, which conflicts with "practically endless" unless the bound is accepted or a wider/hierarchical key is chosen |
| 11 D-T | **Flag** | OK | OK | Note | OK | OK | Note | broad wording may clash with Mesh/SDF layers (P1) and is undefined for gameplay state (P4); persisted canonical form needed (P7) |
| 12 D-C | Note | OK | OK | OK | OK | OK | Note | SDF density quantisation must stay compatible with P1 backends; widths unspecified |
| 13 D-0 | OK | OK | OK | OK | OK | OK | OK | honest Provisional handling |
| 14 D-E | OK | OK | OK | OK | OK | OK | Note | the proposal is consistent; the spike profile (`panic = "abort"`) would make FFI `catch_unwind` inert; panic strategy is per artifact |
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

**Personal choice:** #5 OAM-10 (R-A/R-B/R-C), #7 OD-R2, #8 OD-R6 (algorithm and collision policy), #10 D-K (including `VoxelPos` width and the world bound), #11 D-T (scope amendment), #12 D-C, #13 D-0, #14 D-E (parameters a-j), #15 branch option.
**Confirm the recommendation (low risk):** #1 ACR-001, #2 ACR-002, #3 ACR-003, #4 ACR-004 A, #6 OD-R1, #9 D-R8 (dedupe retention is still open).
**Not additional decisions of GOV-01:** X-1 (confirmation batch inside GOV-01's acceptance), X-2 (GOV-02), X-3 (GOV-03), X-4 (GOV-05); see Register section 3.

## 5. What is missing as source data (not invented here)

* Voxel Architecture 1.0, Geometry Processing Specification, GDD/TDD, original AU-1.0: absent *(verified, `docs/archive/README.md`)*.
* OD-R6: hash algorithm, collision policy, which uses are "authoritative". `content_hash` algorithm (CORE-11). Persistence checksum algorithm (GOV-10).
* D-R8: dedupe window/retention for idempotent retry.
* D-K: `VoxelPos` width; level direction and cross-level coordinate mapping; production voxel size (needed to state the world extent); benchmark comparing brick edges.
* D-C: quantisation widths and the attribute registry.
* D-T: representation of mesh and SDF samples; numeric policies for classes 3-5.
* D-E: Rust version and edition, CI host, project licence, licence allow-list, panic strategy per artifact, server/network runtime.
