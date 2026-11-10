"""Gameplay smoke in the REAL Godot scene: the player falls, lands, walks, jumps, digs, quits and
continues. Uses the scenario hooks of godot/main.gd (QA only) and the Rust player through the
adapter; nothing is mocked. Not a QA-of-pictures tool and no network.

Run:  python walk_smoke.py        (needs Godot 4.2.2; set GODOT_BIN if it is elsewhere)
"""

import json
import math
import os
import sys
import tempfile

import godot_runner

W = lambda n: {"op": "wait", "frames": n}


def scenario(seed=7):
    p1 = [W(20), {"op": "mode", "walk": True}, {"op": "probe", "name": "spawn"}, W(240), {"op": "probe", "name": "landed"},
          {"op": "turn", "yaw": 0.0, "pitch": 0.6, "frames": 3},
          {"op": "move", "fwd": 1, "frames": 150}, W(10), {"op": "probe", "name": "walked"},
          {"op": "move", "up": 1, "frames": 1}, W(8), {"op": "probe", "name": "air"}, W(90), {"op": "probe", "name": "back_on_ground"},
          {"op": "mode", "walk": True, "third": True}, W(5), {"op": "probe", "name": "third_person"},
          {"op": "mode", "walk": True, "third": False}, W(5),
          {"op": "turn", "yaw": 0.0, "pitch": -3.0, "frames": 3}, W(20), {"op": "probe", "name": "before_dig"},
          {"op": "edit", "material": 5}, {"op": "edit", "material": 0}, W(120), {"op": "probe", "name": "dug"},
          {"op": "turn", "yaw": 3.1416, "pitch": 3.0, "frames": 3}, {"op": "turn", "yaw": 0.0, "pitch": -0.5, "frames": 3},
          {"op": "move", "fwd": 1, "up": 1, "frames": 1800}, W(30), {"op": "probe", "name": "far"},
          {"op": "turn", "yaw": 0.0, "pitch": -3.0, "frames": 3}, W(20), {"op": "probe", "name": "far_before_dig"},
          {"op": "edit", "material": 0}, W(120), {"op": "probe", "name": "far_dug"},
          {"op": "probe", "name": "final"}]
    p2 = [W(150), {"op": "probe", "name": "restored"}]
    return {"world_seed": seed, "phases": [{"name": "p1", "create": True, "steps": p1},
                                           {"name": "p2", "create": False, "same_world_as": "p1", "steps": p2}]}


def probes(tel, phase):
    return {e["name"]: e for e in tel[phase]["events"] if e["op"] == "probe"}


def main():
    if godot_runner.godot_exe() is None:
        print("Godot not found (GODOT_BIN)")
        return 2
    out = tempfile.mkdtemp(prefix="cube-walk-smoke-")
    tel = godot_runner.run_scenario(scenario(), out, timeout=240)
    for ph, t in tel.items():
        if "events" not in t:
            print("Godot phase failed:", ph, t)
            return 1
    p, q = probes(tel, "p1"), probes(tel, "p2")
    edits = [e for e in tel["p1"]["events"] if e["op"] == "edit"]
    d = lambda a, b: math.hypot(a["feet"][0] - b["feet"][0], a["feet"][2] - b["feet"][2])
    checks = []

    def check(name, ok, detail):
        checks.append((name, ok, detail))
        print(("ok   " if ok else "FAIL ") + name + ": " + detail)

    check("walking mode on", p["spawn"]["walking"], "spawn feet y %.1f" % p["spawn"]["feet"][1])
    check("gravity: fell and landed", p["landed"]["grounded"] and p["landed"]["feet"][1] < p["spawn"]["feet"][1] - 3,
          "y %.1f -> %.1f" % (p["spawn"]["feet"][1], p["landed"]["feet"][1]))
    check("first-person camera sits at the eye", abs(p["landed"]["camera_position"][1] - p["landed"]["feet"][1] - 1.62) < 0.01,
          "eye offset %.2f" % (p["landed"]["camera_position"][1] - p["landed"]["feet"][1]))
    check("walked forward", d(p["landed"], p["walked"]) > 5.0, "%.1f voxels" % d(p["landed"], p["walked"]))
    check("jumped and landed again", (not p["air"]["grounded"]) and p["air"]["feet"][1] > p["walked"]["feet"][1] + 0.3 and p["back_on_ground"]["grounded"],
          "air %.2f above" % (p["air"]["feet"][1] - p["walked"]["feet"][1]))
    boom = math.dist(p["third_person"]["camera_position"], [p["third_person"]["feet"][0], p["third_person"]["feet"][1] + 1.62, p["third_person"]["feet"][2]])
    check("third-person camera behind the player", 0.3 < boom <= 4.01, "boom %.2f" % boom)
    check("block placed into the body refused, removal hit", len(edits) >= 2 and (not edits[0]["hit"]) and edits[1]["hit"],
          "place hit=%s remove hit=%s" % (edits[0]["hit"] if edits else None, edits[1]["hit"] if len(edits) > 1 else None))
    drop = p["before_dig"]["feet"][1] - p["dug"]["feet"][1]
    check("digging under the feet drops the player one voxel", p["dug"]["grounded"] and abs(drop - 1.0) < 0.3, "dropped %.2f" % drop)
    check("never fell out of the world", all(e["feet"][1] > -64 for e in tel["p1"]["events"]), "min y %.1f" % min(e["feet"][1] for e in tel["p1"]["events"]))
    far = math.hypot(p["far"]["feet"][0] - p["spawn"]["feet"][0], p["far"]["feet"][2] - p["spawn"]["feet"][2])
    check("streaming: walked far beyond the initial area and stood on generated ground", far > 100 and p["far"]["grounded"] and p["far"]["bricks"] > 50,
          "%.0f voxels from the spawn, %d bricks in the scene" % (far, p["far"]["bricks"]))
    # (The player may stand on two columns far from the origin, so one removed block need not drop them;
    # persistence of a far edit is proved in the Rust test an_edit_far_from_the_start_survives_a_restart.)
    check("an edit far from the start hits and rebuilds geometry", edits[-1]["hit"] and edits[-1]["bricks_changed"] > 0,
          "hit=%s, %d bricks rebuilt" % (edits[-1]["hit"], edits[-1]["bricks_changed"]))
    r, f = q["restored"], p["final"]
    check("after quit and restart the player continues where they left off", r["walking"] and d(r, f) < 0.05 and abs(r["feet"][1] - f["feet"][1]) < 0.3,
          "restored %.2f voxels from where they left, dy %.2f" % (d(r, f), r["feet"][1] - f["feet"][1]))
    with open(os.path.join(out, "walk-smoke.json"), "w", encoding="utf-8") as fh:
        json.dump({"checks": [{"name": n, "ok": ok, "detail": dt} for n, ok, dt in checks]}, fh, indent=1)
    failed = [n for n, ok, _ in checks if not ok]
    print("walk smoke:", "PASS" if not failed else "FAIL " + ", ".join(failed), "(", out, ")")
    return 0 if not failed else 1


if __name__ == "__main__":
    sys.exit(main())
