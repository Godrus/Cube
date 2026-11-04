# Cube Roadmap v1: from the Rust + Godot Spike to a Voxel RPG/MMORPG

**Status:** **Proposed** roadmap for owner review. The only approved inputs are the Approved Owner Decisions (AOD-01…19, `docs/decisions/owner-architecture-decisions-v1.md`). Gates, budgets, owners and task orders below are proposals; nothing here approves a contract, an ACR, AU-1.0 or a blocker closure. P0/P1 contracts stay **Draft** (AOD-18); AU-1.0 and ACR-001/002/003 are untouched (AOD-19).
**Companions:** `docs/architecture/system-boundaries-v1.md` (boundaries), `docs/decisions/worldgen-core-boundary.md` (WorldGen decision frame), `docs/research/reference-comparison.md` (Veloren, CubeWorld-Reversal), existing `docs/decisions/audit-resolution-report.md` (blockers B-1…B-13, ACRs), `docs/api-contracts/` (Draft contracts), `docs/spike/` (experimental evidence).
**Evidence rule:** all spike numbers are experimental and from a desktop (Windows, RTX 3070 Ti) or from Android **emulators**. There is no physical Android ARM64 result (`Blocked: no physical ARM64 device`) and no iOS result (`blocked: no macOS/Xcode`). Budgets are therefore **provisional** until the first device measurements replace them.

## 1. How the roadmap is organised

* **Gates** are sequential decision points with exit criteria; a gate may overlap with the next one only where its "Starts before completion" line says so.
* Every gate lists: input dependencies, owner, artifacts, contract tests, performance budget, exit criteria, what is forbidden before it completes, open decisions.
* **Owners** are the module owners of AU-1.0 (Core Agent, Processing Agent, Main Agent/Integration Owner, A-H, Composite, Resolution, I) where they exist. Modules without an AU-1.0 owner are marked "unassigned" and need an owner decision (blocker B-5, open items O-1…O-5).
* **Open items (OI-n)** are owner decisions this roadmap needs and that no existing record makes.

### Open items referenced

| ID | Open item | Needed by |
|---|---|---|
| OI-1 | Supply GDD/TDD v0.3, Voxel Architecture v1.0, Geometry Processing Specification (B-1) | Gate 0 |
| OI-2 | Supply the original AU-1.0 or confirm the reconstructed copy (B-13) | Gate 0 |
| OI-3 | Decide ACR-001/002/003 (B-3) | Gate 0 |
| OI-4 | Reconcile persistence contracts and the `cube-persistence` spike with AOD-17 (seed + generator version + journal) | Gate 5 |
| OI-5 | License policy: Cube's license, rules for third-party code/assets, whether studying GPL or reverse-engineered projects is allowed and how | Gate 0 |
| OI-6 | Single-player runs an embedded server instance (recommended; Veloren embeds its server for single-player, REF-32) | Gate 6 |
| OI-7 | RevisionToken width and commit algorithm, generated-data revision rule, system-commit port (B-6; possible new ACR) | Gate 2 |
| OI-8 | Owners for networking (O-3), engine adapters (O-5), import/export, WorldGen, server host, content, live service | Gate 0/1 |
| OI-9 | Transport, serialisation and authentication for the protocol | Gate 7 |
| OI-10 | Physical device matrix (low/mid/high Android ARM64) and acquisition | Gate 1 |
| OI-11 | Access to macOS/Xcode and an iOS device | Gate 1/10 |
| OI-12 | WorldGen direction: approve option C of `worldgen-core-boundary.md` (or choose A/B), C1 vs C2, generator-version policy, chunk delivery | Gate 2/3 |

## 2. Current state by gate (what the spike already shows)

