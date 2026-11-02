# Cube — Revision & Snapshot Semantics

**Status:** PROPOSED (Draft 0.1). Not approved. Basis: Architecture Update v1.0 (AU-1.0) §7, §9, §10, §11.
**Owner:** Integration Owner (semantics); Core Agent (primitives); Processing Agent (validation protocol).
**Open decisions:** OD-R1…OD-R8 (end of document).

Everything below is a technical proposal for approval. Nothing here is an approved contract until recorded in `docs/decisions/`.

---

# 1. Principles

1. A revision is a **monotonic counter scoped to one resource**, never a timestamp and never comparable across scopes.
2. Revisions only increase. **Undo is a new forward commit**, never a revision rollback.
3. A revision changes only when authoritative data **actually changes** (AU-1.0 §7.1).
4. The hierarchy is cheap to check from the top and precise at the bottom: Asset → Layer → Brick.
5. Operation ID identifies an operation; it never substitutes for a revision (AU-1.0 §7).

# 2. Revision Levels

| Level | Scope key | Incremented when | Stored |
|---|---|---|---|
| Asset Revision | AssetId | Any committed transaction that changes anything in the Asset (layers added/removed, junctions, composite refs, any layer content) | Yes |
| Layer Revision | LayerId | Any committed transaction that changes the Layer (bricks, transform, resolution, modifiers, palette mapping, metadata) | Yes |
| Brick Revision | BrickKey | A committed write that changed that Brick's payload (any channel) | Yes (per brick) |
| Region Revision | Region (set of BrickKeys) | **Derived, not stored.** `RegionStamp` = the set of `(BrickKey, BrickRevision)` for bricks intersecting the Region | No |
| Dependency Revision | Derived-data slot | A derived result is published; records the exact input stamps it consumed | Yes (in Processing; optionally cached) |
| Operation ID | OperationId | Never "incremented"; allocated once per submitted operation | Journal only |

**Commit increments are exactly +1 per scope per transaction**, regardless of how many bricks changed. A transaction that changes 40 bricks in a Layer yields 40 brick increments, +1 Layer, +1 Asset (single commit, atomic).

Invariant (checked by property tests): `Brick changed ⇒ Layer changed ⇒ Asset changed`, within the same commit.

## 2.1 RevisionToken (data model, conceptual)

```
RevisionToken
  Scope        : Asset | Layer | Brick | Region(stamp) | Derived
  ResourceId   : AssetId | LayerId | BrickKey | RegionStamp | DerivedSlotId
  Epoch        : integer  -- changes when a resource is re-created or recovered from older state
  Value        : unsigned integer (width: Open Decision OD-R1)
```

Comparison rules:

- Two tokens are comparable only if `Scope`, `ResourceId` and `Epoch` are equal.
- Same triple: `Value` equality means "unchanged"; greater means "newer". Smaller than current means "stale".
- Different `Epoch` ⇒ result is `Incomparable`, which every consumer must treat as stale. Comparison returns a tri-state (Equal / Newer / Older) plus Incomparable. A boolean compare is not part of the contract.
- **Local prediction namespace:** client-predicted revisions carry `Epoch = LocalPrediction` and are never compared with server tokens.

# 3. Snapshot Capture

A `GeometrySnapshot` / `ProcessingSnapshot` is captured as one consistent cut:

1. Take the Asset commit gate in **shared** mode (blocks only the very short commit swap, not editing work).
2. Read the Layer token and the token of every Brick in the requested Region plus the dependency-expanded Processing Region.
3. Pin each Brick by a reference-counted immutable reference (copy-on-write handle owned by Core `BrickStorage`).
4. Record `DependencyRevisions` for non-brick inputs (transform, resolution profile, palette mapping, modifier stack, other layers for junction/boolean inputs).
5. Release the gate. Snapshot is now immutable and independent of later commits.

Snapshot contents: `AssetId, LayerId, LayerToken, BrickPins[(BrickKey, BrickToken, handle)], DependencyRevisions[], OperationContext?, CreatedAt (diagnostic only), LifetimePolicy`.

Lifetime: reference-counted; released on job completion/cancellation/failure. Bounded by max age and memory budget (Open Decision OD-R4). Leaked pins are a defect; Core exposes a pin-count diagnostic.

MVP allowance (AU-1.0 §9): small affected regions may be **copied** instead of pinned. The observable contract (immutability, consistent cut) is identical.

# 4. Validation of Currency

## 4.1 Transaction (write path)

A transaction declares: `ReadRegions`, `WriteRegions`, `ExpectedTokens` (stamps captured when it began or from the caller), and optional `LayerWideDependencies` (transform, resolution, topology changes).

At commit, under the exclusive Asset commit gate:

