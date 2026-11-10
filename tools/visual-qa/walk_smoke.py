"""Gameplay smoke in the REAL Godot scene: the player falls, lands, walks, jumps, digs, quits and
continues. Uses the scenario hooks of godot/main.gd (QA only) and the Rust player through the
adapter; nothing is mocked. Not a QA-of-pictures tool and no network.

Run:  python walk_smoke.py        (needs Godot 4.2.2; set GODOT_BIN if it is elsewhere)
"""

import json
import math
import os
import time
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
    check("a normal game (no --creatures) has no creatures and the combat hooks stay off", all(not e.get("creatures") for e in ev if "game" in e) and all(e["op"] not in ("attack", "fight") for e in ev), "%d events checked" % len(ev))
    for name in ("hud_start", "hud_mined", "inventory", "hud_creative"):
        f = os.path.join(out, "p1", name + ".png")
        check("picture " + name, os.path.exists(f) and os.path.getsize(f) > 10000, f)
    failed = [n for n, ok, _ in checks if not ok]
    print("play smoke:", "PASS" if not failed else "FAIL " + ", ".join(failed), "(", out, ")")
    return 0 if not failed else 1


def combat():
    """Creatures on: a slime is drawn, chases and hurts the player, the player kills it with the sword, loot arrives,
    and after quit and restart the kill and the loot are still there (the creature does not come back)."""
    p1 = [W(20), {"op": "mode", "walk": True}, W(240), {"op": "probe", "name": "start"},
          {"op": "attack"}, {"op": "probe", "name": "nothing_in_reach"}, W(3), {"op": "shot", "name": "creatures_far"},
          {"op": "fight", "frames": 330}, {"op": "shot", "name": "chase"}, {"op": "fight", "frames": 1500}, W(3), {"op": "probe", "name": "after_fight"}, {"op": "shot", "name": "after_fight"}]
    p2 = [W(240), {"op": "probe", "name": "restored"}]
    return {"world_seed": 7, "hud": True, "creatures": True,
            "phases": [{"name": "p1", "create": True, "steps": p1}, {"name": "p2", "create": False, "same_world_as": "p1", "steps": p2}]}


