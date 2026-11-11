# Owner Approval Matrix v1

**Status:** decision package for the project owner. First issued 2026-11-04 as a proposal; **updated 2026-11-05** to record the owner's decisions: OAM-01, OAM-02, OAM-03 and OAM-07 are **Approved**, OAM-08 is **Conditionally Approved** (full records, scope and limits: [`owner-architecture-decisions-v1.md`](owner-architecture-decisions-v1.md), AOD-20…24, section 5). OAM-04 stays Proposed, OAM-05 Spike Required, OAM-06 Proposed. The options, pros and cons below are kept as the reasoning behind each decision; text that is still a recommendation says so. No contract, ACR-001/002/003, format or transport changes because of this file.
**Inputs:** the Approved Owner Decisions AOD-01…24 ([`owner-architecture-decisions-v1.md`](owner-architecture-decisions-v1.md)), the roadmap ([`../architecture/roadmap-v1.md`](../architecture/roadmap-v1.md)), the WorldGen frame ([`worldgen-core-boundary.md`](worldgen-core-boundary.md)), the system boundaries ([`../architecture/system-boundaries-v1.md`](../architecture/system-boundaries-v1.md)), the reference study (`../research/`), the audit record ([`audit-resolution-report.md`](audit-resolution-report.md)) and the spike reports in `../spike/`.
**Untouched by this package:** P0/P1 contracts (Draft, AOD-18), AU-1.0 and ACR-001/002/003 (AOD-19), the persistence format, the TCP protocol, the spike code and `master`.

## 0. How to read this document

**Status vocabulary** (the same four words are used in the roadmap):

| Status | Meaning |
|---|---|
| **Approved** | recorded by the owner (AOD-01…24 except AOD-24, which is conditional) |
| **Conditionally Approved** | approved with stated conditions; the conditions are part of the decision (today AOD-24 only) |
| **Proposed** | a recommendation exists, the owner has not decided |
| **Spike Required** | the owner cannot reasonably decide without measured evidence; the spike is named |
| **Blocked** | cannot proceed until an external input exists (a source document, a device, an owner assignment) |

**Evidence labels** used in this file and in the research notes:

| Label | Meaning |
|---|---|
| **Fact (Cube)** | measured or observed in the Cube repository or spike (experimental, desktop or emulator only) |
| **Fact (upstream)** | observed in an upstream repository (Veloren, CubeWorld-Reversal); proves nothing about Cube |
| **Agent inference** | a conclusion drawn by the agent from facts; may be wrong |
| **Recommendation** | what the agent suggests the owner decides |
| **Owner decision** | an Approved record (AOD) or a future entry in the decision log |
| **Hypothesis** | not confirmed by any source or measurement |

**Index**

| ID | Decision | Status | Recorded decision or recommendation | Still needed from the owner |
|---|---|---|---|---|
| OAM-01 | WorldGen to Core boundary | **Approved** (AOD-20, 2026-11-05) | hybrid option C, variant C3; **ACR-004 prepared, Pending** | decision on ACR-004 (review package [`acr-004-owner-review.md`](acr-004-owner-review.md)), OAM-09 (C1 / C2), OAM-10 (revision model) |
| OAM-02 | Persistence | **Approved** (AOD-21, 2026-11-05) | `WorldSeed + GeneratorVersion`, durable journal, compaction into chunk snapshots, load order base generation, snapshot, journal tail | binary formats, concrete database, exact fields, journal granularity, retention numbers, per-world generator-version policy (all not approved) |
| OAM-03 | Embedded server | **Approved** (AOD-22, 2026-11-05; transport rule confirmed) | one authoritative Rust server runtime, embedded (in-process transport) or dedicated (external transport); no client-only authoritative mode | dedicated transport (OAM-05), runtime placement in the module grid, mobile measurement |
| OAM-04 | Renderer | Proposed (`faces` reference/default, `greedy` experimental per AOD-10) | no change; mobile decision needs a physical device | the default after a device benchmark |
| OAM-05 | Network layer | Spike Required | no transport chosen | approve the spike and its criteria; networking owner |
| OAM-06 | Mobile targets | Proposed | adopt the criteria table | approve criteria; devices and macOS/Xcode |
| OAM-07 | Licences and sources | **Approved** (AOD-23, 2026-11-05) | policy of section 7.1 | Cube's own licence, dependency licence list |
| OAM-09 | Core layer model: C1 or C2 | **Recommended / Requires Owner Approval** (C1 first baseline; C2 Deferred) | C1 for the first production baseline; no C2 before a separate spike | choose C1 / C2 |
| OAM-10 | Revision model for system commits | **Proposed / Requires Owner Approval** | same asset/layer scopes, `ActorKind::System`, no separate generation epoch | choose the model (with ACR-004 part D) |
| OAM-08 | Missing sources | **Conditionally Approved** (AOD-24, 2026-11-05) | limited temporary baseline; originals still required for final P0/P1 approval | supply the documents; confirm baseline entries |

Relation to roadmap open items: OAM-01 = OI-12 (and OI-7 for the system-commit port), OAM-02 = OI-4, OAM-03 = OI-6, OAM-05 = OI-9, OAM-06 = OI-10/OI-11, OAM-07 = OI-5, OAM-08 = OI-1/OI-2 (and OI-3 for the ACRs).

**Notation for option C (uniform in all documents):** **C1** = the first-baseline layer model: one logical geometry layer, the procedural base given by `WorldSeed + GeneratorVersion`, user changes held as block-level overrides, stored in the journal and snapshots (AOD-21), no C2 overlay; **C2** = the extended model: a separate generated base layer plus a separate mutable overlay read through compositing; **C3** = the approved architecture: hybrid system generation through the WorldGen/System-commit boundary, Editing API for user changes, and a separate persistence strategy. C3 is Approved (AOD-20); C1 versus C2 is a choice inside C3 (OAM-09, `Recommended / Requires Owner Approval`: C1; C2 Deferred).

---

## 1. OAM-01 WorldGen to Core

