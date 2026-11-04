# Owner Approval Matrix v1

**Status:** **Proposed**, a decision package for the project owner. It records options, a recommendation and the consequences for each open decision. **It decides nothing.** No document, contract, ACR, format or transport changes status because of this file; the recommendations stay `Proposed` until the owner records a decision (see section 12).
**Inputs:** the Approved Owner Decisions AOD-01…19 (`owner-architecture-decisions-v1.md`), the roadmap (`../architecture/roadmap-v1.md`), the WorldGen frame (`worldgen-core-boundary.md`), the system boundaries (`../architecture/system-boundaries-v1.md`), the reference study (`../research/`), the audit record (`audit-resolution-report.md`) and the spike reports in `../spike/`.
**Untouched by this package:** P0/P1 contracts (Draft, AOD-18), AU-1.0 and ACR-001/002/003 (AOD-19), the persistence format, the TCP protocol, the spike code and `master`.

## 0. How to read this document

**Status vocabulary** (the same four words are used in the roadmap):

| Status | Meaning |
|---|---|
| **Approved** | recorded by the owner (today only AOD-01…19) |
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

| ID | Decision | Status | Recommended | Owner decision needed |
|---|---|---|---|---|
| OAM-01 | WorldGen to Core boundary (A / B / C, C1 / C2 / C3) | Proposed (direction), Spike Required (C1/C2/C3) | C, layer model decided after a spike | direction now, layer model later |
| OAM-02 | Persistence model | Proposed (direction), Spike Required (details) | hybrid: seed + generator version + journal, snapshots as optimisation | confirm AOD-17 reading, approve the spike |
| OAM-03 | Embedded server for single-player | Proposed | embedded server instance | yes / no, plus the transition conditions |
| OAM-04 | Renderer | AOD-10 Approved, device decision Blocked | `faces` default, `greedy` experimental | none now; the device benchmark later |
| OAM-05 | Network layer | Spike Required | no choice | approve the transport spike and its criteria |
| OAM-06 | Mobile targets and acceptance criteria | Proposed (criteria), Blocked (evidence) | adopt the criteria table | approve criteria, provide devices and macOS |
| OAM-07 | Licences and sources policy | Proposed, owner-only | clean-room: observations only | approve or amend the policy |
| OAM-08 | Missing sources | Blocked | supply the sources; temporary assumptions only where listed | supply files or approve named assumptions |

Relation to roadmap open items: OAM-01 = OI-12 (and OI-7 for the system-commit port), OAM-02 = OI-4, OAM-03 = OI-6, OAM-05 = OI-9, OAM-06 = OI-10/OI-11, OAM-07 = OI-5, OAM-08 = OI-1/OI-2 (and OI-3 for the ACRs).

---

## 1. OAM-01 WorldGen to Core

| Field | Content |
|---|---|
| **ID** | OAM-01 (also O-WG-1…3 in `worldgen-core-boundary.md`) |
| **Current state** | AOD-09, AOD-15, AOD-16 are Approved: generation is pure and deterministic; user edits go through the Editing API; procedural generation is a separate WorldGen/System boundary. **Which write path and which data model WorldGen uses is not decided.** The spike writes generated chunks and edits through the direct commit primitive (documented as non-compliant). |
| **Options** | **A** WorldGen writes straight into Core. **B** WorldGen works only through the Editing API. **C** hybrid: pure Generator, WorldGen service, a Materialiser that writes generated data through a narrow Core system-commit port, user edits through the Editing API. C is then split into the data models C1, C2, C3 below. |
| **Recommended** | **Direction C**, `Proposed`. Layer model (C1, C2 or C3): decide **after a spike**, `Spike Required`. |
| **Pros** | A: fewest new contracts, lowest implementation risk. B: single write path, no new Core port. C: generated data stays out of history and the edit revision, matches AOD-15/16/17 structurally, regeneration and eviction never touch user data. |
| **Cons** | A: generation enters revisions and history unless special-cased; two meanings of "write"; weak authority story. B: generation becomes millions of operations (bulk operations, system actor, history pollution, storms on replication, a huge journal), a risk for mobile. C: one new boundary (system-commit port, generation revision) that touches P0 types and possibly ACR-001; more contracts to write. |
| **Impact on Core** | A: none. B: bulk-operation support is required on the Editing side, none in Core. C: a new **system-commit port** and a generation revision rule (B-6); C2 adds layer composition rules; C1 and C3 need no new layer semantics. |
| **Impact on WorldGen** | A, B: the module must call the write path directly or through an actor. C: Generator stays a pure function of (seed, version, coordinate); the service owns lifecycle, cancellation and cache; the Materialiser is the only writer for generated data. |
| **Impact on Persistence** | A: needs a custom "generated vs edited" rule. B: operations only, huge. C: generated data is not stored; seed + generator version + user changes only (AOD-17). |
| **Impact on Server** | A: authority is implicit. B: explicit but slow. C: the server runs the boundary (AOD-13); clients may regenerate locally only if version and chunk hash match, else they receive data (O-WG-3). |
| **Impact on Mobile** | A: fine. B: risky (overhead per operation). C: fine; baseline can be regenerated or received. The mobile cost of C2 (two layers in memory) is a **Hypothesis** until measured. |
| **Reversibility** | Direction A to C is cheap before production Core code exists and expensive afterwards (it changes P0 surface and the save format). The C1/C2/C3 choice is cheap while the journal schema is independent of it and expensive once a save format is frozen. |
| **Blocks** | The system-commit port and generation-revision contracts (Gate 2), production `cube-worldgen` (Gate 3), journal schema (Gate 5), replication of chunk baselines (Gate 7). |
| **Owner decision required** | (1) Approve direction C, or choose A or B with the stated consequences. (2) Approve a spike that compares C1, C2, C3. (3) Decide whether a new ACR is filed for the system-commit port and the generation revision rule. |

