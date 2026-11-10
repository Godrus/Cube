"""Reproducible defect fixtures for scene-smoke, derived from fixtures/good-1.png.

Run:  python make_fixtures.py
Deterministic: the same good-1.png always gives byte-identical defect images. The defects are
synthetic (pixels edited), so they say how Jev reacts to missing ground, not to a real renderer
bug; a real defect is a Godot screenshot with bricks left out (see README).
"""

import json
import os

import pngio

HERE = os.path.dirname(os.path.abspath(__file__))
FIX = os.path.join(HERE, "fixtures")


def pixel(px, w, c, x, y):
    i = (y * w + x) * c
    return list(px[i:i + c])


def main():
    with open(os.path.join(FIX, "good-1.png"), "rb") as f:
        w, h, c, px = pngio.read_png(f.read())

    def save(name, pixels):
        with open(os.path.join(FIX, name), "wb") as f:
            f.write(pngio.write_png(w, h, c, pixels))

    # 1. a completely black frame
    save("black.png", bytearray(w * h * c))

    # 2. sky only: a vertical gradient from the sampled top colour to the horizon colour
    top, horizon = pixel(px, w, c, w // 2, 2), pixel(px, w, c, w // 2, 40)
    sky = bytearray(w * h * c)
    for y in range(h):
        t = min(1.0, y / 40.0)
        row = [round(top[k] + (horizon[k] - top[k]) * t) for k in range(c)]
        for x in range(w):
            sky[(y * w + x) * c:(y * w + x) * c + c] = bytes(row)
    save("no-terrain.png", sky)

    # 3. a large missing area of ground: a rectangle in the lower centre replaced by sky colour
    hole = bytearray(px)
    x0, x1, y0, y1 = w * 3 // 10, w * 7 // 10, h * 45 // 100, h * 90 // 100
    for y in range(y0, y1):
        for x in range(x0, x1):
            hole[(y * w + x) * c:(y * w + x) * c + c] = bytes(horizon)
    save("missing-chunk.png", hole)

    labels = {
        "good-1.png": {"expect": "PASS", "why": "screenshot of the first world, seed 7"},
        "good-2.png": {"expect": "PASS", "why": "screenshot of a second world, seed 11"},
        "black.png": {"expect": "FAIL", "why": "completely black frame"},
        "no-terrain.png": {"expect": "FAIL", "why": "sky gradient only, no ground"},
        "missing-chunk.png": {"expect": "FAIL", "why": "good-1 with a large rectangle of ground replaced by sky colour"},
    }
    with open(os.path.join(FIX, "labels.json"), "w", encoding="utf-8") as f:
        json.dump(labels, f, indent=2)
    print("fixtures written:", ", ".join(labels))


if __name__ == "__main__":
    main()
