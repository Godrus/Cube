# Owner decision review and M2-T critical path optimisation: overview

**Status:** review for the Project Owner, **Proposed**. It decides nothing, confirms nothing, authorises no implementation, merge, commit, push, branch or PR. PS-0 is **not** complete; PS-1 is **not** authorised; no owner decision is approved (no registry exists, no commit is signed, all 39 branch commits share one identity).
**Verified state (2026-11-08):** branch `docs/architecture-roadmap-v1`, HEAD `a8584cdd8a6377fbec1a5e8d2d8a6d06c54d4609` = `origin`; working tree: only the owner's ` M .gitignore`; `master` = `bf60534` (13 files, 0 behind / 39 ahead, fast-forward, no conflicts); no `.github/`; `master` unprotected; decision registry absent; previous package committed as `docs/archive/m2-delivery-review/` (the generator `_build/` is not committed). The verified state above is the state when the review was written; the eleven documents were then saved into this folder `docs/archive/owner-decision-m2t-optimization/` (not committed). The graph generator (`_build/`) is not saved.

## Documents

| File | Content |
|---|---|
| `OWNER_DECISION_REGISTER.md` | consolidated register: cards for ODM-01, 02, 03, 09, 10, 11, 12, 16, 17, GOV-01#14, #15, the Core batch; table for the rest; corrections of earlier labels |
| `OWNER_CONFIRMATION_PROTOCOL.md` | authorship, signature, confirmation and traceability kept apart; registry template; confirmation acts E1-E4; process; enforcement |
| `M2T_CRITICAL_PATH_REVIEW.md` | baselines, the local authoritative runtime model, NET/SRV classification, task-by-task review, changes OPT-01...OPT-08 |
| `M2T_OPTIMIZED_TASK_GRAPH.md` | complete v2 graph (Pre 17, M2-T 74, M2 27), change table, validation |
| `M2T_TASK_CLASSIFICATION.md` | the 75 M2-T tasks classified (+2 rows) |
| `M2T_ACCEPTANCE_CRITERIA.md` | AC-01...AC-16, measurable, no invented thresholds |
| `PROVISIONAL_CONTRACT_REVIEW.md` | TC-01...TC-11 reviewed against the Draft contracts |
| `DOCUMENTATION_INTEGRATION_PLAN.md` | D1-D4 files, dependencies, script failures with tested portable fixes |
| `IMPLEMENTATION_READINESS.md` | the governing rule, requirements A-D, gate table R0-R8, M2-T0 |
| `OWNER_ACTION_PLAN.md` | decide-first list, later list, sequence |

## Main findings