| Field | Content |
|---|---|
| **ID** | OAM-01 (also O-WG-1…3 in [`worldgen-core-boundary.md`](worldgen-core-boundary.md)) |
| **Current state** | **Approved 2026-11-05 as AOD-20.** Earlier: AOD-09, AOD-15, AOD-16 (Approved) left the write path open. The spike still writes generated chunks and edits through the direct commit primitive (documented as non-compliant, experimental). |
| **Options** | **A** WorldGen writes straight into Core. **B** WorldGen works only through the Editing API. **C** hybrid: pure Generator, WorldGen service, a Materialiser that writes generated data through a narrow Core system-commit port, user edits through the Editing API. The variants of C (C1, C2, C3) are defined in 1.1; C3 is the approved one. |
| **Recommended** | **Decision (Approved): option C, variant C3.** Not decided: the Core layer model (OAM-09, `Recommended / Requires Owner Approval`) and the revision model (OAM-10, `Proposed / Requires Owner Approval`). |
| **Pros** | A: fewest new contracts, lowest implementation risk. B: single write path, no new Core port. C: generated data stays out of history and the edit revision, matches AOD-15/16/17 structurally, regeneration and eviction never touch user data. |
| **Cons** | A: generation enters revisions and history unless special-cased; two meanings of "write"; weak authority story. B: generation becomes millions of operations (bulk operations, system actor, history pollution, storms on replication, a huge journal), a risk for mobile. C: one new boundary (system-commit port, generation revision) that touches P0 types and possibly ACR-001; more contracts to write. |
| **Impact on Core** | A: none. B: bulk-operation support is required on the Editing side, none in Core. C: a new **system-commit port** and a system OperationId and a revision rule for system commits (B-6; ACR-004); C2 would add layer composition rules; C1 needs none. |
| **Impact on WorldGen** | A, B: the module must call the write path directly or through an actor. C: Generator stays a pure function of (seed, version, coordinate); the service owns lifecycle, cancellation and cache; the Materialiser is the only writer for generated data. |
| **Impact on Persistence** | A: needs a custom "generated vs edited" rule. B: operations only, huge. C: generated data is not stored; seed + generator version + user changes only (AOD-17). |
| **Impact on Server** | A: authority is implicit. B: explicit but slow. C: the server runs the boundary (AOD-13); clients may regenerate locally only if version and chunk hash match, else they receive data (O-WG-3). |
| **Impact on Mobile** | A: fine. B: risky (overhead per operation). C: fine; baseline can be regenerated or received. The mobile cost of C2 (two layers in memory) is a **Hypothesis** until measured. |
| **Reversibility** | Direction A to C is cheap before production Core code exists and expensive afterwards (it changes P0 surface and the save format). The C1 versus C2 choice (OAM-09) is cheap while the journal schema is independent of it and expensive once a save format is frozen. |
| **Blocks** | The system-commit port and generation-revision contracts (Gate 2), production `cube-worldgen` (Gate 3), journal schema (Gate 5), replication of chunk baselines (Gate 7). |
| **Owner decision required** | Recorded: direction C3 with the seven points of AOD-20 (system generation through a WorldGen/System-commit boundary; not an Editing-API operation; user changes only through the Editing API; server owns the world; initial generation outside ordinary undo/redo; system OperationId and revision/commit semantics; no bypass of the public Core ports). **Still required:** decision on **ACR-004 (prepared, Pending; owner review package: [`acr-004-owner-review.md`](acr-004-owner-review.md))**; OAM-09; OAM-10. |

### 1.1 Variants of C and the resulting position

**Notation.** C1, C2 and C3 are defined once, below and in the notation paragraph above, and used identically in every document.

| Aspect | **C1** first baseline | **C2** extended model | **C3** approved architecture |
|---|---|---|---|
| Idea | one logical geometry layer; base = `WorldSeed + GeneratorVersion`; user changes as block-level overrides; journal and snapshots store the changes | separate generated base layer plus separate mutable overlay; reads through compositing | hybrid system generation (WorldGen/System-commit boundary), Editing API for user changes, separate persistence strategy (AOD-21) |
| Core change | none beyond the system-commit port (overrides are ordinary committed voxels; the loader re-applies them) | layer composition rules, precedence, eviction, compositing reads | system-commit port, `ActorKind` on OperationId, revision rule (ACR-004) |
| Meshing | reads one layer | meshing and dirty tracking must follow compositing | independent of the layer model |
| Persistence | overrides recorded in the journal; snapshots per chunk | overlay and base persisted or regenerated separately; more files and rules | AOD-21 |
| Revision semantics | the same scopes for generated and user data | two data sources per layer; revision of a composed read is derived | OAM-10 |
| Generator-version change | overrides can contradict a changed base: pin or explicit bake/migration (AOD-21) | same risk; the overlay can be baked separately | explicit policy (AOD-21) |
| Undo | the journal must carry the previous effective value (pre-image) or the base must be recomputable | natural: remove or restore the overlay entry | user operations only; generation excluded |
| Mobile memory | one layer | two layers (Hypothesis: higher) | depends on the layer model |
| Evidence | Fact (upstream): Veloren re-applies block-level overrides over regenerated terrain (per-chunk files, written on unload); nothing measured for Cube | none upstream; nothing measured for Cube | Fact (Cube): the spike re-applies an edit list after regeneration; the boundary is not implemented |
| **Status** | **Recommended / Requires Owner Approval** (OAM-09) | **Deferred**: not implemented before a separate spike | **Approved** (AOD-20) |

**Agent inference:** C3 is the architecture; C1 and C2 are alternative layer models inside it. C1 is the smaller change and keeps meshing and revision semantics simple; C2 gives cleaner separation at higher cost in meshing, persistence and revision rules.
**Spike criteria for C2 (proposed, only if the owner later wants C2):** generator-version bump behaviour, memory on an Android-class budget, compositing cost in meshing, replication size, undo correctness, cost of the second layer's persistence.

---

## 2. OAM-02 Persistence

