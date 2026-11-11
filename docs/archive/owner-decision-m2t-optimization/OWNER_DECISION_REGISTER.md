# Owner decision register (consolidated, proposed)

**Status:** **Proposed.** A consolidated preparation register. **No decision in it is approved, confirmed or recorded as made**; recommendations are not approvals. Statuses used: `PENDING_OWNER_DECISION`, `PENDING_OWNER_CONFIRMATION` (recorded from chat or recommendation, not yet confirmed), `PROPOSED` (new, not yet put to the owner as a question), `DEFERRED`; `APPROVED` is used for nothing, because **no explicit evidence of approval exists** (no registry file, no signed commit, section 0.3).
**Base:** branch `docs/architecture-roadmap-v1`, HEAD `a8584cd`, verified 2026-11-08.

## 0. Method and corrections to earlier labels

### 0.1 The ODM labels are not primary sources
The labels `ODM-01...ODM-15` were **created by the previous review package** (`docs/archive/m2-delivery-review/OWNER_DECISIONS_REQUIRED.md`, commit `a8584cd`). They are convenience labels. Each decision below therefore cites the **primary documents** (plan, AOD, contracts, spike) and not the label. Two new labels are added by this review: ODM-16 (local authoritative runtime model) and ODM-17 (start versus merge conditions of gate G0). `GOV-01#n` means item n of the 15 GOV-01 decisions (`docs/archive/gov01/GOV-01-DECISION-REGISTER.md`).

### 0.2 Discrepancies found while re-reading primary sources

| # | Previous statement | Verified fact | Source | Handling |
|---|---|---|---|---|
| D-REG-1 | ACR-003 is "Deferred" (needed only at PS-8) | the plan's **PS-1 entry criteria** list ACR-003 among the registry entries required before PS-1; ACR-002, D-E and the branch option are not in that list | `production-architecture-plan.md` section 14, item 1 | formally ACR-003 is a PS-1 entry item although technically needed later (`CORE-14`); keep it in the Core batch (it is a one-line confirmation) or ask the owner to amend section 14 |
| D-REG-2 | ACR-002 needed "before PS-3" | `API-02` (ports in `cube-editing-api`) is an **M2-T** task | ADG; `M2_TASK_GRAPH.md` | ACR-002 is needed **before `API-02` starts** (M2-T wave W2) |
| D-REG-3 | the previous M2-T graph defers recovery to M2 (difference D-4) | acceptance T-08 (reload after restart) needs recovery from base generation and snapshot (`WP-08` order steps 1-2); in the ADG that is `PERS-06` | `world-persistence-contracts.md` WP-08; ADG | corrected here: `PERS-06` in its base+snapshot form belongs to M2-T (OPT-08) |
| D-REG-4 | 75 M2-T tasks form the acceptance closure | 11 of the 75 are not reachable from the acceptance task `XI-02` (for example `VOX-01`) | `_build/opt_graph.py` | edges added in the corrected baseline B1 |
| D-REG-5 | session/handshake types (`NET-02`) can be deferred without cost | the Draft contracts say the embedded local player is a session with an `AuthorizationContext` and passes the same handshake/authorisation (SR-06, SR-08) | `server-runtime-contracts.md` | session **semantics** are kept in the World Operations interface (`TGAP-03a`); only wire encoding is deferred |
| D-REG-6 | "38 commits unsigned" | 39 commits on the branch, all unsigned, all under one identity | `git log` | counts updated |

### 0.3 State of the confirmation infrastructure (verified)
`docs/decisions/decision-registry.md` does not exist; no tracked file is named `decision-registry`. All 39 branch commits are unsigned (`%G?` = `N`) and authored under one identity. Therefore **nothing can currently be `APPROVED`/`CONFIRMED` by the plan's own rule (3.3)**. See `OWNER_CONFIRMATION_PROTOCOL.md`.

## 1. Summary

