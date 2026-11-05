# Decision Record: WorldGen / Geometry Core Boundary

**Status:** The direction (option C, variant C3) is **Approved** by AOD-20 (2026-11-05, `owner-architecture-decisions-v1.md` section 5.1); everything below that AOD-20 lists as "not approved" remains **Proposed** or **Spike Required**. It follows AOD-15, AOD-16, AOD-17 and AOD-21 and does not change any contract, ACR-001/002/003, RevisionToken rule or AU-1.0 text; ACR-004 is prepared as **Pending** in `audit-resolution-report.md`. No code is added. Supersedes nothing; extends the spike analysis in `docs/spike/rust-world-generation/worldgen-core-boundary.md` (options A and B only) with option C and a decision frame.

## 1. Question

How does procedurally generated terrain enter the authoritative geometry, how do user changes relate to it, what is persisted, and who owns the truth in a client/server game?

## 2. Evidence from the spike (experimental, not approved)

| Fact | Source |
|---|---|
| Generation is cheap: 0.008 ms per chunk (desktop release), 0.009 ms with the Core commit; 3x3 area 0.11 ms | `docs/spike/rust-world-generation/report.md` section 3 |
| Generation is a pure function of `(seed, coordinate)`, integer-only, order independent; pinned hashes detect changes; neighbours agree on borders | tests `world.rs`, `determinism.rs`; hash comparison across physical devices **not done** |
| The spike writes generated chunks and user edits straight through the Core commit primitive, outside the Editing API | `cube-worldgen/src/streamer.rs` |
| Consequences seen: one asset revision per streamed chunk (load or unload); no undo, validation, permission or observer for these writes; edits are kept in a side list and re-applied after regeneration (`world-state-v0`) | report sections 5 and 28 |
| An edit changes revisions the same way a streamed chunk does, so a client cannot tell "terrain streamed" from "the user changed something" | design note in the spike report |
| Regeneration after a generator change alters saved worlds: the terrain function and layout are part of the save contract | `GENERATOR_VERSION`, README "Not production-ready" |

### External evidence (upstream Veloren, `docs/research/veloren-notes.md`; not proof for Cube)

* Veloren keeps the generated world out of its save: only **block-level overrides** of modified chunks are persisted (per chunk, versioned files with fallback readers) and they are **re-applied over regenerated terrain**, dropping any override equal to the generated block. That is variant **C1** (seed plus block-level overrides) in practice; Cube's approved model (C3 with the AOD-21 journal) keeps an operation journal as the durable record instead.
* Its writes happen only when a chunk is unloaded and the source acknowledges this is not durable across power loss; Cube's journal-first design (AOD-17) avoids that gap by construction.
* Terrain generation takes a seed and a cancellation callback, runs on a dedicated job pool, and its dynamic (entity) output uses non-deterministic randomness while terrain follows the seed.
* Single-player runs the server in-process (a `singleplayer` feature), which supports the embedded-server proposal in section 7.

## 3. Requirements (from the approved decisions)

R1 server authoritative (AOD-13); R2 client never authoritative (AOD-14); R3 user changes through the Editing API with operation ids, validation and history (AOD-15); R4 generation is a separate boundary and not a user edit (AOD-16); R5 untouched world stored as seed + generator version, user changes as versioned deltas (AOD-17); R6 pure deterministic Rust generation (AOD-09); R7 mobile: cheap streaming, bounded memory, no long main-thread work (AOD-08); R8 no Godot in the boundary (AOD-04); R9 existing P0/P1 stay Draft (AOD-18).

## 4. Options

### Option A: WorldGen writes directly into Core

WorldGen holds a write handle to the Core asset and commits generated bricks (what the spike does).

* **For:** simplest; no new interface; fastest to build; already measured cheap.
* **Against:** violates R3/R4 in spirit (generation and user writes are indistinguishable at the write port); bypasses validation and observers; every streamed chunk advances revisions; generation is part of history/undo unless special-cased; persistence has no principled "what is generated vs what is user data" line; cannot express "this change is from the world, not from a player"; couples WorldGen to Core internals (brick addressing, commit scopes); server authority is only implicit (whoever holds the handle).
* **Verdict:** acceptable for a spike only (already stated), not for production.

### Option B: WorldGen is an operation through the Editing API

Generation becomes ordinary editing operations ("fill region with generated content") submitted by a system actor.

* **For:** one write path; validation, operation ids, observers, revision rules and persistence apply uniformly; no special Core entry point.
* **Against:** every streamed chunk becomes an operation: history, journal and replication traffic explode unless every consumer special-cases "system" operations (then the special case is the separate boundary again, hidden in flags); per-operation overhead sits on the hot streaming path (generation costs about 0.01 ms, operation handling could dominate); persistence cannot store "untouched = seed" cleanly because generated content is recorded as operations; undo of generation is meaningless; it contradicts AOD-16 directly (generation must not be an ordinary edit); the Editing API (P1) would have to define bulk system operations before it is approved.
* **Verdict:** rejected by AOD-16 as a default; usable only for explicit "world-editing" tools (for example a designer stamping a generated structure), which are user/designer operations anyway.