| Field | Content |
|---|---|
| **ID** | OAM-02 (OI-4, NB-5) |
| **Current state** | **Approved 2026-11-05 as AOD-21** (refines AOD-17). The Draft persistence contracts and the `cube-persistence` spike are not aligned with it (OI-4). `world-state-v0` stays a temporary spike file. **No binary format is approved.** |
| **Options** | **P-A** full chunk storage. **P-B** seed + block-level overrides. **P-C** seed + operation journal. **P-D** hybrid: journal as durable record, per-chunk snapshots as compaction, seed + generator version for untouched chunks. |
| **Recommended** | **Decision (Approved): P-D hybrid.** `WorldSeed + GeneratorVersion` for the untouched world; a durable mutation journal for user changes; periodic compaction into chunk snapshots (acceleration, not the sole source of truth); the journal survives sudden process termination; load order base generation, snapshot, journal tail; a generator change never silently changes an existing world (pin, or explicit bake/migration). Not decided: journal granularity, schemas, compaction policy, flush numbers, per-world version policy. |
| **Pros** | P-A: simplest, readers need no generator; immune to generator changes. P-B: small, per-chunk files, direct chunk load. P-C: exact history, replay, natural server order. P-D: bounded recovery time (snapshots), append-only durability (journal), small untouched world. |
| **Cons** | P-A: large (a mobile storage and write-amplification risk), loses the AOD-17 benefit, cannot distinguish generated from edited data. P-B: no operation identity or history; ordering across players needs extra data; power-loss behaviour depends on when files are written. P-C: growth without compaction; replay time; generator changes can invalidate recorded absolute values. P-D: two formats to specify, test and migrate; compaction is a new failure source. |
| **Impact on Core** | P-A: serialisation of full bricks. P-B/P-C/P-D: need stable content hashes and the generated-vs-user distinction (OAM-01). None changes Core types if the commit port exists. |
| **Impact on WorldGen** | P-A: the generator is only needed for new chunks. P-B/C/D: the generator version becomes part of the data and old versions must be runnable, baked or migrated (O-WG-2). |
| **Impact on Persistence** | The subject of this decision; see the table below. |
| **Impact on Server** | The server owns the journal and snapshots (AOD-13); clients hold no authoritative save. P-C/P-D make the journal order the server order. |
| **Impact on Mobile** | Write volume, storage growth, flush behaviour on suspend or kill and low-memory behaviour are device questions; no measurement exists. P-A has the largest write volume (Agent inference). |
| **Reversibility** | The direction is reversible while no format is frozen. A frozen, shipped format is expensive to change, so versioned schemas and a migration framework are required from the first release (roadmap Gate 5). |
| **Blocks** | The persistence contract rewrite (OI-4), Gate 5, replication of baselines (Gate 7), the answer for O-WG-2 (generator version policy). |
| **Owner decision required** | Recorded as AOD-21; the compaction protocol and recovery rules were **confirmed as Approved architecture on 2026-11-05** (section 2.2). **Still required:** the persistence spike and format approval (Gate 5); formats, concrete database, exact fields (Proposed / Spike Required); the per-world generator-version policy; the local single-player store (REF-23). |

### 2.1 Comparison on the requested criteria

| Criterion | P-A full chunk | P-B seed + overrides | P-C seed + journal | P-D hybrid |
|---|---|---|---|---|
| **Power loss** | write amplification; atomic replace of a whole file is possible; loses changes since the last write | durable only as often as files are written (Fact (upstream): Veloren writes on unload only and acknowledges this) | append of small records with a flush policy; loss bounded by the last unflushed record | as P-C; snapshots never needed to be current for durability |
| **Atomicity** | atomic file replace per chunk; multi-chunk edits are not atomic across files unless grouped | same per chunk | one operation = one record or a group record with a commit marker (to design) | journal group commit plus atomic snapshot replace |
| **Recovery** | load the last good file | load files; unknown or corrupt files backed up and the chunk starts empty (Fact (upstream): Veloren behaviour) | replay up to the last valid record, tolerate a truncated tail, checksum per record | last good snapshot plus journal tail replay |
| **Version migration** | migrate every stored chunk; readers for old formats | file-format versions with fallback readers (Fact (upstream)); generator version not covered | schema version per record; replay with converters | schema version per record and per snapshot; converters; snapshots can be rewritten at upgrade |
| **Generator change** | none (data is stored) | overrides can contradict the new base; pin or bake needed | recorded operations may land in the wrong place; pin or bake or migrate with a conflict rule | snapshots bake the old base for retired generators; journal on top |
| **Compacting** | not needed (rewrite on change) | drop overrides equal to generated (Fact (upstream)); delete empty overlays | required to bound growth and replay time | required and central: snapshot then truncate the journal prefix; failure leaves the old files valid (design requirement) |
| **Multiplayer consistency** | last write wins per chunk unless versioned; revision storms | needs per-block ordering information | server order is the journal order; operation ids give idempotence | as P-C; replicas catch up from snapshot plus deltas |

**Agent inference:** only P-C and P-D give operation identity for undo, idempotent retry and conflict handling without extra machinery; P-B is attractive for per-chunk loading and is what Veloren does, but for a server-authoritative multiplayer game with undo it lacks operation identity. This is an inference, not a measurement.
**Spike criteria (proposed):** kill-during-write and truncated-tail recovery tests; replay time per 10 000 operations on a mobile-class device; storage growth per hour of scripted play; compaction crash safety; generator-bump behaviour for the three version policies.

### 2.2 Compaction protocol and recovery rules (Approved architecture, AOD-21, confirmed 2026-11-05)

`write snapshot → fsync snapshot → verify snapshot → atomically update manifest → fsync manifest → compact/truncate journal only after successful verification`

* a snapshot is not the only source of truth before verification; the previous snapshot is kept until the new one is activated; the journal tail is kept until the new recovery point is confirmed;
* a crash between any steps leaves a recoverable state (crash table in [`../api-contracts/world-persistence-contracts.md`](../api-contracts/world-persistence-contracts.md), WP-07);
* a snapshot contains its base generator version and revision boundary; a generator change never alters an existing world silently; compaction is idempotent; recovery checks hash, epoch, revision and generator version.

Status: the **architecture is Approved**. Binary format, concrete database, exact file fields, hash algorithm, platform durability mechanism and retention numbers remain `Proposed` or `Spike Required`.

