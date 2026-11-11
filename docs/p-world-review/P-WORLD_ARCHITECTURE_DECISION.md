RECOMMENDATION: C (Selective Integration), executed as a narrow first step (see section 10)
PROTOTYPE_STATUS: Experimental L2 spike, commit `fea3deb` on local branch `spike/p-world`; "APPROVE WITH CONDITIONS" as an experiment result only; not merged, not pushed
PRODUCTION_READINESS: NOT READY. Not approved for production, for integration "as is", or as the current architecture
FIRST_PLAYABLE_CONTRIBUTION: High as a reference and as a source of requirements, tests and measured parameters; low as reusable production code. Roughly requirements 1-5, 10-12 of the proposed First Playable scope have a working local demonstration; none is yet through the approved Editing API, server authority or approved persistence
NEXT_STAGE: "P-WORLD Port Slice": a small, owner-approved L2 step that puts the world-port (read / raycast / collision / edit / persist) behind one Rust-owned interface, with edits routed through the existing `cube-editing` crate, and a human play-test. Details in section 10
OWNER_APPROVAL_REQUIRED: YES

# P-WORLD Architecture Decision Report

Date: 2026-11-08. Author: the agent that wrote P-WORLD (conflict of interest: the owner should weigh this; every conclusion below is tied to a source). This report is a **proposal**. It makes no Owner Decision, changes no ADG, PS or production status, and starts no implementation.

Marks: VERIFIED = checked by me in this session against files/source; REPORTED = from earlier reports, not rechecked now; INFERRED = my technical conclusion from facts; UNKNOWN = not enough data.

---

## 1. Executive Decision

**Recommendation: Option C, Selective Integration**, done narrowly and in two steps: first extract *knowledge and tests* from P-WORLD (cheap, no governance risk), then re-implement the few pieces that are genuinely reusable behind the already-approved boundaries. Do **not** adopt P-WORLD as a foundation (B) and do **not** discard it (D).

Why, in five lines:

1. P-WORLD proves the *shape* of First Playable steps 1-5 (visible world, walking character, collision, raycast edit, edit-then-restart persistence) in about 2,100 lines. That proof is valuable and reproducible. (REPORTED + VERIFIED for reproducibility by the Owner Review; the Owner Review is by the same agent, see conflict note.)
2. Its write path and save format are **already ruled non-compliant by approved owner decisions** (AOD-15, AOD-16, AOD-17, AOD-20, AOD-21): it bypasses the Editing API (`streamer.rs:480,502,615` use `operation: None`) and uses a temporary format `world-state-v0`. (VERIFIED in source and in `docs/decisions/owner-architecture-decisions-v1.md`.)
3. The same files also say authority belongs in Rust (AOD-07, AOD-13, AOD-14), while P-WORLD's call order lives in `world.gd`. Building more on it would deepen a parallel architecture. (VERIFIED.)
4. The parts that are architecture-neutral (terrain function, seam test, DDA raycast, AABB step/collision, tests) are small and cheap to keep or re-implement, so A (freeze) alone wastes useful momentum, and B (keep growing it) accumulates integration debt.
5. A is the correct *fallback* if the owner wants zero further code work now; D is not justified because most of the design is compatible with AOD-20/21 (seed + edits re-applied over regenerated terrain is exactly the approved C3/AOD-21 base model). (INFERRED, with the approved text VERIFIED.)

**What I do not claim:** that P-WORLD is First Playable (it is not; no combat, creatures, items, inventory, menus, second biome, server authority, or human play-test), or that anything is production-ready.

---

## 2. Evidence and Confidence

### 2.1 What I read in this session

