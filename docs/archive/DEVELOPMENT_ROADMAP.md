# Development Roadmap: from the current state to Release 1.0

**Status:** **Proposed.** No milestone is reached; none may be declared reached before the owner agrees the criteria in [`RELEASE_CRITERIA.md`](../RELEASE_CRITERIA.md) and accepts evidence. **No dates are given:** there is no benchmark, prototype or estimate in the repository that could support one (see section 4 for what is needed).

## 1. Two planning levels (not one-to-one)

* **Production Stages PS-0…PS-14** (Production Plan v2.1, ADG) describe how *technical subsystems* are built and merged. They end at PS-14 "Full QA and Release Validation" and cover Core, Processing, Editing API, Orchestration, backends, tools, history, composite, resampling, persistence, world generation integration, client/adapter, server/networking, QA.
* **Release Milestones M0…M8** describe how ready *the game* is for testers and players. A milestone draws on several stages, and a stage can contribute to several milestones. One milestone may need only **a slice** of a stage; the ADG currently contains **no slice definitions** (finding R-01 below).

**Scope warning.** The Production Plan contains **no gameplay stages**: combat, creatures/AI, items and equipment, RPG progression and classes, dungeons/bosses, quests/world content, UI/HUD, audio, accounts/matchmaking/server browser, monetisation (if any), live-ops, localisation, anti-cheat, regional sharding for 500 players per logical world. These are required by M3…M8 and by the project pillars. They are listed as **Unplanned work packages** (section 3) and need their own plan and owner decision. Nothing is removed from the project scope here; no requirement is lowered.

## 2. Milestone table

Legend for "Depends on": PS-n = stage gate Gn satisfied; "slice" = only part of the stage, to be defined (R-01).

| Milestone | Depends on (technical) | Additionally needs (unplanned / external) |
|---|---|---|
| M0 Architecture & Governance | PS-0 (G0) | owner decisions B (15), owners (GOV-03), ADG correction (D-01/D-02), ST-1 |
| M1 Core Technology Prototype | PS-1, PS-2, PS-3, PS-4, voxel part of PS-5 (VOX-*), tests TEST-02/05/06/08/03a | Geometry Processing Spec (EXT-GPS) for PS-2; RG-1 for backends |
| M2 First Playable Prototype | M1; PS-10 (save/load via persistence, `PERS-*`); PS-12 slice (client runtime, Godot adapter, mesh upload, input); PS-13 slice (embedded server host SRV-01 and in-process transport, because AOD-22 makes single-player run the authoritative runtime in-process); test-world source: either a PS-11 slice (WGEN-01…03) or a fixed test-world fixture (owner choice) | unplanned: character controller/movement, minimal interaction UI, minimal playable build pipeline |
| M3 Vertical Slice | M2; PS-6 (tools), PS-7 (history), PS-11 (generation), PS-8 as needed for building | all gameplay packages G-1…G-9 at slice depth (section 3); art/content; GDD/TDD (BLK-06) |
| M4 Internal Alpha | M3; PS-13 (server, netcode), PS-9 as needed, ITG gates | content breadth, automated test infrastructure for gameplay, crash/telemetry hooks (G-10) |
| M5 Closed Alpha | M4; DG-1…DG-3 (devices, requires DEV-01); PS-14 partial | update/recovery procedure, crash reporting, performance telemetry, tester distribution |
| M6 Beta | M5; PS-14; gameplay breadth complete | cross-play, load tests up to the agreed player target, matchmaking/server browser (G-8) |
| M7 Release Candidate | M6; PS-14 full; PC-1 frozen via GOV-15; all formats Owner-Approved | scope freeze, security review, store/compliance, release infrastructure |
| M8 Release 1.0 | M7 | operations, support, documentation, live infrastructure |

## 3. Milestone specifications

Twelve fields each. "Req" = requirement. Numeric thresholds are **never invented here**; they are "TBD by benchmark B-n" and become binding only through the procedure in [`RELEASE_CRITERIA.md`](../RELEASE_CRITERIA.md) section 1.

### M0: Architecture & Governance