---

## 3. OAM-03 Embedded server

| Field | Content |
|---|---|
| **ID** | OAM-03 (OI-6) |
| **Current state** | **Approved 2026-11-05 as AOD-22.** Builds on AOD-05 and AOD-14. The spike has a loopback TCP server and an in-process runtime that does not follow the authority model. |
| **Options** | **S-A** single-player as an embedded server instance (same components, in-process transport). **S-B** a separate server process even for single-player. **S-C** local client-only mode (client holds authoritative state). |
| **Recommended** | **Decision (Approved): S-A.** One authoritative Rust server runtime: embedded in the client process for single-player, in its own process as the dedicated server; no client-only authoritative mode (S-C not introduced); transport, persistence and authority boundaries identical; the modes differ only in launch and lifecycle. |
| **Pros** | S-A: one authoritative code path, the same tests and the same Editing/replication interfaces for single-player and multiplayer; no second process on mobile; works offline. S-B: strongest isolation, simplest to reason about as a product ("always a server"). S-C: least work now, lowest runtime cost. |
| **Cons** | S-A: server and client share a process, memory and thermal budget on a phone (unmeasured); a crash takes both down; needs a clean in-process transport. S-B: a second process is difficult or forbidden on mobile (iOS) and conflicts with the no-child-process rule (AOD-12). S-C: two authority models, edits and persistence bypass the Editing service, contradicts AOD-14, and migration to multiplayer becomes a rewrite. |
| **Impact on Core** | S-A and S-B: none (Core is the same). S-C: needs a local-authority write path. |
| **Impact on WorldGen** | S-A: the service runs in process; threading budget is shared with meshing (mobile concern). S-B: separate budget. S-C: ad hoc. |
| **Impact on Persistence** | S-A: the embedded server owns the journal and snapshots locally (the local single-player store, REF-23, must be decided). S-B: same, in the other process. S-C: a client save, i.e. a client-authoritative save (not allowed). |
| **Impact on Server** | S-A: the server must be embeddable as a library with a transport abstraction (in-process channel and network). This is an additional requirement on the server host design. |
| **Impact on Mobile** | S-A: CPU, memory and battery of the server loop must fit the phone budget; Hypothesis, no measurement. S-B: on iOS a second process is not an option (Agent inference; to verify). S-C: lowest cost. |
| **Reversibility** | S-A to S-B (dedicated) is cheap if the interfaces stay transport-independent. S-C to either is expensive. |
| **Blocks** | The server host design (Gate 7 scoping), the local store decision (REF-23), Gate 6 local game structure, the prediction and reconciliation scaffolding. |
| **Owner decision required** | Recorded as AOD-22; the embedded transport rule was **confirmed on 2026-11-05** (section 3.2). **Still required:** the external transport of the dedicated server (OAM-05, Spike Required); placement of the runtime in the module grid; the transition conditions of 3.1 (Proposed); a physical-device measurement of the embedded runtime. Fact (upstream): Veloren embeds its server behind a `singleplayer` feature; its authority rules for that mode were not examined, so it is not evidence for Cube. |

### 3.1 Conditions to move from embedded to dedicated server (Proposed; not part of AOD-22)

Move a given world from an embedded instance to a dedicated server when **any** holds:

1. more than one remote participant must share the world (the world is hosted, not local);
2. the world must run while no client is open (persistent simulation, scheduled events);
3. the embedded instance exceeds the device budget on the reference devices in OAM-06 (memory, frame time or thermal) and cannot be brought inside it;
4. server-side anti-cheat or moderation is required for the world;
5. the world is migrated to a hosted environment by the owner or the player (export of seed, generator version and journal).

The migration is possible only if the persisted world is transport- and process-independent (OAM-02) and the Editing/replication interfaces are identical in both modes. The in-process versus external transport difference is decided (3.2); whether the conformance suite proves identical behaviour is a Gate 6/7 verification item.

### 3.2 Transport rule (Approved, AOD-22, confirmed 2026-11-05)

* the embedded server uses an **in-process transport**; the dedicated server uses an **external transport**; the implementations may differ;
* protocol, authority, authentication, authorization, OperationId, revision and conflict rules are identical on both; the embedded mode has no special rights and is not client-authoritative;
* the conformance tests run on **both** transport implementations ([`../api-contracts/server-runtime-contracts.md`](../api-contracts/server-runtime-contracts.md), SR-09).

The external transport of the dedicated server is **not chosen** (OAM-05 stays `Spike Required`).

---

## 4. OAM-04 Renderer

| Field | Content |
|---|---|
| **ID** | OAM-04 |
| **Current state** | AOD-10 (Approved): `faces` is the reference renderer, `greedy` is experimental until a real mobile benchmark. `cubes` is a control mode. Fact (Cube, experimental): `greedy` is T-junction free and pixel-identical to `faces` in the compared scenes, with about 55% fewer triangles at the 5x5 area; all numbers are from a desktop (Windows, RTX 3070 Ti) or Android emulators. **There is no physical-device result.** |
| **Options** | **R-A** `faces`. **R-B** `greedy`. **R-C** a future hybrid renderer (for example a per-section or per-distance choice between `faces` and `greedy`; or `greedy` with AO or an atlas). |
| **Recommended** | Status: **Proposed** (the `faces`/`greedy` statuses are already Approved in AOD-10). Keep: `faces` = reference/default, `greedy` = experimental, hybrid = not started. **The mobile production decision is not taken without a physical device.** No new owner decision is needed now. |
| **Pros** | R-A: simple, edit-friendly, oracle for correctness. R-B: fewer triangles and draw cost (desktop and emulator measurements), smaller mesh memory. R-C: could combine edit cost and render cost. |
| **Cons** | R-A: more triangles (24 292 against 10 858 at 5x5, Fact (Cube)). R-B: a larger rebuild unit, a documented seam rule, more code; benefit on a real GPU unproven. R-C: Hypothesis only, two code paths, unmeasured, interacts with AO/atlas ideas (REF-15). |
| **Impact on Core** | None for all (renderers read views and dirty regions). |
| **Impact on WorldGen** | None directly; generated data is the input. |
| **Impact on Persistence** | None (derived data is never persisted). |
| **Impact on Server** | None (meshing is client-side; server-side meshing is not planned). |
| **Impact on Mobile** | The deciding factor: GPU draw cost, memory, thermal behaviour and rebuild latency on ARM64 devices. Emulator results are not evidence (AOD-11). |
| **Reversibility** | High: renderers sit behind the adapter and are selected by mode. The decision to bake an AO/atlas format into vertex layout is less reversible. |
| **Blocks** | The Gate 4 exit (renderer default), mobile memory budgets, any decision on AO/lighting/atlas. |
| **Owner decision required** | None now. Later: record the default renderer after a physical-device comparison of `faces` and `greedy` on at least two devices. |

