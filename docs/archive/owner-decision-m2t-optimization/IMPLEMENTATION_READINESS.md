# Implementation readiness (proposed)

**Status:** **Proposed.** States what must exist before production implementation may begin. It authorises nothing. PS-0 is **not** complete (0 of 8 gate tasks; verified: no registry, no CI, no baseline branch, no `xtask`, no testkit); PS-1 is **not** authorised.

## 1. The governing rule, exactly

| Rule | Source (verified) | Content |
|---|---|---|
| Start condition of the first Core tasks | `docs/production/module-boundaries.md` Part C, rows `CORE-01`, `CORE-02`, `CORE-07`: start-after `G0` | `G0` is satisfied when all eight member tasks (`GOV-01...GOV-06`, `TEST-01`, `XT-01`) are merged and accepted (Part D) |
| PS-1 start | Part D, stage table row PS-1: start-after `G0`, `TEST-01` | |
| Closing condition of G0 | Part D | CI specification Owner-Approved **and**, separately, CI running with workflow-run URL and required-status-check settings; registry entries owner-confirmed |
| PS-1 entry criteria | `production-architecture-plan.md` section 14 | (1) registry entries (owner-confirmed): ACR-001, ACR-003, ACR-004 A, OAM-10 principle, D-R1, D-R2, D-R6, D-R8, D-K, D-T, D-C, D-0; (2) P0-alpha Owner-Approved and PC-1 registered Provisional; (3) named owners for Core Agent, Integration Owner, Agent I (unassigned modules **not** required); (4) docs-only baseline on `master`; (5) CI specification approved and CI running with required checks, archcheck v0, testkit; (6) ADG adopted |
| Agents may not start a stage early | `docs/archive/AGENT_HANDOFF.md` hard rules | "Never start the next Production Stage before its entry criteria hold." |
| Confirmation | plan 3.3 | a chat instruction alone is not a confirmation source |