| Gate | Spike coverage (experimental, not approved) |
|---|---|
| 0 | Source register, AU-1.0 reconstructed copy, 48 Draft contract rows, ACR drafts, blocker register. B-1, B-3, B-4, B-6, B-7, B-9, B-13 open |
| 1 | Rust workspace, Godot 4.7.2, GDExtension, Android builds on emulators, `scripts/build-android.sh`, guards against child processes; no CI, no iOS, no device |
| 2 | `cube-core`: assets, layers, 16^3 COW bricks, commit primitive, RevisionToken, DirtyRegion, snapshots; dependency-rule tests; P0 not approved |
| 3 | `cube-worldgen`: pure integer terrain, stable chunk hash, streamer (synchronous), `world-state-v0`; not production |
| 4 | `faces` and `greedy` renderers, section dirty rebuild, pixel-equivalence checks, adapter (`CubeRuntime`, `CubeWorld`); no device benchmark |
| 5 | `cube-editing`, `cube-persistence` spikes, spike undo journal; direct-commit world edits (non-compliant, documented) |
| 6 | Buttons and a camera in the Godot spike; no player, collision, inventory |
| 7 | Loopback TCP server spike, unauthenticated; no replication, prediction or auth |
| 8-9 | nothing |
| 10 | Emulator-only Android runs; scripts and APKs prepared for the owner's device run |

## 3. Gates

### Gate 0: Governance and Sources

* **Input dependencies:** none.
* **Owner:** project owner (decisions); Main Agent / Integration Owner (document set).
* **Artifacts:** the missing sources added to the repository (GDD/TDD, Voxel Architecture, Geometry Processing Specification) and the Geometry documents re-cross-checked against them (B-1, OI-1); AU-1.0 original or an owner confirmation of the reconstructed copy (B-13, OI-2); decisions on ACR-001/002/003 (OI-3); owner confirmation of AOD-01…19; a license policy document (OI-5: Cube license, inbound/outbound code and asset rules, clean-room rule for studying other projects); named owners or explicit deferrals for unassigned modules (OI-8); updated blocker register and readiness assessment; definition of how a contract is approved and who records it.
* **Contract tests:** none (documents). Review checklist: every contract cites its source section; every AOD has an enforcement or an open item.
* **Performance budget:** not applicable.
* **Exit criteria:** B-1, B-3 and B-13 closed (or deferred by the owner in writing with consequences); license policy recorded; owners named or deferred; AOD confirmed; readiness assessment re-issued.
* **Forbidden before completion:** approving any P0/P1 contract; starting production Core code on `master`; merging any spike branch; copying third-party or reverse-engineered code; deciding an ACR implicitly through implementation.
* **Open decisions:** license model; which GDD requirements constrain the Voxel contracts C-07…C-12; ownership of WorldGen, server host and content.
* **Starts before completion:** Gate 1 (technology foundation) may run in parallel.

### Gate 1: Technology Foundation

* **Input dependencies:** AOD-01…12 (given); Gate 0 for the license policy and owners.
* **Owner:** Main Agent / Integration Owner with an unassigned build/platform owner (OI-8).
* **Artifacts:** pinned stable Rust toolchain and MSRV policy; pinned Godot 4.x version; GDExtension ABI policy (engine version vs godot-rust version matrix, `compatibility_minimum`, rebuild rule on engine upgrade); target matrix (Windows/Linux desktop, Linux headless server, Android ARM64 with x86_64 emulator for functional runs, iOS ARM64); Android pipeline (NDK pin, 16 KB page alignment, signing policy and secret handling); iOS pipeline definition (macOS runner, Xcode, library delivery method; OI-11); purpose-built Cargo profiles (REF-11; panic behaviour across the GDExtension boundary tested); CI that builds, tests, lints and runs the dependency-rule test, the `no_child_processes` guard and `scripts/check-apk-no-exec.py` on exported packages (AOD-12); artifact checksums and size tracking; crash-reporting and logging policy; physical device matrix and acquisition (OI-10).
* **Contract tests:** build matrix per target; `dependency_rules`; process-call guard; extension-load smoke test on each target; package inspection check.
* **Performance budget (proposed):** pull-request pipeline within 15 minutes; client package size tracked (baseline: ARM64 client APK about 30 MB including the engine); startup time budget set from the first device measurement (emulator startup is not evidence).
* **Exit criteria:** CI green for desktop, server and Android (emulator functional run); toolchains and engine pinned; at least one physical Android ARM64 device available (or an owner deferral with the consequence "no Gate 4 renderer decision, no Gate 10"); iOS either has a macOS/Xcode path or stays recorded as blocked.
* **Forbidden before completion:** treating emulator runs as device proof; any `OS.execute`/child-process call in runtime code; unpinned toolchains in CI; release builds signed with debug keys; any iOS readiness claim.
* **Open decisions:** CI host; engine upgrade policy; signing infrastructure; crash-reporting vendor; minimum OS versions.

