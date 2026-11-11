# Owner decision confirmation protocol (proposed)

> **Superseded 2026-11-09** by DEC-002 in `docs/decisions/decision-registry.md` (ODM-10 closed). Kept as history; the registry rules there are the only process. Acts E1-E3 remain valid evidence types; chat confirmation by the owner is now also an accepted source.

**Status:** **Proposed** (`PENDING_OWNER_DECISION` as ODM-10). Nothing here is in force until the owner adopts it. It describes a mechanism; it confirms no decision.
**Basis (verified 2026-11-08):** plan 3.3 (`docs/production/production-architecture-plan.md`): a registry entry needs a *confirmation source*, accepted sources are "(a) an owner-signed commit or pull-request approval in the repository; (b) a decision text written by the owner in the repository"; "a chat instruction alone is not a confirmation source". `owner-architecture-decisions-v1.md` section 4 instead accepts the owner's "explicit approval in chat or in a signed-off pull request" to *change* an AOD (a difference already recorded as CR-01 in GOV-01). The registry file does not exist (`docs/decisions/decision-registry.md` absent; no file named `decision-registry` is tracked).

## 1. What the repository can and cannot prove today

| Fact | Evidence | Consequence |
|---|---|---|
| Every commit on the branch (39 incl. `a8584cd`) is authored under one Git identity | `git log master..HEAD --format=%an` | **author metadata cannot show who decided**: it is set by local configuration and is the same for the owner and for agent sessions working in the owner's checkout |
| No commit is cryptographically signed | `git log --format=%G?` = `N` for all 39 | the plan's "owner-signed commit" is not satisfiable by the existing history |
| `Co-Authored-By` trailers name the model | commit messages | informational only; trailers are free text |
| The repository is public; GitHub records PR reviews, comments and web-UI commits under the authenticated account | GitHub model (not re-verified here) | the platform, not Git metadata, authenticates the actor |
| Branch protection does not exist | GitHub API | nothing forces a review today |

## 2. Four things that must not be confused

| Concern | Question it answers | Evidence type | Role in this protocol |
|---|---|---|---|
| **Authorship** | which identity is written in the commit | `user.name`/`user.email` | none; **never evidence of a decision** |
| **Cryptographic verification** | was the commit made with a given private key | GPG/SSH signature, GitHub "Verified" badge | integrity of *who held the key*; it says nothing about *who decided* when a key is available to agent sessions |
| **Owner decision confirmation** | did the Project Owner choose this option | an act that only the owner can perform (section 4) | **the binding act** |
| **Historical traceability** | what was decided, when, on what basis, and what replaced it | the append-only registry plus Git history | audit |

## 3. The registry (single place for decisions)

File `docs/decisions/decision-registry.md` (path to be confirmed by the owner; the plan names it, several status documents name `docs/production/decision-registry.md`). **Append-only**: an entry is never rewritten; a change is a new entry that supersedes it.

Entry fields (all mandatory unless marked optional):