---

## 5. OAM-05 Network layer

| Field | Content |
|---|---|
| **ID** | OAM-05 (OI-9, REF-04, REF-33) |
| **Current state** | Only the spike's loopback TCP protocol exists; it is Draft, unauthenticated, and not designed for replication, prediction or mobile networks. **No transport is chosen and none may be chosen without a spike.** |
| **Options** | **N-A** TCP. **N-B** QUIC / UDP-based. **N-C** an existing external transport or networking library. **N-D** a split control channel (reliable, ordered: sessions, edits, permissions) and data channel (bulk and unreliable-tolerant: chunk streaming, entity state), over one or several transports. |
| **Recommended** | **No selection.** Agent recommendation: design the protocol as transport-independent (REF-33, an I/O-free protocol layer testable without sockets), allow N-D as a structure, and run a transport spike comparing N-A, N-B and a candidate of N-C. `Spike Required`. |
| **Pros** | N-A: ubiquitous, firewall friendly, simple, trivially secured with TLS. N-B: multiplexed streams without head-of-line blocking, connection migration (useful when a phone changes networks), built-in encryption. N-C: less code, tested implementation. N-D: priorities, no bulk transfer starving edits. |
| **Cons** | N-A: head-of-line blocking, poor behaviour under packet loss and network changes. N-B: library maturity on Android and iOS, UDP blocked by some networks, battery and radio behaviour unmeasured, more complexity. N-C: dependency, licence and platform support to verify (OAM-07), lock-in. N-D: two channels to keep consistent; ordering across channels. |
| **Impact on Core** | None (Core is transport-independent). |
| **Impact on WorldGen** | Chunk delivery policy (seed + version versus chunk data) depends on bandwidth; no direct dependency. |
| **Impact on Persistence** | None, except that the journal format and the wire delta format should share record definitions where possible (Agent inference). |
| **Impact on Server** | The central dependency: sessions, interest management, replication and the tick loop depend on stream semantics. |
| **Impact on Mobile** | NAT traversal, radio wake-ups, battery, network switches (Wi-Fi to cellular), suspend/resume and reconnect are mobile-specific risks with no measurement. |
| **Reversibility** | Moderate if the protocol layer is transport-independent; expensive if protocol semantics follow one transport. |
| **Blocks** | The protocol specification, authentication handshake, server load tests, Gate 7 exit. |
| **Owner decision required** | Approve a transport spike with the criteria below; assign the networking owner (O-3). **Not** a transport selection. |

**Spike criteria (proposed):** throughput and latency under loss and jitter; behaviour on a network switch and on resume from suspend; connection time and reconnect; CPU and battery on a physical device; encryption and authentication fit; Android/iOS library support and licence; head-of-line behaviour with mixed edit and chunk traffic; testability without sockets. **Fact (upstream):** Veloren has a separate network crate with an I/O-free protocol core, backends for TCP, in-process channels and QUIC, stream priorities and a bandwidth budget (not examined in depth, and no mobile client exists there); it is a design reference, not evidence of fitness for Cube.

---

## 6. OAM-06 Mobile targets

| Field | Content |
|---|---|
| **ID** | OAM-06 (OI-10, OI-11, NB-1, NB-2) |
| **Current state** | AOD-08 (Android and iOS mandatory) and AOD-11 (emulator is not a device) are Approved. Fact (Cube): no physical Android ARM64 result (`Blocked: no physical ARM64 device`), no iOS (`blocked: no macOS/Xcode`). The Android emulator runs (x86_64 native, ARM64 translated) are functional runs only. `scripts/device-validation.sh` exists and refuses to run without a physical ARM64 device. |
| **Options** | **M-A** adopt the acceptance criteria below as the definition of "mobile ready". **M-B** adopt a reduced set (for example Android only) and record the iOS exclusion. **M-C** defer the criteria until devices exist. |
| **Recommended** | **M-A** `Proposed`. Thresholds are placeholders until the first device data (roadmap marks them provisional). |
| **Pros** | M-A: one agreed acceptance definition prevents later disputes and emulator claims. M-B: matches the resources available today. M-C: no commitment with no data. |
| **Cons** | M-A: thresholds may change; some items cannot be tested yet. M-B: contradicts AOD-08. M-C: renderer, memory and WorldGen budgets have no target. |
| **Impact on Core** | Brick size, memory per chunk and snapshot costs need device budgets. 16^3 bricks are not validated on a device. |
| **Impact on WorldGen** | Per-chunk generation budget, cross-device hash equality (server vs Android ARM64 vs iOS), cache bounds. |
| **Impact on Persistence** | Write volume, flush on suspend and kill, storage growth, low-memory behaviour. |
| **Impact on Server** | Embedded server cost (OAM-03), reconnect on network change (OAM-05). |
| **Impact on Mobile** | The subject of this decision. |
| **Reversibility** | The criteria are easy to amend. The facts they reveal (for example that a device class cannot run the game) are not. |
| **Blocks** | The Gate 4 exit, Gate 10, the Gate 3 device hash exit, the claim "mobile-ready". |
| **Owner decision required** | Approve the criteria table (thresholds as provisional); supply a physical device matrix and macOS/Xcode access, or record an explicit deferral with its consequences. |

### 6.1 Acceptance criteria (proposed; every row needs a physical device, an emulator never satisfies it)

