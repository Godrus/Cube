# Release Criteria

> **Нумерация:** M0..M8 в этом файле это вехи готовности к релизу (Internal Alpha, Closed Alpha, Beta, RC, 1.0), а не продуктовые M0..M12 из `docs/roadmap/PRODUCT-ROADMAP.md`. Сводка пути к релизу: последний раздел того файла.

**Status:** **Proposed.** No milestone is declared reached. A milestone is reached only when the owner records acceptance of its evidence. Words like "stable", "optimised enough", "ready" are not criteria; each item below is a check with a pass condition and evidence.

## 0. Rules for all criteria

* **R0.1** A criterion is *passed* only with evidence: a command output, CI workflow-run URL, report file in the repository, or an owner registry entry.
* **R0.2** **No number is invented here.** Where a number is needed it is written `[B-n]`. The value is set by this procedure: (1) the owner names the target device/server class; (2) Agent I runs benchmark B-n on that class and publishes the raw result; (3) the owner approves a budget in the decision registry (value, device class, measurement method, tolerance); (4) CI enforces it as a regression threshold. Until step 3, the criterion reads "measured and reported", not "passed".
* **R0.3** Severity scale (Critical/Major/Minor) and the defect budget per milestone are decided by the owner before M4.
* **R0.4** Criteria never weaken pillars. A scope change is a separate owner decision recorded as such.
* **R0.5** Spike numbers (e.g. chunk commit 0.009 ms) are reference points only.

## 1. Benchmark register (values undefined by design)

| ID | Measures | Where it must run | Needed by |
|---|---|---|---|
| B-1 | commit latency per brick/op batch; snapshot capture cost | CI host | M1 |
| B-2 | processing (dirty-region derive, mesh/derived data) throughput | CI host | M1 |
| B-3 | memory per asset/layer for reference worlds | CI host | M1 |
| B-4 | content hash and export/import time | CI host | M1 |
| B-5 | client frame time with reference view distance | minimum desktop spec (owner-defined) | M2 |
| B-6 | edit round-trip latency (input to visible change) | desktop; later device | M2 |
| B-7 | server tick time vs. players and edits | dedicated host class (owner-defined) | M4 |
| B-8 | bandwidth per player (up/down) | network emulation | M4 |
| B-9 | concurrent players per logical world and per server | load rig | M6 |
| B-10 | world load/save time, journal size growth, compaction time | desktop + device | M2, M5 |
| B-11 | mobile frame time, thermal behaviour, memory | real Android ARM64 devices, iOS devices | M5 |

## 2. Criteria per milestone

Each row: **ID | Check | Pass condition | Evidence**.

### 2.1 First Playable (M2)

| ID | Check | Pass condition | Evidence |
|---|---|---|---|
| FP-1 | Launch | a built binary starts on the declared desktop OS from a clean install and reaches the world view | scripted smoke run log |
| FP-2 | World create/load | a test world is created from a seed or fixture and loaded after restart | test log + world hash |
| FP-3 | Movement | scripted input moves the character through a defined path, collision with voxels respected | scenario test |
| FP-4 | Voxel interaction | place and remove operations change geometry and are visible | scenario + screenshot hash |
| FP-5 | Edit path | 100% of edits pass through the Editing API; archcheck and TEST-10 green; commit log complete | CI URL |
| FP-6 | Save/load | after save, restart and load, content hash of every edited asset equals the pre-save hash | test log |
| FP-7 | Crash during save | crash injected after each AOD-21 step; recovery yields a verified snapshot or the previous state, never an unverified one | crash matrix report |
| FP-8 | Authority | the client contains no authoritative geometry mutation (adapter has no write path) | archcheck + TEST-10 |
| FP-9 | Performance reported | B-5, B-6, B-10 measured and published; budgets approved or marked "reported only" | benchmark files |
| FP-10 | Missing items listed | the milestone report lists everything absent (combat, items, progression, multiplayer, mobile, content) | report |

### 2.2 Vertical Slice (M3)

| ID | Check | Pass condition | Evidence |
|---|---|---|---|
| VS-1 | Owner-approved scenario script exists | the play session covers exploration, build/destroy, combat, loot/equipment, progression, save/load | script file |
| VS-2 | Unassisted run | a tester not involved in development completes the script without intervention | recording + signed report |
| VS-3 | Content | at least the number of biomes/creatures/items the owner puts into the slice definition | slice definition + inventory list |
| VS-4 | Determinism | same seed + generator version gives identical hashes; user edits survive regeneration/eviction | ITG-02 + tests |
| VS-5 | Persistence | world, edits, character and inventory reload identically | hash comparison |
| VS-6 | Regression | all M1/M2 tests still green in CI | CI URL |
| VS-7 | Performance | budgets `[B-5, B-6, B-10]` approved and met on the declared device classes | benchmark reports |

### 2.3 Internal Alpha (M4)