### 1.1 Sub-variants of C

**Naming note.** `worldgen-core-boundary.md` (sections 6 and 12) used the names C1 ("single layer + journal re-application") and C2 ("generated + edit overlay"). This package uses the owner's request naming below. The old C1 corresponds to **C3** here (operation journal re-applied over a materialised chunk); the old C2 corresponds to **C2** here. The WorldGen document carries a pointer to this mapping.

| Aspect | **C1** seed + block-level overrides | **C2** generated base + mutable overlay (two Core layers) | **C3** operation journal + materialised chunk state |
|---|---|---|---|
| Idea | A chunk's saved data is a map of position to block value for changed blocks only; regenerated terrain is the base and overrides are applied over it | Generated data in one layer, user changes in a second layer with defined precedence; reads compose the two | The authoritative record is the ordered, versioned journal of user operations; a chunk is materialised and the journal for that chunk is replayed over it; snapshots optional |
| Core change | none (overrides are applied by the Materialiser/loader through the commit port) | layer composition rules, precedence, eviction rules (new Core semantics) | none beyond the commit port |
| History / undo | undo restores the previous effective voxel; the override store has no operation identity unless added | natural: overlay entries carry the operation; undo removes or restores overlay entries | natural: operations are the record |
| Generator-version change | absolute values stay valid only where the old terrain is still the base; a changed base can contradict overrides (Fact (upstream): Veloren versions the file format, not the generator; not examined further) | same risk, but overlay is cleanly separable so a bake or migrate policy can act on one layer | same risk; replay of operations with absolute values may land in the wrong place |
| Size on disk | proportional to changed blocks; "override equals generated" can be dropped | proportional to changed blocks | proportional to operation count until compaction |
| Power loss / atomicity | per-chunk atomic replace works, but durability depends on write timing (Fact (upstream): Veloren writes on unload only and states this is not durable across power loss) | same as C1 per layer | append-only record with a checksum per record; a truncated last record is tolerated (design proposal, to be tested) |
| Replication | whole-chunk override sets or diffs | overlay deltas | operation stream |
| Multiplayer consistency | needs a per-block sequence or revision to order concurrent changes | overlay revision per chunk | journal order is the server order |
| Mobile memory | one layer | two layers in memory (Hypothesis: higher) | one layer plus a bounded journal tail |
| Evidence | Fact (upstream): Veloren persists block-level overrides per modified chunk and re-applies them on regeneration; Cube has no measurement | none upstream; Cube has nothing measured | Fact (Cube): the spike keeps an edit list re-applied after generation (`world-state-v0`) |
| Status | Spike Required | Spike Required | Spike Required |

