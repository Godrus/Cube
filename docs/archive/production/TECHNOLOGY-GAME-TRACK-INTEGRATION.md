# Technology Track and Game Track: integration plan

**Status:** **Proposed** [NP]. A way of working for the owner to accept, amend or reject. It adds no task to the ADG, authorises no work, and changes no governance gate. Labels as in [`GAME_TRACK_ROADMAP.md`](GAME_TRACK_ROADMAP.md).
**Premise:** the nearest major product goal is M2 First Playable ([`FIRST_PLAYABLE_SCOPE.md`](FIRST_PLAYABLE_SCOPE.md)). Technology is judged by whether it brings the game closer; the game is judged by whether it can run on the architecture the project has decided.

## 1. The convergence model

```
Technology Track:  PS-0 -> PS-1 -> PS-2 -> ... (ADG stages)  --- minimal, stable capabilities, early ---+
                                                                                                      v
                                                                         Playable increments PI-0 ... PI-7  ->  M2  ->  M3
                                                                                                      ^
Game Track:        design -> experiments -> rules/content on mocks -> rules/content on the real runtime ---+
```

* The Technology Track delivers *capabilities in thin slices* that a game consumer can use as soon as they exist (section 5), not whole stages.
* The Game Track develops rules, content and tests against a narrow **world port** (section 4.1) that has an in-memory stand-in first and the real runtime later, so that it never waits for the full geometry system.
* Integration happens at the end of **every playable increment**, not at the end of a stage.
* **Convergence at M2:** the M2 technology slice ([`FIRST_PLAYABLE_DEPENDENCIES.md`](FIRST_PLAYABLE_DEPENDENCIES.md)) plus the M2 content and rules ([`FIRST_PLAYABLE_SCOPE.md`](FIRST_PLAYABLE_SCOPE.md)) meet in the real client; M2 is accepted by the player-facing criteria FPS-01...FPS-17 and doubles as validation of the architecture (authority, edit path, persistence, streaming).
* **Convergence at M3:** the technology added for M3 (generation features, building aids, optional co-op) plus the M3 content meet in a representative slice, accepted by VSG-01...VSG-14. M3 additionally validates *scale*: content volume, entity counts, save size.

## 2. Four levels of work (what is allowed when)

| Level | Definition | Allowed today (PS-0 open)? | Condition |
|---|---|---|---|
| **L1 Documentation and design** | scope, rules, content tables, scenario scripts, UI sketches, art/audio direction, module designs | **Yes** | stays Proposed; no status changes |
| **L2 Isolated experiments and spikes** | non-production code to answer a design or feasibility question | **Yes, with the owner's go-ahead per experiment** (the plan lists only SPK-PERS and SPK-NET as spikes; game experiments are proposed additions) | rules in section 3 |
| **L3 Approved implementation** | production code behind approved contracts | **No** | PS-0 closed (G0), owners assigned, the module exists in the ADG, contracts Owner-Approved or Provisional (policy H-3) |
| **L4 Production integration** | merge into the production base | **No** | ADG merge-after conditions met (including G1 under the proposed D-01 rule); CI and archcheck running as required checks |

## 3. Rules for experiments (L2) [NP]

1. **Labelled and isolated:** a spike branch or a directory outside the production base; the label *experimental, not for production* is in the README and in the build. The existing rule stays: spike code never merges; moving it into production needs a separate owner decision ([`AGENT_HANDOFF.md`](../AGENT_HANDOFF.md)).
2. **What flows back is knowledge, not code:** requirements, tuned parameters, test scenarios, measurements, a written finding. Parameters go back as data tables; scenarios go back as acceptance tests.
3. **Engine scripts may be used for speed, but are throwaway** (AOD-07 governs production: GDScript is UI and glue only, authority lives in Rust). An experiment must not be the basis of a save format, a protocol, or a contract.
4. **Decision-bounded, not date-bounded:** each experiment states the question it answers and the finding that ends it. No dates are proposed here.
5. **No governance bypass:** an experiment may not approve, freeze or promote any contract, may not use the Editing API as a de-facto contract, and must say if it writes outside the Editing API (as the existing spike does, documented as non-compliant).
6. **Registers:** each experiment gets an entry in the simplification/experiment register (section 7).

## 4. What can proceed before PS-1, and what cannot

