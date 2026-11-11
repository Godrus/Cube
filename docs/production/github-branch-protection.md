# Enabling branch protection (GOV-06 / XT-01 evidence)

**Why this file exists:** agents have no GitHub API credentials in the current environment (no `gh`, no token), and the public API shows `protected: false` for `master` and `dev` and no rulesets. Enabling protection is a repository-admin action. The rules are in `.github/rulesets/protect-dev-master.json`.

## Steps (about two minutes, repository admin)

1. GitHub, repository `Godrus/Cube`: **Settings -> Rules -> Rulesets -> New ruleset -> Import a ruleset**, choose `.github/rulesets/protect-dev-master.json` (take it from the `dev` branch).
2. Check the imported values: Enforcement **Active**; target branches `dev` and `master`; rules: **Restrict deletions**, **Block force pushes**, **Require status checks to pass** with exactly `fmt`, `clippy`, `test (ubuntu-latest)`, `test (windows-latest)`, `deny`, `archcheck` (the job names of `.github/workflows/ci.yml`); bypass list empty. Click **Create**.
3. Feature branches (`prod/*`, `docs/*`) are not covered, so normal work on them is unaffected.
4. Integration into `dev`/`master` then needs commits whose required checks have passed; if GitHub rejects a direct fast-forward push, merge through a pull request (Merge button).

## How it is verified afterwards

The anonymous API must show `GET /repos/Godrus/Cube/branches/dev` -> `"protected": true` (same for `master`) and `GET /repos/Godrus/Cube/rulesets` -> one active ruleset. An agent re-checks this and then marks GOV-06 and XT-01 done in [`ps-0-exit-checklist.md`](ps-0-exit-checklist.md).

## Verification 2026-11-09 (after the first import)

`GET /repos/Godrus/Cube/rulesets` shows the active ruleset `protect-dev-master` (id 26848724), and `GET /rules/branches/{dev,master}` shows `deletion` and `non_fast_forward` applied to both branches. **Mismatch:** `required_status_checks` is applied with an **empty list**, so no check is required. Minimal fix: Settings -> Rules -> Rulesets -> `protect-dev-master` -> Edit -> *Require status checks to pass* -> **Add checks** and select `fmt`, `clippy`, `test (ubuntu-latest)`, `test (windows-latest)`, `deny`, `archcheck` (source: GitHub Actions; they appear in the list because they have already run) -> Save. The JSON file now carries `integration_id` 15368 (GitHub Actions) so a re-import keeps the checks. The bypass list cannot be read anonymously: keep it empty and confirm it on the edit page.
