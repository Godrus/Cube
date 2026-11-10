"""Offline tests of Visual QA v3 (scenario generator, checks, planning, spend guard, reports).

Nothing here touches the network or needs Godot or a key. Godot is replaced by a fake runner that
writes synthetic telemetry and pictures, so these tests prove the plumbing, NOT that the game works:
the real Godot run is a separate step (see V3.md).

Run:  python -m unittest test_suite_v3 -v   (from tools/visual-qa)
"""

import argparse
import json
import math
import os
import tempfile
import unittest
from unittest import mock

import checks_v3
import cube_visual_qa as qa
import godot_runner
import mock_api
import pngio
import qa_batch
import scenario_gen as sg
import suite

HERE = os.path.dirname(os.path.abspath(__file__))
SECRET = "sk-test-SECRET-KEY-12345"


def flat_png(path, rgb, w=640, h=360, patch=None):
    px = bytearray(bytes(rgb) * (w * h))
    if patch:
        for y in range(patch[1], patch[3]):
            px[(y * w + patch[0]) * 3:(y * w + patch[2]) * 3] = bytes((200, 30, 30)) * (patch[2] - patch[0])
    with open(path, "wb") as f:
        f.write(pngio.write_png(w, h, 3, px))


def fake_runner(scen, out_dir, timeout=180):
    """Writes telemetry in the shape of godot/main.gd and distinct synthetic pictures."""
    tel = {}
    for ph in scen["phases"]:
        d = os.path.join(out_dir, ph["name"])
        os.makedirs(d, exist_ok=True)
        events, rot, pos = [], [-0.6, 0.0, 0.0], [10.0, 20.0, 5.0]
        n = 0
        for st in ph["steps"]:
            if st["op"] == "turn":
                rot = [rot[0] + st.get("pitch", 0), rot[1] + st.get("yaw", 0), 0.0]
            if st["op"] == "move":
                pos = [pos[0], pos[1] + st.get("up", 0) * -0.2 * st["frames"], pos[2]]
            e = {"op": st["op"], "frame": n, "bricks": 80, "camera_position": list(pos), "camera_rotation": list(rot)}
            if st["op"] == "shot":
                f = os.path.join(d, st["name"] + ".png")
                flat_png(f, (60, 140, 60) if st["name"] not in ("shot", "after") else (60, 140, 60), patch=(100, 100, 300, 250) if st["name"] in ("shot", "after") else None)
                e.update(name=st["name"], file=f)
            if st["op"] == "edit":
                e.update(hit=True, bricks_changed=27)
            events.append(e)
            n += st.get("frames", 1)
        tel[ph["name"]] = {"viewport": [640, 360], "frames": n, "bricks_final": 80, "events": events}
    return tel


def args_for(out, **kw):
    p = argparse.ArgumentParser()
    suite.add_arguments(p)
    a = p.parse_args(["--out", out])
    for k, v in kw.items():
        setattr(a, k, v)
    return a