**Agent inference:** C1 and C3 are not exclusive. A journal (C3) can be the primary durable record while chunk-level override snapshots (C1) are the compaction format. C2 is about the in-memory model, C1/C3 about durable form, so the owner's real choice is the pair (in-memory model, durable record).
**Recommendation:** run one spike that implements C3 as the durable record with C1-style snapshots as compaction, over both a single-layer and a two-layer in-memory model (C2), and measures: generator version bump with existing journals, memory on an Android-class budget, replication size, undo correctness, cost of compaction. Do not approve a layer model before that.
**Spike criteria (proposed):** journal replay equals live state; undo restores the effective voxel; a generator bump has a defined, tested outcome; overlay eviction never touches user data; peak memory per loaded chunk; replication bytes per edit.

---

## 2. OAM-02 Persistence

| Field | Content |
|---|---|
| **ID** | OAM-02 (OI-4, NB-5) |
| **Current state** | AOD-17 is Approved: persistence = seed + generator version + versioned deltas/journal. The Draft persistence contracts and the `cube-persistence` spike are not aligned with it (OI-4). `world-state-v0` (seed + edits + CRC) is a temporary spike file. No persistence format is approved. |
| **Options** | **P-A** full chunk storage. **P-B** seed + block-level overrides. **P-C** seed + operation journal. **P-D** hybrid: journal as durable record, per-chunk snapshots as compaction, seed + generator version for untouched chunks. |
| **Recommended** | **P-D** `Proposed`, which is consistent with AOD-17. Journal granularity, snapshot format and compaction policy are `Spike Required`. |
| **Pros** | P-A: simplest, readers need no generator; immune to generator changes. P-B: small, per-chunk files, direct chunk load. P-C: exact history, replay, natural server order. P-D: bounded recovery time (snapshots), append-only durability (journal), small untouched world. |
| **Cons** | P-A: large (a mobile storage and write-amplification risk), loses the AOD-17 benefit, cannot distinguish generated from edited data. P-B: no operation identity or history; ordering across players needs extra data; power-loss behaviour depends on when files are written. P-C: growth without compaction; replay time; generator changes can invalidate recorded absolute values. P-D: two formats to specify, test and migrate; compaction is a new failure source. |
| **Impact on Core** | P-A: serialisation of full bricks. P-B/P-C/P-D: need stable content hashes and the generated-vs-user distinction (OAM-01). None changes Core types if the commit port exists. |
| **Impact on WorldGen** | P-A: the generator is only needed for new chunks. P-B/C/D: the generator version becomes part of the data and old versions must be runnable, baked or migrated (O-WG-2). |
| **Impact on Persistence** | The subject of this decision; see the table below. |
| **Impact on Server** | The server owns the journal and snapshots (AOD-13); clients hold no authoritative save. P-C/P-D make the journal order the server order. |
| **Impact on Mobile** | Write volume, storage growth, flush behaviour on suspend or kill and low-memory behaviour are device questions; no measurement exists. P-A has the largest write volume (Agent inference). |
| **Reversibility** | The direction is reversible while no format is frozen. A frozen, shipped format is expensive to change, so versioned schemas and a migration framework are required from the first release (roadmap Gate 5). |
| **Blocks** | The persistence contract rewrite (OI-4), Gate 5, replication of baselines (Gate 7), the answer for O-WG-2 (generator version policy). |
| **Owner decision required** | (1) Confirm that AOD-17 is read as P-D (journal primary, snapshots as optimisation), or choose another reading. (2) Approve a persistence spike. (3) Decide the local single-player store question (REF-23) together with OAM-03. (4) Decide the generator-version policy (pin / bake / migrate; recommendation: pin by default, bake on retirement). |

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

---

## 3. OAM-03 Embedded server

