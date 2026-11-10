"""Runs v3 scenarios in the real Godot scene (godot/main.gd, `--scenario`). Local only, no network.

One Godot process per phase. Phases that share a world (`same_world_as`) reopen the same world
directory, so a restart really reloads the saved world. Returns the telemetry of every phase; a
phase that fails is reported with an `error`, never silently skipped.
"""

import json
import os
import shutil
import subprocess

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
GODOT_DEFAULT = r"D:\Downloads\Projects\Engine\Godot_v4.2.2-stable_mono_win64\Godot_v4.2.2-stable_mono_win64_console.exe"


def godot_exe():
    exe = os.environ.get("GODOT_BIN", GODOT_DEFAULT)
    return exe if os.path.exists(exe) else None


def run_scenario(scen, out_dir, timeout=180, extra_engine_args=()):
    """Returns {phase: telemetry dict or {"error": text}}. Pictures land in out_dir/<phase>/."""
    exe = godot_exe()
    if exe is None:
        return {p["name"]: {"error": "Godot binary not found (set GODOT_BIN)"} for p in scen["phases"]}
    result, worlds = {}, {}
    for ph in scen["phases"]:
        name = ph["name"]
        pdir = os.path.abspath(os.path.join(out_dir, name))
        os.makedirs(pdir, exist_ok=True)
        wdir = worlds.get(ph.get("same_world_as")) or os.path.abspath(os.path.join(out_dir, "world-" + name))
        if ph.get("create", True) and os.path.exists(wdir):
            shutil.rmtree(wdir)
        worlds[name] = wdir
        spec = os.path.join(pdir, "scenario.json")
        with open(spec, "w", encoding="utf-8") as f:
            json.dump({"out_dir": pdir.replace("\\", "/"), "steps": ph["steps"]}, f)
        for stale in ("telemetry.json",):
            if os.path.exists(os.path.join(pdir, stale)):
                os.remove(os.path.join(pdir, stale))
        cmd = [exe, "--path", os.path.join(ROOT, "godot"), "--rendering-driver", "opengl3", *extra_engine_args, "--",
               "--world", wdir, "--seed", str(scen["world_seed"]), "--scenario", spec]
        try:
            r = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout)
            tel_path = os.path.join(pdir, "telemetry.json")
            if r.returncode != 0 or not os.path.exists(tel_path):
                result[name] = {"error": f"Godot exit {r.returncode}: {(r.stdout or '')[-300:]}"}
                continue
            with open(tel_path, encoding="utf-8") as f:
                result[name] = json.load(f)
        except subprocess.TimeoutExpired:
            result[name] = {"error": f"Godot did not finish within {timeout}s"}
    return result