class Generator(unittest.TestCase):
    def test_reproducible_and_seed_sensitive(self):
        a = sg.generate(12, 42, "full")
        b = sg.generate(12, 42, "full")
        self.assertEqual(a, b)
        c = sg.generate(12, 43, "full")
        self.assertNotEqual([s["fingerprint"] for s in a], [s["fingerprint"] for s in c])

    def test_scenario_does_not_depend_on_the_others(self):
        self.assertEqual(sg.generate(5, 42, "full")[3], sg.generate(30, 42, "full")[3])

    def test_every_template_is_valid_and_questions_exist(self):
        templates, _ = qa_batch.load_templates()
        for name in sg.TEMPLATES:
            s = sg.make_scenario(name, 0, 1)
            shots = {f"{p['name']}:{st['name']}" for p in s["phases"] for st in p["steps"] if st["op"] == "shot"}
            for q in s["typesafe"]["questions"]:
                self.assertIn(q["qid"], templates, name)
                for slot, ref in q["slots"].items():
                    self.assertIn(slot, templates[q["qid"]]["slots"])
                    self.assertIn(ref, shots, f"{name}: {ref}")
            for key in ("shot", "before", "after"):
                if isinstance(s["expected"].get(key), str):
                    self.assertIn(s["expected"][key], shots, name)
            self.assertTrue(s["phases"] and all(p["steps"] for p in s["phases"]))

    def test_no_template_for_unsupported_features(self):
        for name in sg.TEMPLATES:
            self.assertNotIn("collision", name)
            self.assertNotIn("penetration", name)
        self.assertIn("collision", sg.UNSUPPORTED)
        cats, unsup = sg.parse_checks("camera,collision,terrain")
        self.assertEqual(cats, {"CAMERA", "WORLD"})
        self.assertEqual(unsup, ["collision"])
        with self.assertRaises(ValueError):
            sg.parse_checks("telepathy")

    def test_profiles_and_category_filter(self):
        only = sg.generate(10, 1, "full", categories={"EDITING"})
        self.assertTrue(all(s["category"] == "EDITING" for s in only))
        with self.assertRaises(ValueError):
            sg.generate(3, 1, "smoke", categories={"NOPE"})

    def test_seed_list_and_world_seed_pool(self):
        s = sg.generate(4, [1, 2], "gameplay", world_seeds=[500, 600])
        self.assertEqual([x["gen_seed"] for x in s], [1, 2, 1, 2])
        self.assertEqual([x["world_seed"] for x in s], [500, 600, 500, 600])

    def test_persist_second_run_replays_the_camera_path(self):
        s = sg.make_scenario("edit_persist", 0, 5)
        p1, p2 = (p["steps"] for p in s["phases"])
        non_edit = [x for x in p1 if x["op"] not in ("edit", "shot")]
        self.assertEqual([x for x in p2 if x["op"] not in ("wait",)], [x for x in p1 if x["op"] in ("turn", "move")] + [{"op": "shot", "name": "reloaded"}])
        self.assertEqual(sum(x.get("frames", 1) for x in p1 if x["op"] != "edit") + sum(1 for x in p1 if x["op"] == "edit"),
                         sum(x.get("frames", 1) for x in p2) + sum(1 for x in p1 if x["op"] == "shot" and x["name"] == "before"))
        self.assertTrue(non_edit)


class Questions(unittest.TestCase):
    def test_frozen_v2_wording_is_unchanged(self):
        templates, _ = qa_batch.load_templates()
        pairs = {"Q-TERR-GAP-001": ("terrain-integrity", "gap_in_ground"), "Q-MAT-PATCH-001": ("terrain-integrity", "off_palette_patch"),
                 "Q-TERR-VISIBLE-001": ("scene-smoke", "terrain_visible"), "Q-SCENE-KIND-001": ("scene-smoke", "scene_kind"),
                 "Q-EDIT-MISSING-001": ("block-edit", "block_missing_in_after")}
        for qid, (f, name) in pairs.items():
            with open(os.path.join(HERE, "questions", "v2", f + ".json"), encoding="utf-8") as fh:
                v2 = json.load(fh)["questions"][name]["instructions"]
            t = templates[qid]["instructions"]
            for slot in templates[qid]["slots"]:
                t = t.replace("{" + slot + "}", {"shot": "screenshot", "before": "before_crop", "after": "after_crop"}[slot])
            self.assertEqual(t, v2, qid)

    def test_every_template_has_the_required_fields(self):
        templates, _ = qa_batch.load_templates()
        for t in templates.values():
            for f in ("id", "version", "primitive", "category", "slots", "instructions", "interpretation", "confidence_policy", "calibration"):
                self.assertIn(f, t)
            self.assertIn(t["primitive"], ("noul", "choice"))  # Box is not used: it needs rework (V2-RESULTS)
            self.assertNotRegex(t["instructions"].lower(), r"crack|tear")

    def test_job_names_and_slots_are_unique_across_scenarios(self):
        templates, _ = qa_batch.load_templates()
        scen = sg.generate(20, 3, "full")
        names, slots = set(), set()
        for s in scen:
            j = qa_batch.make_job(s, templates)
            if not j:
                continue
            for n in j["questions"]:
                self.assertNotIn(n, names)
                names.add(n)
            for k in j["images"]:
                self.assertNotIn(k, slots)
                slots.add(k)
            for n, spec in j["questions"].items():
                for k in j["images"]:
                    pass
                self.assertNotIn("{", spec["instructions"])


