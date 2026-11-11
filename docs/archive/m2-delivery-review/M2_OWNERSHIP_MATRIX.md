# M2 ownership matrix (proposed)

**Status:** **Proposed** / **Pending Owner Approval**. No owner is assigned by this document. Task ids and counts come from [`M2_TASK_GRAPH.md`](M2_TASK_GRAPH.md) (generator output).

## 1. Categories used

| Category | Meaning |
|---|---|
| **Existing Owner (role)** | the module has an owner **role** in the documents ([`module-boundaries.md`](../../production/module-boundaries.md) A.2: Core Agent, Processing Agent, Main Agent / Integration Owner, Agents A, B, H, I). These are agent roles defined by the architecture documents; no document names a person, and no confirmation that a role is staffed exists in the repository |
| **Proposed Owner** | this review suggests a role; it is shown inside "Owner Required (proposed: ...)" and decides nothing |
| **Owner Required** | the Project Owner must decide (assign a role/person, or defer in writing) |
| **Shared Responsibility** | allowed only where a single Integration Owner (Main Agent) is named; used for three integration tasks |

## 2. Re-check of the earlier figures

| Earlier statement | Re-check |
|---|---|
| 65 technology tasks proposed for M2 | reproduced by `docs/archive/audit/m2_slice.py` (65, of which 20 UNASSIGNED) |
| 20 of them without owner | confirmed for that selection (Engine Adapter 4, Client 5, WorldGen 5, Networking 3, Server 3) |
| Game tasks are absent from the ADG | confirmed: `grep` of Part C finds no gameplay task; the module grid has no gameplay module |
| No ownership for gameplay, content, art, UI | confirmed; GOV-03's list (WorldGen, Networking, Server, Client, Engine Adapter, Import/Export, O-1, O-2) contains none of them |

The refined selection of this review ([`M2_TASK_GRAPH.md`](M2_TASK_GRAPH.md)) has **81 existing tasks**, of which **18 have no owner** (all M2-T): `LEG-01` and `LEG-02` are dropped (D-5 there), the rest is unchanged. It adds **38 new tasks, of which 29 need an owner decision**. Totals over the 119 tasks: Existing Owner (role) 69, Owner Required 47, Shared Responsibility 3.

## 3. Matrix by module

Counts are tasks in the graph (Pre / M2-T / M2). "Owner Required" rows are where the owner decides; the suggested role is only a name for the decision.

| Module / area | Category | Current state | Proposed (not assigned) | Tasks | Pre | M2-T | M2 |
|---|---|---|---|---|---|---|---|
| Geometry Core | Existing Owner (role) | Core Agent | - | 12 | 0 | 12 | 0 |
| Processing | Existing Owner (role) | Processing Agent | - | 5 | 0 | 5 | 0 |
| Query API (TGAP-01) | Owner Required | home of the query API open (PR-08) | Processing Agent | 1 | 0 | 1 | 0 |
| Editing API | Existing Owner (role) | Main Agent / Integration Owner | - | 5 | 0 | 5 | 0 |
| Editing Orchestration | Existing Owner (role) | Agent A | - | 5 | 0 | 5 | 0 |
| Voxel backend and providers | Existing Owner (role) | Agent B (provider ownership pending O-1) | confirm O-1 | 4 | 0 | 4 | 0 |
| Persistence (incl. record store) | Existing Owner (role) | Agent H | - | 9 | 1 | 4 | 4 |
| World Generation | Owner Required | UNASSIGNED | WorldGen Agent | 6 | 0 | 4 | 2 |
| Networking Adapter | Owner Required | UNASSIGNED (O-3) | Networking Agent | 5 | 0 | 4 | 1 |
| Server Runtime | Owner Required | UNASSIGNED | Server Runtime Agent | 3 | 0 | 3 | 0 |
| Client Runtime | Owner Required | UNASSIGNED | Client Runtime Agent | 6 | 0 | 5 | 1 |
| Engine Adapter (incl. UI and camera, option 1) | Owner Required | UNASSIGNED (O-5) | Engine Adapter Agent | 6 | 0 | 5 | 1 |
| Game Rules (proposed new module) | Owner Required | does not exist | Gameplay Agent | 17 | 2 | 4 | 11 |
| Content design | Owner Required | does not exist | Content/Design owner | 1 | 1 | 0 | 0 |
| Art and audio assets | Owner Required | does not exist | Art/Audio owner | 2 | 0 | 1 | 1 |
| Quality, build tooling | Existing Owner (role) | Agent I | - | 15 | 2 | 9 | 4 |
| Architecture checks and Integration Owner tasks | Existing Owner (role) | Main Agent / Integration Owner | - | 7 | 5 | 2 | 0 |
| Owner decisions (GOV tasks) | Project Owner | Project owner | - | 6 | 6 | 0 | 0 |
| Integration tasks XI-01..04 | Shared Responsibility (XI-01..03) / Existing (XI-04) | Integration Owner (Main Agent) with module owners | - | 4 | 0 | 2 | 2 |

