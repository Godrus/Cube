# Cube Decision Registry

The single place where Cube decisions are recorded. It replaces the competing recording rules of `production-architecture-plan.md` 3.3, `owner-architecture-decisions-v1.md` section 4, `owner-approval-matrix-v1.md` section 12 and the ODM-10 protocol drafts (they now refer here; see DEC-002). Existing labels (AOD-nn, OAM-nn, ACR-nnn, GOV-01#n, ODM-nn, OCC-nn) are kept as `aliases` and are not renumbered.

## Rules (decided by DEC-002)

**Status** (one value): `PROPOSED` (suggestion, not decided) · `RECORDED` (the owner's decision transcribed by an agent from a source that cannot be verified in the repository; awaiting an explicit owner confirmation) · `CONFIRMED` · `DEFERRED` (with a trigger) · `SUPERSEDED` · `REJECTED`.

**Level:** `CONCEPT` (a goal, product requirement or principle) · `TECHNICAL` (an algorithm, format, width, module structure, public API, ACR, contract, toolchain or stage). A CONCEPT entry never approves a technical detail; its `not_decided` list names what stays open. Contracts and tasks cite a CONCEPT entry only as a requirement source.

**decided_by:** `OWNER` (the owner's own statement) · `OWNER (as transcribed by an agent)` (RECORDED) · `AGENT_DELEGATED` (a reversible engineering decision made by an agent under DEC-001; it names its `delegation_basis` and the owner may reject or amend it at any time) · `PENDING`.

**Sources accepted for CONFIRMED:** an explicit owner statement in chat (E-CHAT); a pull-request approval; an owner-written text in the repository; an engineering decision under DEC-001 (E-DELEGATION, TECHNICAL level only). A commit under the owner's Git account is not a decision. Cryptographic signatures are optional. If the primary source is unavailable the entry says `UNAVAILABLE`; no link or proof is invented.

**Dates:** `decided_on` is the date given by the source or `UNKNOWN`; `recorded_on` is when the entry entered the repository; `confirmed_on` is the date of the confirming act and is never back-dated.

**Changes:** entries are append-only. A revision is a new entry with `supersedes`; the old entry gets `superseded_by`. Agents may create PROPOSED and RECORDED entries and, within DEC-001, AGENT_DELEGATED TECHNICAL entries. Only the owner confirms CONCEPT entries, confirms RECORDED entries, supersedes or rejects an OWNER decision. Several decisions may be registered in one commit.

**Contract link:** a contract becomes Owner-Approved (plan 3.1) through a CONFIRMED TECHNICAL entry that names it. The contract status model of plan 3.1 is unchanged.

Entry fields: id, aliases, title, category, level, status, statement, not_decided, decided_by, delegation_basis, decided_on, decided_on_basis, recorded_on, confirmed_on, confirmation_evidence, source, source_availability, owner, refines, conflicts_with, affected_documents, history, supersedes, superseded_by.

---

## DEC-001  Delegation of engineering decisions to agents
id: DEC-001
aliases: [ODM-12 (partly)]
category: PROCESS
level: TECHNICAL
status: CONFIRMED
statement: >
  The owner allows agents to: create and switch working branches; commit through the owner's Git identity; push working branches; change architecture documents and the decision registry; implement production code under the approved architectural principles; create and run tests; configure CI; take low-level engineering decisions that do not contradict the confirmed Cube concept. Not allowed without separate permission: force push, destructive reset, deleting others' changes, merge into master. The grant does not allow presenting an agent's product proposals as historically approved owner decisions. Reversible engineering questions are decided by the agent; the owner is asked only if the decision is needed for the next stage, cannot safely be postponed, materially affects architecture or product concept, and has no reversible variant.
decided_by: OWNER
decided_on: 2026-11-09
decided_on_basis: date of the owner's message
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT (explicit owner statement in chat; permitted source under DEC-002)
source: OWNER-CHAT-2026-11-09 (instruction 'Close Governance, Complete PS-0, Start Production Development') sections 1, 2, 12
source_availability: UNAVAILABLE in repository (session transcript only)
owner: Project Owner
affected_documents: [docs/decisions/decision-registry.md]
history:
  - 2026-11-09 entry created

## DEC-002  Decision confirmation model (closes ODM-10)
id: DEC-002
aliases: [ODM-10, X-1 (mechanism part), C-1..C-8 of odm-10-governance-review.md]
category: PROCESS
level: TECHNICAL
status: CONFIRMED
statement: >
  Agents may create commits and branches through the owner's existing Git identity. A commit under the owner's account is not by itself an approval of an architectural decision. An explicit confirmation by the owner in chat is a permitted decision source. The agent transfers confirmed decisions into this registry keeping the exact wording and the available source; if the primary source is unavailable this is stated explicitly and no link or proof is invented. Cryptographic commit signatures are optional unless a separate policy requires them. A batch of confirmed decisions may be registered in one commit. An agent must not confirm product decisions on the owner's behalf. Status set: PROPOSED, RECORDED, CONFIRMED, DEFERRED, SUPERSEDED, REJECTED; level: CONCEPT or TECHNICAL; concept entries carry not_decided. 'owner-signed' (plan 3.3) means: an explicit owner statement in chat, a pull-request approval, or an owner-written text; a cryptographic signature is not required. This is the only decision-recording process; AOD section 4 and OAM section 12 refer to this registry.
decided_by: OWNER
decided_on: 2026-11-09
decided_on_basis: date of the owner's message
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT
source: OWNER-CHAT-2026-11-09 (instruction 'Close Governance, Complete PS-0, Start Production Development') section 3
source_availability: UNAVAILABLE in repository (session transcript only)
owner: Project Owner
affected_documents: [docs/production/production-architecture-plan.md, docs/decisions/owner-architecture-decisions-v1.md, docs/decisions/owner-approval-matrix-v1.md, docs/archive/owner-decision-m2t-optimization/OWNER_DECISION_REGISTER.md]
history:
  - 2026-11-09 entry created

## DEC-003  Git strategy: master / dev / feature branches
id: DEC-003
aliases: [GOV-01#15, ODM-01]
category: PROCESS
level: TECHNICAL
status: CONFIRMED
statement: >
  master is the stable release branch. dev is the integration branch. Short-lived feature branches are created from dev. Verified and implemented changes are integrated into dev. master is updated only after separate release checks and the owner's permission. The documentation branch is not merged into master; documentation is carried over in a controlled way without spike code. P-WORLD is not carried into production.
decided_by: OWNER
decided_on: 2026-11-09
decided_on_basis: date of the owner's message
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT
source: OWNER-CHAT-2026-11-09 (instruction 'Close Governance, Complete PS-0, Start Production Development') section 8
source_availability: UNAVAILABLE in repository (session transcript only)
owner: Project Owner
affected_documents: [docs/production/integration-and-qa-plan.md]
history:
  - 2026-11-09 entry created; dev created from master as a docs-only baseline (commit b9df5c4 on dev)

## DEC-004  AOD-01: Language: Rust
id: DEC-004
aliases: [AOD-01]
category: ARCHITECTURE
level: TECHNICAL
status: CONFIRMED
statement: >
  Language: Rust. Consequence as recorded: Core, WorldGen, Editing, Persistence, Server, protocol are Rust. No C#/C++ gameplay or core code.
decided_by: OWNER
decided_on: 2026-11-04
decided_on_basis: header of owner-architecture-decisions-v1.md ('AOD-01..19 were recorded from the owner's instruction in chat on 2026-11-04')
recorded_on: 05d7e7e (2026-11-04)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT (explicit owner confirmation message, session 2026-11-09; see DEC-075)
source: docs/decisions/owner-architecture-decisions-v1.md, row AOD-01
source_availability: chat UNAVAILABLE in repository; transcription IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/decisions/owner-architecture-decisions-v1.md]
history:
  - 2026-11-09 entered into the registry as RECORDED; the word 'Approved' in the AOD file is read as RECORDED until countersigned (plan 3.3 Fact)
  - 2026-11-09 RECORDED -> CONFIRMED by the owner's explicit chat confirmation of AOD-01..24 under the conditions of DEC-075
## DEC-005  AOD-02: First client: Godot 4.x
id: DEC-005
aliases: [AOD-02]
category: ARCHITECTURE
level: TECHNICAL
status: CONFIRMED
statement: >
  First client: Godot 4.x. Consequence as recorded: The first shippable client is a Godot client. No second engine is started before the replacement question (AOD-06) is explicitly opened.
decided_by: OWNER
decided_on: 2026-11-04
decided_on_basis: header of owner-architecture-decisions-v1.md ('AOD-01..19 were recorded from the owner's instruction in chat on 2026-11-04')
recorded_on: 05d7e7e (2026-11-04)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT (explicit owner confirmation message, session 2026-11-09; see DEC-075)
source: docs/decisions/owner-architecture-decisions-v1.md, row AOD-02
source_availability: chat UNAVAILABLE in repository; transcription IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/decisions/owner-architecture-decisions-v1.md]
history:
  - 2026-11-09 entered into the registry as RECORDED; the word 'Approved' in the AOD file is read as RECORDED until countersigned (plan 3.3 Fact)
  - 2026-11-09 RECORDED -> CONFIRMED by the owner's explicit chat confirmation of AOD-01..24 under the conditions of DEC-075
## DEC-006  AOD-03: Integration: Rust GDExtension
id: DEC-006
aliases: [AOD-03]
category: ARCHITECTURE
level: TECHNICAL
status: CONFIRMED
statement: >
  Integration: Rust GDExtension. Consequence as recorded: The only Rust-to-Godot path is a GDExtension library (godot-rust). No C# bridge, no sidecar process driven through OS calls.
decided_by: OWNER
decided_on: 2026-11-04
decided_on_basis: header of owner-architecture-decisions-v1.md ('AOD-01..19 were recorded from the owner's instruction in chat on 2026-11-04')
recorded_on: 05d7e7e (2026-11-04)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT (explicit owner confirmation message, session 2026-11-09; see DEC-075)
source: docs/decisions/owner-architecture-decisions-v1.md, row AOD-03
source_availability: chat UNAVAILABLE in repository; transcription IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/decisions/owner-architecture-decisions-v1.md]
history:
  - 2026-11-09 entered into the registry as RECORDED; the word 'Approved' in the AOD file is read as RECORDED until countersigned (plan 3.3 Fact)
  - 2026-11-09 RECORDED -> CONFIRMED by the owner's explicit chat confirmation of AOD-01..24 under the conditions of DEC-075
## DEC-007  AOD-04: Geometry Core is pure Rust with no Godot dependency
id: DEC-007
aliases: [AOD-04]
category: ARCHITECTURE
level: TECHNICAL
status: CONFIRMED
statement: >
  Geometry Core is pure Rust with no Godot dependency. Consequence as recorded: Core crates must build and test without Godot; no engine types in Core.
decided_by: OWNER
decided_on: 2026-11-04
decided_on_basis: header of owner-architecture-decisions-v1.md ('AOD-01..19 were recorded from the owner's instruction in chat on 2026-11-04')
recorded_on: 05d7e7e (2026-11-04)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT (explicit owner confirmation message, session 2026-11-09; see DEC-075)
source: docs/decisions/owner-architecture-decisions-v1.md, row AOD-04
source_availability: chat UNAVAILABLE in repository; transcription IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/decisions/owner-architecture-decisions-v1.md]
history:
  - 2026-11-09 entered into the registry as RECORDED; the word 'Approved' in the AOD file is read as RECORDED until countersigned (plan 3.3 Fact)
  - 2026-11-09 RECORDED -> CONFIRMED by the owner's explicit chat confirmation of AOD-01..24 under the conditions of DEC-075
## DEC-008  AOD-05: Server is a separate headless Rust process using the same Core
id: DEC-008
aliases: [AOD-05]
category: ARCHITECTURE
level: TECHNICAL
status: CONFIRMED
statement: >
  Server is a separate headless Rust process using the same Core. Consequence as recorded: A dedicated binary, no Godot linked, built from the same Core/Editing crates as the client runtime.
decided_by: OWNER
decided_on: 2026-11-04
decided_on_basis: header of owner-architecture-decisions-v1.md ('AOD-01..19 were recorded from the owner's instruction in chat on 2026-11-04')
recorded_on: 05d7e7e (2026-11-04)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT (explicit owner confirmation message, session 2026-11-09; see DEC-075)
source: docs/decisions/owner-architecture-decisions-v1.md, row AOD-05
source_availability: chat UNAVAILABLE in repository; transcription IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/decisions/owner-architecture-decisions-v1.md]
history:
  - 2026-11-09 entered into the registry as RECORDED; the word 'Approved' in the AOD file is read as RECORDED until countersigned (plan 3.3 Fact)
  - 2026-11-09 RECORDED -> CONFIRMED by the owner's explicit chat confirmation of AOD-01..24 under the conditions of DEC-075
## DEC-009  AOD-06: Godot is a temporary, replaceable frontend
id: DEC-009
aliases: [AOD-06]
category: ARCHITECTURE
level: TECHNICAL
status: CONFIRMED
statement: >
  Godot is a temporary, replaceable frontend. Consequence as recorded: Everything a frontend needs sits behind a narrow adapter; game rules, geometry, generation, persistence and protocol never live in Godot assets or scripts.
decided_by: OWNER
decided_on: 2026-11-04
decided_on_basis: header of owner-architecture-decisions-v1.md ('AOD-01..19 were recorded from the owner's instruction in chat on 2026-11-04')
recorded_on: 05d7e7e (2026-11-04)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT (explicit owner confirmation message, session 2026-11-09; see DEC-075)
source: docs/decisions/owner-architecture-decisions-v1.md, row AOD-06
source_availability: chat UNAVAILABLE in repository; transcription IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/decisions/owner-architecture-decisions-v1.md]
history:
  - 2026-11-09 entered into the registry as RECORDED; the word 'Approved' in the AOD file is read as RECORDED until countersigned (plan 3.3 Fact)
  - 2026-11-09 RECORDED -> CONFIRMED by the owner's explicit chat confirmation of AOD-01..24 under the conditions of DEC-075
## DEC-010  AOD-07: GDScript: UI and glue only
id: DEC-010
aliases: [AOD-07]
category: ARCHITECTURE
level: TECHNICAL
status: CONFIRMED
statement: >
  GDScript: UI and glue only. Consequence as recorded: No authoritative logic, no persistence, no generation, no networking in GDScript. Glue = mirroring Rust state into nodes, input forwarding, HUD.
decided_by: OWNER
decided_on: 2026-11-04
decided_on_basis: header of owner-architecture-decisions-v1.md ('AOD-01..19 were recorded from the owner's instruction in chat on 2026-11-04')
recorded_on: 05d7e7e (2026-11-04)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT (explicit owner confirmation message, session 2026-11-09; see DEC-075)
source: docs/decisions/owner-architecture-decisions-v1.md, row AOD-07
source_availability: chat UNAVAILABLE in repository; transcription IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/decisions/owner-architecture-decisions-v1.md]
history:
  - 2026-11-09 entered into the registry as RECORDED; the word 'Approved' in the AOD file is read as RECORDED until countersigned (plan 3.3 Fact)
  - 2026-11-09 RECORDED -> CONFIRMED by the owner's explicit chat confirmation of AOD-01..24 under the conditions of DEC-075
## DEC-011  AOD-08: Android and iOS are mandatory targets
id: DEC-011
aliases: [AOD-08]
category: ARCHITECTURE
level: TECHNICAL
status: CONFIRMED
statement: >
  Android and iOS are mandatory targets. Consequence as recorded: Every gate that touches runtime must state its mobile impact; mobile is not a late port.
decided_by: OWNER
decided_on: 2026-11-04
decided_on_basis: header of owner-architecture-decisions-v1.md ('AOD-01..19 were recorded from the owner's instruction in chat on 2026-11-04')
recorded_on: 05d7e7e (2026-11-04)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT (explicit owner confirmation message, session 2026-11-09; see DEC-075)
source: docs/decisions/owner-architecture-decisions-v1.md, row AOD-08
source_availability: chat UNAVAILABLE in repository; transcription IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/decisions/owner-architecture-decisions-v1.md]
history:
  - 2026-11-09 entered into the registry as RECORDED; the word 'Approved' in the AOD file is read as RECORDED until countersigned (plan 3.3 Fact)
  - 2026-11-09 RECORDED -> CONFIRMED by the owner's explicit chat confirmation of AOD-01..24 under the conditions of DEC-075
## DEC-012  AOD-09: WorldGen is pure Rust and deterministic, seed-based
id: DEC-012
aliases: [AOD-09]
category: ARCHITECTURE
level: TECHNICAL
status: CONFIRMED
statement: >
  WorldGen is pure Rust and deterministic, seed-based. Consequence as recorded: Generation is a pure function of (seed, generator version, coordinates); no floats in world-defining math, no global state, no time. Pinned hashes guard it.
decided_by: OWNER
decided_on: 2026-11-04
decided_on_basis: header of owner-architecture-decisions-v1.md ('AOD-01..19 were recorded from the owner's instruction in chat on 2026-11-04')
recorded_on: 05d7e7e (2026-11-04)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT (explicit owner confirmation message, session 2026-11-09; see DEC-075)
source: docs/decisions/owner-architecture-decisions-v1.md, row AOD-09
source_availability: chat UNAVAILABLE in repository; transcription IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/decisions/owner-architecture-decisions-v1.md]
history:
  - 2026-11-09 entered into the registry as RECORDED; the word 'Approved' in the AOD file is read as RECORDED until countersigned (plan 3.3 Fact)
  - 2026-11-09 RECORDED -> CONFIRMED by the owner's explicit chat confirmation of AOD-01..24 under the conditions of DEC-075
## DEC-013  AOD-10: Renderer: `faces` is the reference; `greedy` is an experimental candid
id: DEC-013
aliases: [AOD-10]
category: ARCHITECTURE
level: TECHNICAL
status: CONFIRMED
statement: >
  Renderer: `faces` is the reference; `greedy` is an experimental candidate until a real mobile benchmark. Consequence as recorded: `faces` is the default and the correctness oracle; `greedy` may not become the default before a physical-device comparison.
decided_by: OWNER
decided_on: 2026-11-04
decided_on_basis: header of owner-architecture-decisions-v1.md ('AOD-01..19 were recorded from the owner's instruction in chat on 2026-11-04')
recorded_on: 05d7e7e (2026-11-04)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT (explicit owner confirmation message, session 2026-11-09; see DEC-075)
source: docs/decisions/owner-architecture-decisions-v1.md, row AOD-10
source_availability: chat UNAVAILABLE in repository; transcription IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/decisions/owner-architecture-decisions-v1.md]
history:
  - 2026-11-09 entered into the registry as RECORDED; the word 'Approved' in the AOD file is read as RECORDED until countersigned (plan 3.3 Fact)
  - 2026-11-09 RECORDED -> CONFIRMED by the owner's explicit chat confirmation of AOD-01..24 under the conditions of DEC-075
## DEC-014  AOD-11: An ARM64 emulator never replaces a physical device
id: DEC-014
aliases: [AOD-11]
category: ARCHITECTURE
level: TECHNICAL
status: CONFIRMED
statement: >
  An ARM64 emulator never replaces a physical device. Consequence as recorded: Emulator runs may be reported as functional checks only; mobile readiness needs a physical device (Android ARM64) and an iOS device.
decided_by: OWNER
decided_on: 2026-11-04
decided_on_basis: header of owner-architecture-decisions-v1.md ('AOD-01..19 were recorded from the owner's instruction in chat on 2026-11-04')
recorded_on: 05d7e7e (2026-11-04)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT (explicit owner confirmation message, session 2026-11-09; see DEC-075)
source: docs/decisions/owner-architecture-decisions-v1.md, row AOD-11
source_availability: chat UNAVAILABLE in repository; transcription IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/decisions/owner-architecture-decisions-v1.md]
history:
  - 2026-11-09 entered into the registry as RECORDED; the word 'Approved' in the AOD file is read as RECORDED until countersigned (plan 3.3 Fact)
  - 2026-11-09 RECORDED -> CONFIRMED by the owner's explicit chat confirmation of AOD-01..24 under the conditions of DEC-075
## DEC-015  AOD-12: `OS.execute` (and any child-process or shell call) is forbidden in the
id: DEC-015
aliases: [AOD-12]
category: ARCHITECTURE
level: TECHNICAL
status: CONFIRMED
statement: >
  `OS.execute` (and any child-process or shell call) is forbidden in the production runtime. Consequence as recorded: No GDScript, adapter or client code may start processes, call `getprop` or shell out. Test tooling runs on the host, never inside the app.
decided_by: OWNER
decided_on: 2026-11-04
decided_on_basis: header of owner-architecture-decisions-v1.md ('AOD-01..19 were recorded from the owner's instruction in chat on 2026-11-04')
recorded_on: 05d7e7e (2026-11-04)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT (explicit owner confirmation message, session 2026-11-09; see DEC-075)
source: docs/decisions/owner-architecture-decisions-v1.md, row AOD-12
source_availability: chat UNAVAILABLE in repository; transcription IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/decisions/owner-architecture-decisions-v1.md]
history:
  - 2026-11-09 entered into the registry as RECORDED; the word 'Approved' in the AOD file is read as RECORDED until countersigned (plan 3.3 Fact)
  - 2026-11-09 RECORDED -> CONFIRMED by the owner's explicit chat confirmation of AOD-01..24 under the conditions of DEC-075
## DEC-016  AOD-13: Server authority: the authoritative world state lives on the server
id: DEC-016
aliases: [AOD-13]
category: ARCHITECTURE
level: TECHNICAL
status: CONFIRMED
statement: >
  Server authority: the authoritative world state lives on the server. Consequence as recorded: Any state that matters (voxels, entities, inventory, time) is decided and stored by the server; replication goes server to client.
decided_by: OWNER
decided_on: 2026-11-04
decided_on_basis: header of owner-architecture-decisions-v1.md ('AOD-01..19 were recorded from the owner's instruction in chat on 2026-11-04')
recorded_on: 05d7e7e (2026-11-04)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT (explicit owner confirmation message, session 2026-11-09; see DEC-075)
source: docs/decisions/owner-architecture-decisions-v1.md, row AOD-13
source_availability: chat UNAVAILABLE in repository; transcription IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/decisions/owner-architecture-decisions-v1.md]
history:
  - 2026-11-09 entered into the registry as RECORDED; the word 'Approved' in the AOD file is read as RECORDED until countersigned (plan 3.3 Fact)
  - 2026-11-09 RECORDED -> CONFIRMED by the owner's explicit chat confirmation of AOD-01..24 under the conditions of DEC-075
## DEC-017  AOD-14: The client is never the source of truth
id: DEC-017
aliases: [AOD-14]
category: ARCHITECTURE
level: TECHNICAL
status: CONFIRMED
statement: >
  The client is never the source of truth. Consequence as recorded: The client holds a cache and a prediction; it may be wrong and is corrected. No client-side save is authoritative.
decided_by: OWNER
decided_on: 2026-11-04
decided_on_basis: header of owner-architecture-decisions-v1.md ('AOD-01..19 were recorded from the owner's instruction in chat on 2026-11-04')
recorded_on: 05d7e7e (2026-11-04)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT (explicit owner confirmation message, session 2026-11-09; see DEC-075)
source: docs/decisions/owner-architecture-decisions-v1.md, row AOD-14
source_availability: chat UNAVAILABLE in repository; transcription IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/decisions/owner-architecture-decisions-v1.md]
history:
  - 2026-11-09 entered into the registry as RECORDED; the word 'Approved' in the AOD file is read as RECORDED until countersigned (plan 3.3 Fact)
  - 2026-11-09 RECORDED -> CONFIRMED by the owner's explicit chat confirmation of AOD-01..24 under the conditions of DEC-075
## DEC-018  AOD-15: User changes go through the Editing API
id: DEC-018
aliases: [AOD-15]
category: ARCHITECTURE
level: TECHNICAL
status: CONFIRMED
statement: >
  User changes go through the Editing API. Consequence as recorded: Every user-originated change to geometry is an operation with an operation id, validation, revision effect and history semantics.
decided_by: OWNER
decided_on: 2026-11-04
decided_on_basis: header of owner-architecture-decisions-v1.md ('AOD-01..19 were recorded from the owner's instruction in chat on 2026-11-04')
recorded_on: 05d7e7e (2026-11-04)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT (explicit owner confirmation message, session 2026-11-09; see DEC-075)
source: docs/decisions/owner-architecture-decisions-v1.md, row AOD-15
source_availability: chat UNAVAILABLE in repository; transcription IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/decisions/owner-architecture-decisions-v1.md]
history:
  - 2026-11-09 entered into the registry as RECORDED; the word 'Approved' in the AOD file is read as RECORDED until countersigned (plan 3.3 Fact)
  - 2026-11-09 RECORDED -> CONFIRMED by the owner's explicit chat confirmation of AOD-01..24 under the conditions of DEC-075
## DEC-019  AOD-16: Procedural generation is a separate WorldGen/System boundary, not an o
id: DEC-019
aliases: [AOD-16]
category: ARCHITECTURE
level: TECHNICAL
status: CONFIRMED
statement: >
  Procedural generation is a separate WorldGen/System boundary, not an ordinary user edit. Consequence as recorded: Generated data does not enter history, is not a user operation, and has its own revision and persistence rules.
decided_by: OWNER
decided_on: 2026-11-04
decided_on_basis: header of owner-architecture-decisions-v1.md ('AOD-01..19 were recorded from the owner's instruction in chat on 2026-11-04')
recorded_on: 05d7e7e (2026-11-04)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT (explicit owner confirmation message, session 2026-11-09; see DEC-075)
source: docs/decisions/owner-architecture-decisions-v1.md, row AOD-16
source_availability: chat UNAVAILABLE in repository; transcription IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/decisions/owner-architecture-decisions-v1.md]
history:
  - 2026-11-09 entered into the registry as RECORDED; the word 'Approved' in the AOD file is read as RECORDED until countersigned (plan 3.3 Fact)
  - 2026-11-09 RECORDED -> CONFIRMED by the owner's explicit chat confirmation of AOD-01..24 under the conditions of DEC-075
## DEC-020  AOD-17: Persistence: an untouched world is stored as seed + generator version;
id: DEC-020
aliases: [AOD-17]
category: ARCHITECTURE
level: TECHNICAL
status: CONFIRMED
statement: >
  Persistence: an untouched world is stored as seed + generator version; user changes as versioned deltas/journal. Consequence as recorded: The save of an untouched world is tiny and regenerated; user changes are an append-only versioned journal with snapshots/compaction as an optimisation. Generator versions must stay reproducible (or be baked) for every saved world.
decided_by: OWNER
decided_on: 2026-11-04
decided_on_basis: header of owner-architecture-decisions-v1.md ('AOD-01..19 were recorded from the owner's instruction in chat on 2026-11-04')
recorded_on: 05d7e7e (2026-11-04)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT (explicit owner confirmation message, session 2026-11-09; see DEC-075)
source: docs/decisions/owner-architecture-decisions-v1.md, row AOD-17
source_availability: chat UNAVAILABLE in repository; transcription IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/decisions/owner-architecture-decisions-v1.md]
history:
  - 2026-11-09 entered into the registry as RECORDED; the word 'Approved' in the AOD file is read as RECORDED until countersigned (plan 3.3 Fact)
  - 2026-11-09 RECORDED -> CONFIRMED by the owner's explicit chat confirmation of AOD-01..24 under the conditions of DEC-075
## DEC-021  AOD-18: P0/P1 contracts stay Draft until the owner separately approves them
id: DEC-021
aliases: [AOD-18]
category: ARCHITECTURE
level: TECHNICAL
status: CONFIRMED
statement: >
  P0/P1 contracts stay Draft until the owner separately approves them. Consequence as recorded: No code may claim a P0/P1 contract as Approved or Frozen. Spike implementations are experimental.
decided_by: OWNER
decided_on: 2026-11-04
decided_on_basis: header of owner-architecture-decisions-v1.md ('AOD-01..19 were recorded from the owner's instruction in chat on 2026-11-04')
recorded_on: 05d7e7e (2026-11-04)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT (explicit owner confirmation message, session 2026-11-09; see DEC-075)
source: docs/decisions/owner-architecture-decisions-v1.md, row AOD-18
source_availability: chat UNAVAILABLE in repository; transcription IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/decisions/owner-architecture-decisions-v1.md]
history:
  - 2026-11-09 entered into the registry as RECORDED; the word 'Approved' in the AOD file is read as RECORDED until countersigned (plan 3.3 Fact)
  - 2026-11-09 RECORDED -> CONFIRMED by the owner's explicit chat confirmation of AOD-01..24 under the conditions of DEC-075
## DEC-022  AOD-19: AU-1.0 and ACR-001/002/003 are not rewritten automatically
id: DEC-022
aliases: [AOD-19]
category: ARCHITECTURE
level: TECHNICAL
status: CONFIRMED
statement: >
  AU-1.0 and ACR-001/002/003 are not rewritten automatically. Consequence as recorded: These documents change only by explicit owner action. Where a decision here seems to touch them, the interaction is listed for the owner (section 3), not applied.
decided_by: OWNER
decided_on: 2026-11-04
decided_on_basis: header of owner-architecture-decisions-v1.md ('AOD-01..19 were recorded from the owner's instruction in chat on 2026-11-04')
recorded_on: 05d7e7e (2026-11-04)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT (explicit owner confirmation message, session 2026-11-09; see DEC-075)
source: docs/decisions/owner-architecture-decisions-v1.md, row AOD-19
source_availability: chat UNAVAILABLE in repository; transcription IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/decisions/owner-architecture-decisions-v1.md]
history:
  - 2026-11-09 entered into the registry as RECORDED; the word 'Approved' in the AOD file is read as RECORDED until countersigned (plan 3.3 Fact)
  - 2026-11-09 RECORDED -> CONFIRMED by the owner's explicit chat confirmation of AOD-01..24 under the conditions of DEC-075
## DEC-023  AOD-20: WorldGen to Core: hybrid option C, realised as variant C3. System gene
id: DEC-023
aliases: [AOD-20, OAM-01]
category: ARCHITECTURE
level: TECHNICAL
status: CONFIRMED
statement: >
  WorldGen to Core: hybrid option C, realised as variant C3. System generation goes through a WorldGen/System-commit boundary; user changes only through the Editing API. Details, scope and limits: section 5.1. Consequence as recorded: The server owns the authoritative world; WorldGen never bypasses the public Core/system-commit ports; initial generation is not in ordinary undo/redo; system generation gets an OperationId and takes part in revision/commit semantics.
decided_by: OWNER
decided_on: 2026-11-05
decided_on_basis: header and section 5 of owner-architecture-decisions-v1.md (2026-11-05)
recorded_on: 8f487ef (2026-11-05)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT (explicit owner confirmation message, session 2026-11-09; see DEC-075)
source: docs/decisions/owner-architecture-decisions-v1.md, row AOD-20
source_availability: chat UNAVAILABLE in repository; transcription IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/decisions/owner-architecture-decisions-v1.md]
history:
  - 2026-11-09 entered into the registry as RECORDED; the word 'Approved' in the AOD file is read as RECORDED until countersigned (plan 3.3 Fact)
  - 2026-11-09 RECORDED -> CONFIRMED by the owner's explicit chat confirmation of AOD-01..24 under the conditions of DEC-075
## DEC-024  AOD-21: Persistence: seed + GeneratorVersion for the untouched world, a durabl
id: DEC-024
aliases: [AOD-21, OAM-02]
category: ARCHITECTURE
level: TECHNICAL
status: CONFIRMED
statement: >
  Persistence: seed + GeneratorVersion for the untouched world, a durable mutation journal for user changes, periodic compaction into chunk snapshots. Section 5.2. Consequence as recorded: Load order base generation, then snapshot, then journal tail; snapshots are not the only source of truth; the journal survives sudden process termination; a generator change never silently changes an existing world (pin, or explicit bake/migration).
decided_by: OWNER
decided_on: 2026-11-05
decided_on_basis: header and section 5 of owner-architecture-decisions-v1.md (2026-11-05)
recorded_on: 8f487ef (2026-11-05)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT (explicit owner confirmation message, session 2026-11-09; see DEC-075)
source: docs/decisions/owner-architecture-decisions-v1.md, row AOD-21
source_availability: chat UNAVAILABLE in repository; transcription IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/decisions/owner-architecture-decisions-v1.md]
history:
  - 2026-11-09 entered into the registry as RECORDED; the word 'Approved' in the AOD file is read as RECORDED until countersigned (plan 3.3 Fact)
  - 2026-11-09 RECORDED -> CONFIRMED by the owner's explicit chat confirmation of AOD-01..24 under the conditions of DEC-075
## DEC-025  AOD-22: Embedded and dedicated server run the same authoritative Rust server r
id: DEC-025
aliases: [AOD-22, OAM-03]
category: ARCHITECTURE
level: TECHNICAL
status: CONFIRMED
statement: >
  Embedded and dedicated server run the same authoritative Rust server runtime. Section 5.3. Consequence as recorded: Single-player starts the runtime in the client process; the dedicated server starts the same runtime in its own process; no client-only authoritative mode; transport, persistence and authority boundaries are identical.
decided_by: OWNER
decided_on: 2026-11-05
decided_on_basis: header and section 5 of owner-architecture-decisions-v1.md (2026-11-05)
recorded_on: 8f487ef (2026-11-05)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT (explicit owner confirmation message, session 2026-11-09; see DEC-075)
source: docs/decisions/owner-architecture-decisions-v1.md, row AOD-22
source_availability: chat UNAVAILABLE in repository; transcription IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/decisions/owner-architecture-decisions-v1.md]
history:
  - 2026-11-09 entered into the registry as RECORDED; the word 'Approved' in the AOD file is read as RECORDED until countersigned (plan 3.3 Fact)
  - 2026-11-09 RECORDED -> CONFIRMED by the owner's explicit chat confirmation of AOD-01..24 under the conditions of DEC-075
## DEC-026  AOD-23: Licence and sources policy. Section 5.4
id: DEC-026
aliases: [AOD-23, OAM-07]
category: ARCHITECTURE
level: TECHNICAL
status: CONFIRMED
statement: >
  Licence and sources policy. Section 5.4. Consequence as recorded: Veloren code/assets are not copied; GPL projects are architectural references only; CubeWorld-Reversal is research material only; nothing reverse-engineered is carried over; ideas are re-thought and implemented independently; legally sensitive cases are marked for separate review.
decided_by: OWNER
decided_on: 2026-11-05
decided_on_basis: header and section 5 of owner-architecture-decisions-v1.md (2026-11-05)
recorded_on: 8f487ef (2026-11-05)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT (explicit owner confirmation message, session 2026-11-09; see DEC-075)
source: docs/decisions/owner-architecture-decisions-v1.md, row AOD-23
source_availability: chat UNAVAILABLE in repository; transcription IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/decisions/owner-architecture-decisions-v1.md]
history:
  - 2026-11-09 entered into the registry as RECORDED; the word 'Approved' in the AOD file is read as RECORDED until countersigned (plan 3.3 Fact)
  - 2026-11-09 RECORDED -> CONFIRMED by the owner's explicit chat confirmation of AOD-01..24 under the conditions of DEC-075
