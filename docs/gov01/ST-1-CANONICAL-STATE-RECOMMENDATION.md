# ST-1: where the canonical project state lives (recommendation)

**Status:** **Proposed.** A recommendation to the owner; it changes no file. Needed because `AGENTS.md` (the entry file loaded by the agent harness) mandates a state file that does not exist, while the status package created another.

## 1. Facts (verified at HEAD `64e8a76`)

* `AGENTS.md` exists on disk, is **untracked**, and the owner's uncommitted `.gitignore` change adds `AGENTS.md` to the ignore list. A fresh clone therefore has no `AGENTS.md`.
* `AGENTS.md` instructs agents to read `.agent/README.md`, `.agent/tasks/NEXT_STEPS.md`, `.agent/history/recent.md`, `.agent/rules/*.md`, and to maintain `.agent/state/CURRENT_CONTEXT.md` (fixed sections, ≤ ~150 lines, one active task). None of `.agent/` or `.cursor/rules/` exists in the repository.
* The status package defines `docs/PROJECT_STATUS.md` as the single source of the current state, and `docs/AGENT_HANDOFF.md` as the entry instruction.
* Plan section 3.3 puts decisions into a registry `docs/decisions/decision-registry.md` (not yet created).

## 2. Principle

**One fact, one writer, one place.** Every kind of fact has exactly one authoritative document; all other documents may only *reference* it or contain a *generated* copy that a script can re-check. Git history is evidence, not state.

## 3. Source-of-truth map (recommended)

| Kind of fact | Single source of truth | Written by | Other documents may |
|---|---|---|---|
| Decisions of the owner (approved/amended/rejected, confirmation source) | `docs/decisions/decision-registry.md` (create in GOV-01/02) | owner, or an agent quoting the owner with a confirmation source | cite ids only |
| Contract approval and maturity | `docs/api-contracts/README.md` status columns (two axes) | Integration Owner on a registry entry | cite |
| Definition of tasks, dependencies, gates | ADG: `docs/production/module-boundaries.md` Part C/D + `agent-task-breakdown.md` rows | Integration Owner (changes need owner acceptance) | cite; generated tables only |
| **Execution state of stages, gates, tasks, blockers, next task** | **`docs/PROJECT_STATUS.md`** | the agent finishing work, with evidence rows | `AGENT_HANDOFF.md` and `.agent/state/CURRENT_CONTEXT.md` point to it, never copy it |
| Release milestones: definitions and criteria | `docs/RELEASE_CRITERIA.md` | owner-approved | roadmap cites ids |
| Release milestones: plan and dependencies | `docs/DEVELOPMENT_ROADMAP.md` | Integration Owner | status cites |
| Release milestones: **reached or not** | `docs/PROJECT_STATUS.md` (only after a registry entry of the owner) | agent, on the owner's entry | none |
| How an agent works | `docs/AGENT_HANDOFF.md` | Integration Owner | `AGENTS.md` points to it |
| Audit findings at a date | `docs/audit/*` | the auditing agent; **immutable after commit**, superseded by a new dated audit | status cites finding ids |
| Plan text (Production Plan v2.1) | `docs/production/*` | Integration Owner | cite |
| What happened and when | Git history | — | status cites commit hashes |

Consequences: (1) the progress table in `PROJECT_STATUS.md` 5.4 is the *only* place with per-stage task counts and must stay re-generable (`docs/audit/gen.py`) from the ADG; (2) the roadmap and criteria never contain "done/reached"; (3) `PROJECT_STATUS.md` never holds a decision, only a registry id and its state read from the registry.

## 4. Options for the entry file

| Option | Description | Pros | Cons |
|---|---|---|---|
| **A (recommended)** | `docs/PROJECT_STATUS.md` canonical. `AGENTS.md` becomes a **tracked** short entry (≤ 30 lines): "read `docs/AGENT_HANDOFF.md`, then `docs/PROJECT_STATUS.md`", plus the owner's standing rules. `.agent/state/CURRENT_CONTEXT.md` is created only if the owner wants it, as a ≤ 30-line pointer (baseline commit id + next task id + link) with no duplicated content | no conflicting copy; fresh clones work; keeps the owner's AGENTS.md mechanism | requires the owner to edit `AGENTS.md` (their file) and to change its ignore entry |
| B | Create `.agent/` per `AGENTS.md`; make `CURRENT_CONTEXT.md` canonical; reduce `PROJECT_STATUS.md` to a generated view | follows the existing owner convention to the letter | two heavy formats (`CURRENT_CONTEXT` ≤150 lines vs 5.4 progress table); `.agent/history/recent.md` duplicates Git; more files to keep consistent |
| C | `PROJECT_STATUS.md` only; amend `AGENTS.md` to remove `.agent` references and leave it untracked | least work | an ignored file does not travel with the repository; agents on other machines get no entry |

## 5. Recommended amendments (text only, nothing applied)

1. `AGENTS.md` (owner's file; proposed content): *"Start with docs/AGENT_HANDOFF.md and docs/PROJECT_STATUS.md. The decision registry is docs/decisions/decision-registry.md. Do not commit, push or open PRs without the owner's explicit permission."* Remove the `.agent/...` list, or mark it "not used".
2. `.gitignore`: remove the `AGENTS.md` entry if the file is to be tracked (the owner's change; not touched here).
3. If `CURRENT_CONTEXT.md` is kept: fixed headings of AGENTS.md (PROJECT STATE, ACTIVE TASK, ARCHITECTURAL DECISIONS, CHANGED FILES, NEXT STEP); each section a pointer line to `PROJECT_STATUS.md`; update it at the same time as the status; `val.py` asserts its baseline commit equals the status file's.
4. Extend `docs/audit/val.py` to check: the baseline commit named in `PROJECT_STATUS.md` is an ancestor of `HEAD`; every decision id cited in the status exists in the registry; no document outside the registry uses "Owner-Approved" as a status without an id.

## 6. Interaction with the earlier rule about commits

The status file separates the **verified baseline**, the **commit holding the status documentation** and the **current HEAD** (never written in a file). Keep that: it avoids the endless "update HEAD" commit chain.

## 7. Open points for the owner

* Choose A, B or C.
* Whether `AGENTS.md` should be tracked (your `.gitignore` change says no).
* Confirm the registry path `docs/decisions/decision-registry.md` (plan 3.3) over `docs/production/decision-registry.md` (used in earlier status documents; to be corrected).
