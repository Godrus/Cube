# Owner Architecture Decisions v1

**Status:** Approved Owner Decisions (AOD), recorded from the owner's instruction in chat on 2026-11-04 for the docs-only branch `docs/architecture-roadmap-v1`. The owner should confirm this record when the branch is merged.
**Scope:** technology, boundary and process decisions needed to go from the current Rust + Godot spike to a full voxel RPG/MMORPG. This record **does not approve any API contract**, does not decide any ACR, does not edit AU-1.0 and does not close any blocker. It extends `owner-decision-rust-godot.md` (which stays valid).
**How to read the "Today" column:** it states what the repository demonstrates now (spike branches), so that an approved decision is never mistaken for an implemented one.

## 1. Decisions

| ID | Decision | Consequence / enforcement | Today (evidence) |
|---|---|---|---|
| AOD-01 | **Language: Rust.** | Core, WorldGen, Editing, Persistence, Server, protocol are Rust. No C#/C++ gameplay or core code. | Rust workspace on `spike/rust-godot` and `spike/rust-world-generation`. Same as `owner-decision-rust-godot.md`; B-2 stays resolved. |
| AOD-02 | **First client: Godot 4.x.** | The first shippable client is a Godot client. No second engine is started before the replacement question (AOD-06) is explicitly opened. | Godot 4.7.2 client exists (desktop verified, Android emulator verified). |
| AOD-03 | **Integration: Rust GDExtension.** | The only Rust-to-Godot path is a GDExtension library (godot-rust). No C# bridge, no sidecar process driven through OS calls. | `cube-godot-adapter` (`CubeRuntime`, `CubeWorld`). |
| AOD-04 | **Geometry Core is pure Rust with no Godot dependency.** | Core crates must build and test without Godot; no engine types in Core. | Enforced by `crates/cube-core/tests/dependency_rules.rs` (allowed crate edges, no engine type names in Core sources). |
| AOD-05 | **Server is a separate headless Rust process using the same Core.** | A dedicated binary, no Godot linked, built from the same Core/Editing crates as the client runtime. | `cube-server` (spike: loopback TCP, no authentication). The production server design is open (Gate 7). |
| AOD-06 | **Godot is a temporary, replaceable frontend.** | Everything a frontend needs sits behind a narrow adapter; game rules, geometry, generation, persistence and protocol never live in Godot assets or scripts. | Adapter boundary documented (`docs/spike/rust-godot/core-adapter-boundary.md`); `world.gd` still holds streaming/mesh-node glue (see AOD-07). |
| AOD-07 | **GDScript: UI and glue only.** | No authoritative logic, no persistence, no generation, no networking in GDScript. Glue = mirroring Rust state into nodes, input forwarding, HUD. | `world.gd` contains no test code and no process calls (guard test); node/mesh mirroring is glue. Further logic should move to Rust, not to GDScript. |
| AOD-08 | **Android and iOS are mandatory targets.** | Every gate that touches runtime must state its mobile impact; mobile is not a late port. | Android: built, emulator only. iOS: **blocked, no macOS/Xcode**; nothing built. These are obligations, not achievements. |
| AOD-09 | **WorldGen is pure Rust and deterministic, seed-based.** | Generation is a pure function of (seed, generator version, coordinates); no floats in world-defining math, no global state, no time. Pinned hashes guard it. | `cube-worldgen`: integer-only noise, `GENERATOR_VERSION`, stable chunk hash and pinned values (`tests/determinism.rs`). Cross-device hash comparison **not done**. |
| AOD-10 | **Renderer: `faces` is the reference; `greedy` is an experimental candidate until a real mobile benchmark.** | `faces` is the default and the correctness oracle; `greedy` may not become the default before a physical-device comparison. | `faces`/`greedy` pixel-identical in compared scenes; no device benchmark exists. |
| AOD-11 | **An ARM64 emulator never replaces a physical device.** | Emulator runs may be reported as functional checks only; mobile readiness needs a physical device (Android ARM64) and an iOS device. | ARM64 translated emulator runs exist and are labelled "not a device result". Status: `Blocked: no physical ARM64 device`. |
| AOD-12 | **`OS.execute` (and any child-process or shell call) is forbidden in the production runtime.** | No GDScript, adapter or client code may start processes, call `getprop` or shell out. Test tooling runs on the host, never inside the app. | Guard test `crates/cube-godot-adapter/tests/no_child_processes.rs`, package check `scripts/check-apk-no-exec.py`. CI wiring is open (Gate 1). |
| AOD-13 | **Server authority: the authoritative world state lives on the server.** | Any state that matters (voxels, entities, inventory, time) is decided and stored by the server; replication goes server to client. | Not implemented beyond the spike server. Defined in `docs/architecture/system-boundaries-v1.md`. |
| AOD-14 | **The client is never the source of truth.** | The client holds a cache and a prediction; it may be wrong and is corrected. No client-side save is authoritative. | The spike client mirrors Rust state; `world-state-v0` is a spike file, not a production save. |
| AOD-15 | **User changes go through the Editing API.** | Every user-originated change to geometry is an operation with an operation id, validation, revision effect and history semantics. | **Spike exceptions** (not compliant, documented): world edits and the spike undo journal write through the Core commit primitive outside the Editing API (`docs/spike/rust-world-generation/worldgen-core-boundary.md`, report section 28). |
| AOD-16 | **Procedural generation is a separate WorldGen/System boundary, not an ordinary user edit.** | Generated data does not enter history, is not a user operation, and has its own revision and persistence rules. | The spike writes generated chunks through the same commit primitive; the boundary is analysed in `docs/decisions/worldgen-core-boundary.md` (recommendation: option C). |
| AOD-17 | **Persistence: an untouched world is stored as seed + generator version; user changes as versioned deltas/journal.** | The save of an untouched world is tiny and regenerated; user changes are an append-only versioned journal with snapshots/compaction as an optimisation. Generator versions must stay reproducible (or be baked) for every saved world. | `world-state-v0` (seed + edits + CRC, temporary, not approved) is a spike precursor. The `cube-persistence` spike format and the Draft persistence contracts must be reconciled with this decision (open item OI-4 in the roadmap). |
| AOD-18 | **P0/P1 contracts stay Draft until the owner separately approves them.** | No code may claim a P0/P1 contract as Approved or Frozen. Spike implementations are experimental. | Unchanged: all contracts `0.1.0-draft`. |
| AOD-19 | **AU-1.0 and ACR-001/002/003 are not rewritten automatically.** | These documents change only by explicit owner action. Where a decision here seems to touch them, the interaction is listed for the owner (section 3), not applied. | AU-1.0 is a reconstructed Proposed/Draft copy (B-13); ACR-001/002/003 are Pending. |

