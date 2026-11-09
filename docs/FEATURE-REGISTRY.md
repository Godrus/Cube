# Cube feature registry

> GENERATED from `docs/features.json` by `cargo xtask features --write`. Do not edit by hand; CI fails if this file is out of date.

Three separate dimensions: **Decision** (IDEA, PROPOSED, APPROVED, DEFERRED, REJECTED), **Implementation** (NOT_STARTED, IN_PROGRESS, IMPLEMENTED, TESTED, INTEGRATED) and **Player availability** (NOT_AVAILABLE, INTERNAL_ONLY, DEMO_AVAILABLE, PLAYER_AVAILABLE). *Blocked* is a flag with blocker ids, not a status. Integrated code is not a player feature. Ids are stable and never reused.

**45 features.** Integrated: 27. Player available: 0. Approved: 31. Proposed: 14. Blocked: 12.

## Geometry Core

| ID | Feature | Decision | Implementation | Player | Prio | Depends | Gates | Evidence | Blockers | Verified |
|---|---|---|---|---|---|---|---|---|---|---|
| F-CORE-001 | **Voxel coordinates, brick keys and regions**: i64 voxel and brick coordinates, floor division for negative space, cell keys with level, half-open regions. Acceptance: Negative coordinates symmetric; no overflow wrap; deterministic ordering. | APPROVED | INTEGRATED | INTERNAL_ONLY | P0 |  | PS-1; G1 | CORE-01/02 (dev d0a1ec8, DEC-076); crates/cube-core/tests/coordinates.rs | - | `f994c83` |
| F-CORE-002 | **Sparse brick storage with explicit absent-data states**: Cell store with Present, KnownEmpty, NotLoaded, OutsideBounds and LoadFailure never merged. Acceptance: Unloaded or failed data is never reported as empty. | APPROVED | INTEGRATED | INTERNAL_ONLY | P0 | F-CORE-001 | PS-1; G1 | CORE-03 (dev d0a1ec8); crates/cube-core/tests/cells.rs | - | `f994c83` |
| F-CORE-003 | **Revisions, epochs and region stamps**: Monotonic per-resource revisions with epochs, incomparable-is-stale rule, region stamps. Acceptance: Overflow is an error; stamps are order independent. | APPROVED | INTEGRATED | INTERNAL_ONLY | P0 | F-CORE-001 | PS-1; G1 | CORE-05 (dev d0a1ec8); revision.rs goldens | - | `f994c83` |
| F-CORE-004 | **Atomic commit gate and mutation boundary**: Single commit primitive reachable only through user, system and recovery handles; batches apply entirely or not at all. Acceptance: No direct write path; loom model of the gate; compile-fail bypass tests. | APPROVED | INTEGRATED | INTERNAL_ONLY | P0 | F-CORE-003 | PS-1; G1 | CORE-06/08 (dev d0a1ec8); tests/loom_gate.rs; tests/ui/* | - | `f994c83` |
| F-CORE-005 | **Export/import views and stable content hash**: Plain-data view of an asset and a content fingerprint independent of encoding. Acceptance: Round trip restores revisions and content; hash independent of channel encoding. | APPROVED | INTEGRATED | INTERNAL_ONLY | P0 | F-CORE-002 | PS-1; G1 | CORE-11 (dev d0a1ec8); tests/views.rs | - | `f994c83` |
| F-CORE-006 | **Geometry bounds**: Checked integer boxes: union, intersection, growth, level conversion, transform; no world-size limit. Acceptance: Overflow is an error; property tests for the algebra. | APPROVED | INTEGRATED | INTERNAL_ONLY | P1 | F-CORE-001 | PS-2; G2 | CORE-15 (dev 377ad25, DEC-079); crates/cube-core/src/bounds.rs | - | `f994c83` |
| F-CORE-007 | **World identity values**: WorldSeed and GeneratorVersion newtypes with fixed-width byte forms. Acceptance: Exact comparison; stable byte form. | APPROVED | INTEGRATED | INTERNAL_ONLY | P1 |  | PS-2; G2 | CORE-16 (dev b50bdd0) | - | `f994c83` |
| F-CORE-008 | **Material palette (content ids to runtime indices)**: Namespaced content ids, runtime indices with air = 0, collision diagnostics, strict and recovery-mode resolution of saved tables. Provisional (C-10). Acceptance: Unknown content is an error or a visible placeholder, never another material. | APPROVED | INTEGRATED | INTERNAL_ONLY | P1 | F-CORE-002 | PS-5; RG-1 | CORE-12 (dev 718e29b, CI 42642936047); DEC-090 | - | `f994c83` |

## Processing

| ID | Feature | Decision | Implementation | Player | Prio | Depends | Gates | Evidence | Blockers | Verified |
|---|---|---|---|---|---|---|---|---|---|---|
| F-PRC-001 | **Immutable snapshots with apron**: Pin-only brick neighbourhood snapshots with per-brick input stamps. Acceptance: Snapshot isolation under concurrent commits; apron consistency. | APPROVED | INTEGRATED | INTERNAL_ONLY | P0 | F-CORE-002; F-CORE-003 | PS-2; G2 | PROC-01 (dev 4436df3); tests/processing_snapshot.rs | - | `f994c83` |
| F-PRC-002 | **Derived-data executor with stale-result rejection and caps**: Provider graph, task state machine, cancellation, single publish path that rejects stale results, pin and lifetime budgets. Acceptance: No stale result is ever published (loom, stress); caps enforced on the synthetic mobile budget. | APPROVED | INTEGRATED | INTERNAL_ONLY | P0 | F-PRC-001 | PS-2; G2 | PROC-02..06, TEST-08 (dev d164d21, DEC-081); tests/processing_loom.rs | - | `f994c83` |

## Editing

| ID | Feature | Decision | Implementation | Player | Prio | Depends | Gates | Evidence | Blockers | Verified |
|---|---|---|---|---|---|---|---|---|---|---|
| F-EDT-001 | **Editing API contracts**: Operations, transactions, results, conflicts, authorization context, backend and storage ports. Acceptance: No behaviour in the crate; API snapshot guards changes. | APPROVED | INTEGRATED | INTERNAL_ONLY | P0 | F-CORE-004 | PS-3; G3 | API-01..05 (dev 1f588c9, DEC-082..087); crates/cube-editing-api/tests/contracts.rs | - | `f994c83` |
| F-EDT-002 | **Transactional editing through one user commit path**: Authorization, backend dispatch, private overlay, expected-revision conflicts, atomic commit, rollback, cancel. Acceptance: A failed or conflicting transaction changes nothing; only user commits arise from user requests. | APPROVED | INTEGRATED | INTERNAL_ONLY | P0 | F-EDT-001 | PS-4; G4 | ORCH-01..05, TEST-03a, TEST-10 (dev 5c6653e, DEC-088); tests/orchestration_path.rs; tests/mutation_negative.rs | - | `f994c83` |
| F-EDT-003 | **Idempotent retry by operation id, including after a restart**: Committed operations are not applied twice; the retry table is rebuilt from the journal and bounded safely; undecided intents block retries until resolved. Acceptance: Confirmed op not re-applied after restart; same id with other content refused. | APPROVED | INTEGRATED | INTERNAL_ONLY | P0 | F-EDT-002; F-PER-003 | PS-4; PS-10 | TD-4 (dev 22d08c4, f994c83); tests/orchestration_restart.rs; tests/persistence_replay.rs | - | `f994c83` |
| F-EDT-004 | **Observers and dirty propagation after commit**: Dirty regions go to Processing, then observers in registration order; observer failures are isolated. Acceptance: Observers see post-commit state only; order equals commit order. Known limit: callbacks run inside the commit section (TD-1). | APPROVED | INTEGRATED | INTERNAL_ONLY | P1 | F-EDT-002; F-PRC-002 | PS-4; G4 | ORCH-04 (dev 5c6653e); tests/orchestration_durability.rs | - | `f994c83` |

## Geometry Backends

| ID | Feature | Decision | Implementation | Player | Prio | Depends | Gates | Evidence | Blockers | Verified |
|---|---|---|---|---|---|---|---|---|---|---|
| F-BKD-001 | **Voxel editing operations**: Set, replace, fill air, clear, copy and move (with rotation) of voxel regions as write sets, at any level, with no world-size limit. Acceptance: Unloaded bricks never guessed; other channels preserved; brute-force model equality. | APPROVED | INTEGRATED | INTERNAL_ONLY | P0 | F-EDT-002; F-CORE-002 | PS-5; G5 | VOX-01 (dev bf8d244, CI 42633356471); tests/voxel_backend.rs | - | `f994c83` |
| F-BKD-002 | **Faces mesh provider (reference oracle)**: Visible unit faces per brick with apron; the correctness reference for meshing. Acceptance: Equals the brute-force definition across brick borders. | APPROVED | INTEGRATED | INTERNAL_ONLY | P0 | F-BKD-001; F-PRC-002 | PS-5; G5 | VOX-02 (dev bf8d244); tests/voxel_providers.rs | - | `f994c83` |
| F-BKD-003 | **Greedy mesh provider**: Material-aware merging of coplanar faces with a unit-ring seam rule; optimisation candidate, not the default renderer. Acceptance: Coverage equals faces; no T-junctions. Known limit: T-junctions inside a brick are not claimed fixed. | APPROVED | INTEGRATED | INTERNAL_ONLY | P2 | F-BKD-002 | PS-5; G5 | VOX-03 (dev bf8d244) | - | `f994c83` |
| F-BKD-004 | **Incremental mesh rebuild by a dirty rule**: After a commit only the brick and its six face neighbours need new meshes. Acceptance: Incremental rebuild equals full rebuild over random edits. | APPROVED | INTEGRATED | INTERNAL_ONLY | P1 | F-BKD-002 | PS-5; G5 | VOX-04 (dev bf8d244) | - | `f994c83` |
| F-BKD-005 | **Voxel collision boxes**: Disjoint boxes covering exactly the solid voxels of a brick; unknown is never free space. Acceptance: Boxes equal the voxel model; stale results never served. | APPROVED | INTEGRATED | INTERNAL_ONLY | P1 | F-BKD-001; F-PRC-002 | PS-5; G5 | VOX-05 (dev baaff7f, CI 42636955092); tests/voxel_collision_lod.rs | - | `f994c83` |
| F-BKD-006 | **One-step voxel LOD**: A coarser derived level per brick; presence of thin structures survives. Acceptance: Equals brute-force coarsening; authoritative data untouched. Mixed-resolution neighbours belong to PS-9. | APPROVED | INTEGRATED | INTERNAL_ONLY | P2 | F-BKD-001 | PS-5; PS-9 | VOX-05 (dev baaff7f) | - | `f994c83` |
| F-BKD-007 | **SDF backend**: Technical SDF representation behind the backend port; not a required building mechanic. Acceptance: SDF-01..03 pass backend conformance; no implicit voxel to SDF conversion. | APPROVED | NOT_STARTED | NOT_AVAILABLE | P2 | F-EDT-001 | PS-5; G5 |  | - | `f994c83` |
| F-BKD-008 | **Mesh backend**: Mesh pages with validation and normal recalculation behind the backend port. Acceptance: MSH-01..03 pass backend conformance; typed topology errors. | APPROVED | NOT_STARTED | NOT_AVAILABLE | P2 | F-EDT-001 | PS-5; G5 |  | - | `f994c83` |
| F-BKD-009 | **Backend conformance suite**: Generic suite every backend must pass, self-checked against broken fakes. Acceptance: Nine broken fakes are each caught; voxel backend passes. | APPROVED | INTEGRATED | INTERNAL_ONLY | P0 | F-EDT-001 | PS-4; G4 | TEST-03a (dev 5c6653e); tests/backend_conformance.rs | - | `f994c83` |

## Persistence

| ID | Feature | Decision | Implementation | Player | Prio | Depends | Gates | Evidence | Blockers | Verified |
|---|---|---|---|---|---|---|---|---|---|---|
| F-PER-001 | **Versioned checksummed record codec (experimental)**: Asset snapshots, journal records and write batches with schema version, CRC-32 and size limits. Not an approved format. Acceptance: Every byte flip and truncation detected; hostile counts refused without allocation. | APPROVED | INTEGRATED | INTERNAL_ONLY | P0 | F-CORE-005 | PS-10 | PERS-01 (dev baaff7f); crates/cube-persistence/src/codec.rs | - | `f994c83` |
| F-PER-002 | **Atomic snapshot save and load**: Temp file, fsync, rename, directory sync; load verifies the checksum. Acceptance: Every modelled crash point leaves a complete old or new snapshot. Model and real-directory evidence only; no device durability (DG-2). | APPROVED | INTEGRATED | INTERNAL_ONLY | P0 | F-PER-001 | PS-10 | dev baaff7f; tests/persistence_world.rs | - | `f994c83` |
| F-PER-003 | **Write-ahead journal with payload and replay**: Intent/Confirmed/Aborted phases, write payload, reconciliation and replay of confirmed changes after the last snapshot. Acceptance: Replay is idempotent and reproduces the pre-crash world. The journal is a contract fake in tests: no file journal yet; CB-1 (commit and journal not atomic) is open. | APPROVED | INTEGRATED | INTERNAL_ONLY | P0 | F-PER-001; F-EDT-002 | PS-10 | dev f994c83, CI 42644198701; tests/persistence_replay.rs | - | `f994c83` |
| F-PER-004 | **Durable journal file, manifest, snapshot store, compaction**: File-backed operation journal, world record (seed and generator version), manifest, snapshot store and compaction keeping the last recoverable state. Acceptance: PERS-02..05 integrated; crash-recovery matrix passes; generator version never changed silently. | APPROVED | NOT_STARTED | NOT_AVAILABLE | P0 | F-PER-003 | PS-10; G10 |  | - | `f994c83` |
| F-PER-005 | **Crash-point fault-injection harness**: In-memory filesystem with a weakest-POSIX model and a runner that crashes a scenario after every step. Acceptance: Finds a missing fsync; a model is not proof of real platform behaviour. | APPROVED | INTEGRATED | INTERNAL_ONLY | P1 |  | PS-10 | TEST-04 (dev bf8d244); crates/cube-testkit/src/faultfs | - | `f994c83` |
| F-PER-006 | **Final save format**: The approved on-disk format of worlds, journals and snapshots. Acceptance: GOV-10 approved after RG-1. | PROPOSED | NOT_STARTED | NOT_AVAILABLE | P1 | F-PER-004 | PS-10; RG-1 |  | GOV-10; RG-1 | `f994c83` |

## World Generation

| ID | Feature | Decision | Implementation | Player | Prio | Depends | Gates | Evidence | Blockers | Verified |
|---|---|---|---|---|---|---|---|---|---|---|
| F-WGN-001 | **Create a world from a seed**: Deterministic pure generator producing terrain for a seed and generator version, materialised by the system writer. Acceptance: Same seed and version give the same world on every platform. Product detail (terrain variants) still needs owner approval. | PROPOSED | NOT_STARTED | NOT_AVAILABLE | P0 | F-CORE-007; F-BKD-001 | PS-11 |  | WorldGen owner unassigned (DEC-072); GOV-21 | `f994c83` |

## Rendering and Client

| ID | Feature | Decision | Implementation | Player | Prio | Depends | Gates | Evidence | Blockers | Verified |
|---|---|---|---|---|---|---|---|---|---|---|
| F-CLI-001 | **See the world in a game window**: Client renders derived meshes of the loaded region. Acceptance: Mesh of a generated region visible on desktop. | PROPOSED | NOT_STARTED | NOT_AVAILABLE | P0 | F-BKD-002; F-PRC-002 | PS-12 |  | Client Runtime owner unassigned (DEC-072); Engine Adapter owner unassigned (O-5) | `f994c83` |
| F-CLI-002 | **Camera and character control**: Move a camera or a character through the world with voxel collision. Acceptance: Walk and jump on generated terrain without falling through. | PROPOSED | NOT_STARTED | NOT_AVAILABLE | P0 | F-CLI-001; F-BKD-005 | PS-12 |  | Client Runtime owner unassigned (DEC-072); gameplay model open (REF-13) | `f994c83` |
| F-CLI-003 | **Player places and removes a block**: Input in the game window becomes an editing operation through the production chain. Acceptance: A placed block is visible, authoritative and survives save and load. | PROPOSED | NOT_STARTED | NOT_AVAILABLE | P0 | F-CLI-001; F-EDT-002; F-BKD-001 | PS-12 |  | Client Runtime owner unassigned (DEC-072) | `f994c83` |

## Construction Tools

| ID | Feature | Decision | Implementation | Player | Prio | Depends | Gates | Evidence | Blockers | Verified |
|---|---|---|---|---|---|---|---|---|---|---|
| F-TLS-001 | **Sculpt brush**: One sculpt stroke is one transaction. Acceptance: Stroke commits atomically (DEC-084); tool behaviour needs product detail. | PROPOSED | NOT_STARTED | NOT_AVAILABLE | P2 | F-EDT-002 | PS-6 |  | - | `f994c83` |
| F-TLS-002 | **Boolean operations on regions**: Boolean tools over the editing API. Acceptance: Needs product detail. | PROPOSED | NOT_STARTED | NOT_AVAILABLE | P3 | F-EDT-002 | PS-6 |  | - | `f994c83` |

## History

| ID | Feature | Decision | Implementation | Player | Prio | Depends | Gates | Evidence | Blockers | Verified |
|---|---|---|---|---|---|---|---|---|---|---|
| F-HST-001 | **Undo and redo**: History of confirmed operations with one stroke as one unit. Acceptance: Undo respects authorization; History does not own journal or storage. | PROPOSED | NOT_STARTED | NOT_AVAILABLE | P1 | F-EDT-002; F-PER-003 | PS-7 |  | GOV-25 | `f994c83` |

## Resolution and Microvoxels

| ID | Feature | Decision | Implementation | Player | Prio | Depends | Gates | Evidence | Blockers | Verified |
|---|---|---|---|---|---|---|---|---|---|---|
| F-RES-001 | **Level-aware voxel editing (micro-voxel levels)**: Keys, storage, operations and providers work at any non-negative level. Acceptance: An operation at level 2 touches only level 2 data. | APPROVED | INTEGRATED | INTERNAL_ONLY | P1 | F-CORE-001; F-BKD-001 | PS-5; PS-9 | tests/voxel_backend.rs micro_voxel_levels_use_the_same_operations_at_their_own_level | - | `f994c83` |
| F-RES-002 | **Selective micro-voxel detail and resampling**: Promote parts of a brick to finer levels and convert between resolutions. Acceptance: Mixed-resolution neighbours consistent. | PROPOSED | NOT_STARTED | NOT_AVAILABLE | P2 | F-RES-001 | PS-9 |  | CORE-13; RG-1 | `f994c83` |

## Composite Geometry

| ID | Feature | Decision | Implementation | Player | Prio | Depends | Gates | Evidence | Blockers | Verified |
|---|---|---|---|---|---|---|---|---|---|---|
| F-CMP-001 | **Composite geometry**: Assemblies of layers and junctions. Acceptance: Needs product detail. | PROPOSED | NOT_STARTED | NOT_AVAILABLE | P3 | F-EDT-002 | PS-8 |  | GOV-19; GOV-23 | `f994c83` |

## Server and Multiplayer

| ID | Feature | Decision | Implementation | Player | Prio | Depends | Gates | Evidence | Blockers | Verified |
|---|---|---|---|---|---|---|---|---|---|---|
| F-SRV-001 | **Authoritative server runtime**: Single-writer authoritative world with authorization at the server. Acceptance: Needs product detail; permission schema owner is Server/Authorization (DEC-083). | PROPOSED | NOT_STARTED | NOT_AVAILABLE | P2 | F-EDT-002; F-PER-004 | PS-13 |  | Server Runtime owner unassigned (DEC-072) | `f994c83` |
| F-SRV-002 | **Multiplayer synchronisation**: Replication of operations and deltas to clients. Acceptance: Needs product detail. | PROPOSED | NOT_STARTED | NOT_AVAILABLE | P3 | F-SRV-001 | PS-13 |  | Networking owner unassigned (O-3); GOV-22 | `f994c83` |

## Gameplay

| ID | Feature | Decision | Implementation | Player | Prio | Depends | Gates | Evidence | Blockers | Verified |
|---|---|---|---|---|---|---|---|---|---|---|
| F-GPL-001 | **Character, inventory, crafting, combat and creatures**: RPG systems proposed for the first playable slice (character, items, resources, one hostile and one passive creature, minimal combat). Acceptance: Product detail and a gameplay owner are still required; this entry is a placeholder for the later split into separate features. | PROPOSED | NOT_STARTED | NOT_AVAILABLE | P1 | F-CLI-002 | PS-12 |  | gameplay owner unassigned (OI-8); M2-T scope not confirmed | `f994c83` |

## Content and Mods

| ID | Feature | Decision | Implementation | Player | Prio | Depends | Gates | Evidence | Blockers | Verified |
|---|---|---|---|---|---|---|---|---|---|---|
| F-MOD-001 | **Content packages and mods**: Registration of materials and content by package; later loading of mods. Acceptance: Palette model exists (F-CORE-008); no Mod SDK is planned yet. | PROPOSED | NOT_STARTED | NOT_AVAILABLE | P3 | F-CORE-008 | PS-14 |  | - | `f994c83` |

## Platforms and QA

| ID | Feature | Decision | Implementation | Player | Prio | Depends | Gates | Evidence | Blockers | Verified |
|---|---|---|---|---|---|---|---|---|---|---|
| F-PLT-001 | **Architecture and quality checks in CI**: fmt, clippy, tests on Linux and Windows, loom, cargo-deny, archcheck with mutation-boundary and conflict-marker rules, API snapshots. Acceptance: Required checks block integration into dev. | APPROVED | INTEGRATED | INTERNAL_ONLY | P0 |  | PS-0; G0 | .github/workflows/ci.yml; DEC-073 | - | `f994c83` |
| F-PLT-002 | **Mobile device validation**: Memory, determinism, durability and release checks on physical devices. Acceptance: Device gates pass on a physical device matrix; CI-host results are never device measurements. | APPROVED | NOT_STARTED | NOT_AVAILABLE | P1 | F-CLI-001 | DG-1; DG-2; DG-3 |  | EXT-DEV (physical devices) | `f994c83` |