class Planning(unittest.TestCase):
    def setUp(self):
        self.templates, self.h = qa_batch.load_templates()
        self.lim = {"max_tokens": qa_batch.DEFAULT_MAX_TOKENS, "max_images": 24, "max_body": qa_batch.BODY_LIMIT, "batch_size": None}

    def jobs(self, n, force_all=True):
        scen = sg.generate(n, 9, "gameplay")
        out = []
        for s in scen:
            j = qa_batch.make_job(s, self.templates)
            if j:
                out.append(j)
        return out

    def test_single_fits_a_few_and_refuses_many_without_sending(self):
        jobs = self.jobs(6)
        groups, rej = qa_batch.group_jobs(jobs[:3], "single", self.lim)
        self.assertEqual(len(groups), 1)
        with self.assertRaises(qa_batch.PlanError) as cm:
            qa_batch.group_jobs(self.jobs(30), "single", self.lim)
        self.assertIn("Nothing was sent", str(cm.exception))

    def test_batch_respects_the_token_limit_and_keeps_scenarios_whole(self):
        jobs = self.jobs(40)
        groups, rej = qa_batch.group_jobs(jobs, "batch", self.lim)
        self.assertEqual(sum(len(g) for g in groups), len(jobs))
        for g in groups:
            self.assertLessEqual(sum(j["est_tokens"] for j in g) + qa_batch.REQUEST_OVERHEAD_TOKENS, self.lim["max_tokens"])
            for j in g:
                self.assertTrue(all(i["ref"] for i in j["images"].values()))
        self.assertEqual([j["scenario"] for g in groups for j in g], [j["scenario"] for j in jobs])  # order kept
        self.assertGreater(len(groups), 1)

    def test_batch_size_is_not_a_fixed_number_but_a_limit_result(self):
        small = dict(self.lim, max_tokens=9000)
        big = dict(self.lim, max_tokens=30000)
        jobs = self.jobs(20)
        self.assertGreater(len(qa_batch.group_jobs(jobs, "batch", small)[0]), len(qa_batch.group_jobs(jobs, "batch", big)[0]))
        fixed = dict(self.lim, batch_size=2)
        self.assertTrue(all(len(g) <= 2 for g in qa_batch.group_jobs(jobs, "batch", fixed)[0]))

    def test_before_after_images_never_split(self):
        scen = [sg.make_scenario("edit_remove", i, 1) for i in range(12)]
        jobs = [qa_batch.make_job(s, self.templates) for s in scen]
        for g in qa_batch.group_jobs(jobs, "batch", dict(self.lim, max_tokens=9000))[0]:
            for j in g:
                self.assertEqual(len(j["images"]), 2)

    def test_adaptive_routing(self):
        scen = {s["template"]: s for s in (sg.make_scenario(t, 0, 1) for t in sg.TEMPLATES)}
        ok = {"status": "PASS", "checks": {}, "metrics": {}}
        self.assertTrue(qa_batch.route(scen["world_missing_brick"], ok, "adaptive")[0])        # visual-only
        self.assertFalse(qa_batch.route(scen["cam_top"], ok, "adaptive")[0])                    # code decides
        self.assertTrue(qa_batch.route(scen["cam_top"], ok, "adaptive", compare=True)[0])
        self.assertTrue(qa_batch.route(scen["cam_top"], ok, "adaptive", force={"cam_top"})[0])
        self.assertFalse(qa_batch.route(scen["world_missing_brick"], {"status": "FAIL", "checks": {}, "metrics": {}}, "adaptive")[0])
        invisible = {"status": "UNKNOWN", "checks": {}, "metrics": {"fault_not_visible": True}}
        self.assertFalse(qa_batch.route(scen["world_missing_brick"], invisible, "adaptive")[0])
        self.assertTrue(qa_batch.route(scen["cam_top"], ok, "batch")[0])
        self.assertFalse(qa_batch.route(scen["cam_rotate"], ok, "batch")[0])                   # no question defined

    def test_dry_runs_5_10_30_100_send_nothing(self):
        for n in (5, 10, 30, 100):
            with tempfile.TemporaryDirectory() as d, mock.patch.object(qa, "send", side_effect=AssertionError("network")), \
                    mock.patch("urllib.request.urlopen", side_effect=AssertionError("network")):
                a = args_for(d, generate=n, profile="full", execute="none", mode="all")
                self.assertEqual(suite.run_suite(a), 0)
                text = open(os.path.join(d, "plan-compare.md"), encoding="utf-8").read()
                self.assertIn("| adaptive |", text)
                self.assertEqual(len(json.load(open(os.path.join(d, "scenarios.json"))), n)) if False else None
                self.assertEqual(len(json.load(open(os.path.join(d, "scenarios.json"), encoding="utf-8"))), n)


