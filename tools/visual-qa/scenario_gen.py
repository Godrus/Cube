"""Scenario generator of Visual QA v3: reproducible Cube play scenarios (no model, no network).

A scenario is plain data: world seed, phases (each one Godot run) with a list of steps, the
deterministic expectations, the injected fault (if any) and the TypeSafe questions that apply.
Scenario i of a run depends only on (generator version, seed, i, template): never on the order in
which scenarios run. Only what the current Godot scene and API can really do is generated;
everything else is listed in UNSUPPORTED and never produces a (fake) test.
"""

import hashlib
import json
import os
import random

HERE = os.path.dirname(os.path.abspath(__file__))
GENERATOR_VERSION = "3.0.0"

CATEGORIES = {"camera": "CAMERA", "terrain": "WORLD", "world": "WORLD", "editing": "EDITING", "occlusion": "OCCLUSION"}
# What the current client/Godot adapter cannot do (no API for it); nothing is generated for these.
UNSUPPORTED = {
    "collision": "no collision or physics query exists in the client API (the camera is a free camera)",
    "camera_penetration": "needs collision queries; a screenshot cannot prove it",
    "move_to_obstacle / move_along_wall": "needs collision queries",
    "bottom_view_of_overhangs": "WorldGen produces no overhangs; 'camera looking up' sees sky only",
    "edit_on_brick_boundary": "the adapter does not expose brick coordinates or a raycast hit position",
    "core_hash": "no state hash is exposed by the adapter",
    "inside_surface_occlusion / z_order_bug": "no such rendering path exists to test",
    "absolute_camera_position": "only relative moves exist (a teleport would need a new API)",
}
SETTLE = 45  # frames before the first picture (the first batches arrive)


def _w(n):
    return {"op": "wait", "frames": n}


def _shot(name):
    return [_w(3), {"op": "shot", "name": name}]


def _turn(yaw=0.0, pitch=0.0, frames=10):
    return {"op": "turn", "yaw": round(yaw, 4), "pitch": round(pitch, 4), "frames": frames}


def _move(fwd=0, right=0, up=0, frames=10):
    return {"op": "move", "fwd": fwd, "right": right, "up": up, "frames": frames}


def _ts(role, questions):
    return {"role": role, "questions": questions}


def _q(qid, expect, **slots):
    return {"qid": qid, "expect": expect, "slots": slots}


def _base(rng, intensity):
    """A camera start shared by the picture scenarios: a small, seeded yaw/pitch variation."""
    return [_turn(yaw=round(rng.uniform(-0.8, 0.8), 3), pitch=round(rng.uniform(-0.1, 0.1), 3), frames=5)]


# --- templates: each returns the scenario body (phases, expected, ts, ...) ---------------------

def t_cam_top(rng, k):
    up = 8 + 6 * k
    steps = [_w(SETTLE), {"op": "probe", "name": "start"}, _move(up=1, frames=up), _turn(pitch=-0.8, frames=8)] + _shot("top")
    return dict(camera="top", phases=[{"name": "p1", "create": True, "steps": steps}],
                expected={"pitch_delta": -0.8, "pitch_tolerance": 0.03, "climbed": True, "sky_fraction_max": 0.05, "shot": "p1:top"},
                ts=_ts("confirm", [_q("Q-CAM-TOP-001", True, shot="p1:top")]))


def t_cam_up(rng, k):
    steps = [_w(SETTLE), {"op": "probe", "name": "start"}, _turn(pitch=1.7, frames=10)] + _shot("up")
    return dict(camera="up", phases=[{"name": "p1", "create": True, "steps": steps}],
                expected={"pitch_delta": 1.7, "pitch_tolerance": 0.03, "sky_fraction_min": 0.9, "shot": "p1:up"},
                ts=_ts("confirm", [_q("Q-CAM-SKY-001", True, shot="p1:up")]))


def t_cam_horizontal(rng, k):
    steps = [_w(SETTLE), {"op": "probe", "name": "start"}, _turn(pitch=0.6, frames=8)] + _shot("level")
    return dict(camera="horizontal", phases=[{"name": "p1", "create": True, "steps": steps}],
                expected={"pitch_delta": 0.6, "pitch_tolerance": 0.03, "sky_fraction_min": 0.02, "sky_fraction_max": 0.98, "shot": "p1:level"},
                ts=_ts("confirm", [_q("Q-CAM-SKY-001", True, shot="p1:level")]))


