# Owner action plan (proposed)

**Status:** **Proposed.** A recommended order; every item is the owner's to accept, amend or reject. Nothing is decided, confirmed or authorised by this document. IDs refer to [`OWNER_DECISION_REGISTER.md`](OWNER_DECISION_REGISTER.md).

## 1. Decide first (they unblock everything else)

| # | Decision | Why first | Recommended | Effort for the owner |
|---|---|---|---|---|
| 1 | **ODM-10** how a decision is confirmed (E2 PR approval, or E3 web-UI confirmation commit; separate identity or token for agent pushes) | no entry can count without it; today all 39 commits are unsigned and share the owner's identity | E2 or E3; agents use a distinct identity | a choice and (optionally) one account |
| 2 | **ODM-01** documentation integration: docs-only PR set D1-D4, spike code and `docs/spike` out; confirm the branch name (`architecure_roadmap` does not exist) | production base for `GOV-04` | accept [`DOCUMENTATION_INTEGRATION_PLAN.md`](DOCUMENTATION_INTEGRATION_PLAN.md) | a choice and permission to create branches and PRs |
| 3 | **ODM-09** protect `master` (documentation check first), Actions permissions, project licence | `GOV-05`/`GOV-06`; the repository is public without a licence | protect early; decide the licence with D-E | settings and a choice |
| 4 | **GOV-01#14** D-E parameters a-e, i, j and **GOV-01#15** branch option | CI specification | decide now; f, g with `ADP-03` | values (Rust version, edition, CI host, licence list) |

## 2. Decide before the wave that needs it

| Decision | Needed before | Recommended |
|---|---|---|
| Core batch: GOV-01#1, #4-#9, #11, #13 (+#10, #12), with ACR-002 (before `API-02`) and ACR-003 (plan section 14) | `GOV-02` / `CORE-*` | as in [`docs/archive/gov01/GOV-01-TECHNICAL-RECOMMENDATIONS.md`](../gov01/GOV-01-TECHNICAL-RECOMMENDATIONS.md) |
| **ODM-11** `VoxelPos` width, world bound, vertical extent | `CORE-02` | decide with GOV-01#10 |
| **ODM-03** PC-1 and experimental snapshot-only format | `CORE-02/03` | Provisional with expiry |
| **ODM-07** register TC-01...TC-11 (TC-03, TC-10 first) | W1 / W2 | accept |
| **ODM-16** local authoritative runtime model | `SRV-01`, `TGAP-03a` | accept option (a) |
| **ODM-02** Processing Specification: supply it or accept TC-01 under AOD-24 | W2 | supply, else Provisional |
| **ODM-14** ADG amendments (OPT-01...OPT-08, R-MERGE-G1, release closure) | W1 | accept after reading the change table |
| **ODM-17** start versus merge conditions of G0 | `CORE-01` | optional overlap |
| **ODM-04** owners, by wave: WorldGen, Networking, Server (for M2-T0); Client, Engine Adapter (for the window); Gameplay, Content, Art/Audio (for M2) | W4 / W5 | name by wave |
| **ODM-05**, **ODM-12** Game Rules module; experiments (`GM-02`, `SPK-PERS`) | `GM-10`; `PERS-02` | yes after the spike |
| **ODM-06** M2-T/M2 scope and [`M2T_ACCEPTANCE_CRITERIA.md`](M2T_ACCEPTANCE_CRITERIA.md) | W6 (better before W1) | accept |

## 3. Can wait

ODM-08 (mobile checkpoints; before M3), ODM-13 (content data format; at `GM-20`), ODM-15 (M3 choices), thresholds for AC-13 and AC-14 (after two measured runs), bit-identical builds (AC-01), evidence storage location, owners of modules M2 does not use.

## 4. What can be integrated into `master` (after the owner's approval of each PR)

D1 (records and proposals), then D2, D3, D4 in that order, after the two script fixes and the status refresh ([`DOCUMENTATION_INTEGRATION_PLAN.md`](DOCUMENTATION_INTEGRATION_PLAN.md) section 6). Not the spike code. The branch stays (and is tagged by the owner) so that commit hashes cited in the documents keep resolving.

## 5. What must be done before M2-T starts (gates R0, R1, R2 of [`IMPLEMENTATION_READINESS.md`](IMPLEMENTATION_READINESS.md))

Confirmation mechanism and registry; the base and CI with required checks and archcheck v0; the Core batch, coordinate decisions and P0-alpha; temporary contracts TC-03 and TC-10; named roles for Core Agent, Integration Owner and Agent I.

## 6. Recommended sequence (short)

1. Decide ODM-10, ODM-01, ODM-09 and GOV-01#14/#15 (one sitting, answered in batches).
2. Prepare the documents for integration: fix the two scripts and refresh the status documents (one docs-only commit, with permission); then the PR set D1...D4.
3. Close PS-0: `GOV-02`, `GOV-03`, `GOV-04`, `GOV-05`, `GOV-06`, `TEST-01`, `XT-01`.
4. Start the Technology Track at wave W1 (Core) as soon as R2 holds; in parallel, owners are named by wave and the isolated experiments run (`SPK-PERS`, `GM-02`).
5. Reach **M2-T0** (headless authoritative world, 57 tasks, 16-task chain): the smallest production milestone and the first real feedback.
6. Add the window and the character (W5/W6) to reach M2-T; then the Game Rules for M2.

## 7. Explicitly outside M2-T

Combat, creatures, loot, progression, HUD beyond menus; the journal and compaction; the second terrain variant; History, tools, Composite, Resampling, SDF and mesh backends, greedy mesher, LOD; the wire protocol, external transport, sessions, reconnect, permissions, dedicated host, prediction; Android and iOS builds (checkpoint MAC-1 after M2); cross-play; 500-player architecture; the Processing Specification and Voxel Architecture themselves (only their temporary contracts are used).
