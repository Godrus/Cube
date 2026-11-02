# Spike: Rust Core + Godot Client (branch `spike/rust-godot`)

**Status: EXPERIMENTAL.** A thin end-to-end slice to test the stack decided in `docs/decisions/owner-decision-rust-godot.md`. Not approved for `master`. All API contracts remain `Draft`; nothing here changes them. AU-1.0, ACR-001/002/003 and blockers B-1, B-3, B-4, B-5, B-6, B-7, B-9 are untouched.

## Layout

```
Cargo.toml                    workspace
crates/cube-core              L0   assets, layers, bricks, revisions, dirty regions, commit primitive (std only)
crates/cube-processing        L1   immutable snapshots, currency check, derived cells/bounds, stale rejection
crates/cube-editing           L3   FillRegion transaction, delta, undo/redo
crates/cube-persistence       L8   atomic single-file save/load, checksums
crates/cube-server                 headless binary + line protocol + TCP loop
crates/cube-godot-adapter          GDExtension (src/lib.rs) over a pure-Rust bridge (src/bridge.rs)
godot-client                       Godot 4 project: UI and renderer (GDScript)
```

## What is implemented (and its draft contract)

| Item | Where | Draft contract | Level |
|---|---|---|---|
| Geometry Asset with voxel layer | core `asset.rs` | C-01, C-02 | one layer type only |
| One BrickStorage (sparse, 16³, copy-on-write, `u16` material) | core `brick.rs` | C-07, C-08, C-09 | `NotLoaded`/`LoadFailure`/`OutsideBounds` states exist but are never produced |
| RevisionToken (asset/layer/brick, epoch) | core `revision.rs` | C-16 | tri-state compare, `u64`/`u32` |
| DirtyRegion | core `dirty.rs` | C-18 | one merged bounding box per commit |
| Commit primitive (validate-then-apply, +1 per scope) | core `asset.rs` | semantics §4.1 | single-threaded |
| Immutable snapshot, stale rejection | processing | PR-01, PR-02, PR-04 | no scheduler/tasks/cancellation |
| FillRegion, Delta, Undo/Redo | editing | E-03, E-04, E-05, E-09 | one op, bounded history |
| Save/Load with checksums, atomic rename, new epoch on load | persistence | P-01, P-02, P-05, P-06 (minimal) | spike file format, no deltas/migrations |
| Headless server | server | E-13 (boundary) | loopback, no auth |
| Godot adapter + client | adapter, godot-client | — | extension not loaded in Godot (see findings) |

Not implemented: SDF/Mesh, Sculpting, Boolean, Composite, Resampling, networking protocol, authorization, async jobs, any renderer beyond a MultiMesh of cubes.

## Tests (run on this branch)

49 Rust tests, all passing: core 20 + dependency rules 3, processing 6, editing 9, persistence 6, server 2 + TCP integration 1, adapter bridge integration 2. `cargo clippy --workspace --all-targets` reports no warnings. See `build-and-targets.md` for commands and timings.

Not run: anything inside Godot (no Godot binary available). `godot-client/main.gd` has a `--smoke` mode for the owner to run.

## Findings

1. **Undo semantics gap in the revision proposal.** `revision-and-snapshot-semantics.md` §6 says undo expects the `PostTokens` of its entry. Because undo is a forward commit (new, larger revisions), a chain of undos fails on the older entries (found by tests). The spike refreshes the expectations of other history entries for the bricks touched by each undo/redo. The proposal needs this rule stated; it is an input for OD-R5, not a decision.
2. **Windows toolchain is GNU, not MSVC.** Loading a GNU-built DLL in an MSVC-built Godot is unverified.
3. **Godot was not available**, so extension loading and the client scene are untested. Only the exported symbol `gdext_rust_init` was checked.
4. **Android and iOS not built** (no NDK, no macOS).
5. **Debug adapter DLL is 158 MB** (gdext debug symbols); `target/` reached 2.7 GB on a disk with about 9.5 GB free.
6. **DirtyRegion granularity** is coarse (one bounding box per commit); acceptable here, would over-invalidate for scattered edits.
7. **No face culling:** every occupied voxel becomes a cube; display limits keep calls bounded.
8. **History outside this work:** the commit `83b835b` created on `master` earlier in the session now appears as `bf60534`; the branch was cut from the current `master` tip (`bf60534`). I did not rewrite it.

## Decisions needed before anything moves to `master`

See the report at the end of the branch work; in short: whether the spike layout (`crates/`, `godot-client/`, root `Cargo.toml`, `.gitignore`) becomes the repository layout; which contracts the spike behaviour may feed back into (undo rule, brick states); the mobile build plan; whether `bridge.rs` becomes a neutral client-API crate.
