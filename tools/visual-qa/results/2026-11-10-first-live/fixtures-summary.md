# Cube Visual QA: scene-smoke on the fixtures

- model: `jev-1.14.0`, mode: live

| image | expected | verdict |
|---|---|---|
| good-1.png | PASS | FAIL |
| good-2.png | PASS | FAIL |
| black.png | FAIL | FAIL |
| no-terrain.png | FAIL | FAIL |
| missing-chunk.png | FAIL | FAIL |

Agreement: 3 of 5.

- WRONG: good-1.png: expected PASS, got FAIL (screenshot of the first world, seed 7)
- WRONG: good-2.png: expected PASS, got FAIL (screenshot of a second world, seed 11)
