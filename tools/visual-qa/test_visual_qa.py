"""Offline tests of Cube Visual QA: nothing here touches the network or needs a key.

Run:  python -m unittest test_visual_qa -v   (from tools/visual-qa)
"""

import contextlib
import io
import json
import os
import tempfile
import unittest
from unittest import mock

import cube_visual_qa as qa
import pngio

HERE = os.path.dirname(os.path.abspath(__file__))
FIX = os.path.join(HERE, "fixtures")


def fixture(name):
    with open(os.path.join(FIX, name), "rb") as f:
        return f.read()


def response(**answers):
    return {"model": "jev-1.14.0", "answers": answers, "usage": {"input_tokens": 1, "output_tokens": 0}}


def good_smoke():
    return response(
        voxel_ground_visible={"type": "noul", "noul": 0.97},
        frame_is_one_flat_colour={"type": "noul", "noul": 0.02},
        large_hole_in_ground={"type": "noul", "noul": 0.05},
        tears_in_terrain={"type": "noul", "noul": 0.1},
        scene_kind={"type": "choice", "choice": "voxel_landscape_with_ground", "confidence": 0.9,
                    "probabilities": {"voxel_landscape_with_ground": 0.93, "other_scene": 0.07}},
    )


class Png(unittest.TestCase):
    def test_round_trip_and_dimensions(self):
        px = bytearray(range(256)) * 3
        data = pngio.write_png(16, 16, 3, px[:16 * 16 * 3])
        self.assertEqual(pngio.dimensions(data), (16, 16))
        w, h, c, back = pngio.read_png(data)
        self.assertEqual((w, h, c, bytes(back)), (16, 16, 3, bytes(px[:768])))

    def test_the_godot_screenshot_decodes(self):
        w, h, c, px = pngio.read_png(fixture("good-1.png"))
        self.assertEqual((w, h), (1280, 720))
        self.assertEqual(len(px), w * h * c)


class Questionnaires(unittest.TestCase):
    def test_all_shipped_questionnaires_are_valid(self):
        for name in ("scene-smoke", "terrain-integrity", "block-edit"):
            q = qa.load_questionnaire(name)
            self.assertTrue(q["questions"])
            # a Box is always gated by a presence question that exists and is a noul
            for spec in q["questions"].values():
                if spec["type"] == "box":
                    self.assertEqual(q["questions"][spec["only_if"]]["type"], "noul")

    def test_a_box_without_a_presence_question_is_refused(self):
        q = {"name": "x", "images": ["a"], "questions": {"b": {"type": "box", "instructions": "i"}}}
        with self.assertRaises(qa.QAError):
            qa.validate_questionnaire(q)


class Requests(unittest.TestCase):
    def test_every_image_is_referenced_once_with_a_unique_reference(self):
        q = qa.load_questionnaire("block-edit")
        body, parts = qa.build_request(
            q, {"before": fixture("edit-before.png"), "after": fixture("edit-after.png")}, "jev-1.14.0")
        self.assertEqual(set(body["state"]), {"before", "after"})
        refs = list(body["state"].values())
        self.assertEqual(len(set(refs)), 2)
        self.assertEqual(set(body["assets"]), set(refs))
        for ref, asset in body["assets"].items():
            self.assertEqual(asset["type"], "image")
            self.assertIn(asset["part"], parts)
        self.assertNotIn("data_base64", json.dumps(body))
        for qn, spec in body["questions"].items():
            self.assertNotIn("expect", spec, "expectations stay in code, never sent to the model")

    def test_the_stored_request_has_no_secret_and_no_image_bytes(self):
        q = qa.load_questionnaire("scene-smoke")
        body, parts = qa.build_request(q, {"screenshot": fixture("good-1.png")}, "jev-1.14.0")
        d = json.dumps(qa.describe_request(body, parts))
        self.assertIn("<redacted>", d)
        self.assertIn("TypeSafe-Danger-Zone-Images", d)
        self.assertLess(len(d), 5000)

    def test_multipart_has_the_request_part_first_and_one_file_per_image(self):
        q = qa.load_questionnaire("scene-smoke")
        body, parts = qa.build_request(q, {"screenshot": fixture("black.png")}, "m")
        payload, ctype = qa.multipart(body, parts)
        self.assertTrue(ctype.startswith("multipart/form-data; boundary="))
        self.assertLess(payload.index(b'name="request"'), payload.index(b'name="image-screenshot"'))
        self.assertIn(b"Content-Type: application/json", payload)
        self.assertIn(b"Content-Type: image/png", payload)

    def test_documented_image_limits_are_checked_before_sending(self):
        small = pngio.write_png(100, 100, 3, bytearray(100 * 100 * 3))
        with self.assertRaises(qa.QAError):
            qa.check_image(small, "tiny")
        with self.assertRaises(qa.QAError):
            qa.check_image(b"not a png", "junk")
        self.assertEqual(qa.check_image(fixture("good-1.png"), "ok"), (1280, 720))


