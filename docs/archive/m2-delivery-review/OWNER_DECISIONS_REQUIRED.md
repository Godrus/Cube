# Decisions required from the Project Owner (M2 delivery)

**Status:** **Proposed.** A list of questions; nothing here is decided and a recommendation is not an approval. This list **adds to, and does not replace**, the 15 GOV-01 decisions and the items X-1...X-4, ST-1, ST-2 ([`docs/archive/gov01/OWNER-DECISION-REQUIRED.md`](../gov01/OWNER-DECISION-REQUIRED.md), [`GOV-01-DECISION-REGISTER.md`](../gov01/GOV-01-DECISION-REGISTER.md)). A decision becomes effective only as a registry entry with a confirmation source (plan 3.3); see ODM-10 for the open question of what that source can be.

## 1. New decisions (ODM-nn)

| ID | Question | Options | Recommendation | Blocks | When |
|---|---|---|---|---|---|
| **ODM-01** | How is the documentation integrated into `master`? | (a) several docs-only PRs from `master`, tree-based, spike code excluded; docs/spike decided separately (D5) / (b) one PR of the whole branch with the spike code (contradicts roadmap-v1 section 7) / (c) leave documentation on the branch | (a); PR set D1...D4 now, D5 optional; confirm the branch name (`architecure_roadmap` does not exist) and give permission to create branches and open PRs | all use of `master` as a base | **now** |
| **ODM-02** | The Geometry Processing Specification is absent. What may M2-T rely on? | (a) the owner supplies it / (b) the owner states that a Provisional registration of PR-01...PR-05 (TC-01) is acceptable under AOD-24 (scope, expiry) / (c) wait | (a) if it can be supplied; otherwise (b) | every production merge from `API-02` on (on the published ADG) | before W2 |
| **ODM-03** | The Voxel Architecture is absent. May PC-1 (TC-02) and an experimental snapshot-only persistence format (TC-08) carry M2-T? | (a) yes, with the expiry stated / (b) wait for the document / (c) a different baseline | (a) | `CORE-02/03`, `PERS-*`, formats | before W1 |
| **ODM-04** | Owners for WorldGen, Networking Adapter, Server Runtime, Client Runtime, Engine Adapter, Gameplay (Game Rules), Content/design, Art/Audio; where UI and camera live (Engine Adapter owner, or a separate Presentation owner) | name a role/person or defer each in writing | name them in the order of the waves: none needed for W1-W3; WorldGen, Networking, Server, Client, Engine Adapter, Gameplay before W4/W5; UI under the Engine Adapter owner for M2 | 18 existing + 29 new tasks | by wave; start naming now |
| **ODM-05** | Add a Game Rules module to the architecture? | (a) yes, as in [`M2_OWNERSHIP_MATRIX.md`](M2_OWNERSHIP_MATRIX.md) section 5, after the entity-model spike / (b) place gameplay in an existing module (rejected: breaks layering) / (c) postpone gameplay | (a) | all `GM-` tasks | before the first `GM-10` |
| **ODM-06** | Accept the split M2-T / M2 and the acceptance criteria T-01...T-13 and FPS-01...FPS-17? Keep the old M2 of [`DEVELOPMENT_ROADMAP.md`](../DEVELOPMENT_ROADMAP.md) as M2-T? Is a developer-only first reading enough for FPS-14? | accept / amend / reject | accept; old M2 = M2-T; FPS-14 with an outside tester, developer-only reading allowed as a first step | milestone definition | before W6 at the latest; better before W1 |
| **ODM-07** | Register the temporary contracts TC-01...TC-10 as Provisional (including the policy for production code against Proposed contracts, H-3)? | accept all / some / none | accept; TC-10 and TC-03 are needed first | `PROC-03`, `CORE-02` and later | before W1-W2 |
| **ODM-08** | Mobile: desktop acceptance for M2-T/M2 and checkpoints MAC-1 (device smoke before M3), MAC-2 (M3 baseline), MAC-3 (M5)? Who provides a physical Android device? | accept / require a device inside M2 / defer all | accept; provide a device before M3 | MAC-1, `DEV-01` | before M3 planning; the device decision earlier is better |
| **ODM-09** | Enable branch protection on `master` and required checks; Actions permissions; the project licence (the repository is public and has no licence file) | owner admin action; licence options are D-E parameter d | protect with `docs-validate` first, the full set after the skeleton; decide the licence with D-E | all PR gating | **now** |
| **ODM-10** | What counts as the owner's confirmation of a decision? Today every commit is unsigned and authored under the owner's own identity, including those made by agent sessions, so "owner-signed commit" cannot be met | (a) the owner approves PRs through GitHub review (approval = confirmation for the decisions in that PR) / (b) the owner signs commits with a key that agents do not hold / (c) the owner writes the registry entries by hand | (a) or (b); (c) for the most important decisions | X-1, GOV-01, GOV-02 and every later entry | **now** |
| **ODM-11** | Vertical extent, chunk layout and world bound (tied to GOV-01 item 10) | a bounded world with a stated size / wider coordinates / hierarchical key | decide with GOV-01 item 10; pick the vertical extent from the M2 terrain needs | `CORE-02`, generator, controller | before W1 (extent before W4) |
| **ODM-12** | Authorise isolated experiments one by one: entity-model spike (GM-02), persistence durability spike (SPK-PERS), controller/camera/combat-feel prototype, optionally a throw-away "Game Prototype 0" on the existing spike code to get playable feedback early | yes/no per experiment | yes for GM-02 and SPK-PERS; GP-0 only with the rules of [`TECHNOLOGY-GAME-TRACK-INTEGRATION.md`](../production/TECHNOLOGY-GAME-TRACK-INTEGRATION.md) section 3 | design learning | any time |
| **ODM-13** | Content data format | decide when `GM-20` starts | defer | `GM-20` | at M2 |
| **ODM-14** | Accept the proposed ADG changes: the M2 slice with the differences D-1...D-7 of [`M2_TASK_GRAPH.md`](M2_TASK_GRAPH.md), the rule R-MERGE-G1 and release closure (D-01/D-02, ST-2), the Game Rules module row | accept / amend / reject each | accept after reading the patch proposals | scheduling | before W1 |
| **ODM-15** | M3: Minimum or Recommended; mobile inside M3; co-op as a separate technical spike | choose at M3 planning | Minimum with a co-op spike; decide with M2 results | M3 | defer |

