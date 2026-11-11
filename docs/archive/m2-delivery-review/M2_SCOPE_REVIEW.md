# M2-T and M2: scope review

**Status:** **Proposed** / **Pending Owner Approval**. No official milestone is changed. Task ids refer to `M2_TASK_GRAPH.md`; contracts to `M2_MINIMUM_CONTRACTS.md` (TC-nn); decisions to `OWNER_DECISIONS_REQUIRED.md` (ODM-nn).
**Reading rule:** existing text = `FIRST_PLAYABLE_SCOPE.md` (FPS-01...FPS-17, itself Proposed, commit `362a124`); this document reviews it and adds the technical checkpoint.

## 1. Two checkpoints, kept apart

| | **M2-T Technical Playable** | **M2 First Playable** |
|---|---|---|
| Question it answers | Does the real stack (authoritative runtime, generation, edit path, rendering, persistence, client) work together with a character in a small world? | Is there a game: a complete loop with limited content that a stranger can play? |
| Gameplay | none beyond walking, jumping, digging and placing | fight, loot, equip, build, die and respawn, continue |
| Nature | engineering checkpoint, **not** First Playable and not to be presented as one | product milestone |
| Evidence | scripted + manual run, stability report | FPS-01...FPS-17, outside-tester session |
| Who accepts | Integration Owner (engineering) records; Project Owner is informed | Project Owner records acceptance |

M2-T is a *subset* of M2: everything built for M2-T stays; M2 adds the Game Rules layer on top.

## 2. M2-T: review of the proposed scope

The ten items of the brief, reviewed against the architecture:

