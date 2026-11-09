# Cube — World Persistence Contracts (Draft)

**Status:** DRAFT 0.1.0-draft, **Proposed text, nothing approved.** Not part of the 45 numbered contracts and not assigned to a tier; P0/P1 and the Draft P-01…P-07 in `persistence-contracts.md` are untouched. This file is the **reconciliation draft for OI-4** (alignment of persistence with AOD-17 and AOD-21); how it merges with P-01…P-07 is an owner/Integration Owner decision.
**Basis:** AOD-21 (hybrid model, load order `base generation → snapshot → journal tail`, confirmed compaction protocol), AOD-17, AOD-20, `revision-and-snapshot-semantics.md`.
**Approved vs not approved.** The *architecture* of AOD-21 (including the compaction protocol of WP-07) is Approved. **Binary formats, the concrete storage engine or database, exact record fields, file layout, hash algorithm and numeric limits are `Proposed` or `Spike Required`** and nothing below fixes them.
Field order as in `worldgen-contracts.md`. No language-specific signatures.

## WP-01 WorldRecord

* **Purpose:** identity and root of a persistent world.
* **Model:** `WorldId, WorldSeed (WG-01), GeneratorVersion (WG-02), SchemaVersion, WorldConfig (opaque, versioned), CreatedAt, ManifestRef`.
* **Rules:** an untouched world is fully described by this record. `WorldSeed` never changes; `GeneratorVersion` changes only through an explicit migration or bake (WP-09), never silently.
* **Open:** config content; container and engine (Spike Required).

## WP-02 Seed and GeneratorVersion binding

* **Rules:** every snapshot and journal record names the generator version it was produced against; recovery refuses a mismatch it cannot reconcile by policy (pin, bake or explicit migration).

## WP-03 JournalRecord

* **Purpose:** the durable, ordered record of user changes (and replayable recovery inputs).
* **Model:** `Sequence (gapless, per world), OperationId, ActorId, ServerTick, GeneratorVersion, SchemaVersion, Revision/epoch effect, Payload (granularity open), Checksum`. Records are appended; a record is valid only with its checksum.
* **Rules:** append-only; one logical operation is one record or a group with a commit marker; the journal survives sudden process termination; a **truncated last record is detected and ignored**, never repaired silently; records ≤ the active recovery point are redundant, not wrong.
* **Open:** payload granularity (voxel deltas versus operation intents; under the recommended C1 baseline the journal records block-level overrides and should carry the previous effective value so that undo can restore it), flush policy and its numbers, segment layout.

## WP-04 SnapshotRecord

* **Purpose:** an accelerator for loading a chunk set; never the only source of truth before verification.
* **Model:** `SnapshotId, chunk set (ChunkCoords), BaseGeneratorVersion, RevisionBoundary (last journal Sequence and revision/epoch included), ContentHash, SchemaVersion, Payload (format open)`.
* **Rules:** a snapshot states which generator version its base was and exactly which journal prefix it contains; it is immutable once written; it is **valid** only after verification (WP-07 step 4).

## WP-05 Manifest

* **Purpose:** the single atomic pointer to the current **recovery point**.
* **Model:** `ManifestEpoch (monotonic), ActiveSnapshots[], JournalPosition (first retained Sequence), RecoveryPointHash, PreviousRecoveryPoint (manifest epoch + snapshots), Checksum`.
* **Rules:** replaced only atomically; the previous manifest and its snapshots remain until the new one is durable and readable; a manifest never references a file that has not been fsynced and verified.
* **Open:** file format, whether two alternating manifests or rename-replace is used (Spike Required, platform atomicity: P-05).

## WP-06 Durability and verification protocol

* **Rules:** "fsync" means making the file contents and, where the platform needs it, the containing directory entry durable; the platform mechanism (and its behaviour on Android and iOS storage) is a Spike Required item. Verification means re-reading the written bytes, checking the checksum and content hash, and (for snapshots) checking replay equality.
* **Tests:** fault injection at every step boundary (see the crash table in WP-07); power-loss emulation on desktop; on-device run before approval.

## WP-07 Compaction (Approved architecture)

Sequence, per chunk set `S` and cut point `P` (a journal Sequence at a clean boundary):