## 2. What can wait

* ODM-13 (data format), ODM-15 (M3 choices), MAC-2/MAC-3 details, a licence-sensitive part of D-E beyond the allow-list, the full crash matrix, the second device class.
* Deferrals of owners for modules not used by M2 (SDF, mesh, tools, History, Composite, Resolution, dedicated host).

## 3. Order in which the decisions unblock work

1. **ODM-10, ODM-01, ODM-09, GOV-01 items 14 and 15**: confirm how decisions are confirmed; integrate documentation; protect `master`; fix the CI parameters and the baseline.
2. **GOV-01 Core items, ODM-03, ODM-11, ODM-07 (TC-03, TC-10)**: unblock W1.
3. **ODM-02**: unblocks W2 and everything that merges after it.
4. **ODM-04 (by wave), ODM-05, ODM-12**: unblock W4/W5 and the Game Rules.
5. **ODM-06, ODM-14**: fix the target and the graph.
6. **ODM-08, ODM-15**: before M3.

## 4. Proposals that need the Integration Owner (after ODM-14)

ADG change PR (slice, Game Rules module, D-01/D-02); regeneration of Parts C and D (the generator is not in the repository, finding F-5 of GOV-01); fixes of `gov01_check.py` and `gametrack_check.py`; the `docs-validate` workflow; erratum notes for the dated audit reports; refresh of [`PROJECT_STATUS.md`](../PROJECT_STATUS.md).