## DEC-027  AOD-24: Missing sources: Conditionally Approved. Section 5.5
id: DEC-027
aliases: [AOD-24, OAM-08]
category: ARCHITECTURE
level: TECHNICAL
status: CONFIRMED
statement: >
  Missing sources: Conditionally Approved. Section 5.5. Condition: Conditionally Approved; a temporary baseline never substitutes for the Geometry Processing Specification before PR-01..PR-08 are approved. Consequence as recorded: A temporary owner-approved baseline is allowed for limited parts of Geometry Core, WorldGen and the roadmap; it never substitutes the Geometry Processing Specification before PR-01…PR-08 are approved; missing documents still block final P0/P1 approval.
decided_by: OWNER
decided_on: 2026-11-05
decided_on_basis: header and section 5 of owner-architecture-decisions-v1.md (2026-11-05)
recorded_on: 8f487ef (2026-11-05)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT (explicit owner confirmation message, session 2026-11-09; see DEC-075)
source: docs/decisions/owner-architecture-decisions-v1.md, row AOD-24
source_availability: chat UNAVAILABLE in repository; transcription IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/decisions/owner-architecture-decisions-v1.md]
history:
  - 2026-11-09 entered into the registry as RECORDED; the word 'Approved' in the AOD file is read as RECORDED until countersigned (plan 3.3 Fact)
  - 2026-11-09 RECORDED -> CONFIRMED by the owner's explicit chat confirmation of AOD-01..24 under the conditions of DEC-075