| Field | Content |
|---|---|
| **ID** | OAM-03 (OI-6) |
| **Current state** | AOD-05 (separate headless Rust server using the same Core) and AOD-14 (client is never the source of truth) are Approved. Whether single-player runs a server instance inside the client process is **not decided**. The spike has a loopback TCP server and an in-process runtime that does not follow the authority model. |
| **Options** | **S-A** single-player as an embedded server instance (same components, in-process transport). **S-B** a separate server process even for single-player. **S-C** local client-only mode (client holds authoritative state). |
| **Recommended** | **S-A** `Proposed`, with the conditions in 3.1. S-C is excluded by AOD-14 unless the owner amends it. |
| **Pros** | S-A: one authoritative code path, the same tests and the same Editing/replication interfaces for single-player and multiplayer; no second process on mobile; works offline. S-B: strongest isolation, simplest to reason about as a product ("always a server"). S-C: least work now, lowest runtime cost. |
| **Cons** | S-A: server and client share a process, memory and thermal budget on a phone (unmeasured); a crash takes both down; needs a clean in-process transport. S-B: a second process is difficult or forbidden on mobile (iOS) and conflicts with the no-child-process rule (AOD-12). S-C: two authority models, edits and persistence bypass the Editing service, contradicts AOD-14, and migration to multiplayer becomes a rewrite. |
| **Impact on Core** | S-A and S-B: none (Core is the same). S-C: needs a local-authority write path. |
| **Impact on WorldGen** | S-A: the service runs in process; threading budget is shared with meshing (mobile concern). S-B: separate budget. S-C: ad hoc. |
| **Impact on Persistence** | S-A: the embedded server owns the journal and snapshots locally (the local single-player store, REF-23, must be decided). S-B: same, in the other process. S-C: a client save, i.e. a client-authoritative save (not allowed). |
| **Impact on Server** | S-A: the server must be embeddable as a library with a transport abstraction (in-process channel and network). This is an additional requirement on the server host design. |
| **Impact on Mobile** | S-A: CPU, memory and battery of the server loop must fit the phone budget; Hypothesis, no measurement. S-B: on iOS a second process is not an option (Agent inference; to verify). S-C: lowest cost. |
| **Reversibility** | S-A to S-B (dedicated) is cheap if the interfaces stay transport-independent. S-C to either is expensive. |
| **Blocks** | The server host design (Gate 7 scoping), the local store decision (REF-23), Gate 6 local game structure, the prediction and reconciliation scaffolding. |
| **Owner decision required** | Approve S-A as the single-player model (or choose S-B), and approve the transition conditions in 3.1. Fact (upstream): Veloren embeds its server behind a `singleplayer` feature; its authority rules for that mode were not examined, so it is not evidence for Cube. |

### 3.1 Conditions to move from embedded to dedicated server (proposal)

Move a given world from an embedded instance to a dedicated server when **any** holds:

1. more than one remote participant must share the world (the world is hosted, not local);
2. the world must run while no client is open (persistent simulation, scheduled events);
3. the embedded instance exceeds the device budget on the reference devices in OAM-06 (memory, frame time or thermal) and cannot be brought inside it;
4. server-side anti-cheat or moderation is required for the world;
5. the world is migrated to a hosted environment by the owner or the player (export of seed, generator version and journal).

The migration is possible only if the persisted world is transport- and process-independent (OAM-02) and the Editing/replication interfaces are identical in both modes. **Hypothesis:** the same binary logic can serve both with an in-process channel replacing the socket; this needs the Gate 7 spike.

---

## 4. OAM-04 Renderer

| Field | Content |
|---|---|
| **ID** | OAM-04 |
| **Current state** | AOD-10 (Approved): `faces` is the reference renderer, `greedy` is experimental until a real mobile benchmark. `cubes` is a control mode. Fact (Cube, experimental): `greedy` is T-junction free and pixel-identical to `faces` in the compared scenes, with about 55% fewer triangles at the 5x5 area; all numbers are from a desktop (Windows, RTX 3070 Ti) or Android emulators. **There is no physical-device result.** |
| **Options** | **R-A** `faces`. **R-B** `greedy`. **R-C** a future hybrid renderer (for example a per-section or per-distance choice between `faces` and `greedy`; or `greedy` with AO or an atlas). |
| **Recommended** | Keep: `faces` = reference/default, `greedy` = experimental, hybrid = not started. **The mobile production decision is not taken without a physical device.** No new owner decision is needed now. |
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
| **Current state** | No licence policy for Cube is recorded. The reference study followed a clean-room rule as a self-imposed constraint. **Changing the licence policy needs a decision by the owner; this document only proposes wording.** |
| **Options** | **L-A** observation-only policy (below). **L-B** no study of third-party projects at all. **L-C** permit direct code reuse under compatible licences case by case. |
| **Recommended** | **L-A** `Proposed`. |
| **Pros** | L-A: keeps Cube's licence choice open and avoids GPL and provenance exposure while retaining the useful design observations. L-B: zero exposure. L-C: speed. |
| **Cons** | L-A: slower, needs discipline on identifiers and wording. L-B: loses cheap, valuable lessons. L-C: GPL code binds Cube to GPL terms; reverse-engineered code has provenance risk; needs legal review per item. |
| **Impact on Core** | None, except that no third-party algorithm may enter Core without review. |
| **Impact on WorldGen** | The generator must be original: algorithms from other projects (noise, layering) are not carried over without review. |
| **Impact on Persistence** | Formats must not mirror another project's file layout. |
| **Impact on Server** | The network layer (OAM-05) must check the licence of any external library. |
| **Impact on Mobile** | Store licensing and open-source notices for dependencies. |
| **Reversibility** | A published choice of Cube's own licence is hard to reverse; copied third-party code is hard to remove once distributed. |
| **Blocks** | Gate 0 exit, any third-party dependency on the roadmap, publication of the repository or builds. |
| **Owner decision required** | Approve or amend the policy; choose Cube's licence model when ready; decide whether studying the reverse-engineered project is acceptable at all (the policy below allows only README/tree-level observations until decided). |