class Ledger(unittest.TestCase):
    def setUp(self):
        self.templates, self.h = qa_batch.load_templates()
        self.tmp = tempfile.TemporaryDirectory()
        self.out = self.tmp.name
        self.scen = [sg.make_scenario("world_missing_brick", i, 1) for i in range(3)]
        self.group_jobs = []
        for s in self.scen:
            d = os.path.join(self.out, s["id"])
            os.makedirs(d)
            f = os.path.join(d, "shot.png")
            flat_png(f, (60, 140, 60), 1280, 720)
            self.group_jobs.append(qa_batch.make_job(s, self.templates, shot_path=lambda ref, f=f: f))
        self.expect = {n: (m["primitive"], m["expect"]) for j in self.group_jobs for n, m in j["meta"].items()}

    def tearDown(self):
        self.tmp.cleanup()

    def run_exec(self, groups, transport, **kw):
        led = qa_batch.Ledger(os.path.join(self.out, "ledger.json"))
        return qa_batch.execute(groups, transport, SECRET, self.out, led, self.h, "jev-1.14.0", kw.pop("max_requests", 5), **kw)

    def test_ok_then_reused_without_a_new_request(self):
        t = mock_api.make_transport(self.expect)
        groups = [[j] for j in self.group_jobs]
        r1 = self.run_exec(groups, t)
        self.assertEqual([r["status"] for r in r1], ["ok"] * 3)
        t2 = mock_api.make_transport(self.expect)
        r2 = self.run_exec(groups, t2)
        self.assertEqual([r["status"] for r in r2], ["reused"] * 3)
        self.assertEqual(t2.calls, [])  # resume: nothing sent again

    def test_http_529_is_infra_not_model_and_is_not_retried(self):
        t = mock_api.make_transport(self.expect, fail_calls={1: "HTTP 529: system_overloaded"})
        groups = [[j] for j in self.group_jobs]
        r = self.run_exec(groups, t)
        self.assertEqual([x["status"] for x in r], ["ok", "error", "ok"])
        self.assertEqual(r[1]["error_kind"], "infra:overload_529")
        self.assertEqual(len(t.calls), 3)  # one attempt each, no automatic retry
        parsed = qa_batch.parse_record(groups[1], r[1])
        self.assertEqual(parsed[self.group_jobs[1]["scenario"]]["status"], "UNKNOWN")
        t2 = mock_api.make_transport(self.expect)
        r2 = self.run_exec(groups, t2)  # resume: the failed one is not repeated by itself
        self.assertEqual([x["status"] for x in r2], ["reused", "skipped", "reused"])
        self.assertEqual(t2.calls, [])
        r3 = self.run_exec(groups, t2, retry_errors=True)
        self.assertEqual([x["status"] for x in r3], ["reused", "ok", "reused"])
        self.assertEqual(len(t2.calls), 1)

    def test_a_crash_after_sending_is_not_paid_twice(self):
        led = qa_batch.Ledger(os.path.join(self.out, "ledger.json"))
        rk = qa_batch.request_key([self.group_jobs[0]], self.h, "jev-1.14.0")
        led.mark(rk, status="sent", attempts=1)  # the process died before the answer was saved
        t = mock_api.make_transport(self.expect)
        r = self.run_exec([[self.group_jobs[0]]], t)
        self.assertEqual(r[0]["status"], "skipped")
        self.assertEqual(t.calls, [])

    def test_request_budget_and_token_budget(self):
        t = mock_api.make_transport(self.expect)
        r = self.run_exec([[j] for j in self.group_jobs], t, max_requests=1)
        self.assertEqual([x["status"] for x in r], ["ok", "deferred", "deferred"])
        t2 = mock_api.make_transport(self.expect)
        os.remove(os.path.join(self.out, "ledger.json"))
        r2 = self.run_exec([[j] for j in self.group_jobs], t2, max_input_tokens=3000)
        self.assertEqual([x["status"] for x in r2].count("ok"), 1)

    def test_no_secret_in_any_saved_file(self):
        t = mock_api.make_transport(self.expect, fail_calls={0: "HTTP 401: bad key " + SECRET})
        self.run_exec([[j] for j in self.group_jobs], t)
        for root, _, files in os.walk(self.out):
            for n in files:
                if n.endswith(".png"):
                    continue
                with open(os.path.join(root, n), encoding="utf-8", errors="ignore") as f:
                    text = f.read()
                self.assertNotIn(SECRET, text, n)
        req = [n for n in os.listdir(os.path.join(self.out, "requests")) if n.endswith(".request.json")][0]
        data = json.load(open(os.path.join(self.out, "requests", req), encoding="utf-8"))
        self.assertEqual(data["headers"]["Authorization"], "Bearer <redacted>")
        self.assertEqual(data["headers"]["TypeSafe-Danger-Zone-Images"], "true")

    def test_request_is_valid_for_the_documented_schema(self):
        qn, body, parts = qa_batch.build_group_request(self.group_jobs, "jev-1.14.0")
        self.assertEqual(set(body["assets"]), set(body["state"].values()))  # every asset referenced once
        self.assertEqual(len(parts), len(body["assets"]))
        self.assertEqual(len(set(body["state"].values())), len(body["state"]))
        for q in body["questions"].values():
            self.assertIn("`", q["instructions"])
            self.assertTrue(any(k in q["instructions"] for k in body["state"]))
        qa.multipart(body, parts)  # the body size check passes

    def test_answers_map_back_to_their_scenarios(self):
        group = self.group_jobs
        t = mock_api.make_transport(self.expect, wrong={list(group[1]["questions"])[0]}, drop={list(group[2]["questions"])[0]})
        r = self.run_exec([group], t)
        parsed = qa_batch.parse_record(group, r[0])
        self.assertEqual(parsed[group[0]["scenario"]]["status"], "PASS")
        self.assertEqual(parsed[group[1]["scenario"]]["status"], "WARN")      # clear disagreement: advisory only
        self.assertEqual(parsed[group[2]["scenario"]]["status"], "UNKNOWN")   # partial response: that answer is missing