| Item | Level | Now? | Why |
|---|---|---|---|
| Review and refine [`FIRST_PLAYABLE_SCOPE.md`](FIRST_PLAYABLE_SCOPE.md); a game design document for M2 (rules, content tables, stat tables) | L1 | **Yes** | documents only; it is the substitute for the missing GDD for this scope |
| Scenario library: player stories written as steps with expected results (FPS-01...FPS-17 as scripts) | L1 | **Yes** | become automated acceptance tests later |
| Item, creature, loot and spawn tables as plain tables (format not decided) | L1 | **Yes** | the content-data format is an open decision; tables are format-neutral |
| UI sketches, HUD layout, control scheme, camera design | L1 | **Yes** | |
| Art direction notes, placeholder asset list, asset naming and import conventions | L1 | **Yes** | |
| Game Rules module design (ports, responsibilities, relation to Editing API) | L1 | **Yes** | needs owner and ADG change to become real |
| Entity-model spike (ECS vs explicit domain model, REF-13) | L2 | **Yes, with go-ahead** | roadmap-v1 gate 6 marks it "Needs spike" |
| Character-controller, camera and combat-feel prototype with placeholders | L2 | **Yes, with go-ahead** | answers design questions about feel; throwaway |
| AI state-machine and spawn-rule prototype | L2 | **Yes, with go-ahead** | |
| Loot/progression tuning experiments (spreadsheet or script) | L2 | **Yes** | parameters flow back as data |
| Production Game Rules, controller, creatures, inventory | L3 | **No** | no owner; no module in the ADG; G0 open; contracts Proposed |
| Production client runtime, engine adapter, embedded server, world generation | L3 | **No** | PS-12/PS-13/PS-11; owners unassigned; PS-1..PS-10 first |
| Moving spike code into production | -- | **Prohibited** | separate owner decision |
| Any commit/push without the owner's permission | -- | **Prohibited** | standing rule |

## 5. Avoiding the wait for the whole geometry system

### 5.1 The world port [NP]
The game does not need "the geometry system"; it needs a small set of capabilities, expressed as an interface that reuses the existing Core/Processing/Editing concepts (no parallel world model):

| Capability | Meaning | Real provider (ADG) | Stand-in for early work |
|---|---|---|---|
| read | voxel/material at a position; a read-only view of a neighbourhood | Core snapshot + Processing (PROC-01) | in-memory voxel array in the test kit |
| query | ray walk, overlap/collision of a volume | collision provider (VOX-05), query home open | simple brick-based queries |
| edit | request to place/destroy voxels as an operation | `IEditingService` (API-02, ORCH-01..05) | recorder that applies to the in-memory world |
| events | chunk loaded/unloaded, voxel changed | observers (ORCH-04), replication (SRV-04) | direct callbacks |
| persist | save/restore of game records next to the world | persistence (PERS-*) + a player record (gap) | files in a temp directory |
| seed | deterministic random streams per purpose | world seed (CORE-16) | seeded generator |
| time | fixed simulation tick | server tick (SRV-03) | manual stepping in tests |

The port is a *design* artefact until the Game Rules module is approved; its parts must be traced to existing contracts, and any part that has no contract becomes an open item.

### 5.2 Scenario scripts as the common currency
Every M2 capability has a **scenario script** (L1). Scripts run first against the stand-ins, headless; later, unchanged, against the real runtime. A capability is "done" when its script passes on the real runtime and a player can repeat it in the client. This lets Game Track work be verified long before graphics exist, and gives the Technology Track real consumers.

### 5.3 Milestone slices and priority
* Define the M2 **slice** in the ADG (roadmap gap R-01) so that the 65 technology tasks of [`FIRST_PLAYABLE_DEPENDENCIES.md`](FIRST_PLAYABLE_DEPENDENCIES.md) are scheduled first and are not delayed by stage-level gates for work M2 does not use (proposal P-A, P-B there).
* **Priority rule:** for any owner, an M2-slice task precedes a non-slice task. Non-slice technology may proceed in parallel only where it does not take an M2-slice owner's time.
* **Consumer rule:** no technology task enters the M2 slice without a named game consumer and a scenario; no game system is specified without a world-port capability that serves it.

### 5.4 Time-to-playable is mostly governance
The published ADG puts the M2 path behind: the 15 GOV-01 decisions, G0, owners for 20 of the 65 slice tasks, and two absent documents (Processing Specification, Voxel Architecture). These, not coding speed, decide how early M2 can exist. The highest-leverage actions are decisions: supply or replace the two documents (AOD-24), assign owners, accept slices, register Provisional groups. Until then the Game Track can still advance at L1/L2.

## 6. Playable increments (PI) [NP]

An increment is a state of the product that someone can *run and judge*. The sequence is an illustration of the order of dependencies; each increment names its demonstration. Task identifiers are ADG tasks; "after" means the real provider exists.

| PI | Player/judge can... | Game Track delivers | Technology needed (ADG) | Stage-independent? |
|---|---|---|---|---|
| PI-0 (now) | read and run the scenario library, see the design | M2 design, scenarios, tables, experiments | none | yes |
| PI-1 | run the scenarios headless against the real world core: create a world by seed, edit, save, load (no graphics) | rules and scenarios on the world port | G1; PROC-01..05; API/ORCH minimal; PERS minimal; WGEN-01 | after G0 and those tasks |
| PI-2 | open a window, fly around the generated world and edit voxels | HUD sketch, controls | client/adapter slice, VOX-02, CLI-01..02 | after PS-12 slice |
| PI-3 | walk and jump as the character in the authoritative embedded runtime | controller rules, camera | SRV-01/03, NET-03, VOX-05, CLI-04 | after server slice |
| PI-4 | meet and fight a creature | creature kinds, combat rules, spawn rules | tick, replication (SRV-04), WGEN spawn step | after Game Rules module exists |
| PI-5 | loot, inventory, equipment effect | item/loot tables, player record | player persistence (gap), PERS-* | after the gap is closed |
| PI-6 | the second terrain variant, complete HUD and menus, death/respawn | content and UI complete | WGEN variant, UI | |
| PI-7 | crash/recovery, reload-and-continue, outside tester session | hardening, playtest note | PERS-05/06, crash matrix | |
| **M2** | all FPS criteria | -- | -- | owner acceptance |

