# Cube Visual Style Guide (canonical)

**Status:** owner directive, 2026-11-11. This is the single place for the visual direction. No other style or art-governance document exists; do not add one.

## Direction

**Cube World Alpha (2013) and Veloren.** A cohesive, pleasant, playable stylised voxel RPG, not a showcase of maximum voxel detail.

* Characters and creatures: stylised, expressive, readable silhouettes, simple strong shapes, exaggerated proportions allowed.
* Nature: varied (biomes, trees, rocks, water, flora) with one coherent palette per biome.
* Colour: harmonised palettes, limited per area, saturated but not noisy.
* Light: pleasant and warm, soft day/night mood, fantasy atmosphere.
* Voxel size: the existing world voxel (1 unit) stays the visual standard for terrain and objects.

## Priority change (explicit)

The previous direction, high-detail micro-voxel characters after the Town to City / Teardown concept (`F-CHR-001`, branch `feat/voxel-character-proto`), is **cancelled as the current task and deferred as a visual standard**. It is not deleted.

Not to be done now, without a separate owner task:

* a new high-resolution voxel character pipeline;
* raising character geometry resolution for micro-detail;
* rewriting the voxel renderer or WorldGen;
* complex LOD systems made for micro-voxel models;
* continuing the detailed traveller character;
* reworking the Geometry Core.

Kept, not used as a mandatory visual standard: UGS architecture, support for different resolutions, Composite Geometry, micro-voxels, and everything already implemented. They may be used where they help, never required.

## State of earlier visual work

* `feat/voxel-character-proto` (HEAD `533f69d`, pushed, CI green, **not merged to `dev`**): parked as an experiment with its report `docs/handoff/CHAR-01-REPORT.md` on that branch. Nothing from it is in `dev`. Do not merge it automatically; the owner may later reuse it or discard it.
* A separate "VFX task": no branch, file or commit about it exists in the repository or its worktrees (searched 2026-11-11), so there is nothing to keep or integrate. If it exists elsewhere (another chat or machine), it stays unintegrated until the owner asks.
* No other uncommitted visual experiment was touched.

## What to do next (visual side)

Only small steps that serve the game and do not start a large graphics rework: for new content (creatures, items, trees) use simple stylised shapes at world-voxel scale and the biome palettes. A large graphics change needs its own owner task.