## DEC-028  Rust runtime and Godot first client (spike decision)
id: DEC-028
aliases: [owner-decision-rust-godot, AOD-01..03 (same content)]
category: ARCHITECTURE
level: TECHNICAL
status: RECORDED
statement: >
  Geometry Core, Processing, Editing, Persistence, Server in Rust; first client Godot 4.x via GDExtension; GDScript for UI and simple glue only; targets Windows, Android ARM64, iOS ARM64; C#, C++, Unity, Unreal excluded.
decided_by: OWNER (as transcribed by an agent)
decided_on: 2026-11-02
decided_on_basis: owner-decision-rust-godot.md status line ('chat, 2026-11-02')
recorded_on: 8ac400a (2026-11-02)
confirmed_on: 
confirmation_evidence: NONE (agent transcription)
source: docs/decisions/owner-decision-rust-godot.md
source_availability: chat UNAVAILABLE in repository; transcription IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/decisions/owner-decision-rust-godot.md]
history:
  - 2026-11-09 entry created

## DEC-029  Ratification of AOD-01..24 and OAM-01/02/03/07/08
id: DEC-029
aliases: [X-1 (confirmation batch)]
category: PROCESS
level: TECHNICAL
status: CONFIRMED
statement: >
  Owner statement needed: confirm each of DEC-004..DEC-027 (AOD-01..24) and the Rust+Godot record as written, or list amendments or rejections. Until then they stay RECORDED. GOV-02 evidence lists 'AOD-01..24 countersigned'.
decided_by: OWNER
decided_on: UNKNOWN
decided_on_basis: 
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT (explicit owner confirmation message, session 2026-11-09)
source: docs/archive/gov01/GOV-01-DECISION-REGISTER.md item X-1; docs/production/ps-0-exit-checklist.md section 1 row 2
source_availability: IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/production/ps-0-exit-checklist.md]
history:
  - 2026-11-09 entry created
  - 2026-11-09 done: the owner confirmed AOD-01..24 in chat (DEC-004..DEC-027, conditions in DEC-075); no further ratification step
## DEC-030  Built-in content as package `cube`; shared registration; controlled extension
id: DEC-030
aliases: [OCC-02 (part), CPR-CON-01, CPR-CON-02, CPR-CON-11, S-3, S-4, S-8 (concept)]
category: PRODUCT
level: CONCEPT
status: CONFIRMED
statement: >
  Встроенный контент оформляется как пакет `cube`. Встроенный и сторонний контент используют совместимые механизмы регистрации. Моды могут расширять контент и игровые системы только через контролируемые интерфейсы.
not_decided:
  - loader implementation
  - package file format and schema
  - public API
  - crate layout
  - implementation stage
decided_by: OWNER
decided_on: UNKNOWN
decided_on_basis: the owner's message states these were confirmed 'in the discussion' without naming the date
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT
source: owner message of 2026-11-09, previous stage 'Product Vision Consolidation', section 2 (bullets quoted); also in the user message that started the session
source_availability: UNAVAILABLE in repository (session transcript only); quoted text IN_REPOSITORY in docs/product-vision
owner: Project Owner
affected_documents: [docs/product-vision/cube-product-vision.md]
history:
  - 2026-11-09 entry created

## DEC-031  Persistent identity `namespace:name` and runtime numeric IDs
id: DEC-031
aliases: [OCC-02 (part), CPR-CON-03..06, R-02, R-03]
category: PRODUCT
level: CONCEPT
status: CONFIRMED
statement: >
  Постоянная идентичность объекта — `namespace:name`. В runtime используются числовые ID и компактные представления. Точная ширина числовых ID пока не утверждена. Разные namespace могут содержать одинаковые локальные имена. Повторный полный ID обрабатывается как диагностируемая ошибка, а не crash.
not_decided:
  - numeric ID width (explicitly not approved)
  - name grammar, case, length
  - mapping-table algorithm
decided_by: OWNER
decided_on: UNKNOWN
decided_on_basis: the owner's message states these were confirmed 'in the discussion' without naming the date
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT
source: owner message of 2026-11-09, previous stage 'Product Vision Consolidation', section 2 (bullets quoted); also in the user message that started the session
source_availability: UNAVAILABLE in repository (session transcript only); quoted text IN_REPOSITORY in docs/product-vision
owner: Project Owner
affected_documents: [docs/product-vision/cube-product-vision.md]
history:
  - 2026-11-09 entry created

## DEC-032  Package dependencies, load order, conflict diagnostics, logging
id: DEC-032
aliases: [OCC-02 (part), CPR-CON-07..10]
category: PRODUCT
level: CONCEPT
status: CONFIRMED
statement: >
  Пакеты объявляют зависимости и совместимые версии. Порядок загрузки вычисляется автоматически. Конфликты диагностируются, независимые пакеты по возможности продолжают работать. Требуется логирование ошибок, зависимостей и конфликтов в файлы логов игры.
not_decided:
  - version scheme
  - ordering algorithm
  - log format and location
decided_by: OWNER
decided_on: UNKNOWN
decided_on_basis: the owner's message states these were confirmed 'in the discussion' without naming the date
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT
source: owner message of 2026-11-09, previous stage 'Product Vision Consolidation', section 2 (bullets quoted); also in the user message that started the session
source_availability: UNAVAILABLE in repository (session transcript only); quoted text IN_REPOSITORY in docs/product-vision
owner: Project Owner
affected_documents: [docs/product-vision/cube-product-vision.md]
history:
  - 2026-11-09 entry created

## DEC-033  Package-aware world save: manifest and ID tables
id: DEC-033
aliases: [OCC-03 (part), CPR-SAV-01..03, R-10, R-11]
category: PRODUCT
level: CONCEPT
status: CONFIRMED
statement: >
  Формат M2 должен изначально предусматривать пакетную модель. Сохранение содержит manifest необходимых пакетов и версий. Постоянные ID связываются с компактными ID посредством таблиц соответствия.
not_decided:
  - manifest fields
  - binary format
  - which milestone 'M2' denotes (M2 is still defined differently in DEVELOPMENT_ROADMAP, Game Track and M2-T; read here as 'the first save format that is approved')
decided_by: OWNER
decided_on: UNKNOWN
decided_on_basis: the owner's message states these were confirmed 'in the discussion' without naming the date
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT
source: owner message of 2026-11-09, previous stage 'Product Vision Consolidation', section 2 (bullets quoted); also in the user message that started the session
source_availability: UNAVAILABLE in repository (session transcript only); quoted text IN_REPOSITORY in docs/product-vision
owner: Project Owner
affected_documents: [docs/product-vision/cube-product-vision.md, docs/api-contracts/world-persistence-contracts.md]
history:
  - 2026-11-09 entry created

## DEC-034  Load blocking, recovery mode, preservation of unknown content
id: DEC-034
aliases: [OCC-03 (part), CPR-SAV-04..06, CPR-SAV-09, R-13, R-14]
category: PRODUCT
level: CONCEPT
status: CONFIRMED
statement: >
  Обычная загрузка блокируется при отсутствии обязательных зависимостей. Отдельный режим восстановления допускает заглушки неизвестных объектов. Исходные данные неизвестного содержимого должны сохраняться. Восстановление и сохранение не должны приводить к необратимой потере содержимого отсутствующих модов.
not_decided:
  - recovery policy details
  - payload format
  - backup rules
  - who may declare id replacements
decided_by: OWNER
decided_on: UNKNOWN
decided_on_basis: the owner's message states these were confirmed 'in the discussion' without naming the date
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT
source: owner message of 2026-11-09, previous stage 'Product Vision Consolidation', section 2 (bullets quoted); also in the user message that started the session
source_availability: UNAVAILABLE in repository (session transcript only); quoted text IN_REPOSITORY in docs/product-vision
owner: Project Owner
affected_documents: [docs/product-vision/cube-product-vision.md]
history:
  - 2026-11-09 entry created

## DEC-035  Package version compatibility
id: DEC-035
aliases: [OCC-03 (part), CPR-SAV-07, CPR-SAV-08]
category: PRODUCT
level: CONCEPT
status: CONFIRMED
statement: >
  Совместимые версии пакетов могут использоваться вместо точных версий. Нельзя без проверки считать любую новую версию совместимой.
not_decided:
  - compatibility range syntax
  - migration rules
decided_by: OWNER
decided_on: UNKNOWN
decided_on_basis: the owner's message states these were confirmed 'in the discussion' without naming the date
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT
source: owner message of 2026-11-09, previous stage 'Product Vision Consolidation', section 2 (bullets quoted); also in the user message that started the session
source_availability: UNAVAILABLE in repository (session transcript only); quoted text IN_REPOSITORY in docs/product-vision
owner: Project Owner
affected_documents: [docs/product-vision/cube-product-vision.md]
history:
  - 2026-11-09 entry created

