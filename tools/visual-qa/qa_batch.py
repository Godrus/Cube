"""Question rendering, request grouping (SINGLE / BATCH / ADAPTIVE), spend guard and answer parsing
for Visual QA v3. Pure standard library; nothing here sends anything unless `execute` is given a
real transport AND the caller passed --live (see suite.py).

Limits come from the TypeSafe docs (images and models pages): 32 MiB per request body, 280..1568 px
per side, and for jev-1.14.0 a 32k-token context PER QUESTION. Every question in a request sees the
whole state, so all images of a request count against that 32k; we plan with a margin.
Token numbers below are ESTIMATES (about 3 tokens per 1000 pixels, measured on v1/v2 requests:
2,910 input tokens for one 1280x720 image with a few questions); the report compares them with the
real usage after a live run. Whether one merged request is cheaper or more accurate than several
small ones is NOT assumed anywhere: it is measured by `compare` (see suite.py).
"""

import datetime
import hashlib
import json
import os
import re

import cube_visual_qa as qa
import pngio

HERE = os.path.dirname(os.path.abspath(__file__))
TEMPLATES_PATH = os.path.join(HERE, "questions", "v3", "templates.json")

CONTEXT_PER_QUESTION = 32_000        # jev-1.14.0 without the long-context header
PLANNING_MARGIN = 0.8                 # plan to 80 % of the limit: the token estimate is rough
DEFAULT_MAX_TOKENS = int(CONTEXT_PER_QUESTION * PLANNING_MARGIN)
BODY_LIMIT = 30 * 1024 * 1024         # below the 32 MiB documented limit
TOKENS_PER_PIXEL = 0.0030
REQUEST_OVERHEAD_TOKENS = 120
AVG_PNG_BYTES = 150_000               # mean of the 44 v2 fixtures (147 kB); real size used when the file exists
DEFAULT_VIEWPORT = (1280, 720)
EDIT_CROP = qa.EDIT_CROP


class PlanError(Exception):
    pass


# ---------------------------------------------------------------------------------------------
# templates

def load_templates(path=TEMPLATES_PATH):
    with open(path, encoding="utf-8") as f:
        data = json.load(f)
    return {t["id"]: t for t in data["templates"]}, hashlib.sha256(open(path, "rb").read()).hexdigest()[:16]


def slug(text):
    return re.sub(r"[^a-z0-9]+", "_", text.lower()).strip("_")


def _tokens_for_text(text):
    return int(len(text) / 3.5) + 25


def est_image_tokens(w, h):
    return int(w * h * TOKENS_PER_PIXEL)


# ---------------------------------------------------------------------------------------------
# jobs: one per scenario that needs TypeSafe

def make_job(scen, templates, shot_path=None, viewport=DEFAULT_VIEWPORT, only_qids=None):
    """Renders the scenario's questions. `shot_path(ref)` returns a file path or None (plan-only).
    Image slot keys and question names are unique across scenarios (scenario id based)."""
    sid = slug(scen["id"])
    images, questions, meta = {}, {}, {}
    for q in scen["typesafe"]["questions"]:
        if only_qids and q["qid"] not in only_qids:
            continue
        t = templates[q["qid"]]
        subst = {}
        for slot, ref in q["slots"].items():
            key = f"{sid}_{slug(ref)}"
            crop = EDIT_CROP if t["category"] == "EDITING" else None
            images[key] = {"ref": ref, "path": shot_path(ref) if shot_path else None, "crop": crop}
            subst[slot] = key
        text = t["instructions"]
        for slot, key in subst.items():
            text = text.replace("{" + slot + "}", key)
        name = f"{sid}__{slug(q['qid'])}"
        spec = {"type": t["primitive"], "instructions": text, "expect": q["expect"]}
        if "criteria" in t:
            spec["criteria"] = t["criteria"]
        questions[name] = spec
        meta[name] = {"scenario": scen["id"], "qid": q["qid"], "version": t["version"], "primitive": t["primitive"],
                      "expect": q["expect"], "category": t["category"], "calibration": t["calibration"]}
    if not questions:
        return None
    tokens = sum(est_image_tokens(*(EDIT_CROP[1] * 320, EDIT_CROP[1] * 180) if im["crop"] else viewport) for im in images.values())
    tokens += sum(_tokens_for_text(json.dumps(s)) for s in questions.values())
    body_bytes = 0
    for im in images.values():
        body_bytes += os.path.getsize(im["path"]) if im["path"] and os.path.exists(im["path"]) else AVG_PNG_BYTES
    return {"scenario": scen["id"], "images": images, "questions": questions, "meta": meta,
            "est_tokens": tokens, "est_bytes": body_bytes}