1. **The previous M2 graph was incomplete** (my own error): 11 of its 75 M2-T tasks were not reachable from the acceptance task, and reload after restart was impossible in it because recovery (`PERS-06`) had been deferred to M2. Corrected baseline B1: 79 tasks, strict chain 20.
2. **Optimised graph v2:** 74 M2-T tasks, strict chain **18** (24 -> 22 with the proposed rule R-MERGE-G1), **76 fewer dependency edges (-18 %)**, wider parallelism (15 against 12). The chain is now bound by persistence and the runtime sequence, not by Processing or NET. Task-count and chain changes are reported separately; the numbers are counts of tasks, **not durations** (none exist).
3. **Local authoritative runtime is compatible with AOD-22 and the Draft server contracts** if the interface is message-shaped: `SRV-01` and `SRV-03` are required (they *are* the runtime), `NET-03` is a thin in-process binding, `NET-01` a compatibility contract, `NET-02`, `NET-04`, `SRV-02`, `SRV-05...08` and `CLI-05` are multiplayer work. Removing `NET-01`/`NET-03` altogether gains nothing in the metric (computed) and would add a second access path.
4. **No exception to the PS-0 rule exists** in the plan (searched). The legitimate route is an owner-approved amendment of the ADG (ODM-14, ODM-17), which also allows `CORE-01` to start after `GOV-01...GOV-06` while `TEST-01`/`XT-01` finish in parallel.
5. **The smallest production milestone** is proposed as M2-T0 (headless authoritative world): 57 tasks, chain 16, no client, adapter or gameplay owner needed.
6. **Documentation integration:** the two failing audit scripts fail for fixable reasons (one compares a whole directory with an old commit and **passes falsely in a shallow clone**; one encodes the owner's local state); the corrected copies pass in a clean clone (29/0 and 23/0) and fail closed in a shallow one. Tree-based PRs make the hashes cited in documents dangling on `master`: keep and tag the branch.
7. **No owner decision can currently be confirmed** under plan 3.3; the protocol proposes E2/E3 and a separate agent identity.

## Verdicts

| Verdict | Value | Evidence and conditions |
|---|---|---|
| `OWNER_DECISIONS_READY_FOR_REVIEW` | **YES** | register complete for the named decisions and the Core batch, each with primary sources, options, consequences, recommendation, blocking effect and latest safe point; none marked approved. Condition: the owner reads it; the `ODM-` labels are not primary sources (stated) |
| `OWNER_CONFIRMATION_PROTOCOL_READY` | **CONDITIONAL** | the protocol is complete as a design; it takes effect only after the owner chooses the evidence type (ODM-10), creates the registry and (for E2) separates the agent identity |
| `M2_T_CRITICAL_PATH_OPTIMIZED` | **CONDITIONAL** | computed reduction against the honest baseline: chain 20 -> 18, edges -76, tasks 79 -> 74; conditions: ODM-14 (ADG amendments), ODM-16 (runtime model), ODM-07 (temporary contracts); the strict chain is bounded below by governance, Core and persistence |
| `M2_T_TASK_GRAPH_READY_FOR_OWNER_REVIEW` | **YES** | nine generator checks pass (acyclic, all dependencies exist, M2-T closed under dependencies, every M2-T task reachable from `XI-02`, no lower layer depends on gameplay, ADG unchanged); classification complete. Owners for 18 existing and 29 new tasks remain open |
| `M2_T_ACCEPTANCE_READY` | **CONDITIONAL** | AC-01...AC-16 defined with test method, environment, implementation and evidence; thresholds for AC-13/AC-14, the declared platform and the evidence location are pending decisions by design |
| `PROVISIONAL_CONTRACTS_READY` | **CONDITIONAL** | TC-01...TC-11 reviewed; gaps found (dense index order missing in TC-02, pin versus copy and per-brick not chosen in TC-01, no local-authority contract before TC-11); none is registered; two long-term counterparts do not exist (TC-05, TC-06) |
| `DOCUMENTATION_INTEGRATION_READY` | **CONDITIONAL** | corrected plan with exact files, dependencies (order D1 -> D2 -> D3 -> D4) and tested script fixes on copies; the fixes are not applied, status documents are not refreshed, the owner has not approved ODM-01, ODM-10 |
| `PRODUCTION_IMPLEMENTATION_READY` | **NO** | gates R0 (decisions), R1 (base and CI), R2 (Core decisions, P0-alpha, coordinate decisions) are open; PS-0 0 of 8 |

## What was verified in this session and what was not

Verified: Git state and relations; GitHub settings through the public API (earlier in the session: `master` unprotected, 0 workflows, no licence); commit signing status and authorship; the committed ADG, graph generator and previous package; primary sources for each decision (plan 3.3, section 14, AOD-22, AOD-24, SR-05...SR-10, WP-08, PR-01...); the audit scripts in a **fresh clone at `a8584cd`** (original) and on patched copies (full and shallow); the generator checks for v2; reachability and chain figures.
Not verified: any CI execution (none exists); effort, duration or staffing; the content of the Geometry Processing Specification and the Voxel Architecture (absent); the AOD records against their chat primary sources; GitHub secrets and Actions permissions; behaviour on devices.

## Final recommendation

1. **Decide first:** ODM-10, ODM-01, ODM-09, GOV-01 items 14 and 15; then the Core batch with ODM-11 and ODM-03.
2. **Prepare for integration:** fix the two scripts and refresh the status documents in one docs-only commit; then D1, D2, D3, D4 in order; keep and tag the branch.
3. **After approval, start technically:** Core wave W1 (and `SPK-PERS`, `GM-02` as experiments); then Processing, API, orchestration, backend, persistence, generation, runtime to M2-T0.
4. **Smallest production milestone:** M2-T0, the headless authoritative world (57 tasks, 16-task chain).
5. **Explicitly outside M2-T:** gameplay, journal and compaction, second terrain variant, wire protocol, external transport, sessions, reconnect, permissions, dedicated host, prediction, mobile builds, History, tools, Composite, Resampling, SDF/mesh backends.