## DEC-036  Free world modification; voxels and micro-voxels; fixed base detail levels; smooth surfaces not required
id: DEC-036
aliases: [OCC-04 (part), CPR-BLD-01..04]
category: PRODUCT
level: CONCEPT
status: CONFIRMED
statement: >
  Полноценное свободное изменение мира — одна из основных возможностей Cube. Обычные воксели и микровоксели поддерживаются совместно. Детализация имеет фиксированные базовые уровни с возможностью расширения. Плавные поверхности не являются требованием строительной системы.
not_decided:
  - number and values of levels
  - key structure
  - which geometry backends are built (SDF/Mesh scope is open)
  - micro-voxel transition policy
decided_by: OWNER
decided_on: UNKNOWN
decided_on_basis: the owner's message states these were confirmed 'in the discussion' without naming the date
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT
source: owner message of 2026-11-09, previous stage 'Product Vision Consolidation', section 2 (bullets quoted); also in the user message that started the session
source_availability: UNAVAILABLE in repository (session transcript only); quoted text IN_REPOSITORY in docs/product-vision
owner: Project Owner
affected_documents: [docs/product-vision/cube-product-vision.md]
history:
  - 2026-11-09 entry created

## DEC-037  Building tools; Creative and RPG share a technology base
id: DEC-037
aliases: [OCC-04 (part), CPR-BLD-05, CPR-BLD-06]
category: PRODUCT
level: CONCEPT
status: CONFIRMED
statement: >
  Поддерживаются ручное строительство и инструменты проектирования. Creative и RPG используют общую технологическую основу.
not_decided:
  - tool list
decided_by: OWNER
decided_on: UNKNOWN
decided_on_basis: the owner's message states these were confirmed 'in the discussion' without naming the date
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT
source: owner message of 2026-11-09, previous stage 'Product Vision Consolidation', section 2 (bullets quoted); also in the user message that started the session
source_availability: UNAVAILABLE in repository (session transcript only); quoted text IN_REPOSITORY in docs/product-vision
owner: Project Owner
affected_documents: [docs/product-vision/cube-product-vision.md]
history:
  - 2026-11-09 entry created

## DEC-038  Persistence of builds; permanent and temporary world changes
id: DEC-038
aliases: [OCC-04 (part), CPR-BLD-07, CPR-BLD-08]
category: PRODUCT
level: CONCEPT
status: CONFIRMED
statement: >
  Постройки, изменения рельефа и разрушения сохраняются. Поддерживаются постоянные и временные изменения мира.
not_decided:
  - semantics of temporary changes
  - storage representation
decided_by: OWNER
decided_on: UNKNOWN
decided_on_basis: the owner's message states these were confirmed 'in the discussion' without naming the date
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT
source: owner message of 2026-11-09, previous stage 'Product Vision Consolidation', section 2 (bullets quoted); also in the user message that started the session
source_availability: UNAVAILABLE in repository (session transcript only); quoted text IN_REPOSITORY in docs/product-vision
owner: Project Owner
affected_documents: [docs/product-vision/cube-product-vision.md]
history:
  - 2026-11-09 entry created

## DEC-039  Blueprints and transfer between worlds
id: DEC-039
aliases: [OCC-04 (part), CPR-BLD-09]
category: PRODUCT
level: CONCEPT
status: CONFIRMED
statement: >
  Поддерживаются чертежи и их перенос между мирами.
not_decided:
  - blueprint format
  - owner of Import/Export
decided_by: OWNER
decided_on: UNKNOWN
decided_on_basis: the owner's message states these were confirmed 'in the discussion' without naming the date
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT
source: owner message of 2026-11-09, previous stage 'Product Vision Consolidation', section 2 (bullets quoted); also in the user message that started the session
source_availability: UNAVAILABLE in repository (session transcript only); quoted text IN_REPOSITORY in docs/product-vision
owner: Project Owner
affected_documents: [docs/product-vision/cube-product-vision.md]
history:
  - 2026-11-09 entry created

## DEC-040  Collaborative building, territory rights, Undo/Redo, server history, snapshots
id: DEC-040
aliases: [OCC-04 (part), CPR-BLD-10]
category: PRODUCT
level: CONCEPT
status: CONFIRMED
statement: >
  Поддерживаются совместное строительство, права территорий, Undo/Redo, серверная история и snapshots.
not_decided:
  - rights model
  - history storage
decided_by: OWNER
decided_on: UNKNOWN
decided_on_basis: the owner's message states these were confirmed 'in the discussion' without naming the date
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT
source: owner message of 2026-11-09, previous stage 'Product Vision Consolidation', section 2 (bullets quoted); also in the user message that started the session
source_availability: UNAVAILABLE in repository (session transcript only); quoted text IN_REPOSITORY in docs/product-vision
owner: Project Owner
affected_documents: [docs/product-vision/cube-product-vision.md]
history:
  - 2026-11-09 entry created

## DEC-041  No arbitrary global structure size limit; optional admin limits; mandatory protections remain
id: DEC-041
aliases: [OCC-04 (part), CPR-BLD-11, CPR-BLD-12]
category: PRODUCT
level: CONCEPT
status: CONFIRMED
statement: >
  Архитектура не вводит произвольный глобальный предел размера сооружений. Административные лимиты опциональны; обязательные меры защиты вычислительных ресурсов и данных сохраняются.
not_decided:
  - which protections and their numbers
  - world size bound (separate: D-K)
decided_by: OWNER
decided_on: UNKNOWN
decided_on_basis: the owner's message states these were confirmed 'in the discussion' without naming the date
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT
source: owner message of 2026-11-09, previous stage 'Product Vision Consolidation', section 2 (bullets quoted); also in the user message that started the session
source_availability: UNAVAILABLE in repository (session transcript only); quoted text IN_REPOSITORY in docs/product-vision
owner: Project Owner
affected_documents: [docs/product-vision/cube-product-vision.md]
history:
  - 2026-11-09 entry created

## DEC-042  Hybrid structure physics; movable voxel constructions
id: DEC-042
aliases: [OCC-04 (part), CPR-BLD-13, CPR-BLD-14]
category: PRODUCT
level: CONCEPT
status: CONFIRMED
statement: >
  Физика построек гибридная: статичные конструкции по умолчанию, отдельные динамические конструкции и материалы. Предусматриваются подвижные воксельные конструкции.
not_decided:
  - physics module and owner
  - motion model
  - implementation stage
decided_by: OWNER
decided_on: UNKNOWN
decided_on_basis: the owner's message states these were confirmed 'in the discussion' without naming the date
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT
source: owner message of 2026-11-09, previous stage 'Product Vision Consolidation', section 2 (bullets quoted); also in the user message that started the session
source_availability: UNAVAILABLE in repository (session transcript only); quoted text IN_REPOSITORY in docs/product-vision
owner: Project Owner
affected_documents: [docs/product-vision/cube-product-vision.md]
history:
  - 2026-11-09 entry created

## DEC-043  Resource renewal, no automatic decay of buildings, natural change and ageing
id: DEC-043
aliases: [OCC-05 (part), CPR-SIM-01..03]
category: PRODUCT
level: CONCEPT
status: CONFIRMED
statement: >
  Возобновление природных ресурсов зависит от их типа и правил мира. Постройки не разрушаются автоматически от времени по умолчанию. Природные изменения и старение могут быть разрешены отдельно.
not_decided:
  - rules
decided_by: OWNER
decided_on: UNKNOWN
decided_on_basis: the owner's message states these were confirmed 'in the discussion' without naming the date
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT
source: owner message of 2026-11-09, previous stage 'Product Vision Consolidation', section 2 (bullets quoted); also in the user message that started the session
source_availability: UNAVAILABLE in repository (session transcript only); quoted text IN_REPOSITORY in docs/product-vision
owner: Project Owner
affected_documents: [docs/product-vision/cube-product-vision.md]
history:
  - 2026-11-09 entry created

## DEC-044  Active-region and unloaded-region simulation; configurable background rules
id: DEC-044
aliases: [OCC-05 (part), CPR-SIM-04..07]
category: PRODUCT
level: CONCEPT
status: CONFIRMED
statement: >
  Активные регионы используют подробную симуляцию. Незагруженные регионы могут использовать упрощённый расчёт прогресса. Фоновое изменение воксельного мира разрешается только при возможности корректной и безопасной обработки. Правила фоновой симуляции должны настраиваться.
not_decided:
  - catch-up model
  - rule set
decided_by: OWNER
decided_on: UNKNOWN
decided_on_basis: the owner's message states these were confirmed 'in the discussion' without naming the date
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT
source: owner message of 2026-11-09, previous stage 'Product Vision Consolidation', section 2 (bullets quoted); also in the user message that started the session
source_availability: UNAVAILABLE in repository (session transcript only); quoted text IN_REPOSITORY in docs/product-vision
owner: Project Owner
affected_documents: [docs/product-vision/cube-product-vision.md]
history:
  - 2026-11-09 entry created

## DEC-045  Mechanisms and automation (concept; implementation deferred)
id: DEC-045
aliases: [OCC-06, CPR-MEC-01..08]
category: PRODUCT
level: CONCEPT
status: CONFIRMED
statement: >
  Поддерживаются простые интерактивные механизмы. Архитектура допускает инженерные схемы, сигналы и автоматизацию. Энергетическая модель гибридная. В перспективе поддерживаются программируемые устройства. Предусматривается полноценная автоматизация добычи и производства. Моды могут добавлять новые устройства и системы через контролируемые API. Разные системы модов могут взаимодействовать посредством общих контрактов и адаптеров. Реализация сложной инженерии, runtime и Mod SDK откладывается.
not_decided:
  - everything technical
decided_by: OWNER
decided_on: UNKNOWN
decided_on_basis: the owner's message states these were confirmed 'in the discussion' without naming the date
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT
source: owner message of 2026-11-09, previous stage 'Product Vision Consolidation', section 2 (bullets quoted); also in the user message that started the session
source_availability: UNAVAILABLE in repository (session transcript only); quoted text IN_REPOSITORY in docs/product-vision
owner: Project Owner
affected_documents: [docs/product-vision/cube-product-vision.md]
history:
  - 2026-11-09 entry created

## DEC-046  Deep multiclass and build management
id: DEC-046
aliases: [OCC-07 (part), CPR-RPG-01..03]
category: PRODUCT
level: CONCEPT
status: CONFIRMED
statement: >
  Поддерживается глубокая мультиклассовость и смешивание классов. Архитектура не устанавливает обязательного глобального ограничения количества классов. Игрок может сохранять сборки и перераспределять развитие.
not_decided:
  - class model
  - whether 'no limit' means a design cap or arbitrary numeric precision
decided_by: OWNER
decided_on: UNKNOWN
decided_on_basis: the owner's message states these were confirmed 'in the discussion' without naming the date
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT
source: owner message of 2026-11-09, previous stage 'Product Vision Consolidation', section 2 (bullets quoted); also in the user message that started the session
source_availability: UNAVAILABLE in repository (session transcript only); quoted text IN_REPOSITORY in docs/product-vision
owner: Project Owner
affected_documents: [docs/product-vision/cube-product-vision.md]
history:
  - 2026-11-09 entry created

## DEC-047  Progression model
id: DEC-047
aliases: [OCC-07 (part), CPR-RPG-04..07]
category: PRODUCT
level: CONCEPT
status: CONFIRMED
statement: >
  Прогрессия сочетает уровни, очки, мастерство использования, обучение и исследования. Развитие потенциально бесконечно. Поддерживается вертикальный и горизонтальный рост. Правила роста силы, убывающей отдачи и ограничений настраиваются.
not_decided:
  - numeric types and ranges
  - formulas
  - whether 'potentially infinite' means unbounded numeric precision
decided_by: OWNER
decided_on: UNKNOWN
decided_on_basis: the owner's message states these were confirmed 'in the discussion' without naming the date
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT
source: owner message of 2026-11-09, previous stage 'Product Vision Consolidation', section 2 (bullets quoted); also in the user message that started the session
source_availability: UNAVAILABLE in repository (session transcript only); quoted text IN_REPOSITORY in docs/product-vision
owner: Project Owner
affected_documents: [docs/product-vision/cube-product-vision.md]
history:
  - 2026-11-09 entry created

## DEC-048  World difficulty and power scaling
id: DEC-048
aliases: [OCC-07 (part), CPR-RPG-08..10]
category: PRODUCT
level: CONCEPT
status: CONFIRMED
statement: >
  Сложность мира гибридная, с полноценной поддержкой динамического масштабирования. Масштабирование не должно автоматически уравнивать игроков с огромной разницей в развитии. Power Leveling допускается.
not_decided:
  - algorithm
decided_by: OWNER
decided_on: UNKNOWN
decided_on_basis: the owner's message states these were confirmed 'in the discussion' without naming the date
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT
source: owner message of 2026-11-09, previous stage 'Product Vision Consolidation', section 2 (bullets quoted); also in the user message that started the session
source_availability: UNAVAILABLE in repository (session transcript only); quoted text IN_REPOSITORY in docs/product-vision
owner: Project Owner
affected_documents: [docs/product-vision/cube-product-vision.md]
history:
  - 2026-11-09 entry created

## DEC-049  Group rewards
id: DEC-049
aliases: [OCC-07 (part), CPR-RPG-11]
category: PRODUCT
level: CONCEPT
status: CONFIRMED
statement: >
  Групповые награды учитывают разные виды участия.
not_decided:
  - model
decided_by: OWNER
decided_on: UNKNOWN
decided_on_basis: the owner's message states these were confirmed 'in the discussion' without naming the date
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT
source: owner message of 2026-11-09, previous stage 'Product Vision Consolidation', section 2 (bullets quoted); also in the user message that started the session
source_availability: UNAVAILABLE in repository (session transcript only); quoted text IN_REPOSITORY in docs/product-vision
owner: Project Owner
affected_documents: [docs/product-vision/cube-product-vision.md]
history:
  - 2026-11-09 entry created

## DEC-050  Server rules, characters, separate state of independent servers, trusted-server sharing
id: DEC-050
aliases: [OCC-08 (part), CPR-MP-01..04]
category: PRODUCT
level: CONCEPT
status: CONFIRMED
statement: >
  Миры и серверы имеют настраиваемые правила. Поддерживаются несколько персонажей и настраиваемая прогрессия. Независимые серверы по умолчанию имеют отдельные состояния персонажей и экономики. Доверенные серверы потенциально могут поддерживать общий прогресс или контролируемый перенос.
not_decided:
  - account model
  - authority model of shared progress (tension with AOD-13 to be resolved when designed)
decided_by: OWNER
decided_on: UNKNOWN
decided_on_basis: the owner's message states these were confirmed 'in the discussion' without naming the date
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT
source: owner message of 2026-11-09, previous stage 'Product Vision Consolidation', section 2 (bullets quoted); also in the user message that started the session
source_availability: UNAVAILABLE in repository (session transcript only); quoted text IN_REPOSITORY in docs/product-vision
owner: Project Owner
affected_documents: [docs/product-vision/cube-product-vision.md]
history:
  - 2026-11-09 entry created