# ---------------------------------------------------------------------------------------------
# routing (ADAPTIVE)

def route(scen, det, mode, force=(), compare=False):
    """Returns (send: bool, reason). `det` is the deterministic result of the scenario, or None
    when the scenarios were not executed (plan only: assumes they pass)."""
    det_status = det["status"] if det else None
    ts = scen["typesafe"]
    if not ts["questions"]:
        return False, "no TypeSafe question defined for this scenario"
    if scen["id"] in force or scen["template"] in force:
        return True, "forced by the user"
    if mode != "adaptive":
        return True, f"mode {mode}: every scenario with questions is sent"
    if det_status == "FAIL":
        return False, "already decided by deterministic checks (FAIL)"
    if det and det.get("metrics", {}).get("fault_not_visible"):
        return False, "the injected fault is not visible in the picture: label unreliable, nothing to ask"
    if det_status == "UNKNOWN" and not scen.get("needs_visual") and det and "executed" in det["checks"] and det["checks"]["executed"]["status"] == "unknown":
        return False, "the scenario did not run: there is no picture"
    if ts["role"] == "primary":
        return True, "visual-only judgement: code cannot decide it"
    if compare:
        return True, "comparison mode: asked although code already covers it"
    return False, "covered by deterministic checks (use --compare or --force-typesafe to ask anyway)"


# ---------------------------------------------------------------------------------------------
# grouping

def _fits(tokens, images, nbytes, lim):
    return (tokens + REQUEST_OVERHEAD_TOKENS <= lim["max_tokens"] and nbytes <= lim["max_body"]
            and images <= lim["max_images"])


def group_jobs(jobs, mode, lim):
    """Returns (groups, rejected). A group keeps whole jobs (a scenario's before/after images and all
    its questions stay together). Scenario order is preserved, so grouping is deterministic.
    SINGLE: one group or a rejection. BATCH/ADAPTIVE: greedy packing by estimated tokens, image
    count and body size, or by a fixed --batch-size."""
    rejected, groups = [], []
    usable = []
    for j in jobs:
        n_img = len(j["images"])
        if not _fits(j["est_tokens"], n_img, j["est_bytes"], lim):
            rejected.append((j["scenario"], f"alone exceeds the limits (~{j['est_tokens']} tokens, {n_img} images)"))
        else:
            usable.append(j)
    if mode == "single":
        if not usable:
            return [], rejected
        tokens = sum(j["est_tokens"] for j in usable)
        images = sum(len(j["images"]) for j in usable)
        if not _fits(tokens, images, sum(j["est_bytes"] for j in usable), lim):
            raise PlanError(f"SINGLE does not fit: {len(usable)} scenarios, {images} images, about {tokens} input tokens "
                            f"against a planning limit of {lim['max_tokens']} tokens per request "
                            f"(documented: 32k tokens per question, every question sees all images). Nothing was sent. "
                            f"Use --mode batch, or fewer scenarios.")
        return [usable], rejected
    cur, tok, img, byt = [], 0, 0, 0
    for j in usable:
        full = lim.get("batch_size") and len(cur) >= lim["batch_size"]
        if cur and (full or not _fits(tok + j["est_tokens"], img + len(j["images"]), byt + j["est_bytes"], lim)):
            groups.append(cur)
            cur, tok, img, byt = [], 0, 0, 0
        cur.append(j)
        tok += j["est_tokens"]
        img += len(j["images"])
        byt += j["est_bytes"]
    if cur:
        groups.append(cur)
    return groups, rejected


