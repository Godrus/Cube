# WorldGen -> Core boundary: two options (analysis, no decision)

**Status:** experimental analysis for the spike on `spike/rust-world-generation`. It does not change the Editing API, any contract, ACR-001/002/003, or P0/P1. Nothing here is approved; the owner decides.

## What the spike does today

`WorldStreamer` generates a chunk (`cube-worldgen`, a pure function of `(seed, coord)`) and writes its two bricks into the Geometry Core asset with `GeometryAsset::commit`, **directly**, from outside the Editing API. Unloading is another direct commit. Player edits (`set_voxel`) also use the commit primitive directly and are kept in a side list (`world-state-v0`). There is no undo, no transaction record, no permission check.

## Option A: generation as system load / initialisation of the world

The world is a *derived view* of `(seed, generator version)` plus a list of player edits. Generated terrain is not user content: it is materialised on demand and can be thrown away and regenerated.

- Terrain bricks are loaded by a system service (a "world host") through a dedicated, narrow Core entry point (for example "materialise/evict generated bricks in a generated layer").
- Generated data lives in its own layer or brick class, so it is excluded from undo, from the persistence delta log, and from history. Revision effects must be defined explicitly: either generated loads do not advance the asset revision seen by editing clients, or they advance a separate counter (open question; the spike advances the asset revision by one per chunk, which is its main downside).
- Player edits are the only persisted and undoable data and go through the Editing API on top of the generated layer (override layer).
- Strengths: cheap, no history churn, matches "seed + edits" storage, simple streaming.
- Weaknesses: needs a new Core capability (generated-layer semantics, eviction), a rule for how edits interact with regeneration, and generator-version migration for saved edits.

## Option B: generation as an operation through the Editing API

Generation is an ordinary edit: a service submits "fill region with generated content" operations through `IEditingService`, getting validation, transactions, revision tokens, observers and persistence like any editor.

- Strengths: one write path, one set of invariants, history and replication for free, no special Core entry point.
- Weaknesses: every streamed chunk becomes an undoable, persisted, replicated operation unless excluded. A 5x5 window walking across the world would flood history and the delta store. Per-operation overhead sits on a hot path (generation measures about 0.01 ms per chunk, full operation handling could dominate). It needs a system-actor identity, bulk operations and a "do not record in history" flag that the Draft contracts do not define.

## Why direct Core writes are acceptable only for this spike

- The Editing API contracts are Draft, so there was nothing to call. The spike used the existing commit primitive to learn what streaming costs.
- It bypasses validation, permissions, transactions and observers, so no layer above Core sees these writes and world edits cannot be undone.
- It couples worldgen to Core internals (brick coordinates, commit scopes). The dependency rules allow `cube-worldgen -> cube-core` today, but a production architecture should not rely on it.
- Revision semantics are bent: one asset revision per chunk load or unload.

## What a production variant needs (either option)

1. A decision on A vs B (or a hybrid: A for terrain, B for player edits), as an ACR if it touches approved decisions.
2. A defined write port for system-origin geometry in the Editing API layer: actor identity, bulk operations, and a rule for history/persistence inclusion.
3. Revision semantics for generated loads: which tokens advance, and what clients compare.
4. Separation of generated and edited data, and a regeneration rule (what happens to an edit when the generator version changes).
5. A persisted generator version and seed with a migration path (`GENERATOR_VERSION` and the pinned hash tests are the start of this).
6. Backpressure and scheduling through the (still Draft) job/async contracts instead of the synchronous `pump`.
7. Tests: keep the hash-based determinism suite and add a cross-platform hash comparison on real devices.