## DEC-051  Trade, loot distribution, item transfer, no duplication or loss
id: DEC-051
aliases: [OCC-08 (part), CPR-MP-05..08]
category: PRODUCT
level: CONCEPT
status: CONFIRMED
statement: >
  Предусмотрены прямой обмен, магазины игроков, рынки и аукционы. Поддерживаются разные модели распределения добычи. Передача предметов может быть свободной или ограниченной свойствами предметов. Передачи и торговые транзакции не должны допускать дублирования или потери имущества.
not_decided:
  - economy model
  - transaction mechanism
decided_by: OWNER
decided_on: UNKNOWN
decided_on_basis: the owner's message states these were confirmed 'in the discussion' without naming the date
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT
source: owner message of 2026-11-09, previous stage 'Product Vision Consolidation', section 2 (bullets quoted); also in the user message that started the session
source_availability: UNAVAILABLE in repository (session transcript only); quoted text IN_REPOSITORY in docs/product-vision
owner: Project Owner
affected_documents: [docs/product-vision/cube-product-vision.md]
history:
  - 2026-11-09 entry created

## DEC-052  Product principles stated as context (PRN-05..08)
id: DEC-052
aliases: [OCC-01 (part), CPR-PRN-05..08]
category: PRODUCT
level: CONCEPT
status: RECORDED
statement: >
  Продукт — Cube, а не универсальный игровой движок. Technology Track и Game Track развиваются согласованно и сходятся в вертикальных срезах. Полноценную поддержку модов и сложные игровые подсистемы можно отложить, но их ключевые архитектурные требования учитываются заранее. Экспериментальный P-WORLD не является production-архитектурой и не должен автоматически переноситься в неё.
not_decided:
  - any technical consequence
decided_by: OWNER
decided_on: UNKNOWN
decided_on_basis: stated in the 'Исходный контекст' section of the owner's message as project principles, not in the list of decisions confirmed in discussion (status lowered from the earlier overclaim in cube-product-vision.md)
recorded_on: 2026-11-09 (this registry)
confirmed_on: 
confirmation_evidence: NONE
source: owner message of 2026-11-09, previous stage 'Product Vision Consolidation', section 2 (bullets quoted); also in the user message that started the session
source_availability: UNAVAILABLE in repository (session transcript only); quoted text IN_REPOSITORY in docs/product-vision
owner: Project Owner
affected_documents: [docs/product-vision/cube-product-vision.md]
history:
  - 2026-11-09 entry created

## DEC-053  Extensibility principle (S-1..S-7 of the extensibility draft)
id: DEC-053
aliases: [AOD-25 (proposed id, not assigned), S-1..S-7]
category: PRODUCT
level: CONCEPT
status: RECORDED
statement: >
  Cube is built first; a full modding ecosystem (tools, SDK, scripting API, WASM runtime, distribution platform) comes later. The architecture is designed so a full modding system can be added later without a fundamental rework of the core. Built-in mechanics and future mods use compatible extension mechanisms. Not every internal interface becomes public. Execution technology for mod code is not chosen; WASM is a candidate only. Three conceptual extension kinds: Content Packs, Game Extensions, Runtime Plugins (classification only).
not_decided:
  - the mechanism for any of the three kinds
  - code-execution technology
  - extension API surface
decided_by: OWNER (as transcribed by an agent)
decided_on: 2026-11-09
decided_on_basis: draft text states 'confirmed in chat on 2026-11-09'
recorded_on: 2026-11-09 (this registry)
confirmed_on: 
confirmation_evidence: NONE
source: docs/decisions/owner-decision-extensibility-v1.md section 1 (S-1..S-7 marked CONFIRMED there by the agent)
source_availability: chat UNAVAILABLE in repository; draft IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/decisions/owner-decision-extensibility-v1.md]
history:
  - 2026-11-09 entered as RECORDED; AOD-25/26 are not assigned as bundles (see historical-decisions-reconciliation.md section 4)

## DEC-054  Extensibility and save-format technical proposals (R-01..R-17 of the draft)
id: DEC-054
aliases: [AOD-26 (proposed id, not assigned), R-01..R-17]
category: ARCHITECTURE
level: TECHNICAL
status: PROPOSED
statement: >
  Agent proposals from owner-decision-extensibility-v1.md: single registry model, grammar, per-world id map, extension API manifest, pack manifest, deterministic load order, no-silent-loss placeholders, recovery with backup, migrations, generation identity including packs. None is a confirmed decision; each needs its own technical decision when the corresponding stage starts. The concept parts are covered by the CONCEPT entries above.
decided_by: PENDING
decided_on: UNKNOWN
decided_on_basis: 
recorded_on: 2026-11-09 (this registry)
confirmed_on: 
confirmation_evidence: NONE
source: docs/decisions/owner-decision-extensibility-v1.md sections 2-5
source_availability: IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/decisions/owner-decision-extensibility-v1.md]
history:
  - 2026-11-09 entry created

## DEC-055  ACR-001: foundation types live in Core
id: DEC-055
aliases: [GOV-01#1, ACR-001]
category: ARCHITECTURE
level: TECHNICAL
status: CONFIRMED
statement: >
  Option A. Revision, operation-id, dirty-region and error foundation types (C-16..C-19) are owned by cube-core. No cube-foundation crate.
  Consequences: cube-core stays dependency-free and exposes these types; Processing and Editing import them from Core.
  Related contracts: C-16, C-17, C-18, C-19
  Reversibility: moderate (moving types later costs a crate split)
decided_by: AGENT_DELEGATED
delegation_basis: DEC-001 (owner instruction 2026-11-09, sections 2 and 6): reversible engineering decision inside the confirmed concept
decided_on: 2026-11-09
decided_on_basis: date of this registry commit (agent decision under delegation)
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-DELEGATION (decision by the agent under DEC-001; the owner may reject or amend any entry)
source: docs/archive/gov01/GOV-01-TECHNICAL-RECOMMENDATIONS.md row 1; docs/archive/gov01/GOV-01-DECISION-REGISTER.md item 1
source_availability: IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/production/architecture-blocker-decisions.md]
history:
  - 2026-11-09 entry created

## DEC-056  ACR-002: orchestration reaches backends only through L2 ports
id: DEC-056
aliases: [GOV-01#2, ACR-002]
category: ARCHITECTURE
level: TECHNICAL
status: CONFIRMED
statement: >
  Option A. Ports live in cube-editing-api (L2); Orchestration and backends do not depend on each other directly.
  Consequences: No Orchestration->backend edge in the module grid; archcheck encodes the L2 edges.
  Related contracts: E-series, module-boundaries Part B
decided_by: AGENT_DELEGATED
delegation_basis: DEC-001 (owner instruction 2026-11-09, sections 2 and 6): reversible engineering decision inside the confirmed concept
decided_on: 2026-11-09
decided_on_basis: date of this registry commit (agent decision under delegation)
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-DELEGATION (decision by the agent under DEC-001; the owner may reject or amend any entry)
source: docs/archive/gov01/GOV-01-TECHNICAL-RECOMMENDATIONS.md row 2; docs/archive/gov01/GOV-01-DECISION-REGISTER.md item 2
source_availability: IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/production/architecture-blocker-decisions.md]
history:
  - 2026-11-09 entry created

## DEC-057  ACR-003: composite data model in Core, operations in Composite
id: DEC-057
aliases: [GOV-01#3, ACR-003]
category: ARCHITECTURE
level: TECHNICAL
status: CONFIRMED
statement: >
  Option A. Passive composite data (part, assembly graph) is in cube-core; operations are in cube-composite. Supports movable constructions without a Persistence->Composite edge.
  Consequences: Core gets passive data types only at PS-8/CORE-14; no behaviour in Core.
  Related contracts: C-13, C-14, C-15
  Reversibility: cheap before CORE-14
decided_by: AGENT_DELEGATED
delegation_basis: DEC-001 (owner instruction 2026-11-09, sections 2 and 6): reversible engineering decision inside the confirmed concept
decided_on: 2026-11-09
decided_on_basis: date of this registry commit (agent decision under delegation)
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-DELEGATION (decision by the agent under DEC-001; the owner may reject or amend any entry)
source: docs/archive/gov01/GOV-01-TECHNICAL-RECOMMENDATIONS.md row 3; docs/archive/gov01/GOV-01-DECISION-REGISTER.md item 3
source_availability: IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/production/architecture-blocker-decisions.md]
history:
  - 2026-11-09 entry created

## DEC-058  ACR-004 A: Core prerequisites for a system-commit boundary
id: DEC-058
aliases: [GOV-01#4, ACR-004 part A]
category: ARCHITECTURE
level: TECHNICAL
status: CONFIRMED
statement: >
  Approve as the structural form of AOD-20 (RECORDED): capability handles UserCommit, SystemCommit, RecoveryCommit; ActorKind and OperationId origin on every commit. Handle ABI and system-commit port signatures are not decided here.
  Consequences: Core exposes handle-based commit entry classes (MB-1); parts B, C, E stay for GOV-11.
  Related contracts: C-16, C-17, SC-01, SC-02, SC-05, SC-10
decided_by: AGENT_DELEGATED
delegation_basis: DEC-001 (owner instruction 2026-11-09, sections 2 and 6): reversible engineering decision inside the confirmed concept
decided_on: 2026-11-09
decided_on_basis: date of this registry commit (agent decision under delegation)
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-DELEGATION (decision by the agent under DEC-001; the owner may reject or amend any entry)
source: docs/archive/gov01/GOV-01-TECHNICAL-RECOMMENDATIONS.md row 4; docs/archive/gov01/GOV-01-DECISION-REGISTER.md item 4
source_availability: IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/decisions/acr-004-owner-review.md]
history:
  - 2026-11-09 entry created

## DEC-059  Revision model: R-A with an extensible actor set
id: DEC-059
aliases: [GOV-01#5, OAM-10 / ACR-004 part D, OAM-09, OAM-10]
category: ARCHITECTURE
level: TECHNICAL
status: CONFIRMED
statement: >
  R-A: the same asset and layer revision scopes, ActorKind::System, no separate generation epoch. ActorKind is a non-exhaustive, persisted enum with a reserved value for world-simulation originated changes (renewal, collapse, mechanisms); its policy (history, permissions) is not decided here. OAM-09 stays: C1 first baseline, C2 deferred.
  Consequences: Journal and replication filter by ActorKind; adding a variant later is a format-versioned addition, not a rewrite. Addresses the missing entry class for simulation-originated changes found in production-plan-alignment-report.md R-8.
  Related contracts: C-16, C-17, OAM-09, OAM-10
decided_by: AGENT_DELEGATED
delegation_basis: DEC-001 (owner instruction 2026-11-09, sections 2 and 6): reversible engineering decision inside the confirmed concept
decided_on: 2026-11-09
decided_on_basis: date of this registry commit (agent decision under delegation)
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-DELEGATION (decision by the agent under DEC-001; the owner may reject or amend any entry)
source: docs/archive/gov01/GOV-01-TECHNICAL-RECOMMENDATIONS.md row 5; docs/archive/gov01/GOV-01-DECISION-REGISTER.md item 5
source_availability: IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/production/architecture-blocker-decisions.md]
history:
  - 2026-11-09 entry created

## DEC-060  OD-R1: RevisionToken width
id: DEC-060
aliases: [GOV-01#6, OD-R1]
category: ARCHITECTURE
level: TECHNICAL
status: CONFIRMED
statement: >
  `u64` revision value and `u32` epoch. The persisted width is a versioned constant independent of the in-memory type.
  Consequences: No rollover handling needed in practice.
  Related contracts: C-16
decided_by: AGENT_DELEGATED
delegation_basis: DEC-001 (owner instruction 2026-11-09, sections 2 and 6): reversible engineering decision inside the confirmed concept
decided_on: 2026-11-09
decided_on_basis: date of this registry commit (agent decision under delegation)
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-DELEGATION (decision by the agent under DEC-001; the owner may reject or amend any entry)
source: docs/archive/gov01/GOV-01-TECHNICAL-RECOMMENDATIONS.md row 6; docs/archive/gov01/GOV-01-DECISION-REGISTER.md item 6
source_availability: IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/production/architecture-blocker-decisions.md]
history:
  - 2026-11-09 entry created

## DEC-061  OD-R2: commit algorithm
id: DEC-061
aliases: [GOV-01#7, OD-R2]
category: ARCHITECTURE
level: TECHNICAL
status: CONFIRMED
statement: >
  Option A: immutable Arc state per asset, a per-asset commit gate serialising writers, lock-free snapshot readers; validate-then-apply is one atomic step across bricks.
  Consequences: Fine-grained gating can be added behind the same API.
  Related contracts: C-01, C-16
decided_by: AGENT_DELEGATED
delegation_basis: DEC-001 (owner instruction 2026-11-09, sections 2 and 6): reversible engineering decision inside the confirmed concept
decided_on: 2026-11-09
decided_on_basis: date of this registry commit (agent decision under delegation)
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-DELEGATION (decision by the agent under DEC-001; the owner may reject or amend any entry)
source: docs/archive/gov01/GOV-01-TECHNICAL-RECOMMENDATIONS.md row 7; docs/archive/gov01/GOV-01-DECISION-REGISTER.md item 7
source_availability: IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/production/architecture-blocker-decisions.md]
history:
  - 2026-11-09 entry created

## DEC-062  OD-R6: RegionStamp
id: DEC-062
aliases: [GOV-01#8, OD-R6]
category: ARCHITECTURE
level: TECHNICAL
status: CONFIRMED
statement: >
  Abstract contract: a versioned, canonical, deterministic 128-bit summary over sorted (CellKey, revision) pairs plus count; it carries an algorithm version; a stamp from another algorithm version is never read as 'unchanged'. Collision policy P-3: authoritative uses verify exactly against per-cell revisions on equality; the stamp alone is a cache hint. Algorithm v1: a dependency-free in-repo 128-bit mixing function (no external hash crate in Core). It is provisional and replaceable because of the algorithm version. The content_hash and persistence-checksum algorithms are separate and not decided here.
  Consequences: Core keeps zero external dependencies. Replacing the algorithm is a versioned change.
  Related contracts: C-16
  Reversibility: cheap (versioned)
decided_by: AGENT_DELEGATED
delegation_basis: DEC-001 (owner instruction 2026-11-09, sections 2 and 6): reversible engineering decision inside the confirmed concept
decided_on: 2026-11-09
decided_on_basis: date of this registry commit (agent decision under delegation)
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-DELEGATION (decision by the agent under DEC-001; the owner may reject or amend any entry)
source: docs/archive/gov01/GOV-01-TECHNICAL-RECOMMENDATIONS.md row 8; docs/archive/gov01/GOV-01-DECISION-REGISTER.md item 8
source_availability: IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/production/architecture-blocker-decisions.md]
history:
  - 2026-11-09 entry created