## 2. What these decisions do not decide

The owner still has to decide (none is decided by this record): approval of any P0/P1 contract; ACR-001/002/003; RevisionToken width and commit algorithm (B-6); whether a generated layer or a single layer plus journal is used (worldgen boundary, section 5 of `worldgen-core-boundary.md`); the network protocol and transport; the license policy for Cube and the policy for studying GPL and reverse-engineered sources; the persistence file formats; the account/MMO model; whether single-player runs an embedded server instance (recommended, not decided); the replacement trigger for Godot; owners for networking, engine adapters, import/export and the other unassigned modules (O-1…O-5, B-5).

## 3. Interactions with existing documents (for the owner; nothing is changed here)

| Decision | Touches | Interaction |
|---|---|---|
| AOD-16 / AOD-15 | ACR-001 (foundation types in Core), RevisionToken proposal | A system write path next to the Editing API implies a Core entry point for system commits and revision rules for generated data. That is a RevisionToken/commit-primitive question (B-6) and, if it changes placement of contracts, a candidate **new** ACR. This record does not file it. |
| AOD-17 | Persistence contracts P-01…P-09 (Draft), `IDeltaStore`, `ISnapshotStore`, B-9 | The Draft persistence contracts describe snapshots and deltas of geometry; AOD-17 adds seed + generator version as the baseline of untouched regions. The contracts need a rewrite pass before approval. |
| AOD-13 / AOD-14 | Editing API ports (E-series), operation ids, conflicts | Authority and conflict handling must be stated in the Editing/Server contracts (Gate 7); the history module (Agent G) currently has no tool-level interface (B-7). |
| AOD-04 / AOD-06 | AU-1.0 layer L10 (engine adapters) | The adapter layer is unassigned (O-2); AOD-06 makes its owner a Gate 1 requirement. |
| AOD-12 | none | New rule; no document conflicts. |

## 4. Change control

An Approved Owner Decision is changed only by a new owner decision recorded in `docs/decisions/` that names the AOD id it replaces. A change needs the owner's explicit approval in chat or in a signed-off pull request; an agent may propose a change but may not apply it.
