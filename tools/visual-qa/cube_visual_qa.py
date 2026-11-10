"""Cube Visual QA: asks TypeSafe Jev typed questions about Godot screenshots (experimental).

TypeSafe never takes part in world generation, rendering or the game loop; it only judges the
picture. Geometry invariants stay in the Rust tests. Standard library only.

Safety: no request is sent unless --live is given AND TYPESAFE_API_KEY is set; the key is read
from the environment, never printed, never written to a file; one attempt per request (no retry),
a request budget per run, a timeout. A failed call is reported as UNKNOWN, never as PASS.
"""

import argparse
import datetime
import hashlib
import json
import os
import sys
import urllib.error
import urllib.request
import uuid

import pngio

HERE = os.path.dirname(os.path.abspath(__file__))
ENDPOINT = "https://api.typesafe.ai/v1/systemone"
DEFAULT_MODEL = "jev-1.14.0"
# Documented limits (docs.typesafe.ai/images): checked before anything is sent.
MIN_SIDE, MAX_SIDE = 280, 1568
MIN_PIXELS, MAX_PIXELS = 78_400, 2_458_624
MAX_FILE = 16 * 1024 * 1024
MAX_BODY = 32 * 1024 * 1024
# Experimental thresholds (the skill's starting values); to be measured, not trusted.
YES, NO = 0.7, 0.3


class QAError(Exception):
    """A problem with the inputs (reported without a stack trace)."""


# ---------------------------------------------------------------------------------------------
# questionnaires


def load_questionnaire(name_or_path):
    path = name_or_path
    if not os.path.exists(path):
        path = os.path.join(HERE, "questions", name_or_path + ".json")
    if not os.path.exists(path):
        raise QAError(f"questionnaire not found: {name_or_path}")
    with open(path, encoding="utf-8") as f:
        q = json.load(f)
    validate_questionnaire(q)
    return q


def validate_questionnaire(q):
    for key in ("name", "images", "questions"):
        if key not in q:
            raise QAError(f"questionnaire misses '{key}'")
    names = set(q["images"])
    for qname, spec in q["questions"].items():
        if spec.get("type") not in ("noul", "choice", "box"):
            raise QAError(f"question {qname}: unsupported type")
        if spec["type"] in ("noul", "choice") and "expect" not in spec:
            raise QAError(f"question {qname}: needs 'expect'")
        if spec["type"] == "box" and "only_if" not in spec:
            raise QAError(f"question {qname}: a box needs 'only_if' (a presence noul, see the docs)")
        if spec["type"] == "choice" and not isinstance(spec.get("criteria"), dict):
            raise QAError(f"question {qname}: a choice needs criteria")
        for ref in spec.get("images_used", []):
            if ref not in names:
                raise QAError(f"question {qname}: unknown image slot {ref}")


# ---------------------------------------------------------------------------------------------
# request building


def check_image(data, label):
    if data[:8] != pngio.SIGNATURE:
        raise QAError(f"{label}: only PNG files are supported by this tool")
    w, h = pngio.dimensions(data)
    if len(data) > MAX_FILE:
        raise QAError(f"{label}: file is larger than 16 MiB")
    if not (MIN_SIDE <= w <= MAX_SIDE and MIN_SIDE <= h <= MAX_SIDE):
        raise QAError(f"{label}: {w}x{h} is outside 280..1568 pixels per side")
    if not (MIN_PIXELS <= w * h <= MAX_PIXELS):
        raise QAError(f"{label}: {w * h} pixels is outside 78400..2458624")
    return w, h


def build_request(questionnaire, images, model):
    """images: slot name -> PNG bytes. Returns (json_body, parts) where parts maps part name to
    bytes. References are unique strings; every image is referenced exactly once, in the state."""
    state, assets, parts = {}, {}, {}
    for slot in questionnaire["images"]:
        if slot not in images:
            raise QAError(f"image slot '{slot}' has no file")
        data = images[slot]
        check_image(data, slot)
        ref = f"cubeqa-{slot}-{uuid.uuid5(uuid.NAMESPACE_URL, hashlib.sha256(data).hexdigest()).hex[:12]}"
        part = f"image-{slot}"
        state[slot] = ref
        assets[ref] = {"type": "image", "part": part}
        parts[part] = data
    questions = {}
    for qname, spec in questionnaire["questions"].items():
        q = {"type": spec["type"], "instructions": spec["instructions"]}
        if "criteria" in spec:
            q["criteria"] = spec["criteria"]
        questions[qname] = q
    body = {"model": model, "state": state, "assets": assets, "questions": questions}
    return body, parts