| Material | Result |
|---|---|
| `handoff\1_P-WORLD_STATE.md`, `2_..HANDOFF.md`, `3_..EVIDENCE.md`, `4_..LIMITATIONS.md` | read in full |
| `OWNER_REVIEW.md` (archive copy) | read in full |
| Diff `820f538..fea3deb` | VERIFIED `git diff --stat`: 18 files, +2119/-12; worktree `Cube-pworld` clean |
| Source spot checks | `streamer.rs` (`operation: None` at lines 480, 502, 615; `set_voxel` line 582; `active_voxels` line 252), `world.gd:565` (per-frame call), `brick.rs` (sparse COW `HashMap` of `Arc` bricks, `Constant`/`Dense`), crate dependency lists, `Cargo.toml` dev profile |
| Governance docs | `owner-architecture-decisions-v1.md` (AOD-06/07/13-17/20/21), `worldgen-core-boundary.md`, `TECHNOLOGY-GAME-TRACK-INTEGRATION.md` (L2 rules, world port), `FIRST_PLAYABLE_SCOPE.md` (status: Proposed) |

### 2.2 What I did NOT do

* I did **not** re-run the tests, the Godot scenes or the persistence test in this session. Results below are REPORTED (earlier runs and the Owner Review's reproduction, which is the same agent). The previous review states that all deterministic results reproduced exactly.
* I did **not** read every line of the 18 changed files. I read the Owner Review's findings and checked the ones that drive architecture conclusions (write path, version in save, HUD call, dependencies). C-4, C-5, C-12 and the physics details are REPORTED.
* No human has played the prototype (UNKNOWN: feel of controls, camera, jump).
* Not verified at all (UNKNOWN): clean-clone start, other hardware/GPU, Android/iOS, disk-full/crash-during-save, two processes sharing one save, effect of `opt-level=3` alone, far-from-centre edits across a restart.

### 2.3 Confidence summary

| Claim | Confidence |
|---|---|
| P-WORLD works as a local, single-process demo on the author's machine | High (REPORTED, reproduced by same-agent review; deterministic outputs match exactly) |
| P-WORLD violates AOD-15/16/17/07 as production rules | High (VERIFIED in source and approved text) |
| Radius 6/8 overrun is caused mainly by the HUD `active_voxels()` call | Medium-high (measured per-call cost REPORTED; removal not tested because code was frozen) |
| Physics feels good | UNKNOWN |
| Terrain/chunk size choices are right for the game | UNKNOWN (ODM-11 open) |

### 2.4 Status of architecture ideas referenced here (as far as I know)

| Idea | Status |
|---|---|
| Server authority (AOD-13), client not source of truth (AOD-14) | **Approved** Owner Decisions (VERIFIED) |
| User edits via Editing API (AOD-15); generation is a separate boundary (AOD-16, AOD-20 variant C3) | **Approved** direction; port signatures, error sets, revision rule **not approved** (VERIFIED) |
| Persistence: seed + GeneratorVersion + journal + snapshots (AOD-17/21) | **Approved** direction; contracts P-01..P-09 Draft (VERIFIED) |
| GDScript = UI and glue only (AOD-07), Godot replaceable (AOD-06) | **Approved** (VERIFIED) |
| Fixed tick, World Operations, job contract (SR/TC-11, TC-10) | **Proposed** only (REPORTED in Owner Review) |
| First Playable scope, world port | **Proposed** (VERIFIED in file headers) |
| World/Micro/Model voxel levels, Hybrid Cubic+SDF+Mesh, LOD, Floating Origin | I found no approved decision for these in the files I read; treat as **Proposed or Unknown** (UNKNOWN) |
| PS-0 open, PS-1 not authorised, L3/L4 blocked | VERIFIED (integration doc section 2) |

---

## 3. Current P-WORLD Capabilities

Compact per-subsystem table. Columns: Implemented / Works (evidence) / Limits / Reuse / Keep experimental / Production change.

| Subsystem | Implemented and works (evidence) | Limits | Reuse | Keep experimental | Production change |
|---|---|---|---|---|---|
| **Procedural generation** | `cube-worldgen`: deterministic integer-noise height map, chunk 16x32x16; same seed -> same hash across processes (4 seeds; REPORTED, reproduced); neighbours agree on borders; 480 seam borders on 12 seeds | one biome, heights 6..19, no caves/water; fixed 32 height; no generator version stored in saves (C-3) | The *function-of-(seed, coordinate)* approach, determinism tests, seam test, pinned hashes (all small) | The actual terrain function and 32-high layout | Second biome (First Playable needs two); height/width decision (ODM-11); versioned generator |
| **Chunk storage/streaming** | `WorldStreamer` over sparse COW bricks (VERIFIED `brick.rs`); synchronous load/unload around a centre; unloaded chunks block character | main-thread, synchronous; one Core revision per chunk load/unload; second state list (`edits`) next to Core | Brick storage (it is Core, not P-WORLD); streaming *state machine* ideas and tests | Synchronous `pump` loop | Job contract before any background work (A-5); system-commit port (AOD-20) |
| **Voxel data / queries** | `voxel_at`, `column_top`, `voxel` reads via bridge (REPORTED, tested) | reads depend on chunk Ready; coordinate type width undecided | Read semantics ("unloaded = not solid-readable") as a requirement | - | World-port `read` provider with decided coordinates |
| **Mesh / rendering** | faces / greedy / cubes meshing in Godot, flat colours, one light; dirty-section rebuild incl. neighbour on border edits (REPORTED, mesh equals rebuild from save) | no LOD, no textures; HUD cost C-1 | Dirty-section rule and the "mesh == rebuild from save" test | Flat-colour look | Processing-owned mesh derivation (existing `cube-processing` concepts); LOD only if required |
| **Player controller** | `player.rs` AABB 0.6x1.8, gravity 28, jump 8.5, speed 6, one-voxel step-up, ground probe, sub-step 1/60 (REPORTED; 5 unit tests + walk tests); orbit camera in GDScript | feel unjudged by a human; physics driven by display frame time (C-10); camera clips terrain; NaN/inf unsanitised (C-5) | Collision algorithm and its regression tests (jitter test is a real finding) | Constants | Fixed-step simulation owned by Rust; intent-based input (FIRST_PLAYABLE 4.1) |
| **Physics/collision** | Per-axis AABB vs voxel read; border walking across 230 borders; unloaded-chunk hold (REPORTED, reproduced) | heuristics (step-up, ground probe) not proofs (C-12); no entities/projectiles | Test scenarios and the "hold when chunk missing" rule | Everything numeric | Collision provider behind a contract (VOX-05, query home open) |
| **Raycast / editing** | DDA raycast agrees with brute force on 1600 rays; remove/place rules; border and collision update (REPORTED, reproduced) | unbounded `max_dist` (C-5); **write bypasses Editing API** (A-1, VERIFIED); one material; journal grows unbounded | DDA algorithm and its brute-force cross-test; rule list (not inside body, only Ready chunks, y range) | Spike undo journal | Route through `cube-editing` with typed operations (AOD-15) |
| **Persistence** | `world-state-v0`: seed + config + centre + chunk list + per-chunk edits, CRC-32, atomic write; 3-process real-window restart PASS (REPORTED, reproduced) | temporary format; no generator version; CRC-valid crafted file can panic (C-4); one slot; shutdown hook only (C-6); `.bad` handling blunt (C-7) | The **model** (seed + overrides re-applied over regenerated terrain) = AOD-21 baseline; the restart scenario as an acceptance test | The file format | Approved journal/snapshot format with GeneratorVersion inside; validated load; lifecycle design |
| **Rust/Godot integration** | `WorldBridge` pure Rust + `CubeWorld` GDExtension class; logic mostly in Rust, tests run without Godot (VERIFIED crate layout) | `world.gd` owns call order, streaming, save path, shutdown, test hooks (A-3, C-9); debug lib with optimised crates (D-7) | The pattern "pure-Rust bridge testable without Godot" (matches AOD-06) | `world.gd` as a whole | Rust runtime object owns tick/edits/save; GDScript sends intents only |
| **Performance** | Stage A/B tables (REPORTED): r1 avg 2.2 ms, r2 3.0, r4 7.1, r6 12.7, r8 20.2 | numbers from earlier code states; one machine; r6: 120/1241 frames over 16.7 ms; C-1 explains most growth | The measurement *method* (fixed-step walk, p95/worst) | All absolute numbers | Remove per-frame full-voxel scan; re-measure on the real build; budgets are not approved |

**Not a capability of P-WORLD, relevant for First Playable:** creatures, combat, items, inventory, equipment, menus, player-position save, second biome, server/client split, autosave beyond window-close. (VERIFIED by the 4 handoff docs; they list these as not done.)

---

## 4. Architecture Compatibility

| Principle | Status of principle | P-WORLD vs it | Parallel-architecture risk |
|---|---|---|---|
| World Voxels / Micro Voxels / Models as distinct levels | UNKNOWN (no approved decision found) | Only world voxels (u16 material). Nothing to conflict with | None now |
| Hybrid Cubic + SDF/Density + Mesh | UNKNOWN | Cubic only, mesh derived. No SDF | None now; not a P-WORLD problem |
| Sparse brick storage | In Core (draft C-08) | **Compatible**: P-WORLD uses Core bricks unchanged (VERIFIED) | Low |
| Separation Core / Processing / Rendering / Physics / Persistence | Approved direction (system-boundaries, AOD) | Partly: Core and Processing are used; **Physics lives in the adapter crate** (`player.rs`) and persistence format lives in `cube-worldgen` (`state_v0.rs`), not in their own modules | **Medium**: physics and save-state are in the wrong homes |
| Core independent of Godot/UI/Network | Approved | **Compatible**: `cube-worldgen`, `player`, `world_bridge` are pure Rust; only `lib.rs` touches Godot (VERIFIED dependency lists; Core has no Godot dep) | Low |
| Editing API single mutation point | Approved (AOD-15) | **Violated** by design: `set_voxel` commits directly, `operation: None`; adapter has two write paths (C-2) | **High**: this is the main parallel path |
| Server-authoritative multiplayer | Approved (AOD-13/14) | Single process; physics and edits decided client-side; `cube-server` crate exists but P-WORLD doesn't use it (no dependency) | **High** if the single-process loop is extended; Low if used only as a reference |
| World Operations + fixed tick | Proposed | Variable frame-time physics in `_process`; no operation messages | Medium: character physics would need reshaping |
| Async chunk processing | Proposed job contract | Synchronous on main thread | Low (cost is elsewhere, C-1) |
| LOD | UNKNOWN | None | n/a |
| Floating origin | UNKNOWN | `VoxelPos` coordinates, overflow at extreme chunk coords (C-4, A-6) | Low now; ODM-11 open |
| Versioned persistence incl. generator version | Approved (AOD-17/21) | **Violated**: no generator version in the file (C-3) | High if saves are kept |

**Does P-WORLD create a parallel architecture?** INFERRED: **yes, in two specific places** (write path, save state) and **in one structural habit** (game logic ordered by a GDScript scene). Everything else sits on top of the shared Core/Processing crates and does not duplicate them. The risk is therefore *bounded* and *known*: it is the write path, the save format and the call-order ownership. This is why Selective Integration is feasible and why growing P-WORLD in place (B) would increase, not reduce, rewrite cost.

Note on direction: the approved AOD-20 variant C3 and AOD-21 (generation separate; seed + generator version + edit deltas) are structurally the same as P-WORLD's "regenerate from seed and re-apply edits". So the *model* is aligned; the *mechanism* (bypass, temporary file, no version, no journal) is not. (VERIFIED against `worldgen-core-boundary.md` section 2 and AOD-20/21 text.)

---

## 5. Technical Debt Assessment

I do not propose to fix everything. Classification against the recommended next stage (section 10), not against "production in general".

| Category | Items | Reason |
|---|---|---|
| **Blocks further use of the prototype** (even as a reference/demo for the owner) | C-1 (HUD per-frame scan: radius 6/8 unusable; trivial cause) | Without it, the demo itself is misleading about performance. But only matters if we run the prototype again; see note below |
| **Blocks production integration** | A-1/C-2 (write path), A-2/C-3 (format, generator version), A-3 (authority in GDScript), A-4 (tick), C-4 (validated load), C-5 (input sanitising), C-6/C-7 (lifecycle, `.bad` handling), C-9/A-8 (test hooks in runtime) | Each is a contract or approved-decision conflict; none can be waived for L3/L4 |
| **Defer to the matching stage** | A-5 job contract, A-6/ODM-11 coordinates and world bound, C-10 variable-step physics (until fixed tick exists), D-7 profile, LOD/floating origin, second biome, multi-material | Not needed for the next stage |
| **Acceptable for an experimental prototype** | D-1..D-8 (declared deviations), C-8 hash print, C-11, C-12 heuristic step-up (kept under tests), flat rendering, one save slot, synthetic-input tests | By L2 rules (integration doc section 3) |

**Minimal set actually needed for the proposed next stage** (section 10): C-2/A-1 handled by *re-implementing the edit path against `cube-editing`* (not patching P-WORLD), C-1 avoided by not carrying the HUD call forward, C-5 sanitising in the new interface, and a human play-test (open item). Everything else is explicitly out of scope.

Note: I recommend **not** fixing C-1..C-12 in the P-WORLD branch. They are cheaper to avoid in a fresh port than to fix in code that will be discarded.

---

## 6. Reuse Assessment

| Reuse class | Items |
|---|---|
| **Keep as-is (reference, no code change)** | Commit `fea3deb` / branch `spike/p-world` / two bundles; the handoff set; Owner Review; logs/screenshots (only if copied out of the temp folder) |
| **Reuse as tests and requirements (highest value, lowest risk)** | Seed -> hash determinism and seam check (`world_hash`, `seams`); raycast vs brute-force march; remove/place rule list; "walk across N chunk borders", "unloaded chunk holds", "jitter-dt ground flicker" (a genuine defect class found); edit-then-restart scenario with three-process structure; mesh-equals-rebuild-from-save; frame-time measurement method |
| **Reuse as algorithms, re-implemented behind an approved boundary** | DDA raycast; AABB axis-swept collision with step-up and ground probe; dirty-section rule including neighbours; edit de-duplication (`retain(|e| e.pos != p)`) |
| **Reuse as data/parameters** | Character constants (width, height, gravity, jump, speed) as *tunable data*, after a human play-test |
| **Keep experimental (do not extend)** | `world.gd`, orbit camera, HUD, test scenes (`world_edit/interactive/seams`), `run-world.ps1`, `test-world-persistence.ps1` (these may be copied as a *reference harness*), terrain function, 32-high layout, flat rendering |
| **Do not move into production without rework** | `state_v0.rs` (format and loader), `WorldStreamer::set_voxel` and the direct commit path, `player.rs` in the adapter crate (home and tick model), `world_bridge.rs` as a unit (two write paths), `world.gd` authority logic, test hooks |

---

## 7. Options A/B/C/D Comparison

Costs are relative and **not estimates**: I have no basis for calendar numbers and do not invent them (UNKNOWN).

| | **A. Reference Prototype** | **B. Experimental Foundation** | **C. Selective Integration** | **D. Replacement** |
|---|---|---|---|---|
| Meaning | Freeze; use as reference only | Keep growing in `spike/p-world` | Port selected pieces behind approved boundaries; no wholesale move | Discard as a basis; keep as history |
| Advantages | Zero risk, zero governance load; knowledge preserved | Fastest visible progress toward a playable-looking game; no contract waiting | Keeps the value (tests, algorithms, parameters, scenarios) while building on the approved write path; avoids growing the bypass | Cleanest architecture; no legacy to carry |
| Disadvantages | No forward motion; repeats work later; "visible world" stays a demo | Spike grows, becomes de-facto foundation; owner's rule: spike code never merges; integration debt grows with each feature | Needs owner permission for an L2 follow-up; requires discipline on scope; some re-implementation | Throws away working determinism/seam/raycast/collision logic and ~150 tests of knowledge; slowest |
| Architecture risk | None | **High**: deepens A-1, A-2, A-3; first-playable features (items, combat) would sit on the bypass; likely Editing-API rewrite of everything touching voxels | Low-medium: the risk is a half-ported mixture; mitigated by one narrow interface and stop conditions | Low (but re-introduces the same decisions: write path, persistence) |
| Cost (relative) | Lowest now | Low now, **highest later** (INFERRED) | Medium now, low later | Highest now |
| Effect on First Playable | Neutral-to-negative (no progress) | Looks fastest, but First Playable *requires* the Editing API path (scope item 10) and authoritative runtime, so work above the bypass is not First Playable as scoped | Positive: shortest path that lands on the required boundaries | Negative (restart) |
| Probability of re-development | Certain for anything that is later wanted | High (INFERRED) | Low for ported pieces; moderate for glue | Certain |
| Owner Decisions needed | None beyond "keep it" | Waiver of "spike never merges" rule, or acceptance that P-WORLD stays throwaway forever | Go-ahead for a new L2 step; decide which write path to use for the port (below); possibly the ODM-11 numbers | Decision to abandon; plus the same architecture decisions as C |

**A more suitable variant?** I considered "C-lite": do only the *knowledge extraction* (tests and requirement table) with no new code. It is cheap but does not advance First Playable. I fold it into C as the zero-cost first step; it can be stopped there.

---

## 8. Recommended Owner Decision

**Option C, Selective Integration**, beginning with the narrow stage in section 10. This is a *recommendation*; decisions are the owner's.

| Group | Items |
|---|---|
| **Save (preserve)** | `spike/p-world` branch, both bundles in `Cube-archive\p-world\`, handoff docs, Owner Review. Recommended: make a second copy on another drive or remote, and copy logs/screenshots out of the temporary session folder (owner's call) |
| **Continue to use** | P-WORLD as the **reference demo** and as the source of acceptance scenarios; the pure-Rust-bridge-tested-without-Godot pattern; Core sparse bricks; `cube-editing` as the edit path |
| **Do not develop further** | `world.gd` (and its test hooks), `state_v0`, the `set_voxel` bypass, the HUD, flat rendering, the 32-high terrain function, the PowerShell run scripts as a product |
| **Not to production without rework** | Everything in the "Do not move" row of section 6, especially the write path, the save format and loader, and any authority in GDScript |
| **Open questions** | Which write path serves generation (AOD-20 says "WorldGen/System-commit boundary", signature not approved); how a system commit advances revisions (B-6 open); coordinate width and vertical extent (ODM-11); whether single-player runs an embedded server (proposed in `worldgen-core-boundary.md`, not approved); whether the physics lives in the server runtime or a separate module; the exact First Playable scope (Proposed) |
| **Owner decisions needed** | See section 13 |

---

## 9. First Playable Path

First Playable is defined (Proposed, `FIRST_PLAYABLE_SCOPE.md`) as a minimum *game*, not a tech demo: a loop with creature, combat, loot, equipment, dig/build, save/return. P-WORLD does **not** satisfy it; it covers part of the front half.

| Step | P-WORLD today (evidence) | What is still missing for First Playable |
|---|---|---|
| 1. Launchable visual world | Yes (REPORTED+reproduced; start script verified in worktree, not on a clean clone) | New-world UI (name/seed), clean-clone start, a second biome |
| 2. Controllable character | Yes, third-person walking (synthetic input; no human) | Human play-test; follow-camera without terrain clipping; character model; intent-based input; fixed-step simulation |
| 3. Movement and interaction | Walk/jump/step-up/borders; one "interact" = dig/place | Interact with items/creatures; reach rules in authoritative runtime |
| 4. Change voxel world | Dig/place with correct mesh/collision (REPORTED) | Through the **Editing API** (AOD-15) with validation and history; resource drops; material palette |
| 5. Save and reload | Edits survive real restart (REPORTED) | Approved persistence format with generator version; player position, inventory, equipment, stats; robust shutdown/autosave |
| 6. Minimal gameplay loop | **None** | Creatures (2), combat, loot, inventory, equipment slot, death/respawn, HUD/menus: the entire game-rules layer, which depends on Game Rules module contracts not yet approved |

**Minimal path from here** (INFERRED, no durations):

1. Human play-test of the current prototype to settle feel and camera (cheap, resolves UNKNOWNs).
2. **P-WORLD Port Slice** (section 10): world port behind a Rust-owned interface; edit through `cube-editing`.
3. Player/world save through the approved persistence direction (seed + generator version + journal) once the system-commit and persistence contracts exist.
4. Game-rules work (creatures, combat, items) *after* steps 2-3, because it will call the world port; building it earlier on the bypass would be rewritten.

The critical dependency is governance (PS-0 open, system-commit port and revision rule unapproved), as already noted in `TECHNOLOGY-GAME-TRACK-INTEGRATION.md` section 5.4. P-WORLD does not remove that dependency; it only shows the code behind it is small.

---

## 10. Proposed Next Stage

**Name:** P-WORLD Port Slice (L2, isolated, owner-authorised)

**Goal:** Show that the *same* P-WORLD behaviours (read, raycast, collide, dig/place, restart) work when the world is accessed only through a single Rust-owned "world port" and edits go through the existing `cube-editing` crate, and give the owner a human-played build to judge feel. No new content.

**Expected result:** A new isolated branch (not `spike/p-world`) with a small port interface in Rust, P-WORLD's tests adapted to it, and a clean-clone-verified start command. `spike/p-world` stays frozen as reference.

**Tasks (6):**

1. **Human play-test first (owner).** Owner plays the existing build for a session and records notes on controls, jump, camera. (Zero code; resolves the largest UNKNOWN.)
2. **Define the world-port as a Rust trait/struct** with the capabilities `read`, `raycast`, `collide(AABB)`, `edit(request)`, `persist` (see integration doc 5.1, a design artefact there); no Godot types; inputs sanitised (finite floats, capped ray distance).
3. **Back `edit` with `cube-editing`** (`EditOp` through `Editor`); document honestly how generated chunks still enter Core in the spike (system-commit stand-in, labelled), without choosing the AOD-20 port signature.
4. **Port tests, not code:** adapt the determinism/seam, raycast-vs-brute-force, remove/place rule, border-walk, unloaded-chunk and mesh-equals-rebuild tests to the port; carry over character constants as data.
5. **Remove per-frame global scans** (no `active_voxels()` in HUD); re-measure frame time at radius 2, 4, 6 on this build (one machine, label it preliminary).
6. **Clean-clone run** of the start command and a restart scenario (edit, close, restart) on a fresh clone.

**Dependencies:** owner go-ahead for a new L2 step; the owner's play-test notes; no ADG/PS changes needed.

**Acceptance criteria (section 11).**

**Stop conditions (section 12).**

**Explicitly out of scope:** fixing C-1..C-12 / A-1..A-8 in `spike/p-world`; merge or cherry-pick into master/docs branch; creatures, combat, items, inventory, menus; second biome; multiplayer/server runtime; the approved persistence format; fixed-tick design; LOD, floating origin, SDF/micro voxels; new documentation sets; any ADG/PS/governance change.

---

## 11. Acceptance Criteria

| # | Criterion | Evidence |
|---|---|---|
| 1 | Every voxel read/raycast/collision/edit used by the demo goes through the port; no call to `WorldStreamer::set_voxel` or direct Core commit from the player path | Code search + a test that fails if the player edit path changes Core without an `Editor` operation |
| 2 | Edits create `cube-editing` operations (with history and revision effect) | Unit test reading history after an edit |
| 3 | The adapted tests pass: determinism, seams (12 seeds), raycast vs brute force, rule list, border walk, unloaded chunk hold, mesh == rebuild | `cargo test` log saved outside the repo |
| 4 | Non-finite inputs and huge ray distances are rejected or clamped | Tests |
| 5 | Edit, close window, restart: edits present (real window, same scenario shape as before) | Saved logs, `RESULT: PASS` |
| 6 | Frame time at radius 2/4/6 re-measured on this build, no per-frame full-world scan | Saved log; numbers reported, not claimed as budgets |
| 7 | Clean-clone start works on this machine | Log of a fresh clone start |
| 8 | Owner play-test notes recorded and any change to constants made as data | Notes file |
| 9 | Report separates: automated / synthetic-input / human / one-process / after-restart / not verified | Report |
| 10 | No change to ADG, PS-0/PS-1, production docs, `spike/p-world`, or master | Git status/log evidence |

---

## 12. Risks and Stop Conditions

| Risk | Mitigation / stop |
|---|---|
| The port quietly becomes a new framework | Stop if it exceeds the six capabilities, or if a task needs a contract signature that is not approved |
| `cube-editing` cannot express the needed edit or has too much overhead | Stop and report; do not bypass; ask the owner whether a system-commit stand-in or a different path is wanted |
| Generated chunks still bypass the Editing API (AOD-20 port unapproved) | Allowed only as a labelled stand-in; stop if it needs a Core signature change |
| Performance worse than P-WORLD after porting | Report the measurement; stop and ask before optimising beyond removing global scans |
| Play-test shows the controls are unusable | Stop; constants/camera decisions go to the owner before more code |
| Scope drift toward gameplay (creatures, items) | Hard stop at the listed scope |
| Governance blocks (PS-0 open) make the port unusable for production | Expected; the result remains L2 knowledge, not code to merge |
| Single copy of `fea3deb` (all on drive `D:`, no remote) | Owner decides a second copy; I did nothing |
| Same-agent bias in the previous Owner Review | Owner may want a different reviewer for the stage acceptance |

General stop: any need to change source in `spike/p-world`, to touch ADG/PS, to push/merge, or to exceed 6 tasks -> stop and ask.

---

## 13. Open Owner Decisions

I make none of these; each is a question with my recommendation.

| # | Decision needed | My recommendation (not a decision) |
|---|---|---|
| OD-1 | Choose A, B, C or D for P-WORLD | C (narrow, section 10); A if no further code work is wanted now |
| OD-2 | Authorise (or not) the L2 "P-WORLD Port Slice" on a new isolated branch | Yes, with the stop conditions above |
| OD-3 | Which write path may the stage use for generated chunks, given AOD-20 variant C3 (port signature and revision rule not approved) | Labelled system-commit stand-in; Editing API for player edits only |
| OD-4 | Is a human play-test of the current build the first step, and who does it | Yes, owner |
| OD-5 | Storage of the result: second copy (other drive or remote), logs/screenshots copied out of the temporary folder, keep or remove the `Cube-pworld` worktree | Copy evidence out; keep worktree until the stage ends; second copy preferred |
| OD-6 | ODM-11: coordinate width, vertical extent, world bound (affects terrain, C-4, A-6) | Defer; the stage uses the spike's 32-high layout labelled as such |
| OD-7 | Is the finding "a visible, walkable, editable, restartable world is reachable with small code" accepted as knowledge flowing back (integration doc rule 2)? | Yes |
| OD-8 | Which tests and requirements from P-WORLD become acceptance scenarios (list in section 6) | Accept the list as a proposal |
| OD-9 | Does the owner want an independent reviewer (different from the author) for the stage acceptance | Yes |
| OD-10 | Is the "single-player embedded server" proposal wanted, since it decides where character physics and edits live | Decide before game-rules work, not before this stage |

No ADG, PS-0, PS-1 or production document has been changed by this report; no stage starts without a separate owner decision.