### Option C: Hybrid (Approved by AOD-20, realised as variant C3)

* First generation of untouched terrain goes through a dedicated, system-owned **WorldGen boundary** (read-only generator plus a materialiser that is the only non-Editing writer of generated data).
* User changes go through the **Editing API** and are recorded in a **mutation journal** (versioned deltas).
* Persistence stores **seed + generator version + journal** (AOD-17).
* The **server** owns the authoritative state: it generates, applies edits, persists, and replicates (AOD-13/14).

### Notation for the variants of option C (used consistently in all documents)

| Name | Meaning | Status |
|---|---|---|
| **C1** | seed plus block-level overrides (per-chunk map of changed blocks re-applied over regenerated terrain) | a data-model variant; **not chosen** as Cube's persistence model |
| **C2** | generated base plus a mutable overlay (two Core layers with defined precedence) | a data-model variant; remains an open Core design option (`Spike Required`) |
| **C3** | hybrid system generation (WorldGen/System-commit boundary), Editing API for user changes, and a separate persistence strategy (AOD-21) | **Approved** (AOD-20) |

C3 is the architecture; C1 and C2 describe how generated and user data may be organised inside it.

## 5. Comparison

| Criterion | A: direct | B: via Editing API | C: hybrid |
|---|---|---|---|
| Matches AOD-15/16/17 | no | partly (15 yes, 16 no, 17 awkward) | yes |
| Separation of generated vs user data | none | by flag only | structural |
| Undo/history pollution | yes unless special-cased | yes unless special-cased | none (generation never enters history) |
| Revision semantics | generation bumps edit revisions | same | separate generation revision; edit revision counts user changes only (needs a rule, B-6) |
| Streaming cost | lowest | high per chunk | low (one materialisation per chunk) |
| Replication | whole chunk data or revision storms | operation storms | chunk baseline (data or seed+version) + delta journal |
| Persistence | needs a custom "generated vs edited" rule | operations only, huge | seed + version + journal, compact |
| Server authority | implicit | explicit | explicit (server runs the boundary) |
| New contracts needed | none | system actor, bulk ops in Editing API | WorldGen port, system commit/materialise port, generated-vs-edit rule, journal (listed in section 11) |
| Determinism check | possible | possible | required and central (stable hash per chunk) |
| Mobile | fine | risky (overhead) | fine; baseline can be regenerated locally or received |
| Migration of generator versions | ad hoc | operations baked in | explicit policy (section 8) |
| Implementation risk | lowest | medium | medium: one new boundary, clear tests |

## 6. Approved boundary (Option C, variant C3, in detail)

Components (the structure is Approved by AOD-20; names and signatures are proposals, not contracts):

1. **Generator (pure):** `generate(seed, generator_version, chunk_coord) -> GeneratedChunk` (a value: voxel data, biome ids, optional metadata). Deterministic, integer-based, no I/O, no time, no global state, no dependence on neighbours except through pure world-coordinate functions. Lives in `cube-worldgen` (depends on Core value types only, as today).
2. **WorldGen service (server side):** owns the streaming state machine, queue, cancellation, cache and the chunk lifecycle (Unloaded, Queued, Generating, Ready, Failed). Never exposes a write handle.
3. **Materialiser:** the only non-Editing component allowed to write generated data into the authoritative asset, through a **system commit port** of the Core (a narrow, audited entry point: "materialise generated chunk", "evict generated chunk"). It has no authority over user-modified data.
4. **Editing API:** all user and designer operations; a user operation may touch generated and user-modified data alike; the result is recorded as journal entries.
5. **Mutation journal:** versioned, append-only deltas of user changes with operation id, author, server timestamp/tick, generator version at the time and a schema version; compaction produces snapshots of modified chunks (optimisation, never the primary record).
6. **Persistence:** world record = `{seed, generator_version, config}` + journal (+ optional compaction snapshots). Untouched chunks cost nothing.

### Layer model options inside Core (open decision O-WG-1, `Spike Required`)

AOD-20 does not decide whether the Core keeps generated and user data in one layer or two.

* **Single-layer model:** one voxel layer; the materialised chunk is written once and journal entries for that chunk are replayed on top (this is what the spike does with its edit list). Minimal Core change; a generator-version change can invalidate absolute-voxel deltas (see section 8).
* **C2: generated base plus mutable overlay:** two layers with defined precedence. Reads compose overlay over generated; user data is physically separate; eviction of generated data never touches user data. More Core surface (layer composition rules), cleaner persistence and replication of user changes.
* **Recommendation (Proposed):** measure both in the next spike (generator-version bump with existing journals, mobile memory, replication size) and keep the journal format independent of the choice. A decision point, not a commitment.

### Rules

