"""Visual QA v3 suite: one command -> N scenarios -> Godot -> evidence -> deterministic checks ->
batched TypeSafe questions (advisory) -> one QA report.

    python cube_visual_qa.py suite --generate 30 --seed 42 --profile gameplay --mode adaptive

Dry run by default: Godot runs locally (free), NO API request is sent without --live. TypeSafe never
turns a technically correct scenario into FAIL (a clear disagreement is a WARN); an unsure answer
or an infrastructure error (HTTP 529, timeout) is UNKNOWN, never a failure of the game.
"""

import datetime
import hashlib
import json
import os
import shutil
import sys

import checks_v3
import godot_runner
import mock_api
import qa_batch
import scenario_gen
import cube_visual_qa as qa

HERE = os.path.dirname(os.path.abspath(__file__))
PRICE_PER_MILLION = 0.042  # USD per million input tokens, output is free (docs.typesafe.ai/models)
REGRESSION_DIR = os.path.join(HERE, "regression")


# ---------------------------------------------------------------------------------------------
# status logic

def final_status(scen, det, ts):
    """det / ts are status strings. Statuses: PASS WARN FAIL UNKNOWN SKIPPED."""
    if det == "SKIPPED":
        return "SKIPPED"
    if det == "FAIL":
        return "FAIL"
    if det == "UNKNOWN":
        return "UNKNOWN"
    if ts == "WARN":
        return "WARN"                       # advisory: a model disagreement never becomes FAIL
    if scen.get("needs_visual") and ts != "PASS":
        return "UNKNOWN"                    # only the visual judge can see this one and it did not confirm
    return det


# ---------------------------------------------------------------------------------------------
# planning

def estimate_cost(tokens):
    return tokens * PRICE_PER_MILLION / 1e6


def limits_from(args):
    return {"max_tokens": args.max_batch_tokens, "max_images": args.max_images, "max_body": qa_batch.BODY_LIMIT,
            "batch_size": None if args.batch_size == "auto" else int(args.batch_size)}


def plan_mode(scenarios, dets, templates, mode, lim, force, compare, viewport=qa_batch.DEFAULT_VIEWPORT, shot_path=None):
    """Routing + grouping for one mode. Returns a dict with jobs, groups, skipped (with reasons)."""
    jobs, skipped = [], {}
    for s in scenarios:
        send, why = qa_batch.route(s, dets.get(s["id"]), "adaptive" if mode == "adaptive" else mode, force, compare)
        if not send:
            skipped[s["id"]] = why
            continue
        job = qa_batch.make_job(s, templates, shot_path=(lambda ref, sid=s["id"]: shot_path(sid, ref)) if shot_path else None, viewport=viewport)
        if job:
            jobs.append(job)
        else:
            skipped[s["id"]] = "no question"
    error = None
    try:
        groups, rejected = qa_batch.group_jobs(jobs, "single" if mode == "single" else "batch", lim)
    except qa_batch.PlanError as e:
        groups, rejected, error = [], [], str(e)
    for sid, why in rejected:
        skipped[sid] = why
    return {"mode": mode, "jobs": jobs, "groups": groups, "skipped": skipped, "error": error}


def plan_summary(p):
    groups = p["groups"]
    tokens = [sum(j["est_tokens"] for j in g) + qa_batch.REQUEST_OVERHEAD_TOKENS for g in groups]
    return {"mode": p["mode"], "scenarios_with_questions": len(p["jobs"]), "requests": len(groups),
            "images": sum(len(j["images"]) for g in groups for j in g),
            "questions": sum(len(j["questions"]) for g in groups for j in g),
            "est_input_tokens": sum(tokens), "est_cost_usd": round(estimate_cost(sum(tokens)), 5),
            "largest_request_tokens": max(tokens) if tokens else 0,
            "scenarios_per_request": [len(g) for g in groups], "not_sent": len(p["skipped"]), "error": p["error"]}


# ---------------------------------------------------------------------------------------------
# running

def scenario_dir(out, sid):
    return os.path.join(out, "scenarios", sid)