| Area | Criterion (provisional) | How it is measured | Evidence today |
|---|---|---|---|
| Real Android ARM64 | at least a low, a mid and a high class device run the signed client package; the scripted test, touch input and a 10-minute soak pass | `scripts/device-validation.sh` (faces and greedy, three launches each) | none; blocked |
| iOS ARM64 | the same scripted run on a physical iPhone; library delivery method decided | Xcode/macOS pipeline (to define) | none; blocked (no macOS/Xcode) |
| Memory | peak and steady resident memory below the device's low-memory-kill margin at the profile's view radius; no growth over the soak | platform memory sampling during the run | none |
| Cold start | time to first interactive frame within a target (proposal 5 s) | repeated cold launches, median and worst | none (emulator start-up is not evidence) |
| Thermal soak | sustained run of at least 10 minutes (longer for release) without throttling-induced frame rate below the floor | device thermal telemetry plus frame statistics | none |
| Battery | drain per hour of scripted play, compared across renderer modes | battery stats on device | none |
| Memory pages | the native libraries load and pass the scripted run on devices and images with **4 KB** pages and with **16 KB** pages; ELF alignment is correct | run on both configurations; static alignment check of the shared objects | Not verified: 16 KB alignment is a planned Gate 1 artifact (roadmap), no alignment check exists yet and no run on a 16 KB device was made. **Unconfirmed:** the exact store requirement and its dates must be re-checked against the current platform policy when the release plan is written. |
| Store build and signing | release-signed package (not debug keys), provisioning/signing secrets handled outside the repository, store privacy and permission declarations | release pipeline (Gate 1/10) | none; the spike uses debug signing |
| No child processes | no `OS.execute`, no test hooks in the client package | `no_child_processes` guard and `scripts/check-apk-no-exec.py` | Fact (Cube): present and passing for the spike packages |

The frame floor (30 FPS low-end, 60 FPS target on mid-range) and the other roadmap budgets remain proposals.

---

## 7. OAM-07 Licences and sources

| Field | Content |
|---|---|
| **ID** | OAM-07 (OI-5, NB-3, REF-28) |
| **Current state** | **Approved 2026-11-05 as AOD-23.** The reference study already followed this rule. Cube's own licence is not decided. |
| **Options** | **L-A** observation-only policy (below). **L-B** no study of third-party projects at all. **L-C** permit direct code reuse under compatible licences case by case. |
| **Recommended** | **Decision (Approved): L-A**, wording in 7.1. |
| **Pros** | L-A: keeps Cube's licence choice open and avoids GPL and provenance exposure while retaining the useful design observations. L-B: zero exposure. L-C: speed. |
| **Cons** | L-A: slower, needs discipline on identifiers and wording. L-B: loses cheap, valuable lessons. L-C: GPL code binds Cube to GPL terms; reverse-engineered code has provenance risk; needs legal review per item. |
| **Impact on Core** | None, except that no third-party algorithm may enter Core without review. |
| **Impact on WorldGen** | The generator must be original: algorithms from other projects (noise, layering) are not carried over without review. |
| **Impact on Persistence** | Formats must not mirror another project's file layout. |
| **Impact on Server** | The network layer (OAM-05) must check the licence of any external library. |
| **Impact on Mobile** | Store licensing and open-source notices for dependencies. |
| **Reversibility** | A published choice of Cube's own licence is hard to reverse; copied third-party code is hard to remove once distributed. |
| **Blocks** | Gate 0 exit, any third-party dependency on the roadmap, publication of the repository or builds. |
| **Owner decision required** | Recorded as AOD-23. **Still required:** Cube's own licence model; the licence list for dependencies (including any external transport library, OAM-05); separate legal review for each case marked as legally sensitive. |

### 7.1 Policy (Approved, AOD-23, 2026-11-05)