1. **Goal:** the project can start production implementation on an agreed base.
2. **Content:** decision registry with the 15 PS-0 decisions; P0-alpha Owner-Approved, PC-1 Provisional; status model adopted; owners assigned or deferred in writing; docs-only baseline; CI specification and a running CI with required status checks; archcheck v0; corrected ADG.
3. **Depends on:** GOV-01…06, TEST-01, XT-01 (gate G0); DEV-01 and SPK-PERS may run in parallel.
4. **Acceptance:** every G0 member merged and accepted; registry entries carry owner confirmation sources; a workflow run is green on the skeleton and red on seeded failures; branch protection on the production base lists the required checks; ADG validator extended with the stage-chain checks.
5. **Tests:** archcheck seeded-violation self-tests; CI seeded failures (format, clippy, test, forbidden dependency, forbidden `use`, licence).
6. **Performance:** none.
7. **Platform:** CI on the project's chosen host(s) (D-E).
8. **Multiplayer:** none. 9. **Persistence:** none.
10. **Blockers:** BLK-01, 02, 03, 07, 08, 09, 11.
11. **Evidence:** registry commit hash; workflow-run URL; protection-settings JSON; validator output.
12. **Next:** G0 satisfied → PS-1.

### M1: Core Technology Prototype (technical, not a game)

1. **Goal:** prove the geometry kernel and its mutation path on the production architecture.
2. **Content:** production `cube-core` (assets/layers, 16³ bricks, revisions, commit, snapshots, mutation boundary); Processing foundation; Editing API (types, ports, authorization); orchestration for voxel edits; voxel backend; benchmark harness.
3. **Depends on:** G1, G2, G3, G4, VOX-01…05 of G5; EXT-GPS (spec), EXT-VA-RG1 for non-provisional parts.
4. **Acceptance:** all member tasks merged and accepted; contract tests for P0-alpha and PC-1 slice; property tests (atomicity, snapshot isolation, monotonic revisions, negative coordinates); loom tests of the commit gate; compile-fail tests prove only `UserCommit`/`SystemCommit`/`RecoveryCommit` holders can mutate; API snapshot enforced in CI.
5. **Required tests:** unit, property, loom, trybuild, golden-hash, spike scenarios replayed through the new path with equal content hashes.
6. **Performance:** benchmark suite B-1…B-4 (commit, snapshot, processing, memory) run on the CI host; thresholds TBD by benchmark and approved by the owner; spike numbers are reference points only.
7. **Platform:** desktop CI only; no device or emulator condition.
8. **Multiplayer:** none (types for `OperationId`/`ActorKind` exist). 9. **Persistence:** none (export/import views and content hash only).
10. **Blockers:** M0; BLK-04, BLK-05.
11. **Evidence:** CI runs, benchmark reports, `INT-01` status record.
12. **Next:** M2.

### M2: First Playable Prototype

1. **Goal:** a launchable build in which a person can play with the voxel world.
2. **Content (must be demonstrable):** start the game; create or load a test world; move a character; interact with the voxel environment; edit geometry (place/remove); save and load the edits.
3. **Depends on:** see section 2 (M1 + PS-10 + client/adapter slice + embedded-server slice + test-world source).
4. **Acceptance:** scripted playthrough (record) executes the six verbs above; edited world reloaded after a restart is byte-hash identical to before; crash during save recovers per AOD-21 protocol; every edit goes through the Editing API (archcheck TEST-10 green); client holds no authoritative state.
5. **Required tests:** end-to-end test ITG-01, persistence crash matrix (crash after each AOD-21 step), adapter tests (no write path or history), smoke test of the build on the target desktop OS.
6. **Performance:** frame time and edit-latency budgets TBD by benchmark B-5/B-6 on the minimum desktop spec (to be defined by the owner).
7. **Platform:** desktop (PC) only. Android/iOS **not** claimed.
8. **Multiplayer:** none claimed; embedded server runs in-process; single-player only.
9. **Persistence:** snapshots + journal per AOD-21; no binary format is Owner-Approved until GOV-10.
10. **What is still missing (must be stated in the milestone report):** combat, creatures, items, progression, biomes beyond the test world, UI beyond debug, audio, multiplayer, mobile, dedicated server, content.
11. **Blockers:** M1, GOV-03 owners (client/adapter/server), GOV-10, R-01 slice definition, BLK-05.
12. **Next:** M3.