class Evaluation(unittest.TestCase):
    def setUp(self):
        self.q = qa.load_questionnaire("scene-smoke")

    def test_a_good_scene_passes(self):
        verdict, results, _ = qa.evaluate(self.q, good_smoke())
        self.assertEqual(verdict, "PASS", results)

    def test_one_clear_failure_fails(self):
        r = good_smoke()
        r["answers"]["large_hole_in_ground"]["noul"] = 0.95
        self.assertEqual(qa.evaluate(self.q, r)[0], "FAIL")

    def test_a_middling_answer_is_a_warning_not_a_pass(self):
        r = good_smoke()
        r["answers"]["tears_in_terrain"]["noul"] = 0.5
        self.assertEqual(qa.evaluate(self.q, r)[0], "WARN")

    def test_a_wrong_scene_class_fails(self):
        r = good_smoke()
        r["answers"]["scene_kind"] = {"type": "choice", "choice": "empty_or_black_frame", "confidence": 0.9,
                                       "probabilities": {"voxel_landscape_with_ground": 0.02,
                                                         "empty_or_black_frame": 0.98}}
        self.assertEqual(qa.evaluate(self.q, r)[0], "FAIL")

    def test_missing_answers_or_a_broken_response_are_unknown_never_pass(self):
        r = good_smoke()
        del r["answers"]["scene_kind"]
        self.assertEqual(qa.evaluate(self.q, r)[0], "UNKNOWN")
        self.assertEqual(qa.evaluate(self.q, {})[0], "UNKNOWN")
        self.assertEqual(qa.evaluate(self.q, None)[0], "UNKNOWN")

    def test_a_box_is_only_reported_when_its_presence_question_says_yes(self):
        q = qa.load_questionnaire("terrain-integrity")

        def resp(hole):
            return response(
                hole_in_terrain={"type": "noul", "noul": hole},
                hole_location={"type": "box", "box": {"top": 0.4, "bottom": 0.8, "left": 0.3, "right": 0.7},
                               "confidence": 0.8},
                streaks_or_slivers={"type": "noul", "noul": 0.1},
                grass_on_top={"type": "noul", "noul": 0.9},
                placeholder_colour={"type": "noul", "noul": 0.05})

        v, results, _ = qa.evaluate(q, resp(0.95))
        box = [r for r in results if r["question"] == "hole_location"][0]
        self.assertEqual((v, box["result"]), ("FAIL", "info"))
        self.assertIsNotNone(box["box"])
        v, results, _ = qa.evaluate(q, resp(0.02))
        box = [r for r in results if r["question"] == "hole_location"][0]
        self.assertEqual((v, box["result"], box["box"]), ("PASS", "not-present", None))