def group_request(group):
    """Merged questionnaire dict (cube_visual_qa format) and the image files of a group."""
    qn = {"name": "cube-v3-" + "+".join(slug(j["scenario"]) for j in group), "images": [], "questions": {}}
    files = {}
    for j in group:
        for key, im in j["images"].items():
            qn["images"].append(key)
            files[key] = im
        qn["questions"].update(j["questions"])
    return qn, files


def build_group_request(group, model):
    """The real request (needs the picture files). Image prep reuses the v2 crop for edit pairs."""
    qn, files = group_request(group)
    images = {}
    for key, im in files.items():
        if not im["path"] or not os.path.exists(im["path"]):
            raise PlanError(f"picture for {key} ({im['ref']}) does not exist")
        images[key] = qa.prepare_image(im["path"], im["crop"])
    body, parts = qa.build_request(qn, images, model)
    return qn, body, parts


def request_key(group, templates_hash, model):
    """Identity of a request: same scenarios, same questionnaire version, same picture bytes."""
    h = hashlib.sha256()
    h.update(f"{model}|{templates_hash}".encode())
    for j in group:
        h.update(j["scenario"].encode())
        for key in sorted(j["images"]):
            im = j["images"][key]
            h.update(key.encode())
            if im["path"] and os.path.exists(im["path"]):
                with open(im["path"], "rb") as f:
                    h.update(hashlib.sha256(f.read()).digest())
        for name in sorted(j["questions"]):
            h.update(name.encode())
            h.update(json.dumps(j["questions"][name], sort_keys=True).encode())
    return h.hexdigest()[:20]


# ---------------------------------------------------------------------------------------------
# spend guard: ledger of attempts, written BEFORE a request is sent

class Ledger:
    def __init__(self, path):
        self.path = path
        self.data = {}
        if os.path.exists(path):
            with open(path, encoding="utf-8") as f:
                self.data = json.load(f)

    def save(self):
        os.makedirs(os.path.dirname(self.path), exist_ok=True)
        with open(self.path, "w", encoding="utf-8") as f:
            json.dump(self.data, f, indent=2, sort_keys=True)

    def get(self, key):
        return self.data.get(key)

    def mark(self, key, **fields):
        entry = self.data.setdefault(key, {"attempts": 0})
        entry.update(fields)
        self.save()
        return entry


def classify_error(text):
    t = text or ""
    if "529" in t[:12] or "system_overloaded" in t:
        return "infra:overload_529"
    if "429" in t[:12]:
        return "infra:rate_limit_429"
    if "timed out" in t.lower() or "timeout" in t.lower():
        return "infra:timeout"
    if t.startswith("HTTP 5"):
        return "infra:server_error"
    if t.startswith("HTTP 4"):
        return "request_rejected"
    return "infra:network"