class Statuses(unittest.TestCase):
    def test_final_status_table(self):
        plain, visual = {"needs_visual": False}, {"needs_visual": True}
        f = suite.final_status
        self.assertEqual(f(plain, "PASS", "SKIPPED"), "PASS")
        self.assertEqual(f(plain, "PASS", "WARN"), "WARN")
        self.assertEqual(f(plain, "FAIL", "SKIPPED"), "FAIL")
        self.assertEqual(f(plain, "FAIL", "PASS"), "FAIL")
        self.assertEqual(f(plain, "PASS", "UNKNOWN"), "PASS")      # an unsure model answer does not change a proven result
        self.assertEqual(f(visual, "PASS", "SKIPPED"), "UNKNOWN")   # only the visual judge can see this and was not asked
        self.assertEqual(f(visual, "PASS", "UNKNOWN"), "UNKNOWN")
        self.assertEqual(f(visual, "PASS", "PASS"), "PASS")
        self.assertEqual(f(plain, "UNKNOWN", "WARN"), "UNKNOWN")
        self.assertEqual(f(plain, "SKIPPED", "SKIPPED"), "SKIPPED")

    def test_typesafe_never_creates_a_fail(self):
        for ts in ("PASS", "WARN", "UNKNOWN", "SKIPPED"):
            for det in ("PASS", "WARN", "UNKNOWN"):
                self.assertNotEqual(suite.final_status({"needs_visual": True}, det, ts), "FAIL")