def multipart(body, parts):
    boundary = "----cubeqa" + uuid.uuid4().hex
    out = bytearray()

    def add(header, payload):
        out.extend(f"--{boundary}\r\n{header}\r\n\r\n".encode())
        out.extend(payload)
        out.extend(b"\r\n")

    add('Content-Disposition: form-data; name="request"\r\nContent-Type: application/json',
        json.dumps(body).encode())
    for name, data in parts.items():
        add(f'Content-Disposition: form-data; name="{name}"; filename="{name}.png"\r\n'
            "Content-Type: image/png", data)
    out.extend(f"--{boundary}--\r\n".encode())
    if len(out) > MAX_BODY:
        raise QAError("request body would exceed 32 MiB")
    return bytes(out), f"multipart/form-data; boundary={boundary}"


def describe_request(body, parts):
    """The request as stored in reports: everything except the image bytes (size and hash only)."""
    return {
        "endpoint": ENDPOINT,
        "headers": {"TypeSafe-Danger-Zone-Images": "true", "Authorization": "Bearer <redacted>"},
        "request": body,
        "parts": {n: {"bytes": len(d), "sha256": hashlib.sha256(d).hexdigest()} for n, d in parts.items()},
    }


def send(body, parts, key, timeout=90):
    """One attempt. Returns ('ok', response_json) or ('error', description). Never raises for
    network or HTTP problems; the key is never part of the returned text."""
    payload, content_type = multipart(body, parts)
    req = urllib.request.Request(ENDPOINT, data=payload, method="POST")
    req.add_header("Content-Type", content_type)
    req.add_header("Authorization", "Bearer " + key)
    req.add_header("TypeSafe-Danger-Zone-Images", "true")
    try:
        with urllib.request.urlopen(req, timeout=timeout) as r:
            return "ok", json.loads(r.read().decode("utf-8"))
    except urllib.error.HTTPError as e:
        text = e.read().decode("utf-8", "replace")[:1000]
        return "error", f"HTTP {e.code}: {text}".replace(key, "<key>")
    except (urllib.error.URLError, TimeoutError, OSError, ValueError) as e:
        return "error", f"{type(e).__name__}: {e}".replace(key, "<key>")


# ---------------------------------------------------------------------------------------------
# evaluation (the truth table lives here, in code)


def judge_noul(p, expect):
    """'pass', 'fail' or 'unsure' for a probability against the expected truth value."""
    q = p if expect else 1.0 - p
    if q >= YES:
        return "pass"
    if q <= NO:
        return "fail"
    return "unsure"


def judge_choice(answer, expect):
    probs = answer.get("probabilities", {})
    p = probs.get(expect)
    if p is None:
        return "unsure"
    if answer.get("choice") == expect and p >= 0.5:
        return "pass"
    if p <= NO:
        return "fail"
    return "unsure"


def evaluate(questionnaire, response):
    """Turns an API response into per-question results and one verdict.
    PASS: every check passed. FAIL: some check clearly failed. WARN: no clear failure but some
    check is unsure. UNKNOWN: the response lacks answers, so nothing is concluded."""
    answers = (response or {}).get("answers")
    if not isinstance(answers, dict):
        return "UNKNOWN", [], "the response has no answers"
    results = []
    for qname, spec in questionnaire["questions"].items():
        a = answers.get(qname)
        if a is None:
            results.append({"question": qname, "result": "missing"})
            continue
        if spec["type"] == "noul":
            p = a.get("noul")
            results.append({"question": qname, "p": p, "expect": spec["expect"],
                            "result": judge_noul(p, spec["expect"]) if p is not None else "missing"})
        elif spec["type"] == "choice":
            results.append({"question": qname, "choice": a.get("choice"),
                            "probabilities": a.get("probabilities"), "expect": spec["expect"],
                            "result": judge_choice(a, spec["expect"])})
        else:  # box: reported only when its presence question says the target is there
            gate = answers.get(spec["only_if"], {}).get("noul")
            present = gate is not None and gate >= YES
            results.append({"question": qname, "box": a.get("box") if present else None,
                            "confidence": a.get("confidence"), "present_p": gate,
                            "result": "info" if present else "not-present"})
    judged = [r["result"] for r in results if r["result"] in ("pass", "fail", "unsure", "missing")]
    if "missing" in judged:
        return "UNKNOWN", results, "some answers are missing"
    if "fail" in judged:
        return "FAIL", results, ""
    if "unsure" in judged:
        return "WARN", results, ""
    return "PASS", results, ""


