# Cube — Agent Development Specification v1.0

**Document type:** Engineering and AI Agent Development Standard\
**Applies to:** All Cube development agents\
**Status:** Proposed project-wide development standard\
**Revision:** 1.1 — aligned with Architecture Update v1.0 (AU-1.0)

---

# 1. Purpose

Define common development rules for AI agents implementing Cube.

The specification controls:

- Architecture compliance.
- Repository structure.
- Module ownership.
- API contracts.
- Source code quality.
- Testing.
- Git workflow.
- Integration.
- Documentation.
- Reporting.
- Conflict resolution.

All agents must follow this document and the applicable technical specifications.

# 2. Source of Truth

Documents have the following authority:

1. Approved project architecture decisions.
2. GDD/TDD.
3. Voxel Architecture Specification.
4. Geometry Processing Specification.
5. Architecture Update v1.0 (AU-1.0) and later approved architecture decisions.
6. Geometry Editing Specification.
7. Agent Development Specification.
8. Individual Agent Task Specification.
9. Approved API Contracts (`docs/api-contracts/`; contracts in Draft state are not authoritative).

A task prompt cannot silently override an approved architecture document.

If two specifications conflict:

- Stop the affected implementation.
- Identify the conflict.
- Report both conflicting requirements.
- Propose a resolution.
- Wait for approval.

Do not resolve architectural conflicts through assumptions.

# 3. Agent Roles

## 3.1 Integration Owner

Responsible for:

- Architecture consistency.
- Shared API contracts.
- Dependency graph.
- Integration order.
- Pull Request approval.
- Release acceptance.

Only the Integration Owner may approve changes to shared public contracts.

The Integration Owner also owns the **composition root** (wiring of modules and ports) and the dependency matrix. Module owners are listed in AU-1.0 §2 (Core Agent, Processing Agent, Agents A–I, Composite Agent, Resolution Agent). Every module has exactly one owner; a module without an implementation may not be silently replaced by another agent.

## 3.2 Module Agents

Responsible for:

- Their assigned module.
- Implementation within scope.
- Module tests.
- Module documentation.
- Performance measurements.
- Integration readiness.

Agents must not modify another module's implementation without explicit authorization.

## 3.3 QA / Benchmark Agent

Responsible for:

- Automated test infrastructure.
- Regression suites.
- Benchmark datasets.
- Performance reports.
- Stress tests.
- Integration validation.

QA is a test-time dependency only and never a runtime dependency.

# 4. Development Principles

Mandatory principles:

1. Explicit ownership.
2. Strict modularity.
3. Minimal dependencies.
4. No hidden global state.
5. No undocumented coupling.
6. Engine-independent Core.
7. Deterministic behavior where required.
8. Safe asynchronous execution.
9. Versioned data formats.
10. Automated verification.
11. No silent architectural changes.
12. No placeholder implementation presented as complete.

# 5. Repository Structure

The following is a proposed logical structure. Adapt directory names to the actual repository without changing module ownership.

```
Cube/
  docs/
    architecture/
    specifications/
    api-contracts/
    decisions/
    agent-tasks/

  source/
    geometry-core/
      assets/
      layers/
      storage/
      channels/
      revision/
      bounds/
      transform/

    geometry-processing/
      snapshots/
      scheduler/
      dependency-graph/
      derived/

    geometry-editing/
      api/
      orchestration/
      backends/
        voxel/
        sdf/
        mesh/
      tools/
        sculpting/
        boolean/
        resampling/
      history/
      composite/
      persistence/
      networking-adapter/

    engine-adapters/

  tests/
    unit/
    module/
    integration/
    property/
    regression/
    persistence/
    multiplayer/
    stress/

  benchmarks/
    datasets/
    geometry/
    editing/
    storage/
    processing/

  tools/
  samples/
  build/
```

The dependency direction between these directories is fixed by the dependency grid in the Integration & QA Pack §1 and is enforced by CI. Agents must inspect the actual repository before creating directories.

Do not duplicate existing modules or introduce a second architecture tree.

# 6. Module Ownership

Each module must have:

- One primary owner.
- One public API.
- Explicit dependencies.
- Defined input and output types.
- Tests.
- Documentation.
- Acceptance criteria.

Shared code must have an explicit owner.

Do not create shared utility modules merely to avoid making a design decision.

# 7. API Contract Rules

Every public API must specify:

```
Contract
  Name
  Version
  Status (Draft | Proposed | Approved | Implemented | Frozen)
  Owner
  Purpose
  Public Data Model
  Input Types
  Output Types
  Error Types
  Ownership Rules
  Lifetime Rules
  Access Rules
  Threading Rules
  Revision Rules
  Dirty Tracking
  Async Behavior
  Cancellation
  Serialization
  Compatibility
  Tests
  Open Decisions
```

Rules:

- No undocumented public API.
- No public mutable internal storage.
- No raw internal pointers crossing module boundaries unless explicitly permitted by the implementation language and lifetime contract.
- No implicit geometry conversion.
- No direct mutation outside approved editing interfaces.
- No hidden cache invalidation.
- No silent API breaking changes.
- No agent implements against a contract that is not Approved, except in clearly marked prototypes.
- Dependencies between modules must follow the dependency grid; cross-module shared abstractions live in Editing API & Contracts.

# 8. Coding Rules

## 8.1 General