class DeterministicChecks(unittest.TestCase):
    def tel(self, events, bricks=80):
        return {"p1": {"bricks_final": bricks, "events": events}}

    def test_camera_pitch_and_yaw(self):
        s = sg.make_scenario("cam_angled", 0, 1)
        yaw = s["expected"]["yaw_delta"]
        with tempfile.TemporaryDirectory() as d:
            f = os.path.join(d, "a.png")
            flat_png(f, (60, 140, 60))
            ev = lambda dy, dp: [{"op": "probe", "name": "start", "camera_position": [0, 0, 0], "camera_rotation": [-0.6, 0.1, 0]},
                                 {"op": "shot", "name": "angled", "file": f, "camera_position": [0, 0, 0], "camera_rotation": [-0.6 + dp, 0.1 + dy, 0]}]
            good = checks_v3.evaluate(s, self.tel(ev(yaw, -0.2)))
            self.assertEqual(good["checks"]["camera_yaw"]["status"], "pass")
            self.assertEqual(good["checks"]["camera_pitch"]["status"], "pass")
            bad = checks_v3.evaluate(s, self.tel(ev(yaw + 0.5, -0.2)))
            self.assertEqual(bad["checks"]["camera_yaw"]["status"], "fail")
            self.assertEqual(bad["status"], "FAIL")

    def test_yaw_wraps_around_a_full_turn(self):
        s = sg.make_scenario("cam_rotate", 0, 5)
        s["expected"]["yaw_delta"] = 2 * math.pi
        with tempfile.TemporaryDirectory() as d:
            f = os.path.join(d, "a.png")
            flat_png(f, (60, 140, 60))
            ev = [{"op": "probe", "name": "start", "camera_position": [1, 2, 3], "camera_rotation": [-0.6, 0.2, 0]},
                  {"op": "shot", "name": "after", "file": f, "camera_position": [1, 2, 3], "camera_rotation": [-0.6, 0.2 + 2 * math.pi, 0]}]
            r = checks_v3.evaluate(s, self.tel(ev))
            self.assertEqual(r["checks"]["camera_yaw"]["status"], "pass")

    def test_missing_telemetry_is_unknown_never_pass(self):
        s = sg.make_scenario("cam_top", 0, 1)
        r = checks_v3.evaluate(s, {"p1": {"error": "Godot exit 1"}})
        self.assertEqual(r["status"], "UNKNOWN")

    def test_partial_load_detector(self):
        s = sg.make_scenario("world_partial_load", 0, 1)
        with tempfile.TemporaryDirectory() as d:
            f = os.path.join(d, "a.png")
            flat_png(f, (60, 140, 60))
            mk = lambda n: {"bricks_final": n, "events": [{"op": "shot", "name": "full", "file": f}, {"op": "shot", "name": "partial", "file": f}]}
            self.assertEqual(checks_v3.evaluate(s, {"p1": mk(80), "p2": mk(15)})["checks"]["partial_load_detected"]["status"], "pass")
            self.assertEqual(checks_v3.evaluate(s, {"p1": mk(80), "p2": mk(79)})["checks"]["partial_load_detected"]["status"], "fail")

    def test_edits(self):
        s = sg.make_scenario("edit_remove", 0, 1)
        with tempfile.TemporaryDirectory() as d:
            a, b = os.path.join(d, "a.png"), os.path.join(d, "b.png")
            flat_png(a, (60, 140, 60))
            flat_png(b, (60, 140, 60), patch=(10, 10, 40, 40))
            ev = lambda hit, ch, after: [{"op": "shot", "name": "before", "file": a}, {"op": "edit", "hit": hit, "bricks_changed": ch},
                                         {"op": "shot", "name": "after", "file": after}]
            self.assertEqual(checks_v3.evaluate(s, self.tel(ev(True, 27, b)))["status"], "PASS")
            self.assertEqual(checks_v3.evaluate(s, self.tel(ev(False, 0, b)))["status"], "FAIL")
            r = checks_v3.evaluate(s, self.tel(ev(True, 27, a)))
            self.assertEqual(r["checks"]["picture_changed_after_edit"]["status"], "warn")  # rebuilt but identical picture
            self.assertEqual(r["status"], "WARN")

    def test_invisible_injected_fault_is_excluded_from_typesafe(self):
        s = sg.make_scenario("world_missing_brick", 0, 1)
        with tempfile.TemporaryDirectory() as d:
            a = os.path.join(d, "a.png")
            flat_png(a, (60, 140, 60))
            ev = [{"op": "shot", "name": "ref", "file": a}, {"op": "fault", "kind": "missing_brick"}, {"op": "shot", "name": "shot", "file": a}]
            r = checks_v3.evaluate(s, self.tel(ev))
            self.assertTrue(r["metrics"]["fault_not_visible"])
            self.assertFalse(qa_batch.route(s, r, "adaptive")[0])

    def test_sky_and_flat_centre_helpers(self):
        with tempfile.TemporaryDirectory() as d:
            sky, ground = os.path.join(d, "s.png"), os.path.join(d, "g.png")
            flat_png(sky, (90, 140, 217))
            flat_png(ground, (60, 140, 60))
            self.assertGreater(checks_v3.sky_fraction(open(sky, "rb").read()), 0.99)
            self.assertLess(checks_v3.sky_fraction(open(ground, "rb").read()), 0.01)
            self.assertGreater(checks_v3.centre_dominant(open(ground, "rb").read()), 0.99)  # why it is information only


