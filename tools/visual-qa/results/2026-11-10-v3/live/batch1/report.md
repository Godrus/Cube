# Cube Visual QA v3 report: live-batch1

- generated: 2026-11-10T14:26:59+00:00; generator 3.0.0; question templates hash `53acf98b3d55772d`
- scenarios generated: **7**; executed in Godot: **7**; not executed: 0
- TypeSafe: mode `batch`, transport **LIVE**, requests sent 7, reused 0, skipped 0, deferred 0, errors 0
- estimated input tokens of the plan: 20,738 (about $0.00087); measured input tokens: 19153
- scenarios that need TypeSafe (visual-only): 7; routed to TypeSafe: 7

## Results

| Scenario | Category | Template | Deterministic | TypeSafe | Final |
|---|---|---|---|---|---|
| WORLD-008 | WORLD | world_missing_brick [missing_brick] | PASS | PASS | PASS |
| WORLD-009 | WORLD | world_hidden_area [hidden_area] | PASS | UNKNOWN | UNKNOWN |
| WORLD-010 | WORLD | world_wrong_material [wrong_material] | PASS | PASS | PASS |
| OCCLUSION-017 | OCCLUSION | occ_panel_opaque [occluder_panel] | PASS | PASS | PASS |
| OCCLUSION-018 | OCCLUSION | occ_panel_translucent [occluder_panel] | PASS | PASS | PASS |
| OCCLUSION-019 | OCCLUSION | occ_object_in_front [occluder_object] | PASS | PASS | PASS |
| OCCLUSION-020 | OCCLUSION | occ_hud_corner | PASS | PASS | PASS |

## Summary by category

| Category | n | PASS | WARN | FAIL | UNKNOWN | SKIPPED |
|---|---|---|---|---|---|---|
| WORLD | 3 | 2 | 0 | 0 | 1 | 0 |
| OCCLUSION | 4 | 4 | 0 | 0 | 0 | 0 |

(status = the final status of the scenarios of that category; counts, not rates)

## Details

### WORLD-008 world_missing_brick (WORLD): PASS

- reproduce: `python cube_visual_qa.py suite --from-file reports/suite/live-batch1\scenarios.json --only WORLD-008` (fingerprint `41e47e23b1f7`, world seed 3122, generator seed 7)
- injected fault: missing_brick; expected: `{"bricks_min": 1, "fault_visible": ["p1:ref", "p1:shot"], "shot": "p1:shot"}`
- pictures: `../runA/scenarios/WORLD-008/p1/ref.png`, `../runA/scenarios/WORLD-008/p1/shot.png`
  - PASS    executed: 1 Godot run(s) finished with telemetry
  - PASS    bricks_loaded: 82 bricks in the scene (need >= 1)
  - PASS    frame_not_black: black 0.000
  - PASS    frame_not_flat: dominant colour 0.280
  - PASS    injected_fault_visible: the injected fault changed 0.0157 of the frame
  - TypeSafe Q-TERR-GAP-001 v1 (v2 wording): pass answer=0.9, expected True

### WORLD-009 world_hidden_area (WORLD): UNKNOWN

- reproduce: `python cube_visual_qa.py suite --from-file reports/suite/live-batch1\scenarios.json --only WORLD-009` (fingerprint `fd5798ca5a89`, world seed 8868, generator seed 7)
- injected fault: hidden_area; expected: `{"bricks_min": 1, "fault_visible": ["p1:ref", "p1:shot"], "shot": "p1:shot"}`
- pictures: `../runA/scenarios/WORLD-009/p1/ref.png`, `../runA/scenarios/WORLD-009/p1/shot.png`
  - PASS    executed: 1 Godot run(s) finished with telemetry
  - PASS    bricks_loaded: 69 bricks in the scene (need >= 1)
  - PASS    frame_not_black: black 0.000
  - PASS    frame_not_flat: dominant colour 0.488
  - PASS    injected_fault_visible: the injected fault changed 0.7899 of the frame
  - TypeSafe Q-TERR-GAP-001 v1 (v2 wording): unsure answer=0.42, expected True

### WORLD-010 world_wrong_material (WORLD): PASS

- reproduce: `python cube_visual_qa.py suite --from-file reports/suite/live-batch1\scenarios.json --only WORLD-010` (fingerprint `df1f0193c244`, world seed 8969, generator seed 7)
- injected fault: wrong_material; expected: `{"bricks_min": 1, "fault_visible": ["p1:ref", "p1:shot"], "shot": "p1:shot"}`
- pictures: `../runA/scenarios/WORLD-010/p1/ref.png`, `../runA/scenarios/WORLD-010/p1/shot.png`
  - PASS    executed: 1 Godot run(s) finished with telemetry
  - PASS    bricks_loaded: 77 bricks in the scene (need >= 1)
  - PASS    frame_not_black: black 0.000
  - PASS    frame_not_flat: dominant colour 0.325
  - PASS    injected_fault_visible: the injected fault changed 0.0162 of the frame
  - TypeSafe Q-MAT-PATCH-001 v1 (v2 wording): pass answer=0.95, expected True