### Gate 2: Geometry Core

* **Input dependencies:** Gate 0 (B-1 sources, ACR-001/ACR-003 decided); owner approval of the P0 contracts C-01…C-19 (AOD-18: until then Draft); OI-7 (B-6); OI-12 for the system-commit port.
* **Owner:** Core Agent (implementation); Main Agent / Integration Owner (contracts).
* **Artifacts:** a production `cube-core` hardened from the spike: asset, layer, brick (16^3 copy-on-write, sparse), chunk concept (decide whether it belongs to Core or to streaming), coordinates including negative coordinates (Euclidean division), material/channel model (spike: 16-bit material; channels open), RevisionToken (width, epoch, tri-state compare), DirtyRegion, commit primitive (validate then apply, scoped +1 revisions), snapshots, memory ownership and lifetimes, export/import views, deterministic stable content hash for bricks/chunks, the **system-commit port** for WorldGen and the persistence loader (OI-7, OI-12), P0 contract documents promoted by the owner.
* **Contract tests:** one test set per C-xx contract; property tests (commit atomicity, snapshot isolation, revision monotonicity, negative-coordinate symmetry); negative tests for every error code; the dependency-direction (DEP-01) test closing B-9.
* **Performance budget (proposed, desktop baseline from the spike):** commit cost linear in committed bricks (baseline 0.009 ms for a chunk commit); snapshot O(1) plus changed bricks; dense brick 8 KiB, constant brick 16 bytes; about 10 KB per terrain chunk in the spike (264 KB for 25 chunks); mobile limits to be set from device data.
* **Exit criteria:** P0 contracts Approved by the owner and implemented; all Core contract/property tests green on desktop and in the Android emulator functional run; B-6 and B-9 closed; no engine dependency (AOD-04 test).
* **Forbidden before completion:** production Processing/Editing code depending on unapproved P0 types; any save or network format that freezes Core types; WorldGen production code that writes to Core other than through the approved system-commit port.
* **Open decisions:** integer widths (BrickKey, offsets), f32/f64 for transforms, brick layout and compression, channel model, chunk as a Core concept, generation revision.

### Gate 3: World Generation

* **Input dependencies:** Gate 2 (approved Core contracts and system-commit port); OI-12 (WorldGen direction); AOD-09, AOD-16, AOD-17.
* **Owner:** unassigned; proposed "WorldGen Agent" (OI-8).
* **Artifacts:** `cube-worldgen` as a production module: WorldSeed (64-bit), generator version registry and version policy (pin, bake), deterministic chunk generation (integer-only, no floats in world math, enforced by a lint or test), biome interface, chunk/brick hierarchy (chunk height and vertical streaming decided), streaming states, bounded prioritised generation queue with cancellation and per-job-class thread budget (REF-05), border consistency tests, generation cache with a memory bound, stable chunk hash contract with pinned values, **cross-device hash comparison** (desktop vs Android ARM64 vs iOS), materialiser through the system-commit port, generation revision, WorldGen contract documents.
* **Contract tests:** determinism (same seed and coordinate, any order, different seeds, alone vs in area), version replay (old version reproduces pinned hashes), cancellation, failure isolation, materialise/evict idempotence, authority (a client cannot materialise), property tests for noise continuity and borders.
* **Performance budget (proposed):** generation at most 0.1 ms per chunk on desktop (baseline 0.008-0.009 ms) and a device budget fixed from the first device run; materialisation at most 1 ms per chunk; no main-thread block longer than 2 ms; queue and cache memory bounded.
* **Exit criteria:** hash equality server vs client on a physical Android ARM64 device (or an owner deferral); streaming meets budgets on desktop and device; version policy decided and tested; WorldGen contracts Approved; spike streamer and `world-state-v0` retired.
* **Forbidden before completion:** a saved-world format that depends on the terrain function without version fields; changing the terrain function without a version bump; floating-point arithmetic in world-defining math; writing generated data around the system-commit port.
* **Open decisions:** C1 vs C2 layer model, version policy, chunk delivery to clients (OI-12); whether a world-scale pass (rivers/biomes) is added (REF-09); chunk height.
* **Starts before completion:** Gate 4 may start once Gate 2's read views are approved.

