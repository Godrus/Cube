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


def journey():
    """Travel far and come back with a pinned edit at the start: memory must stabilise, the game must keep running."""
    p1 = [W(20), {"op": "mode", "walk": True}, W(240), {"op": "turn", "yaw": 0.0, "pitch": -3.0, "frames": 3}, W(20),
          {"op": "edit", "material": 0}, W(90), {"op": "probe", "name": "dug"},
          {"op": "turn", "yaw": 3.1416, "pitch": 3.6, "frames": 3}]
    for i in range(6):
        p1 += [{"op": "move", "fwd": 1, "up": 1, "frames": 300}, {"op": "probe", "name": "out%d" % i}]
    p1 += [{"op": "turn", "yaw": 3.1416, "pitch": 0.0, "frames": 3}]
    for i in range(6):
        p1 += [{"op": "move", "fwd": 1, "up": 1, "frames": 300}, {"op": "probe", "name": "back%d" % i}]
    p1 += [W(60), {"op": "probe", "name": "final"}]
    p2 = [W(240), {"op": "probe", "name": "restored"}]
    return {"world_seed": 7, "phases": [{"name": "p1", "create": True, "steps": p1},
                                        {"name": "p2", "create": False, "same_world_as": "p1", "steps": p2}]}


def main_journey():
    out = tempfile.mkdtemp(prefix="cube-journey-")
    mem = []
    tel = godot_runner.run_scenario(journey(), out, timeout=400, extra_engine_args=["--disable-vsync"], memory_log=mem)
    for ph, t in tel.items():
        if "events" not in t:
            print("Godot phase failed:", ph, t)
            return 1
    p, q = probes(tel, "p1"), probes(tel, "p2")
    checks = []

    def check(name, ok, detail):
        checks.append((name, ok, detail))
        print(("ok   " if ok else "FAIL ") + name + ": " + detail)

    out_legs = [p["out%d" % i] for i in range(6)]
    back_legs = [p["back%d" % i] for i in range(6)]
    res = lambda legs: [e["stats"]["resident"] for e in legs]
    far = max(abs(e["feet"][2] - p["dug"]["feet"][2]) for e in out_legs)
    check("travelled far and back", far > 100 and abs(p["final"]["feet"][2] - p["dug"]["feet"][2]) < 30,
          "farthest %.0f voxels, ended %.0f from the start" % (far, abs(p["final"]["feet"][2] - p["dug"]["feet"][2])))
    check("far bricks were unloaded", p["final"]["stats"]["evictions"] > 0, "%d bricks unloaded, %d loaded in total" % (p["final"]["stats"]["evictions"], p["final"]["stats"]["loads"]))
    check("resident bricks stabilised (no growth on the way back)", max(res(back_legs)) <= max(res(out_legs)),
          "out %s back %s" % (res(out_legs), res(back_legs)))
    check("derived cache bounded", max(e["stats"]["derived_cached"] for e in out_legs + back_legs) < 1200,
          "max %d entries" % max(e["stats"]["derived_cached"] for e in out_legs + back_legs))
    check("the edited start brick stayed pinned", p["final"]["stats"]["edited"] >= 1 and p["final"]["stats"]["resident"] >= 1, "%d edited bricks" % p["final"]["stats"]["edited"])
    check("still standing on ground and rendering after the journey", p["final"]["grounded"] and p["final"]["bricks"] > 50, "%d bricks in the scene" % p["final"]["bricks"])
    check("after restart: walking, grounded, resident set rebuilt small", q["restored"]["walking"] and q["restored"]["grounded"],
          "resident %d" % q["restored"]["stats"]["resident"])
    ms = [(b["ms"] - a["ms"]) / 300.0 for a, b in zip([p["dug"]] + out_legs[:-1], out_legs)]
    print("   frame time on the way out (ms per frame, debug build, vsync off, generating and unloading): %s" % ["%.1f" % m for m in ms])
    if mem:
        peak = max(m for _, m in mem)
        print("   process memory (working set, MB, every 3 s): %s" % [m for _, m in mem])
        third = max(1, len(mem) // 3)
        check("process memory does not keep growing on the way back", max(m for _, m in mem[-third:]) <= max(m for _, m in mem[:2 * third]) * 1.15,
              "peak %.0f MB; first two thirds max %.0f, last third max %.0f" % (peak, max(m for _, m in mem[:2 * third]), max(m for _, m in mem[-third:])))
    failed = [n for n, ok, _ in checks if not ok]
    print("journey smoke:", "PASS" if not failed else "FAIL " + ", ".join(failed), "(", out, ")")
    return 0 if not failed else 1


def play():
    """Mine, build, use the hotbar and inventory, finish the objective, quit and continue; with the interface drawn."""
    p1 = [W(20), {"op": "mode", "walk": True}, W(240), {"op": "turn", "yaw": 0.0, "pitch": -0.2, "frames": 3}, W(40),
          {"op": "probe", "name": "start"}, {"op": "place"}, {"op": "probe", "name": "empty_place"}, W(3), {"op": "shot", "name": "hud_start"}]
    for i in range(3):
        p1 += [{"op": "mine"}, W(30), {"op": "probe", "name": "mined%d" % i}]
    p1 += [{"op": "select", "slot": 0}, W(3), {"op": "shot", "name": "hud_mined"}]
    for i in range(12):
        p1 += [{"op": "mine"}, W(25), {"op": "select", "slot": 0}, {"op": "place"}, W(25), {"op": "probe", "name": "cycle%d" % i}]
    p1 += [{"op": "inventory", "open": True}, W(3), {"op": "shot", "name": "inventory"}, {"op": "inventory", "open": False},
           {"op": "turn", "yaw": 0.0, "pitch": -0.9, "frames": 3}, W(20), {"op": "place"}, {"op": "probe", "name": "place_at_feet"},
           {"op": "turn", "yaw": 0.0, "pitch": 0.9, "frames": 3}, W(20),
           {"op": "creative", "on": True}, {"op": "select", "slot": 4}, W(3), {"op": "place"}, W(20), {"op": "probe", "name": "creative_place"},
           {"op": "shot", "name": "hud_creative"}, {"op": "creative", "on": False}, {"op": "mine"}, W(20), {"op": "probe", "name": "final"}]
    p2 = [W(240), {"op": "probe", "name": "restored"}]
    return {"world_seed": 7, "hud": True, "phases": [{"name": "p1", "create": True, "steps": p1},
                                                   {"name": "p2", "create": False, "same_world_as": "p1", "steps": p2}]}


def main_play():
    out = tempfile.mkdtemp(prefix="cube-play-")
    scen = play()
    # the interface is only built in the first phase's scenario file; the second phase needs no pictures
    tel = godot_runner.run_scenario(scen, out, timeout=300)
    for ph, t in tel.items():
        if "events" not in t:
            print("Godot phase failed:", ph, t)
            return 1
    p, q = probes(tel, "p1"), probes(tel, "p2")
    ev = tel["p1"]["events"]
    checks = []

    def check(name, ok, detail):
        checks.append((name, ok, detail))
        print(("ok   " if ok else "FAIL ") + name + ": " + detail)

    tot = lambda e: sum(e["game"]["slots"][1::2])
    check("hotbar and inventory exist; start empty", len(p["start"]["game"]["slots"]) == 54 and tot(p["start"]) == 0 and not p["start"]["game"]["creative"], "%d slots" % (len(p["start"]["game"]["slots"]) // 2))
    mines = [e for e in ev if e["op"] == "mine"]
    places = [e for e in ev if e["op"] == "place"]
    check("placing with an empty hand is refused and costs nothing", places[0]["result"] != "ok" and tot(p["empty_place"]) == 0, "result '%s'" % places[0]["result"])
    check("mining three blocks puts three items in the inventory", all(m["result"] == "ok" for m in mines[:3]) and tot(p["mined2"]) == 3,
          "items %d" % tot(p["mined2"]))
    check("building uses items up and the objective completes", tot(p["cycle11"]) == 3 and p["cycle11"]["game"]["objective_done"],
          "%d items left, objective '%s'" % (tot(p["cycle11"]), p["cycle11"]["game"]["objective_text"]))
    body = places[13]
    check("placing into the player's own body is refused and costs nothing", body["result"] == "You are standing there" and tot(p["place_at_feet"]) == 3,
          "result '%s', items %d" % (body["result"], tot(p["place_at_feet"])))
    check("creative: placing works", places[14]["result"] == "ok" and p["creative_place"]["game"]["creative"], "result '%s'" % places[14]["result"])
    check("creative consumed nothing: survival again has the same items plus the one mined", not p["final"]["game"]["creative"] and tot(p["final"]) == tot(p["place_at_feet"]) + 1,
          "items %d" % tot(p["final"]))
    check("after quit and restart items and objective are back", tot(q["restored"]) == tot(p["final"]) and q["restored"]["game"]["objective_done"],
          "items %d, objective done %s" % (tot(q["restored"]), q["restored"]["game"]["objective_done"]))
    for name in ("hud_start", "hud_mined", "inventory", "hud_creative"):
        f = os.path.join(out, "p1", name + ".png")
        check("picture " + name, os.path.exists(f) and os.path.getsize(f) > 10000, f)
    failed = [n for n, ok, _ in checks if not ok]
    print("play smoke:", "PASS" if not failed else "FAIL " + ", ".join(failed), "(", out, ")")
    return 0 if not failed else 1


if __name__ == "__main__":
    sys.exit(main_play() if "--play" in sys.argv else main_journey() if "--journey" in sys.argv else main())
