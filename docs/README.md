# Cube — Documentation Index (Geometry, rev 1.1)

## Source Documents Register

Authority order follows Agent Development Specification §2. Content of absent documents is **not** reconstructed anywhere in this repository.

| Document | Version | Location | State |
|---|---|---|---|
| Architecture Update (AU-1.0) | 1.0 | `docs/decisions/architecture-update-v1.0.md` | Present as reconstructed copy — **Proposed/Draft**, not byte-verified; original unavailable |
| GDD/TDD | 0.3 | — | **MISSING — blocker B-1** |
| Voxel Architecture | 1.0 | — | **MISSING — blocker B-1** |
| Geometry Processing Specification | — | — | **MISSING — blocker B-1** |
| Geometry Editing Specification | 1.0, rev 1.1 | `docs/architecture/geometry-editing-specification-v1.0.md` | Present |
| Cube Agent Development Specification | 1.0, rev 1.1 | `docs/specifications/cube-agent-development-specification-v1.0.md` | Present |
| Geometry Editing Agent Task Pack | 1.0, rev 1.1 | `docs/agent-tasks/geometry-editing-agent-task-pack-v1.0.md` | Present |
| Geometry Editing Integration & QA Pack | 1.0, rev 1.1 | `docs/specifications/geometry-editing-integration-qa-pack-v1.0.md` | Present |
| API Contracts (Draft 0.1) | 0.1.0-draft | `docs/api-contracts/` | Present, unapproved |
| Audit Resolution Report | — | `docs/decisions/audit-resolution-report.md` | Present |

Documents that cite the missing sources (Geometry Editing Specification, Agent Spec §2, Task Pack §1) do so by name only. Anything those sources decide that is not repeated in AU-1.0 or the present documents is **unknown to the repository** and must not be assumed.

## Technology Decision and Spike

| Item | Location | State |
|---|---|---|
| Owner decision: Rust runtime + Godot 4.x first client | `docs/decisions/owner-decision-rust-godot.md` | Recorded; contracts remain Draft |
| Experimental implementation (branch `spike/rust-godot`) | `crates/`, `godot-client/` | Experimental, not approved for `master` |
| Spike README, results, findings | `docs/spike/rust-godot/README.md` | Present |
| Interactive client check (Godot window, timings, screenshots) | `docs/spike/rust-godot/interactive-client-report.md` | Present; desktop only |
| Mobile validation (Android emulator; iOS blocked) | `docs/spike/rust-godot/mobile-validation-report.md` | Present; no real-device results |
| Build and platform targets | `docs/spike/rust-godot/build-and-targets.md` | Present; Android and iOS unverified |
| Rust Core to client adapter boundary | `docs/spike/rust-godot/core-adapter-boundary.md` | Present |
| World generation spike (branch `spike/rust-world-generation`) | `docs/spike/rust-world-generation/README.md`, `report.md` | Experimental; not for `master` without owner decision |

## Architecture Roadmap and Reference Study (branch `docs/architecture-roadmap-v1`, Proposed)

| Item | Location | State |
|---|---|---|
| Approved Owner Decisions AOD-01…24 (AOD-20…23 Approved and AOD-24 Conditionally Approved on 2026-11-05) | `docs/decisions/owner-architecture-decisions-v1.md` | Recorded from the owner's instructions (2026-11-04, 2026-11-05); contracts, ACR-001/002/003 and AU-1.0 untouched; ACR-004 Pending |
| WorldGen / Core boundary: options A, B, C; C3 Approved (AOD-20), open items listed | `docs/decisions/worldgen-core-boundary.md` | direction Approved; layer model, revision rule, version policy open |
| Owner Approval Matrix: eight decisions with options, impacts, blockers and the recorded outcome (OAM-01/02/03/07 Approved, OAM-08 Conditionally Approved, OAM-04/06 Proposed, OAM-05 Spike Required) | `docs/decisions/owner-approval-matrix-v1.md` | updated 2026-11-05; decides no contract, ACR or format |
| Draft contracts WG-xx, SC-xx, WP-xx, SR-xx (WorldGen, system commit, world persistence, server runtime) | `docs/api-contracts/worldgen-contracts.md`, `system-commit-contracts.md`, `world-persistence-contracts.md`, `server-runtime-contracts.md` | Draft; Proposed; nothing approved |
| ACR-004 System-commit boundary (Architecture Change Request) | `docs/decisions/audit-resolution-report.md` (Architecture Change Requests) | Pending |
| System boundaries (processes, trust zones, write paths, data ownership) | `docs/architecture/system-boundaries-v1.md` | Proposed |
| Roadmap with gates 0-10, critical path, blockers | `docs/architecture/roadmap-v1.md` | Proposed; budgets provisional |
| Reference comparison (Veloren, CubeWorld-Reversal, Cube) | `docs/research/reference-comparison.md` | Research record |
| Veloren notes (read-only, GPL-3.0, no code copied) | `docs/research/veloren-notes.md` | Research record |
| CubeWorld-Reversal notes (listings only, legal caveats) | `docs/research/cubeworld-reversal-notes.md` | Research record |
