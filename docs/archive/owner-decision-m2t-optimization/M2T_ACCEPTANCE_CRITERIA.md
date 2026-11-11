# M2-T acceptance criteria (proposed)

**Status:** **Proposed** / `PENDING_OWNER_DECISION` (ODM-06). Reviews and refines the criteria T-01...T-13 of `docs/archive/m2-delivery-review/M2_SCOPE_REVIEW.md` against the fifteen target results of the brief. **No numerical performance threshold is invented.** Where a threshold would be needed, the criterion is a *measure-and-report* condition plus a stated method for the owner to set the threshold after measurement.
**M2-T is not First Playable:** no combat, creatures, loot, progression or HUD beyond menus.

## 1. Mapping to the previous criteria

| New id | Target result of the brief | Previous id |
|---|---|---|
| AC-01 | reproducible client build | T-12 |
| AC-02 | client startup | T-01 |
| AC-03 | world creation using a seed | T-02 |
| AC-04 | world loading | T-08 (part) |
| AC-05 | terrain generation | T-03 |
| AC-06 | third-person character movement | T-04 (part) |
| AC-07 | collision with terrain | T-04 (part) |
| AC-08 | voxel geometry rendering | T-05 |
| AC-09 | voxel destruction | T-06 (part) |
| AC-10 | voxel placement | T-06 (part) |
| AC-11 | world persistence | T-07 |
| AC-12 | reload after restart | T-08 |
| AC-13 | basic stability | T-10 |
| AC-14 | no unacceptable main-thread blocking | T-09 |
| AC-15 | clearly defined local authority model | T-11 (extended) |
| AC-16 | written mobile-impact statement (documentation only, not in the brief's list) | T-13 |

## 2. Test environment (common)

* **Declared desktop platform:** one desktop OS and GPU class named in the evidence (the spike was exercised on Windows; the CI host and the declared platform are open: D-E parameter c, ODM-09). **Pending decision.**
* Clean checkout of the commit under test; toolchain pin per D-E; no network.
* Scripted runs use a harness **timeout** only as a liveness bound; a timeout value is a harness parameter and **not** a performance criterion.
* Evidence retained for every criterion: commit hash, command line, log, the artefact named in the last column. Storage location of evidence (for example `docs/evidence/<id>/` or CI artefacts) is **pending decision** (not created here).

## 3. Criteria

| ID | Measurable acceptance condition | Test method | Required implementation (tasks) | Dependencies | Evidence to retain |
|---|---|---|---|---|---|
| **AC-01** | on a second, clean checkout of the same commit, the documented command(s) produce the runtime library, the extension and the client project that start; the toolchain version is the pinned one | scripted build from a clean clone | `TGAP-04`, `ADP-01`, `GOV-06` | pinned toolchain (D-E a, b) | build log, commit hash, toolchain file. *Bit-identical output is **not** claimed; whether it is required is an open question* |
| **AC-02** | the client process reaches the main menu and exits with code 0 on quit; the engine log contains no `ERROR`/`SCRIPT ERROR` lines | scripted smoke run (as the spike's UI test did) | `GM-15`, `ADP-02`, `ADP-03`, `CLI-00` | AC-01 | log, exit code |
| **AC-03** | creating worlds with the same seed twice yields equal stable chunk hashes for a fixed list of chunk coordinates; a different seed yields different hashes; the world record stores seed and generator version | determinism suite | `WGEN-01`, `CORE-16`, `PERS-08`, `TEST-09`, `TEST-02` | decisions D-K / D-T (integer-only generation) | hash table, suite output |
| **AC-04** | opening an existing world restores the world record and the saved modifications; base chunks are regenerated from seed and version, modified bricks come from the snapshot | integration scenario | `PERS-06` (base + snapshot), `PERS-03`, `WGEN-03`, `SRV-01` | AC-11 | scenario log, content hash |
| **AC-05** | with the player at a position, all chunks inside the configured interest radius reach state Present; walking across chunk borders in each horizontal direction loads new chunks and releases far ones; no chunk is left in state Failed | integration scenario + log assertion | `WGEN-02`, `WGEN-04`, `SRV-04`, `CLI-01`, `PROC-03/04` | AC-14 | chunk-state log, route definition. *The interest radius is a configuration parameter, not a requirement* |
| **AC-06** | a recorded input sequence (move, turn, jump) produces the same position trajectory on two runs on the same platform; the orbit camera stays outside solid voxels along the route | replay test + assertion | `GM-11`, `GM-12`, `GM-14`, `CLI-04`, `SRV-03` | AC-07 | input log, trajectory file. *Cross-platform equality of floating-point camera/rendering values is **not** claimed (rendering-only data, class 7 of D-T)* |
| **AC-07** | the character never ends inside a solid voxel along the test route; it steps up one voxel, falls and stops on ground, and is not tunnelled through terrain at the controller's configured maximum speed; chunk borders and negative coordinates are included | property/scenario tests on the collision provider | `VOX-05`, `GM-12`, `TGAP-01` | TC-06 | test output |
| **AC-08** | the set of rendered faces equals the brute-force set of exposed faces for generated terrain, at chunk borders and negative coordinates; no holes or seams in screenshots along the defined route; after an edit at a brick border the neighbouring section is rebuilt | mesh correctness tests + manual screenshots | `VOX-02`, `VOX-04`, `CLI-02`, `PROC-02/05` | TC-05 | test output, screenshots. *Pixel comparison against golden images is platform-specific and is optional, not required* |
| **AC-09** | a destroy request within reach changes the authoritative voxel and the client view; out-of-reach and invalid requests change nothing; the commit log shows one `UserCommit` with `ActorKind::User` and an `OperationId` per request | scenario + commit-log assertion | `GM-13`, `TGAP-01`, `ORCH-01..05`, `VOX-01`, `API-02/03` | TC-09, ACR-002 | commit log excerpt |
| **AC-10** | the same for placement, including rejection when the target cell is occupied or the player's body is inside it | as AC-09 | as AC-09 | as AC-09 | as AC-09 |
| **AC-11** | save publishes through the atomic manifest; with a kill injected after each step (temp file written, file synced, rename, directory synced) the next start finds either the previous complete state or the new complete state; checksums are verified on load; a corrupted snapshot is rejected with a stable error code | fault-injection test on the persistence layer | `PERS-01`, `PERS-02`, `PERS-03`, `SPK-PERS` | TC-08 | test output with the crash points listed. *The journal and the full crash matrix are M2* |
| **AC-12** | after process exit and relaunch the world content hash equals the pre-save hash; the player respawns at the default spawn (player position is not persisted in M2-T) | scripted restart scenario | `XI-01`, `PERS-06`, `SRV-01` | AC-04, AC-11 | before/after hashes |
| **AC-13** | the stability harness (scripted traverse, edit, save, reload loop) completes the configured number of iterations without a crash, panic or `ERROR` line, and writes a trend report (memory, time per iteration, error count) | harness run on the declared platform | `TGAP-06`, `XI-01` | AC-01...AC-12 | harness output, trend report. **Pending decision:** number of iterations or duration and any growth limit; method: run the harness twice, report, the owner sets the limit from the reports |
| **AC-14** | (a) no generation, meshing, save or load call executes on the engine main thread (thread-name assertions in tests; allowed main-thread call list reviewed in the adapter audit); (b) a frame-time and adapter-call-duration trace of a scripted session (streaming, edits, save, load) is recorded and reported | assertions + instrumented session on named hardware | `PROC-03`, `CLI-02`, `ADP-01..03`, `TGAP-06` | TC-10 | audit list, trace files. **Pending decision:** what is "unacceptable"; method: three runs on named hardware, the owner sets a limit after reading the distribution (benchmarks B-5/B-6 of `RELEASE_CRITERIA.md`) |
| **AC-15** | the local authority model of `M2T_CRITICAL_PATH_REVIEW.md` section 2 holds: all world mutations are user commits by the single writer; the client holds only views and cannot mutate the authoritative asset (compile-fail/archcheck); engine scripts contain no world or gameplay logic (script audit); the World Operations interface carries `OperationId`/`ActorKind` | compile-fail tests, archcheck, script audit, commit-log assertion | `XT-02`, `TEST-06`, `TEST-10`, `CORE-08`, `ORCH-05`, `TGAP-03a`, `ADP-02` | ODM-16 accepted | tool output, audit list |
| AC-16 | a written statement of what would block an Android build (input, storage location, threading, build) | document review | none (documentation) | AOD-08 | document |

## 4. Not required for M2-T

Prediction and reconciliation; the external transport and any serialised protocol; sessions beyond the one local session; the journal and compaction (M2); two terrain variants (M2); player position and inventory persistence (M2); creatures, combat, loot; HUD beyond menus; History; tools; device runs (checkpoint MAC-1 is after M2).

## 5. Open items for the owner

* the declared desktop platform (and whether the CI host is the same);
* thresholds for AC-13 and AC-14 (by the measure-and-report method);
* whether bit-identical builds are required for AC-01;
* where evidence is stored.
