"""Deterministic checks of a Godot test picture: plain code, no model, no network.

These are the checks that ordinary code does better than a language model: image size, a black or
flat frame, batches delivered, number of loaded bricks against a reference, and whether the scene
changed after an edit. TypeSafe is used only for what is hard to write as an algorithm.
"""

import collections

import pngio


def frame_stats(png):
    w, h, c, px = pngio.read_png(png)
    rgb = pngio.to_rgb(w, h, c, px)
    n = w * h
    black = 0
    colours = collections.Counter()
    for i in range(0, n, 5):  # every 5th pixel is plenty for a fraction
        r, g, b = rgb[3 * i], rgb[3 * i + 1], rgb[3 * i + 2]
        if r < 8 and g < 8 and b < 8:
            black += 1
        colours[(r >> 3, g >> 3, b >> 3)] += 1
    sampled = sum(colours.values())
    return {"width": w, "height": h, "black_fraction": black / sampled,
            "dominant_colour_fraction": colours.most_common(1)[0][1] / sampled,
            "distinct_colours": len(colours)}


def check_scene(png, truth, reference_truth=None):
    """Checks of one scene picture. Returns {name: (ok, detail)}."""
    s = frame_stats(png)
    out = {
        "size_matches_viewport": (
            [s["width"], s["height"]] == [int(truth["viewport"][0]), int(truth["viewport"][1])],
            f"{s['width']}x{s['height']}"),
        "frame_not_black": (s["black_fraction"] < 0.98, f"black {s['black_fraction']:.3f}"),
        "frame_not_flat": (s["dominant_colour_fraction"] < 0.9,
                           f"dominant colour {s['dominant_colour_fraction']:.3f}"),
        "batches_delivered": (truth["bricks_in_scene"] > 0, f"{truth['bricks_in_scene']} bricks"),
    }
    if reference_truth is not None:
        want = reference_truth["bricks_in_scene"]
        out["loaded_bricks_vs_reference"] = (
            truth["bricks_in_scene"] >= 0.95 * want, f"{truth['bricks_in_scene']} of {want}")
    return out


def check_edit(before_png, after_png, truth):
    """Did the scene change after an edit that the world accepted?"""
    wb, hb, cb, pb = pngio.read_png(before_png)
    wa, ha, ca, pa = pngio.read_png(after_png)
    if (wb, hb) != (wa, ha):
        return {"same_size": (False, "different sizes")}
    box, frac = pngio.diff_box(wb, hb, pngio.to_rgb(wb, hb, cb, pb), pngio.to_rgb(wa, ha, ca, pa))
    return {"scene_updated_after_edit": ((frac > 0) if truth.get("edit_hit") else True,
                                         f"changed fraction {frac:.5f}")}
