# Visual QA v2 report: calibration

## Requests and tokens

- answered: 9; infrastructure errors: 3; not sent in this split: 2
- input tokens (answered requests): 32,956; output tokens: 855
- price: $0.042 per million input tokens, output free (docs.typesafe.ai/models): about $0.00138 for the answered requests
- infrastructure error (not a model result): seed101-good: HTTP 529: {"detail":{"error_type":"system_overloaded","messa
- infrastructure error (not a model result): seed102-wrong_material: HTTP 529: {"detail":{"error_type":"system_overloaded","messa
- infrastructure error (not a model result): seed104-missing_brick: HTTP 529: {"detail":{"error_type":"system_overloaded","messa

## scene-smoke-v2

- `terrain_visible`: TP 3, FP 0, TN 0, FN 0, unsure 3
- `scene_kind` (Choice, expected class vs top option): correct 6

## terrain-integrity-v2

- `gap_in_ground`: TP 1, FP 0, TN 1, FN 0, unsure 4
- `off_palette_patch`: TP 0, FP 0, TN 4, FN 0, unsure 2

## block-edit-v2

- `block_missing_in_after`: TP 0, FP 0, TN 2, FN 0, unsure 1

## Per defect kind (the check that should fire)

| defect | items answered | fired (yes) | unsure | missed (no) | infra error |
|---|---|---|---|---|---|
| missing_brick (`gap_in_ground`) | 1 of 2 | 0 | 1 | 0 | 1 |
| hidden_area (`gap_in_ground`) | 2 of 2 | 1 | 1 | 0 | 0 |
| wrong_material (`off_palette_patch`) | 0 of 1 | 0 | 0 | 0 | 1 |
| edit changed (`block_missing_in_after`, truth True) | 1 of 1 | 0 | 1 | 0 | 0 |
| edit stale (`block_missing_in_after`, truth False) | 2 of 2 | 0 | 0 | 2 | 0 |

## Concrete cases (item, check, truth, p)

- FP: none
- FN: none
- unsure: seed101-missing_brick gap_in_ground truth=True p=0.6; seed101-missing_brick off_palette_patch truth=False p=0.32; seed101-hidden_area gap_in_ground truth=True p=0.6; seed101-hidden_area off_palette_patch truth=False p=0.42; seed102-good terrain_visible truth=True p=0.67; seed102-good gap_in_ground truth=False p=0.63; seed103-good terrain_visible truth=True p=0.51; seed103-good gap_in_ground truth=False p=0.34; seed104-good terrain_visible truth=True p=0.57; edit305-normal block_missing_in_after truth=True p=0.49

## Box (shown only when its presence check is >= 0.7)

- seed102-hidden_area gap_box: presence 0.85, IoU 0.92, confidence 0.12, true box area 0.39 of the picture

## Deterministic checks (every item, no API)

| class | items | deterministic check fired |
|---|---|---|
| partial_load | 2 | 2 |
| stale | 2 | 2 |
| good (false alarms) | 5 | 0 |
| other faults (should not fire) | 5 | 0 |