def run_godot(scen, out, rerun):
    d = scenario_dir(out, scen["id"])
    marker = os.path.join(d, "fingerprint.txt")
    tel = {}
    if not rerun and os.path.exists(marker) and open(marker).read() == scen["fingerprint"]:
        ok = True
        for ph in scen["phases"]:
            tp = os.path.join(d, ph["name"], "telemetry.json")
            if not os.path.exists(tp):
                ok = False
                break
            with open(tp, encoding="utf-8") as f:
                tel[ph["name"]] = json.load(f)
        if ok:
            return tel, True
    os.makedirs(d, exist_ok=True)
    tel = godot_runner.run_scenario(scen, d)
    if all("events" in t for t in tel.values()):
        with open(marker, "w") as f:
            f.write(scen["fingerprint"])
    return tel, False


def load_scenarios(args):
    if args.from_file:
        out = []
        for p in args.from_file:
            paths = [os.path.join(p, n) for n in sorted(os.listdir(p)) if n.endswith(".json")] if os.path.isdir(p) else [p]
            for path in paths:
                with open(path, encoding="utf-8") as f:
                    data = json.load(f)
                out.extend(data if isinstance(data, list) else [data])
        return out
    cats = None
    if args.checks:
        cats, _ = scenario_gen.parse_checks(args.checks)
    seeds = [int(x) for x in str(args.seed).split(",")]
    templates = None
    if args.camera != "all":
        keep = {"top": "cam_top", "up": "cam_up", "bottom": "cam_up", "horizontal": "cam_horizontal", "angled": "cam_angled",
                "rotate": "cam_rotate", "move": "cam_move"}[args.camera]
        templates = [t for t in scenario_gen.PROFILES[args.profile] if scenario_gen.TEMPLATES[t][0] != "CAMERA" or t == keep] or None
    wseeds = [int(x) for x in args.world_seeds.split(",")] if args.world_seeds else None
    scen = scenario_gen.generate(args.generate, seeds, args.profile, cats, args.intensity, wseeds, templates)
    if args.profile == "regression" and os.path.isdir(os.path.join(REGRESSION_DIR, "failures")):
        d = os.path.join(REGRESSION_DIR, "failures")
        for n in sorted(os.listdir(d)):
            if n.endswith(".json"):
                with open(os.path.join(d, n), encoding="utf-8") as f:
                    scen.append(json.load(f))
    return scen


def run_id(args):
    return f"{args.profile}-n{args.generate}-seed{str(args.seed).replace(',', '_')}-i{args.intensity}"


# ---------------------------------------------------------------------------------------------
# report