### Gate 4: Meshing and Client Adapter

* **Input dependencies:** Gate 2 (read views, DirtyRegion), Gate 3 chunk lifecycle, Gate 1 physical device; AOD-06, AOD-07, AOD-10, AOD-11.
* **Owner:** unassigned (client runtime/renderer; engine adapters O-5); Core Agent and Processing Agent for derived data inputs.
* **Artifacts:** `faces` reference mesher (correctness oracle); `greedy` candidate with a documented T-junction policy (today `SeamRule::VisibleEdges`: boundary-plane lattice plus lattice along edges that border another surface); section-level rebuild and dirty mesh propagation rules; background mesh generation with cancellation and stale-result dropping (REF-06); Godot adapter boundary specification (versioned API surface, no engine types into Rust); mobile memory budgets and view-radius profiles per device class; benchmark harness (the device validation script is the starting point); pixel-equivalence and visual regression tests; the owner's **renderer default decision after a physical-device comparison of `faces` and `greedy`** (triangles, draw calls, memory, edit and load latency, visual differences, 10-minute stability).
* **Contract tests:** faces-vs-greedy geometric equality (unit-face expansion) and pixel identity; dirty-rule sufficiency (incremental equals full rebuild); no T-junction; adapter API snapshot test; no-child-process guard on client packages.
* **Performance budget (proposed; baselines are desktop/emulator, experimental):** section rebuild at most 1 ms on desktop (baseline 0.1-0.3 ms per chunk, greedy 0.26 ms per chunk in Rust); main-thread upload at most 2 ms per frame; mesh memory for a 5x5 area 2.2 MB (`faces`) and 0.7 MB (`greedy`); 48 draw calls and 24 292 versus 10 858 triangles at 5x5; frame floor 33 ms with a 16.6 ms target on reference devices; all device numbers TBD.
* **Exit criteria:** physical-device benchmark of both renderers on at least two devices recorded; owner records the default renderer; background meshing works; adapter surface specified; visual-equality tests green.
* **Forbidden before completion:** making `greedy` the default; claiming mobile readiness from emulator data; adding baked AO or atlases before the benchmark; putting gameplay logic into GDScript.
* **Open decisions:** where meshing lives (runtime vs adapter), AO/lighting/atlas (REF-15), LOD, chunk size and view radius per device class, material model.

### Gate 5: Editing and Persistence

* **Input dependencies:** Gate 2 (P0 Approved); P1 contracts E-02…E-07, E-09, E-11, E-12 Approved; ACR-002 decided; AOD-15, AOD-17; OI-12; OI-4.
* **Owner:** Main Agent / Integration Owner (Editing API), A (Orchestration), G (History), H (Persistence).
* **Artifacts:** production Editing API and operation model (ids, results, errors, conflict result variants); edits and deltas; undo/redo semantics (undo restores the effective previous voxel; revision effects of undo as forward commits as proposed in the revision document); the **mutation journal** (versioned, append-only, with operation id, author, tick, generator version); world record (seed, generator version, config); save/load; snapshots/compaction (REF-07 evaluated); migrations framework; crash recovery (atomic writes, tolerance to a truncated last journal record, flush policy on mobile); seed + delta persistence tests; first tool-level contracts (B-7); removal of the spike's direct-commit edits and undo journal.
* **Contract tests:** every E/P contract; journal replay equals live state; kill-during-write recovery; schema upgrade; undo/redo property tests; operation idempotence; conflict result tests.
* **Performance budget (proposed):** edit commit at most 2 ms end to end on device (desktop baseline for edit plus rebuild 0.2-0.3 ms); journal append at most 1 ms; save and load times and storage growth per hour of play set from the first measurements.
* **Exit criteria:** P1 Approved and implemented; journal and snapshot round trips, migrations and recovery tests green on desktop and device; persistence format approved by the owner; `world-state-v0` and the spike undo journal removed.
* **Forbidden before completion:** freezing any save format; shipping a client-authoritative save; more than one writer; any format without a schema version.
* **Open decisions:** journal granularity (voxel deltas vs operation intents), compaction policy and snapshot format, local single-player store (REF-23), history depth, conflict policy details.

