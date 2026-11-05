# PROJECT-STATUS-VALIDATION

Proof that the statuses, references, tasks and dependencies in the package are correct. Check date 2026-11-05. Repository HEAD `2c0c1f3`. The repository was not modified; the one command that writes (`cargo test`) writes only to the git-ignored `target/`; `git status` before and after: only ` M .gitignore`.

## 1. Facts and commands

| Statement in PROJECT_STATUS | Command / source | Result |
|---|---|---|
| branch `docs/architecture-roadmap-v1`, HEAD `2c0c1f37…` | `git rev-parse` | matches |
| `2c0c1f3` touches only `docs/production/` (9 files, +476/-134) | `git diff --name-only 2c0c1f3~1 2c0c1f3 \| grep -v ^docs/production/` | empty output |
| tree not clean: `.gitignore` +`AGENTS.md` | `git diff .gitignore` | matches |
| `origin` branch = HEAD | `git log origin/docs/architecture-roadmap-v1 -1` | `2c0c1f3` |
| 33 commits / 259 files ahead of `master` | `git rev-list --count master..HEAD`; `git diff --shortstat master HEAD` | 33; 259 files |
| no CI | `ls .github` | not found |
| no production crates/xtask | `ls`, `git ls-files` | only the 7 spike crates |
| `.agent`, `.cursor` absent | `ls -a` | absent |
| spike tests | `cargo test --workspace --offline` | 131 passed, 0 failed |
| `gh` unavailable (branch protection unknown) | `gh --version` | command not found |

## 2. Independent ADG reproduction

Script `ia.py` (parses only the published tables of `module-boundaries.md`; shipped with this package). Output:

```
tasks 147 gates 15
unknown refs []
stage inversions []
multi-gate [] ungated ['DEV-01', 'SPK-NET', 'SPK-PERS']
G1 ancestors outside PS0/1: [] EXT in G1: []
G2 anc of ORCH-*  True
G2 anc VOX-01,SDF-01,MSH-01,SRV-01,API-02 [True, True, True, True, True]
PS-1 owners {'Core Agent': 11, 'Agent I (QA and Benchmarks)': 3, 'Main Agent / Integration Owner': 2}
stages PS-0:11 PS-1:16 PS-2:12 PS-3:6 PS-4:7 PS-5:14 PS-6:7 PS-7:5 PS-8:6 PS-9:5 PS-10:14 PS-11:9 PS-12:10 PS-13:17 PS-14:8 (sum 147)
tasks missing in breakdown []   /   breakdown ids not in ADG []
gate chain: G1 ancestor gates [G0]; G2 [G0]; G3 [G0,G2]; G4 [G0,G2,G3]; G10 [G0,G2,G3,G4];
            G14 [G0,G2,G3,G4,G10,G13]
```
Original validator (`production-plan-v2.1/_build/validate.py`): `tasks: 147 gates: 15 external: 3 ... errors: 0`. Reproduced.

**Conclusion:** the graph is well formed (no cycles, no unknown references, no stage inversions) **and** it has the semantic gap D-01/D-02 (stage gates do not chain; G1 and G5–G9, G11, G12 are not ancestors of G14). Evidence: the last two lines above.

## 3. Package self-checks

| Check | Result |
|---|---|
| every task/gate/external id mentioned in the five new documents exists in the ADG | PASS (`val.py`: 0 unknown ids; 24 / 142 / 20 / 6 / 2 ids checked) |
| no `PS-n`/`Gn` above 14 | PASS |
| no unfilled `{{placeholders}}` | PASS (after table generation) |
| relative repository paths in backticks exist | PASS except two **intended future** files: `docs/PROJECT_STATUS.md` and `docs/production/decision-registry.md` (do not exist yet; the documents say so) |
| stage table in PROJECT_STATUS generated from Part D, not hand-typed | `gen.py`; 15 rows; member counts 8,16,12,6,7,14,7,5,6,5,14,9,10,17,8 |
| no status stronger than evidence | no task is marked Implemented/Verified/Accepted; no milestone Reached; every decision in 5.5 B shows "not confirmed" |
| separation of task status and contract status | `AGENT_HANDOFF.md` section 6 keeps two systems; contract approval and maturity axes separate |
| milestones vs stages not 1:1 | `DEVELOPMENT_ROADMAP.md` section 1, 2 |
| no invented dates or numeric budgets | numbers appear only as `[B-n]`; R0.2 procedure |

## 4. Not verified (Unknown)

* GitHub branch protection / rulesets / Actions settings (no `gh`).
* Whether `AOD-01…19` are complete relative to the owner's intent.
* The truth of statements that depend on the absent GDD/TDD, Voxel Architecture, Processing Spec and original AU-1.0.
* Correctness of the *architectural judgment* behind individual dependencies (only structure was checked).
* The earlier "full technical audit"/"second review" texts were not available.

## 5. Update procedure for this validation

On each status update, re-run: the git commands in section 1, `ia.py` (after fixing D-01/D-02 extend it with the gate-chain check as an assertion), and `val.py`. Record the outputs here with the commit hash.