def t_cam_angled(rng, k):
    yaw = round(rng.uniform(-1.2, 1.2), 3)
    steps = [_w(SETTLE), {"op": "probe", "name": "start"}, _turn(yaw=yaw, pitch=-0.2, frames=10)] + _shot("angled")
    return dict(camera="angled", phases=[{"name": "p1", "create": True, "steps": steps}],
                expected={"yaw_delta": yaw, "pitch_delta": -0.2, "pitch_tolerance": 0.03, "yaw_tolerance": 0.03, "shot": "p1:angled"},
                ts=_ts("confirm", [_q("Q-TERR-VISIBLE-001", True, shot="p1:angled")]))


def t_cam_rotate(rng, k):
    angle = rng.choice([1.5708, 3.1416, 6.2832])
    steps = [_w(SETTLE)] + _shot("before") + [{"op": "probe", "name": "start"}, _turn(yaw=angle, frames=24)] + _shot("after")
    return dict(camera=f"rotate_{round(angle * 57.2958)}", phases=[{"name": "p1", "create": True, "steps": steps}],
                expected={"yaw_delta": angle, "yaw_tolerance": 0.03, "position_unchanged": True,
                          "same_view_as": ["p1:before", "p1:after"] if angle > 6.0 else None},
                ts=_ts("none", []))


def t_cam_move(rng, k):
    n = 20 + 10 * k
    steps = [_w(SETTLE), {"op": "probe", "name": "start"}, _move(fwd=1, frames=n), _w(2)] + _shot("moved")
    return dict(camera="move_forward", phases=[{"name": "p1", "create": True, "steps": steps}],
                expected={"moved_along_heading": True, "shot": "p1:moved"}, ts=_ts("none", []))


def t_world_view(rng, k):
    steps = [_w(SETTLE + 15)] + _base(rng, k) + _shot("view")
    return dict(camera="default", phases=[{"name": "p1", "create": True, "steps": steps}],
                expected={"bricks_min": 1, "shot": "p1:view"},
                ts=_ts("confirm", [_q("Q-TERR-VISIBLE-001", True, shot="p1:view"), _q("Q-SCENE-KIND-001", "block_landscape", shot="p1:view")]))


def _fault(kind, qid):
    def t(rng, k):
        steps = [_w(SETTLE)] + _base(rng, k) + _shot("ref") + [_w(2), {"op": "fault", "kind": kind}] + _shot("shot")
        return dict(camera="default", phases=[{"name": "p1", "create": True, "steps": steps}], injected_fault=kind,
                    needs_visual=True,
                    expected={"bricks_min": 1, "fault_visible": ["p1:ref", "p1:shot"], "shot": "p1:shot"},
                    ts=_ts("primary", [_q(qid, True, shot="p1:shot")]))
    return t


def t_world_partial(rng, k):
    ref = [_w(SETTLE)] + _base(rng, k) + _shot("full")
    cut = [{"op": "freeze"}, _w(20)] + _shot("partial")
    return dict(camera="default", injected_fault="partial_load",
                phases=[{"name": "p1", "create": True, "steps": ref}, {"name": "p2", "create": True, "steps": cut}],
                expected={"partial_detected": {"reference": "p1", "partial": "p2", "ratio_below": 0.95}}, ts=_ts("none", []))


def t_world_restore(rng, k):
    steps = [_w(SETTLE)] + _base(rng, k) + _shot("view")
    return dict(camera="default",
                phases=[{"name": "p1", "create": True, "steps": steps}, {"name": "p2", "create": False, "same_world_as": "p1", "steps": steps}],
                expected={"restored_equal": {"a": "p1:view", "b": "p2:view"}, "pixel_diff_max": 0.001},
                ts=_ts("confirm", [_q("Q-TERR-VISIBLE-001", True, shot="p2:view")]))


def _close_in():
    """Camera steeper and lower, so that one removed block is a visible part of the picture
    (from the default camera a single block changes 0 pixels: measured, see V3.md)."""
    return [_turn(pitch=-0.5, frames=5), _move(up=-1, frames=20), _w(10)]