def md_report(run):
    L = [f"# Cube Visual QA v3 report: {run['run_id']}", "",
         f"- generated: {run['when']}; generator {scenario_gen.GENERATOR_VERSION}; question templates hash `{run['templates_hash']}`",
         f"- scenarios generated: **{run['counts']['generated']}**; executed in Godot: **{run['counts']['executed']}**; "
         f"not executed: {run['counts']['not_executed']}",
         f"- TypeSafe: mode `{run['mode']}`, transport **{run['transport']}**, requests sent {run['api']['sent']}, "
         f"reused {run['api']['reused']}, skipped {run['api']['skipped']}, deferred {run['api']['deferred']}, errors {run['api']['errors']}",
         f"- estimated input tokens of the plan: {run['plan']['est_input_tokens']:,} (about ${run['plan']['est_cost_usd']}); "
         f"measured input tokens: {('n/a (mock)' if run['transport'].startswith('MOCK') else run['api']['input_tokens']) if run['api']['input_tokens'] is not None else 'n/a (no answered request)'}",
         f"- scenarios that need TypeSafe (visual-only): {run['counts']['needs_typesafe']}; routed to TypeSafe: {run['counts']['routed']}", ""]
    if run.get("unsupported_requested"):
        L += [f"- requested but unsupported (not generated): {', '.join(run['unsupported_requested'])}", ""]
    L += ["## Results", "", "| Scenario | Category | Template | Deterministic | TypeSafe | Final |", "|---|---|---|---|---|---|"]
    for r in run["scenarios"]:
        L.append(f"| {r['id']} | {r['category']} | {r['template']}{(' [' + r['injected_fault'] + ']') if r['injected_fault'] else ''} | {r['deterministic']} | {r['typesafe']} | {r['final']} |")
    L += ["", "## Summary by category", "", "| Category | n | PASS | WARN | FAIL | UNKNOWN | SKIPPED |", "|---|---|---|---|---|---|---|"]
    for cat, c in run["by_category"].items():
        L.append(f"| {cat} | {c['n']} | {c['PASS']} | {c['WARN']} | {c['FAIL']} | {c['UNKNOWN']} | {c['SKIPPED']} |")
    L += ["", "(status = the final status of the scenarios of that category; counts, not rates)", "", "## Details", ""]
    for r in run["scenarios"]:
        L += [f"### {r['id']} {r['template']} ({r['category']}): {r['final']}", "",
              f"- reproduce: `{r['reproduce']}` (fingerprint `{r['fingerprint']}`, world seed {r['world_seed']}, generator seed {r['gen_seed']})",
              f"- injected fault: {r['injected_fault'] or 'none'}; expected: `{json.dumps(r['expected'])}`"]
        if r.get("pictures"):
            L.append("- pictures: " + ", ".join(f"`{p}`" for p in r["pictures"]))
        if r.get("godot_error"):
            L.append(f"- Godot: {r['godot_error']}")
        for name, c in (r.get("checks") or {}).items():
            L.append(f"  - {c['status'].upper():7} {name}: {c['detail']}")
        if r.get("typesafe_questions"):
            for q in r["typesafe_questions"]:
                ans = q.get("answer")
                L.append(f"  - TypeSafe {q['qid']} v{q['version']} ({q['calibration'].split(',')[0]}): {q['result']}"
                         f"{'' if ans is None else ' answer=' + json.dumps(ans)}, expected {q['expect']}{(' (' + q['why'] + ')') if q.get('why') else ''}")
        elif r["typesafe_reason"]:
            L.append(f"- TypeSafe: not asked: {r['typesafe_reason']}")
        L.append("")
    L += ["Statuses: PASS / WARN / FAIL / UNKNOWN / SKIPPED. For injected-fault scenarios the expectation is that the fault is "
          "found; a status says whether the observation matched it. TypeSafe is advisory: a model disagreement is a WARN, an unsure "
          "answer or an HTTP 529 is UNKNOWN. Nothing here is a measurement of TypeSafe accuracy; see V2-RESULTS.md for that.", ""]
    return "\n".join(L)


def plan_compare_md(rows, n):
    L = [f"## Plans for {n} scenarios (estimates; nothing sent)", "",
         "| mode | scenarios asked | requests | images | questions | est. input tokens | est. cost USD | largest request (tokens) | note |", "|---|---|---|---|---|---|---|---|---|"]
    for r in rows:
        L.append(f"| {r['mode']} | {r['scenarios_with_questions']} | {r['requests']} | {r['images']} | {r['questions']} | "
                 f"{r['est_input_tokens']:,} | {r['est_cost_usd']} | {r['largest_request_tokens']:,} | {(r['error'] or '')[:90]} |")
    return "\n".join(L) + "\n"


# ---------------------------------------------------------------------------------------------
# main entry

