"""Pre-registers the per-check ground truth of the v2 fixtures BEFORE any model sees them.

Rules (fixed in code, derived from what the scene was made to do and from pixels, never from a model):
- good: no gap, no off-palette patch, terrain visible.
- missing_brick / hidden_area: `gap_in_ground` is true only if the pixels that differ from the good
  picture are mostly dark background colour (the brick really exposed background); otherwise the
  check is excluded (None), because what the hole shows depends on the terrain behind it.
- wrong_material: `off_palette_patch` is true only if the changed pixels are mostly blue; else None.
- partial_load: not a TypeSafe target (the loaded-brick count is a deterministic check); the
  TypeSafe checks `gap_in_ground` and `off_palette_patch` are None.
- edit pairs: `block_missing_in_after` is true for a normal edit, false for a stale pair.

Run:  python label_fixtures.py     (writes `typesafe_checks` into fixtures-v2/manifest.json)
"""

import json
import os

import pngio

HERE = os.path.dirname(os.path.abspath(__file__))
FIX = os.path.join(HERE, "fixtures-v2")
BACKGROUND = (51, 41, 31)


def rgb(path):
    with open(path, "rb") as f:
        w, h, c, px = pngio.read_png(f.read())
    return w, h, pngio.to_rgb(w, h, c, px)


def changed_pixels(w, h, good, bad, tol=12):
    for i in range(0, w * h * 3, 3):
        if (abs(good[i] - bad[i]) > tol or abs(good[i + 1] - bad[i + 1]) > tol
                or abs(good[i + 2] - bad[i + 2]) > tol):
            yield bad[i], bad[i + 1], bad[i + 2]


def main():
    with open(os.path.join(FIX, "manifest.json"), encoding="utf-8") as f:
        manifest = json.load(f)
    items = manifest["items"]
    for it in items:
        if it["kind"] == "edit":
            it["typesafe_checks"] = {"block_missing_in_after": it["label"] == "changed"} if it["split"] != "excluded" else {}
            continue
        base = {"terrain_visible": True, "scene_kind": "block_landscape"}
        fault = it["fault"]
        checks = dict(base, gap_in_ground=None, off_palette_patch=None)
        if not it["visible"]:
            checks = {}
        elif fault == "none":
            checks.update(gap_in_ground=False, off_palette_patch=False)
        elif fault in ("missing_brick", "hidden_area", "wrong_material"):
            seed = it["id"].split("-")[0]
            w, h, good = rgb(os.path.join(FIX, f"{seed}-good.png"))
            _, _, bad = rgb(os.path.join(FIX, it["image"]))
            px = list(changed_pixels(w, h, good, bad))
            if fault == "wrong_material":
                share = sum(1 for r, g, b in px if b > r + 40 and b > g + 20) / max(1, len(px))
                checks.update(off_palette_patch=True if share > 0.5 else None, gap_in_ground=False if share > 0.5 else None)
            else:
                share = sum(1 for r, g, b in px if all(abs(c - t) < 25 for c, t in zip((r, g, b), BACKGROUND))) / max(1, len(px))
                checks.update(gap_in_ground=True if share > 0.3 else None, off_palette_patch=False if share > 0.3 else None)
            it["label_evidence"] = {"changed_pixels": len(px), "share": round(share, 3)}
        it["typesafe_checks"] = checks
    manifest["labelled_before_any_model_call"] = True
    with open(os.path.join(FIX, "manifest.json"), "w", encoding="utf-8") as f:
        json.dump(manifest, f, indent=2)
    for it in items:
        print(it["id"], it["split"], it["label"], it.get("typesafe_checks"), it.get("label_evidence", ""))


if __name__ == "__main__":
    main()
