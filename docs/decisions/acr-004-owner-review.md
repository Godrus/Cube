# ACR-004 Owner Review Package

**Status of ACR-004: `Pending`.** This document prepares the owner's decision; it **approves nothing**. ACR-004 changes status only when the owner explicitly approves or rejects it in a recorded decision (`owner-architecture-decisions-v1.md`, section 4). AU-1.0, ACR-001/002/003, all P0/P1 contracts and the Draft contracts named below are not changed by this document.
**Prepared:** 2026-11-05, from commit `bef9bda`. **Companions:** ACR-004 text in `audit-resolution-report.md`; AOD-20 (`owner-architecture-decisions-v1.md` section 5.1); `worldgen-core-boundary.md`; `owner-approval-matrix-v1.md` (OAM-01, OAM-09, OAM-10); drafts `../api-contracts/worldgen-contracts.md`, `system-commit-contracts.md`, `world-persistence-contracts.md`, `server-runtime-contracts.md`.

## 0. Recommendation in one table

| Item | Recommended status | Reason |
|---|---|---|
| ACR-004 as a whole | **stays `Pending`** until the owner approves | the recommendation below is the owner's to accept |
| Part A, hybrid boundary placement | recommend **approve** | expresses AOD-20, which is already Approved |
| Part B, new public Core port | recommend **approve the concept**, signature stays open | needed by AOD-20; not a P0 edit |
| Part C, OperationId origin (ActorKind) | recommend **approve** `ActorKind` on OperationId (`User`, `System`, `Recovery`) | the owner's instruction already uses `ActorKind::System` |
| Part D, revision semantics | **Proposed / Requires Owner Approval**: same asset/layer scopes | see OAM-10; this is the part that touches the P0 revision contract |
| Part E, Draft contracts | keep **Draft**; approve only as working drafts | no contract is approved by ACR-004 |
| Part F, further ACRs | **not filed**; list candidates only | the owners and the module grid for the server runtime are not assigned |

Order if the owner wants to approve in steps: A, then B and C together, then D; E and F need no ACR decision now.

## Part A: hybrid WorldGen/System-commit boundary

* **Already accepted by AOD-20:** procedural generation creates the base world through a separate WorldGen/System-commit boundary; it is not an ordinary Editing-API operation; user terrain changes go only through the Editing API; the server owns the world; initial generation is outside ordinary undo/redo; system generation receives an OperationId and takes part in revision/commit semantics; WorldGen never bypasses public Core ports.
* **Needs separate approval (ACR-004):** recording this as a structural change: a **WorldGen module** in the module and layer grid (depending on Core value types and ports only), and the rule that exactly three callers write to the authoritative asset (Editing service, Materialiser, persistence loader).
* **Effect on AU-1.0:** AU-1.0 (reconstructed copy) has no WorldGen module and no non-user actor, so the grid gains one module. AU-1.0 text is **not edited**; the ACR records the placement and the original approval stays with the owner (AOD-19).
* **Effect on P0/P1:** none directly (no contract is edited); the structure shapes the consumers of P0 contracts C-16…C-18.
* **Reversibility:** high before production code exists (a documentation placement); expensive after modules and dependency tests are built on it.
* **Acceptance criteria:** the owner confirms the module name, its dependency direction (Core value types and ports only; no Godot) and the three-writer rule; the dependency-rule test is extended to the new module (Gate 2); the call-site audit test exists before any production writer is merged.

## Part B: new public Core port (system-commit port)

* **Already accepted by AOD-20:** that generated data reaches Core only through a public system-commit port and never around it. The port's existence follows from AOD-20; **its shape does not.**
* **Needs separate approval:** the port as a new public Core interface: operations `Materialise`, `Evict`, `Restore`; callers limited to the Materialiser and the persistence loader; atomic and idempotent; never reachable from the network (draft: `system-commit-contracts.md` SC-05, SC-10). Signature, error set and thread-safety stay open.
* **Effect on AU-1.0:** adds a Core port that AU-1.0 does not list; the foundation placement of related types is already the subject of ACR-001 (Pending).
* **Effect on P0/P1:** extends the surface around the commit primitive and C-16/C-18 (P0). No P0 text is edited; P0 approval must include the port or explicitly exclude it. Editing contracts (P1) gain the rule that generated data is excluded from history.
* **Reversibility:** moderate: a port can be narrowed or renamed while Draft; once P0 is approved and implemented, changing it is a breaking change needing an ACR.
* **Acceptance criteria:** owner approves the concept and the caller restriction; later, as contract tests: atomicity and idempotence properties, a client cannot reach the port, WorldGen sees no Core internals beyond it, a dependency test for the three writers.

## Part C: origin of OperationId

