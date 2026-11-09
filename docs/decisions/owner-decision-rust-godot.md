# Owner Decision: Rust Runtime and Godot First Client

**Status:** Recorded from the owner's instruction (chat, 2026-11-02) for the experimental branch `spike/rust-godot`. The owner should confirm this record when merging anything to `master`.
**Scope:** technology stack for a first end-to-end spike. It does **not** approve any API contract, ACR or architecture change.

## Decision

| Area | Choice |
|---|---|
| Geometry Core, Processing, Editing, Persistence, Server | Rust |
| First client frontend | Godot 4.x |
| Rust ↔ Godot binding | GDExtension (godot-rust / gdext) |
| GDScript | UI and simple glue only |
| Target platforms | Windows, Android (ARM64), iOS (ARM64) |
| Excluded | C#, C++, Unity, Unreal or any other engine/runtime alternative |

## Constraints that come with the decision

1. Rust Core has no Godot dependency and no Godot types (no Node, Resource, SceneTree, GDScript, signals, `.tscn`/`.tres`, engine math or ownership types).
2. `cube-core`, `cube-processing`, `cube-editing`, `cube-persistence`, `cube-server` build and test without Godot.
3. The Godot adapter is thin and owns no authoritative geometry.
4. The runtime ↔ client boundary is described separately (`docs/spike/rust-godot/core-adapter-boundary.md`) so the client can be replaced later (for example by a Rust-native client adapter, not implemented).

## Effect on open items

| Item | Effect |
|---|---|
| B-2 (language not chosen) | Resolved by this decision for Rust. |
| O-0a / O-0b (language / engine) | Language: Rust. Engine: Godot 4.x as the first client; this does not make Godot a dependency of Core. |
| B-1, B-3, B-4, B-5, B-6, B-7, B-9 | **Not closed** by this decision or by the spike. |
| ACR-001/002/003 | Unchanged (Pending). |
| AU-1.0 | Unchanged. |
| Contract status | All contracts stay `Draft`. The spike implements parts of them experimentally; nothing is promoted to `Approved`. |

## Not decided here

Integer/float widths beyond what the spike uses, RevisionToken final semantics, atomic commit algorithm, serialization format, network protocol, mobile packaging details, iOS delivery method for the native library.