1. **write snapshot** `N` for `S` at `P` to a temporary name;
2. **fsync snapshot**;
3. **verify snapshot:** checksum, content hash, and replay equality: `base generation(GeneratorVersion) + previous recovery point + journal up to P` yields the same content hash as `N`;
4. **atomically update manifest** (write new manifest naming `N`, `P` and the previous recovery point; rename into place);
5. **fsync manifest**;
6. **compact/truncate the journal prefix ≤ P only after steps 3 to 5 succeeded** (truncation by whole segments so that a partial step leaves only redundant records).

Rules:

* a snapshot is never the only source of truth before it is verified;
* the previous snapshot is kept until the new one is activated by the manifest;
* the journal tail is kept until the new recovery point is confirmed (durable manifest);
* **idempotent:** repeating compaction for the same `(S, P)` either finds the state already active or reproduces an identical snapshot (deterministic serialisation requirement) and changes nothing else;
* a retention rule for older recovery points (proposal: keep one previous recovery point until the next compaction succeeds) is `Proposed`;
* compaction runs off the commit path and may be cancelled; cancel leaves a recoverable state.

**Crash between any steps leaves a recoverable state:**

| Crash point | Visible state | Recovery |
|---|---|---|
| during 1 or after 1 | temporary snapshot only; manifest and journal unchanged | ignore and delete the orphan; recover from the old recovery point |
| after 2, during 3 | durable but unverified snapshot, not referenced | same as above (never used) |
| after 3, before 4 | verified snapshot, not referenced | may be reused by a repeated compaction or deleted |
| during 4 | old manifest or new manifest (atomic replace), never a mix | either is valid; old keeps the previous snapshot and the full tail |
| after 4, before 5 | new manifest possibly not durable | recovery sees old or new manifest; both consistent |
| after 5, before 6 | new recovery point durable; journal prefix still present | replay skips records ≤ P (idempotent); truncation is retried |
| during 6 | partially removed prefix segments | only redundant records are affected; retried |

## WP-08 Recovery (load order and checks)

* **Order:** read manifest, then **base generation** for each needed chunk with the world's `GeneratorVersion`, then apply the **snapshot**, then replay the **journal tail** (Sequence > snapshot boundary) through the system-commit port as `Recovery` actor (`SC-02`), preserving original OperationIds.
* **Mandatory checks:** manifest checksum and monotonic epoch; snapshot **hash**; snapshot **epoch and revision boundary** consistent with the journal (no gap or overlap in Sequence); **generator version** of base, snapshot and journal equals the world's (or a declared pin/bake/migration applies); journal record checksums.
* **Outcomes:** consistent state; or fall back to the previous recovery point with a report; or a controlled error with a stable code (`ManifestCorrupt`, `SnapshotHashMismatch`, `RevisionGap`, `GeneratorVersionMismatch`, `JournalCorrupt`). A damaged or unknown-version file is preserved (renamed) rather than overwritten; recovery never regenerates under a different generator version without an explicit policy.
* **Tests:** recovery without any snapshot; `snapshot + tail` equals full replay; truncated tail; corrupted snapshot falls back; version mismatch refused; double recovery is idempotent.

## WP-09 Migration and generator-version policy

* **Rules:** a change of `GeneratorVersion` never changes an existing world silently. For an old version the world uses **pin** (keep the old generator runnable), **bake** (write the affected generated chunks as data, then retire the generator) or an **explicit migration** (regenerate with the new version and re-apply the journal under an owner-approved conflict rule, with player-visible consequences). The choice is recorded in the WorldRecord. Schema migration (`MigrationDescriptor` P-07) is separate and versioned per record type.
* **Open:** which policy is the default (recommendation: pin by default, bake on retirement), conflict rule for migrate, who runs migrations.

## WP-10 Single-player store

* **Rules:** the embedded server uses the same persistence contracts as the dedicated server; there is no client-authoritative save (AOD-14, AOD-22). Location and engine of the local store are open (REF-23).

## Not approved

WP-01…WP-10 are Draft. Approved architecture: AOD-21 and the compaction protocol and recovery rules above. Not approved: every binary format, the storage engine or database, file layout, exact fields, hash algorithm, flush policy numbers, retention numbers, migration conflict rule.
