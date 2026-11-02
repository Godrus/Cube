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