def combat_once(n):
    out = tempfile.mkdtemp(prefix="cube-combat-")
    t0 = time.time()
    tel = godot_runner.run_scenario(combat(), out, timeout=240)
    for ph, t in tel.items():
        if "events" not in t:
            print("FAIL godot phase %s: %s" % (ph, t.get("error", t)))
            return False
    p, q = probes(tel, "p1"), probes(tel, "p2")
    ev = tel["p1"]["events"]
    checks = []

    def check(name, ok, detail):
        checks.append((name, ok, detail))
        print(("ok   " if ok else "FAIL ") + name + ": " + detail)

    st = p["start"]
    check("full health and creatures exist near the start; the HUD shows it", st["game"]["health"] == 20 == st["game"]["max_health"] and len(st["creatures"]) >= 1 and st["hp_label"] == "HP 20/20",
          "health %s/%s, label '%s', %d creatures" % (st["game"]["health"], st["game"]["max_health"], st["hp_label"], len(st["creatures"])))
    att = [e for e in ev if e["op"] == "attack"]
    check("a swing with nothing in reach hits nothing", bool(att) and att[0]["result"] == "none", "result '%s'" % (att[0]["result"] if att else None))
    fights = [e for e in ev if e["op"] == "fight"]
    f = fights[-1] if fights else {"result": "missing", "trace": [], "hits": 0, "frames_used": 0}
    check("the bot reached the slime and killed it", f["result"] == "killed", "result '%s' after %d frames, %d hits" % (f["result"], f["frames_used"], f["hits"]))
    if f["result"] != "killed":
        for t in f["trace"][-12:]:
            print("     trace", t)
    states = set(f.get("states", [])) | {c["state"] for c in st["creatures"]}
    names = {0: "IDLE", 1: "CHASE", 2: "ATTACK"}
    check("the AI went through IDLE, CHASE and ATTACK (DEAD = removed with a message)", {0, 1, 2} <= states, "seen " + ",".join(names.get(x, str(x)) for x in sorted(states)))
    a = p["after_fight"]
    msg = a["game"]["message"]
    check("the creature's death is shown and counted (it is gone, a message says so)", "defeated" in msg and f["hits"] >= 2, "message '%s'" % msg)
    loot = sum(a["game"]["slots"][1::2])
    import re
    m = re.search(r"\+(\d+) ", msg)
    check("the inventory holds exactly the loot the message names", loot == (int(m.group(1)) if m else 0), "items %d, message '%s'" % (loot, msg))
    lowest = min(e["game"]["health"] for e in ev if "game" in e)
    check("the creature hurt the player (damage is decided in Rust)", lowest < 20, "lowest health %s; HUD at the end '%s'" % (lowest, a["hp_label"]))
    check("the HUD health equals the Rust health", a["hp_label"] == "HP %d/%d" % (a["game"]["health"], a["game"]["max_health"]), "'%s'" % a["hp_label"])
    # Displayed positions equal the Rust state in every probe (the box sits 0.4 above the feet).
    bad = 0
    total = 0
    for e in ev:
        if "creatures" not in e:
            continue
        nodes = e.get("nodes", [])
        total += len(nodes)
        for nd in nodes:
            if not any(abs(nd[0] - c["pos"][0]) < 0.01 and abs(nd[1] - 0.4 - c["pos"][1]) < 0.01 and abs(nd[2] - c["pos"][2]) < 0.01 for c in e["creatures"]):
                bad += 1
    check("each drawn creature sits where Rust says", total > 0 and bad == 0, "%d positions compared, %d off" % (total, bad))
    for name in ("creatures_far", "chase", "after_fight"):
        pic = os.path.join(out, "p1", name + ".png")
        check("picture " + name, os.path.exists(pic) and os.path.getsize(pic) > 10000, pic)
    r = q["restored"]
    check("after restart the items are back and the health is not reset by magic", sum(r["game"]["slots"][1::2]) == loot, "items %d" % sum(r["game"]["slots"][1::2]))
    if f["last_hit_pos"]:
        lx, lz = f["last_hit_pos"][0], f["last_hit_pos"][2]
        back = [c for c in r["creatures"] if abs(c["pos"][0] - lx) < 8 and abs(c["pos"][2] - lz) < 8]
        check("the killed creature did not come back", len(back) == 0, "%d creatures near the kill" % len(back))
    failed = [x for x, ok, _ in checks if not ok]
    print("combat smoke run %d: %s (%.0f s, %s)" % (n, "PASS" if not failed else "FAIL " + ", ".join(failed), time.time() - t0, out))
    return not failed


def main_combat():
    runs = int(sys.argv[sys.argv.index("--runs") + 1]) if "--runs" in sys.argv else 2
    oks = [combat_once(i + 1) for i in range(runs)]
    print("combat smoke:", "PASS" if all(oks) else "FAIL", "(%d of %d runs passed)" % (sum(oks), runs))
    return 0 if all(oks) else 1


def world_tour():
    """Fly over the land at a fixed height above the ground (seed 7) and look at each stop. Terrain-following only,
    no teleport: what a person flying would see. Phase 1 flies +Z over plains, tundra and desert; phase 2 starts
    again at the origin and flies -X over the forest."""
    south = [W(20), {"op": "mode", "walk": False}, {"op": "turn", "yaw": 3.14159265, "pitch": -0.3, "frames": 3}, W(10)]
    for name, z in (("plains", 30), ("tundra", 180), ("desert", 400)):
        south += [{"op": "hover", "frames": 4000, "height": 28, "until_z": z}, W(15), {"op": "probe", "name": name}, {"op": "shot", "name": name}]
    west = [W(20), {"op": "mode", "walk": False}, {"op": "turn", "yaw": 1.5707963, "pitch": -0.3, "frames": 3}, W(10),
            {"op": "hover", "frames": 4000, "height": 28, "until_x": -170}, W(15), {"op": "probe", "name": "forest"}, {"op": "shot", "name": "forest"}]
    return {"world_seed": 7, "hud": True, "phases": [{"name": "p1", "create": True, "steps": south},
                                                   {"name": "p2", "create": True, "steps": west}]}


