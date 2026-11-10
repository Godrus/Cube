"""Compares live v2 answers with the pre-registered labels (fixtures-v2/manifest.json).

Usage:  python analyze_v2.py <split> [reports-dir]        e.g.  python analyze_v2.py calibration

Uses the fixed thresholds (yes >= 0.7, no <= 0.3); it never changes them. On the evaluation split the
numbers are reported once, as they are. Also compares every Box with the pixel-difference box of the
test defect (IoU, whether the centres are close, whether the box covers the defect).
"""

import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
FIX = os.path.join(HERE, "fixtures-v2")
YES, NO = 0.7, 0.3
CHECKS = ("gap_in_ground", "off_palette_patch", "block_missing_in_after", "terrain_visible")
BOXES = {"gap_box": "gap_in_ground", "notch_box": "block_missing_in_after"}


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
    out = [f"# Visual QA v2 analysis: {split}", ""]
    conf = {c: {True: {"yes": 0, "unsure": 0, "no": 0}, False: {"yes": 0, "unsure": 0, "no": 0}} for c in CHECKS}
    wrong, boxes, ran = [], [], 0
    for it in items:
        path = os.path.join(reports, "v2", it["id"] + ".response.json")
        if not os.path.exists(path):
            continue
        ran += 1
        with open(path, encoding="utf-8") as f:
            answers = json.load(f)["answers"]
        for check, truth in it.get("typesafe_checks", {}).items():
            if check not in CHECKS or truth is None or check not in answers:
                continue
            p = answers[check].get("noul")
            if p is None:
                continue
            cell = "yes" if p >= YES else ("no" if p <= NO else "unsure")
            conf[check][bool(truth)][cell] += 1
            if (truth and cell == "no") or ((not truth) and cell == "yes"):
                wrong.append((it["id"], check, truth, p))
        for bq, gate in BOXES.items():
            if bq in answers and it.get("typesafe_checks", {}).get(gate):
                with open(os.path.join(FIX, it["truth"]), encoding="utf-8") as f:
                    truth = json.load(f)
                b = answers[bq].get("box")
                gate_p = answers.get(gate, {}).get("noul")
                if b and truth.get("defect_box") and gate_p is not None and gate_p >= YES:
                    pb = (b["left"], b["top"], b["right"], b["bottom"])
                    tb = truth["defect_box"]
                    centre = lambda r: ((r[0] + r[2]) / 2, (r[1] + r[3]) / 2)
                    (cx, cy), (tx, ty) = centre(pb), centre(tb)
                    boxes.append({"item": it["id"], "predicted": [round(v, 3) for v in pb],
                                  "truth": [round(v, 3) for v in tb], "iou": round(iou(pb, tb), 3),
                                  "centre_offset": round(((cx - tx) ** 2 + (cy - ty) ** 2) ** 0.5, 3),
                                  "confidence": answers[bq].get("confidence")})
                else:
                    boxes.append({"item": it["id"], "note": "box not shown (presence check below 0.7)", "gate_p": gate_p})
    out.append(f"Items with live answers: {ran} of {len(items)}.\n")
    for c in CHECKS:
        t = conf[c]
        if sum(sum(v.values()) for v in t.values()) == 0:
            continue
        out += [f"### {c}", "", "| truth | yes | unsure | no |", "|---|---|---|---|"]
        for tv in (True, False):
            out.append(f"| {tv} | {t[tv]['yes']} | {t[tv]['unsure']} | {t[tv]['no']} |")
        out.append("")
    out.append("Wrong at the fixed thresholds (confident and opposite): " +
               (", ".join(f"{i} {c} truth={t} p={p:.2f}" for i, c, t, p in wrong) or "none"))
    out += ["", "## Box vs the pixel-difference box of the defect", ""]
    out += [json.dumps(b) for b in boxes] or ["(no Box answers)"]
    text = "\n".join(out) + "\n"
    os.makedirs(os.path.join(reports, "v2"), exist_ok=True)
    with open(os.path.join(reports, "v2", f"analysis-{split}.md"), "w", encoding="utf-8") as f:
        f.write(text)
    print(text)


if __name__ == "__main__":
    main()
