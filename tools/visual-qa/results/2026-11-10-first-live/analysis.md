# Visual QA analysis (scene-smoke)

## Image level (positive = defective image, predicted positive = verdict FAIL)

| | predicted FAIL | not FAIL |
|---|---|---|
| defective | 3 (TP) | 0 (FN) |
| good | 2 (FP) | 0 (TN) |

False negatives: 0 of 3 defective. False positives: 2 of 2 good. WARN verdicts: 0.

## Per check (rows: truth, columns: the answer at the fixed thresholds)

### voxel_ground_visible

| truth | yes | unsure | no |
|---|---|---|---|
| True | 2 | 1 | 0 |
| False | 0 | 0 | 2 |

Wrong (confident and opposite): none

### frame_is_one_flat_colour

| truth | yes | unsure | no |
|---|---|---|---|
| True | 1 | 0 | 0 |
| False | 0 | 0 | 3 |

Wrong (confident and opposite): none

### large_hole_in_ground

| truth | yes | unsure | no |
|---|---|---|---|
| True | 0 | 1 | 0 |
| False | 1 | 1 | 0 |

Wrong (confident and opposite): good-2.png truth=False p=0.76

### tears_in_terrain

| truth | yes | unsure | no |
|---|---|---|---|
| True | 0 | 0 | 0 |
| False | 2 | 0 | 0 |

Wrong (confident and opposite): good-1.png truth=False p=0.89, good-2.png truth=False p=0.75

Choice `scene_kind`: good-1.png: voxel_landscape_with_ground, good-2.png: voxel_landscape_with_ground, black.png: empty_or_black_frame, no-terrain.png: sky_without_ground, missing-chunk.png: voxel_landscape_with_ground