# ---------------------------------------------------------------------------------------------
# reports


def markdown_report(run):
    lines = [f"# Cube Visual QA: {run['questionnaire']}", "",
             f"- model: `{run['model']}`", f"- mode: **{run['mode']}**",
             f"- verdict: **{run['verdict']}**" + (f" ({run['note']})" if run.get("note") else ""),
             "- thresholds (experimental): yes >= 0.7, no <= 0.3",
             f"- images: {', '.join(f'{k} = `{v}`' for k, v in run['images'].items())}",
             f"- usage: {json.dumps(run.get('usage')) if run.get('usage') else 'n/a'}", ""]
    if run.get("results"):
        lines += ["| question | answer | expected | result |", "|---|---|---|---|"]
        for r in run["results"]:
            if "p" in r:
                ans = f"{r['p']:.3f}" if r["p"] is not None else "-"
            elif "choice" in r:
                ans = f"{r['choice']} {json.dumps(r.get('probabilities'))}"
            elif r.get("box"):
                ans = json.dumps(r["box"])
            else:
                ans = "-"
            lines.append(f"| {r['question']} | {ans} | {r.get('expect', '')} | {r['result']} |")
        lines.append("")
    if run.get("error"):
        lines += [f"**Error:** {run['error']}", ""]
    lines.append("Not a proof of anything: TypeSafe judges a picture; geometry is proven by the Rust tests.")
    return "\n".join(lines) + "\n"


def save_run(out_dir, label, run, request_desc, raw):
    os.makedirs(out_dir, exist_ok=True)
    stem = os.path.join(out_dir, label)
    with open(stem + ".request.json", "w", encoding="utf-8") as f:
        json.dump(request_desc, f, indent=2)
    if raw is not None:
        with open(stem + ".response.json", "w", encoding="utf-8") as f:
            json.dump(raw, f, indent=2)
    with open(stem + ".report.json", "w", encoding="utf-8") as f:
        json.dump(run, f, indent=2)
    with open(stem + ".report.md", "w", encoding="utf-8") as f:
        f.write(markdown_report(run))
    return stem


# ---------------------------------------------------------------------------------------------
# commands


def run_one(questionnaire, files, model, live, key, out_dir, label, mock=None):
    images = {}
    for slot, path in files.items():
        with open(path, "rb") as f:
            images[slot] = f.read()
    body, parts = build_request(questionnaire, images, model)
    desc = describe_request(body, parts)
    run = {"questionnaire": questionnaire["name"], "model": model, "images": files,
           "when": datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="seconds")}
    raw = None
    if mock is not None:
        raw = mock
        run["mode"] = "mock response (no network)"
    elif not live:
        run.update(mode="dry run (nothing sent)", verdict="UNKNOWN",
                   note="dry run: the request was built and checked, not sent", results=[])
        save_run(out_dir, label, run, desc, None)
        return run
    else:
        run["mode"] = "live"
        status, raw = send(body, parts, key)
        if status == "error":
            run.update(verdict="UNKNOWN", error=raw, results=[], note="the API call failed")
            save_run(out_dir, label, run, desc, None)
            return run
    verdict, results, note = evaluate(questionnaire, raw)
    run.update(verdict=verdict, results=results, note=note, usage=raw.get("usage"))
    save_run(out_dir, label, run, desc, raw)
    return run


