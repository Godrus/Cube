"""Full measurement report of a v2 split: requests, tokens, confusion counts per check and per
questionnaire, errors per defect kind, deterministic checks, Box, concrete FP/FN examples.

Usage:  python report_v2.py <calibration|evaluation> [reports-dir]

FROZEN BEFORE THE EVALUATION RUN: this file, the questionnaires, the thresholds and the labels must
not change after the evaluation split has been looked at (see FROZEN.json for the hashes).
Infrastructure errors (HTTP 529, timeouts) are counted separately and are never a FAIL or a miss.
Counts are the result; percentages would mean little on samples this small.
"""

import json
import os
import sys

import deterministic

HERE = os.path.dirname(os.path.abspath(__file__))
FIX = os.path.join(HERE, "fixtures-v2")
YES, NO = 0.7, 0.3
QUESTIONNAIRE_OF = {
    "terrain_visible": "scene-smoke-v2", "scene_kind": "scene-smoke-v2",
    "gap_in_ground": "terrain-integrity-v2", "gap_box": "terrain-integrity-v2",
    "off_palette_patch": "terrain-integrity-v2",
    "block_missing_in_after": "block-edit-v2", "notch_box": "block-edit-v2",
}
# which check is the one a defect kind should trigger
TARGET_CHECK = {"missing_brick": "gap_in_ground", "hidden_area": "gap_in_ground",
                "wrong_material": "off_palette_patch", "none": None}


def cell(p):
    return "yes" if p >= YES else ("no" if p <= NO else "unsure")


def iou(a, b):
    ix = max(0.0, min(a[2], b[2]) - max(a[0], b[0]))
    iy = max(0.0, min(a[3], b[3]) - max(a[1], b[1]))
    inter = ix * iy
    union = (a[2] - a[0]) * (a[3] - a[1]) + (b[2] - b[0]) * (b[3] - b[1]) - inter
    return inter / union if union else 0.0