### 7.1 Policy (Proposed wording; becomes effective only when the owner approves it)

1. **Veloren (GPL-3.0-or-later):** its source code is **not copied**, translated or closely paraphrased into Cube. Only architectural observations in our own words are recorded.
2. **CubeWorld-Reversal:** it is **not used as a source of code or assets**, nor as a specification or data format reference. It is derived from a proprietary game; its README states that the original rights belong to its creators.
3. Permitted: **architectural observations and links** (structure, ideas, trade-offs) with an evidence label (section 0).
4. **Names, interfaces and algorithms** of another project are not carried into Cube without a separate legal review by the owner. Class/function names of the reverse-engineered project are not recorded at all. Veloren file paths appear only as verification references.
5. Third-party dependencies are listed with their licences before adoption.
6. Any change to this policy is an owner decision recorded in the decision log.

### 7.2 Upstream sources (remote, read-only)

| Source | Link | State read |
|---|---|---|
| Veloren (GitLab) | https://gitlab.com/veloren/veloren | `master`, commit `7daa95fd53ae62aa185d31ad316d0cabfe2d70ac`, 2026-10-30; GPL-3.0-or-later |
| Veloren book (codebase structure) | https://book.veloren.net | page `contributors/developers/codebase-structure` |
| CubeWorld-Reversal (GitHub) | https://github.com/qad3n/CubeWorld-Reversal (tree: https://github.com/qad3n/CubeWorld-Reversal/tree/master/cube) | `master`: README, file tree, folder listings, four declaration-only headers; no implementation file read; no licence shown |

The study notes are `../research/veloren-notes.md`, `../research/cubeworld-reversal-notes.md` and `../research/reference-comparison.md`.

---

## 8. OAM-08 Missing sources

| Field | Content |
|---|---|
| **ID** | OAM-08 (OI-1, OI-2, OI-3, B-1, B-13) |
| **Current state** | Four inputs are absent from the repository (`audit-resolution-report.md` O-6, B-1, B-13). Architecture Update v1.0 is stored (`architecture-update-v1.0.md`). |
| **Options** | **X-A** the owner supplies the files. **X-B** the owner approves named, temporary assumptions per item (table below). **X-C** proceed without either (not recommended). |
| **Recommended** | **X-A** first. **X-B** only for the items marked "acceptable" below, each as a written, dated assumption with an expiry at supply of the file. |
| **Pros** | X-A: contracts can be cross-checked and approved. X-B: lets non-dependent work continue. |
| **Cons** | X-A: waits on the owner. X-B: risk of building on a wrong assumption; contracts approved on an assumption may need re-approval. X-C: unverifiable contracts. |
| **Impact on Core** | Voxel Architecture may constrain C-07…C-12; AU-1.0 original gates the identity of the module and layer grid. |
| **Impact on WorldGen** | The GDD/TDD defines what generation must deliver (biomes, rivers, scale); without it Gate 8 scope is open. |
| **Impact on Persistence** | The GDD/TDD may define save, offline and sharing requirements. |
| **Impact on Server** | The GDD/TDD defines player counts and multiplayer scope (Gates 7 and 9). |
| **Impact on Mobile** | The GDD/TDD defines device targets and performance expectations. |
| **Reversibility** | Supplying a file is always reversible. A contract approved on an assumption is reversible only by re-approval. |
| **Blocks** | See the table. |
| **Owner decision required** | Supply the four items or approve the named temporary assumptions. |