* **Write path rule:** only three callers may commit to the authoritative asset: the Editing service (user/designer operations), the Materialiser (generated data) and the persistence loader (restoring a saved state). Everything else reads snapshots/views. Enforced by dependency rules and a call-site audit test (to be written).
* **Revision rule (AOD-20 fixes the principle; details need the B-6 decision):** system generation receives an OperationId (origin: system) and takes part in the commit and revision semantics. Whether a system commit advances the same revision scope as an edit or a separate, chunk-scoped generation epoch is **not decided**; the earlier proposal (edit revision counts only Editing-API commits, generation uses a separate epoch) is a candidate. Clients must be able to tell a terrain update from a user edit either way.
* **History rule (Approved, AOD-20):** initial generation never appears in ordinary undo/redo. Undo applies to journal operations only; undoing a user edit restores the previous *effective* voxel (generated or earlier edit), not an absolute stored value.
* **Authority rule:** the server runs WorldGen and the Editing service; clients never run authoritative generation. A client may generate **locally for display/prediction** only if the generator version and the stable chunk hash match the server's; otherwise it receives chunk data. Which of "send seed + version" and "send chunk data" is default is open (O-WG-3).
* **Determinism rule:** the stable chunk hash (seed, coordinate, generator version, voxel content) is part of the contract; a mismatch between server and client generation is a protocol-visible error, never silently accepted.
* **Cancellation and queue rule:** generation jobs are cancellable (a chunk leaving the interest area cancels its pending job), bounded, prioritised by distance, and never block the Editing path.
* **Interest rule:** the set of chunks a client sees is decided by the server (view distance, permissions); the client does not request arbitrary chunks.

## 7. Server, client, single-player (embedded server Approved by AOD-22)

* Server: WorldGen service + Materialiser + Editing service + journal + persistence, one authoritative instance per world/shard.
* Client: receives chunk baselines and deltas, keeps a cache, predicts its own edits, reconciles. It does not persist an authoritative world.
* Single-player: **Approved (AOD-22)** to run the same authoritative server runtime **in process** (embedded), so that there is one authoritative path; a dedicated server runs the same runtime in its own process. A separate "local mode" with client-side authority is not introduced (AOD-14).

## 8. Generator versions and migration

A saved world records the generator version it was created with. Options when the generator changes (open decision O-WG-2):

| Policy | Meaning | Trade-off |
|---|---|---|
| Pin | keep every released generator version runnable and use the saved one | exact worlds forever; code and test burden grows with every version |
| Bake on change | when a generator version is retired, write the affected generated chunks as data (snapshot) and keep only the journal on top | bounded code; save grows; a deterministic bake needs the old generator once |
| Migrate | regenerate with the new version and re-apply the journal with a conflict rule | smallest saves; edits can land in the wrong place; needs a conflict policy and player-visible consequences |

Recommendation: **pin by default, bake on retirement**; "migrate" only for explicitly versioned world upgrades with an owner-approved conflict rule. A generator change that alters output without a version bump is a defect caught by the pinned hashes.

## 9. Failure and recovery

Generation failure marks only the chunk Failed (already the spike behaviour) and is retried or reported by the server; clients never see a half-materialised chunk (materialisation is atomic at brick/chunk granularity under the existing commit rule). A journal entry whose target is not representable (for example a generator version that no longer exists and was not baked) is a controlled error with a stable code, never a silent drop.

## 10. What stays experimental in the spike

`world-state-v0`, the streamer API, the direct commit writes, the revision-per-chunk behaviour and the spike undo journal remain experimental (report section 28) and are not to be taken as the implementation of this recommendation.

## 11. Contracts and tests needed before implementation (names only)

New or revised Draft contracts: WorldGen port (generator, chunk lifecycle, cancellation, progress), Materialiser/system-commit port in Core, generated-vs-edit precedence rule (single-layer model or C2), mutation journal schema (versioned), persistence world record, replication of baselines and deltas, determinism/hash contract. Contract and property tests: pinned and cross-device hash tests, generator-version replay tests, journal replay equals live state, materialise/evict idempotence, cancellation, border consistency, authority tests (a client cannot commit), conflict tests (generator bump with journal). The Draft persistence contracts and the RevisionToken proposal need review against this record (B-6, B-9, OI-4).

## 12. Decisions: recorded and still required

**Recorded (2026-11-05):** option C, variant C3, with the seven points of AOD-20; persistence model of AOD-21; embedded server of AOD-22; ACR-004 prepared (Pending).

**Still required from the owner (not decided by AOD-20):**

1. Single-layer model or C2 overlay in the Core (O-WG-1): the recommendation is to decide after the next spike.
2. Generator-version policy per world (O-WG-2): AOD-21 requires pin or explicit bake/migration and forbids silent change; the recommendation remains pin by default, bake on retirement.
3. Chunk delivery to clients (O-WG-3): chunk data by default, seed + version only for matched generators (Proposed).
4. The decision on ACR-004 and the revision effect of system commits (B-6).
5. Where the server runtime and WorldGen sit in the module grid and who owns them.