def run_suite(args):
    templates, thash = qa_batch.load_templates()
    scen = load_scenarios(args)
    if args.only:
        keep = set(args.only.split(","))
        scen = [s for s in scen if s["id"] in keep]
    unsupported = []
    if args.checks:
        _, unsupported = scenario_gen.parse_checks(args.checks)
    if args.live and args.mock:
        raise qa.QAError("--live and --mock exclude each other")
    out = args.out or os.path.join(HERE, "reports", "suite", run_id(args))
    os.makedirs(out, exist_ok=True)
    with open(os.path.join(out, "scenarios.json"), "w", encoding="utf-8") as f:
        json.dump(scen, f, indent=1)
    key = qa.require_live(args) if args.live else None
    lim = limits_from(args)
    force = set(x for x in (args.force_typesafe or "").split(",") if x)
    modes = ["single", "batch", "adaptive"] if args.mode == "all" else [args.mode]
    if args.mode == "all" and (args.live or args.mock or args.execute == "godot"):
        raise qa.QAError("--mode all only compares plans: use it with --execute none")

    # 1. execute in Godot (local, free)
    tel_by, dets, executed, godot_err = {}, {}, 0, {}
    if args.execute == "godot":
        if godot_runner.godot_exe() is None:
            raise qa.QAError("Godot binary not found: set GODOT_BIN or use --execute none")
        for i, s in enumerate(scen):
            tel, reused = run_godot(s, out, args.rerun_godot)
            tel_by[s["id"]] = tel
            errs = [f"{k}: {v['error']}" for k, v in tel.items() if "error" in v]
            dets[s["id"]] = checks_v3.evaluate(s, tel)
            if errs:
                godot_err[s["id"]] = "; ".join(errs)
            else:
                executed += 1
            print(f"[{i + 1}/{len(scen)}] {s['id']} {s['template']}: deterministic {dets[s['id']]['status']}{' (reused)' if reused else ''}", flush=True)

    def shot_path(sid, ref):
        return checks_v3.shot_file(tel_by.get(sid, {}), ref)

    sp = shot_path if args.execute == "godot" else None
    plans = {m: plan_mode(scen, dets, templates, m, lim, force, args.compare, shot_path=sp) for m in modes}
    summaries = [plan_summary(plans[m]) for m in modes]
    needs_ts = sum(1 for s in scen if s["typesafe"]["role"] == "primary")
    if args.mode == "all":
        text = plan_compare_md(summaries, len(scen))
        with open(os.path.join(out, "plan-compare.md"), "w", encoding="utf-8") as f:
            f.write(text)
        print(text)
        print(f"scenarios generated: {len(scen)}; executed in Godot: 0 (plan only); needing TypeSafe (visual-only): {needs_ts}")
        return 0

    plan = plans[modes[0]]
    summary = summaries[0]
    if plan["error"]:
        print("PLAN ERROR:", plan["error"])
    records, transport_name = [], "none (dry run: nothing sent)"
    ledger = qa_batch.Ledger(os.path.join(out, "ledger.json"))
    if (args.live or args.mock) and args.execute != "godot":
        raise qa.QAError("sending needs the pictures: use --execute godot")
    if args.live or args.mock:
        if args.mock:
            expect = {n: (m["primitive"], m["expect"]) for j in plan["jobs"] for n, m in j["meta"].items()}
            transport = mock_api.make_transport(expect, fail_calls=_mock_fail(args.mock_fail))
            transport_name = "MOCK (no network, answers come from the scenarios' own expectations)"
        else:
            transport = lambda body, parts: qa.send(body, parts, key, args.timeout)
            transport_name = "LIVE"
        if args.live and len(plan["groups"]) > args.max_requests:
            print(f"note: {len(plan['groups'])} requests planned, --max-requests {args.max_requests}: the rest are deferred")
        records = qa_batch.execute(plan["groups"], transport, key, out, ledger, thash, args.model, args.max_requests,
                                   args.max_input_tokens, args.retry_errors, args.timeout)

    # 2. per-scenario results
    ts_results = {}
    for g, rec in zip(plan["groups"], records):
        ts_results.update(qa_batch.parse_record(g, rec))
    rows, by_cat = [], {}
    for s in scen:
        sid = s["id"]
        det = dets.get(sid)
        det_status = det["status"] if det else "SKIPPED"
        if sid in godot_err and det and det["status"] == "UNKNOWN":
            det_status = "UNKNOWN"
        ts = ts_results.get(sid)
        reason = plan["skipped"].get(sid)
        if ts:
            ts_status = ts["status"]
        elif sid in plan["skipped"] or not s["typesafe"]["questions"]:
            ts_status = "SKIPPED"
        else:
            ts_status = "SKIPPED"
            reason = reason or ("planned, not sent (dry run)" if not (args.live or args.mock) else "not planned")
        fin = final_status(s, det_status, ts_status)
        pics = []
        if det:
            for ph in s["phases"]:
                for e in (tel_by[sid].get(ph["name"]) or {}).get("events", []):
                    if e["op"] == "shot":
                        pics.append(os.path.relpath(e["file"], out).replace("\\", "/"))
        row = {"id": sid, "category": s["category"], "template": s["template"], "injected_fault": s["injected_fault"],
               "deterministic": det_status, "typesafe": ts_status, "final": fin, "fingerprint": s["fingerprint"],
               "world_seed": s["world_seed"], "gen_seed": s["gen_seed"], "expected": s["expected"], "pictures": pics,
               "checks": det["checks"] if det else None, "metrics": det["metrics"] if det else None,
               "godot_error": godot_err.get(sid), "typesafe_questions": ts["questions"] if ts else None,
               "typesafe_reason": None if ts else reason,
               "reproduce": f"python cube_visual_qa.py suite --from-file {os.path.join(out, 'scenarios.json')} --only {sid}"}
        rows.append(row)
        c = by_cat.setdefault(s["category"], {"n": 0, "PASS": 0, "WARN": 0, "FAIL": 0, "UNKNOWN": 0, "SKIPPED": 0})
        c["n"] += 1
        c[fin] += 1
    inp = [r["response"]["usage"]["input_tokens"] for r in records if r["status"] == "ok" and r.get("response", {}).get("usage")]
    api = {"sent": sum(1 for r in records if r["status"] in ("ok", "error")), "reused": sum(1 for r in records if r["status"] == "reused"),
           "skipped": sum(1 for r in records if r["status"] == "skipped"), "deferred": sum(1 for r in records if r["status"] == "deferred"),
           "errors": sum(1 for r in records if r["status"] == "error"),
           "error_kinds": sorted({r["error_kind"] for r in records if r["status"] == "error"}),
           "input_tokens": sum(inp) if inp else None}
    run = {"run_id": os.path.basename(out), "when": datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="seconds"),
           "templates_hash": thash, "mode": args.mode, "transport": transport_name, "plan": summary,
           "counts": {"generated": len(scen), "executed": executed, "not_executed": len(scen) - executed,
                      "needs_typesafe": needs_ts, "routed": len(plan["jobs"])},
           "api": api, "unsupported_requested": unsupported, "by_category": by_cat, "scenarios": rows,
           "requests": [{k: v for k, v in r.items() if k != "response"} for r in records]}
    formats = [x.strip() for x in args.report.split(",")]
    if "json" in formats:
        with open(os.path.join(out, "results.json"), "w", encoding="utf-8") as f:
            json.dump(run, f, indent=1)
    text = md_report(run)
    if "markdown" in formats:
        with open(os.path.join(out, "report.md"), "w", encoding="utf-8") as f:
            f.write(text)
    if args.save_failures:
        os.makedirs(os.path.join(REGRESSION_DIR, "failures"), exist_ok=True)
        for s, r in zip(scen, rows):
            if r["final"] in ("FAIL", "WARN"):
                with open(os.path.join(REGRESSION_DIR, "failures", f"{s['id']}-{s['fingerprint']}.json"), "w", encoding="utf-8") as f:
                    json.dump(s, f, indent=1)
    print(text.split("## Details")[0])
    print(f"report: {os.path.join(out, 'report.md')}")
    return 1 if any(r["final"] == "FAIL" for r in rows) else 0


