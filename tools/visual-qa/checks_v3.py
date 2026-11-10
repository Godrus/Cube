"""Deterministic checks of v3 scenarios: plain code on Godot telemetry and pictures (no model).

Every property that code can prove is proved here (camera pose, number of bricks, hits, batches
changed after an edit, restore after reopening, pixel statistics). A check returns
(status, detail) with status one of pass / fail / warn / unknown / info. Nothing here claims
anything that was not measured: a missing telemetry or picture is `unknown`, never `pass`.
"""

import math

import deterministic
import pngio

FAULT_MIN_VISIBLE = 0.002  # same pre-registered rule as v2: an injected fault must change >= 0.2 % of the frame


def _wrap(a):
    return (a + math.pi) % (2 * math.pi) - math.pi


def shot_file(tel, ref):
    """'p1:top' -> path of that picture from the phase telemetry."""
    phase, name = ref.split(":")
    for e in (tel.get(phase) or {}).get("events", []):
        if e["op"] == "shot" and e["name"] == name:
            return e["file"]
    return None


def _load(path):
    with open(path, "rb") as f:
        return f.read()


def _rgb(png):
    w, h, c, px = pngio.read_png(png)
    return w, h, pngio.to_rgb(w, h, c, px)


def diff_fraction(path_a, path_b):
    wa, ha, a = _rgb(_load(path_a))
    wb, hb, b = _rgb(_load(path_b))
    if (wa, ha) != (wb, hb):
        return None
    return pngio.diff_box(wa, ha, a, b)[1]


def sky_fraction(png):
    """Share of pixels that look like the scene's sky (blue-dominant, b > g > r)."""
    w, h, rgb = _rgb(png)
    sky = n = 0
    for i in range(0, w * h, 7):
        r, g, b = rgb[3 * i], rgb[3 * i + 1], rgb[3 * i + 2]
        n += 1
        if b > g > r and b - r >= 25:
            sky += 1
    return sky / n


def centre_dominant(png, box=(0.4, 0.4, 0.6, 0.6)):
    """Fraction of the central region covered by its most common (coarsely quantised) colour."""
    w, h, rgb = _rgb(png)
    cw, ch, c = pngio.crop_enlarge(w, h, rgb, box, 1)
    counts = {}
    n = 0
    for i in range(0, cw * ch, 3):
        k = (c[3 * i] >> 4, c[3 * i + 1] >> 4, c[3 * i + 2] >> 4)
        counts[k] = counts.get(k, 0) + 1
        n += 1
    return max(counts.values()) / n


def _events(tel, phase, op):
    return [e for e in (tel.get(phase) or {}).get("events", []) if e["op"] == op]


def _start_and_end(tel, phase="p1"):
    ev = (tel.get(phase) or {}).get("events", [])
    start = next((e for e in ev if e["op"] == "probe" and e.get("name") == "start"), None)
    shots = [e for e in ev if e["op"] == "shot"]
    return start, (shots[-1] if shots else None)