def _edit(material, label):
    def t(rng, k):
        steps = [_w(SETTLE)] + _close_in() + _shot("before") + [{"op": "edit", "material": material}, _w(20)] + _shot("after")
        return dict(camera="close", operation=label, phases=[{"name": "p1", "create": True, "steps": steps}],
                    expected={"edits_hit": 1, "bricks_changed_min": 1, "before": "p1:before", "after": "p1:after"},
                    ts=_ts("confirm", [_q("Q-EDIT-MISSING-001", True, before="p1:before", after="p1:after")]) if label == "remove" else _ts("none", []))
    return t


def _series_steps(n):
    steps = [_w(SETTLE)] + _close_in() + _shot("before")
    for _ in range(n):
        steps += [{"op": "edit", "material": 0}, _w(8), _turn(yaw=0.06, frames=2)]
    return steps + [_w(12)] + _shot("after")


def t_edit_series(rng, k):
    n = 2 + k
    return dict(camera="close", operation="remove_series", phases=[{"name": "p1", "create": True, "steps": _series_steps(n)}],
                expected={"edits_hit": n, "bricks_changed_min": 1, "before": "p1:before", "after": "p1:after"}, ts=_ts("none", []))


def t_edit_persist(rng, k):
    n = 3 + k
    p1 = _series_steps(n)
    # Second run: the same camera path with the edits replaced by waits, one picture where `after` was taken.
    p2 = []
    for st in p1:
        if st["op"] == "edit":
            p2.append(_w(1))
        elif st["op"] == "shot":
            if st["name"] == "after":
                p2.append({"op": "shot", "name": "reloaded"})
        else:
            p2.append(st)
    return dict(camera="close", operation="remove_series_then_reload",
                phases=[{"name": "p1", "create": True, "steps": p1}, {"name": "p2", "create": False, "same_world_as": "p1", "steps": p2}],
                expected={"edits_hit": n, "bricks_changed_min": 1, "before": "p1:before", "after": "p1:after",
                          "persisted": {"after": "p1:after", "reloaded": "p2:reloaded", "pixel_diff_max": 0.001}},
                ts=_ts("none", []))


def _occ(kind, role, qexpect, color, rect_for, injected=True, control=False):
    def t(rng, k):
        w = 0.12 + 0.06 * k
        occ = {"op": "occlude", "kind": kind, "color": color}
        if kind == "panel":
            occ["rect"] = rect_for(w)
        else:
            occ.update(w=1.0 + 0.5 * k, h=0.7 + 0.3 * k, distance=1.5)
        steps = [_w(SETTLE)] + _base(rng, k) + _shot("ref") + [occ, _w(3)] + _shot("shot")
        # `fault_visible` compares with the reference picture and only proves that the overlay is really on
        # screen (label validity). Whether a SINGLE picture shows a blocked centre is a visual judgement:
        # a flat-colour test of the centre was tried and wrongly fired on healthy flat ground (see V3.md).
        exp = {"bricks_min": 1, "fault_visible": ["p1:ref", "p1:shot"] if not control else None, "shot": "p1:shot",
               "centre_flat_info": True}
        return dict(camera="default", injected_fault=("occluder_" + kind) if injected else None, needs_visual=True,
                    phases=[{"name": "p1", "create": True, "steps": steps}], expected=exp,
                    ts=_ts(role, [_q("Q-OCC-CENTER-001", qexpect, shot="p1:shot")]))
    return t


def _centre(w):
    return [0.5 - w * 1.5, 0.5 - w * 1.2, 0.5 + w * 1.5, 0.5 + w * 1.2]