1. **Veloren code and assets are not copied.** Veloren (GPL-3.0-or-later) is used only as an architectural reference.
2. **CubeWorld-Reversal is used only as research material.**
3. **Reverse-engineered code, assets, original implementation names and decompiled algorithms are not carried into Cube.** Class and function names of the reverse-engineered project are not recorded in Cube documents.
4. **Architectural ideas are re-thought and implemented independently**; observations are written in our own words with an evidence label (section 0). Veloren file paths appear only as verification references.
5. **Legally sensitive cases are marked as requiring separate review** (marker: "legal review required") and are not acted on until reviewed.
6. Third-party dependencies are listed with their licences before adoption (a consequence of the policy, to be confirmed with the owner's licence decision).
7. A change to this policy is a new owner decision (AOD change control).

### 7.2 Upstream sources (remote, read-only)

| Source | Link | State read |
|---|---|---|
| Veloren (GitLab) | https://gitlab.com/veloren/veloren | `master`, commit `7daa95fd53ae62aa185d31ad316d0cabfe2d70ac`, 2026-10-30; GPL-3.0-or-later |
| Veloren book (codebase structure) | https://book.veloren.net | page `contributors/developers/codebase-structure` |
| CubeWorld-Reversal (GitHub) | https://github.com/qad3n/CubeWorld-Reversal (tree: https://github.com/qad3n/CubeWorld-Reversal/tree/master/cube) | `master`: README, file tree, folder listings, four declaration-only headers; no implementation file read; no licence shown |

The study notes are [`../research/veloren-notes.md`](../research/veloren-notes.md), [`../research/cubeworld-reversal-notes.md`](../research/cubeworld-reversal-notes.md) and [`../research/reference-comparison.md`](../research/reference-comparison.md).

---

## 8. OAM-08 Missing sources

| Field | Content |
|---|---|
| **ID** | OAM-08 (OI-1, OI-2, OI-3, B-1, B-13) |
| **Current state** | **Conditionally Approved 2026-11-05 as AOD-24.** Four inputs are absent ([`audit-resolution-report.md`](audit-resolution-report.md) O-6, B-1, B-13). Architecture Update v1.0 is stored ([`architecture-update-v1.0.md`](architecture-update-v1.0.md)). B-1 and B-13 stay open. |
| **Options** | **X-A** the owner supplies the files. **X-B** the owner approves named, temporary assumptions per item (table below). **X-C** proceed without either (not recommended). |
| **Recommended** | **Decision (Conditionally Approved):** a temporary owner-approved baseline for limited parts of Geometry Core, WorldGen and the roadmap; the original AU-1.0 stays preferred and the reconstructed copy is not regarded as byte-verified; the Geometry Processing Specification is **not** replaced by a general assumption before PR-01…PR-08 are approved; the missing documents remain a blocker for final P0/P1 approval. |
| **Pros** | X-A: contracts can be cross-checked and approved. X-B: lets non-dependent work continue. |
| **Cons** | X-A: waits on the owner. X-B: risk of building on a wrong assumption; contracts approved on an assumption may need re-approval. X-C: unverifiable contracts. |
| **Impact on Core** | Voxel Architecture may constrain C-07…C-12; AU-1.0 original gates the identity of the module and layer grid. |
| **Impact on WorldGen** | The GDD/TDD defines what generation must deliver (biomes, rivers, scale); without it Gate 8 scope is open. |
| **Impact on Persistence** | The GDD/TDD may define save, offline and sharing requirements. |
| **Impact on Server** | The GDD/TDD defines player counts and multiplayer scope (Gates 7 and 9). |
| **Impact on Mobile** | The GDD/TDD defines device targets and performance expectations. |
| **Reversibility** | Supplying a file is always reversible. A contract approved on an assumption is reversible only by re-approval. |
| **Blocks** | See the table. |
| **Owner decision required** | Recorded as AOD-24. **Still required:** supply the originals; confirm the baseline entries BL-1…BL-3 (proposed in AOD-24 section 5.5); no P0/P1 approval on a baseline alone. |

### 8.1 Per item

The column "Temporary owner-approved assumption" shows what the analysis of 2026-11-04 considered acceptable; **AOD-24 narrows it**: only for limited parts of Geometry Core, WorldGen and the roadmap, never for final P0/P1 approval, and never for PR-01…PR-08.

| Missing item | What is missing | What it blocks | Temporary owner-approved assumption | Contracts that must not be approved without the original |
|---|---|---|---|---|
| **GDD/TDD v0.3** | product requirements and technical design: platforms, scale, player counts, art direction, feature scope | scope of Gates 6, 8, 9; mobile targets and budgets (OAM-06); multiplayer scale (OAM-05, Gate 7); parts of the Voxel contracts | **Acceptable** for Gates 1 to 5 only: "AOD-01…24 are the complete product constraints" plus a stated list of scale assumptions (for example player count class) approved by the owner | gameplay-facing contracts; any contract whose behaviour depends on game scope; anything that sets player-count or content budgets |
| **Voxel Architecture v1.0** | the original voxel model specification (brick model, channels, layers, coordinate and material conventions) | cross-check of C-07…C-12 and the Core brick/material model; the chunk-as-Core-concept decision | **Acceptable, narrowly:** "the Rust spike's brick model (16^3 COW, 16-bit material) is the working model, without approval" for **spike and design work only** | C-07…C-12 and every P0 contract that freezes brick layout, material/channel model or coordinate conventions |
| **Geometry Processing Specification** | the original specification of derived data, bounds, validation and providers | PR-01…PR-08 and Processing behaviour; derived-data provider ownership (O-1) | **Not acceptable** for approval; acceptable only as a working draft for experiments | PR-01…PR-08 and any contract exposing derived data semantics |
| **AU-1.0 original** | the original approved Architecture Update; the repository holds a reconstructed copy, not byte-verified (B-13, partially closed) | the identity of the module and layer grid, ACR-001/002/003 (the copy states them), the dependency rules | **Acceptable** only if the owner explicitly confirms the reconstructed copy as the working text, recorded in writing; AU-1.0 itself is not rewritten (AOD-19) | any contract that cites an AU-1.0 clause not verifiable in the copy; any decision on ACR-001/002/003 |

---

## 8A. OAM-09 Core layer model: C1 or C2

| Field | Content |
|---|---|
| **ID** | OAM-09 (the open part of OI-12 and O-WG-1) |
| **Current state** | **`Recommended / Requires Owner Approval`.** AOD-20 (C3) does not decide it. Nothing is implemented; the spike uses a single layer. |
| **Options** | **C1** first baseline: one logical geometry layer; procedural base from `WorldSeed + GeneratorVersion`; user changes as block-level overrides; journal and snapshots store the changes; the C2 overlay is not implemented. **C2** extended model: a separate generated base layer, a separate mutable overlay, reads through compositing. |
| **Recommended** | **C1: approve for the first production baseline. C2: Deferred, not to be implemented before a separate spike.** Recorded as a recommendation only. |
| **Pros** | C1: no new Core layer semantics, simple meshing and revision rules, smallest memory, consistent with the AOD-21 journal; matches what the spike already does. C2: physically separates generated and user data, eviction of the base never touches user data, may simplify replication of user changes. |
| **Cons** | C1: a changed generator base can contradict overrides (needs pin or explicit bake/migration, already required by AOD-21); undo needs the previous effective value from the journal or a recomputable base; eviction must not drop overrides. C2: compositing in every read, harder meshing and dirty propagation, two persistence paths, more revision rules, higher memory (Hypothesis), and a larger P0 surface. |
| **Impact on Core** | C1: none beyond the system-commit port. C2: layer composition rules and precedence in P0. |
| **Impact on WorldGen** | C1: the Materialiser writes the base into the single layer; overrides are re-applied by the loader. C2: the Materialiser writes the base layer only. |
| **Impact on Persistence** | C1: journal of block-level changes plus snapshots (record granularity remains open under AOD-21, voxel deltas are the natural fit). C2: base and overlay lifecycle differ. |
| **Impact on Server** | C1: one authoritative layer. C2: composition on the server and in replication. |
| **Impact on Mobile** | C1: lower memory and CPU. C2: unmeasured, expected higher. |
| **Reversibility** | C1 to C2 is possible while the journal format is independent of the layer model and no save format is frozen; afterwards it needs a migration. Choosing C1 first keeps that path open. |
| **Blocks** | The P0 review of layers and the commit primitive, the meshing design for generated data, the journal record design. |
| **Owner decision required** | Choose C1 or C2 for the first production baseline. Not changed automatically. |

## 8B. OAM-10 Revision model for system commits

| Field | Content |
|---|---|
| **ID** | OAM-10 (ACR-004 part D, B-6) |
| **Current state** | **`Proposed / Requires Owner Approval`.** AOD-20 says system generation takes part in revision/commit semantics; the model is not chosen. RevisionToken width and commit algorithm (OD-R1, OD-R2) are open independently. |
| **Options** | **R-A** system operations use the same asset and layer revision scopes as other commits. **R-B** a separate generation epoch. **R-C** separate revision domains for the generated base and for user overrides. |
| **Recommended** | **R-A**, with `ActorKind::System` on the OperationId (ACR-004 part C). Do not introduce a separate generation epoch without proven necessity. Recorded as a recommendation only. |
| **Pros** | R-A: uses AU-1.0's existing revision levels (asset, layer, brick) and the invariant `Brick ⇒ Layer ⇒ Asset`; no new token kind; undo-as-forward-commit and conflict rules stay uniform; simplest to test; extendable later. R-B: terrain updates and user edits are told apart by the token. R-C: strongest separation of data sources. |
| **Cons** | R-A: streaming advances asset and layer revisions (Fact (Cube): one asset revision per streamed chunk in the spike), so a coarse revision no longer means "a user changed something"; clients must use the commit's `ActorKind`/`ChangeType`, and user edits must be validated with brick or region stamps ([`revision-and-snapshot-semantics.md`](../api-contracts/revision-and-snapshot-semantics.md) section 2), not asset-level tokens, to avoid spurious conflicts. R-B: a new revision kind, more comparison rules (the existing `Epoch` already means re-creation or recovery and would be overloaded or duplicated), more tests. R-C: two domains per layer, cross-domain consistency and conflict rules, effectively requires C2. |
| **Impact on Core** | R-A: none to RevisionToken; the commit result carries `ActorKind`. R-B/R-C: change C-16 and the comparison rules. |
| **Impact on WorldGen** | R-A: materialisation is a normal commit. R-B/R-C: the Materialiser needs its own revision handling. |
| **Impact on Persistence** | R-A: the journal records only user commits; system commits are regenerable. Recovery uses the existing `Epoch` rule. R-B/R-C: more state to record. |
| **Impact on Server** | R-A: the interest and replication logic filters by `ActorKind`. Others: more protocol state. |
| **Impact on Mobile** | R-A: least overhead. Others: unmeasured. |
| **Reversibility** | R-A is the easiest to extend later (additive); removing a separate epoch after release would be breaking, so introducing one first is the riskier order. |
| **Blocks** | ACR-004 part D, the P0 review of C-16/C-17/C-18 (with B-6), the system-commit contract (SC-04). |
| **Owner decision required** | Choose R-A, R-B or R-C. B-6 (token width, commit algorithm) stays open either way. |

## 9. Decisions that can be taken in parallel (remaining)

Independent of each other: Cube's own licence (OAM-07 remainder), supply of the missing sources (OAM-08), device and iOS access with the mobile criteria (OAM-06), owner assignments (OI-8), the transport spike approval (OAM-05), and the persistence durability spike (OAM-02). ACR-004 (parts A to C), OAM-09 and OAM-10 can be decided together in one review ([`acr-004-owner-review.md`](acr-004-owner-review.md)).

Sequential: OAM-10 and ACR-004 before the system-commit port contract is put up for approval; OAM-09 before the journal record design and the meshing design for generated data; the persistence spike before any persistence format; the transport spike before any protocol approval; the sources before P0/P1 approval; physical-device data before the renderer default.

## 10. What each blocker waits on

| Blocker | Waits on | Changed by 2026-11-05 decisions? |
|---|---|---|
| B-1 sources | owner supply (OAM-08) | no (limited baseline allowed, blocker stays open) |
| B-3 ACRs | owner decision on ACR-001/002/003 and the new ACR-004 | ACR-004 added as Pending |
| B-4 P0 not approved | B-1, B-3, B-6 | no |
| B-5 owners | owner assignment | no |
| B-6 revision rule | revision effect of system commits (OAM-10, with ACR-004 part D); token width and commit algorithm separately | principle fixed by AOD-20; recommendation R-A not yet approved |
| B-7 tool-level contracts | after B-4 | no |
| B-9 History/Persistence cycle | verifying test in CI | no |
| B-13 AU-1.0 original | owner supply or confirmation | no (copy stays unverified) |
| OI-4 / NB-5 persistence alignment | contract rewrite against AOD-21, persistence spike | direction decided |
| NB-1 physical device | owner acquisition (OAM-06) | no |
| NB-2 iOS | macOS/Xcode access (OAM-06) | no |
| NB-3 licence policy | Cube's own licence | policy approved (AOD-23) |
| NB-4 WorldGen direction | ACR-004, OAM-09, OAM-10 | direction approved (AOD-20); the three items are recommended, not approved |
| OI-9 transport | OAM-05 spike | no |

## 11. Next decisions to take (recommendation)

1. **Decide ACR-004 (parts A to C), OAM-10 (revision model) and OAM-09 (C1 or C2)** in one review with [`acr-004-owner-review.md`](acr-004-owner-review.md): they unblock the P0 review of the commit primitive, the journal record design and the meshing design.
2. **Supply the missing sources** (OAM-08) or confirm the baseline entries BL-1…BL-3: it is the only item that blocks final P0/P1 approval.
3. **Approve the two spikes** (persistence durability; transport) and assign the networking and WorldGen owners.

Not a decision but an action to start at once: acquire a physical Android ARM64 device and a path to macOS/Xcode (OAM-06).

## 12. Recording decisions

Decisions are recorded in [`docs/decisions/decision-registry.md`](decision-registry.md) (rules: DEC-002). The earlier text of this section, which pointed to [`owner-architecture-decisions-v1.md`](owner-architecture-decisions-v1.md), is replaced so that there is one recording process. A matrix row changes status only through a registry entry; nothing in this file is to be read as approved by itself.