### Gate 6: Player and Local Game

* **Input dependencies:** Gate 2, Gate 4, Gate 5 (operations); OI-6; AOD-07.
* **Owner:** unassigned gameplay owner (OI-8); Godot adapter owner for UI.
* **Artifacts:** player controller (fixed-step movement), camera, collision against a voxel snapshot, interaction and raycast, inventory and items (data model in Rust, server-owned), local UI in Godot, input abstraction (intents, keyboard/gamepad/touch), touch controls, the embedded server instance for single-player (OI-6), prediction and reconciliation scaffolding, basic accessibility; GDD-driven scope (B-1).
* **Contract tests:** deterministic physics replay, raycast correctness on chunk borders and negative coordinates, inventory invariants, touch-input mapping tests; an audit that no authoritative logic lives in GDScript.
* **Performance budget (proposed):** fixed simulation step of 20-60 Hz; input-to-photon latency at most 100 ms on the reference device; collision queries at most 0.1 ms; touch targets at least 48 dp; frame floor as Gate 4.
* **Exit criteria:** a playable local loop (move, look, break and place blocks through the Editing API, inventory) on desktop and a physical Android device at the frame floor.
* **Forbidden before completion:** gameplay rules in GDScript; a client-authoritative local mode; networking assumptions inside the input layer.
* **Open decisions:** ECS or explicit domain model for entities (REF-13); camera and touch control scheme; physics determinism; item and inventory model.

### Gate 7: Server and Multiplayer

* **Input dependencies:** Gate 3, Gate 5, Gate 6; AOD-05, AOD-13, AOD-14; OI-9.
* **Owner:** Networking adapter (O-3, unassigned) and server host (unassigned).
* **Artifacts:** the authoritative server (headless binary, tick loop, sessions); chunk ownership (single writer per world instance or chunk, with shard-ready boundaries); interest management (REF-18); replication of baselines and deltas with revisions (compression, REF-19); client prediction and reconciliation; reconnect and catch-up; permissions and protected regions; anti-cheat boundary (the server validates reach, rate, tool permission; the client never supplies voxel data it did not derive from an operation); operation ids and idempotent retry; conflict resolution (server order, rejection with current state); protocol specification (versioned, authenticated handshake); transport decision with a spike (REF-04); bot-driven load tests; mobile network behaviour tests (NAT, radio wake, background, poor network); security review.
* **Contract tests:** authority tests (a client cannot commit; replayed operations rejected); replication consistency (client cache equals server state after any loss pattern); reconnect; permission matrix; protocol conformance and fuzzing.
* **Performance budget (proposed, no baseline yet):** server tick within its budget at the target player count (to be set); per-client bandwidth budget (to be set); reconnect within 5 s; playable at 200 ms round trip; an hour-long bot soak without growth.
* **Exit criteria:** bot soak and fault injection pass; authority and protocol tests green; reconnect and poor-network tests pass on a physical device; transport and protocol approved by the owner.
* **Forbidden before completion:** exposing the server to untrusted players without authentication and permissions; approving a protocol without spike evidence; client-trusting shortcuts.
* **Open decisions:** transport (TCP/QUIC/UDP), serialisation format, authentication provider, tick rate, prediction scope.

### Gate 8: World Content