def execute(groups, transport, key, out_dir, ledger, templates_hash, model, max_requests, max_input_tokens=None,
            retry_errors=False, timeout=90):
    """Sends the groups one attempt each. `transport(body, parts)` -> ('ok', json) | ('error', text).
    Never retries inside a run; a request already answered (same key in the ledger) is reused from
    disk and costs nothing; one that was sent or failed earlier is NOT repeated unless
    retry_errors. Returns a list of per-request records."""
    records, sent, tokens_planned = [], 0, 0
    req_dir = os.path.join(out_dir, "requests")
    os.makedirs(req_dir, exist_ok=True)
    for idx, group in enumerate(groups):
        rk = request_key(group, templates_hash, model)
        rec = {"index": idx, "request_key": rk, "scenarios": [j["scenario"] for j in group],
               "est_tokens": sum(j["est_tokens"] for j in group) + REQUEST_OVERHEAD_TOKENS}
        resp_path = os.path.join(req_dir, rk + ".response.json")
        prev = ledger.get(rk)
        if prev and prev.get("status") == "ok" and os.path.exists(resp_path):
            with open(resp_path, encoding="utf-8") as f:
                rec.update(status="reused", response=json.load(f), note="answered in an earlier run: no new request")
        elif prev and prev.get("status") in ("sent", "error") and not retry_errors:
            rec.update(status="skipped", note=f"an earlier attempt ({prev.get('status')}: {prev.get('error', 'no result recorded')}) "
                       "is not repeated automatically; pass --retry-errors to try again")
        elif sent >= max_requests:
            rec.update(status="deferred", note=f"--max-requests {max_requests} reached")
        elif max_input_tokens is not None and tokens_planned + rec["est_tokens"] > max_input_tokens:
            rec.update(status="deferred", note=f"--max-input-tokens {max_input_tokens} would be exceeded (estimate)")
        else:
            qn, body, parts = build_group_request(group, model)
            with open(os.path.join(req_dir, rk + ".request.json"), "w", encoding="utf-8") as f:
                json.dump(qa.describe_request(body, parts), f, indent=2)  # no key, no image bytes
            ledger.mark(rk, status="sent", attempts=(prev or {}).get("attempts", 0) + 1,
                        when=datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="seconds"))
            sent += 1
            tokens_planned += rec["est_tokens"]
            status, raw = transport(body, parts)
            if status == "ok":
                with open(resp_path, "w", encoding="utf-8") as f:
                    json.dump(raw, f, indent=2)
                ledger.mark(rk, status="ok", usage=raw.get("usage"))
                rec.update(status="ok", response=raw)
            else:
                if key:
                    raw = raw.replace(key, "<key>")  # defence in depth: send() already redacts it
                kind = classify_error(raw)
                ledger.mark(rk, status="error", error=kind + ": " + raw[:200])
                rec.update(status="error", error_kind=kind, error=raw[:300])
        records.append(rec)
    return records


# ---------------------------------------------------------------------------------------------
# parsing answers back to scenarios

def judge(meta, answer):
    if meta["primitive"] == "choice":
        return qa.judge_choice(answer, meta["expect"])
    p = answer.get("noul")
    return qa.judge_noul(p, bool(meta["expect"])) if p is not None else "missing"


def parse_record(group, rec):
    """Per scenario: {"status": PASS|WARN|UNKNOWN, "questions": [...], "note": ...}. A model answer
    that clearly contradicts the scenario is a WARN (advisory), never a FAIL; an unsure or missing
    answer, and every infrastructure error, is UNKNOWN."""
    out = {}
    answers = (rec.get("response") or {}).get("answers")
    for j in group:
        qs, verdicts = [], []
        for name, meta in j["meta"].items():
            item = {"qid": meta["qid"], "question": name, "expect": meta["expect"], "version": meta["version"],
                    "calibration": meta["calibration"]}
            a = answers.get(name) if isinstance(answers, dict) else None
            if rec["status"] in ("error", "skipped", "deferred") or a is None:
                item.update(result="unknown", why=rec.get("error_kind") or rec.get("note") or "answer missing")
            else:
                r = judge(meta, a)
                item.update(result={"pass": "pass", "fail": "fail", "unsure": "unsure", "missing": "unknown"}[r],
                            answer=a.get("noul") if meta["primitive"] == "noul" else {"choice": a.get("choice"), "probabilities": a.get("probabilities")})
            qs.append(item)
            verdicts.append(item["result"])
        if "fail" in verdicts:
            st = "WARN"
        elif verdicts and all(v == "pass" for v in verdicts):
            st = "PASS"
        else:
            st = "UNKNOWN"
        out[j["scenario"]] = {"status": st, "questions": qs}
    return out
