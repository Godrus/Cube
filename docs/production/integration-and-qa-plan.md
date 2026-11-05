# Cube Integration and QA Plan v2 (proposal)

**Status:** **Proposed**; nothing approved. Integration is **one module at a time**, only after its task acceptance criteria and the conditions in the ADG (`module-boundaries.md` parts C and D). Stage names are `PS-n`; this document restates no dependency.

## 1. PR and branch strategy

* **Baseline first (GOV-04).** `master` receives a docs-only baseline before any production PR; the existing branch `docs/architecture-roadmap-v1` contains the spike code and is not merged as is (fact: 31 commits ahead, 247 files versus 13 on `master`). The owner picks the option: (a) a new docs-only branch from `master` with the six docs commits (links into `docs/spike/` are then removed or the spike reports are added by a separate docs commit), (b) path-limited PRs, (c) accept the spike code as an explicitly experimental directory (contradicts roadmap-v1 section 7).
* **Skeleton PR (GOV-06)** adds workspace, toolchain pin and CI; nothing else.
* **One module, one PR chain.** Branches `prod/ps<stage>-<crate>[-topic]` from `master`. A PR changes one crate (plus its tests, benches, docs). Cross-crate edits: the API-owner PR first, consumer PR second.
* **Merge order** follows the ADG `I` edges; draft PRs may open earlier against the dependency branch.
* **Spike code never merges.** Useful code is re-implemented; spike tests are ported (section 4). The "not for master" list of roadmap-v1 section 7 stays in force.
* **Size:** over about 1 500 changed lines excluding tests and golden data, split.

## 2. Roles

* **Integration Owner:** owns `cube-editing-api`, `xtask`, the composition-root specification and the PR checklist; reviews every cross-crate API change; maintains the ownership table and the baseline register; gates a stage exit (all gate members merged and accepted, CI green, documents updated, open decisions listed). **Does not approve contracts or ACRs**; records approvals only on the owner's registry entry.
* **Project owner:** all contract, ACR, format, transport, owner-assignment and licence decisions, each as a decision-registry entry with a confirmation source.
* **Agent I:** testkit, integration-test crate, benches, CI implementation, device tooling; proposes thresholds; does not change them alone.

## 3. CI: specification versus running CI (separated)

| Deliverable | Task | Evidence |
|---|---|---|
| **CI specification** (what must run, required statuses, tool pins, pass conditions) | GOV-05 | an Owner-Approved document |
| **CI implementation** (a running pipeline) | GOV-06 | workflow-run URL: green on the empty skeleton, red on seeded failures (format error, clippy warning, failing test, forbidden dependency, forbidden `use`, forbidden licence) |

Required checks (specification): `cargo fmt --check`; `clippy -D warnings`; `cargo test` (workspace); `cargo deny` (licence allow-list set by the owner, advisories); `xtask archcheck` (crate graph equals ADG part B; `use`-level scan; mutation-boundary rules; no `std::process`, no `OS.execute` outside tests/tools; no floating point in authoritative or world-defining code per D-T); public-API snapshots for `cube-core` and `cube-editing-api`; contract-id check against `docs/api-contracts/README.md`; benchmarks within thresholds (desktop). **No check requires a device or an Android emulator.** Nightly jobs: loom, stress, soak.

## 4. Spike tests ported as acceptance cases

| Spike tests (path) | Acceptance case for |
|---|---|
| `cube-core/src/asset.rs` (9), `brick.rs` (3), `coord.rs` (3), `region.rs` (3), `revision.rs` (2) | CORE-02, CORE-03, CORE-05, CORE-06 |
| `cube-core/tests/dependency_rules.rs` (3) | XT-01, XT-02 (superseded and extended) |
| `cube-processing/src/lib.rs` (6) | PROC-01, PROC-05 |
| `cube-editing/src/lib.rs` (9) | ORCH-01…05, VOX-01, HIST-02 |
| `cube-persistence/src/lib.rs` (6) | PERS-01, PERS-02 (only the ideas; the format is replaced) |
| `cube-worldgen/tests/determinism.rs` (9), `tests/world.rs` (20) | WGEN-01, WGEN-02, WGEN-04 |
| `cube-godot-adapter/tests/chunk_mesh.rs` (13), `greedy.rs` (11), `sections.rs` (11), `tjunction.rs` (9) | VOX-02…VOX-04, RND-01 |
| `no_child_processes.rs` (3), `scripts/check-apk-no-exec.py` | ADP-02, GOV-06 |
| `cube-server` tests (3) | seed cases for TEST-03b |

A test that cannot be ported is replaced by one of equal or greater strength and the change is recorded. Spike results are never evidence of production readiness.

## 5. Contract change approval

1. A change to a Proposed/Provisional/Owner-Approved contract, port or public-API snapshot is a **change request** (problem, affected contracts and crates, compatibility class, migration, tests, decision requested).
2. Additive optional changes: Integration Owner reviews, owner informed. Anything else is breaking: ACR and owner approval.
3. API-snapshot tests fail CI on any public-API difference in `cube-core` and `cube-editing-api` without a linked accepted change request.
4. Status changes follow the Contract Status Model: only the owner's registry entry moves a contract to Owner-Approved; the Integration Owner records Implemented; Provisional never becomes Frozen without its revalidation gate.
5. An implementation never defines a contract implicitly; where code and contract differ, the PR says which is wrong.