**Does PS-0 formally block implementation?** Yes. Searching the plan, the ADG, the checklist, the handoff and the integration plan for an exception, waiver or exemption clause finds **none**. A "limited exception" therefore does not exist and is not invented here.
**What the owner can legitimately do:** the plan documents are Proposed; the owner decides plan changes. A change of *start* conditions (for example ODM-17: starting `CORE-01/02/07` after `GOV-01...GOV-06` while `TEST-01` and `XT-01` finish in parallel and gate the first merge, using the ADG's own start/merge distinction) is an **owner-approved amendment recorded in the registry and applied to the ADG** (ODM-14), not an exception. Isolated experiments (L2) are not production and need only the owner's go-ahead (ODM-12). Note the plan's own PS-1 entry list (section 14) is narrower than the G0 membership: it does not list the whole of GOV-01, `GOV-03` beyond three roles, or `XT-01`/`TEST-01` as separate items except through item 5.

## 2. A. Requirements to begin M2-T implementation (the first production task, `CORE-01`)

| # | Requirement | Minimal form | Can be narrowed? |
|---|---|---|---|
| A-1 | confirmation mechanism and registry (ODM-10) | registry file; evidence type chosen | no |
| A-2 | docs-only base (`GOV-04`, ODM-01) | D1 merged (or the owner's chosen base) | the PR set is flexible |
| A-3 | CI specification approved (`GOV-05`; D-E a-e, i, j decided; ODM-09) | one-page specification | f, g may be deferred to `ADP-03` |
| A-4 | CI running with required checks; archcheck v0; testkit skeleton (`GOV-06`, `XT-01`, `TEST-01`) | workflow-run URL green on skeleton and red on seeded failures; settings export | `TEST-01` and `XT-01` may overlap the *start* of Core (ODM-17) but gate the first merge |
| A-5 | Core decisions recorded: ACR-001, ACR-003 (plan section 14), ACR-004 A, OAM-10, OD-R1, OD-R2, OD-R6, D-R8, D-T, D-0 | per `OWNER_DECISION_REGISTER.md` | ACR-003 is a formal PS-1 item although technically needed later |
| A-6 | `P0-alpha` Owner-Approved and PC-1 registered Provisional (`GOV-02`; ODM-03, ODM-07) | registry entries with the five mandatory fields | no |
| A-7 | coordinate decisions D-K, D-C with `VoxelPos` width, level semantics, world bound and vertical extent (ODM-11; TC-03, TC-04) | recorded before `CORE-02/03` | no |
| A-8 | threading contract TC-10 accepted | rule set | needed before `PROC-03`, not before `CORE-01` |
| A-9 | owners for Core Agent, Integration Owner, Agent I named (roles exist; confirmation in `GOV-03`) | named | yes: no other owner is needed for the first wave |
| A-10 | ADG adopted with the amendments of ODM-14 | patch applied | the amendments can be adopted incrementally |

## 3. B. What can proceed in parallel without waiting for all blockers

| Work | Level | Needs |
|---|---|---|
| documentation integration (D1...D4) and script fixes | documents | A-1, ODM-01 |
| decision preparation and the owner's answers | documents | |
| `SPK-PERS` (persistence durability experiment), `GM-02` (entity-model spike) | isolated experiment | ODM-12 |
| `GM-01`, `GM-03`, `GM-04`, `TGAP-05` (design, tables, scenarios, threading contract text), `GM-16` (placeholder) | documents / assets | none |
| naming owners (ODM-04) and benchmark plan | decisions | |
| drafting of TC registry entries, the serialising-loopback design, registry lint | documents | |
| optional "Game Prototype 0" on spike code | isolated experiment, never promoted | ODM-12 and the rules of `TECHNOLOGY-GAME-TRACK-INTEGRATION.md` section 3 |

## 4. C. What can safely wait until after M2-T (belongs to M2)

Journal, compaction and journal-tail recovery (`PERS-04`, `PERS-05`, `TGAP-09`, `TEST-04`, `QA-P2`, `ITG-01`, `ITG-02`); the second terrain variant and spawn step (`TGAP-07/08`); Game Rules content (`GM-20...GM-33`), `TGAP-02`, `TGAP-03b`; the serialising loopback conformance test; crash matrix beyond the atomic-publish points; the outside-tester session; data format for content (ODM-13); HUD beyond menus.

## 5. D. What must not be pulled into M2-T (multiplayer)

`NET-02` wire protocol; `NET-04` external transport and `SPK-NET`, `GOV-12`; `SRV-02` session authentication; `SRV-05` reconnect; `SRV-06` permissions; `SRV-07` history integration; `SRV-08` dedicated host; `CLI-05` prediction and reconciliation; `TEST-03b`, `TEST-07`; 500-player architecture; cross-play; accounts. The compatibility boundaries that keep them additive are listed in `M2T_CRITICAL_PATH_REVIEW.md` section 2. **Condition to pull any of them:** the owner explicitly chooses option (b) of ODM-16, which adds `NET-02` and its owner to M2-T.

## 6. Gate table

| Gate | Condition | Evidence | Unblocks |
|---|---|---|---|
| **R0** Decisions | ODM-10, ODM-01, ODM-09; GOV-01 items 14, 15 | registry entries | `GOV-04`, `GOV-05` |
| **R1** Base and CI | `GOV-04`, `GOV-05`, `GOV-06`, `XT-01`, `TEST-01` | PR merged; workflow run; protection export | any production merge |
| **R2** Core decisions | A-5 ... A-7, `GOV-02`, ODM-03, ODM-07 (TC-03) | registry entries | start of `CORE-01/02/07` |
| **R3** Core wave | G1 members merged; API snapshot | CI, tests | Processing, Editing API |
| **R4** Processing and edit path | ODM-02 (TC-01), ACR-002, TC-10 | entries | `API-02` merge and later |
| **R5** Owners for world, runtime | ODM-04 for WorldGen, Networking, Server | ownership table | `WGEN-*`, `NET-*`, `SRV-*` |
| **R6** Client | ODM-04 for Client Runtime and Engine Adapter | | `CLI-*`, `ADP-*` |
| **R7** Game Rules | ODM-05, ODM-12 (`GM-02`) | | `GM-10...` |
| **R8** M2-T acceptance | AC-01...AC-16 | evidence index | M2 work |

Implementation of the first production task (`CORE-01`) needs **R0, R1, R2** (with the amendment of ODM-17 it may start when `GOV-01...GOV-06` are done and `TEST-01`/`XT-01` are in flight).

## 7. Smallest production implementation milestone (proposed): M2-T0, "headless authoritative world"

A subset of M2-T with **no client, no adapter, no Game Rules, no rendering**: create a world by seed, generate on demand, edit voxels through the Editing API, save, restart, reload, compare content hashes, all through the local runtime and the World Operations interface (`XI-01` without `GM-13`).

| Measure | Value (generator) |
|---|---|
| Tasks | **57**: 12 Pre + 45 M2-T |
| Strict chain | **16** tasks: `GOV-01 -> GOV-04 -> GOV-06 -> TEST-01 -> CORE-01 -> CORE-03 -> CORE-06 -> CORE-10 -> API-01 -> API-04 -> PERS-01 -> PERS-02 -> PERS-03 -> PERS-06 -> SRV-01 -> XI-01` |
| Owners needed beyond existing roles | WorldGen (`WGEN-01..04`), Networking (`NET-01`, `NET-03`, `TGAP-03a`), Server Runtime (`SRV-01`); **no** client, adapter or gameplay owner |
| Acceptance | AC-03, AC-04, AC-05 (server side), AC-09/AC-10 (headless), AC-11, AC-12, AC-15 |
| Value | the whole authority, edit, generation and persistence stack runs under test; the first moment real feedback on the architecture exists, before any engine work |

M2-T0 is a *proposal for ordering*, not a new official milestone.

## 8. Verdicts of this document

* Ready to start implementation now: **no** (R0...R2 are open).
* Readiness is reachable without finishing the whole architecture roadmap: **yes**: only the gates above, and with ODM-17 even G0 can overlap the start of Core.