| ID | Check | Pass condition | Evidence |
|---|---|---|---|
| IA-1 | Core loops | scripted runs of every main loop pass in CI nightly | CI URLs |
| IA-2 | Server authority | conformance suite green on embedded and dedicated hosts and on both transports | TEST-03b/TEST-07 reports |
| IA-3 | Recovery | kill -9, disk full, truncated journal, corrupted snapshot: recovery or clean rejection | crash matrix |
| IA-4 | Defects | zero open Critical; Major within owner budget | defect export |
| IA-5 | Automation | CI runs fmt, clippy, tests, deny, archcheck as required checks; branch protection export attached | JSON |
| IA-6 | Co-op | two or more clients on a server complete the core loop | scenario |
| IA-7 | Protocol robustness | fuzz corpus on protocol and persistence loaders run N minutes without crash (`N` approved by the owner) | fuzz report |

### 2.4 Closed Alpha (M5)

| ID | Check | Pass condition | Evidence |
|---|---|---|---|
| CA-1 | Distribution | a signed/installable build per declared platform reaches testers | distribution log |
| CA-2 | Crash reporting | crash reports from testers arrive and are symbolicated | dashboard export |
| CA-3 | Metrics | performance metrics from devices arrive for B-11 | dashboard export |
| CA-4 | Update | update N to N+1 preserves worlds (migration tests) | test log |
| CA-5 | Recovery | restore from backup procedure executed on a real world | runbook record |
| CA-6 | Device gates | DG-1, DG-2, DG-3 reports exist | reports |
| CA-7 | Multiplayer under real network | latency/jitter/loss profile tests pass | report |

### 2.5 Beta (M6)

| ID | Check | Pass condition | Evidence |
|---|---|---|---|
| BE-1 | Load | the agreed concurrent-player target [B-9] reached per logical world with distribution across servers/regions as designed | load report |
| BE-2 | Cross-play | PC, Android, iOS matrix: join, play, edit, save across platforms | matrix report |
| BE-3 | Platforms | each declared platform builds, installs, runs the full loop | CI/device logs |
| BE-4 | Dedicated and private servers | operator can create a private server from documented steps | runbook test |
| BE-5 | Single-player and co-op | core loop on embedded host passes with identical semantics (AOD-22) | conformance |
| BE-6 | Security | authority, replay, malformed input, rate limiting tests pass; independent review done | review report |
| BE-7 | Defects | zero Critical, Major within budget | defect export |
| BE-8 | Soak | server soak for the owner-defined duration without leak/regression beyond tolerance | soak report |

### 2.6 Release Candidate (M7)

| ID | Check | Pass condition | Evidence |
|---|---|---|---|
| RC-1 | Scope freeze | no feature change after the freeze commit (only fixes) | git log |
| RC-2 | Contracts | all contracts used in shipped code Owner-Approved; PC-1 Frozen via GOV-15 after RG-1/GOV-08; all formats Owner-Approved | registry |
| RC-3 | Compatibility | upgrade from every supported previous build; worlds from earlier alpha/beta load or migrate | matrix |
| RC-4 | Performance | all approved budgets met on all device classes | benchmark reports |
| RC-5 | Stability | crash-free rate threshold approved by the owner is met over the owner-defined period | telemetry export |
| RC-6 | Security | no open High findings; dependency advisories clean (`cargo deny`) | reports |
| RC-7 | Release builds | reproducible builds from tagged commit; licence audit clean | build logs |
| RC-8 | Infrastructure | release servers provisioned, rollback rehearsed | runbook record |

### 2.7 Release 1.0 (M8)

| ID | Check | Pass condition | Evidence |
|---|---|---|---|
| R1-1 | All RC criteria still pass on the release build | re-run on the tagged artefact | reports |
| R1-2 | Platform delivery | client builds available per declared platform and store/channel | store/channel records |
| R1-3 | Servers | official servers live; private/dedicated server package published | deployment record |
| R1-4 | Operations | update, support and incident procedures documented and exercised | runbooks |
| R1-5 | Documentation | player and operator documentation published | docs |
| R1-6 | Owner sign-off | recorded acceptance of the release report | registry |

## 3. Cross-cutting checklist (must be covered by some milestone, never dropped)

| Area | First required evidence |
|---|---|
| PC | FP-1 |
| Android | CA-6 (DG-1…3), BE-3 |
| iOS | CA-6 (needs macOS/Xcode, NB-2), BE-3 |
| Cross-play | BE-2 |
| Single-player | FP-*, BE-5 |
| Co-op | IA-6 |
| Dedicated servers | IA-2 |
| Private servers | BE-4 |
| World persistence | FP-6, FP-7 |
| Geometry editing | FP-4, FP-5 |
| Save/load/recovery | FP-7, IA-3, CA-5 |
| Server authority | FP-8, IA-2 |
| Performance | B-1…B-11 |
| Security | BE-6, RC-6 |
| Crash recovery | IA-3, CA-2 |
| Compatibility and migration | CA-4, RC-3 |

## 4. Declaring a milestone

1. The agent assembles an evidence list mapping each criterion to a reference.
2. The Integration Owner reviews it; missing evidence keeps the milestone **Not reached**.
3. The owner records acceptance in the decision registry. Only then is the milestone shown as Reached (in `.agent/BOARD.md` and `docs/FUNCTIONAL-ROADMAP.md`).