def mean_rgb(path):
    import pngio
    with open(path, "rb") as f:
        w, h, ch, px = pngio.read_png(f.read())
    # the lower two thirds: land, not sky and not the interface at the top
    tot, n = [0, 0, 0], 0
    for y in range(h // 3, h - 80):
        row = y * w * ch
        for x in range(0, w, 4):
            o = row + x * ch
            tot[0] += px[o]
            tot[1] += px[o + 1]
            tot[2] += px[o + 2]
            n += 1
    return [t / n for t in tot]


def main_world():
    out = tempfile.mkdtemp(prefix="cube-world-")
    t0 = time.time()
    tel = godot_runner.run_scenario(world_tour(), out, timeout=300)
    for ph in ("p1", "p2"):
        if "events" not in tel[ph]:
            print("FAIL godot:", ph, tel[ph].get("error", tel[ph]))
            return 1
    p = dict(probes(tel, "p1"))
    p2 = probes(tel, "p2")
    p.update(p2)
    checks = []

    def check(name, ok, detail):
        checks.append((name, ok, detail))
        print(("ok   " if ok else "FAIL ") + name + ": " + detail)

    biomes = {k: p[k]["game"]["biome"] for k in p}
    pos = {k: p[k]["camera_position"] for k in p}
    for k in p:
        print("     stop %-7s biome %-7s camera z %.0f y %.0f" % (k, biomes[k], pos[k][2], pos[k][1]))
    check("the tour met all four biomes", set(biomes.values()) == {"Plains", "Forest", "Tundra", "Desert"}, ", ".join(sorted(set(biomes.values()))))
    check("the HUD names a biome everywhere", all(b != "" for b in biomes.values()), str(biomes))
    means = {}
    for k in p:
        f = os.path.join(out, "p2" if k == "forest" else "p1", k + ".png")
        check("picture " + k, os.path.exists(f) and os.path.getsize(f) > 10000, f)
        if os.path.exists(f):
            means[k] = mean_rgb(f)
            print("     mean colour %-7s %s" % (k, [round(c) for c in means[k]]))
    by_biome = {}
    for k, b in biomes.items():
        by_biome.setdefault(b, means.get(k))
    dist = lambda a, b: sum(abs(x - y) for x, y in zip(a, b))
    names = [b for b in by_biome if by_biome[b]]
    far = [(dist(by_biome[a], by_biome[b]), a, b) for i, a in enumerate(names) for b in names[i + 1:]]
    check("the pictures of different biomes look different (colour distance)", all(d > 25 for d, _, _ in far), ", ".join("%s/%s %.0f" % (a, b, d) for d, a, b in far))
    if "Desert" in by_biome and by_biome["Desert"]:
        r, g, b = by_biome["Desert"]
        check("the desert is sandy (red and green above blue)", r > b * 1.1 and g > b * 1.1, "mean %s" % [round(c) for c in by_biome["Desert"]])
    if "Tundra" in by_biome and by_biome["Tundra"]:
        r, g, b = by_biome["Tundra"]
        check("the tundra is pale (bright and not green)", min(r, g, b) > 120 and b >= g * 0.9, "mean %s" % [round(c) for c in by_biome["Tundra"]])
    failed = [x for x, ok, _ in checks if not ok]
    print("world tour: %s (%.0f s, %s)" % ("PASS" if not failed else "FAIL " + ", ".join(failed), time.time() - t0, out))
    return 0 if not failed else 1


def caves_scenario():
    """Seed 7, generator version 3: a shaft open to the sky at (98, -7, -70) and a coal voxel at (0, -54, -23) that a
    cave exposes (pinned by worldgen_v3.rs). The camera is put there by a QA-only teleport; mining the coal is the
    ordinary game action."""
    steps = [W(20), {"op": "mode", "walk": False},
             {"op": "teleport", "x": 98.5, "y": 14.0, "z": -69.5, "yaw": 0.0, "pitch": -1.3}, W(200), {"op": "probe", "name": "above"}, {"op": "shot", "name": "mouth_from_above"},
             {"op": "teleport", "x": 98.5, "y": -8.4, "z": -69.5, "yaw": 0.0, "pitch": -0.2}, W(200), {"op": "probe", "name": "in_shaft"}, {"op": "shot", "name": "in_shaft"},
             {"op": "teleport", "x": 0.5, "y": -53.4, "z": -23.5, "yaw": 3.14159265, "pitch": 0.0}, W(240), {"op": "probe", "name": "coal_seen"}, {"op": "shot", "name": "coal_wall"},
             {"op": "mine"}, W(30), {"op": "probe", "name": "coal_mined"}]
    return {"world_seed": 7, "hud": True, "phases": [{"name": "p1", "create": True, "steps": steps}]}


def main_caves():
    out = tempfile.mkdtemp(prefix="cube-caves-")
    t0 = time.time()
    tel = godot_runner.run_scenario(caves_scenario(), out, timeout=300)
    if "events" not in tel["p1"]:
        print("FAIL godot:", tel["p1"].get("error", tel["p1"]))
        return 1
    p = probes(tel, "p1")
    ev = tel["p1"]["events"]
    checks = []

    def check(name, ok, detail):
        checks.append((name, ok, detail))
        print(("ok   " if ok else "FAIL ") + name + ": " + detail)

    a, s_, c = p["above"], p["in_shaft"], p["coal_seen"]
    check("the camera really is below the surface in the shaft and in the cave", s_["camera_position"][1] < -7 and c["camera_position"][1] < -40,
          "shaft y %.1f, cave y %.1f" % (s_["camera_position"][1], c["camera_position"][1]))
    check("the world was streamed in at both places (bricks around the camera)", s_["bricks"] > 100 and c["bricks"] > 100, "%d and %d bricks in the scene" % (s_["bricks"], c["bricks"]))
    check("the crosshair is on the coal wall of the cave", c["target_material"] == 11, "target material %s (11 = coal ore)" % c["target_material"])
    mined = [e for e in ev if e["op"] == "mine"]
    check("mining the coal works and the inventory gets coal ore (item 11)", bool(mined) and mined[0]["result"] == "ok" and 11 in p["coal_mined"]["game"]["slots"][0::2],
          "result '%s', slots %s" % (mined[0]["result"] if mined else None, [x for x in p["coal_mined"]["game"]["slots"][:6]]))
    means = {}
    for name in ("mouth_from_above", "in_shaft", "coal_wall"):
        f = os.path.join(out, "p1", name + ".png")
        check("picture " + name, os.path.exists(f) and os.path.getsize(f) > 10000, f)
        means[name] = mean_rgb(f) if os.path.exists(f) else [0, 0, 0]
        print("     mean colour %-17s %s" % (name, [round(v) for v in means[name]]))
    r, g, b = means["coal_wall"]
    check("the cave picture is rock, not sky or grass (grey-brown, not blue, not green)", b < r + 25 and g < r + 25 and max(r, g, b) < 200, "mean %s" % [round(v) for v in means["coal_wall"]])
    failed = [x for x, ok, _ in checks if not ok]
    print("caves smoke: %s (%.0f s, %s)" % ("PASS" if not failed else "FAIL " + ", ".join(failed), time.time() - t0, out))
    return 0 if not failed else 1


if __name__ == "__main__":
    sys.exit(main_caves() if "--caves" in sys.argv else main_world() if "--world" in sys.argv else main_combat() if "--combat" in sys.argv else main_play() if "--play" in sys.argv else main_journey() if "--journey" in sys.argv else main())