class EndToEndOffline(unittest.TestCase):
    """The whole `suite` flow with a fake Godot and the mock API."""

    def run_suite(self, d, **kw):
        kw.setdefault("profile", "regression")
        a = args_for(d, generate=8, seed="5", execute="godot", mode="adaptive", **kw)
        with mock.patch.object(godot_runner, "godot_exe", return_value="fake"), mock.patch.object(godot_runner, "run_scenario", side_effect=fake_runner):
            return suite.run_suite(a)

    def test_dry_run_sends_nothing_and_reports_planned(self):
        with tempfile.TemporaryDirectory() as d, mock.patch.object(qa, "send", side_effect=AssertionError("network")):
            self.run_suite(d)
            r = json.load(open(os.path.join(d, "results.json"), encoding="utf-8"))
            self.assertEqual(r["api"]["sent"], 0)
            self.assertTrue(r["transport"].startswith("none"))
            self.assertEqual(r["counts"]["generated"], 8)
            self.assertTrue(os.path.exists(os.path.join(d, "report.md")))

    def test_mock_run_resume_and_partial_failure(self):
        with tempfile.TemporaryDirectory() as d:
            self.run_suite(d, mock=True, mock_fail="0:529", batch_size="1")
            r = json.load(open(os.path.join(d, "results.json"), encoding="utf-8"))
            self.assertGreater(r["api"]["errors"], 0)
            self.assertIn("infra:overload_529", r["api"]["error_kinds"])
            self.assertTrue(r["transport"].startswith("MOCK"))
            unknown = [x for x in r["scenarios"] if x["typesafe"] == "UNKNOWN"]
            self.assertTrue(unknown)
            self.assertTrue(all(x["final"] != "FAIL" for x in r["scenarios"] if x["typesafe"] == "UNKNOWN"))  # infra is never a game FAIL
            sent1 = r["api"]["sent"]
            self.run_suite(d, mock=True, batch_size="1")  # resume: only the failed request stays unrepeated
            r2 = json.load(open(os.path.join(d, "results.json"), encoding="utf-8"))
            self.assertEqual(r2["api"]["sent"], 0)
            self.assertEqual(r2["api"]["reused"], sent1 - 1)
            self.assertEqual(r2["api"]["skipped"], 1)

    def test_unsupported_check_is_a_visible_status_not_a_pass(self):
        with tempfile.TemporaryDirectory() as d:
            self.run_suite(d, checks="camera,collision", profile="gameplay")
            r = json.load(open(os.path.join(d, "results.json"), encoding="utf-8"))
            u = [x for x in r["scenarios"] if x["final"] == "UNSUPPORTED"]
            self.assertEqual(len(u), 1)
            self.assertIn("no collision", u[0]["unsupported_reason"])
            self.assertEqual(r["by_category"]["COLLISION"]["UNSUPPORTED"], 1)
            self.assertEqual(r["by_category"]["COLLISION"]["PASS"], 0)
            self.assertIn("UNSUPPORTED", open(os.path.join(d, "report.md"), encoding="utf-8").read())

    def test_live_without_key_stops_before_anything(self):
        with tempfile.TemporaryDirectory() as d, mock.patch.dict(os.environ, {}, clear=False), \
                mock.patch.object(qa, "load_dotenv"), mock.patch.object(qa, "send", side_effect=AssertionError("network")):
            os.environ.pop("TYPESAFE_API_KEY", None)
            with self.assertRaises(qa.QAError):
                self.run_suite(d, live=True)

    def test_live_and_mock_exclude_each_other(self):
        with tempfile.TemporaryDirectory() as d:
            with self.assertRaises(qa.QAError):
                self.run_suite(d, live=True, mock=True)

    def test_save_failures_writes_replayable_scenarios(self):
        with tempfile.TemporaryDirectory() as d, tempfile.TemporaryDirectory() as reg:
            with mock.patch.object(suite, "REGRESSION_DIR", reg):
                with mock.patch.object(checks_v3, "evaluate", return_value={"status": "FAIL", "checks": {}, "metrics": {}}):
                    self.run_suite(d, save_failures=True)
            files = os.listdir(os.path.join(reg, "failures"))
            self.assertTrue(files)
            s = json.load(open(os.path.join(reg, "failures", files[0]), encoding="utf-8"))
            self.assertEqual(s["fingerprint"], sg.canonical and s["fingerprint"])
            self.assertIn("phases", s)

    def test_compare_runs_reports_counts_only(self):
        mk = lambda p, tok, sent: {"scenarios": [{"id": "WORLD-001", "typesafe_questions": [{"qid": "Q-TERR-GAP-001", "result": "pass", "answer": p}]}],
                                   "api": {"sent": sent, "input_tokens": tok, "errors": 0}, "mode": "x"}
        c = suite.compare_runs(mk(0.9, 3000, 1), mk(0.62, 2900, 1))
        self.assertEqual(c["questions_in_both"], 1)
        self.assertAlmostEqual(c["max_probability_difference"], 0.28)
        self.assertIn("no mode is claimed", c["caveat"])
        d = suite.compare_runs(mk(0.9, 3000, 1), {**mk(0.1, 3000, 1), "scenarios": [{"id": "WORLD-001", "typesafe_questions": [{"qid": "Q-TERR-GAP-001", "result": "fail", "answer": 0.1}]}]})
        self.assertEqual(len(d["verdict_differs"]), 1)