Notes: the table is generator output (`_build/m2_graph.py`); every one of the 119 tasks is in exactly one row. `GOV-04`, `GOV-05` and `GOV-06` are listed under their owners per the ADG (Integration Owner, Agent I); the other GOV tasks are the Project Owner's own decisions.

### 3.1 The UI question (one module, one owner)

Menus, HUD and the camera are presentation. The documents place "GDScript glue" in the Engine Adapter module and forbid algorithms and world logic there; they do not name an owner of UI scenes. Two options (decision ODM-04): **(1)** UI and camera belong to the Engine Adapter owner for M2 (one owner, no new module; recommended while the UI is small); **(2)** a separate Presentation module with its own owner (justified once M3's UI grows: inventory, crafting, quests, map). This review assigns `GM-14`, `GM-15`, `GM-31` to the Engine Adapter owner under option (1). Art assets are not code and belong to the Art/Audio owner.

## 4. Principle check

| Principle | Check |
|---|---|
| One module, one responsible owner | every task has exactly one owner entry; the only joint rows are the three integration tasks, where the Integration Owner is named; `GM-30` and `GM-33` were split so that Game Rules and Client Runtime each own their side |
| One Integration Owner | Main Agent / Integration Owner for all integration, ADG changes and contract preparation; the Project Owner approves |
| Lower layers do not depend on upper layers | verified by the generator: no CORE, PROC, API, ORCH, VOX, PERS or WGEN task depends on a `GM-` task |
| Gameplay is not in Geometry Core | Game Rules is a separate module (section 5); `CORE-*` contains no gameplay type; `GM-10` carries the archcheck rule |
| Processing holds no game business logic | `TGAP-01` (queries) is geometric; no creature, combat or item concept enters Processing |
| Persistence does not manage Editing | persistence stores opaque game records (`TGAP-02`) and implements the storage ports; it neither calls `IEditingService` nor interprets game data |
| QA is not a runtime dependency | `cube-testkit` and `cube-integration-tests` stay test-time; no production crate depends on them |

## 5. Game Rules module (proposal)

Needed because no module in the grid can hold authoritative game rules, and putting them into Core, Processing, Orchestration or the server host would break the principles above. This is the **smallest** module that satisfies them; no further abstraction layers are proposed.

| Aspect | Proposal |
|---|---|
| Name | to be chosen by the owner (working name: Game Rules) |
| Responsibility | authoritative game state and rules on the server side: player state and controller, world interaction requests, creatures (kinds, spawn rules, AI), combat and health, items, inventory, equipment, loot, death and respawn, the player record |
| Not responsible for | geometry, world generation, persistence mechanics, networking, rendering, UI, assets |
| Position in the grid | above the Editing API ports and Processing read views, below the server composition root: it depends on Core (read types), Processing (snapshots, queries), Editing API (`IEditingService`, origin and authorisation types); it is composed by the server host. The server host depends on it, not the other way |
| Interfaces it **consumes** | world read (snapshot, collision/ray query), edit request (`IEditingService`), tick, seeded random streams, record store (`TGAP-02`), spawn candidates (`TGAP-08`) |
| Interfaces it **provides** | game state and events for replication (`GM-30`), definitions loader, player/creature stepping on the tick |
| Dependencies forbidden | godot, I/O, clocks, global randomness, persistence internals, networking |
| Player | player entity, state, controller; input arrives as intents |
| Combat | attack resolution against hit volumes; damage and health; death events |
| Items | definitions as data, inventory (server-owned), equipment, derived stats, loot rolls from seeded streams |
| Creatures | entity kinds, deterministic spawn at chunk load, state-machine behaviours |
| World | reads through the port; changes terrain **only** through `IEditingService` with origin Player (or a creature-originated edit, never in M2) |
| ADG/archcheck consequences | a new row/column in Part A/B (D: core, processing, editing-api; no dependency on persistence, worldgen, net); `archcheck` rule for the module; owner and acceptance authority |
| Open | entity model (ECS or explicit domain model): decided by the spike `GM-02` before `GM-10` |

## 6. Candidate pool for unassigned modules (information, not a recommendation)

M2 does not use the SDF backend (Agent C), the mesh backend (Agent D), sculpt and Boolean tools (Agents E, F), History (Agent G), Composite and Resolution agents. If these roles are real parallel workers, their capacity is the natural pool for the unassigned modules. Assigning them is the owner's decision; the review does not assume it.

## 7. Appendix: generator tally by owner

| Module / area | Owner (as in the documents, or proposed for new tasks) | Category | Tasks | Pre | M2-T | M2 |
|---|---|---|---|---|---|---|
| Architecture checks | Main Agent / Integration Owner | Existing Owner (role) | 2 | 1 | 1 | 0 |
| Art and audio | Art/audio owner | Owner Required | 2 | 0 | 1 | 1 |
| Build and tooling | Agent I | Existing Owner (role) | 1 | 0 | 1 | 0 |
| Client Runtime | Client Runtime | Owner Required | 5 | 0 | 5 | 0 |
| Client runtime | Client Runtime owner | Owner Required | 1 | 0 | 0 | 1 |
| Content design | Content/design owner | Owner Required | 1 | 1 | 0 | 0 |
| Contracts (threading) | Main Agent / Integration Owner | Existing Owner (role) | 1 | 1 | 0 | 0 |
| Editing API | Main Agent / Integration Owner | Existing Owner (role) | 5 | 0 | 5 | 0 |
| Editing Orchestration | Agent A | Existing Owner (role) | 5 | 0 | 5 | 0 |
| Engine Adapter | Engine Adapter | Owner Required | 3 | 0 | 3 | 0 |
| Game Rules (proposed new module) | Gameplay owner | Owner Required | 17 | 2 | 4 | 11 |
| Game UI / client presentation | Engine Adapter owner | Owner Required | 3 | 0 | 2 | 1 |
| Geometry Core | Core Agent | Existing Owner (role) | 12 | 0 | 12 | 0 |
| Governance | Main Agent / Integration Owner | Existing Owner (role) | 1 | 0 | 1 | 0 |
| Integration | Main Agent / Integration Owner | Existing Owner (role) | 1 | 0 | 0 | 1 |
| Integration | Main Agent / Integration Owner | Shared Responsibility | 3 | 0 | 2 | 1 |
| Networking Adapter | Networking Adapter | Owner Required | 3 | 0 | 3 | 0 |
| Networking Adapter | Networking owner | Owner Required | 2 | 0 | 1 | 1 |
| Persistence | Agent H | Existing Owner (role) | 9 | 1 | 4 | 4 |
| Processing | Processing Agent | Existing Owner (role) | 5 | 0 | 5 | 0 |
| Processing | Processing Agent | Owner Required | 1 | 0 | 1 | 0 |
| Project owner decisions | Project owner | Existing Owner (role) | 6 | 6 | 0 | 0 |
| Project owner decisions | Main Agent / Integration Owner | Existing Owner (role) | 2 | 2 | 0 | 0 |
| Project owner decisions | Agent I (QA and Benchmarks) | Existing Owner (role) | 1 | 1 | 0 | 0 |
| Quality (QA) | Agent I (QA and Benchmarks) | Existing Owner (role) | 12 | 1 | 7 | 4 |
| Quality (QA) | Integration Owner (with QA) | Existing Owner (role) | 1 | 1 | 0 | 0 |
| Quality (QA) | Agent I | Existing Owner (role) | 1 | 0 | 1 | 0 |
| Server Runtime | Server Runtime | Owner Required | 3 | 0 | 3 | 0 |
| Voxel backend | Agent B | Existing Owner (role) | 4 | 0 | 4 | 0 |
| World Generation | WorldGen | Owner Required | 4 | 0 | 4 | 0 |
| World Generation | WorldGen owner | Owner Required | 2 | 0 | 0 | 2 |