def compare_runs(a, b):
    """Compares two runs of the SAME scenarios and questions (for example SINGLE against BATCH).
    Reports counts only: requests, tokens, errors, and per question whether the verdict and the
    probability agree. It makes no claim about which mode is cheaper or better."""
    def answers(run):
        out = {}
        for r in run["scenarios"]:
            for q in r.get("typesafe_questions") or []:
                out[(r["id"], q["qid"])] = q
        return out
    qa_, qb = answers(a), answers(b)
    common = sorted(set(qa_) & set(qb))
    flips, diffs, both_answered = [], [], 0
    for k in common:
        x, y = qa_[k], qb[k]
        if x.get("answer") is not None and y.get("answer") is not None:
            both_answered += 1
            if isinstance(x["answer"], (int, float)) and isinstance(y["answer"], (int, float)):
                diffs.append(abs(x["answer"] - y["answer"]))
        if x["result"] != y["result"]:
            flips.append({"scenario": k[0], "qid": k[1], "a": x["result"], "b": y["result"]})
    return {"questions_in_both": len(common), "only_in_a": len(set(qa_) - set(qb)), "only_in_b": len(set(qb) - set(qa_)),
            "answered_in_both": both_answered, "verdict_differs": flips,
            "max_probability_difference": round(max(diffs), 4) if diffs else None,
            "mean_probability_difference": round(sum(diffs) / len(diffs), 4) if diffs else None,
            "requests": [a["api"]["sent"], b["api"]["sent"]], "input_tokens": [a["api"]["input_tokens"], b["api"]["input_tokens"]],
            "errors": [a["api"]["errors"], b["api"]["errors"]], "mode": [a["mode"], b["mode"]],
            "caveat": "counts from one pair of runs; no mode is claimed to be cheaper or more accurate"}