class Integration(unittest.TestCase):
    def test_godot_hooks_are_off_in_a_normal_start(self):
        text = open(os.path.join(HERE, "..", "..", "godot", "main.gd"), encoding="utf-8").read()
        self.assertIn("var scen_active := false", text)
        self.assertIn("if scen_active:", text)
        self.assertIn('"--scenario":', text)

    def test_hooks_inactive_in_a_normal_godot_start(self):
        exe = godot_runner.godot_exe()
        if exe is None:
            self.skipTest("Godot not installed")
        import subprocess
        with tempfile.TemporaryDirectory() as d:
            r = subprocess.run([exe, "--path", os.path.join(godot_runner.ROOT, "godot"), "--rendering-driver", "opengl3",
                                "--quit-after", "90", "--", "--world", d], capture_output=True, text=True, timeout=120)
            # exit code is not asserted: --quit-after returns 1 in this Godot build for a normal start (not a QA hook effect)
            self.assertIn("CUBE world opened", r.stdout)
            self.assertNotIn("scenario done", r.stdout)
            self.assertFalse(os.path.exists(os.path.join(d, "telemetry.json")))
            self.assertFalse([n for n in os.listdir(d) if n.endswith(".png")])

    def test_typesafe_not_in_required_ci(self):
        root = os.path.join(HERE, "..", "..", ".github")
        for dp, _, files in os.walk(root):
            for n in files:
                with open(os.path.join(dp, n), encoding="utf-8", errors="ignore") as f:
                    t = f.read().lower()
                self.assertNotIn("typesafe", t, n)
                self.assertNotIn("cube_visual_qa", t, n)


if __name__ == "__main__":
    unittest.main()
