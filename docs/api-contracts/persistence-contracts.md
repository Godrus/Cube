# Cube — Persistence Contracts

**Status:** DRAFT 0.1 — specified for approval; none approved.
**Implementing module:** Persistence Integration (Agent H). Basis: AU-1.0 §15–§18, §23; Spec §14.
Conventions: [`README.md`](README.md). Field order: Purpose · Model · Ownership · Lifetime · Access · Revision · Dirty · Threading · Cancel · Errors · Serialization · Compat · Tests · Open.

## 0. Boundary rules

- Persistence depends on Core (read/export views, import), Processing (async jobs, snapshots, cancellation) and Editing contracts (GeometryDelta, Checkpoint, ports). It does **not** depend on History or Orchestration implementations, and does **not** control editing (AU-1.0 §3).
- Core owns runtime storage; Persistence owns the **serialized format** of everything below. The Task Pack's "reuse Core serialization interfaces" is replaced by "use Core export/import views" (BrickStorage C-08) — see audit report.
- Runtime and serialized storage are separate. All saves work from snapshots and are asynchronous.

## P-01 AssetManifest

- **Purpose:** authoritative index of one Asset's persisted state.
- **Model:** `SchemaVersion, RequiredFeatures[], OptionalFeatures[], AssetId, Epoch, AssetToken, Layers[(LayerId, Type, LayerToken, Transform, ResolutionProfile, MaterialMapping, ChannelDescriptors, BrickIndex ref)], Junctions, CompositeReferences, Palettes, Dependencies, LastDeltaId, LastCheckpointId, ContentHash`.
- **Ownership:** Persistence. **Lifetime:** immutable per saved generation; new generation per save.
- **Rules:** never references missing/corrupt Brick data (verified before the manifest generation becomes current).
- **Revision:** stores tokens of the saved cut; recovery bumps Epoch.
- **Threading:** written by one writer per asset; readers see whole generations only.
- **Errors:** `PersistenceFailure`, unknown required feature ⇒ controlled `IncompatibleVersion`-class error (category to add under `PersistenceFailure`). **Compat:** unknown optional features preserved byte-for-byte.
- **Tests:** manifest/brick consistency after crash at each step; unknown-feature handling.
- **Open:** container layout (file vs directory vs database), serialization format (OD in AU-1.0 §24 #10).

## P-02 BrickSerialization

- **Model:** `BrickKey, BrickRevision, ChannelPayloads[(ChannelId, StorageMode, Codec, Bytes)], ContentHash`. Chunked storage; identical content deduplicated by hash.
- **Rules:** exact integer coordinates; unknown channels preserved opaque; decode validates hash.
- **Ownership:** bytes owned by the store; decoded data handed to Core via import view. **Tests:** round-trip for sparse/dense/constant; dedup correctness; hash mismatch ⇒ `LoadFailure` state.
- **Open:** compression combination (RLE/bit-pack/dictionary/LZ4/Zstd), index order (benchmark-driven).

## P-03 DeltaSerialization

- **Model:** versioned encoding of E-09 `GeometryDelta`; delta log segments with `LastDeltaId` chain and per-segment hash.
- **Implements:** `IDeltaStore`. **Rules:** append-only; truncated tail detected and ignored safely; replay validates `PreTokens` chain.
- **Tests:** replay equals live state; torn write recovery; migration of delta schema.
- **Open:** segment size / flush policy; fsync/durability level.

## P-04 SnapshotSerialization

- **Model:** durable form of E-10 `Checkpoint`: manifest generation + brick set reference at a consistent cut.
- **Implements:** `ISnapshotStore`. **Rules:** produced from a ProcessingSnapshot (no blocking of editing); superseded only after verification.
- **Tests:** recovery from snapshot + deltas; snapshot during concurrent edits is consistent.
- **Open:** incremental vs full snapshots.

## P-05 StorageTransaction

- **Purpose:** atomic multi-part commit protocol (manifest + bricks + deltas).
- **Model:** `StorageTxId, PreparedParts[], State (Preparing | Prepared | Committed | Aborted)`, commit marker.
- **Proposed protocol (not approved):** write new bricks and delta segments under their final content-addressed names → fsync → verify hashes → write new manifest generation under temporary name → fsync → atomically publish via rename/commit marker → fsync directory. Readers only follow the committed marker; unreferenced parts are garbage-collected after a safe delay.
- **Rules:** a manifest generation is never published before all referenced parts are durable. Cancellation before the commit marker aborts cleanly; after it, the transaction is committed.
- **Errors:** `PersistenceFailure`, `ProcessingCancelled`. **Tests:** crash-injection at every step; concurrent save+load; disk-full.
- **Open:** atomic publish primitive per platform (mobile file systems); multi-asset (composite) consistency group commit.

## P-06 IntegrityReport

- **Model:** `Scope (Brick|Channel|Manifest|Asset|Composite), Findings[(Severity, Code, Location, RecoveryPolicy)], Verdict (Ok | Repairable | Corrupt)`.
- **Rules:** produced by load-time and save-time checks (Fast / Deep levels per AU-1.0 §18); repair actions are explicit and never silent. **Tests:** each corruption class detected; no silent data loss.
- **Open:** repair automation level.

## P-07 MigrationDescriptor

- **Model:** `FromVersion, ToVersion, RequiredFeaturesDelta, Steps[], LossReport, Reversible?`. Sequential migrations only; each step deterministic and tested; any loss reported in `LossReport` and never silent.
- **Tests:** v(n)→v(n+1) chain on corpus; idempotence; unknown optional features survive.
- **Open:** migration execution (online vs offline), downgrade policy.

## P-08 Composite persistence

- Parts store **references** (AssetId + expected token/hash), not copies; Assembly Graph and part hierarchy persist in the composite Asset's manifest. Group commit across referenced assets is Open (P-05).

## P-09 Import / Export adapters (AU-1.0 §17)

- Separate adapters, outside native format: coordinate/unit/transform conversion, material and attribute mapping, validation, capability check, **loss report**. Owner **unassigned** (audit report O-4).