| ID | Title | Status | Blocks implementation? | Latest safe decision point | Recommended option |
|---|---|---|---|---|---|
| ODM-10 | Confirmation mechanism | PENDING_OWNER_DECISION | yes (every entry depends on it) | before the first registry entry | E2 or E3 as in the protocol |
| ODM-01 | Documentation integration | PENDING_OWNER_DECISION | yes (production base) | before `GOV-04` | docs-only PR set D1-D4 from `master` |
| ODM-09 | Branch protection, Actions permissions, project licence | PENDING_OWNER_DECISION | yes (`GOV-06`) | before `GOV-05` is approved | protect `master`; decide licence with D-E |
| GOV-01#14 | D-E: toolchain, CI host, licences, panic, runtime | PENDING_OWNER_DECISION | yes (`GOV-05`/`GOV-06`) | before `GOV-05` | decide a-e, i, j now; f, g by `ADP-03` |
| GOV-01#15 | Branch option for the docs-only baseline | PENDING_OWNER_DECISION | yes (`GOV-04`) | before `GOV-04` | new docs-only branch |
| ODM-11 | World boundary, `VoxelPos` width, vertical extent | PENDING_OWNER_DECISION | yes (`CORE-02`) | before `CORE-02` (vertical extent before `WGEN-01`) | decide with GOV-01#10 |
| GOV-01 Core batch (#1, #4-#9, #11, #13; #10, #12 as PC-1 basis) | Core contract decisions | PENDING_OWNER_DECISION | yes (`GOV-02`, `CORE-*`) | before `GOV-02` | as recommended in `GOV-01-TECHNICAL-RECOMMENDATIONS.md` |
| GOV-01#2, #3 | ACR-002, ACR-003 | PENDING_OWNER_DECISION | #2 before `API-02`; #3 formal PS-1 entry item | #2 before W2; #3 before PS-1 start | confirm both with the Core batch |
| ODM-02 | Processing contracts versus AOD-24 | PENDING_OWNER_DECISION | yes (from `API-02` merge) | before W2 | supply the specification, else Provisional TC-01 with the AOD-24 reading |
| ODM-03 | PC-1 and experimental persistence format | PENDING_OWNER_DECISION | yes (`CORE-02/03`, `PERS-*`) | before `CORE-02` | Provisional with expiry |
| ODM-16 | Local authoritative runtime model for M2-T | PROPOSED | yes (`SRV-01`, `TGAP-03a`) | before W2 | typed World Operations interface with in-process binding |
| ODM-17 | G0: start versus merge conditions | PROPOSED | no, shortens overlap only | before `CORE-01` | keep strict; allow overlapped start |
| ODM-12 | Permission for spike experiments | PENDING_OWNER_DECISION | no | any time | GM-02 and SPK-PERS yes |
| ODM-04 | Owners | PENDING_OWNER_DECISION | by wave | W1 none; W4/W5 required | name by wave |
| ODM-05 | Game Rules module | PENDING_OWNER_DECISION | gameplay tasks | before `GM-10` | yes after the spike |
| ODM-06 | M2-T/M2 scope and criteria | PENDING_OWNER_DECISION | acceptance | before W6 (better W1) | accept `M2T_ACCEPTANCE_CRITERIA.md` |
| ODM-07 | Temporary contracts (TC-01...TC-11) | PENDING_OWNER_DECISION | yes | per contract (TC-03, TC-10 before W1/W2) | register as Provisional |
| ODM-08 | Mobile checkpoints and device | PENDING_OWNER_DECISION | no | before M3 | accept MAC-1..3 |
| ODM-14 | ADG change proposals | PENDING_OWNER_DECISION | scheduling | before W1 | accept OPT-01...OPT-08 after review |
| ODM-13 | Content data format | DEFERRED | no | at `GM-20` | defer |
| ODM-15 | M3 choices | DEFERRED | no | M3 planning | defer |
| X-1...X-4 | AOD/OAM confirmation, status model + authority, owners, CI spec approval | PENDING_OWNER_CONFIRMATION (X-1), PENDING_OWNER_DECISION | `GOV-02`..`GOV-05` | see GOV-01 register | as in GOV-01 register |

## 2. Decision cards

Fields: 1 id, 2 question, 3 why necessary, 4 options, 5 advantages and disadvantages, 6 technical consequences, 7 effect on M2-T, 8 effect on M2 and M3, 9 dependencies, 10 recommended option, 11 blocks implementation, 12 latest safe decision point, 13 primary source, 14 confirmation mechanism.

### ODM-10: confirmation mechanism for owner decisions

> **CLOSED 2026-11-09.** Resolved by the owner's instruction recorded as DEC-002 in `docs/decisions/decision-registry.md`: chat confirmation is a permitted source, commits under the owner's account are not decisions, signatures are optional, one registry, statuses `PROPOSED/RECORDED/CONFIRMED/DEFERRED/SUPERSEDED/REJECTED`, levels `CONCEPT/TECHNICAL`. The card text below is kept as history; the statement 'nothing can currently be CONFIRMED' in section 0.3 no longer holds for decisions the owner confirmed in chat.
1. ODM-10, `PENDING_OWNER_DECISION`. 2. What act makes a decision "confirmed" in the sense of plan 3.3, given that no commit is signed and agent sessions commit under the owner's identity? 3. Without an answer no registry entry can be `CONFIRMED`; GOV-01, GOV-02 and X-1 cannot close.
4. Options: **(a)** E2: the owner approves a PR containing the entries; **(b)** E3: the owner makes a web-UI confirmation commit; **(c)** E1: owner-signed commits with a key agents cannot use; **(d)** the owner writes entries by hand. 5. (a) strong, platform-authenticated, no key management; needs a separate account for agent pushes (GitHub forbids self-approval). (b) simple, no second account; one commit per batch. (c) strongest only with a hardware-protected key; useless if the key is reachable. (d) most direct, most manual.
6. Registry file, append-only, lint check (protocol sections 3-6). 7. Needed before the first entry; no code effect. 8. Carries to every later milestone. 9. none. 10. **(a) or (b)**; (c) optional. 11. **Yes**. 12. before the first registry entry (before GOV-01 closes). 13. `production-architecture-plan.md` 3.3; `owner-architecture-decisions-v1.md` section 4; verified commit facts (`git log`). 14. the protocol itself: confirmation by the chosen act, recorded in the entry.

### ODM-01: documentation integration into `master`
1. ODM-01. 2. How does documentation reach `master`, and is the branch name `architecure_roadmap` (which does not exist) to be replaced by `docs/architecture-roadmap-v1`? 3. The branch contains spike code and needs the owner's approval to enter `master`.
4. Options: **(a)** docs-only PR set D1-D4 created from `master` tree-based, spike code and (by default) `docs/spike` excluded; **(b)** merge the whole branch (fast-forward); **(c)** keep documentation on the branch. 5. (a) clean base, reviewable units, history of the mixed branch not reused; needs link policy and work. (b) trivial, puts spike code on `master` (violates roadmap-v1 section 7 and GOV-04 acceptance). (c) no integration, no protection possible on the working base.
6. See `DOCUMENTATION_INTEGRATION_PLAN.md`. 7. `GOV-04` and `GOV-06` need a base. 8. none specific. 9. ODM-10 (confirmation of D1 content), script fixes. 10. **(a)**. 11. **Yes**. 12. before `GOV-04`. 13. `docs/architecture/roadmap-v1.md` section 7; `integration-and-qa-plan.md` section 1; GOV-04 row of `agent-task-breakdown.md`. 14. owner approval of each PR (E2) or web-UI act (E3).

### ODM-02: the Geometry Processing Specification and AOD-24
1. ODM-02. 2. The specification is absent. May M2-T rely on a Provisional registration of PR-01...PR-05 (TC-01), and does that satisfy AOD-24's condition that the specification "may not be replaced by a general assumption before PR-01...PR-08 are approved"? 3. On the published ADG every production merge from `API-02` onward waits for gate G2, which needs `GOV-07`, which needs the document (`EXT-GPS`).
4. Options: **(a)** the owner supplies the specification; **(b)** the owner states that Provisional status with a stated scope and expiry satisfies AOD-24; **(c)** wait. 5. (a) removes the risk, depends on a document that may not exist; (b) unblocks M2-T, accepts rework risk at RG-2; (c) no date, blocks Processing-dependent merges.
6. Provisional register, tests tagged `provisional`, rework budget at arrival. 7. decides whether W2 can proceed. 8. M3: TC-01 expires at M3 content start or at `GOV-07`. 9. ODM-07. 10. **(a) if available, otherwise (b)**. 11. **Yes** (from `API-02`). 12. before W2 (`PROC-01` start). 13. `docs/decisions/owner-architecture-decisions-v1.md` 5.5 (AOD-24 conditions); plan 4.4 (RG-2); `processing-contracts.md`. 14. registry entry with E2/E3 evidence.

### ODM-03: Voxel Architecture, PC-1 and the experimental storage format
1. ODM-03. 2. May Provisional PC-1 (C-07, C-08, C-09 = TC-02) and an experimental snapshot-only persistence format v0 (TC-08, explicitly deviating from AOD-21's "journal is the primary record" until M2) carry M2-T? 3. The Voxel Architecture is absent; formats cannot be approved without it (GOV-10 needs RG-1).
4. Options: (a) yes with expiry (RG-1 or first format approval); (b) wait; (c) other baseline. 5. (a) fastest, risk of migration (M2-T saves are disposable); (b) none; (c) no source.
6. Format marked experimental, version marker, no compatibility promise. 7. required for `CORE-02/03`, `PERS-*`. 8. M2 adds journal and compaction (`PERS-04/05`, `TGAP-09`); M3 saves should not be disposable. 9. ODM-11, GOV-01#10/#12. 10. **(a)**. 11. **Yes**. 12. before `CORE-02`. 13. AOD-24 candidate baseline BL-1 (5.5); plan 4.3/4.4; `world-persistence-contracts.md` WP-08; `worldgen-core-boundary.md` (journal primary). 14. registry entry.

### ODM-09: `master` protection, Actions permissions, project licence
1. ODM-09. 2. Enable protection and required checks on `master`? Grant Actions what permissions? Which licence does Cube itself carry (the repository is public without a licence file; the spike manifest says `UNLICENSED`)? 3. The CI specification requires the checks to be *required status checks* (plan 3.x; `integration-and-qa-plan.md` section 3); `cargo deny` needs the licence policy.
4. Options: (a) protect with the documentation check first and the full set after the skeleton; (b) protect only after the skeleton; (c) no protection. Licence: proprietary (all rights reserved) / an open-source licence / undecided with a note. 5. (a) early safety, small workflow; (b) later, less early benefit; (c) no gate. Licence options differ legally; D-E parameters d, e depend on it.
6. Workflow files, settings. 7. `GOV-06` and every PR. 8. permanent. 9. GOV-01#14, ODM-01. 10. **(a)**; licence together with D-E. 11. **Yes** (`GOV-06`). 12. before `GOV-05` approval. 13. GitHub API read 2026-11-07/08 (`protected: false`, 0 workflows, `license: null`); `integration-and-qa-plan.md` section 3; D-E in `architecture-blocker-decisions.md`. 14. registry entry; settings export as evidence.

### ODM-11: world boundary, `VoxelPos` width, vertical extent
1. ODM-11 (with GOV-01#10). 2. Which widths do `VoxelPos` and the key have, which finite world bound is accepted (or which wider/hierarchical key), and what is the vertical extent of a chunk column for the M2 terrain? 3. The spike's key range does not fit its own voxel range (`i32` brick coordinate times 16 in `i32`); the spike column is 32 voxels high (heights 0..31); a fixed-width key bounds the world.
4. Options K-A (`i32` bricks, range restricted), K-B (`i64` voxel positions), K-C (both `i64`), K-D (hierarchical key); vertical extent: spike value or larger. 5. see `GOV-01-DECISION-REGISTER.md` item 10. 6. persisted key encoding; generator version tied to the vertical extent. 7. needed for `CORE-02`, `WGEN-01`, the controller. 8. M3 content (caves, structures) needs more height; changing widths after persistence exists is a migration. 9. ODM-03. 10. decide with GOV-01#10; **K-B or K-D if the endless-world requirement is to hold, K-A with a stated bound otherwise**. 11. **Yes**. 12. before `CORE-02`; vertical extent before `WGEN-01`. 13. `architecture-blocker-decisions.md` D-K; `crates/cube-core/src/coord.rs` (read); `docs/spike/rust-world-generation/README.md`; roadmap-v1 gate 8 open decisions. 14. registry entry.

### ODM-12: permission for isolated experiments
1. ODM-12. 2. May the following experiments start: entity-model spike (`GM-02`), persistence durability spike (`SPK-PERS`), controller/camera/combat-feel prototype (optional "Game Prototype 0")? 3. The task breakdown states that no task is authorised to start by the plan; the entity-model question is "Needs spike" (REF-13); `PERS-02` waits for `SPK-PERS`.
4. Options: per experiment yes/no. 5. gains early facts without production risk; costs attention. 6. branches `spike/*`, never merged. 7. `SPK-PERS` is a prerequisite of `PERS-02`; `GM-02` of `GM-10`. 8. none. 9. none. 10. **yes for `GM-02` and `SPK-PERS`; Game Prototype 0 only under the integration rules**. 11. no (but their absence delays `PERS-02`/`GM-10`). 12. `SPK-PERS` before `PERS-02`; `GM-02` before `GM-10`. 13. `agent-task-breakdown.md` status line and SPK-PERS row; roadmap-v1 gate 6; `reference-comparison.md` REF-13. 14. registry entry or owner message recorded as PENDING_OWNER_CONFIRMATION until confirmed.

### ODM-16: local authoritative runtime model for M2-T (new, PROPOSED)
1. ODM-16, `PROPOSED`. 2. Is the following interpretation of AOD-22 and SR-01...SR-08 accepted for M2-T: one `ServerRuntime` (`SRV-01`) with a single-writer tick (`SRV-03`), accessed by the embedded client only through a typed **World Operations interface** (`TGAP-03a`: commands in, views and events out, a local session with an `AuthorizationContext`), bound by a thin in-process binding (`NET-03` over the `NET-01` trait), **without** wire encoding (`NET-02`), external transport (`NET-04`), session authentication providers (`SRV-02`), reconnect (`SRV-05`), permissions (`SRV-06`), dedicated host (`SRV-08`) or prediction (`CLI-05`)? 3. These tasks sit on the published critical path and have no owners.
4. Options: **(a)** as stated; **(b)** keep NET-02 and a serialised protocol in M2-T; **(c)** direct function calls with no transport boundary at all. 5. (a) keeps one runtime and one authority path, defers the carrier and encoding, satisfies AOD-22 (1)-(9); (b) complete but pulls the wire protocol and its owner into M2-T; (c) smallest, but creates a second access path and does not exercise the transport boundary (AOD-22 (6) says in-process transport). 6. messages are plain data with no references so serialisation can be added without changing the interface; a serialising loopback adapter becomes an M2 conformance test. 7. removes `NET-02` and `CLI-05` from M2-T and `NET-02` from the chain of `SRV-04`; the strict chain is unchanged by removing `NET-01/NET-03` entirely (computed), so (c) gains nothing in the metric. 8. M2: serialising loopback test; M4: wire protocol, sessions, dedicated host with identical semantics (AOD-22 (8), (10)). 9. ODM-07 (TC-11), GOV-22 (SR contracts). 10. **(a)**. 11. **Yes** (shapes `SRV-01`). 12. before `SRV-01` starts. 13. `owner-architecture-decisions-v1.md` 5.3 (AOD-22); `server-runtime-contracts.md` SR-01...SR-10; `module-boundaries.md` Part C (NET/SRV rows). 14. registry entry.

### ODM-17: gate G0, start versus merge conditions (new, PROPOSED)
1. ODM-17. 2. May the first Core tasks *start* once `GOV-01...GOV-06` are done, while `TEST-01` and `XT-01` are completed in parallel and gate the first *merge* (using the ADG's existing S/I distinction), instead of all eight G0 tasks gating the start? 3. `CORE-01/02/07` have start-after `G0`.
4. Options: keep strict; split as stated. 5. split gives overlap; **no change in the strict chain** because archcheck is a required check and the first Core merge needs it; strict keeps one simple rule. 6. none on code. 7. overlap only. 8. none. 9. ODM-14. 10. **keep strict unless overlap is wanted; it is a plan amendment either way**. 11. no. 12. before `CORE-01` starts. 13. `module-boundaries.md` Part C rows `CORE-01`, `CORE-02`, `CORE-07`; Part D (G0); `production-architecture-plan.md` section 14 (PS-1 entry criteria list a subset, see D-REG-1). 14. registry entry. *No exception clause exists in the plan (searched); this is an owner-approved amendment of the ADG, not an exception.*

### GOV-01#14: D-E (toolchain, execution model, licences)
1. GOV-01#14. 2. the ten parameters a-j (Rust version, edition, CI host, project licence, licence allow-list, panic strategy, FFI boundary, runtime policy, dev-dependencies, Core dependencies). 3. `GOV-05` cannot specify CI without them. 4-5. options per parameter in `GOV-01-DECISION-REGISTER.md` item 14. 6. `rust-toolchain.toml`, `deny.toml`, workflows. 7. required before `GOV-06`; f, g by `ADP-03`; **j (dependencies in Core) decides whether a hash crate may be used for RegionStamp**. 8. licence policy persists. 9. ODM-09. 10. decide a-e, i, j now; f, g with `ADP-03` if no earlier artifact sets `panic`. 11. **Yes**. 12. before `GOV-05` approval. 13. `architecture-blocker-decisions.md` D-E; root `Cargo.toml` (spike: `panic = "abort"`, `UNLICENSED`); AOD-23. 14. registry entry.

### GOV-01#15: branch option for the docs-only baseline
1. GOV-01#15. 2. new docs-only branch / path-limited branches / accept spike code. 3. `GOV-04`. 4-5. `GOV-01-DECISION-REGISTER.md` item 15. 6. see ODM-01. 7. production base. 8. permanent. 9. ODM-01. 10. new docs-only branch. 11. **Yes**. 12. before `GOV-04`. 13. `integration-and-qa-plan.md` section 1. 14. registry entry.

### Core batch (GOV-01#1, #4-#9, #11, #13, with #10 and #12 as PC-1 basis)
Each: question, options, advantages/disadvantages, consequences, recommendation and sources are in `docs/archive/gov01/GOV-01-DECISION-REGISTER.md` items 1, 4-13 and `GOV-01-TECHNICAL-RECOMMENDATIONS.md`; they are not repeated. Status `PENDING_OWNER_DECISION` for all. **Effect on M2-T:** they define P0-alpha (`GOV-02`) and the contracts `CORE-01...CORE-11` implement. **Effect on M2/M3:** persisted widths, OperationId and revision semantics constrain every later format. **Blocks implementation:** yes (`GOV-02`, then `CORE-*`). **Latest safe point:** before `GOV-02`; item 10/12 before `CORE-02/03`. **Confirmation:** per ODM-10.

## 3. Remaining decisions (compact)

| ID | Question | Status | Effect on M2-T | M2 / M3 | Recommended | Blocks? | Latest safe point | Primary source |
|---|---|---|---|---|---|---|---|---|
| ODM-04 | owners (WorldGen, Networking, Server, Client, Engine Adapter, Gameplay, Content, Art/Audio; UI placement) | PENDING_OWNER_DECISION | 18 existing tasks and 29 new need an owner; W1-W3 need none | M3 needs content and art owners | name by wave | by wave | W4/W5 start | `module-boundaries.md` A.2; GOV-03 row |
| ODM-05 | Game Rules module in the architecture | PENDING_OWNER_DECISION | `GM-10...GM-13` | all gameplay | yes after `GM-02` | gameplay tasks | before `GM-10` | roadmap-v1 gate 6 (OI-8, REF-13); `M2_OWNERSHIP_MATRIX.md` |
| ODM-06 | M2-T/M2 split and criteria | PENDING_OWNER_DECISION | defines the target | defines M2 | accept | acceptance | before W6 | `FIRST_PLAYABLE_SCOPE.md`; `DEVELOPMENT_ROADMAP.md` M2 |
| ODM-07 | register TC-01...TC-11 as Provisional (including policy H-3) | PENDING_OWNER_DECISION | TC-03, TC-10 before W1/W2 | expiries at M2/M3 | accept | yes | per TC | plan 3.1/3.2; `PROVISIONAL_CONTRACT_REVIEW.md` |
| ODM-08 | desktop acceptance, MAC-1...3, device | PENDING_OWNER_DECISION | none (desktop) | MAC-1 before M3 | accept | no | before M3 | AOD-08, AOD-11; `RELEASE_CRITERIA.md` |
| ODM-14 | ADG changes: OPT-01...OPT-08, R-MERGE-G1, release closure | PENDING_OWNER_DECISION | scheduling | | accept after review | scheduling | before W1 | `docs/archive/gov01/ADG-D01-D02-PATCH-PROPOSAL.md`; `M2T_OPTIMIZED_TASK_GRAPH.md` |
| ODM-13 | content data format | DEFERRED | none | at `GM-20` | defer | no | `GM-20` | no primary source (label of the previous review) |
| ODM-15 | M3 Minimum/Recommended, mobile in M3, co-op spike | DEFERRED | none | M3 | defer | no | M3 planning | `VERTICAL_SLICE_SCOPE.md` (Proposed) |
| X-1 | confirmation of AOD-01...24, OAM records | PENDING_OWNER_CONFIRMATION | AOD-18/20/24 before `GOV-02` | | bulk act per protocol | `GOV-02` | before `GOV-02` | plan 3.3 |
| X-2 | status model and authority split | PENDING_OWNER_DECISION | | | as in GOV-01 register 0.2 | `GOV-02` | before `GOV-02` | AOD-18; plan 3.1 |
| X-3 | owners or written deferral | PENDING_OWNER_DECISION | | | | `GOV-03` | | GOV-03 row |
| X-4 | CI specification approval | PENDING_OWNER_DECISION | | | | `GOV-06` | | plan 3.x |
| ST-1 | canonical state / `AGENTS.md` | PENDING_OWNER_DECISION | | | option A | no | | `docs/archive/gov01/ST-1-CANONICAL-STATE-RECOMMENDATION.md` |
| ST-2 | accept D-01/D-02 | PENDING_OWNER_DECISION | | | | | | `docs/archive/gov01/ADG-D01-D02-ANALYSIS.md` |