## 6. Test strategy by kind (all CI-host unless stated)

| Kind | Tooling | Where | Required from |
|---|---|---|---|
| Unit and contract tests | `cargo test`, one set per contract | every crate | every PR |
| Negative tests per error code | | every crate | every PR |
| **Negative tests for forbidden mutation paths** | `trybuild` compile-fail, archcheck seeded violations, commit-log test | CORE-08, XT-02, TEST-06 | PS-1 |
| Property-based | `proptest` strategies from testkit | Core (atomicity, isolation, monotonic revisions, coordinate symmetry), Processing (stale never published), History, backends (greedy area equals faces), Persistence (snapshot plus tail equals replay) | PS-1 |
| **Concurrency** | `loom` (commit gate, slot publish), stress runners, ThreadSanitizer nightly where available | Core, Processing, Orchestration | PS-1, PS-2, PS-4 |
| **Fault injection** | `FaultFs` crash at every numbered I/O step; truncated tail; corrupted manifest and snapshot; version mismatch; double recovery | Persistence | PS-10 (TEST-04, QA-P2) |
| Backend conformance | generic suite in `cube-integration-tests` (TEST-03a), independent of any renderer | each backend | PS-5 |
| Transport conformance | one suite on in-process and external transports and on both hosts (TEST-03b) | Net, Server | PS-13 |
| Authority and security | client cannot commit or call system ports; no client-supplied paths; replay rejected; fuzzing | Server, Net | PS-13 (TEST-07) |
| Regression | golden hashes, golden meshes, image regression for faces/greedy/T-junction on the CI host | WorldGen, backends, client | each owning PR |
| **Determinism and golden hashes** | byte-order fixed in the contract; golden values per generator version; **CI-host equality is a PS-11 condition; cross-device equality is DG-2** | Core hash, WorldGen | PS-1 (Core hash), PS-11 |
| Performance | `criterion` baselines on the CI host; device runner scripts (QA-06) | Core commit, Processing, backends, WorldGen, client | thresholds per stage; device thresholds from DG data |
| Mobile validation | physical devices only (DG-1…DG-3) | client, adapter, embedded server | PS-14 |

## 7. Mutation-path verification (summary; design in the architecture plan section 6)

* `UserCommit`, `SystemCommit`, `RecoveryCommit` handles are the only mutating entries; compile-fail tests prove that `cube-worldgen`, `cube-client`, `cube-godot-adapter`, `cube-persistence` (for `UserCommit`) and `cube-orchestration` (for `SystemCommit`/`RecoveryCommit`) cannot construct or name the wrong handle.
* A property test asserts that every revision increment has a commit-log entry with an `ActorKind`.
* archcheck fails on history patterns and `commit` references in adapters and clients, and on direct commits in worldgen after LEG-01.
* A regression test replays the spike scenarios through the new path and compares content hashes.

## 8. Persistence verification (summary; requirements in the architecture plan section 8)

The AOD-21 protocol `write snapshot, fsync, verify, atomic manifest update, fsync manifest, truncate` is tested by crashing after each step on `FaultFs`; asserted: recoverable state, no unverified snapshot as sole truth, hash/epoch/revision/generator-version checks, idempotent compaction, the previous recovery point preserved until the new one is confirmed, directory-fsync behaviour as modelled. Device behaviour of fsync and rename is a DG-2 result, not a PS-10 condition. The spike persistence code is not evidence for any of these.

## 9. Benchmarks

Baselines from the spike are desktop and experimental. Production thresholds are set after PS-1/PS-2/PS-5 measurements on the CI host; device thresholds only from DG-1…DG-3. Initial reference points (not thresholds): chunk commit 0.009 ms, generation 0.008-0.009 ms per chunk, section rebuild 0.1-0.3 ms. A regression gate on the CI host is introduced with QA-02.

## 10. Mobile validation

Physical Android ARM64 matrix (low, mid, high) and an iPhone; scripted runs with touch input, soak, thermal, battery, memory, cold start, 4 KB and 16 KB page configurations, signed release builds, poor-network behaviour. Never replaced by an emulator (AOD-11). Gates DG-1, DG-2, DG-3 are separate from PS-1…PS-13.

## 11. Definition of done for a module PR

1. Contract references and their status are stated; no code claims an Owner-Approved contract that is only Proposed.
2. Task acceptance criteria met with test names listed.
3. CI green including archcheck and API snapshots.
4. Public API and error codes documented; open decisions listed.
5. No new dependency without a licence check; no `unsafe` outside the adapter.
6. Integration Owner review; owner registry entry for contract-affecting changes.
7. Mobile impact stated (memory, threads, main-thread cost).

## 12. Stage exit evidence (QA view)

Each stage exit is its gate `Gn` (ADG part D). The evidence list per stage is in `implementation-roadmap.md` section 3 (tests and acceptance), restricted to what the CI host can show unless the row is a device gate.
