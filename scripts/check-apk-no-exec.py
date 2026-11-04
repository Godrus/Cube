#!/usr/bin/env python3
"""Checks exported Godot APKs of the world-generation spike.

  python scripts/check-apk-no-exec.py godot-client/build/client-faces-arm64.apk [more.apk ...]

For every APK: no packaged GDScript (assets/*.gdc) may contain the identifiers `execute` or `getprop`
(child processes / shell property lookups), and a client APK (feature `worldclient`) must not contain the
test scripts (`world_smoke`, `world_test`). Exit code 1 on any violation. The engine and the Rust binding
libraries contain the *names* of the OS API (for example `create_process`); only GDScript calls matter here.
"""
import sys
import zipfile

FORBIDDEN_IN_SCRIPTS = ("execute", "getprop", "create_process", "shell_open")
TEST_FILES = ("world_smoke", "world_test")


def check(apk: str) -> list[str]:
    problems = []
    z = zipfile.ZipFile(apk)
    names = z.namelist()
    scripts = [n for n in names if n.startswith("assets/") and (n.endswith(".gdc") or n.endswith(".gd"))]
    if not scripts:
        problems.append("no packaged scripts found (unexpected APK layout)")
    for n in scripts:
        text = z.read(n).decode("latin1").lower()
        for bad in FORBIDDEN_IN_SCRIPTS:
            if bad in text:
                problems.append(f"{n} contains `{bad}`")
    is_client = "client-" in apk.replace("\\", "/").split("/")[-1]
    if is_client:
        for n in names:
            if any(t in n for t in TEST_FILES):
                problems.append(f"client APK ships test file {n}")
    return problems


def main() -> int:
    bad = 0
    for apk in sys.argv[1:]:
        problems = check(apk)
        print(("FAIL " if problems else "ok   ") + apk)
        for p in problems:
            print("     " + p)
        bad += bool(problems)
    return 1 if bad or len(sys.argv) < 2 else 0


if __name__ == "__main__":
    sys.exit(main())
