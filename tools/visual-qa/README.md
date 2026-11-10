# Cube Visual QA (experimental)

Asks TypeSafe **Jev 1.14** typed questions about Godot screenshots of Cube. TypeSafe never takes part in world generation, rendering or the game loop: it only judges a picture. Geometry invariants (T-junctions, topology, determinism) stay proven by the Rust tests; nothing here can prove them.

Not part of any required check, not part of CI, not reachable by players. Standard library only (Python 3.10+), no change to the Rust workspace.

## Use

```bash
cd tools/visual-qa

# 1. Dry run (default): builds and checks the request, sends nothing, costs nothing.
python cube_visual_qa.py check scene-smoke screenshot=fixtures/good-1.png
python cube_visual_qa.py check block-edit before=fixtures/edit-before.png after=fixtures/edit-after.png
python cube_visual_qa.py fixtures

# 2. Live (explicit): needs TYPESAFE_API_KEY in the environment, one request per image.
#    Key: put it after TYPESAFE_API_KEY= in tools/visual-qa/.env (git-ignored; copy .env.example if missing),
#    or set it in the shell: $env:TYPESAFE_API_KEY = "<your key>". The environment wins over the file.
#    Paste it yourself; never in chat or a tracked file.
python cube_visual_qa.py check scene-smoke screenshot=fixtures/good-1.png --live
python cube_visual_qa.py fixtures --live --max-requests 5

# Offline tests (no network, no key)
python -m unittest test_visual_qa -v
```

Reports go to `reports/` (git-ignored): `<label>.request.json` (the exact request without image bytes, key redacted), `<label>.response.json` (raw API answer), `<label>.report.json` and `<label>.report.md`. Verdicts: **PASS**, **WARN** (some check unsure), **FAIL** (some check clearly failed), **UNKNOWN** (dry run, API error, missing answers; never a PASS).

## Questionnaires (`questions/`)

| name | images | questions |
|---|---|---|
| `scene-smoke` | 1 screenshot | 4 Noul (ground visible, one flat colour, large hole, tears) + 1 Choice (scene kind) |
| `terrain-integrity` | 1 screenshot | Noul for holes and streaks and grass and placeholder magenta, plus a Box for the hole, asked together with its presence Noul and shown only when that Noul says yes |
| `block-edit` | before and after, same camera | Noul for a different view, a notch in `after`, none in `before`, large changes far from the centre, plus a gated Box |

The truth table lives in code: each question has an `expect` value that is **never sent** to the model. Noul: `>= 0.7` agrees with the expectation, `<= 0.3` contradicts it, between is "unsure". Choice: the expected option on top with at least 0.5 passes. These thresholds are the skill's starting values and are experimental.

## Fixtures (`fixtures/`)

`good-1.png` (seed 7) and `good-2.png` (seed 11) are real Godot screenshots (`godot --path godot -- --screenshot ...`). `black.png`, `no-terrain.png` and `missing-chunk.png` are made by `python make_fixtures.py` from `good-1.png` and are byte-identical on every run; they are **synthetic** defects (edited pixels), not real renderer bugs. `edit-before.png` / `edit-after.png` come from `godot --path godot -- --edit-shots before.png after.png`: one block removed under the crosshair, so the difference is small on purpose. `labels.json` holds the expected verdicts.

## Safety and cost

- No request without `--live` **and** `TYPESAFE_API_KEY`; the key is read from the environment only and never printed or written (a test checks the files).
- One attempt per request, 90 s timeout, no retries, `--max-requests` budget (default 6).
- Required header `TypeSafe-Danger-Zone-Images: true` is sent with every request. Image checks follow the documented limits (PNG, 280..1568 px per side, 78 400..2 458 624 pixels, 16 MiB).
- Only the images you name are sent. Do not send private screenshots.
- Images are an experimental TypeSafe feature (format and limits may change); multi-image comparison is weaker than single-image judgment.

## v2

Results: `V2-RESULTS.md` (calibration and evaluation, per-part verdicts). Design: `V2.md`: questionnaires `questions/v2/`, fixtures `fixtures-v2/` (real Godot pictures with controlled faults, `make_godot_fixtures.py`, labels fixed by `label_fixtures.py` before any request), deterministic checks (`deterministic.py`), `python cube_visual_qa.py v2 --split calibration|evaluation [--live]` (dry run by default) and `python analyze_v2.py <split>`.

## Status

Offline tests pass. One live run was made on 2026-11-10 (5 requests): `results/2026-11-10-first-live/SUMMARY.md`. It caught the black, sky-only and missing-ground images but also flagged both good screenshots (`tears_in_terrain` and `large_hole_in_ground` are unusable as written); the conditional Box was not run live. Five images do not measure the model. `python analyze.py` rebuilds the confusion matrices from `reports/`. `examples/example-synthetic.report.md` is produced from a **synthetic** response (`examples-synthetic-response.json`), not from the model.