## DEC-063  D-R8: OperationId
id: DEC-063
aliases: [GOV-01#9, D-R8]
category: ARCHITECTURE
level: TECHNICAL
status: CONFIRMED
statement: >
  Composite (scope u64, sequence u64) with ActorKind and a schema version stored separately; ids are assigned by the authoritative side, never by clients. The idempotent-retry dedupe window is an operational parameter and is left to the server runtime (configurable).
  Consequences: Deterministic, ordered per actor, journal-replay friendly.
  Related contracts: C-17
decided_by: AGENT_DELEGATED
delegation_basis: DEC-001 (owner instruction 2026-11-09, sections 2 and 6): reversible engineering decision inside the confirmed concept
decided_on: 2026-11-09
decided_on_basis: date of this registry commit (agent decision under delegation)
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-DELEGATION (decision by the agent under DEC-001; the owner may reject or amend any entry)
source: docs/archive/gov01/GOV-01-TECHNICAL-RECOMMENDATIONS.md row 9; docs/archive/gov01/GOV-01-DECISION-REGISTER.md item 9
source_availability: IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/production/architecture-blocker-decisions.md]
history:
  - 2026-11-09 entry created

## DEC-064  D-K: CellKey, VoxelPos, world bound, level direction
id: DEC-064
aliases: [GOV-01#10, D-K]
category: ARCHITECTURE
level: TECHNICAL
status: CONFIRMED
statement: >
  CellKey = (layer, level: i8, brick coordinate [i64;3]); VoxelPos = [i64;3]; brick edge 16; Euclidean division for negative coordinates. Both widths are single type aliases in cube-core. Level 0 is the layer's base resolution; positive levels are finer (2^level subdivisions per axis); negative levels are reserved. Core is dimensionless: voxel size in metres is not a Core concept. The world is bounded only by i64 range (a practically endless world, as the confirmed concept requires). Recorded as Provisional PC-1 with revalidation gate RG-1; edge 16 is unproven by benchmark.
  Consequences: Key width changes before any approved persistence format are cheap; after it they are a migration. Micro-voxels need no new key field (level is already in the key).
  Related contracts: C-07, C-08, C-09, C-11, C-12
  Reversibility: cheap now (aliases), expensive after the first approved format; therefore Provisional
decided_by: AGENT_DELEGATED
delegation_basis: DEC-001 (owner instruction 2026-11-09, sections 2 and 6): reversible engineering decision inside the confirmed concept
decided_on: 2026-11-09
decided_on_basis: date of this registry commit (agent decision under delegation)
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-DELEGATION (decision by the agent under DEC-001; the owner may reject or amend any entry)
source: docs/archive/gov01/GOV-01-TECHNICAL-RECOMMENDATIONS.md row 10; docs/archive/gov01/GOV-01-DECISION-REGISTER.md item 10
source_availability: IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/api-contracts/core-contracts.md]
history:
  - 2026-11-09 entry created

## DEC-065  D-T: transforms
id: DEC-065
aliases: [GOV-01#11, D-T]
category: ARCHITECTURE
level: TECHNICAL
status: CONFIRMED
statement: >
  Integer transform (translation, one of 24 axis rotations, power-of-two scale) for coordinates and GeometryTransform only (classes 1-2). Authoritative geometry state contains no floating point. The continuous pose of a movable or dynamic body is entity/physics state, not Layer.transform. A separate numeric policy is required before Mesh, SDF or gameplay-state contracts are approved; render-only data may use float.
  Consequences: Determinism for server authority and cross-play; dynamic constructions do not weaken the rule.
  Related contracts: C-04
decided_by: AGENT_DELEGATED
delegation_basis: DEC-001 (owner instruction 2026-11-09, sections 2 and 6): reversible engineering decision inside the confirmed concept
decided_on: 2026-11-09
decided_on_basis: date of this registry commit (agent decision under delegation)
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-DELEGATION (decision by the agent under DEC-001; the owner may reject or amend any entry)
source: docs/archive/gov01/GOV-01-TECHNICAL-RECOMMENDATIONS.md row 11; docs/archive/gov01/GOV-01-DECISION-REGISTER.md item 11
source_availability: IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/production/architecture-blocker-decisions.md]
history:
  - 2026-11-09 entry created

## DEC-066  D-C: channel model; opaque material index
id: DEC-066
aliases: [GOV-01#12, D-C]
category: ARCHITECTURE
level: TECHNICAL
status: CONFIRMED
statement: >
  Built-in channels (Material, quantised Density, Attribute) with Constant and Dense encodings. The material value in Core is an opaque `MaterialIndex` (initial width u16, a type alias); Core knows nothing about namespace:name or packs. Persistent identity lives above Core in the content layer; the index width is declared in the save-format header. Quantisation widths for Density stay open until the SDF backend. Provisional PC-1.
  Consequences: Satisfies the namespace:name identity concept (DEC-031) without putting identity into Core.
  Related contracts: C-08, C-09, C-10
  Reversibility: cheap before the first format
decided_by: AGENT_DELEGATED
delegation_basis: DEC-001 (owner instruction 2026-11-09, sections 2 and 6): reversible engineering decision inside the confirmed concept
decided_on: 2026-11-09
decided_on_basis: date of this registry commit (agent decision under delegation)
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-DELEGATION (decision by the agent under DEC-001; the owner may reject or amend any entry)
source: docs/archive/gov01/GOV-01-TECHNICAL-RECOMMENDATIONS.md row 12; docs/archive/gov01/GOV-01-DECISION-REGISTER.md item 12
source_availability: IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/production/architecture-blocker-decisions.md]
history:
  - 2026-11-09 entry created

## DEC-067  D-0: P0 scope
id: DEC-067
aliases: [GOV-01#13, D-0]
category: ARCHITECTURE
level: TECHNICAL
status: CONFIRMED
statement: >
  Option A: P0-alpha plus Provisional PC-1. P0-alpha = C-01, C-02, C-03, C-04, C-06, C-16, C-17, C-18, C-19 (core slices). PC-1 = C-07, C-08, C-09 (Provisional, RG-1).
  Consequences: Consistent with AOD-18 and AOD-24.
  Related contracts: see plan section 4
decided_by: AGENT_DELEGATED
delegation_basis: DEC-001 (owner instruction 2026-11-09, sections 2 and 6): reversible engineering decision inside the confirmed concept
decided_on: 2026-11-09
decided_on_basis: date of this registry commit (agent decision under delegation)
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-DELEGATION (decision by the agent under DEC-001; the owner may reject or amend any entry)
source: docs/archive/gov01/GOV-01-TECHNICAL-RECOMMENDATIONS.md row 13; docs/archive/gov01/GOV-01-DECISION-REGISTER.md item 13
source_availability: IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/production/architecture-blocker-decisions.md]
history:
  - 2026-11-09 entry created

## DEC-068  D-E: toolchain and execution model
id: DEC-068
aliases: [GOV-01#14, D-E]
category: ARCHITECTURE
level: TECHNICAL
status: CONFIRMED
statement: >
  (a) toolchain pinned in rust-toolchain.toml to 1.97.1 (the version installed and verified on the development machine); (b) edition 2024; (c) CI host: GitHub Actions ubuntu-latest and windows-latest; (e) licence allow-list for dependencies: MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, Unicode-3.0, Zlib, MPL-2.0 (cargo-deny); (f) panic strategy per artifact: libraries and the GDExtension artifact use unwind; a future dedicated server binary may use abort by its own entry; (g) FFI boundary uses catch_unwind in the adapter; (h) no async runtime in Core, Processing, Editing or backends; (i) dev-dependencies allowed: proptest, trybuild; (j) Core has zero external dependencies. The project's own licence is not decided here (see the DEFERRED entry).
  Consequences: GOV-05 specification and GOV-06 CI use these values. Toolchain bumps are ordinary PRs.
  Related contracts: -
  Reversibility: cheap
decided_by: AGENT_DELEGATED
delegation_basis: DEC-001 (owner instruction 2026-11-09, sections 2 and 6): reversible engineering decision inside the confirmed concept
decided_on: 2026-11-09
decided_on_basis: date of this registry commit (agent decision under delegation)
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-DELEGATION (decision by the agent under DEC-001; the owner may reject or amend any entry)
source: docs/archive/gov01/GOV-01-TECHNICAL-RECOMMENDATIONS.md row 14; docs/archive/gov01/GOV-01-DECISION-REGISTER.md item 14
source_availability: IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/production/architecture-blocker-decisions.md]
history:
  - 2026-11-09 entry created

## DEC-069  Branch option for the docs-only baseline
id: DEC-069
aliases: [GOV-01#15, branch option]
category: ARCHITECTURE
level: TECHNICAL
status: CONFIRMED
statement: >
  A new docs-only branch from master. Implemented as the integration branch `dev` (see the Git strategy entry): master stays docs-only until a release; dev receives the documentation baseline and production code through feature branches. GOV-04 acceptance is read as: no spike code anywhere in dev; master unchanged.
  Consequences: The wording 'baseline on master' in GOV-04 is amended to 'baseline branch (dev)' by the owner's Git strategy instruction.
  Related contracts: -
decided_by: AGENT_DELEGATED
delegation_basis: DEC-001 (owner instruction 2026-11-09, sections 2 and 6): reversible engineering decision inside the confirmed concept
decided_on: 2026-11-09
decided_on_basis: date of this registry commit (agent decision under delegation)
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-DELEGATION (decision by the agent under DEC-001; the owner may reject or amend any entry)
source: docs/archive/gov01/GOV-01-TECHNICAL-RECOMMENDATIONS.md row 15; docs/archive/gov01/GOV-01-DECISION-REGISTER.md item 15
source_availability: IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/production/integration-and-qa-plan.md]
history:
  - 2026-11-09 entry created

## DEC-070  Project's own licence
id: DEC-070
aliases: [GOV-01#14 (d)]
category: LICENCE
level: TECHNICAL
status: DEFERRED
statement: >
  The licence of Cube's own code is a personal owner choice and is not delegated. Until decided, workspace crates are `publish = false` with no licence field, and cargo-deny treats workspace crates as private.
decided_by: PENDING
decided_on: UNKNOWN
decided_on_basis: 
recorded_on: 2026-11-09 (this registry)
confirmed_on: 
confirmation_evidence: NONE
source: GOV-01#14 parameter d; ODM-09
source_availability: IN_REPOSITORY
owner: Project Owner
affected_documents: [deny.toml (dev, PS-0 skeleton)]
history:
  - 2026-11-09 deferred; trigger: before the first public release or the first external contribution

## DEC-071  P0-alpha contracts and PC-1 Provisional status (GOV-02)
id: DEC-071
aliases: [GOV-02, D-0]
category: PROCESS
level: TECHNICAL
status: CONFIRMED
statement: >
  Contracts C-01, C-02, C-03, C-04, C-06, C-16, C-17, C-18, C-19 (core slices, with the amendments of the GOV-01 entries) are treated as the P0-alpha baseline for PS-1 implementation under delegation. C-07, C-08, C-09 are Provisional PC-1. Basis: the GOV-01 entries above. Assumptions: Voxel Architecture 1.0 absent (RG-1). Affected modules: cube-core. Risks: key width and edge 16 may change at RG-1. Revisit when: RG-1 or the first persistence-format approval. Contract text in docs/api-contracts is still the Draft text; code that deviates from it states which is wrong (integration-and-qa-plan 5.5). Contract status for consumers outside cube-core is unchanged (Draft/Proposed).
decided_by: AGENT_DELEGATED
delegation_basis: DEC-001 (owner instruction 2026-11-09, sections 2 and 6): reversible engineering decision inside the confirmed concept
decided_on: 2026-11-09
decided_on_basis: date of this registry commit (agent decision under delegation)
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-DELEGATION (decision by the agent under DEC-001; the owner may reject or amend any entry)
source: docs/production/production-architecture-plan.md sections 3.1, 3.2, 4
source_availability: IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/production/ps-0-exit-checklist.md]
history:
  - 2026-11-09 entry created

## DEC-072  Ownership matrix and written deferrals (GOV-03)
id: DEC-072
aliases: [GOV-03, X-3]
category: PROCESS
level: TECHNICAL
status: CONFIRMED
statement: >
  Core Agent, Integration Owner and Agent I are agent roles filled by agent sessions working under DEC-001. Modules needed by PS-1..PS-5 (core, processing, editing-api, orchestration, voxel backend, testkit, xtask) are owned by these roles. Owners of WorldGen, Networking Adapter, Server Runtime, Client Runtime, Engine Adapter, Import/Export, O-1 (derived providers) and O-2 (modifier evaluation) are deferred in writing: they are needed from PS-11/PS-12/PS-13, and for derived providers at PS-5, and no earlier task depends on them.
decided_by: AGENT_DELEGATED
delegation_basis: DEC-001 (owner instruction 2026-11-09, sections 2 and 6): reversible engineering decision inside the confirmed concept
decided_on: 2026-11-09
decided_on_basis: date of this registry commit (agent decision under delegation)
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-DELEGATION (decision by the agent under DEC-001; the owner may reject or amend any entry)
source: docs/production/module-boundaries.md A.2; GOV-03 row
source_availability: IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/production/ps-0-exit-checklist.md]
history:
  - 2026-11-09 entry created

## DEC-073  CI specification (GOV-05)
id: DEC-073
aliases: [GOV-05, X-4]
category: PROCESS
level: TECHNICAL
status: CONFIRMED
statement: >
  The CI specification is docs/production/ci-specification.md on dev: required checks fmt, clippy -D warnings, test (workspace), deny, archcheck v0; tool pins from the D-E entry; pass conditions and the self-test of seeded failures. Required status checks on master and dev are named there; enabling branch protection is a GitHub administration act and is tracked as an external prerequisite.
decided_by: AGENT_DELEGATED
delegation_basis: DEC-001 (owner instruction 2026-11-09, sections 2 and 6): reversible engineering decision inside the confirmed concept
decided_on: 2026-11-09
decided_on_basis: date of this registry commit (agent decision under delegation)
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-DELEGATION (decision by the agent under DEC-001; the owner may reject or amend any entry)
source: docs/production/integration-and-qa-plan.md section 3
source_availability: IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/production/ps-0-exit-checklist.md]
history:
  - 2026-11-09 entry created

## DEC-074  ADG correction D-01/D-02 (release closure and G1 merge policy)
id: DEC-074
aliases: [ODM-14 (partly), D-01, D-02, ST-2]
category: PROCESS
level: TECHNICAL
status: CONFIRMED
statement: >
  The patch docs/archive/gov01/ADG-D01-D02.proposed.patch is applied to docs/production/module-boundaries.md: every engineering task of PS-2..PS-14 gets G1 in its merge-after column (rule R-MERGE-G1) and QA-04 gets the gates G5, G6, G7, G8, G9, G11, G12. Verified by `python docs/archive/audit/adg_check.py --strict` (all 8 checks PASS, longest path 25 tasks). The M2-T task-graph optimisations OPT-01..OPT-08 of ODM-14 are not applied here.
decided_by: AGENT_DELEGATED
delegation_basis: DEC-001 (owner instruction 2026-11-09, section 7: defects found by checks are fixed within delegated authority)
decided_on: 2026-11-09
decided_on_basis: date of this registry commit (agent decision under delegation)
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-DELEGATION (decision by the agent under DEC-001; the owner may reject or amend)
source: docs/archive/gov01/ADG-D01-D02-PATCH-PROPOSAL.md; docs/archive/gov01/ADG-D01-D02-ANALYSIS.md
source_availability: IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/production/module-boundaries.md]
history:
  - 2026-11-09 entry created; patch applied in the same change set

