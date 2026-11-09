# Enabling branch protection (GOV-06 / XT-01 evidence)

**Why this file exists:** agents have no GitHub API credentials in the current environment (no `gh`, no token), and the public API shows `protected: false` for `master` and `dev` and no rulesets. Enabling protection is a repository-admin action. The rules are in `.github/rulesets/protect-dev-master.json`.

## Steps (about two minutes, repository admin)

1. GitHub, repository `Godrus/Cube`: **Settings -> Rules -> Rulesets -> New ruleset -> Import a ruleset**, choose `.github/rulesets/protect-dev-master.json` (take it from the `dev` branch).
2. Check the imported values: Enforcement **Active**; target branches `dev` and `master`; rules: **Restrict deletions**, **Block force pushes**, **Require status checks to pass** with exactly `fmt`, `clippy`, `test (ubuntu-latest)`, `test (windows-latest)`, `deny`, `archcheck` (the job names of `.github/workflows/ci.yml`); bypass list empty. Click **Create**.
3. Feature branches (`prod/*`, `docs/*`) are not covered, so normal work on them is unaffected.
4. Integration into `dev`/`master` then needs commits whose required checks have passed; if GitHub rejects a direct fast-forward push, merge through a pull request (Merge button).

## How it is verified afterwards

The anonymous API must show `GET /repos/Godrus/Cube/branches/dev` -> `"protected": true` (same for `master`) and `GET /repos/Godrus/Cube/rulesets` -> one active ruleset. An agent re-checks this and then marks GOV-06 and XT-01 done in `ps-0-exit-checklist.md`.
