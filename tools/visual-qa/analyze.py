"""Compares a live scene-smoke run with the labels: per-check confusion matrices and image-level
false positives / false negatives. Reads reports/<image>.report.json and fixtures/labels.json.

Run after `python cube_visual_qa.py fixtures --live`:   python analyze.py [reports-dir]

No thresholds are tuned here: it uses the tool's fixed ones (yes >= 0.7, no <= 0.3). With a handful
of images these counts describe what happened, they do not measure the model.
"""

import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
YES, NO = 0.7, 0.3


def main():
    reports = sys.argv[1] if len(sys.argv) > 1 else os.path.join(HERE, "reports")
    with open(os.path.join(HERE, "fixtures", "labels.json"), encoding="utf-8") as f:
        labels = json.load(f)
    runs = {}
    for name in labels:
        stem = os.path.splitext(name)[0]
        with open(os.path.join(reports, stem + ".report.json"), encoding="utf-8") as f:
            runs[name] = json.load(f)

    out = ["# Visual QA analysis (scene-smoke)", ""]
    # image level: positive = the image is defective (label FAIL)
    tp = fp = fn = tn = warn = 0
    for name, label in labels.items():
        verdict = runs[name]["verdict"]
        defective = label["expect"] == "FAIL"
        flagged = verdict == "FAIL"
        warn += verdict == "WARN"
        tp += defective and flagged
        fn += defective and not flagged
        fp += (not defective) and flagged
        tn += (not defective) and not flagged
    out += ["## Image level (positive = defective image, predicted positive = verdict FAIL)", "",
            "| | predicted FAIL | not FAIL |", "|---|---|---|",
            f"| defective | {tp} (TP) | {fn} (FN) |", f"| good | {fp} (FP) | {tn} (TN) |", "",
            f"False negatives: {fn} of {tp + fn} defective. False positives: {fp} of {fp + tn} good. "
            f"WARN verdicts: {warn}.", ""]

    out += ["## Per check (rows: truth, columns: the answer at the fixed thresholds)", ""]
    checks = {}
    for name, label in labels.items():
        for check, truth in label.get("checks", {}).items():
            if truth is None:
                continue
            r = [x for x in runs[name]["results"] if x["question"] == check][0]
            p = r.get("p")
            if p is None:
                continue
            cell = "yes" if p >= YES else ("no" if p <= NO else "unsure")
            checks.setdefault(check, []).append((name, truth, p, cell))
    for check, rows in checks.items():
        m = {t: {"yes": 0, "unsure": 0, "no": 0} for t in (True, False)}
        for _, truth, _, cell in rows:
            m[truth][cell] += 1
        out += [f"### {check}", "", "| truth | yes | unsure | no |", "|---|---|---|---|"]
        for t in (True, False):
            out.append(f"| {t} | {m[t]['yes']} | {m[t]['unsure']} | {m[t]['no']} |")
        wrong = [(n, t, p) for n, t, p, c in rows if (t and c == "no") or ((not t) and c == "yes")]
        out += ["", "Wrong (confident and opposite): " + (", ".join(f"{n} truth={t} p={p:.2f}" for n, t, p in wrong) or "none"), ""]
    out.append("Choice `scene_kind`: " + ", ".join(
        f"{n}: {[x for x in runs[n]['results'] if x['question'] == 'scene_kind'][0]['choice']}" for n in labels))
    text = "\n".join(out) + "\n"
    with open(os.path.join(reports, "analysis.md"), "w", encoding="utf-8") as f:
        f.write(text)
    print(text)


if __name__ == "__main__":
    main()
