"""Creates the v2 fixtures from the REAL Godot scene with controlled, reproducible faults.

Run:  python make_godot_fixtures.py        (needs Godot 4.2.2; set GODOT_BIN if it is elsewhere;
                                            opens a window for a few seconds per picture)

Rules (fixed before any model sees a picture):
- the production renderer and the Rust world are never changed; faults are applied to the SCENE by
  the QA options of godot/main.gd (free a brick node, hide bricks, tint a brick, stop delivering
  batches, skip the update after an edit);
- every picture's ground-truth label is written by this script from what was done, never from a
  model answer: the fault, the label, the visible defect region (pixel difference to the good
  picture with the same seed and camera) and the split (calibration or evaluation, fixed by seed);
- a fault picture whose pixel difference is below 0.2 % of the frame is recorded as `visible: false`
  and kept out of both sets (decided by pixels, not by Jev).
"""

import hashlib
import json
import os
import shutil
import subprocess
import sys

import pngio

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
OUT = os.path.join(HERE, "fixtures-v2")
GODOT_DEFAULT = r"D:\Downloads\Projects\Engine\Godot_v4.2.2-stable_mono_win64\Godot_v4.2.2-stable_mono_win64_console.exe"
MIN_VISIBLE = 0.002
MIN_EDIT = 0.0002

# Fixed plan. Seeds 101-104 calibrate, 105-108 evaluate; 301-303 calibrate, 304-306 evaluate.
FAULTS = ["missing_brick", "hidden_area", "wrong_material", "partial_load"]
SCENE_SEEDS = {101: "calibration", 102: "calibration", 103: "calibration", 104: "calibration",
               105: "evaluation", 106: "evaluation", 107: "evaluation", 108: "evaluation"}
EDIT_SEEDS = {301: ("normal", "calibration"), 302: ("stale", "calibration"), 303: ("normal", "calibration"),
              304: ("stale", "calibration"), 305: ("normal", "calibration"),
              306: ("stale", "evaluation"), 307: ("normal", "evaluation"), 308: ("stale", "evaluation"),
              309: ("normal", "evaluation"), 310: ("stale", "evaluation")}


def godot_run(args, world_dir):
    exe = os.environ.get("GODOT_BIN", GODOT_DEFAULT)
    if os.path.exists(world_dir):
        shutil.rmtree(world_dir)
    cmd = [exe, "--path", os.path.join(ROOT, "godot"), "--rendering-driver", "opengl3", "--",
           "--world", world_dir] + args
    r = subprocess.run(cmd, capture_output=True, text=True, timeout=240)
    if r.returncode != 0:
        raise SystemExit(f"Godot failed ({r.returncode}): {' '.join(args)}\n{r.stdout[-800:]}")


def load_rgb(path):
    with open(path, "rb") as f:
        w, h, c, px = pngio.read_png(f.read())
    return w, h, pngio.to_rgb(w, h, c, px)


def main():
    os.makedirs(OUT, exist_ok=True)
    tmp = os.path.join(ROOT, "target", "qa-fixtures")
    manifest = []

    for j, (seed, split) in enumerate(SCENE_SEEDS.items()):
        yaw, pitch = (j % 4) * 0.5 - 0.75, -0.1 if j % 2 else 0.05
        cam = ["--yaw", str(yaw), "--pitch", str(pitch)]
        base = f"seed{seed}-good"
        godot_run(["--screenshot", os.path.join(OUT, base + ".png"), "--truth", os.path.join(OUT, base + ".truth.json"),
                   "--seed", str(seed)] + cam, os.path.join(tmp, "w"))
        entries = [(base, "good", None)]
        for fault in (FAULTS[j % 4], FAULTS[(j + 1) % 4]):
            name = f"seed{seed}-{fault}"
            godot_run(["--screenshot", os.path.join(OUT, name + ".png"), "--truth", os.path.join(OUT, name + ".truth.json"),
                       "--seed", str(seed), "--fault", fault] + cam, os.path.join(tmp, "w"))
            entries.append((name, "defective", fault))
        w, h, good = load_rgb(os.path.join(OUT, base + ".png"))
        for name, label, fault in entries:
            tpath = os.path.join(OUT, name + ".truth.json")
            with open(tpath, encoding="utf-8") as f:
                truth = json.load(f)
            visible, box, frac = True, None, 0.0
            if fault:
                _, _, img = load_rgb(os.path.join(OUT, name + ".png"))
                box, frac = pngio.diff_box(w, h, good, img)
                visible = frac >= MIN_VISIBLE
            truth.update(split=split, set_label=label, fault_kind=fault or "none",
                         defect_box=box, changed_fraction=round(frac, 5), visible=visible)
            with open(tpath, "w", encoding="utf-8") as f:
                json.dump(truth, f, indent=2)
            manifest.append({"id": name, "kind": "scene", "image": name + ".png", "truth": name + ".truth.json",
                             "split": split if visible else "excluded", "label": label, "fault": fault or "none",
                             "visible": visible})
            print("scene", name, label, "changed %.4f" % frac, "visible" if visible else "EXCLUDED")

    for seed, (mode, split) in EDIT_SEEDS.items():
        name = f"edit{seed}-{mode}"
        before, after = f"{name}-before.png", f"{name}-after.png"
        args = ["--edit-shots", os.path.join(OUT, before), os.path.join(OUT, after),
                "--truth", os.path.join(OUT, name + ".truth.json"), "--seed", str(seed)]
        if mode == "stale":
            args += ["--fault", "stale"]
        godot_run(args, os.path.join(tmp, "w"))
        w, h, a = load_rgb(os.path.join(OUT, before))
        _, _, b = load_rgb(os.path.join(OUT, after))
        box, frac = pngio.diff_box(w, h, a, b)
        tpath = os.path.join(OUT, name + ".truth.json")
        with open(tpath, encoding="utf-8") as f:
            truth = json.load(f)
        # Pre-registered rules (decided by what the scene did and by pixels, never by Jev): the edit
        # must have hit a block; a normal edit must change at least 0.02 % of the frame (about 180
        # pixels). A stale pair (edit applied, scene not updated) must show no pixel change at all.
        hit = truth.get("edit_hit", False)
        visible = hit and (frac >= MIN_EDIT if mode == "normal" else frac == 0.0)
        truth.update(split=split if visible else "excluded", operation="remove one block under the crosshair",
                     defect_box=box, changed_fraction=round(frac, 5), visible=visible)
        with open(tpath, "w", encoding="utf-8") as f:
            json.dump(truth, f, indent=2)
        manifest.append({"id": name, "kind": "edit", "before": before, "after": after, "truth": name + ".truth.json",
                         "split": split if visible else "excluded", "label": truth["label"], "fault": truth["fault"],
                         "visible": visible})
        print("edit", name, truth["label"], "changed %.5f" % frac)

    with open(os.path.join(OUT, "manifest.json"), "w", encoding="utf-8") as f:
        json.dump({"version": 2, "min_visible_fraction": MIN_VISIBLE, "items": manifest}, f, indent=2)
    print("manifest:", len(manifest), "items")


if __name__ == "__main__":
    sys.exit(main())
