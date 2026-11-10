"""A MOCK of the TypeSafe API for offline tests and for dry runs of the whole pipeline.

It answers from the scenario's own expectation (so the plumbing can be tested end to end) and can
inject failures. Its answers say NOTHING about Jev; results produced with it are labelled MOCK and
are never mixed with live results. No network access.
"""

import json


def make_transport(expect_by_question, fail_calls=None, wrong=(), drop=(), p_right=0.9):
    """expect_by_question: {question name: ('noul', bool) | ('choice', class)}.
    fail_calls: {call index (0-based): error text}, e.g. {1: 'HTTP 529: system_overloaded'}.
    wrong: question names answered against the expectation; drop: names left out of the answer."""
    fail_calls = fail_calls or {}
    calls = []

    def transport(body, parts):
        i = len(calls)
        calls.append(sorted(body["questions"]))
        if i in fail_calls:
            return "error", fail_calls[i]
        answers = {}
        for name, spec in body["questions"].items():
            if name in drop:
                continue
            kind, expect = expect_by_question[name]
            ok = name not in wrong
            if kind == "noul":
                p = p_right if (bool(expect) == ok) else 1 - p_right
                answers[name] = {"noul": p, "confidence": 0.5}
            else:
                other = [c for c in spec.get("criteria", {}) if c != expect] or ["other"]
                top = expect if ok else other[0]
                rest = [c for c in spec.get("criteria", {"x": 0}) if c != top]
                probs = {top: p_right}
                for c in rest:
                    probs[c] = round((1 - p_right) / max(1, len(rest)), 4)
                answers[name] = {"choice": top, "probabilities": probs}
        tokens = 2800 * len(parts) + 90 * len(body["questions"]) + 100
        return "ok", {"answers": answers, "usage": {"input_tokens": tokens, "output_tokens": 25 * len(answers)}, "mock": True}

    transport.calls = calls
    return transport