| Field | Content |
|---|---|
| `id` | stable, never reused: `DEC-nnn` for new decisions; existing labels (AOD-nn, OAM-nn, ACR-nnn, GOV-01#n, ODM-nn) are kept in `aliases` |
| `question` | the exact question |
| `options` | options considered, with a pointer to the preparation document |
| `selected` | the chosen option, or `Amended: ...` with the amendment text |
| `rationale` | the owner's reason in the owner's words, or "as recommended in <document>" |
| `status` | `PENDING_OWNER_DECISION`, `PENDING_OWNER_CONFIRMATION` (recorded by an agent from chat or from a recommendation), `CONFIRMED`, `DEFERRED` (with the trigger), `SUPERSEDED`, `REJECTED` |
| `decided_on` | date |
| `confirmed_on` | date of the confirmation act |
| `confirmation_evidence` | type and reference: E1 signed commit hash and key id; E2 PR number plus review id plus merge commit; E3 web-UI commit hash by the owner account; E4 none (then the status cannot be `CONFIRMED`) |
| `recorded_by` | `owner` or `agent` (and the session) |
| `affected_documents` | paths |
| `affected_tasks` | ADG and proposed-graph ids |
| `supersedes` / `superseded_by` | ids |
| `source` | primary source document and section |

## 4. Confirmation acts (strongest first)

| Code | Act | Authenticated by | Strength | Practical notes |
|---|---|---|---|---|
| **E2** | the owner **approves a pull request** that contains the entry (GitHub review "Approve") | GitHub account of the reviewer | strong; the review record is on the platform, not editable by an agent without the owner's token | GitHub forbids approving one's own PR: if agents push with the owner's account, the owner cannot approve; agent sessions need a distinct account or token (a bot identity), or the owner uses E3 |
| **E3** | the owner makes a **confirmation commit through the GitHub web UI** that changes only the `status` and `confirmation_evidence` fields of listed entries | GitHub web commit signed by GitHub, account of the committer | strong enough; one small commit per batch | no local key required |
| **E1** | **owner-signed commit** (SSH or GPG key that the owner registers in GitHub and in the registry) | key possession | strong only if the key cannot be used by agent sessions (hardware key requiring a touch, or a passphrase the agent does not know) | otherwise it proves key access, not a decision |
| E4 | chat statement transcribed by an agent | none | **not a confirmation**; the entry stays `PENDING_OWNER_CONFIRMATION` | matches plan 3.3 |

**Signed commits: necessary, optional or insufficient?** *Optional* as a way to confirm, *insufficient by itself* (a signature identifies a key holder, not a decision, and not the person if the key is reachable by agents), and *not necessary* if E2 or E3 is used. They are valuable for integrity and for a later move to a separate agent identity.

## 5. Process with the least overhead

1. An agent prepares entries as `PENDING_OWNER_DECISION` (question, options, recommendation) in a registry PR. Preparation documents already exist for most decisions.
2. The owner decides in the PR (editing the `selected` and `rationale` fields, or commenting the choice) and approves the PR (E2) or confirms by web-UI commit (E3). One PR per batch (for example "Core batch", "CI and branch batch").
3. A merge commit then carries the confirmation reference; `confirmed_on` and the evidence fields are filled in the same PR.
4. Decisions recorded earlier from chat (AOD-01...24, OAM records) are moved into the registry as `PENDING_OWNER_CONFIRMATION` and confirmed **in bulk** by one E2 or E3 act listing the ids (X-1 of GOV-01); the owner may confirm, amend or reject each id.
5. Revisions: a new entry with `supersedes`; the old entry gets `superseded_by` in a confirmation commit. Nothing is deleted.

## 6. Enforcement (optional but cheap, needs the CI of GOV-06)

A `registry-lint` check (Python, no dependencies) that fails when: an entry has `status: CONFIRMED` without evidence; an ID is reused or removed; a field is missing; a status moves to `CONFIRMED` in a commit that also changes other files; a `CONFIRMED` entry is edited other than by adding `superseded_by`; a document or task cites a decision id that does not exist. Verification of the evidence type (merge of an approved PR, or "verified" web commit by the owner account) can use the GitHub API in the workflow; this is not implemented here.

## 7. What is not claimed

* That any listed ODM, GOV-01 item, AOD or OAM record has been confirmed: **none has**.
* That signing would have prevented the present situation: the owner's identity is used by agent sessions; the root fix is a separate identity or token for agents (owner action), not a policy document.
* That this protocol is necessary before PS-0 can close: it is needed for the *entries* of GOV-01/GOV-02 to count under plan 3.3; the owner may choose a lighter act (E3) for the first batch.

## 8. Adoption steps (all owner-gated)

| Step | Action | Needs |
|---|---|---|
| 1 | confirm the registry path and the entry template | ODM-10 |
| 2 | choose evidence types (recommended: E2 or E3; E1 optional) | ODM-10 |
| 3 | create the registry file with the template and the ids to be decided (an agent can prepare it) | permission to commit |
| 4 | optional: separate agent identity (account or token) so that E2 is possible | owner action on GitHub |
| 5 | optional: CODEOWNERS for the registry path; `registry-lint` as a required check | GOV-05/GOV-06 |