1. For each Brick in `ReadRegions ∪ WriteRegions`: current brick token must equal expected → otherwise **Conflict**.
2. For each declared Layer-wide dependency: current Layer-level token must equal expected.
3. A mismatch of the Layer or Asset token alone is **not** a conflict (that is exactly what region-based concurrency allows).
4. If all pass: swap in new Brick versions (copy-on-write), increment tokens (§2), collect `DirtyRegion`s, append to the operation journal, notify the dependency graph. All-or-nothing.

The exclusive gate covers only validation + pointer swap + increments. Operation computation happens before it, against a snapshot.

## 4.2 Derived result (publish path)

A `ProcessingResult` carries the `DependencyRevision` (exact input stamps) it was computed from. Publishing:

1. Under the **shared** gate, re-read current tokens for exactly the input set.
2. All equal ⇒ publish. Any differ ⇒ result is **Stale**, discarded, resources released; the dependency graph may re-queue.
3. Check-and-publish is atomic with respect to commits (the shared gate prevents a commit between check and publish).

Edits outside the Processing Region (expanded by algorithm radius) do **not** make a result stale; edits inside it do. Processing expansion is never written back as a Dirty Region (AU-1.0 §8).

# 5. Conflict Handling

| Case | Outcome |
|---|---|
| Brick token mismatch in read/write set | `Conflict` result containing current tokens and the conflicting BrickKeys (never memory references) |
| Layer-wide dependency changed | `Conflict` (scope: Layer) |
| Heavy op (Boolean, Refinement) | Temporary Region lock acquired **before** snapshot, with lease and owner OperationId; conflicting writers get `TransactionConflict` or wait per policy |
| Authorization failure | `PermissionDenied`, evaluated before revision checks |

Default policy: **reject and report**. No automatic merge. Retry is the caller's decision. Automatic non-overlapping merge policies are an Open Decision (OD-R3).

# 6. Interaction with Other Modules

| Module | Rule |
|---|---|
| **History** | A delta records `PreTokens` and `PostTokens` for each affected scope. Undo = new transaction applying the inverse delta, expecting current tokens == `PostTokens` of the entry. If later edits touched those bricks ⇒ Conflict (policy for partial undo: OD-R5). Undo/redo produce new, larger revisions. |
| **Persistence** | Manifest stores Asset/Layer tokens and per-brick `(revision, content hash)`. A save works from a snapshot and records the saved cut. Recovery from an older checkpoint **increments Epoch**, so tokens issued after the checkpoint can never be mistaken as current. |
| **Networking** | Client submits `(OperationId, ExpectedTokens, payload)`. Server validates (auth first, then §4.1) and returns new authoritative tokens. Client replaces predicted state; predicted tokens (`LocalPrediction` epoch) are dropped. Reconnect replays by OperationId; server journal makes replay **idempotent**. |
| **Processing** | Only consumer of `Stale`. Validates per §4.2. Never mutates revisions of source data. |
| **Core** | Sole writer of revision values (inside the commit primitive). No module increments a revision by itself. |

# 7. Failure & Edge Semantics

- Rolled-back or cancelled transactions change **no** revision.
- No-op writes (resulting data identical) must report empty DirtyRegion and must not increment (backend responsibility; Core provides a payload-equality check helper).
- Brick creation raises Brick revision from "absent" (state `KnownEmpty`/`NotLoaded` is not revision 0 — AU-1.0 / Spec §4.4 keep these states distinct); removal increments and leaves a tombstone token until the Layer is compacted.
- Overflow of `Value`: treated as a critical error, never wraps silently (OD-R1 picks a width making this unreachable in practice).

# 8. Required Tests (all in QA Agent suites)

1. Hierarchy invariant property test (Brick ⇒ Layer ⇒ Asset).
2. Disjoint-region concurrent commits both succeed; overlapping one conflicts.
3. Snapshot immutability under concurrent writes.
4. Stale result never published (race between commit and publish).
5. Rollback/cancel leaves all tokens unchanged.
6. Epoch bump after recovery makes pre-recovery tokens Incomparable.
7. Idempotent replay of the same OperationId.
8. Undo then Redo yields larger revisions and identical data.
9. Prediction-epoch tokens never validate against server state.
10. Determinism: same operation sequence yields identical token sequence.

# 9. Open Decisions

| ID | Question |
|---|---|
| OD-R1 | Integer width of `Value` and `Epoch` |
| OD-R2 | Exact atomic commit algorithm: single gate per Asset (proposed) vs fine-grained brick-level CAS vs hybrid |
| OD-R3 | Optional automatic merge of non-overlapping concurrent edits |
| OD-R4 | Snapshot max age / memory caps |
| OD-R5 | Undo when later edits touched same bricks (reject / partial / transform) |
| OD-R6 | Whether `RegionStamp` is materialised as a set or hashed |
| OD-R7 | Tombstone compaction rules |
| OD-R8 | OperationId format (client-id + sequence vs 128-bit random) and journal retention for replay |