### 8.1 Per item

| Missing item | What is missing | What it blocks | Temporary owner-approved assumption | Contracts that must not be approved without the original |
|---|---|---|---|---|
| **GDD/TDD v0.3** | product requirements and technical design: platforms, scale, player counts, art direction, feature scope | scope of Gates 6, 8, 9; mobile targets and budgets (OAM-06); multiplayer scale (OAM-05, Gate 7); parts of the Voxel contracts | **Acceptable** for Gates 1 to 5 only: "AOD-01…19 are the complete product constraints" plus a stated list of scale assumptions (for example player count class) approved by the owner | gameplay-facing contracts; any contract whose behaviour depends on game scope; anything that sets player-count or content budgets |
| **Voxel Architecture v1.0** | the original voxel model specification (brick model, channels, layers, coordinate and material conventions) | cross-check of C-07…C-12 and the Core brick/material model; the chunk-as-Core-concept decision | **Acceptable, narrowly:** "the Rust spike's brick model (16^3 COW, 16-bit material) is the working model, without approval" for **spike and design work only** | C-07…C-12 and every P0 contract that freezes brick layout, material/channel model or coordinate conventions |
| **Geometry Processing Specification** | the original specification of derived data, bounds, validation and providers | PR-01…PR-08 and Processing behaviour; derived-data provider ownership (O-1) | **Not acceptable** for approval; acceptable only as a working draft for experiments | PR-01…PR-08 and any contract exposing derived data semantics |
| **AU-1.0 original** | the original approved Architecture Update; the repository holds a reconstructed copy, not byte-verified (B-13, partially closed) | the identity of the module and layer grid, ACR-001/002/003 (the copy states them), the dependency rules | **Acceptable** only if the owner explicitly confirms the reconstructed copy as the working text, recorded in writing; AU-1.0 itself is not rewritten (AOD-19) | any contract that cites an AU-1.0 clause not verifiable in the copy; any decision on ACR-001/002/003 |

---

## 9. Decisions that can be taken in parallel

Independent of each other (no shared prerequisite):

* OAM-07 licence policy, OAM-08 supply of sources, OAM-06 device acquisition and criteria, and the assignment of owners (OI-8).
* OAM-01 direction (C) and OAM-03 (embedded server) can be decided together; they only share the system-commit port.
* The spikes for OAM-01/02 (one combined journal/overlay spike) and OAM-05 (transport) are independent of each other.

Sequential: OAM-08 (Voxel Architecture, GDD) before any P0 approval; OAM-01 direction before the system-commit port is drafted; OAM-02 spike before any persistence format; OAM-05 spike before any protocol approval; physical-device data before the renderer default (OAM-04).

## 10. What each blocker waits on

| Blocker | Waits on |
|---|---|
| B-1 sources | owner supply (OAM-08) |
| B-3 ACRs | owner decision; the system-commit port may add an ACR (OAM-01) |
| B-6 revision rule | OAM-01 direction |
| OI-4 / NB-5 persistence alignment | OAM-02 decision and spike |
| NB-1 physical device | owner acquisition (OAM-06) |
| NB-2 iOS | macOS/Xcode access (OAM-06) |
| NB-3 licence policy | OAM-07 |
| NB-4 WorldGen direction | OAM-01 |
| OI-9 transport | OAM-05 spike |

## 11. The three decisions to take first (recommendation)

1. **OAM-08 missing sources:** supply the four items, or approve the named temporary assumptions. It is the only item that blocks P0 approval and the cross-check of the Voxel contracts, and nothing can replace the files.
2. **OAM-07 licence policy:** approve or amend section 7.1. It is cheap, it is a precondition for pinning dependencies in Gate 1, and it fixes the rules for everything that follows.
3. **OAM-01 direction C together with OAM-03 (embedded server):** the direction (not the layer model) lets the system-commit port and the transport-independent interfaces be drafted. It stays `Proposed` until the owner records it.

Not a decision but an action that should start at once: acquire a physical Android ARM64 device and a path to macOS/Xcode (OAM-06); every device-dependent decision waits on it.

## 12. Recording decisions

When the owner decides, the entry is added to `owner-architecture-decisions-v1.md` (or a successor decision log) with the date, the chosen option and any amendment. Only then does the status of a matrix row change to Approved. Nothing in this file is to be read as approved, and no other document is updated automatically.
