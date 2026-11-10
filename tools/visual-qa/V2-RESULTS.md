# Visual QA v2: calibration and evaluation results (2026-11-10)

Experimental. Counts are the result; the samples are small (a handful of defective pictures per kind), so no strong conclusion follows from any percentage. Everything below is reproducible from `results/` with `python report_v2.py <split>`.

## Setup and accounting

- Model `jev-1.14.0`; price from the docs: **$0.042 per million input tokens, output free**. Thresholds fixed in advance: yes >= 0.7, no <= 0.3, in between = unsure.
- Labels committed before any v2 request (`8988b36`). Calibration used first; questionnaires, thresholds, labels and report code frozen afterwards (`FROZEN.json`, bundle hash `66b62b76...6132`, verified unchanged after the evaluation run). **No question or threshold was changed after calibration**: with 9 answered items and the gap scores overlapping (good 0.30 to 0.63, defective 0.60 to 0.85) any edit would have been fitting noise, and the single request left in the calibration budget could not have tested it.
- Evaluation split run **exactly once**, nothing retried.

| stage | attempts | answered | HTTP 529 (service overload) | input tokens (answered) | output tokens |
|---|---|---|---|---|---|
| Box check (earlier) | 2 | 1 | 1 | 2,910 | 122 |
| calibration | 11 (+1 answer reused from the Box check) | 8 | 3 | 30,046 | 733 |
| evaluation | 14 | 14 | 0 | 49,758 | 1,384 |
| this stage (budget 26) | 25 | 22 | 3 | 79,804 | 2,117 |

Cost of the answered requests this stage: about $0.0034 (all stages since v1 about $0.0041). Whether a 529 is billed is not documented; nothing indicates it is. Infrastructure errors are not model errors: 3 calibration items (`seed101-good`, `seed102-wrong_material`, `seed104-missing_brick`) have no answer and were neither retried nor replaced; the only calibration `wrong_material` picture is among them, so that defect was not calibrated at all.

## Calibration (9 answered pictures: 4 good, 5 defective incl. the Box-stage item; 3 infra errors)

| check | TP | FP | TN | FN | unsure |
|---|---|---|---|---|---|
| `terrain_visible` (truth always yes) | 3 | 0 | 0 | 0 | 3 |
| `scene_kind` (expected class on top) | 6 correct | | | | 0 |
| `gap_in_ground` | 1 | 0 | 1 | 0 | 4 |
| `off_palette_patch` | 0 | 0 | 4 | 0 | 2 |
| `block_missing_in_after` | 0 | 0 | 2 | 0 | 1 |

Observations: no false positive and no false negative at the thresholds, but many unsure answers. On normal terrain `terrain_visible` stayed at 0.51 to 0.67 for 3 of 6 pictures. The two `hidden_area`/`missing_brick` pictures at 0.60 and the removed block at 0.49 were not detected.

## Evaluation (independent: 4 good, 6 defective scene pictures, 4 edit pairs)

Scene pictures:

| check | TP | FP | TN | FN | unsure |
|---|---|---|---|---|---|
| `terrain_visible` | 6 | 0 | 0 | 0 | 4 |
| `scene_kind` | 10 correct | | | | 0 |
| `gap_in_ground` | 4 | 0 | 0 | 0 | 6 |
| `off_palette_patch` | 2 | 0 | 6 | 0 | 2 |
| `block_missing_in_after` (edit pairs) | 0 | 0 | 3 | 0 | 1 |

Per defect kind (the check that should fire; infra errors none in this split):

| defect | pictures | fired | unsure | missed |
|---|---|---|---|---|
| missing_brick (`gap_in_ground`) | 2 | 2 | 0 | 0 |
| hidden_area (`gap_in_ground`) | 2 | 2 | 0 | 0 |
| wrong_material (`off_palette_patch`) | 2 | 2 | 0 | 0 |
| edit really applied (`block_missing_in_after`) | 1 | 0 | 1 (0.58) | 0 |
| edit not shown (stale) | 3 | 0 | 0 | 3 correct "no" |

False positives: **none at the thresholds**. Closest calls on good pictures: `seed105-good` gap 0.60, `seed106-good` gap 0.56, `seed108-good` gap 0.51; all four good pictures were "unsure" for `gap_in_ground`, none reached "no" (<= 0.3). So specificity is not demonstrated, only that the good pictures stayed below the 0.7 line, by a margin of 0.1 to 0.25 (and calibration showed defective pictures at 0.60). False negatives: none confident; the one real edit (0.58) and the calibration edit (0.49) were unsure.

Box (shown only when its presence check is >= 0.7; compared with the pixel-difference box of the defect): evaluation IoU **0.95, 0.54, 0.49, 0.23** (true boxes cover 25 %, 3 %, 13 %, 57 % of the picture; confidences 0.72, 0.66, 0.21, 0.40); calibration IoU 0.92 (39 %). No Box was shown for a good picture.

Deterministic checks (plain code, no API), both splits together: partial load (loaded bricks below 95 % of the reference): **4 of 4** found; stale update (edit accepted, pixels identical): **5 of 5** found; false alarms on good pictures and on real edits: **0 of 10**; the checks did not fire on the other fault kinds (11 of 11 quiet).

## Verdict per part

| part | status | why |
|---|---|---|
| deterministic checks | **suitable for experimental monitoring** | exact on the faults they target; found every partial load and stale update, zero false alarms; also covers what v1 wrongly asked the model (black or flat frames, size, batches) |
| TypeSafe `scene-smoke` | **did not demonstrate value** | `scene_kind` was right on 16 of 16 normal landscapes but v2 has no non-landscape picture, and blank or black frames are found by code; `terrain_visible` was unsure on 7 of 16 normal pictures |
| TypeSafe `terrain-integrity` | **suitable for experimental monitoring, advisory only** | missing and hidden geometry and a wrong material: 8 of 8 answered evaluation and calibration defects with a gate-eligible score fired or (calibration, 2) stayed unsure; no false positive at 0.7 on any good picture; but about half of the good pictures sit in the unsure band for the gap check and the margin is thin |
| TypeSafe `block-edit` | **did not demonstrate value** | the 2 real one-block edits were missed (0.49, 0.58) even from an enlarged crop; the stale pairs it answered correctly are found exactly by pixel comparison |
| TypeSafe Box | **needs rework** | IoU from 0.23 to 0.95 on 5 gated boxes, no obvious relation to its confidence; useful as a rough region for large gaps, not as a measurement; not for production QA |

## What this says about TypeSafe for Cube

Narrow, real value: judging a *visual meaning* that is awkward to code, namely "ground is missing here" and "this block has the wrong colour", on a normal Cube picture without being fooled by steps and sharp cube edges (the v1 failure). It was cheap (about $0.0004 per picture) and made no confident mistake on this data. It adds nothing for what a program can check exactly (blank frames, brick counts, stale updates, pixel changes), and it did not see a single removed block. It should stay an optional, advisory, non-blocking report; it is **not** a candidate for required CI, and with 15 defective pictures the evidence is thin and both the missed calibration items and the unsure band say the margin is small.

## Next steps (not done)

1. Rework `gap_in_ground` and `terrain_visible` with a *new* calibration set (the current evaluation set is now used).
2. Add non-landscape and real-renderer-bug pictures to test `scene_kind` and the gap check on cases that are not injected by the scene script.
3. Drop `block-edit` from TypeSafe unless a different framing (for example a larger edit) is shown to help; keep the pixel comparison.
4. Keep Box out of QA until localisation is shown on small defects.