def load_dotenv(path=None):
    """Reads KEY=VALUE lines from tools/visual-qa/.env into the environment. A variable that is
    already set in the environment wins; empty values are ignored; nothing is ever printed."""
    path = path or os.path.join(HERE, ".env")
    if not os.path.exists(path):
        return
    with open(path, encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith("#") or "=" not in line:
                continue
            k, v = line.split("=", 1)
            k, v = k.strip(), v.strip().strip('"').strip("'")
            if k and v and k not in os.environ:
                os.environ[k] = v


def require_live(args):
    if not args.live:
        return None
    load_dotenv()
    key = os.environ.get("TYPESAFE_API_KEY")
    if not key:
        raise QAError("--live needs TYPESAFE_API_KEY (environment or tools/visual-qa/.env); "
                      "it is not set, nothing was sent")
    return key


def cmd_check(args):
    q = load_questionnaire(args.questionnaire)
    key = require_live(args)
    files = dict(item.split("=", 1) for item in args.image)
    if set(files) != set(q["images"]):
        raise QAError(f"questionnaire '{q['name']}' needs image slots {sorted(q['images'])}, "
                      f"given {sorted(files)} (use slot=path)")
    mock = None
    if args.mock_response:
        with open(args.mock_response, encoding="utf-8") as f:
            mock = json.load(f)
    label = args.label or q["name"]
    run = run_one(q, files, args.model, args.live, key, args.out, label, mock)
    print(f"{q['name']}: {run['verdict']} ({run['mode']})")
    print(f"report: {os.path.join(args.out, label)}.report.md")
    return 0 if run["verdict"] in ("PASS", "WARN") or not args.live else 1


def cmd_fixtures(args):
    q = load_questionnaire("scene-smoke")
    key = require_live(args)
    with open(os.path.join(HERE, "fixtures", "labels.json"), encoding="utf-8") as f:
        labels = json.load(f)
    items = list(labels.items())
    if args.live and len(items) > args.max_requests:
        raise QAError(f"{len(items)} fixtures exceed --max-requests {args.max_requests}")
    table, wrong = [], []
    for name, label in items:
        path = os.path.join(HERE, "fixtures", name)
        run = run_one(q, {"screenshot": path}, args.model, args.live, key, args.out,
                      os.path.splitext(name)[0])
        table.append((name, label["expect"], run["verdict"]))
        if args.live and run["verdict"] != label["expect"]:
            wrong.append((name, label["expect"], run["verdict"], label.get("why", "")))
    lines = ["# Cube Visual QA: scene-smoke on the fixtures", "",
             f"- model: `{args.model}`, mode: {'live' if args.live else 'dry run (nothing sent)'}", "",
             "| image | expected | verdict |", "|---|---|---|"]
    lines += [f"| {n} | {e} | {v} |" for n, e, v in table]
    if args.live:
        lines += ["", f"Agreement: {len(table) - len(wrong)} of {len(table)}.", ""]
        for n, e, v, why in wrong:
            lines.append(f"- WRONG: {n}: expected {e}, got {v} ({why})")
    os.makedirs(args.out, exist_ok=True)
    with open(os.path.join(args.out, "fixtures-summary.md"), "w", encoding="utf-8") as f:
        f.write("\n".join(lines) + "\n")
    print("\n".join(lines))
    return 0


def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    sub = p.add_subparsers(dest="cmd", required=True)
    for name in ("check", "fixtures"):
        s = sub.add_parser(name)
        s.add_argument("--model", default=DEFAULT_MODEL)
        s.add_argument("--out", default=os.path.join(HERE, "reports"))
        s.add_argument("--live", action="store_true",
                       help="really call the API (needs TYPESAFE_API_KEY); default is a dry run")
        s.add_argument("--max-requests", type=int, default=6)
        if name == "check":
            s.add_argument("questionnaire")
            s.add_argument("image", nargs="+", help="slot=path, e.g. screenshot=shot.png")
            s.add_argument("--label")
            s.add_argument("--mock-response", help="evaluate a recorded response instead of calling")
    args = p.parse_args(argv)
    try:
        return {"check": cmd_check, "fixtures": cmd_fixtures}[args.cmd](args)
    except QAError as e:
        print(f"error: {e}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
