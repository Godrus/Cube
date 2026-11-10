"""Runs v3 scenarios in the real Godot scene (godot/main.gd, `--scenario`). Local only, no network.

One Godot process per phase. Phases that share a world (`same_world_as`) reopen the same world
directory, so a restart really reloads the saved world. Returns the telemetry of every phase; a
phase that fails is reported with an `error`, never silently skipped.
"""

import json
import os
import shutil
import subprocess
import threading
import time

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
GODOT_DEFAULT = r"D:\Downloads\Projects\Engine\Godot_v4.2.2-stable_mono_win64\Godot_v4.2.2-stable_mono_win64_console.exe"


def ensure_imported(exe):
    """A fresh checkout (or a new git worktree) has no godot/.godot cache, so Godot does not know
    the native extension and every scene runs with a placeholder instead of CubeWorld (a frozen
    picture, errors like "Nonexistent function 'frame' in base 'Node'"). One import run fixes it."""
    cache = os.path.join(ROOT, "godot", ".godot", "extension_list.cfg")
    if not os.path.exists(cache):
        subprocess.run([exe, "--headless", "--path", os.path.join(ROOT, "godot"), "--import"],
                       capture_output=True, text=True, timeout=300)


def godot_exe():
    exe = os.environ.get("GODOT_BIN", GODOT_DEFAULT)
    return exe if os.path.exists(exe) else None


def _working_set_mb():
    """Largest working set (MB) among running Godot processes (Windows; 0 elsewhere)."""
    try:
        r = subprocess.run(["powershell", "-NoProfile", "-Command",
                            "(Get-Process -Name 'Godot*' -ErrorAction SilentlyContinue | Measure-Object WorkingSet64 -Maximum).Maximum"],
                           capture_output=True, text=True, timeout=20)
        return float(r.stdout.strip() or 0) / 1e6
    except Exception:
        return 0.0


def run_scenario(scen, out_dir, timeout=180, extra_engine_args=(), memory_log=None):
    """Returns {phase: telemetry dict or {"error": text}}. Pictures land in out_dir/<phase>/."""
    exe = godot_exe()
    if exe is None:
        return {p["name"]: {"error": "Godot binary not found (set GODOT_BIN)"} for p in scen["phases"]}
    ensure_imported(exe)
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
            json.dump({"out_dir": pdir.replace("\\", "/"), "steps": ph["steps"], "hud": bool(scen.get("hud", False)), "creatures": bool(scen.get("creatures", False))}, f)
        for stale in ("telemetry.json",):
            if os.path.exists(os.path.join(pdir, stale)):
                os.remove(os.path.join(pdir, stale))
        cmd = [exe, "--path", os.path.join(ROOT, "godot"), "--rendering-driver", "opengl3", *extra_engine_args, "--",
               "--world", wdir, "--seed", str(scen["world_seed"]), "--scenario", spec]
        try:
            if memory_log is None:
                r = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout)
            else:
                # Same run, with the process memory sampled every few seconds into memory_log.
                done = threading.Event()

                def sampler():
                    t0 = time.time()
                    while not done.wait(3.0):
                        memory_log.append((round(time.time() - t0, 1), round(_working_set_mb(), 1)))

                th = threading.Thread(target=sampler, daemon=True)
                th.start()
                try:
                    r = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout)
                finally:
                    done.set()
                    th.join(5)
            tel_path = os.path.join(pdir, "telemetry.json")
            if r.returncode != 0 or not os.path.exists(tel_path):
                result[name] = {"error": f"Godot exit {r.returncode}: {(r.stdout or '')[-300:]}"}
                continue
            with open(tel_path, encoding="utf-8") as f:
                result[name] = json.load(f)
        except subprocess.TimeoutExpired:
            where = ""
            try:
                with open(os.path.join(pdir, "progress.txt"), encoding="utf-8") as pf:
                    where = " | last progress: " + pf.read().strip()
            except OSError:
                where = " | no progress file (the scene never started the scenario: native extension not loaded?)"
            result[name] = {"error": f"Godot did not finish within {timeout}s" + where}
    return result