- Follow existing repository conventions.
- Use descriptive names.
- Prefer small, focused modules.
- Avoid unnecessary abstraction.
- Avoid duplicated algorithms.
- Document non-obvious invariants.
- Handle errors explicitly.
- Avoid unexplained magic constants.

## 8.2 Geometry Data

- Respect Layer ownership.
- Respect Brick addressing.
- Respect channel-specific storage.
- Distinguish missing data states.
- Never assume all geometry is dense.
- Never assume all geometry is cubic voxel data.
- Do not treat derived meshes as authoritative geometry.

## 8.3 Memory

- Define resource ownership.
- Avoid unnecessary copying.
- Use immutable snapshots for async jobs.
- Validate buffer lifetime.
- Avoid unbounded caches.
- Track peak memory for expensive operations.

## 8.4 Concurrency

- No uncontrolled shared mutable state.
- No blocking the main game thread with expensive operations.
- Use approved scheduling and cancellation.
- Validate revisions before publishing async results.
- Avoid deadlocks and priority starvation.

# 9. Error Handling

Errors must be explicit and typed. Expected failures use `Result`/`Error` (stable code, context, module, operation id, recovery policy, severity); exceptions are reserved for unexpected critical failures (AU-1.0 §14).

Do not:

- Silently ignore failures.
- Convert errors into success values.
- Hide revision conflicts.
- Return partially modified data without an explicit partial-result contract.
- Continue a transaction after an unrecoverable validation failure.

Every operation must define whether failure:

- Leaves source unchanged.
- Rolls back the transaction.
- Produces a recoverable partial result.
- Requires external reconciliation.

# 10. Testing Rules

Every agent must provide:

1. Unit Tests.
2. Public API Contract Tests.
3. Module Tests.
4. Regression Tests for discovered bugs.
5. Edge-case tests.
6. Performance measurements where applicable.

Property-Based Testing is required for algorithms with meaningful invariants.

Examples:

- Applying an inverse transform restores the original transform within tolerance.
- A failed transaction does not modify authoritative source data.
- Undo followed by Redo restores the committed state.
- Serialization followed by deserialization preserves supported data.
- Stale async results never replace newer revisions.
- Boolean results satisfy operation-specific invariants.

# 11. Performance Rules

Every agent must:

- Identify computational hot paths.
- Measure representative workloads.
- Report allocations and peak memory.
- Measure main-thread blocking.
- Measure scaling.
- Avoid unsupported performance claims.

Benchmarks must include:

- Dataset description.
- Hardware and software environment.
- Build configuration.
- Warm-up policy.
- Number of repetitions.
- Median and percentile measurements where relevant.
- Memory measurements.
- Known limitations.

# 12. Git Workflow

## 12.1 Branches

Suggested format:

```
master
integration
feature/geometry-voxel-backend
feature/geometry-sculpting
fix/geometry-revision-conflict
test/geometry-persistence
```

Branch names may be adapted to repository conventions. The actual default branch of this repository is `master` (remote `origin/master`); "protected main branches" below means `master`.

## 12.2 Pull Requests

Every PR must include:

- Task ID.
- Module.
- Summary.
- Files changed.
- API changes.
- Dependencies.
- Tests executed.
- Benchmark results.
- Known limitations.
- Documentation updates.

## 12.3 Restrictions

Agents must not:

- Push directly to protected main branches.
- Merge their own PR without authorization.
- Rewrite shared history.
- Force-push shared branches.
- Modify unrelated modules.
- Commit generated build artifacts unless required.
- Add dependencies without justification and approval.

# 13. Dependency Management

Before adding a dependency, report:

- Purpose.
- License.
- Platform support.
- Maintenance status.
- Build impact.
- Runtime impact.
- Security considerations.
- Alternative implementations.

No agent may independently introduce a foundational dependency that affects multiple modules.

# 14. Documentation

Each module must contain:

- Overview.
- Responsibilities.
- Public API.
- Data ownership.
- Dependencies.
- Threading model.
- Error handling.
- Tests.
- Performance characteristics.
- Known limitations.

Architecture changes must be reflected in the relevant specification.

# 15. Agent Communication

Agents communicate through:

- Approved task specifications.
- API contracts.
- Pull Requests.
- Technical reports.
- Integration issue reports.

Chat links are supplemental context, not authoritative specifications.

# 16. Architectural Change Requests

An agent proposing a change must provide:

1. Current approved design.
2. Identified problem.
3. Proposed change.
4. Affected modules.
5. API impact.
6. Data compatibility impact.
7. Performance impact.
8. Test plan.
9. Migration plan if needed.

No implementation of a breaking architectural change before approval.

# 17. Required Agent Report

Every completed task must produce:

```
Task ID:
Agent:
Module:
Status:

Implemented:
- ...

Public API:
- ...

Files changed:
- ...

Dependencies:
- ...

Tests:
- ...

Benchmarks:
- ...

Known limitations:
- ...

Unresolved issues:
- ...

Architecture deviations:
- ...

Integration instructions:
- ...

Recommended next steps:
- ...
```

Status values:

- Not Started.
- In Progress.
- Blocked.
- Implemented.
- Ready for Review.
- Accepted.
- Rejected.

Only Integration Owner can mark a module Accepted.

# 18. Definition of Done

A task is complete only when:

- Implementation matches scope.
- API contract is satisfied.
- Required tests pass.
- Documentation is updated.
- No unapproved architectural changes exist.
- Known limitations are documented.
- Integration report is delivered.
- Integration Owner approves acceptance.

**End of Specification.**