TEMPLATES = {
    # name: (category, function)
    "cam_top": ("CAMERA", t_cam_top), "cam_up": ("CAMERA", t_cam_up), "cam_horizontal": ("CAMERA", t_cam_horizontal),
    "cam_angled": ("CAMERA", t_cam_angled), "cam_rotate": ("CAMERA", t_cam_rotate), "cam_move": ("CAMERA", t_cam_move),
    "world_view": ("WORLD", t_world_view),
    "world_missing_brick": ("WORLD", _fault("missing_brick", "Q-TERR-GAP-001")),
    "world_hidden_area": ("WORLD", _fault("hidden_area", "Q-TERR-GAP-001")),
    "world_wrong_material": ("WORLD", _fault("wrong_material", "Q-MAT-PATCH-001")),
    "world_partial_load": ("WORLD", t_world_partial), "world_restore": ("WORLD", t_world_restore),
    "edit_remove": ("EDITING", _edit(0, "remove")), "edit_place": ("EDITING", _edit(5, "place")),
    "edit_series": ("EDITING", t_edit_series), "edit_persist": ("EDITING", t_edit_persist),
    "occ_panel_opaque": ("OCCLUSION", _occ("panel", "primary", True, [0.08, 0.08, 0.08, 1.0], _centre)),
    "occ_panel_translucent": ("OCCLUSION", _occ("panel", "primary", True, [0.5, 0.5, 0.5, 0.45], _centre)),
    "occ_object_in_front": ("OCCLUSION", _occ("object", "primary", True, [0.8, 0.2, 0.6, 1.0], None)),
    "occ_hud_corner": ("OCCLUSION", _occ("panel", "primary", False, [0.1, 0.1, 0.1, 1.0], lambda w: [0.02, 0.02, 0.2, 0.1], injected=False, control=True)),
}

PROFILES = {
    "smoke": ["cam_top", "cam_horizontal", "world_view", "world_missing_brick", "edit_remove", "occ_panel_opaque"],
    "gameplay": ["cam_top", "cam_up", "cam_horizontal", "cam_angled", "cam_rotate", "cam_move", "edit_remove", "edit_place",
                 "edit_series", "occ_panel_opaque", "occ_panel_translucent", "occ_object_in_front", "occ_hud_corner"],
    "regression": ["world_partial_load", "world_restore", "edit_persist", "world_hidden_area", "world_wrong_material", "world_missing_brick"],
    "full": list(TEMPLATES),
}


def canonical(obj):
    return json.dumps(obj, sort_keys=True, separators=(",", ":"))


def make_scenario(template, index, seed, intensity=2, world_seeds=None, gen_version=GENERATOR_VERSION):
    """One scenario: depends only on its arguments (never on other scenarios)."""
    category, fn = TEMPLATES[template]
    rng = random.Random(f"cubeqa|{gen_version}|{seed}|{index}|{template}")
    body = fn(rng, max(1, min(3, intensity)))
    sid = f"{category}-{index + 1:03d}"
    world_seed = world_seeds[index % len(world_seeds)] if world_seeds else 1000 + rng.randrange(9000)
    scen = {"id": sid, "category": category, "template": template, "generator_version": gen_version,
            "gen_seed": seed, "index": index, "world_seed": world_seed, "intensity": intensity,
            "camera": body["camera"], "injected_fault": body.get("injected_fault"), "needs_visual": body.get("needs_visual", False),
            "operation": body.get("operation"), "phases": body["phases"], "expected": body["expected"], "typesafe": body["ts"]}
    scen["fingerprint"] = hashlib.sha256(canonical(scen).encode()).hexdigest()[:12]
    return scen


def generate(count, seed, profile="smoke", categories=None, intensity=2, world_seeds=None, templates=None):
    """`count` scenarios cycling over the templates of the profile (fixed order, no hidden shuffle).
    `seed` is an int or a list of ints (scenario i uses seeds[i % len(seeds)])."""
    seeds = seed if isinstance(seed, (list, tuple)) else [seed]
    names = templates or PROFILES[profile]
    if categories:
        names = [n for n in names if TEMPLATES[n][0] in categories]
    if not names:
        raise ValueError("no template matches the selected profile and categories")
    return [make_scenario(names[i % len(names)], i, seeds[i % len(seeds)], intensity, world_seeds) for i in range(count)]


def parse_checks(text):
    """`camera,collision,terrain` -> (set of categories, list of unsupported names that were asked for)."""
    cats, unsupported = set(), []
    for part in [p.strip().lower() for p in text.split(",") if p.strip()]:
        if part in CATEGORIES:
            cats.add(CATEGORIES[part])
        elif part in ("collision", "camera_penetration"):
            unsupported.append(part)
        else:
            raise ValueError(f"unknown check '{part}' (known: {', '.join(sorted(CATEGORIES))}, collision)")
    return cats, unsupported