### OCCLUSION-017 occ_panel_opaque (OCCLUSION): PASS

- reproduce: `python cube_visual_qa.py suite --from-file reports/suite/live-batch1\scenarios.json --only OCCLUSION-017` (fingerprint `927461acb1af`, world seed 5765, generator seed 7)
- injected fault: occluder_panel; expected: `{"bricks_min": 1, "fault_visible": ["p1:ref", "p1:shot"], "shot": "p1:shot", "centre_flat_info": true}`
- pictures: `../runA/scenarios/OCCLUSION-017/p1/ref.png`, `../runA/scenarios/OCCLUSION-017/p1/shot.png`
  - PASS    executed: 1 Godot run(s) finished with telemetry
  - PASS    bricks_loaded: 74 bricks in the scene (need >= 1)
  - PASS    frame_not_black: black 0.000
  - PASS    frame_not_flat: dominant colour 0.417
  - PASS    injected_fault_visible: the injected fault changed 0.4152 of the frame
  - INFO    centre_flat_fraction: 1.00 of the central region is one colour (information only: flat ground gives high values too)
  - TypeSafe Q-OCC-CENTER-001 v1 (NEW): pass answer=0.93, expected True

### OCCLUSION-018 occ_panel_translucent (OCCLUSION): PASS

- reproduce: `python cube_visual_qa.py suite --from-file reports/suite/live-batch1\scenarios.json --only OCCLUSION-018` (fingerprint `f79b662afdc7`, world seed 2429, generator seed 7)
- injected fault: occluder_panel; expected: `{"bricks_min": 1, "fault_visible": ["p1:ref", "p1:shot"], "shot": "p1:shot", "centre_flat_info": true}`
- pictures: `../runA/scenarios/OCCLUSION-018/p1/ref.png`, `../runA/scenarios/OCCLUSION-018/p1/shot.png`
  - PASS    executed: 1 Godot run(s) finished with telemetry
  - PASS    bricks_loaded: 82 bricks in the scene (need >= 1)
  - PASS    frame_not_black: black 0.000
  - PASS    frame_not_flat: dominant colour 0.198
  - PASS    injected_fault_visible: the injected fault changed 0.4152 of the frame
  - INFO    centre_flat_fraction: 0.53 of the central region is one colour (information only: flat ground gives high values too)
  - TypeSafe Q-OCC-CENTER-001 v1 (NEW): pass answer=0.81, expected True

### OCCLUSION-019 occ_object_in_front (OCCLUSION): PASS

- reproduce: `python cube_visual_qa.py suite --from-file reports/suite/live-batch1\scenarios.json --only OCCLUSION-019` (fingerprint `133853c2990c`, world seed 5106, generator seed 7)
- injected fault: occluder_object; expected: `{"bricks_min": 1, "fault_visible": ["p1:ref", "p1:shot"], "shot": "p1:shot", "centre_flat_info": true}`
- pictures: `../runA/scenarios/OCCLUSION-019/p1/ref.png`, `../runA/scenarios/OCCLUSION-019/p1/shot.png`
  - PASS    executed: 1 Godot run(s) finished with telemetry
  - PASS    bricks_loaded: 82 bricks in the scene (need >= 1)
  - PASS    frame_not_black: black 0.000
  - PASS    frame_not_flat: dominant colour 0.317
  - PASS    injected_fault_visible: the injected fault changed 0.3170 of the frame
  - INFO    centre_flat_fraction: 1.00 of the central region is one colour (information only: flat ground gives high values too)
  - TypeSafe Q-OCC-CENTER-001 v1 (NEW): pass answer=0.97, expected True

### OCCLUSION-020 occ_hud_corner (OCCLUSION): PASS

- reproduce: `python cube_visual_qa.py suite --from-file reports/suite/live-batch1\scenarios.json --only OCCLUSION-020` (fingerprint `4786ccc07b87`, world seed 1645, generator seed 7)
- injected fault: none; expected: `{"bricks_min": 1, "fault_visible": null, "shot": "p1:shot", "centre_flat_info": true}`
- pictures: `../runA/scenarios/OCCLUSION-020/p1/ref.png`, `../runA/scenarios/OCCLUSION-020/p1/shot.png`
  - PASS    executed: 1 Godot run(s) finished with telemetry
  - PASS    bricks_loaded: 79 bricks in the scene (need >= 1)
  - PASS    frame_not_black: black 0.000
  - PASS    frame_not_flat: dominant colour 0.375
  - INFO    centre_flat_fraction: 0.83 of the central region is one colour (information only: flat ground gives high values too)
  - TypeSafe Q-OCC-CENTER-001 v1 (NEW): pass answer=0.09, expected False

Statuses: PASS / WARN / FAIL / UNKNOWN / SKIPPED. For injected-fault scenarios the expectation is that the fault is found; a status says whether the observation matched it. TypeSafe is advisory: a model disagreement is a WARN, an unsure answer or an HTTP 529 is UNKNOWN. Nothing here is a measurement of TypeSafe accuracy; see V2-RESULTS.md for that.
