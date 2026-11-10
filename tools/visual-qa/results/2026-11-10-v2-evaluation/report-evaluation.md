# Visual QA v2 report: evaluation

## Requests and tokens

- answered: 14; infrastructure errors: 0; not sent in this split: 2
- input tokens (answered requests): 49,758; output tokens: 1384
- price: $0.042 per million input tokens, output free (docs.typesafe.ai/models): about $0.00209 for the answered requests

## scene-smoke-v2

- `terrain_visible`: TP 6, FP 0, TN 0, FN 0, unsure 4
- `scene_kind` (Choice, expected class vs top option): correct 10

## terrain-integrity-v2

- `gap_in_ground`: TP 4, FP 0, TN 0, FN 0, unsure 6
- `off_palette_patch`: TP 2, FP 0, TN 6, FN 0, unsure 2

## block-edit-v2

- `block_missing_in_after`: TP 0, FP 0, TN 3, FN 0, unsure 1

## Per defect kind (the check that should fire)

| defect | items answered | fired (yes) | unsure | missed (no) | infra error |
|---|---|---|---|---|---|
| missing_brick (`gap_in_ground`) | 2 of 2 | 2 | 0 | 0 | 0 |
| hidden_area (`gap_in_ground`) | 2 of 2 | 2 | 0 | 0 | 0 |
| wrong_material (`off_palette_patch`) | 2 of 2 | 2 | 0 | 0 | 0 |
| edit changed (`block_missing_in_after`, truth True) | 1 of 1 | 0 | 1 | 0 | 0 |
| edit stale (`block_missing_in_after`, truth False) | 3 of 3 | 0 | 0 | 3 | 0 |

## Concrete cases (item, check, truth, p)

- FP: none
- FN: none
- unsure: seed105-good gap_in_ground truth=False p=0.6; seed105-good off_palette_patch truth=False p=0.37; seed106-good terrain_visible truth=True p=0.65; seed106-good gap_in_ground truth=False p=0.56; seed106-good off_palette_patch truth=False p=0.39; seed106-wrong_material terrain_visible truth=True p=0.69; seed106-wrong_material gap_in_ground truth=False p=0.6; seed107-good terrain_visible truth=True p=0.65; seed107-good gap_in_ground truth=False p=0.45; seed107-wrong_material gap_in_ground truth=False p=0.38; seed108-good terrain_visible truth=True p=0.65; seed108-good gap_in_ground truth=False p=0.51; edit309-normal block_missing_in_after truth=True p=0.58

## Box (shown only when its presence check is >= 0.7)

- seed105-missing_brick gap_box: presence 0.91, IoU 0.54, confidence 0.66, true box area 0.03 of the picture
- seed105-hidden_area gap_box: presence 0.80, IoU 0.49, confidence 0.21, true box area 0.13 of the picture
- seed106-hidden_area gap_box: presence 0.81, IoU 0.23, confidence 0.4, true box area 0.57 of the picture
- seed108-missing_brick gap_box: presence 0.88, IoU 0.95, confidence 0.72, true box area 0.25 of the picture

## Deterministic checks (every item, no API)

| class | items | deterministic check fired |
|---|---|---|
| partial_load | 2 | 2 |
| stale | 3 | 3 |
| good (false alarms) | 5 | 0 |
| other faults (should not fire) | 6 | 0 |