| # | Item | Verdict | Remark |
|---|---|---|---|
| 1 | The client starts | keep | one desktop platform; reproducible build from a clean checkout is added (T-12) |
| 2 | Create **or** load a small world | **change to "create **and** load"** | loading after a restart is the proof of persistence (item 8); a world needs a name and a seed |
| 3 | A limited play area is generated | keep, **define the extent** | the vertical extent and chunk layout are not decided (the spike's column is 32 voxels high); decision ODM-11 / TC-04. One terrain variant is enough for M2-T; two are M2 |
| 4 | The player moves through the world | keep, **as a walking character with collision and jump** | a free-fly camera would be cheaper but would leave collision, the controller and tick-based movement unproven; fly-cam is a fallback the owner may choose |
| 5 | Voxel geometry is displayed correctly | keep | `faces` mesher only; "correctly" = no holes or seams across chunk borders, negative coordinates included |
| 6 | The player destroys and places voxels | keep, **through the Editing API** | AOD-15; the commit log must show user commits only |
| 7 | The world is saved | keep | snapshot-only persistence v0 (TC-08), atomic publish |
| 8 | The world loads after restart | keep | content hash equal before and after |
| 9 | Main operations do not block the main thread | keep, **make it testable** | requires the threading contract (TC-10) and a stated rule: the engine thread only polls results; generation, meshing, save and load run on executor threads. Evidence = architectural audit + a measured frame-time trace during streaming (**reported, not budgeted**) |
| 10 | Minimal stability check | keep, **define it** | an automated scripted loop (traverse, edit, save, reload, repeat) with a trend report of memory and errors; success = completes without crash or error in the log; **no thresholds** |

Added (small, because they prevent rework): T-11 authority audit (no gameplay or world logic in engine scripts; no client-side world mutation); T-13 written mobile-impact statement (AOD-08). Removed from consideration: anything about creatures, combat, loot, progression, UI beyond menus, audio.

**Is M2-T really minimal?** Mostly yes. Its largest cost is not the player but the **authoritative embedded runtime**: `SRV-01`, `SRV-03`, `SRV-04`, `NET-01..03`, `TGAP-03a`, `CLI-05` (nine tasks, all without owners). A shortcut (calling the world directly from the client with no server runtime) would be cheaper now, but it creates the second authority path and the client-owned state that AOD-13, AOD-14 and AOD-22 forbid, and it would have to be thrown away. Recommendation: keep the embedded runtime with the in-process transport (`NET-03`), the simplest transport that exists. Prediction and reconciliation (the rest of `CLI-05`) stay trivial because in-process latency is nil.

### 2.1 M2-T acceptance criteria (proposed)

| ID | Criterion | Evidence | Kind |
|---|---|---|---|
| T-01 | The client starts on the declared desktop platform from a clean build and reaches a menu | scripted smoke run | S |
| T-02 | A world is created from a name and a seed; the same seed gives the same terrain (chunk hashes equal), a different seed does not | determinism suite (`TEST-09`) | S |
| T-03 | The area around the player is generated on demand and unloaded behind | integration scenario `XI-01` | S |
| T-04 | The character walks, jumps, collides with terrain; the camera follows | recorded manual run + replay test of the controller | M/S |
| T-05 | Terrain renders without holes or seams across chunk borders and at negative coordinates | screenshots along a defined route; mesh tests | M/S |
| T-06 | Voxels are destroyed and placed; every change is a user commit through the Editing API | commit log shows only `UserCommit`; `TEST-10` | S |
| T-07 | Save publishes atomically (previous or new state, never a mix) | persistence tests | S |
| T-08 | After a restart the world equals the saved one (content hash) | `XI-01` | S |
| T-09 | No generation, meshing, save or load call runs on the engine main thread | audit of the adapter and client + frame-time trace (reported) | S/M |
| T-10 | The stability harness completes its loop without crash or logged error and produces a trend report | `TGAP-06` | S |
| T-11 | No gameplay or world logic in engine scripts; the client cannot change world state | `XT-02`, `TEST-06`, `TEST-10`, adapter audit | S |
| T-12 | A clean checkout builds the playable build with one documented command | `TGAP-04` from a clean clone | S |
| T-13 | A written mobile-impact statement lists what would block an Android build | document | M |

## 3. M2: review of FPS-01...FPS-17

Columns: **M** = mandatory for M2 as proposed; **M2-T** = part of M2-T; **Deps** = graph tasks; **Modules**; **Owner** category (`M2_OWNERSHIP_MATRIX.md`); **Acceptance** (short); **Test** (S scripted, M manual, O outside tester); **Blocking decisions**; **Deferral**.

| ID | M | M2-T | Deps | Modules | Owner | Acceptance | Test | Blocking decisions | Can move later? |
|---|---|---|---|---|---|---|---|---|---|
| FPS-01 Launch and menus | yes | yes | GM-15, ADP-02, TGAP-04 | client, adapter, UI | Owner Required (UI/client) | menu reached, start, quit | S/M | ODM-04 | no |
| FPS-02 New world by seed | yes | partly (one variant) | WGEN-01..04, CORE-16, PERS-08, TEST-09, GM-15; **TGAP-07** for two variants | worldgen, persistence, UI | Owner Required (WorldGen) | equal chunk hashes for equal seeds | S | GOV-21, GOV-11, ODM-11, D-K | the second variant (TGAP-07) is M2-only |
| FPS-03 Control | yes | yes | GM-11, GM-12, GM-14, VOX-05, SRV-03, CLI-04 | Game Rules, client | Owner Required (Gameplay, UI) | walk/jump/step; camera outside terrain | M + replay test | ODM-05, ODM-11 | no |
| FPS-04 Exploration | yes | borders yes, variants no | WGEN-02/04, CLI-01/02, VOX-02/04, TGAP-07 | worldgen, client, backend | Owner Required | route with no holes or seams | S+M | ODM-11 | variant boundary part: M2 |
| FPS-05 Encounter | yes | no | GM-25, GM-26, TGAP-08, GM-30, GM-32 | Game Rules, worldgen | Owner Required | both creature kinds met on the route | M | ODM-05 | could shrink to the hostile kind (the owner may drop the passive one) |
| FPS-06 Combat | yes | no | GM-23, GM-24, GM-28 | Game Rules | Owner Required | win, lose and respawn | M/S | ODM-05 | no (core of the game) |
| FPS-07 Loot and inventory | yes | no | GM-20, GM-21, GM-27, GM-31 | Game Rules, UI | Owner Required | drop picked up, stacks | S/M | ODM-13 (data format when needed) | no |
| FPS-08 Equipment effect | yes | no | GM-22 | Game Rules | Owner Required | stat changes, fight shortens | S/M | | no |
| FPS-09 World modification via Editing API | yes | **yes** | GM-13, TGAP-01, ORCH-01..05, VOX-01, TEST-10 | Game Rules, orchestration | Owner Required (Gameplay) + Existing (Orchestration) | commit log shows user commits only | S | GOV-11, ODM-07 | no |
| FPS-10 Save | yes | yes | PERS-01..03, PERS-08, GM-15 | persistence, UI | Existing (Persistence) | save through the menu | S | ODM-03, ODM-07 | no |
| FPS-11 Reload and continue | yes | world: yes; player state: M2 | PERS-*, GM-29, TGAP-02 | persistence, Game Rules | Existing + Owner Required | world, position, inventory, equipment, stats restored | S/M | ODM-07 | player-state part: M2 |
| FPS-12 Crash during save | yes | atomic publish: yes; full step matrix: M2 | PERS-04..06, TEST-04, QA-P2 | persistence, QA | Existing (Persistence, QA) | recover to a verified state; never an unverified snapshot | S | ODM-03 | the **broader matrix** (truncated tail, disk full, version mismatch) can move to M3 (section 6) |
| FPS-13 Authority | yes | **yes** (T-11) | XT-02, TEST-06, TEST-10, ADP-02 | all | Existing (Integration Owner, QA) | no gameplay in scripts; no client mutation | S | none | no |
| FPS-14 Outside tester | yes | no | XI-04 | all | Project Owner / Integration Owner | 20+ minutes without instruction, observations recorded | O | none | could move to M3 if no outside tester is available; M2 would then be "developer-demonstrated" (owner decision) |
| FPS-15 Declared absences | yes | yes | documents | documents | Existing (Integration Owner) | exclusions and simplification register published | M | none | no |
| FPS-16 Measured, not budgeted | yes | yes (frame trace, T-09) | benchmarks B-5, B-6, B-10 | QA | Existing (QA) | numbers measured and reported on named hardware | S | none | thresholds later |
| FPS-17 Mobile impact | yes | yes (T-13) | document | all | Existing (Integration Owner) | statement per workstream | M | ODM-08 | the device checkpoint itself: section 5 |

Scope remarks on M2:
* **The weakest criterion is FPS-14** (needs an outside person). Keep it, but let the owner decide whether a developer-only first reading is acceptable for M2 acceptance.
* **The passive creature (FPS-05) is the most deferrable content**: it exists to prove that creatures are not all enemies and to give a second drop source. If time pressure appears, drop it before dropping anything in combat, loot or equipment.
* "Two terrain variants" (FPS-02/04) costs new generator content (`TGAP-07`) but tests the variant boundary, a classic seam source; keep.
* XP and level-ups stay a stretch, as proposed.

## 4. Crash recovery: minimal reliability set

AOD-21 (recorded, awaiting confirmation) fixes the architecture: seed plus generator version, a durable journal, compaction by snapshot-verify-publish-truncate. Two sources constrain the order: the journal is the primary record and snapshots an optimisation (`worldgen-core-boundary.md`); the temporary snapshot-only persistence of M2-T therefore **deviates** from the architecture and is allowed only as a registered temporary contract (TC-08) that expires at M2.

| Level | Contains | Not yet |
|---|---|---|
| **M2-T** | atomic manifest publish; per-file checksum; load verifies before use; stale or partial files are ignored; a crash at any point of a save leaves the previous complete state | durability of edits since the last explicit save; journal; compaction |
| **M2** | durable journal for user commits; recovery by base, snapshot, journal tail; compaction steps (write, fsync, verify, publish, fsync, truncate) with a kill after each step; corrupted snapshot detected and rejected | disk-full, permission errors, version-mismatch matrix, concurrent writers |
| **M3 and later** | the full matrix (`QA-P2` complete): truncated tail, corrupted files, version mismatch, disk full, repeated crashes; migration | |

## 5. Android: desktop acceptance and a mobile checkpoint

AOD-08 (recorded) makes Android and iOS mandatory and requires every runtime-touching gate to state its mobile impact; AOD-11 says an emulator never replaces a physical device. Neither says that *each* milestone must be accepted on a device. Proposal (Android remains a target platform):

| Checkpoint | When | Content | Requires |
|---|---|---|---|
| desktop acceptance | M2-T, M2 | all criteria on one desktop platform; mobile-impact statement (T-13, FPS-17); no desktop-only assumption in rules or protocol | none |
| **MAC-1 device smoke** | between M2 and M3 | the M2 build installs and runs on a physical Android ARM64 device; touch mapped to the existing intents; terrain renders; no crash during a short scripted loop | device (`DEV-01`), build pipeline, device scripts (`QA-06` family); **owner action: provide a device** |
| MAC-2 device baseline | during M3 | measurements on the device class (B-11); budgets proposed from data | MAC-1 |
| MAC-3 device acceptance | M5 | `DG-1..DG-3` | `DEV-01`, macOS for iOS |

The checkpoint is non-blocking for M2 acceptance but **blocking for starting M3 content work that assumes mobile** (owner decision ODM-08). Roadmap-v1 gate 6 wanted a device in the first playable loop; the proposal relaxes that for M2 only and moves it to MAC-1.

## 6. Open points

* ODM-11 vertical extent and world bound; D-K widths (GOV-01 item 10).
* The extent of "small world": a streaming radius around the player is enough for M2-T; no fixed world size is proposed.
* Whether the owner wants M2-T accepted separately (recommended: yes, it is the first moment real feedback on the stack exists).
* No effort or duration is known for any task.