### M3: Vertical Slice

1. **Goal:** a small but complete game fragment demonstrating the pillars.
2. **Content:** a small region with several biomes/representative environments; character; basic combat; several creatures; items and equipment; basic RPG progression; exploration; building and destruction; world save; a full play loop.
3. **Depends on:** M2, PS-6, PS-7, PS-11 (generation, with ACR-004 B/E done: no direct write bypass), PS-8 as needed; unplanned packages G-1…G-9 at slice depth; GDD/TDD.
4. **Acceptance:** a defined play session (scenario script approved by the owner) completes without developer help; each pillar listed has at least one demonstrated, tested feature; generated and user-edited data persist and reload identically.
5. **Tests:** scenario automation, generation determinism (same seed + generator version = same hash), ITG-02, regression suite from M1/M2.
6. **Performance:** budgets approved after benchmarks on desktop and at least one real Android device class (EXT-DEV) if mobile is in the slice; otherwise M3 states desktop only.
7. **Platform:** desktop; mobile only if the owner includes it.
8. **Multiplayer:** optional (2-player embedded host) if the owner includes it.
9. **Persistence:** world + edits + character/inventory state.
10. **Blockers:** BLK-06, BLK-07, unplanned packages.
11. **Evidence:** scenario run recordings, test logs, hash comparisons.
12. **Next:** M4.

### M4: Internal Alpha

1. **Goal:** the integrated game can be played through its main loops internally.
2. **Content:** all core systems integrated; generation + persistence working; basic dedicated/server architecture; automated tests; critical blockers resolved.
3. **Depends on:** M3; PS-13 (NET, SRV, conformance on both transports and both hosts); PS-9 if refinement is used.
4. **Acceptance:** main loops pass automated and manual scripts; no open Critical defect (severity scale approved by the owner); server authority conformance green on embedded and dedicated hosts; regression suite in CI.
5. **Tests:** TEST-03b transport suite, TEST-07, QA-02/QA-04 subsets, fuzz of protocol and persistence loaders.
6. **Performance:** server tick and bandwidth budgets TBD by benchmark B-7…B-9; initial load tests (count of simulated clients TBD).
7. **Platform:** desktop; Android/iOS if declared.
8. **Multiplayer:** co-op with dedicated and embedded host.
9. **Persistence:** recovery from kill -9, disk-full, corrupted snapshot, truncated journal (tests).
10. **Blockers:** M3, unplanned packages G-8, G-10.
11. **Evidence:** CI history, load-test reports, defect list.
12. **Next:** M5.

### M5: Closed Alpha

1. **Goal:** real testers on real builds.
2. **Content:** distribution to a limited group; crash reports; performance metrics; update and data-recovery procedure.
3. **Depends on:** M4; DG-1…DG-3 (device reports); GOV-13 (renderer default) as applicable.
4. **Acceptance:** build is installed by testers on each declared platform; crash reports and metrics are received (proof: ingested reports); an update from build N to N+1 preserves worlds (migration test); a deliberately corrupted world is recovered or rejected cleanly.
5. **Tests:** device matrix runs, upgrade/migration tests, soak tests.
6. **Performance:** device budgets approved from DG data (never from the desktop spike).
7. **Platform:** PC plus those of Android/iOS that have devices; iOS needs macOS/Xcode (NB-2).
8. **Multiplayer:** real network conditions; cross-play test between platforms available.
9. **Persistence:** live data migration policy (per-world generator-version policy D-P).
10. **Blockers:** BLK-10, privacy/legal requirements for telemetry (unplanned G-11).
11. **Evidence:** tester logs, crash dashboards, migration test results.
12. **Next:** M6.

### M6: Beta

