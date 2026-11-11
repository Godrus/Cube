# First live run, 2026-11-10 (scene-smoke, Jev 1.14.0)

5 requests (one per fixture, no retries), all HTTP 200. Tokens: **14,573 input, 721 output** in total (about 2,915 input and 143 output per request). The API reports no price. Raw answers: `*.response.json`; per-question results: `*.report.json`; confusion matrices: [`analysis.md`](analysis.md).

| image | label | verdict | notes |
|---|---|---|---|
| good-1 | PASS | **FAIL** | `tears_in_terrain` 0.89 (wrong), `large_hole_in_ground` 0.39 (unsure) |
| good-2 | PASS | **FAIL** | `large_hole_in_ground` 0.76 and `tears_in_terrain` 0.75 (both wrong), `voxel_ground_visible` 0.53 (unsure) |
| black | FAIL | FAIL | caught by 4 checks, `scene_kind` = empty_or_black_frame |
| no-terrain | FAIL | FAIL | caught by 4 checks, `scene_kind` = sky_without_ground (0.68) |
| missing-chunk | FAIL | FAIL | right verdict for the wrong reason (below) |

**Image level (positive = defective):** 3 TP, 0 FN, **2 FP**, 0 TN. All three defective images were flagged; both good screenshots were flagged too, so the questionnaire cannot separate good from bad on this set.

**What worked:** the gross cases. `voxel_ground_visible`, `frame_is_one_flat_colour` and the `scene_kind` Choice were right on every image where the truth is unambiguous (black frame, sky only, normal landscape; the Choice put 0.97 to 0.99 on the correct class for the three landscapes).

**What did not:**
- `tears_in_terrain` is not usable. It said yes (0.75 to 0.89) on both good screenshots: stair-stepped voxel terrain has sharp edges everywhere, so "sharp gaps or cracks" is true of every healthy scene. It also fired on `missing-chunk`, which is how that image got its FAIL.
- `large_hole_in_ground` did not see the synthetic hole (0.33, unsure) and fired on a good screenshot (0.76). The "hole" is filled with the horizon colour, which looks like sky or haze; the question is too close to "is there sky in the lower half".
- `voxel_ground_visible` was only 0.53 on good-2, so the 0.7 threshold gives an "unsure" on a normal scene.

**Conditional Box: not tested live.** Box questions are in `terrain-integrity` and `block-edit`, not in `scene-smoke`, and the budget was five requests, all spent. The gating (a Box result is shown only when its presence Noul is >= 0.7) is verified offline only (`test_visual_qa.py`). Next live step, if allowed: one `terrain-integrity` request on `missing-chunk.png` and one on `good-1.png`.

**Limits of this evidence.** Five images, two classes, one model version. The per-check ground truth (`labels.json`, `checks`) was fixed for the clear cases from how each image was made and for the ambiguous ones (`None`, excluded) after seeing the results, so it is not a clean blind test. No thresholds were changed. This describes one run; it does not prove or disprove model quality.

## Recommendations (not applied, to be checked on NEW images)

1. Drop `tears_in_terrain` or turn it into a question about something a healthy scene does not have, for example "Is a bright magenta or purple area visible" (already in `terrain-integrity`) or "Are there triangular wedge shapes with sloped faces".
2. Replace `large_hole_in_ground` with a Choice over where the background is visible ("only above the horizon" / "also inside the ground area"), or ask the Box after a presence Noul about "a rectangular patch of sky-coloured pixels inside the ground", and use real defects (a Godot screenshot with bricks left out) instead of colour-filled rectangles.
3. Keep `voxel_ground_visible`, `frame_is_one_flat_colour` and `scene_kind`; they separate the gross failures cleanly. As a smoke gate that is probably all this tool should claim today: "is the scene plainly broken (blank, black, no ground)".
4. Collect 15 to 30 more screenshots (varied seeds and camera angles) and a few real defects before touching thresholds; fit on one set and check on another.
5. Treat FAIL from a single noisy check as WARN until a check has a measured false-positive rate.
