"""Checks that the v2 frozen files still match FROZEN.json, independent of line endings and of later,
unrelated additions. FROZEN.json itself is never edited (the experiment's record stays as it was).

Two reasons the raw hashes in FROZEN.json did not reproduce:
1. they were computed on a Windows checkout (CRLF); the same file in git or on Linux has LF;
2. cube_visual_qa.py later gained the `suite` subcommand (v3).

So: every file is hashed after normalising line endings to LF and compared with BOTH FROZEN.json
(CRLF or LF form) and the git blob at the freezing commit; for cube_visual_qa.py only the functions that
v2 used are compared (their source text at the freezing commit against now).

Usage: python verify_frozen.py        (exit code 0 = frozen v2 behaviour unchanged)
"""

import ast
import hashlib
import json
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
FROZEN_COMMIT = "a4952c9"
# functions of cube_visual_qa.py that the v2 calibration and evaluation runs used
V2_FUNCTIONS = ["load_questionnaire", "_load_one", "validate_questionnaire", "check_image", "build_request", "multipart",
                "describe_request", "send", "judge_noul", "judge_choice", "resolve_expect", "evaluate", "prepare_image",
                "run_one", "load_dotenv", "require_live", "v2_items", "targets_typesafe", "cmd_v2", "save_run", "markdown_report"]
CONSTANTS = ["ENDPOINT", "DEFAULT_MODEL", "YES", "NO", "V2_SCENE", "V2_EDIT", "EDIT_CROP"]


def sha(data):
    return hashlib.sha256(data).hexdigest()


def lf(data):
    return data.replace(b"\r\n", b"\n")


def crlf(data):
    return lf(data).replace(b"\n", b"\r\n")


def git_blob(commit, rel):
    return subprocess.run(["git", "show", f"{commit}:tools/visual-qa/{rel}"], capture_output=True, cwd=ROOT, check=True).stdout


def segments(source):
    text = source.decode("utf-8")
    tree = ast.parse(text)
    out = {}
    for node in tree.body:
        if isinstance(node, ast.FunctionDef) and node.name in V2_FUNCTIONS:
            out[node.name] = sha(ast.get_source_segment(text, node).replace("\r\n", "\n").encode())
        if isinstance(node, ast.Assign) and any(getattr(t, "id", None) in CONSTANTS for t in node.targets):
            out[node.targets[0].id] = sha(ast.get_source_segment(text, node).encode())
    return out


def main():
    with open(os.path.join(HERE, "FROZEN.json"), encoding="utf-8") as f:
        frozen = json.load(f)["sha256"]
    ok = True
    for name, want in frozen.items():
        rel = name.replace("\\", "/")
        with open(os.path.join(HERE, rel), "rb") as f:
            now = f.read()
        try:
            then = git_blob(FROZEN_COMMIT, rel)
        except subprocess.CalledProcessError:
            print(f"?? {rel}: not in git at {FROZEN_COMMIT}")
            ok = False
            continue
        recorded_matches = want in (sha(lf(then)), sha(crlf(then)), sha(then))
        if rel == "cube_visual_qa.py":
            a, b = segments(then), segments(now)
            same = a == b and len(a) >= len(V2_FUNCTIONS)
            print(f"{'ok' if recorded_matches and same else 'FAIL'}  {rel}: FROZEN.json hash matches the file at {FROZEN_COMMIT}: {recorded_matches}; "
                  f"{len(a)} v2 functions and constants unchanged since: {same} (the file as a whole changed: suite subcommand, v3)")
            ok &= recorded_matches and same
        else:
            same = sha(lf(now)) == sha(lf(then))
            print(f"{'ok' if recorded_matches and same else 'FAIL'}  {rel}: recorded hash matches {FROZEN_COMMIT} (LF or CRLF form): {recorded_matches}; unchanged since: {same}")
            ok &= recorded_matches and same
    print("frozen v2 files:", "OK" if ok else "MISMATCH")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