class Runs(unittest.TestCase):
    def test_a_dry_run_sends_nothing_and_reports_unknown(self):
        with tempfile.TemporaryDirectory() as out, mock.patch.object(qa, "send") as send:
            q = qa.load_questionnaire("scene-smoke")
            run = qa.run_one(q, {"screenshot": os.path.join(FIX, "good-1.png")},
                             qa.DEFAULT_MODEL, False, None, out, "t")
            send.assert_not_called()
            self.assertEqual(run["verdict"], "UNKNOWN")
            self.assertTrue(os.path.exists(os.path.join(out, "t.request.json")))
            self.assertIn("nothing sent", run["mode"])

    def test_live_without_a_key_is_a_clear_error_and_sends_nothing(self):
        env = {k: v for k, v in os.environ.items() if k != "TYPESAFE_API_KEY"}
        err = io.StringIO()
        with mock.patch.dict(os.environ, env, clear=True), mock.patch.object(qa, "send") as send, \
                contextlib.redirect_stderr(err), tempfile.TemporaryDirectory() as out:
            code = qa.main(["check", "scene-smoke", "screenshot=" + os.path.join(FIX, "good-1.png"),
                            "--live", "--out", out])
        self.assertEqual(code, 2)
        send.assert_not_called()
        self.assertIn("TYPESAFE_API_KEY", err.getvalue())

    def test_an_api_failure_is_unknown_and_the_key_never_reaches_a_file(self):
        secret = "sk-test-SECRET-123"
        with tempfile.TemporaryDirectory() as out, \
                mock.patch.object(qa, "send", return_value=("error", "HTTP 401: bad key <key>")):
            q = qa.load_questionnaire("scene-smoke")
            run = qa.run_one(q, {"screenshot": os.path.join(FIX, "good-1.png")},
                             qa.DEFAULT_MODEL, True, secret, out, "t")
            self.assertEqual(run["verdict"], "UNKNOWN")
            for name in os.listdir(out):
                with open(os.path.join(out, name), encoding="utf-8") as f:
                    self.assertNotIn(secret, f.read())

    def test_send_replaces_the_key_in_error_text(self):
        import urllib.error
        secret = "sk-test-SECRET-123"
        err = urllib.error.URLError(f"echo {secret}")
        with mock.patch("urllib.request.urlopen", side_effect=err):
            q = qa.load_questionnaire("scene-smoke")
            body, parts = qa.build_request(q, {"screenshot": fixture("black.png")}, "m")
            status, text = qa.send(body, parts, secret)
        self.assertEqual(status, "error")
        self.assertNotIn(secret, text)

    def test_a_recorded_response_is_evaluated_and_reported_as_markdown(self):
        with tempfile.TemporaryDirectory() as out:
            q = qa.load_questionnaire("scene-smoke")
            run = qa.run_one(q, {"screenshot": os.path.join(FIX, "good-1.png")},
                             qa.DEFAULT_MODEL, False, None, out, "m", mock=good_smoke())
            self.assertEqual(run["verdict"], "PASS")
            with open(os.path.join(out, "m.report.md"), encoding="utf-8") as f:
                md = f.read()
            self.assertIn("verdict: **PASS**", md)
            self.assertIn("voxel_ground_visible", md)

    def test_the_fixture_dry_run_covers_all_five_labelled_images(self):
        with tempfile.TemporaryDirectory() as out, contextlib.redirect_stdout(io.StringIO()):
            code = qa.main(["fixtures", "--out", out])
            self.assertEqual(code, 0)
            with open(os.path.join(out, "fixtures-summary.md"), encoding="utf-8") as f:
                md = f.read()
        for name in ("good-1.png", "good-2.png", "black.png", "no-terrain.png", "missing-chunk.png"):
            self.assertIn(name, md)

    def test_the_live_fixture_run_respects_the_request_budget(self):
        with mock.patch.dict(os.environ, {"TYPESAFE_API_KEY": "k"}), \
                mock.patch.object(qa, "send") as send, tempfile.TemporaryDirectory() as out, \
                contextlib.redirect_stderr(io.StringIO()):
            code = qa.main(["fixtures", "--live", "--max-requests", "2", "--out", out])
        self.assertEqual(code, 2)
        send.assert_not_called()


if __name__ == "__main__":
    unittest.main()