def evaluate(scen, tel):
    """Returns {"status": PASS|FAIL|WARN|UNKNOWN, "checks": {name: {"status", "detail"}}, "metrics": {...}}."""
    exp = scen["expected"]
    out, metrics = {}, {}

    def put(name, status, detail):
        out[name] = {"status": status, "detail": detail}

    missing = [p["name"] for p in scen["phases"] if "events" not in (tel.get(p["name"]) or {})]
    if missing:
        put("executed", "unknown", f"no telemetry for phase(s) {', '.join(missing)}: the scenario did not run")
        return _summary(out, metrics)
    put("executed", "pass", f"{len(scen['phases'])} Godot run(s) finished with telemetry")

    p1 = scen["phases"][0]["name"]
    if "bricks_min" in exp:
        n = tel[p1]["bricks_final"]
        put("bricks_loaded", "pass" if n >= exp["bricks_min"] else "fail", f"{n} bricks in the scene (need >= {exp['bricks_min']})")

    main_ref = exp.get("shot")
    main = shot_file(tel, main_ref) if main_ref else None
    if main_ref and not main:
        put("picture_taken", "unknown", f"picture {main_ref} was not produced")
    elif main:
        png = _load(main)
        stats = deterministic.frame_stats(png)
        metrics["frame"] = stats
        put("frame_not_black", "pass" if stats["black_fraction"] < 0.98 else "fail", f"black {stats['black_fraction']:.3f}")
        if "sky_fraction_min" not in exp:
            put("frame_not_flat", "pass" if stats["dominant_colour_fraction"] < 0.9 else "fail",
                f"dominant colour {stats['dominant_colour_fraction']:.3f}")
        if "sky_fraction_min" in exp or "sky_fraction_max" in exp:
            sf = sky_fraction(png)
            metrics["sky_fraction"] = round(sf, 4)
            lo, hi = exp.get("sky_fraction_min", 0.0), exp.get("sky_fraction_max", 1.0)
            put("sky_fraction", "pass" if lo <= sf <= hi else "fail", f"sky {sf:.3f}, expected {lo}..{hi}")

    if "pitch_delta" in exp or "yaw_delta" in exp or exp.get("position_unchanged") or exp.get("moved_along_heading") or exp.get("climbed"):
        start, end = _start_and_end(tel, p1)
        if not start or not end:
            put("camera_pose", "unknown", "start probe or final picture event is missing")
        else:
            if "pitch_delta" in exp:
                got = end["camera_rotation"][0] - start["camera_rotation"][0]
                tol = exp.get("pitch_tolerance", 0.03)
                put("camera_pitch", "pass" if abs(got - exp["pitch_delta"]) <= tol else "fail",
                    f"pitch changed by {got:+.3f} rad, commanded {exp['pitch_delta']:+.3f} (tolerance {tol})")
            if "yaw_delta" in exp:
                got = _wrap(end["camera_rotation"][1] - start["camera_rotation"][1])
                want = _wrap(exp["yaw_delta"])
                tol = exp.get("yaw_tolerance", 0.03)
                put("camera_yaw", "pass" if abs(_wrap(got - want)) <= tol else "fail",
                    f"yaw changed by {got:+.3f} rad, commanded {want:+.3f} (tolerance {tol})")
            if exp.get("position_unchanged"):
                d = max(abs(a - b) for a, b in zip(start["camera_position"], end["camera_position"]))
                put("camera_position_unchanged", "pass" if d < 1e-3 else "fail", f"largest coordinate change {d:.5f}")
            if exp.get("climbed"):
                dy = end["camera_position"][1] - start["camera_position"][1]
                put("camera_climbed", "pass" if dy > 0.5 else "fail", f"height changed by {dy:+.2f}")
            if exp.get("moved_along_heading"):
                dx = end["camera_position"][0] - start["camera_position"][0]
                dz = end["camera_position"][2] - start["camera_position"][2]
                dist = math.hypot(dx, dz)
                yaw = start["camera_rotation"][1]
                cos = (dx * -math.sin(yaw) + dz * -math.cos(yaw)) / dist if dist > 1e-6 else 0.0
                put("moved_along_heading", "pass" if dist > 0.5 and cos >= 0.9 else "fail",
                    f"moved {dist:.2f} horizontally, alignment with the heading {cos:.2f}")

    if exp.get("same_view_as"):
        a, b = (shot_file(tel, r) for r in exp["same_view_as"])
        f = diff_fraction(a, b) if a and b else None
        put("same_view_after_full_turn", "unknown" if f is None else ("pass" if f < 0.002 else "fail"),
            "pictures missing" if f is None else f"{f:.5f} of the pixels differ after a full turn")

    if exp.get("fault_visible"):
        a, b = (shot_file(tel, r) for r in exp["fault_visible"])
        f = diff_fraction(a, b) if a and b else None
        metrics["fault_changed_fraction"] = None if f is None else round(f, 5)
        if f is None:
            put("injected_fault_visible", "unknown", "reference or fault picture missing")
        elif f < FAULT_MIN_VISIBLE:
            put("injected_fault_visible", "unknown", f"the injected fault changed only {f:.5f} of the frame (< {FAULT_MIN_VISIBLE}): "
                "label unreliable, scenario excluded from TypeSafe")
            metrics["fault_not_visible"] = True
        else:
            put("injected_fault_visible", "pass", f"the injected fault changed {f:.4f} of the frame")

    if exp.get("centre_flat_info") and main:
        dom = centre_dominant(_load(main))
        metrics["centre_dominant_fraction"] = round(dom, 3)
        put("centre_flat_fraction", "info", f"{dom:.2f} of the central region is one colour (information only: flat ground gives high values too)")

    if "partial_detected" in exp:
        c = exp["partial_detected"]
        ref, part = tel[c["reference"]]["bricks_final"], tel[c["partial"]]["bricks_final"]
        ratio = part / ref if ref else 0.0
        put("partial_load_detected", "pass" if ratio < c["ratio_below"] else "fail",
            f"{part} of {ref} bricks loaded ({ratio:.2f}); the detector fires below {c['ratio_below']}")

    if "restored_equal" in exp:
        c = exp["restored_equal"]
        ba, bb = tel[c["a"].split(":")[0]]["bricks_final"], tel[c["b"].split(":")[0]]["bricks_final"]
        put("restore_bricks_equal", "pass" if ba == bb else "fail", f"{ba} bricks before, {bb} after reopening")
        a, b = shot_file(tel, c["a"]), shot_file(tel, c["b"])
        f = diff_fraction(a, b) if a and b else None
        mx = exp.get("pixel_diff_max", 0.001)
        put("restore_picture_equal", "unknown" if f is None else ("pass" if f <= mx else "fail"),
            "pictures missing" if f is None else f"{f:.5f} of the pixels differ after reopening the world (limit {mx})")

    if "edits_hit" in exp:
        edits = _events(tel, p1, "edit")
        hits = sum(1 for e in edits if e.get("hit"))
        put("edits_applied", "pass" if hits == exp["edits_hit"] else "fail", f"{hits} of {len(edits)} edits hit a block, expected {exp['edits_hit']}")
        changed = sum(e.get("bricks_changed", 0) for e in edits)
        put("render_batches_updated", "pass" if changed >= exp["bricks_changed_min"] else "fail", f"{changed} brick batches rebuilt after the edits")
        a, b = shot_file(tel, exp["before"]), shot_file(tel, exp["after"])
        f = diff_fraction(a, b) if a and b else None
        metrics["edit_changed_fraction"] = None if f is None else round(f, 5)
        if f is None:
            put("picture_changed_after_edit", "unknown", "pictures missing")
        elif f > 0:
            put("picture_changed_after_edit", "pass", f"{f:.5f} of the pixels changed")
        else:
            put("picture_changed_after_edit", "warn", f"{changed} batches rebuilt but the picture is identical: the edit is not visible from this camera, or the scene was not updated")
        if "persisted" in exp:
            c = exp["persisted"]
            reloaded = shot_file(tel, c["reloaded"])
            if not f:
                put("edit_persisted", "unknown", "the edit is not visible in the picture, so a picture cannot prove persistence")
            else:
                g = diff_fraction(shot_file(tel, c["after"]), reloaded) if reloaded else None
                put("edit_persisted", "unknown" if g is None else ("pass" if g <= c["pixel_diff_max"] else "fail"),
                    "pictures missing" if g is None else f"{g:.5f} of the pixels differ between the edited and the reloaded world")
    return _summary(out, metrics)


def _summary(checks, metrics):
    st = [c["status"] for c in checks.values()]
    status = "FAIL" if "fail" in st else "UNKNOWN" if "unknown" in st else "WARN" if "warn" in st else "PASS"
    return {"status": status, "checks": checks, "metrics": metrics}
