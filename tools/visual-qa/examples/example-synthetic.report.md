# Cube Visual QA: scene-smoke

- model: `jev-1.14.0`
- mode: **mock response (no network)**
- verdict: **PASS**
- thresholds (experimental): yes >= 0.7, no <= 0.3
- images: screenshot = `fixtures/good-1.png`
- usage: {"input_tokens": 0, "output_tokens": 0}

| question | answer | expected | result |
|---|---|---|---|
| voxel_ground_visible | 0.970 | True | pass |
| frame_is_one_flat_colour | 0.020 | False | pass |
| large_hole_in_ground | 0.050 | False | pass |
| tears_in_terrain | 0.100 | False | pass |
| scene_kind | voxel_landscape_with_ground {"voxel_landscape_with_ground": 0.93, "other_scene": 0.07} | voxel_landscape_with_ground | pass |

Not a proof of anything: TypeSafe judges a picture; geometry is proven by the Rust tests.