PI-1...PI-3 can be reached by the Technology Track with almost no gameplay; PI-4...PI-6 need the Game Rules module. The point of the increments is that gameplay work (PI-4...) has already been developed and tested on the stand-ins when the real provider arrives.

## 7. Registers

**Simplification register [NP]** (one row per temporary solution; created with the first experiment or implementation; initial rows are in [`FIRST_PLAYABLE_SCOPE.md`](FIRST_PLAYABLE_SCOPE.md) section 5). Fields: id, system, what is simplified, why acceptable, what it preserves, what it must not prevent, replacement condition and milestone, owner, status (open / replaced / waived by the owner). A simplification without a replacement condition is not allowed. **Technology debt rule:** when a simplification is replaced, its register row is closed with a reference to the evidence (scenario passing on the full version).

**Experiment register [NP]:** id, question, branch/location, owner, finding, what flowed back (data, scenarios, requirements), date closed.

## 8. Governance still blocking, by item

| Blocker | Affects | Needed |
|---|---|---|
| PS-0 open (15 GOV-01 decisions, GOV-02...06, TEST-01, XT-01) | L3/L4 of everything | owner decisions; CI; archcheck; baseline branch |
| No owners for gameplay, content, art/audio, UI; five technology areas unassigned | L3 | GOV-03 or written deferral, extended list |
| Game Rules module absent from ADG | L3 for gameplay | ADG Part A/B change, owner decision |
| Contracts Proposed (PR-*, E-*), production merges against them (H-3) | PS-2, PS-3 merges | Provisional groups or a changed ADG |
| ADG gaps D-01/D-02, no milestone slices (R-01) | scheduling of M2 | owner acceptance (ST-2), slice definition |
| Processing Specification and Voxel Architecture absent | G2, formats, ADP-01 | supply or record an AOD-24 alternative |
| Player-state persistence, gameplay protocol messages, query API home, entity model | M2 rows 6, 8, 10, 13 | design and decisions |
| GDD/TDD absent | final scope approval | supply, or approve the baseline BL-2 and this scope |

## 9. Risks and mitigations

| Risk | Sign | Mitigation |
|---|---|---|
| The Technology Track runs on without a playable result | stages close, no increment runs | priority rule; increments with demonstrations; M2 slice |
| The Game Track builds on sand (stand-ins diverge from the real runtime) | scenarios pass on stand-ins, fail on the real runtime | world port traced to contracts; the same scripts run on both; port conformance tests |
| A prototype becomes the product | prototype code appears in production, or formats depend on it | L2 rules (section 3); register; only data and scenarios flow back |
| Scope grows beyond M2 | content counts creep; stretch items become requirements | counts are proposals; acceptance list fixed; stretch items marked |
| Ownership vacuum | 31 % of the slice and all gameplay unassigned | GOV-03 extension; no L3 start without owner |
| Engine lock-in assumed | rules written in engine scripts | AOD-06/07: rules in Rust; adapter thin |
| Integration late and painful | big-bang at M2 | increments PI-1...PI-7 |
| Mobile postponed until too late | desktop-only assumptions | AOD-08 mobile-impact statement per workstream; device checkpoint before M3 |
| Absent external documents block everything | EXT-GPS, EXT-VA-RG1 | owner action first; AOD-24 alternative recorded with scope and expiry |

## 10. Recommended next steps (all proposals; none starts implementation)

1. Owner reads [`FIRST_PLAYABLE_SCOPE.md`](FIRST_PLAYABLE_SCOPE.md) and decides the questions of [`GAME_TRACK_ROADMAP.md`](GAME_TRACK_ROADMAP.md) section 10.
2. Owner decides how to treat the two absent documents (supply or record an alternative): the highest-leverage decision for M2.
3. Add owners for gameplay, content, art/audio and UI to the GOV-03 list; assign or defer the five technology areas.
4. Authorise L2 experiments one by one (entity-model spike first), with the rules of section 3.
5. Produce the M2 design document and the scenario library at L1 (no governance gate involved).
6. After the owner decides, ask the Integration Owner to prepare the ADG changes: milestone slice for M2, Game Rules module, D-01/D-02 correction, Provisional groups.
7. Keep the GOV-01 decisions and PS-0 closure on the critical path: nothing at L3/L4 starts before G0.