## DEC-075  Conditions of the confirmation of AOD-01..24
id: DEC-075
aliases: [X-1 (confirmation batch), GOV-02 evidence 'AOD-01..24 countersigned']
category: PROCESS
level: TECHNICAL
status: CONFIRMED
statement: >
  The owner confirms the architectural decisions AOD-01..24 in the wording of docs/decisions/owner-architecture-decisions-v1.md (registry entries DEC-004..DEC-027). Conditions stated by the owner: AOD-01..23 are confirmed within their original scope; AOD-24 keeps the status Conditionally Approved and all related constraints; Godot stays the first client of Cube but is not a mandatory dependency of the Rust Core; `faces` stays the correctness reference until proper comparative measurements against `greedy`; the confirmation does not automatically approve missing specifications, unverified API contracts, experimental code or test results; previously recorded open questions stay open unless separately resolved.
not_decided:
  - missing specifications (Voxel Architecture, Geometry Processing Specification, GDD/TDD)
  - any API contract not otherwise approved
  - spike code and spike results
  - questions recorded as open in owner-architecture-decisions-v1.md section 2
decided_by: OWNER
decided_on: 2026-11-09
decided_on_basis: date of the owner's message
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT (explicit owner message 'Cube - Final PS-0 Closure & Immediate Production Implementation', section 1; no Git signature required)
source: owner chat message, session 2026-11-09 (transcript outside the repository)
source_availability: UNAVAILABLE in repository (session transcript only)
owner: Project Owner
refines: [DEC-029]
affected_documents: [docs/decisions/owner-architecture-decisions-v1.md, docs/production/ps-0-exit-checklist.md]
history:
  - 2026-11-09 entry created

## DEC-076  INT-01 record and PS-1 gate G1
id: DEC-076
aliases: [INT-01, G1]
category: PROCESS
level: TECHNICAL
status: CONFIRMED
statement: >
  The Integration Owner role records, with CI evidence, that the Core slice of the P0-alpha contracts (C-01, C-02, C-03, C-04, C-06, C-16, C-17, C-18, C-19) is Implemented and that the PC-1 contracts (C-07, C-08, C-09) are Implemented under Provisional approval; no contract is Frozen. The per-contract slice and the open parts are listed in docs/api-contracts/README.md. Gate G1 (PS-1 exit criteria, production-architecture-plan section 15) is evaluated in docs/production/ps-0-exit-checklist.md section 6.
decided_by: AGENT_DELEGATED
delegation_basis: DEC-001 (owner instruction 2026-11-09: close tasks and gates on fulfilled criteria)
decided_on: 2026-11-09
decided_on_basis: date of this registry commit (agent decision under delegation)
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-DELEGATION (decision by the agent under DEC-001; the owner may reject or amend). CI evidence - commits were admitted into dev by the ruleset only after the six required checks passed
source: docs/production/agent-task-breakdown.md INT-01 row
source_availability: IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/api-contracts/README.md, docs/production/ps-0-exit-checklist.md]
history:
  - 2026-11-09 entry created

## DEC-077  Use of TC-01 (Provisional Processing subset) to implement PS-2
id: DEC-077
aliases: [TC-01, ODM-02 (partly)]
category: PROCESS
level: TECHNICAL
status: CONFIRMED
statement: >
  The owner permits the Provisional contract TC-01 (PR-01...PR-05 subset, docs/archive/m2-delivery-review/M2_MINIMUM_CONTRACTS.md section 3) to be used to prepare and implement PS-2 Processing Foundation: a temporary technical specification, reversible engineering decisions within the approved principles, production code and tests that rest on explicitly marked provisional contracts, spike checks of disputed technical questions, and sequential integration where the entry and merge conditions of each task allow it. The implementation record is docs/production/processing-foundation-contract.md.
not_decided: >
  Owner approval of PR-01...PR-08 (GOV-07); closing ODM-02 (whether a Provisional registration satisfies the condition of AOD-24); the Geometry Processing Specification (EXT-GPS); GOV-16 (C-05 approval); GOV-24 (OD-R4 snapshot lifetime and caps); closing gate G2; declaring any provisional contract Frozen; changing the product concept.
decided_by: OWNER
decided_on: 2026-11-09
decided_on_basis: date of the owner's message
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT (explicit owner statement in chat; permitted source under DEC-002)
source: OWNER-CHAT-2026-11-09 (instruction 'PS-2 Processing Foundation, Provisional Track and Continuous Development') section 3
source_availability: UNAVAILABLE in repository (session transcript only)
owner: Project Owner
affected_documents: [docs/production/processing-foundation-contract.md, docs/production/ps-0-exit-checklist.md]
history:
  - 2026-11-09 entry created

## DEC-078  GOV-07: owner-recorded alternative for the Geometry Processing Specification (PS-2)
id: DEC-078
aliases: [GOV-07, ODM-02, TC-01]
category: PROCESS
level: TECHNICAL
status: CONFIRMED
statement: >
  For PS-2 the owner accepts the limited provisional Processing Foundation specification (docs/production/processing-foundation-contract.md, DEC-077) in place of the absent external Geometry Processing Specification. This is the registered owner alternative that the acceptance criterion of GOV-07 allows ('PR-01..PR-08 Owner-Approved or an owner-recorded alternative') and that the PS-2 stage description names as the way to close G2 without the specification. It applies only to the implemented scope; it records that ODM-02 is answered: a Provisional registration satisfies the condition of AOD-24 for PS-2.
not_decided: >
  PR-01..PR-08 are NOT Owner-Approved and NOT Frozen. Gate RG-2 stays mandatory: when the Geometry Processing Specification appears, the contracts are diffed against it and migrated or corrected before they are frozen (TC-01 expiry: the earlier of GOV-07 approval of PR-01..PR-08 and the start of M3 content work). Unverified assumptions stay marked provisional. Nothing here may contradict the confirmed Cube architecture.
decided_by: OWNER
decided_on: 2026-11-09
decided_on_basis: date of the owner's message
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT (explicit owner statement in chat; permitted source under DEC-002)
source: OWNER-CHAT-2026-11-09 (instruction 'Finalize PS-2 / G2 and Start PS-3 Editing API') sections 2 and 6
source_availability: UNAVAILABLE in repository (session transcript only)
owner: Project Owner
affected_documents: [docs/production/processing-foundation-contract.md, docs/production/ps-0-exit-checklist.md]
history:
  - 2026-11-09 entry created

## DEC-079  GOV-16: C-05 GeometryBounds approved (minimal contract for PS-2)
id: DEC-079
aliases: [GOV-16, C-05]
category: PROCESS
level: TECHNICAL
status: CONFIRMED
statement: >
  The owner approves the minimal contract C-05 for PS-2: integer (i64) boxes compatible with VoxelPos, BrickCoord, CellKey and Region; one half-open inclusion rule [min, max); correct behaviour for negative coordinates; every overflow is an explicit error; deterministic comparison, union and intersection; an empty bound is a distinct state; unknown (not loaded, failed) data is never reported as empty or as outside the world; no global world-size limit (limits belong to operations, allocation, active regions, snapshots and server guards); invalid bounds are rejected; no game logic and no engine dependency. The implementation is CORE-15 (crates/cube-core/src/bounds.rs), checked against these principles with no material discrepancy.
not_decided: >
  Not extended to territory management or server permissions. Derived bounds beyond the exact world box (OBB, bounding sphere) are not part of this approval. 'Tight versus conservative' remains conservative (never shrinks).
decided_by: OWNER
decided_on: 2026-11-09
decided_on_basis: date of the owner's message
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT (explicit owner statement in chat; permitted source under DEC-002)
source: OWNER-CHAT-2026-11-09 (instruction 'Finalize PS-2 / G2 and Start PS-3 Editing API') section 3
source_availability: UNAVAILABLE in repository (session transcript only)
owner: Project Owner
affected_documents: [docs/api-contracts/core-contracts.md, docs/api-contracts/README.md, crates/cube-core/src/bounds.rs]
history:
  - 2026-11-09 entry created

## DEC-080  GOV-24 / OD-R4: snapshot resource budget policy for Processing
id: DEC-080
aliases: [GOV-24, OD-R4]
category: PROCESS
level: TECHNICAL
status: CONFIRMED
statement: >
  The owner approves this policy. Snapshots have a bounded cost of holding resources; the limits apply to active operations, never to the size of the world. Required budgets: per snapshot, total pinned resources per asset, number of concurrently active snapshots, task/snapshot lifetime (release of stale snapshots), safe cancellation, release on completion or cancellation, and a diagnosable refusal when a budget is exceeded. Numeric values are configuration, not architecture; the tested defaults of PROC-06 are the starting values. Hard protective limits need a technical reason, must not limit the game world, must not lose data silently and must not break Processing correctness. Accounting distinguishes pinned bricks, snapshots, estimated data cost and the resources actually counted; a brick count is not claimed as a proven byte bound, and the byte figure is an explicit conservative estimate with stated accuracy limits.
not_decided: >
  Concrete numbers; memory policy of derived-result caches (not snapshots); device budgets (DG-1..3).
decided_by: OWNER
decided_on: 2026-11-09
decided_on_basis: date of the owner's message
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT (explicit owner statement in chat; permitted source under DEC-002)
source: OWNER-CHAT-2026-11-09 (instruction 'Finalize PS-2 / G2 and Start PS-3 Editing API') section 4
source_availability: UNAVAILABLE in repository (session transcript only)
owner: Project Owner
affected_documents: [docs/production/processing-foundation-contract.md, crates/cube-processing/src/caps.rs]
history:
  - 2026-11-09 entry created

## DEC-081  PS-2 closed, gate G2 passed
id: DEC-081
aliases: [G2, PS-2]
category: PROCESS
level: TECHNICAL
status: CONFIRMED
statement: >
  Gate G2 (PS-2 Processing Foundation) is passed. All twelve member tasks are merged and accepted (CORE-15, CORE-16, PROC-01..PROC-06, TEST-08, GOV-07 through the owner alternative DEC-078, GOV-16 DEC-079, GOV-24 DEC-080). Evidence: required CI checks green on the integration commit d164d21 (run 42624528085), adg_check.py --strict PASS, evaluation in docs/production/ps-0-exit-checklist.md section 7. The provisional Processing contracts remain subject to RG-2.
decided_by: AGENT_DELEGATED
delegation_basis: DEC-001 (owner instruction: close tasks and gates on fulfilled criteria)
decided_on: 2026-11-09
decided_on_basis: date of this registry commit (agent decision under delegation)
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-DELEGATION (the owner may reject or amend); CI evidence as listed
source: docs/production/ps-0-exit-checklist.md section 7
source_availability: IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/production/ps-0-exit-checklist.md, docs/archive/PROJECT_STATUS.md]
history:
  - 2026-11-09 entry created

## DEC-082  GOV-09: Editing API contracts approved (PS-3 scope)
id: DEC-082
aliases: [GOV-09, E-02..E-07, E-09..E-12]
category: PROCESS
level: TECHNICAL
status: CONFIRMED
statement: >
  The owner approves the implemented Editing API interfaces E-02, E-03, E-04, E-05, E-06, E-07, E-09, E-11, E-12 and the storage ports, and E-10 as the minimal checkpoint contract the storage ports need. The seven provisional resolutions of docs/production/editing-api-contract.md are accepted as the current implementation basis: (1) backends get read-only views and return write sets; (2) operation parameters are opaque versioned data owned by the kind owners; (3) the facade is synchronous; (4) atomic commit and rollback without partial application; (5) conflicts are returned explicitly with resources and revisions, no automatic resolution; (6) delta payload and serialisation stay abstract until Persistence; (7) the journal uses a monotonic sequence of confirmed records, storage details belong to Persistence. Approval covers the PS-3 scope only.
not_decided: >
  No contract is Frozen. The provisional parts stay provisional. E-08 (ModifierStack) and E-13 (networking boundary) are not approved. Further checks: contract tests, API snapshot (api-snapshot.txt) and the real Orchestration path (PS-4).
decided_by: OWNER
decided_on: 2026-11-09
decided_on_basis: date of the owner's message
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT (explicit owner statement in chat; permitted source under DEC-002, ODM-10)
source: OWNER-CHAT-2026-11-09 (instruction 'Close GOV-09 / G3, Implement PS-4 and Continue Production Development') section 3
source_availability: UNAVAILABLE in repository (session transcript only)
owner: Project Owner
affected_documents: [docs/production/editing-api-contract.md, docs/api-contracts/README.md, crates/cube-editing-api]
history:
  - 2026-11-09 entry created

## DEC-083  Permission schema and region ACL: ownership
id: DEC-083
aliases: [GOV-09, E-02]
category: PROCESS
level: TECHNICAL
status: CONFIRMED
statement: >
  The owner of the permission schema and region ACLs is Server/Authorization (applies and checks rights). Gameplay owns role, group, ownership and game policy rules; Editing Orchestration checks authorization through a port before changing the world; Core stores and changes geometry state without game logic; the Client offers permission UI but takes no authoritative decision. The schema must allow future build zones, shared editing and server permissions. For PS-4 an authorization port and a verifiable deny policy are enough; full regional permissions are not implemented. A local client gets no extra rights because of its origin.
not_decided: >
  The concrete permission model and region ACL implementation.
decided_by: OWNER
decided_on: 2026-11-09
decided_on_basis: date of the owner's message
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT (explicit owner statement in chat; permitted source under DEC-002, ODM-10)
source: OWNER-CHAT-2026-11-09 (instruction 'Close GOV-09 / G3, Implement PS-4 and Continue Production Development') section 3
source_availability: UNAVAILABLE in repository (session transcript only)
owner: Project Owner
affected_documents: [crates/cube-editing-api/src/auth.rs, crates/cube-orchestration]
history:
  - 2026-11-09 entry created

## DEC-084  Sculpt stroke is one transaction; no nested transactions in PS-4
id: DEC-084
aliases: [GOV-09, E-04]
category: PROCESS
level: TECHNICAL
status: CONFIRMED
statement: >
  One completed sculpt stroke is one logical user transaction. PS-4 has no nested transactions and no partially successful commits. A long stroke may accumulate changes inside one logical operation; if resource limits force the computation to be split, that must not silently become several independent user commits. Cancelling before commit leaves the authoritative state unchanged. Undo will treat a stroke as one unit when the tool exists.
not_decided: >
  Streaming sculpt, partial application and distributed transactions are deferred to their stages.
decided_by: OWNER
decided_on: 2026-11-09
decided_on_basis: date of the owner's message
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT (explicit owner statement in chat; permitted source under DEC-002, ODM-10)
source: OWNER-CHAT-2026-11-09 (instruction 'Close GOV-09 / G3, Implement PS-4 and Continue Production Development') section 3
source_availability: UNAVAILABLE in repository (session transcript only)
owner: Project Owner
affected_documents: [docs/production/editing-api-contract.md]
history:
  - 2026-11-09 entry created