* **Input dependencies:** Gate 3 (determinism, version policy), Gate 6, Gate 7 (server-authoritative NPCs); GDD (B-1).
* **Owner:** unassigned content owner.
* **Artifacts:** multiple biomes with deterministic blending, caves, rivers (REF-09 decision), structures and procedural points of interest (REF-31), NPC spawning as a separately seeded step (REF-08), resource distribution, navigation data derived from voxels, weather and time systems (server clock, deterministic seeds), a rehearsed generator version bump with the chosen migration policy; coarse world simulation only if the GDD requires it (REF-10).
* **Contract tests:** per-version pinned hashes for every layer; biome and structure placement determinism across chunk borders; terrain independent of entity randomness; migration rehearsal tests.
* **Performance budget (proposed):** full generation cost per chunk within the device budget fixed in Gate 3 (to be set); NPC count and navigation memory budgets per device class.
* **Exit criteria:** content pass deterministic and inside device budgets; the version bump procedure exercised once end to end.
* **Forbidden before completion:** shipping generator changes without a version bump; letting entity randomness influence terrain; large features without border tests.
* **Open decisions:** world-scale pass, vertical extent, structure format, AI architecture, resource economy.

### Gate 9: MMO Systems

* **Input dependencies:** Gate 7, Gate 5, Gate 8 (content to play).
* **Owner:** unassigned live-service owner.
* **Artifacts:** accounts and authentication service; persistence service (database choice, migrations, backups; REF-17); shard/region model (REF-20); matchmaking and sessions; chat; parties; guilds; economy with audit logs; moderation (reports, mutes, bans, tooling); telemetry with a privacy policy; backups with restore drills; administration tooling; legal and store-facing policies.
* **Contract tests:** account and session security tests; economy invariants (no duplication under retries); backup/restore drill; migration at scale; moderation workflow tests.
* **Performance budget (proposed, none measured):** per-shard player cap, database latency p99, backup recovery point and time objectives (to be set by the owner), chat throughput.
* **Exit criteria:** a staging environment passes a load test at the target concurrency; a restore drill passes; moderation operates; security review closed.
* **Forbidden before completion:** public launch; storing credentials without a reviewed scheme; economy features without audit logs.
* **Open decisions:** hosting, authentication provider, monetisation, data residency, shard model.

### Gate 10: Mobile Production

* **Input dependencies:** the feature gates for what ships; Gate 1 devices; OI-10, OI-11.
* **Owner:** Agent I (QA) with an unassigned platform/release owner.
* **Artifacts:** physical Android ARM64 results across a low/mid/high matrix (start-up, FPS, memory, edit and load latency, 10-minute and longer soaks); an iOS device pipeline and results (requires macOS/Xcode; library delivery decided); thermal tests (sustained run, throttling observed through device telemetry); battery tests (drain per hour); memory budgets and low-memory behaviour; low-end profiles (view radius, renderer mode, effects); store signing (release keys, provisioning); crash reporting with symbolication; offline and poor-network behaviour; app lifecycle (suspend/resume, save on interruption); store compliance (privacy declarations, permissions); size and start-up budgets.
* **Contract tests:** device matrix suite (the scripted test, touch events, soak) per device; lifecycle and interruption tests; crash-injection tests; network-condition tests.
* **Performance budget (proposed):** 30 FPS floor on the low-end device and 60 FPS target on mid-range; a sustained 10-minute run without throttling-induced frame rate below the floor; cold start within a target set from measurements (proposal: 5 s); memory below the device's low-memory kill threshold; package size within the stores' limits.
* **Exit criteria:** every matrix device passes; signed release candidates built; crash-free target met in a beta (target to be set); an equivalent result on an iOS device; owner sign-off.
* **Forbidden before completion:** any "mobile-ready" statement; store submission; using an emulator for acceptance.
* **Open decisions:** device matrix, minimum OS versions, store regions, iOS delivery method.
* **Starts before completion of earlier gates:** a first physical-device benchmark is required inside Gate 4; the full gate closes last.

## 4. Critical path

```
Gate 0 (sources, ACR, owners, license)  ->  Gate 2 (Core P0 approved)  ->  Gate 5 (Editing + persistence)  ->  Gate 7 (server)  ->  Gate 9 (MMO)
        |                                         |                              ^
        +--> Gate 1 (foundation, device) ---------+--> Gate 3 (WorldGen) -------+---> Gate 8 (content)
                          |                       +--> Gate 4 (meshing, device benchmark) --> Gate 6 (local game) --> Gate 7
                          +--------------------------------------------------------------------------------------> Gate 10 (mobile production)
```

