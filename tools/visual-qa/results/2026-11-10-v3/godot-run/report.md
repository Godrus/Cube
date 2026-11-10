# Cube Visual QA v3 report: runA

- generated: 2026-11-10T14:16:15+00:00; generator 3.0.0; question templates hash `53acf98b3d55772d`
- scenarios generated: **20**; executed in Godot: **20**; not executed: 0
- TypeSafe: mode `adaptive`, transport **MOCK (no network, answers come from the scenarios' own expectations)**, requests sent 1, reused 0, skipped 0, deferred 0, errors 0
- estimated input tokens of the plan: 20,018 (about $0.00084); measured input tokens: n/a (mock)
- scenarios that need TypeSafe (visual-only): 7; routed to TypeSafe: 7

## Results

| Scenario | Category | Template | Deterministic | TypeSafe | Final |
|---|---|---|---|---|---|
| CAMERA-001 | CAMERA | cam_top | PASS | SKIPPED | PASS |
| CAMERA-002 | CAMERA | cam_up | PASS | SKIPPED | PASS |
| CAMERA-003 | CAMERA | cam_horizontal | PASS | SKIPPED | PASS |
| CAMERA-004 | CAMERA | cam_angled | PASS | SKIPPED | PASS |
| CAMERA-005 | CAMERA | cam_rotate | PASS | SKIPPED | PASS |
| CAMERA-006 | CAMERA | cam_move | PASS | SKIPPED | PASS |
| WORLD-007 | WORLD | world_view | PASS | SKIPPED | PASS |
| WORLD-008 | WORLD | world_missing_brick [missing_brick] | PASS | PASS | PASS |
| WORLD-009 | WORLD | world_hidden_area [hidden_area] | PASS | PASS | PASS |
| WORLD-010 | WORLD | world_wrong_material [wrong_material] | PASS | PASS | PASS |
| WORLD-011 | WORLD | world_partial_load [partial_load] | PASS | SKIPPED | PASS |
| WORLD-012 | WORLD | world_restore | PASS | SKIPPED | PASS |
| EDITING-013 | EDITING | edit_remove | PASS | SKIPPED | PASS |
| EDITING-014 | EDITING | edit_place | PASS | SKIPPED | PASS |
| EDITING-015 | EDITING | edit_series | PASS | SKIPPED | PASS |
| EDITING-016 | EDITING | edit_persist | PASS | SKIPPED | PASS |
| OCCLUSION-017 | OCCLUSION | occ_panel_opaque [occluder_panel] | PASS | PASS | PASS |
| OCCLUSION-018 | OCCLUSION | occ_panel_translucent [occluder_panel] | PASS | PASS | PASS |
| OCCLUSION-019 | OCCLUSION | occ_object_in_front [occluder_object] | PASS | PASS | PASS |
| OCCLUSION-020 | OCCLUSION | occ_hud_corner | PASS | PASS | PASS |

## Summary by category

| Category | n | PASS | WARN | FAIL | UNKNOWN | SKIPPED |
|---|---|---|---|---|---|---|
| CAMERA | 6 | 6 | 0 | 0 | 0 | 0 |
| WORLD | 6 | 6 | 0 | 0 | 0 | 0 |
| EDITING | 4 | 4 | 0 | 0 | 0 | 0 |
| OCCLUSION | 4 | 4 | 0 | 0 | 0 | 0 |

(status = the final status of the scenarios of that category; counts, not rates)

## Details

### CAMERA-001 cam_top (CAMERA): PASS

- reproduce: `python cube_visual_qa.py suite --from-file reports/suite/runA\scenarios.json --only CAMERA-001` (fingerprint `b690ce37f8da`, world seed 6324, generator seed 7)
- injected fault: none; expected: `{"pitch_delta": -0.8, "pitch_tolerance": 0.03, "climbed": true, "sky_fraction_max": 0.05, "shot": "p1:top"}`
- pictures: `scenarios/CAMERA-001/p1/top.png`
  - PASS    executed: 1 Godot run(s) finished with telemetry
  - PASS    frame_not_black: black 0.000
  - PASS    frame_not_flat: dominant colour 0.592
  - PASS    sky_fraction: sky 0.000, expected 0.0..0.05
  - PASS    camera_pitch: pitch changed by -0.800 rad, commanded -0.800 (tolerance 0.03)
  - PASS    camera_climbed: height changed by +4.00
- TypeSafe: not asked: covered by deterministic checks (use --compare or --force-typesafe to ask anyway)

### CAMERA-002 cam_up (CAMERA): PASS

- reproduce: `python cube_visual_qa.py suite --from-file reports/suite/runA\scenarios.json --only CAMERA-002` (fingerprint `6afd0410cbc3`, world seed 9628, generator seed 7)
- injected fault: none; expected: `{"pitch_delta": 1.7, "pitch_tolerance": 0.03, "sky_fraction_min": 0.9, "shot": "p1:up"}`
- pictures: `scenarios/CAMERA-002/p1/up.png`
  - PASS    executed: 1 Godot run(s) finished with telemetry
  - PASS    frame_not_black: black 0.000
  - PASS    sky_fraction: sky 1.000, expected 0.9..1.0
  - PASS    camera_pitch: pitch changed by +1.700 rad, commanded +1.700 (tolerance 0.03)
- TypeSafe: not asked: covered by deterministic checks (use --compare or --force-typesafe to ask anyway)

### CAMERA-003 cam_horizontal (CAMERA): PASS

- reproduce: `python cube_visual_qa.py suite --from-file reports/suite/runA\scenarios.json --only CAMERA-003` (fingerprint `4dacb108f5e7`, world seed 1271, generator seed 7)
- injected fault: none; expected: `{"pitch_delta": 0.6, "pitch_tolerance": 0.03, "sky_fraction_min": 0.02, "sky_fraction_max": 0.98, "shot": "p1:level"}`
- pictures: `scenarios/CAMERA-003/p1/level.png`
  - PASS    executed: 1 Godot run(s) finished with telemetry
  - PASS    frame_not_black: black 0.000
  - PASS    sky_fraction: sky 0.507, expected 0.02..0.98
  - PASS    camera_pitch: pitch changed by +0.600 rad, commanded +0.600 (tolerance 0.03)
- TypeSafe: not asked: covered by deterministic checks (use --compare or --force-typesafe to ask anyway)

### CAMERA-004 cam_angled (CAMERA): PASS

- reproduce: `python cube_visual_qa.py suite --from-file reports/suite/runA\scenarios.json --only CAMERA-004` (fingerprint `efddb26e1128`, world seed 2682, generator seed 7)
- injected fault: none; expected: `{"yaw_delta": 1.071, "pitch_delta": -0.2, "pitch_tolerance": 0.03, "yaw_tolerance": 0.03, "shot": "p1:angled"}`
- pictures: `scenarios/CAMERA-004/p1/angled.png`
  - PASS    executed: 1 Godot run(s) finished with telemetry
  - PASS    frame_not_black: black 0.000
  - PASS    frame_not_flat: dominant colour 0.326
  - PASS    camera_pitch: pitch changed by -0.200 rad, commanded -0.200 (tolerance 0.03)
  - PASS    camera_yaw: yaw changed by +1.071 rad, commanded +1.071 (tolerance 0.03)
- TypeSafe: not asked: covered by deterministic checks (use --compare or --force-typesafe to ask anyway)

### CAMERA-005 cam_rotate (CAMERA): PASS

- reproduce: `python cube_visual_qa.py suite --from-file reports/suite/runA\scenarios.json --only CAMERA-005` (fingerprint `216245d3fa64`, world seed 5298, generator seed 7)
- injected fault: none; expected: `{"yaw_delta": 3.1416, "yaw_tolerance": 0.03, "position_unchanged": true, "same_view_as": null}`
- pictures: `scenarios/CAMERA-005/p1/before.png`, `scenarios/CAMERA-005/p1/after.png`
  - PASS    executed: 1 Godot run(s) finished with telemetry
  - PASS    camera_yaw: yaw changed by -3.142 rad, commanded -3.142 (tolerance 0.03)
  - PASS    camera_position_unchanged: largest coordinate change 0.00000
- TypeSafe: not asked: no TypeSafe question defined for this scenario

### CAMERA-006 cam_move (CAMERA): PASS

- reproduce: `python cube_visual_qa.py suite --from-file reports/suite/runA\scenarios.json --only CAMERA-006` (fingerprint `401f75420bcb`, world seed 1231, generator seed 7)
- injected fault: none; expected: `{"moved_along_heading": true, "shot": "p1:moved"}`
- pictures: `scenarios/CAMERA-006/p1/moved.png`
  - PASS    executed: 1 Godot run(s) finished with telemetry
  - PASS    frame_not_black: black 0.000
  - PASS    frame_not_flat: dominant colour 0.254
  - PASS    moved_along_heading: moved 6.60 horizontally, alignment with the heading 1.00
- TypeSafe: not asked: no TypeSafe question defined for this scenario

### WORLD-007 world_view (WORLD): PASS

- reproduce: `python cube_visual_qa.py suite --from-file reports/suite/runA\scenarios.json --only WORLD-007` (fingerprint `170086972be7`, world seed 4613, generator seed 7)
- injected fault: none; expected: `{"bricks_min": 1, "shot": "p1:view"}`
- pictures: `scenarios/WORLD-007/p1/view.png`
  - PASS    executed: 1 Godot run(s) finished with telemetry
  - PASS    bricks_loaded: 70 bricks in the scene (need >= 1)
  - PASS    frame_not_black: black 0.000
  - PASS    frame_not_flat: dominant colour 0.314
- TypeSafe: not asked: covered by deterministic checks (use --compare or --force-typesafe to ask anyway)

### WORLD-008 world_missing_brick (WORLD): PASS

- reproduce: `python cube_visual_qa.py suite --from-file reports/suite/runA\scenarios.json --only WORLD-008` (fingerprint `41e47e23b1f7`, world seed 3122, generator seed 7)
- injected fault: missing_brick; expected: `{"bricks_min": 1, "fault_visible": ["p1:ref", "p1:shot"], "shot": "p1:shot"}`
- pictures: `scenarios/WORLD-008/p1/ref.png`, `scenarios/WORLD-008/p1/shot.png`
  - PASS    executed: 1 Godot run(s) finished with telemetry
  - PASS    bricks_loaded: 82 bricks in the scene (need >= 1)
  - PASS    frame_not_black: black 0.000
  - PASS    frame_not_flat: dominant colour 0.280
  - PASS    injected_fault_visible: the injected fault changed 0.0157 of the frame
  - TypeSafe Q-TERR-GAP-001 v1 (v2 wording): pass answer=0.9, expected True

### WORLD-009 world_hidden_area (WORLD): PASS

- reproduce: `python cube_visual_qa.py suite --from-file reports/suite/runA\scenarios.json --only WORLD-009` (fingerprint `fd5798ca5a89`, world seed 8868, generator seed 7)
- injected fault: hidden_area; expected: `{"bricks_min": 1, "fault_visible": ["p1:ref", "p1:shot"], "shot": "p1:shot"}`
- pictures: `scenarios/WORLD-009/p1/ref.png`, `scenarios/WORLD-009/p1/shot.png`
  - PASS    executed: 1 Godot run(s) finished with telemetry
  - PASS    bricks_loaded: 69 bricks in the scene (need >= 1)
  - PASS    frame_not_black: black 0.000
  - PASS    frame_not_flat: dominant colour 0.488
  - PASS    injected_fault_visible: the injected fault changed 0.7899 of the frame
  - TypeSafe Q-TERR-GAP-001 v1 (v2 wording): pass answer=0.9, expected True

### WORLD-010 world_wrong_material (WORLD): PASS

- reproduce: `python cube_visual_qa.py suite --from-file reports/suite/runA\scenarios.json --only WORLD-010` (fingerprint `df1f0193c244`, world seed 8969, generator seed 7)
- injected fault: wrong_material; expected: `{"bricks_min": 1, "fault_visible": ["p1:ref", "p1:shot"], "shot": "p1:shot"}`
- pictures: `scenarios/WORLD-010/p1/ref.png`, `scenarios/WORLD-010/p1/shot.png`
  - PASS    executed: 1 Godot run(s) finished with telemetry
  - PASS    bricks_loaded: 77 bricks in the scene (need >= 1)
  - PASS    frame_not_black: black 0.000
  - PASS    frame_not_flat: dominant colour 0.325
  - PASS    injected_fault_visible: the injected fault changed 0.0162 of the frame
  - TypeSafe Q-MAT-PATCH-001 v1 (v2 wording): pass answer=0.9, expected True

### WORLD-011 world_partial_load (WORLD): PASS

- reproduce: `python cube_visual_qa.py suite --from-file reports/suite/runA\scenarios.json --only WORLD-011` (fingerprint `021fb37e7149`, world seed 6836, generator seed 7)
- injected fault: partial_load; expected: `{"partial_detected": {"reference": "p1", "partial": "p2", "ratio_below": 0.95}}`
- pictures: `scenarios/WORLD-011/p1/full.png`, `scenarios/WORLD-011/p2/partial.png`
  - PASS    executed: 2 Godot run(s) finished with telemetry
  - PASS    partial_load_detected: 15 of 80 bricks loaded (0.19); the detector fires below 0.95
- TypeSafe: not asked: no TypeSafe question defined for this scenario

### WORLD-012 world_restore (WORLD): PASS

- reproduce: `python cube_visual_qa.py suite --from-file reports/suite/runA\scenarios.json --only WORLD-012` (fingerprint `70d6a5ffe128`, world seed 7417, generator seed 7)
- injected fault: none; expected: `{"restored_equal": {"a": "p1:view", "b": "p2:view"}, "pixel_diff_max": 0.001}`
- pictures: `scenarios/WORLD-012/p1/view.png`, `scenarios/WORLD-012/p2/view.png`
  - PASS    executed: 2 Godot run(s) finished with telemetry
  - PASS    restore_bricks_equal: 76 bricks before, 76 after reopening
  - PASS    restore_picture_equal: 0.00000 of the pixels differ after reopening the world (limit 0.001)
- TypeSafe: not asked: covered by deterministic checks (use --compare or --force-typesafe to ask anyway)

### EDITING-013 edit_remove (EDITING): PASS

- reproduce: `python cube_visual_qa.py suite --from-file reports/suite/runA\scenarios.json --only EDITING-013` (fingerprint `350f36240f40`, world seed 9849, generator seed 7)
- injected fault: none; expected: `{"edits_hit": 1, "bricks_changed_min": 1, "before": "p1:before", "after": "p1:after"}`
- pictures: `scenarios/EDITING-013/p1/before.png`, `scenarios/EDITING-013/p1/after.png`
  - PASS    executed: 1 Godot run(s) finished with telemetry
  - PASS    edits_applied: 1 of 1 edits hit a block, expected 1
  - PASS    render_batches_updated: 27 brick batches rebuilt after the edits
  - PASS    picture_changed_after_edit: 0.00198 of the pixels changed
- TypeSafe: not asked: covered by deterministic checks (use --compare or --force-typesafe to ask anyway)

### EDITING-014 edit_place (EDITING): PASS

- reproduce: `python cube_visual_qa.py suite --from-file reports/suite/runA\scenarios.json --only EDITING-014` (fingerprint `c8b1b1a6a387`, world seed 7173, generator seed 7)
- injected fault: none; expected: `{"edits_hit": 1, "bricks_changed_min": 1, "before": "p1:before", "after": "p1:after"}`
- pictures: `scenarios/EDITING-014/p1/before.png`, `scenarios/EDITING-014/p1/after.png`
  - PASS    executed: 1 Godot run(s) finished with telemetry
  - PASS    edits_applied: 1 of 1 edits hit a block, expected 1
  - PASS    render_batches_updated: 27 brick batches rebuilt after the edits
  - PASS    picture_changed_after_edit: 0.00135 of the pixels changed
- TypeSafe: not asked: no TypeSafe question defined for this scenario

### EDITING-015 edit_series (EDITING): PASS

- reproduce: `python cube_visual_qa.py suite --from-file reports/suite/runA\scenarios.json --only EDITING-015` (fingerprint `fca096994d2f`, world seed 6985, generator seed 7)
- injected fault: none; expected: `{"edits_hit": 4, "bricks_changed_min": 1, "before": "p1:before", "after": "p1:after"}`
- pictures: `scenarios/EDITING-015/p1/before.png`, `scenarios/EDITING-015/p1/after.png`
  - PASS    executed: 1 Godot run(s) finished with telemetry
  - PASS    edits_applied: 4 of 4 edits hit a block, expected 4
  - PASS    render_batches_updated: 108 brick batches rebuilt after the edits
  - PASS    picture_changed_after_edit: 0.23161 of the pixels changed
- TypeSafe: not asked: no TypeSafe question defined for this scenario

### EDITING-016 edit_persist (EDITING): PASS

- reproduce: `python cube_visual_qa.py suite --from-file reports/suite/runA\scenarios.json --only EDITING-016` (fingerprint `533b7db41101`, world seed 6138, generator seed 7)
- injected fault: none; expected: `{"edits_hit": 5, "bricks_changed_min": 1, "before": "p1:before", "after": "p1:after", "persisted": {"after": "p1:after", "reloaded": "p2:reloaded", "pixel_diff_max": 0.001}}`
- pictures: `scenarios/EDITING-016/p1/before.png`, `scenarios/EDITING-016/p1/after.png`, `scenarios/EDITING-016/p2/reloaded.png`
  - PASS    executed: 2 Godot run(s) finished with telemetry
  - PASS    edits_applied: 5 of 5 edits hit a block, expected 5
  - PASS    render_batches_updated: 135 brick batches rebuilt after the edits
  - PASS    picture_changed_after_edit: 0.27116 of the pixels changed
  - PASS    edit_persisted: 0.00000 of the pixels differ between the edited and the reloaded world
- TypeSafe: not asked: no TypeSafe question defined for this scenario

### OCCLUSION-017 occ_panel_opaque (OCCLUSION): PASS

- reproduce: `python cube_visual_qa.py suite --from-file reports/suite/runA\scenarios.json --only OCCLUSION-017` (fingerprint `927461acb1af`, world seed 5765, generator seed 7)
- injected fault: occluder_panel; expected: `{"bricks_min": 1, "fault_visible": ["p1:ref", "p1:shot"], "shot": "p1:shot", "centre_flat_info": true}`
- pictures: `scenarios/OCCLUSION-017/p1/ref.png`, `scenarios/OCCLUSION-017/p1/shot.png`
  - PASS    executed: 1 Godot run(s) finished with telemetry
  - PASS    bricks_loaded: 74 bricks in the scene (need >= 1)
  - PASS    frame_not_black: black 0.000
  - PASS    frame_not_flat: dominant colour 0.417
  - PASS    injected_fault_visible: the injected fault changed 0.4152 of the frame
  - INFO    centre_flat_fraction: 1.00 of the central region is one colour (information only: flat ground gives high values too)
  - TypeSafe Q-OCC-CENTER-001 v1 (NEW): pass answer=0.9, expected True

### OCCLUSION-018 occ_panel_translucent (OCCLUSION): PASS

- reproduce: `python cube_visual_qa.py suite --from-file reports/suite/runA\scenarios.json --only OCCLUSION-018` (fingerprint `f79b662afdc7`, world seed 2429, generator seed 7)
- injected fault: occluder_panel; expected: `{"bricks_min": 1, "fault_visible": ["p1:ref", "p1:shot"], "shot": "p1:shot", "centre_flat_info": true}`
- pictures: `scenarios/OCCLUSION-018/p1/ref.png`, `scenarios/OCCLUSION-018/p1/shot.png`
  - PASS    executed: 1 Godot run(s) finished with telemetry
  - PASS    bricks_loaded: 82 bricks in the scene (need >= 1)
  - PASS    frame_not_black: black 0.000
  - PASS    frame_not_flat: dominant colour 0.198
  - PASS    injected_fault_visible: the injected fault changed 0.4152 of the frame
  - INFO    centre_flat_fraction: 0.53 of the central region is one colour (information only: flat ground gives high values too)
  - TypeSafe Q-OCC-CENTER-001 v1 (NEW): pass answer=0.9, expected True

### OCCLUSION-019 occ_object_in_front (OCCLUSION): PASS

- reproduce: `python cube_visual_qa.py suite --from-file reports/suite/runA\scenarios.json --only OCCLUSION-019` (fingerprint `133853c2990c`, world seed 5106, generator seed 7)
- injected fault: occluder_object; expected: `{"bricks_min": 1, "fault_visible": ["p1:ref", "p1:shot"], "shot": "p1:shot", "centre_flat_info": true}`
- pictures: `scenarios/OCCLUSION-019/p1/ref.png`, `scenarios/OCCLUSION-019/p1/shot.png`
  - PASS    executed: 1 Godot run(s) finished with telemetry
  - PASS    bricks_loaded: 82 bricks in the scene (need >= 1)
  - PASS    frame_not_black: black 0.000
  - PASS    frame_not_flat: dominant colour 0.317
  - PASS    injected_fault_visible: the injected fault changed 0.3170 of the frame
  - INFO    centre_flat_fraction: 1.00 of the central region is one colour (information only: flat ground gives high values too)
  - TypeSafe Q-OCC-CENTER-001 v1 (NEW): pass answer=0.9, expected True

### OCCLUSION-020 occ_hud_corner (OCCLUSION): PASS

- reproduce: `python cube_visual_qa.py suite --from-file reports/suite/runA\scenarios.json --only OCCLUSION-020` (fingerprint `4786ccc07b87`, world seed 1645, generator seed 7)
- injected fault: none; expected: `{"bricks_min": 1, "fault_visible": null, "shot": "p1:shot", "centre_flat_info": true}`
- pictures: `scenarios/OCCLUSION-020/p1/ref.png`, `scenarios/OCCLUSION-020/p1/shot.png`
  - PASS    executed: 1 Godot run(s) finished with telemetry
  - PASS    bricks_loaded: 79 bricks in the scene (need >= 1)
  - PASS    frame_not_black: black 0.000
  - PASS    frame_not_flat: dominant colour 0.375
  - INFO    centre_flat_fraction: 0.83 of the central region is one colour (information only: flat ground gives high values too)
  - TypeSafe Q-OCC-CENTER-001 v1 (NEW): pass answer=0.09999999999999998, expected False

Statuses: PASS / WARN / FAIL / UNKNOWN / SKIPPED. For injected-fault scenarios the expectation is that the fault is found; a status says whether the observation matched it. TypeSafe is advisory: a model disagreement is a WARN, an unsure answer or an HTTP 529 is UNKNOWN. Nothing here is a measurement of TypeSafe accuracy; see V2-RESULTS.md for that.