def main():
    split = sys.argv[1]
    reports = sys.argv[2] if len(sys.argv) > 2 else os.path.join(HERE, "reports")
    with open(os.path.join(FIX, "manifest.json"), encoding="utf-8") as f:
        items = [i for i in json.load(f)["items"] if i["split"] == split]
    out = [f"# Visual QA v2 report: {split}", ""]

    # --- requests and tokens
    answered, infra, never_sent, tokens_in, tokens_out = [], [], [], 0, 0
    answers = {}
    for it in items:
        base = os.path.join(reports, "v2", it["id"])
        if os.path.exists(base + ".response.json"):
            with open(base + ".response.json", encoding="utf-8") as f:
                raw = json.load(f)
            answers[it["id"]] = raw["answers"]
            answered.append(it["id"])
            tokens_in += raw.get("usage", {}).get("input_tokens", 0)
            tokens_out += raw.get("usage", {}).get("output_tokens", 0)
        elif os.path.exists(base + ".report.json"):
            with open(base + ".report.json", encoding="utf-8") as f:
                rep = json.load(f)
            if rep.get("error"):
                infra.append((it["id"], rep["error"][:60]))
            else:
                never_sent.append(it["id"])
    out += ["## Requests and tokens", "",
            f"- answered: {len(answered)}; infrastructure errors: {len(infra)}; not sent in this split: {len(never_sent)}",
            f"- input tokens (answered requests): {tokens_in:,}; output tokens: {tokens_out}",
            f"- price: $0.042 per million input tokens, output free (docs.typesafe.ai/models): "
            f"about ${tokens_in * 0.042 / 1e6:.5f} for the answered requests"]
    for i, e in infra:
        out.append(f"- infrastructure error (not a model result): {i}: {e}")
    out.append("")

    # --- per check confusion (positive = the defect is present)
    conf = {}
    examples = {"FP": [], "FN": [], "unsure": []}
    for it in items:
        a = answers.get(it["id"])
        if not a:
            continue
        for check, truth in it.get("typesafe_checks", {}).items():
            if truth is None or check not in a:
                continue
            if check == "scene_kind":
                top = a[check].get("choice")
                p = a[check].get("probabilities", {}).get(truth, 0.0)
                res = "correct" if (top == truth and p >= 0.5) else ("wrong" if p <= NO else "unsure")
                conf.setdefault(check, {}).setdefault("choice", {}).setdefault(res, []).append(it["id"])
                continue
            p = a[check].get("noul")
            if p is None:
                continue
            c = cell(p)
            if check == "terrain_visible":
                key = {"yes": "TP", "unsure": "unsure", "no": "FN"}[c]
            else:
                key = {(True, "yes"): "TP", (True, "no"): "FN", (False, "yes"): "FP", (False, "no"): "TN"}.get((bool(truth), c), "unsure")
            conf.setdefault(check, {}).setdefault("noul", {}).setdefault(key, []).append((it["id"], round(p, 2)))
            if key in ("FP", "FN", "unsure"):
                examples[key if key != "unsure" else "unsure"].append((it["id"], check, truth, round(p, 2)))
    for qn in ("scene-smoke-v2", "terrain-integrity-v2", "block-edit-v2"):
        out += [f"## {qn}", ""]
        shown = False
        for check, d in conf.items():
            if QUESTIONNAIRE_OF.get(check) != qn:
                continue
            shown = True
            if "choice" in d:
                out.append(f"- `{check}` (Choice, expected class vs top option): " +
                           ", ".join(f"{k} {len(v)}" for k, v in d["choice"].items()))
            else:
                n = d["noul"]
                out.append(f"- `{check}`: TP {len(n.get('TP', []))}, FP {len(n.get('FP', []))}, TN {len(n.get('TN', []))}, "
                           f"FN {len(n.get('FN', []))}, unsure {len(n.get('unsure', []))}")
        if not shown:
            out.append("- no answered items")
        out.append("")

    # --- per defect kind
    out += ["## Per defect kind (the check that should fire)", "", "| defect | items answered | fired (yes) | unsure | missed (no) | infra error |", "|---|---|---|---|---|---|"]
    for kind in ("missing_brick", "hidden_area", "wrong_material"):
        its = [i for i in items if i["kind"] == "scene" and i["fault"] == kind and i.get("typesafe_checks", {}).get(TARGET_CHECK[kind])]
        fired = sum(1 for i in its if i["id"] in answers and cell(answers[i["id"]][TARGET_CHECK[kind]]["noul"]) == "yes")
        uns = sum(1 for i in its if i["id"] in answers and cell(answers[i["id"]][TARGET_CHECK[kind]]["noul"]) == "unsure")
        miss = sum(1 for i in its if i["id"] in answers and cell(answers[i["id"]][TARGET_CHECK[kind]]["noul"]) == "no")
        err = sum(1 for i in its if i["id"] not in answers)
        out.append(f"| {kind} (`{TARGET_CHECK[kind]}`) | {len(its) - err} of {len(its)} | {fired} | {uns} | {miss} | {err} |")
    edits = [i for i in items if i["kind"] == "edit" and i.get("typesafe_checks")]
    for label, truth in (("edit changed", True), ("edit stale", False)):
        its = [i for i in edits if i["typesafe_checks"]["block_missing_in_after"] is truth]
        cs = [cell(answers[i["id"]]["block_missing_in_after"]["noul"]) for i in its if i["id"] in answers]
        out.append(f"| {label} (`block_missing_in_after`, truth {truth}) | {len(cs)} of {len(its)} | {cs.count('yes')} | {cs.count('unsure')} | {cs.count('no')} | {len(its) - len(cs)} |")
    out.append("")

    # --- examples
    out += ["## Concrete cases (item, check, truth, p)", ""]
    for k in ("FP", "FN", "unsure"):
        out.append(f"- {k}: " + ("; ".join(f"{i} {c} truth={t} p={p}" for i, c, t, p in examples[k]) or "none"))
    out.append("")

    # --- box
    out += ["## Box (shown only when its presence check is >= 0.7)", ""]
    any_box = False
    for it in items:
        a = answers.get(it["id"])
        if not a:
            continue
        for bq, gate in (("gap_box", "gap_in_ground"), ("notch_box", "block_missing_in_after")):
            if bq not in a or gate not in a:
                continue
            gp = a[gate]["noul"]
            if gp < YES:
                continue
            with open(os.path.join(FIX, it["truth"]), encoding="utf-8") as f:
                t = json.load(f)
            b = a[bq]["box"]
            pb = (b["left"], b["top"], b["right"], b["bottom"])
            tb = t.get("defect_box")
            if tb and it.get("typesafe_checks", {}).get(gate):
                any_box = True
                out.append(f"- {it['id']} {bq}: presence {gp:.2f}, IoU {iou(pb, tb):.2f}, confidence {a[bq].get('confidence')}, "
                           f"true box area {(tb[2]-tb[0])*(tb[3]-tb[1]):.2f} of the picture")
            else:
                any_box = True
                out.append(f"- {it['id']} {bq}: shown for a picture without a defect (presence {gp:.2f}): a false alarm")
    if not any_box:
        out.append("- no Box was shown in this split")
    out.append("")

    # --- deterministic checks (plain code, every item of the split, no API)
    out += ["## Deterministic checks (every item, no API)", ""]
    good_truth = {}
    for it in items:
        if it["kind"] == "scene" and it["fault"] == "none":
            with open(os.path.join(FIX, it["truth"]), encoding="utf-8") as f:
                good_truth[it["id"].split("-")[0]] = json.load(f)
    rows = {"partial_load": [0, 0], "stale": [0, 0], "good (false alarms)": [0, 0], "other faults (should not fire)": [0, 0]}
    for it in items:
        with open(os.path.join(FIX, it["truth"]), encoding="utf-8") as f:
            truth = json.load(f)
        if it["kind"] == "scene":
            with open(os.path.join(FIX, it["image"]), "rb") as f:
                res = deterministic.check_scene(f.read(), truth, good_truth.get(it["id"].split("-")[0]))
            fired = any(not ok for ok, _ in res.values())
            key = "partial_load" if it["fault"] == "partial_load" else ("good (false alarms)" if it["fault"] == "none" else "other faults (should not fire)")
        else:
            with open(os.path.join(FIX, it["before"]), "rb") as f:
                b = f.read()
            with open(os.path.join(FIX, it["after"]), "rb") as f:
                a2 = f.read()
            res = deterministic.check_edit(b, a2, truth)
            fired = any(not ok for ok, _ in res.values())
            key = "stale" if it["label"] == "unchanged" else "good (false alarms)"
        rows[key][0] += 1
        rows[key][1] += fired
    out += ["| class | items | deterministic check fired |", "|---|---|---|"]
    out += [f"| {k} | {v[0]} | {v[1]} |" for k, v in rows.items()]
    text = "\n".join(out) + "\n"
    os.makedirs(os.path.join(reports, "v2"), exist_ok=True)
    with open(os.path.join(reports, "v2", f"report-{split}.md"), "w", encoding="utf-8") as f:
        f.write(text)
    print(text)


if __name__ == "__main__":
    main()