The critical path is **Gate 0 -> Gate 2 -> Gate 5 -> Gate 7 -> Gate 9**, with Gate 3 feeding Gate 7 and Gate 8. The longest-lead external items are the **physical devices and iOS access** (OI-10, OI-11), which gate the renderer decision (Gate 4) and everything in Gate 10: they must start in Gate 1, not at the end. Gate 3, Gate 4 and Gate 6 can proceed in parallel after the Gate 2 read views and the system-commit port exist.

## 5. The nearest three tasks

1. **Gate 0 package for the owner (OI-1, OI-2, OI-3, OI-5, OI-8, OI-12).** Supply or confirm the missing sources and the AU-1.0 original, decide ACR-001/002/003, confirm AOD-01…19, record the license policy, name owners (or defer), and approve or amend the WorldGen direction (`worldgen-core-boundary.md`). Without it no P0 contract can be approved and no production Core code may start.
2. **Physical-device track (OI-10, OI-11).** Obtain one Android ARM64 phone and run `scripts/device-validation.sh` (faces and greedy, three launches each, real touches, 10-minute soak, temperature and memory samples); decide how an iOS build will be produced (macOS/Xcode access). This gives the first real mobile numbers and unblocks the renderer decision.
3. **Gate 1 foundation work order:** pin toolchains (stable Rust, Godot version), define the GDExtension ABI policy and the build profiles, and specify a CI pipeline that includes the dependency-rule test, the `no_child_processes` guard and the package check (a separate implementation task; this roadmap does not change CI). Next after these three: draft contracts for the WorldGen port, the system-commit port and the mutation journal, once the owner decision of task 1 exists.

## 6. Blockers

| ID | Blocker | Gate affected |
|---|---|---|
| B-1 | GDD/TDD, Voxel Architecture, Geometry Processing Specification absent | 0, 2, 6, 8 |
| B-3 | ACR-001/002/003 undecided | 0, 2, 5 |
| B-4 | P0 contracts unapproved (AOD-18) | 2 and every gate after it |
| B-5 | No owner for networking, engine adapters, import/export, derived-data providers, modifier evaluation | 1, 4, 7 |
| B-6 | RevisionToken width, commit algorithm, generated-data revision rule | 2, 3, 5 |
| B-7 | Tool-level contracts not written | 5 |
| B-9 | History to Persistence cycle: rule documented, verifying test not implemented | 2, 5 |
| B-13 | AU-1.0 original not in the repository | 0 |
| NB-1 | **No physical Android ARM64 device** (`Blocked: no physical ARM64 device`) | 1, 3 (cross-device hash), 4, 10 |
| NB-2 | **No macOS/Xcode: iOS blocked** | 1, 10 |
| NB-3 | No license policy (OI-5) | 0 |
| NB-4 | WorldGen direction not approved (OI-12) | 2, 3 |
| NB-5 | Persistence contracts not aligned with AOD-17 (OI-4) | 5 |
| NB-6 | No CI | 1 |

## 7. Not for `master` without a separate owner approval

* The `greedy` renderer, the `faces` renderer code, section meshing and the adapter API (`CubeRuntime`, `CubeWorld`, `WorldBridge`).
* `world-state-v0`, the WorldGen streamer API and the terrain function and chunk layout as a save/network contract.
* The spike undo journal, `fill_box`, and all direct-commit world edits.
* Spike test hooks and entry points (`world_smoke.gd`, `world_test.tscn`, soak mode), the device-validation and package-check scripts as product code, Android debug signing and the experimental export presets.
* Any change to P0/P1 status, AU-1.0, ACR-001/002/003, the persistence format or the TCP protocol.
* **This documentation branch itself** (`docs/architecture-roadmap-v1`): the records here are proposals except the AODs, which the owner should confirm when merging.

## 8. Review checklist for the owner

Confirm or amend: AOD-01…19 (`owner-architecture-decisions-v1.md` section 1); the WorldGen direction (`worldgen-core-boundary.md` section 12); the gate order and the critical path above; the nearest three tasks; the status labels of the borrowed principles (`reference-comparison.md` section 2); and the open items OI-1…OI-12.