1. **Goal:** feature complete for the declared scope; long-duration testing.
2. **Content:** main systems implemented; declared platforms; cross-play; multiplayer under agreed load; no known Critical blockers.
3. **Depends on:** M5; PS-14.
4. **Acceptance:** load test reaches the agreed concurrent-player target (the project requirement is up to 500 players per logical world with load distribution; **how the logical world is distributed across regions/servers is not designed in the production plan**: unplanned G-8); cross-play matrix green; zero open Critical, defect budget for Major set by the owner.
5. **Tests:** load and soak at target, security tests (authority, replay, malformed input), compatibility tests.
6. **Performance:** budgets as approved; regression thresholds enforced in CI.
7. **Platform:** all declared.
8. **Multiplayer:** full.
9. **Persistence:** online migration tests, backup/restore.
10. **Blockers:** G-8 (distribution), security review.
11. **Evidence:** load-test reports, security report.
12. **Next:** M7.

### M7: Release Candidate

1. **Goal:** scope frozen; shippable build.
2. **Content:** only fixes; release builds; update path; release infrastructure ready.
3. **Depends on:** M6; GOV-15 (PC-1 frozen after RG-1); all formats Owner-Approved; G14.
4. **Acceptance:** every Release Criteria item for RC in [`RELEASE_CRITERIA.md`](../RELEASE_CRITERIA.md) passed; changelog shows no feature commits after freeze.
5. **Tests:** full regression, upgrade matrix, clean-install matrix, recovery matrix.
6. **Performance/7/8/9:** as approved; unchanged from M6 or better.
10. **Blockers:** open Critical/Major defects; legal.
11. **Evidence:** RC report signed by the owner.
12. **Next:** M8.

### M8: Release 1.0

1. **Goal:** the game is available to the target audience.
2. **Content:** client builds for declared platforms; servers and infrastructure; update and support operations; operational documentation.
3. **Depends on:** M7.
4. **Acceptance:** Release Criteria for 1.0; operational runbooks exercised; rollback rehearsed.
5…9. as M7 plus live operations monitoring.
10. **Blockers:** store approvals, infrastructure readiness.
11. **Evidence:** release checklist with owner sign-off.
12. **Next:** live operations (outside this roadmap).

## 4. Planning gaps (not hidden)

| ID | Gap | Proposal (needs owner decision) |
|---|---|---|
| R-01 | The ADG has no milestone slices (e.g. M2 needs only part of PS-12/PS-13) | add a "milestone slice" attribute listing task subsets per milestone and validate that each slice is a closed subgraph |
| R-02 | No gameplay/content stages exist (G-1…G-11) | write a Gameplay Production Plan after GDD/TDD is supplied; until then M3+ are "scope Unknown" |
| R-03 | Logical-world distribution for 500 players per world is not designed | architecture study + owner decision before PS-13 detailed design; a prototype benchmark first |
| R-04 | No schedule can be given | required: (a) benchmark results B-1…B-9, (b) a size estimate per task by the assigned owners, (c) a velocity measurement from PS-0/PS-1, (d) owners for unassigned areas |
| R-05 | Gate-chain defects D-01/D-02 | correct before scheduling |

Unplanned packages: G-1 character & controller, G-2 combat, G-3 creatures/AI, G-4 items/equipment, G-5 RPG progression/classes, G-6 world content (biomes, dungeons, bosses), G-7 UI/HUD, G-8 online services (accounts, matchmaking/server browser, regional distribution), G-9 audio/VFX/art pipeline, G-10 telemetry/crash reporting, G-11 legal/privacy/store compliance, G-12 localisation, G-13 anti-cheat/security operations.

## 5. Path summary

`M0 (PS-0) → M1 (PS-1…PS-4, voxel PS-5) → M2 (+PS-10, slices of PS-12/13) → M3 (+PS-6/7/8/11, gameplay) → M4 (+PS-13, integration) → M5 (+devices, DG-1…3) → M6 (+PS-14, cross-play, load) → M7 (freeze, GOV-15) → M8`.
First Playable = M2. Vertical Slice = M3. Release 1.0 = M8.
