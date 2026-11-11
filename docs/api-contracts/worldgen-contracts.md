# Cube — WorldGen Contracts (Draft)

**Status:** DRAFT 0.1.0-draft, **Proposed text, nothing approved.** Not part of the 45 numbered contracts of [`README.md`](README.md) and not assigned to a tier (P0/P1 are untouched, AOD-18). No language-specific signatures.
**Basis:** AOD-09 (pure deterministic Rust generation), AOD-16 and AOD-20 (separate WorldGen/System-commit boundary, variant C3), AOD-21, [`docs/decisions/worldgen-core-boundary.md`](../decisions/worldgen-core-boundary.md). Writing generated data into Core is specified in [`system-commit-contracts.md`](system-commit-contracts.md); storage in [`world-persistence-contracts.md`](world-persistence-contracts.md); hosting in [`server-runtime-contracts.md`](server-runtime-contracts.md).
**Owner of the module:** unassigned (OI-8). **Layer:** not defined in AU-1.0; placement is part of ACR-004 (Pending).
Field order: Purpose · Model · Ownership · Lifetime · Access · Revision · Dirty · Threading · Cancel · Errors · Serialization · Compat · Tests · Open.

## 0. Role

WorldGen produces the **base world** as pure values. It has no write handle to Core. The only way generated data enters the authoritative asset is the Materialiser through the system-commit port ([`system-commit-contracts.md`](system-commit-contracts.md)). Everything marked "spike" below is evidence from the experimental `cube-worldgen` crate, not a decision.

## WG-01 WorldSeed

* **Purpose:** the root of determinism for one world.
* **Model:** an unsigned integer value; width proposed 64 bit (spike: 64 bit). No other state.
* **Ownership:** value; owned by the WorldRecord (`WP-01`). **Access:** read-only after world creation; never changed for an existing world.
* **Errors/Compat:** a seed never changes for an existing world; a different seed is a different world.
* **Tests:** same seed gives identical output; different seeds differ.
* **Open:** seed width and the derivation of sub-seeds per layer (terrain, structures, entities; REF-08 keeps entity randomness separate).

## WG-02 GeneratorVersion

* **Purpose:** identifies the exact generation algorithm and parameters.
* **Model:** an ordered identifier with a registry entry per released version (algorithm id, content hash of pinned reference outputs, status Active | Retired-Baked | Retired-Unavailable).
* **Rules:** any change that alters output **must** bump the version; a version bump without pinned-hash updates, or output change without bump, is a defect. A saved world records its version (`WP-01`); the version is never changed silently (AOD-21).
* **Tests:** version replay (an old version reproduces its pinned hashes); registry completeness for every version referenced by a saved world.
* **Open:** identifier format; how many versions stay runnable (pin) versus baked.

## WG-03 ChunkCoord

* **Purpose:** the address of a generation unit.
* **Model:** integer coordinates; conversion from world coordinates uses Euclidean division (negative coordinates correct). Spike: a horizontal pair (x, z) addressing a 16 x 32 x 16 column of two bricks.
* **Open:** column versus three-dimensional chunk, chunk height and vertical streaming (affects Gate 2 chunk concept), coordinate width.

## WG-04 GenerationRequest / GenerationResult

* **Purpose:** ask for one chunk and receive a value.
* **Model:** `GenerationRequest {WorldSeed, GeneratorVersion, ChunkCoord, Priority, CancellationToken, RequestId}`; `GenerationResult {RequestId, ChunkCoord, GeneratorVersion, ChunkHash, GeneratedChunk | GeneratorFailure}`. `GeneratedChunk` is a value (voxel data per brick, biome ids, optional metadata) with no reference to Core internals.
* **Rules:** pure function of `(seed, version, coord)`: integer-only world math, no I/O, no time, no global state, no dependence on neighbours except through pure functions of world coordinates (spike rule). Result order is irrelevant. Requests are deduplicated per `(coord, version)`.
* **Threading:** value-safe; may run on a bounded worker pool. **Revision/Dirty:** none (a value is not a commit).
* **Tests:** same input same output; order independence; alone versus in an area; neighbour border agreement; different seeds differ.

## WG-05 Cancellation

* **Purpose:** drop work for chunks that left the interest area.
* **Model:** a cooperative token polled at least once per column or brick; cancelled work returns `Cancelled` and produces no result; a result whose request is no longer pending is discarded by the service.
* **Tests:** cancel before start, during, after completion; no partial chunk is ever visible.

## WG-06 ChunkHash (deterministic hash)

* **Purpose:** detect any difference in generated content across versions and devices.
* **Model:** a stable digest over `(seed, version, coord, voxel content)` with a fixed byte order (spike: little-endian bytes, FNV-style). **The algorithm and digest width are not decided** (Spike Required: a collision-resistant hash may be needed once hashes gate trust between server and client).
* **Rules:** the hash is part of the contract; a server/client mismatch is a protocol-visible error, never accepted silently (worldgen-core-boundary section 6).
* **Tests:** pinned values per version; cross-device equality (desktop, Android ARM64, iOS) **not yet demonstrated**.

## WG-07 GeneratorFailure

* **Model:** `StableCode ∈ {Cancelled, GeneratorVersionUnknown, GeneratorVersionRetired, InvalidCoord, ResourceLimit, InternalFault}` with the common Error envelope ([`README.md`](README.md) convention 2).
* **Rules:** a failure marks only that chunk Failed; it never corrupts neighbours or authoritative state; retry policy belongs to the service; clients never see a half-built chunk.
* **Tests:** injected failure isolates to one chunk; error codes are stable.

## WG-08 ChunkState

* **Model:** `Unloaded → Queued → Generating → Ready → (Evicting → Unloaded)`; `Generating → Failed → Queued` (retry) and `Queued/Generating → Unloaded` (cancel). State is per chunk and server-side.
* **Rules:** transitions are driven by the interest set decided by the server (AOD-13); a chunk with user changes is not evicted before its changes are durable in the journal (`WP-03`).
* **Tests:** legal and illegal transitions; hysteresis between load and unload radius (REF-34, proposal).

## WG-09 Ownership of generated data

* **Rules:** the Generator owns nothing after it returns a value. The WorldGen service holds results only in a bounded cache. After the Materialiser commits a chunk (`SC-10`) the **Core asset** owns the authoritative copy; generated data is regenerable and is never persisted for untouched chunks (AOD-21). Evicting generated data never touches user changes. Clients may regenerate locally for display only if version and hash match the server's.
* **Open:** the Core layer model: **C1** (one logical layer, recommended first baseline) or **C2** (generated base plus overlay, Deferred), both awaiting owner approval (OAM-09); chunk delivery to clients (data versus seed + version).

## WG-10 WorldGenService port

* **Model:** `Request(GenerationRequest)`, `Cancel(RequestId)`, `Status(ChunkCoord)`, `SetInterest(set of ChunkCoord)`; results are delivered to the Materialiser; no write handle is exposed.
* **Authority:** server-side only; a client cannot invoke it (authority test).
* **Open:** queue and thread budgets per device class; owner of the module.

## Not approved

All of the above is Draft. Seed width, hash algorithm, chunk shape, version identifier format, the layer model and the module owner are open.