def _mock_fail(text):
    """`--mock-fail 1:529,3:timeout` -> {1: 'HTTP 529: system_overloaded', 3: 'TimeoutError: timed out'}"""
    out = {}
    for part in (text or "").split(","):
        if part:
            i, kind = part.split(":")
            out[int(i)] = "HTTP 529: system_overloaded" if kind == "529" else "TimeoutError: timed out"
    return out


def add_arguments(p):
    p.add_argument("--generate", type=int, default=6, help="number of scenarios")
    p.add_argument("--seed", default="42", help="generator seed, or a comma list")
    p.add_argument("--world-seeds", help="comma list of world seeds (default: derived from the scenario)")
    p.add_argument("--profile", choices=sorted(scenario_gen.PROFILES), default="smoke")
    p.add_argument("--checks", help="camera,terrain,editing,occlusion (collision is reported as unsupported)")
    p.add_argument("--camera", choices=["all", "top", "up", "bottom", "horizontal", "angled", "rotate", "move"], default="all")
    p.add_argument("--intensity", type=int, choices=[1, 2, 3], default=2)
    p.add_argument("--from-file", nargs="+", help="scenario JSON files or directories (replay), instead of generating")
    p.add_argument("--mode", choices=["single", "batch", "adaptive", "all"], default="adaptive")
    p.add_argument("--batch-size", default="auto", help="scenarios per request, or auto (limits decide)")
    p.add_argument("--max-batch-tokens", type=int, default=qa_batch.DEFAULT_MAX_TOKENS)
    p.add_argument("--max-images", type=int, default=24)
    p.add_argument("--max-requests", type=int, default=5)
    p.add_argument("--max-input-tokens", type=int)
    p.add_argument("--force-typesafe", help="comma list of scenario ids or template names to send regardless of routing")
    p.add_argument("--compare", action="store_true", help="also ask TypeSafe where code already decides (for comparison)")
    p.add_argument("--execute", choices=["godot", "none"], default="godot", help="run scenarios in Godot (local) or plan only")
    p.add_argument("--plan-only", dest="execute", action="store_const", const="none")
    p.add_argument("--rerun-godot", action="store_true")
    p.add_argument("--live", action="store_true", help="REALLY call the API (needs TYPESAFE_API_KEY)")
    p.add_argument("--mock", action="store_true", help="answer from a local mock (tests the plumbing, says nothing about Jev)")
    p.add_argument("--mock-fail", help="e.g. 1:529,3:timeout (mock only)")
    p.add_argument("--retry-errors", action="store_true")
    p.add_argument("--timeout", type=int, default=90)
    p.add_argument("--model", default=qa.DEFAULT_MODEL)
    p.add_argument("--report", default="json,markdown")
    p.add_argument("--save-failures", action="store_true")
    p.add_argument("--only", help="comma list of scenario ids to keep")
    p.add_argument("--out")