* **Already accepted by AOD-20:** that system generation receives an OperationId.
* **Needs separate approval:** how the origin is expressed. **Recommendation:** an `ActorKind` attached to the OperationId with values `User`, `System`, `Recovery` (draft SC-01, SC-02): `System` for the Materialiser and server-internal automation; `Recovery` for journal replay (reusing the original ids). The origin is derived by the server from the authenticated path, never supplied by a client. Whether it is a field of the id or of the commit request, and the id format (OD-R8), stay open.
* **Effect on AU-1.0:** AU-1.0 §23 and ACR-001 place OperationId in Core; adding an origin to it is an extension of the same type, not a new location.
* **Effect on P0/P1:** extends C-17 (P0); E-02 AuthorizationContext (P1) already has an `Origin` field (LocalClient | Server | System) whose mapping to `ActorKind` must be fixed; the embedded server's `LocalClient` gains no extra rights.
* **Reversibility:** moderate (an additive enum while Draft; additive values later are minor changes, removal is breaking).
* **Acceptance criteria:** owner approves the three values and the server-derived rule; later: tests that a client cannot supply `System` or `Recovery`, that replay is idempotent, that undo ignores non-`User` commits.

## Part D: revision semantics

* **Already accepted by AOD-20:** that system generation takes part in revision/commit semantics (not which model).
* **Needs separate approval:** the revision model. Options and recommendation: OAM-10 in `owner-approval-matrix-v1.md`. **Recommended (`Proposed / Requires Owner Approval`):** system commits use the **same asset and layer revision scopes** as other commits, identified by `ActorKind::System`; **no separate generation epoch** is introduced without demonstrated necessity.
* **Effect on AU-1.0:** none if the recommendation is chosen (AU-1.0 §7 revision levels are used as they are); a separate epoch or domain would add a revision kind that AU-1.0 does not describe.
* **Effect on P0/P1:** the recommendation changes **no** RevisionToken field (the existing `Epoch` keeps its meaning of re-creation or recovery); B-6 (token width, commit algorithm) stays open and independent. A separate epoch or domain would change C-16 and the comparison rules.
* **Reversibility:** the recommended model is the easiest to extend later (an added scope is additive); removing a separate epoch after release would be breaking, which is the argument for not introducing one first.
* **Acceptance criteria:** owner approves the model; later, as tests: user edits conflict correctly using brick/region stamps while chunks stream in (no spurious conflicts), clients can tell a terrain update from a user edit through the commit's `ActorKind`/`ChangeType`, revisions stay monotonic across restart and recovery.

## Part E: new Draft contracts

* **Already accepted by AOD-20:** nothing about the contracts' content.
* **Needs separate approval:** each contract individually, later. WG-01…10, SC-01…10, WP-01…10, SR-01…10 are `Draft`/Proposed text and stay so. ACR-004 does not approve them.
* **Effect on AU-1.0:** none. **Effect on P0/P1:** none by themselves; SC-xx propose extensions of C-16/C-17/C-18, WP-xx is a reconciliation draft for P-01…P-07 (OI-4).
* **Reversibility:** high (documents).
* **Acceptance criteria:** the Integration Owner reviews each against its sources; contracts that rely on the missing documents are not approved (AOD-24); no implementation starts against a Draft.

## Part F: possible further ACRs (server runtime, manifest/recovery)

* **Already accepted:** AOD-22 (one authoritative server runtime, embedded or dedicated) and AOD-21 (hybrid persistence with a manifest and verified compaction) as decisions.
* **Needs separate approval, only if the owner decides to place them in the AU-1.0 grid:** (1) the **server runtime** as a module (AU-1.0 has the networking adapter L9 and no server host); (2) the **persistence manifest/recovery** module and its relation to History (B-9, the History to Persistence cycle rule). **Not filed.** They become ACR candidates when owners are assigned (OI-8) or when the Draft persistence contracts are reconciled with AU-1.0's persistence description.
* **Effect on AU-1.0:** potential new modules or layer assignments. **Effect on P0/P1:** none until filed. **Reversibility:** high while unfiled.
* **Acceptance criteria for filing:** a named owner for the module, a written conflict with an AU-1.0 clause, and the dependency direction checked against DEP-01.

## Owner response sheet (to be filled by the owner; empty here)

| Part | Approve / Amend / Reject | Note |
|---|---|---|
| A | | |
| B (concept) | | |
| C (ActorKind) | | |
| D (revision model, OAM-10) | | |
| E (drafts stay Draft) | | |
| F (no ACR now) | | |

ACR-004 is recorded as Approved only when the owner states it explicitly; partial approval of A to C with D open is a valid outcome and is recorded as such.