## DEC-085  Delta encoding and checkpoint frequency deferred to PS-10
id: DEC-085
aliases: [GOV-09, E-09, E-10, AOD-17, AOD-21]
category: PROCESS
level: TECHNICAL
status: CONFIRMED
statement: >
  The concrete delta encoding and the checkpoint period are deferred to PS-10 Persistence. Until then the interfaces stay abstract and versioned, no binary format and no database are fixed, and no mandatory checkpoint period is introduced without measurements and crash-recovery checks. Compatible with AOD-17 and AOD-21. This is an explicit deferral, not a blocker of G3.
not_decided: >
  Encoding, format, period.
decided_by: OWNER
decided_on: 2026-11-09
decided_on_basis: date of the owner's message
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT (explicit owner statement in chat; permitted source under DEC-002, ODM-10)
source: OWNER-CHAT-2026-11-09 (instruction 'Close GOV-09 / G3, Implement PS-4 and Continue Production Development') section 3
source_availability: UNAVAILABLE in repository (session transcript only)
owner: Project Owner
affected_documents: [docs/production/editing-api-contract.md]
history:
  - 2026-11-09 entry created

## DEC-086  OperationJournal ownership and observer delivery
id: DEC-086
aliases: [GOV-09, E-12, OperationJournal]
category: PROCESS
level: TECHNICAL
status: CONFIRMED
statement: >
  The durable OperationJournal is owned by Persistence. Editing Orchestration reconciles changes with the required journal records. Success is reported for an operation that requires durability only after the durable record is confirmed; a journal error never produces a false success; the state "change applied, journal lost" is not left without a defined recovery mechanism. Where interfaces cannot give the needed atomicity, it is not imitated: the minimal contract blocker is recorded and a safe boundary is implemented. Ordinary observers need not be durable before the response; losing a UI notification never loses an authoritative change; reliable subscribers need a separate replay/resync mechanism in the future. The journal is not mixed with History or UI observers. Journal sequence numbers must not lead to reuse of an OperationId after failures. File format and durable storage are PS-10.
not_decided: >
  Blocker recorded (CB-1): Core commit and the journal cannot be one atomic action. Safe boundary implemented in the API and Orchestration: write-ahead journal phases (Intent, then Confirmed or Aborted), the result AppliedUnconfirmed when the change is applied but its confirmation is not durable, retry by the same OperationId completing the confirmation without applying twice, and an Intent without a later record resolved at recovery by comparing with Core (Persistence defines the procedure in PS-10). Crash durability is not claimed before PS-10.
decided_by: OWNER
decided_on: 2026-11-09
decided_on_basis: date of the owner's message
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT (explicit owner statement in chat; permitted source under DEC-002, ODM-10)
source: OWNER-CHAT-2026-11-09 (instruction 'Close GOV-09 / G3, Implement PS-4 and Continue Production Development') section 3
source_availability: UNAVAILABLE in repository (session transcript only)
owner: Project Owner
affected_documents: [docs/production/editing-api-contract.md, crates/cube-editing-api/src/storage.rs, crates/cube-editing-api/src/types/result.rs]
history:
  - 2026-11-09 entry created

## DEC-087  PS-3 closed, gate G3 passed
id: DEC-087
aliases: [G3, PS-3]
category: PROCESS
level: TECHNICAL
status: CONFIRMED
statement: >
  Gate G3 (PS-3 Editing API and Contracts) is passed. All six member tasks are merged and accepted (API-01..API-05 on dev, GOV-09 through DEC-082..DEC-086). Evidence: required CI checks green on the integration commit (see docs/production/ps-0-exit-checklist.md section 8), API snapshot, contract tests, archcheck including the new conflict-marker rule. The contracts are Owner-Approved for the PS-3 scope and are not Frozen.
decided_by: AGENT_DELEGATED
delegation_basis: DEC-001 (owner instruction: close tasks and gates on fulfilled criteria)
decided_on: 2026-11-09
decided_on_basis: date of this registry commit (agent decision under delegation)
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-DELEGATION (the owner may reject or amend); CI evidence as listed
source: docs/production/ps-0-exit-checklist.md section 8
source_availability: IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/production/ps-0-exit-checklist.md, docs/archive/PROJECT_STATUS.md]
history:
  - 2026-11-09 entry created

## DEC-088  PS-4 closed, gate G4 passed
id: DEC-088
aliases: [G4, PS-4]
category: PROCESS
level: TECHNICAL
status: CONFIRMED
statement: >
  Gate G4 (PS-4 Editing Orchestration) is passed. All seven member tasks (ORCH-01..ORCH-05, TEST-03a, TEST-10) are merged and accepted. Evidence: required CI checks green on 920f632 (run 42630261984; the integration commit adds only documentation and the merge of dev), real-path mutation tests, backend conformance with self-checks, compile-fail boundary tests, archcheck rules ARCH-HANDLE and ARCH-USER-COMMIT-PATH, evaluation in docs/production/ps-0-exit-checklist.md section 9. No crash durability is claimed (journal is PS-10, boundary CB-1 of DEC-086).
decided_by: AGENT_DELEGATED
delegation_basis: DEC-001 (owner instruction: close tasks and gates on fulfilled criteria)
decided_on: 2026-11-09
decided_on_basis: date of this registry commit (agent decision under delegation)
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-DELEGATION (the owner may reject or amend); CI evidence as listed
source: docs/production/ps-0-exit-checklist.md section 9
source_availability: IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/production/ps-0-exit-checklist.md, docs/archive/PROJECT_STATUS.md]
history:
  - 2026-11-09 entry created

## DEC-089  O-1: technical owners of derived providers
id: DEC-089
aliases: [O-1, GOV-03 (deferral of DEC-072 resolved for derived providers)]
category: PROCESS
level: TECHNICAL
status: CONFIRMED
statement: >
  The deferral of owner O-1 (derived data providers) recorded in DEC-072 is resolved for the needs of PS-5 by assigning technical ownership inside the existing agent roles: the Voxel Agent owns voxel mesh extraction (faces, greedy), voxel collision and voxel LOD (crates/cube-backend-voxel); the SDF Agent owns SDF-specific derived providers; the Mesh Agent owns mesh-specific derived providers; the Processing Agent owns the common scheduling, dependency graph, cancellation, caps and stale-result rejection; the Integration Owner owns the public contracts and the integration tests. This assigns technical responsibility only. It adds no game feature, no new agent management system and no new module owners for WorldGen, Networking, Server, Client, Engine Adapter, Import/Export or O-2, which stay deferred as in DEC-072.
decided_by: AGENT_DELEGATED
delegation_basis: DEC-001 and the owner instruction of 2026-11-09 (section 6 of 'PS-5 Completion, Persistence Foundation and First Playable World')
decided_on: 2026-11-09
decided_on_basis: date of this registry commit (agent decision under delegation)
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-DELEGATION (the owner may reject or amend); the roles are the ones DEC-072 already names (Core Agent, Integration Owner, Agent I) plus the backend roles of the ADG ownership column
source: docs/production/agent-task-breakdown.md VOX-02..VOX-05 rows ('pending O-1')
source_availability: IN_REPOSITORY
owner: Project Owner
affected_documents: [docs/production/ps-0-exit-checklist.md]
history:
  - 2026-11-09 entry created

## DEC-090  GOV-08 and GOV-17 provisional; CORE-12 integration exception from EXT-VA-RG1
id: DEC-090
aliases: [GOV-08, GOV-17, CORE-12, C-10, RG-1 (partly)]
category: PROCESS
level: TECHNICAL
status: CONFIRMED
statement: >
  The owner records: (1) GOV-08: PC-1 (C-07, C-08, C-09) is accepted as the working provisional baseline for PS-5; (2) GOV-17: C-10 is approved as the minimal provisional palette contract in the scope of the implemented CORE-12 (commit a15595e: content ids, runtime indices, registration with collision diagnostics, strict and recovery resolution of saved tables); (3) CORE-12 may be integrated into dev after a check that the implementation matches the contract and after the required CI; (4) this is a limited owner-recorded alternative for the integration of CORE-12, and it is registered here as the exception from the merge condition EXT-VA-RG1 of CORE-12 in the ADG (Part C: merge-after CORE-10, EXT-VA-RG1, GOV-17, G1). The exception is for the CORE-12 merge only.
not_decided: >
  RG-1 is NOT passed and EXT-VA-RG1 is not supplied; RG-1 stays mandatory before the contracts concerned are Frozen and before the final save format (GOV-10). PC-1 and C-10 are not Frozen. No save format is approved. G5 is not passed by this decision (other members are open). The LayerMapping part of C-10 (layer to palette mapping and the revision wiring) is not part of CORE-12 and is not approved by this entry.
decided_by: OWNER
decided_on: 2026-11-09
decided_on_basis: date of the owner's message
recorded_on: 2026-11-09 (this registry)
confirmed_on: 2026-11-09
confirmation_evidence: E-CHAT (explicit owner statement in chat; permitted source under DEC-002)
source: OWNER-CHAT-2026-11-09 ('record GOV-08/GOV-17 and integrate CORE-12')
source_availability: UNAVAILABLE in repository (session transcript only)
owner: Project Owner
affected_documents: [docs/api-contracts/README.md, docs/api-contracts/core-contracts.md, docs/production/ps-0-exit-checklist.md, crates/cube-core/src/palette.rs]
history:
  - 2026-11-09 entry created

## DEC-091  Technical assignment of the WorldGen Agent (WGEN-01) and the Client Runtime Agent (CLI-00..02)
id: DEC-091
aliases: [GOV-03 (partial, WorldGen and Client Runtime), R-UNOWNED]
category: PROCESS
level: TECHNICAL
status: CONFIRMED
statement: >
  Rule R-UNOWNED of the ADG says that tasks of UNASSIGNED modules cannot start before GOV-03 names the owner. The owner message of 2026-11-10 ('First Visible World', sections 7.1 and 8) tells the engineering agent to assign a WorldGen Agent and a Client Runtime Agent in the existing ownership matrix where delegated technical assignment is sufficient, and to continue independent work otherwise. This entry makes that assignment, narrowly: (1) a WorldGen Agent (an agent session under DEC-001, not a new management structure) owns crate cube-worldgen for WGEN-01 (pure generator and version registry) only; (2) a Client Runtime Agent owns crate cube-client for CLI-00 (skeleton) and the engine-neutral parts of CLI-01 and CLI-02 (active region, derived-geometry cache, dirty updates, lifecycle) only. Both are reversible technical work inside the confirmed concept (a deterministic seeded generator, an engine-neutral read-only client), they add no game feature, and the crates keep the ADG edges and the archcheck rules (no floats in cube-worldgen, no commit handle and no engine in cube-client).
not_decided: >
  WGEN-02 (service), WGEN-03 (materialiser through the system-commit port), WGEN-04 and LEG-01 stay UNASSIGNED and gated (GOV-21, GOV-11). Engine Adapter (O-5, ADP-01..03), Networking (O-3), Server Runtime, Import/Export and the gameplay owner stay deferred as in DEC-072; CLI-03 and CLI-04 (input, intents) and everything that needs a game window or gameplay are not started. Generator version 1 (terrain-v1) is an engineering baseline, not an approved world design: no biomes, caves, water, structures or game rules are decided. If the owner reads R-UNOWNED as requiring a named owner in the repository, this entry is that record and the owner may reject or amend it.
decided_by: AGENT_DELEGATED
delegation_basis: DEC-001 and the owner instruction of 2026-11-10 (sections 7.1 and 8 of 'Cube - First Visible World: Recovery Safety, Geometry Backends, WorldGen and Client Foundation')
decided_on: 2026-11-10
decided_on_basis: date of this registry commit (agent decision under delegation)
recorded_on: 2026-11-10 (this registry)
confirmed_on: 2026-11-10
confirmation_evidence: E-DELEGATION (the owner may reject or amend)
source: OWNER-CHAT-2026-11-10 (sections 7.1 and 8); docs/production/module-boundaries.md rule R-UNOWNED
source_availability: UNAVAILABLE in repository (session transcript only)
owner: Project Owner
affected_documents: [docs/production/module-boundaries.md, docs/production/ps-0-exit-checklist.md]
history:
  - 2026-11-10 entry created

## DEC-092  Technical assignment of the Engine Adapter Agent (O-5) and the first-world composition (cube-server local host, cube-app)
id: DEC-092
aliases: [O-5, GOV-03 (partial, Engine Adapter and local Server host), R-UNOWNED]
category: PROCESS
level: TECHNICAL
status: CONFIRMED
statement: >
  The owner message of 2026-11-10 ('First Visible World: Godot 4, WorldGen Integration and Geometry Completion', sections 4 and 5) proposes an Engine Adapter Agent for cube-godot-adapter (GDExtension boundary, Godot API, lifecycle, integration tests) and asks for the first displayed world. Under DEC-001 and that instruction: (1) an Engine Adapter Agent (an agent session, no new management structure) owns crate cube-godot-adapter for ADP-01, ADP-02 and ADP-03 only; (2) the Client Runtime Agent of DEC-091 also covers CLI-03 and CLI-04; (3) to compose a world without a network, the ADG gets two additions that are not strategic: crate cube-server is created as an in-process local host only (the authoritative side of one local world: Core handles, Orchestration, persistence, generator, Processing; ADG edges unchanged, no edge to the client), and a leaf composition crate cube-app joins the local host with the client and exposes one SessionPort to the adapter (edges: Core, Processing, Orchestration, Persistence, voxel backend, client, server); the adapter gets one new edge, to cube-app, and calls nothing but its open function. The Godot/Rust split of the strategic architecture is unchanged: Rust Core stays engine-neutral, the adapter holds no authoritative state, no WorldGen, no editing or persistence logic and no commit, GDScript calls no commit, there is no child process for the local world, and the only `unsafe` is the GDExtension entry impl that the binding requires (the adapter crate alone relaxes the workspace unsafe lint for it).
not_decided: >
  The networked Server Runtime (PS-13, SRV tasks), Networking (O-3), authorisation beyond the local capability, the permission schema, multiplayer and every gameplay feature stay deferred as in DEC-072. WGEN-02..04 and LEG-01 stay gated by GOV-21 and GOV-11 (the first world uses a static bounded area, not streaming). The Godot version policy is not decided beyond this: the binding targets Godot 4.2 API (tested with 4.2.2). Whether cube-server and cube-app stay as they are when PS-13 arrives is a PS-13 matter; the local host is meant to become the local host of the later server. If the owner reads R-UNOWNED as requiring a named owner in the repository, this entry is that record and the owner may reject or amend it (the crates are small and removable).
decided_by: AGENT_DELEGATED
delegation_basis: DEC-001 and the owner instruction of 2026-11-10 (sections 4, 5 and 14 of 'Cube - First Visible World: Godot 4, WorldGen Integration and Geometry Completion')
decided_on: 2026-11-10
decided_on_basis: date of this registry commit (agent decision under delegation)
recorded_on: 2026-11-10 (this registry)
confirmed_on: 2026-11-10
confirmation_evidence: E-DELEGATION (the owner may reject or amend)
source: OWNER-CHAT-2026-11-10 (sections 4, 5); docs/production/module-boundaries.md rule R-UNOWNED
source_availability: UNAVAILABLE in repository (session transcript only)
owner: Project Owner
affected_documents: [docs/production/module-boundaries.md, docs/production/ps-0-exit-checklist.md, xtask/src/archcheck.rs]
history:
  - 2026-11-10 entry created
