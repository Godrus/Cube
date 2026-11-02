# Rust Core ↔ Client Adapter Boundary

**Status:** experimental description for the spike. Not an approved contract.

## Layering

```
godot-client (GDScript: UI, glue, renderer)
      |   Godot types only on this side
      v
cube-godot-adapter  ── src/lib.rs   GDExtension class CubeRuntime   (the only crate that knows Godot)
      |             └─ src/bridge.rs  RuntimeBridge + ChangeBatch    (pure Rust, no Godot types)
      v
cube-server (headless)   cube-editing  ->  cube-processing  ->  cube-core
                         cube-persistence ------------------->  cube-core
```

Dependencies point downward only. `cube-core` depends on nothing. Enforced for this workspace by `crates/cube-core/tests/dependency_rules.rs` (manifest edges, no cycles, no engine references in non-adapter sources).

## Client Adapter Interface (what any client may rely on)

The boundary is plain data. Today it is `bridge::RuntimeBridge`:

| Call | Result |
|---|---|
| `fill_region(region, material)` | `ChangeBatch` or `BridgeError{code, message}` |
| `undo()`, `redo()` | `ChangeBatch` or error |
| `save(path)` | `()` or error |
| `load(path)` | `ChangeBatch` with `reset = true` or error (live state untouched on error) |
| `can_undo()`, `can_redo()` | `bool` |

`ChangeBatch`: `epoch`, `asset_revision`, `layer_revision` (the RevisionToken values for display), `reset`, `dirty` (list of `[min xyz, max xyz)` regions = DirtyRegions), `cells` (flat `x,y,z,material` of occupied voxels inside the dirty regions), `changed_voxels`.

Client rule: if `reset`, drop the whole mirror; otherwise clear each dirty region, then add `cells`.

Errors carry stable codes (`CORE-`, `EDIT-`, `PERS-`, `ADPT-`) and never contain memory references.

## What the adapter may and may not do

| May | May not |
|---|---|
| Convert Godot arguments to plain integers/paths | Hold authoritative geometry, revisions or history |
| Call the Rust API and return arrays | Let a Godot type (`Variant`, `Node`, `Resource`, `GString`, signals) enter any other crate |
| Keep the last result for getters | Mutate geometry outside the editing pipeline |
| Cap batch sizes (display limits) | Reinterpret revisions or invent its own |

The client keeps a **display mirror** only; it can be rebuilt at any time from `load` or from `cells` of a full region.

## Replacing Godot later

```
Rust Core (cube-core / processing / editing / persistence)
    ↓
Client Adapter Interface (bridge::RuntimeBridge, ChangeBatch)
    ├── Godot Adapter (cube-godot-adapter, this spike)
    └── future Rust Native Adapter (NOT implemented)
```

A native adapter would depend on `bridge` types (or the Rust crates directly) and supply its own renderer and input. Nothing in the core crates would change. If more than one adapter is needed, `bridge.rs` should move out of the Godot adapter crate into a neutral crate (for example `cube-client-api`); this is deliberately not done in the spike to avoid adding structure before it is needed. That move is a proposal for the Integration Owner, not a decision.

## Known simplifications

- One `ChangeBatch` per call; no streaming, no incremental subscription.
- Dirty regions of one commit are merged into one bounding box (conservative, correct, not tight).
- Cells are all occupied voxels in the region: no face culling, no meshing; display limits `MAX_FILL_VOLUME = 65 536` voxels per fill and `MAX_BATCH_CELLS = 262 144`.
- Server and adapter each wrap the same crates separately; there is no shared facade crate yet.
