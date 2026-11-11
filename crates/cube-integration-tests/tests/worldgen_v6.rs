//! "terrain-v6" (v5 plus a dungeon under every ruin): bricks equal the voxel oracle across
//! ruin borders, nothing outside a footprint block below the floor differs from v5, the
//! stairway can be walked from the floor to the chamber one voxel at a time, the chamber is
//! closed except for the stairway, and v5 worlds are unchanged. CI host only (DG-2).
#![cfg(not(loom))]

use std::collections::{HashSet, VecDeque};

use cube_core::{BrickCoord, GeneratorVersion, WorldSeed};
use cube_worldgen::materials::{BRICK, STONE};
use cube_worldgen::{generate_brick, ruin_of_region, voxel_at};

fn v(n: u32) -> GeneratorVersion {
    GeneratorVersion::new(n)
}

fn at(seed: WorldSeed, ver: u32, x: i64, y: i64, z: i64) -> u16 {
    voxel_at(seed, v(ver), x, y, z).unwrap()
}

fn ruins(seed: WorldSeed, range: i64) -> Vec<cube_worldgen::v5::Ruin> {
    let mut out = Vec::new();
    for rx in -range..range {
        for rz in -range..range {
            out.extend(ruin_of_region(seed, v(6), rx, rz).unwrap());
        }
    }
    out
}

#[test]
fn version_six_is_the_default_and_version_five_is_legacy_and_unchanged_by_it() {
    assert_eq!(cube_worldgen::current_version(), v(6));
    let seed = WorldSeed::new(7);
    let r = ruin_of_region(
        seed,
        v(5),
        (-95_i64).div_euclid(40),
        (-29_i64).div_euclid(40),
    )
    .unwrap()
    .unwrap();
    // v5 keeps the natural ground under the floor: its voxel in the future chamber is not air.
    assert_ne!(at(seed, 5, r.x, r.floor - 5, r.z + 3), 0);
    assert_eq!(at(seed, 6, r.x, r.floor - 5, r.z + 3), 0, "v6 chamber air");
}

#[test]
fn only_the_dungeon_block_differs_from_v5() {
    let seed = WorldSeed::new(7);
    let mut differing = 0;
    for r in ruins(seed, 12).iter().take(12) {
        for x in r.x - 8..=r.x + 8 {
            for z in r.z - 8..=r.z + 8 {
                for y in r.floor - 12..=r.floor + 24 {
                    let (a, b) = (at(seed, 5, x, y, z), at(seed, 6, x, y, z));
                    if a != b {
                        let (dx, dz, dy) = (x - r.x, z - r.z, y - r.floor);
                        assert!(
                            dx.abs() <= 5 && dz.abs() <= 5 && (-8..=0).contains(&dy),
                            "difference outside the dungeon block: {dx} {dy} {dz}"
                        );
                        differing += 1;
                    }
                }
            }
        }
    }
    assert!(differing > 500, "{differing} voxels differ");
}

#[test]
fn bricks_equal_the_voxel_function_across_ruin_borders() {
    let seed = WorldSeed::new(7);
    let mut checked = 0;
    for r in ruins(seed, 12).iter().take(20) {
        for bx in (r.x - 5).div_euclid(16)..=(r.x + 5).div_euclid(16) {
            for by in (r.floor - 8).div_euclid(16)..=(r.floor + 20).div_euclid(16) {
                for bz in (r.z - 5).div_euclid(16)..=(r.z + 5).div_euclid(16) {
                    let b = BrickCoord::new(bx, by, bz);
                    let o = b.origin().unwrap();
                    let g = generate_brick(seed, v(6), b).unwrap().materials();
                    for (i, m) in g.iter().enumerate() {
                        let (lx, ly, lz) =
                            ((i % 16) as i64, ((i / 16) % 16) as i64, (i / 256) as i64);
                        assert_eq!(
                            *m,
                            at(seed, 6, o.x + lx, o.y + ly, o.z + lz),
                            "brick {b:?} voxel {lx},{ly},{lz} (ruin {r:?})"
                        );
                    }
                    checked += 1;
                }
            }
        }
    }
    assert!(checked > 40, "{checked} bricks around ruins checked");
}

/// A voxel the player body (two voxels tall, one wide) can stand in: it and the one above are air.
fn free(seed: WorldSeed, x: i64, y: i64, z: i64) -> bool {
    at(seed, 6, x, y, z) == 0 && at(seed, 6, x, y + 1, z) == 0
}

#[test]
fn the_stairway_leads_from_the_floor_down_to_the_chamber_one_voxel_at_a_time() {
    let seed = WorldSeed::new(7);
    for r in ruins(seed, 12).iter().take(10) {
        // Standing voxel (feet) of step k: one above its top voxel.
        let mut prev = (r.floor + 1, -4_i64);
        assert!(
            free(seed, r.x - 3, prev.0, r.z + prev.1),
            "entrance at the floor"
        );
        for k in 0..6_i64 {
            let (z, feet) = (r.z - 4 + k, r.floor - k);
            assert_ne!(at(seed, 6, r.x - 3, feet - 1, z), 0, "step {k} is solid");
            assert!(free(seed, r.x - 3, feet, z), "step {k} has headroom");
            assert!(
                free(seed, r.x - 4, feet, z),
                "the stairway is two wide at step {k}"
            );
            assert!(
                prev.0 - feet <= 1,
                "never drops more than one voxel at a time"
            );
            prev = (feet, z - r.z);
        }
        // The chamber.
        for dx in -4..=4 {
            for dz in 2..=4 {
                assert_eq!(at(seed, 6, r.x + dx, r.floor - 7, r.z + dz), BRICK);
                for dy in -6..=-3 {
                    let m = at(seed, 6, r.x + dx, r.floor + dy, r.z + dz);
                    let pedestal = (dx, dz, dy) == (0, 4, -6);
                    assert_eq!(m, if pedestal { BRICK } else { 0 }, "{dx} {dy} {dz}");
                }
            }
        }
        // Down from the last step into the chamber is one voxel.
        assert!(prev.0 - (r.floor - 6) <= 1);
    }
}

#[test]
fn the_chamber_is_closed_except_through_the_stairway() {
    let seed = WorldSeed::new(7);
    for r in ruins(seed, 12).iter().take(10) {
        // Flood fill the air from the chamber inside a box around the ruin.
        let (x0, x1, z0, z1) = (r.x - 8, r.x + 8, r.z - 8, r.z + 8);
        let (y0, y1) = (r.floor - 10, r.floor + 6);
        let start = (r.x + 2, r.floor - 5, r.z + 3);
        let mut seen = HashSet::new();
        let mut q = VecDeque::from([start]);
        seen.insert(start);
        let mut top_reached = false;
        while let Some((x, y, z)) = q.pop_front() {
            if y > r.floor {
                // The open air above the floor is the way out; do not follow it into the sky.
                top_reached = true;
                continue;
            }
            for (dx, dy, dz) in [
                (1, 0, 0),
                (-1, 0, 0),
                (0, 1, 0),
                (0, -1, 0),
                (0, 0, 1),
                (0, 0, -1),
            ] {
                let n = (x + dx, y + dy, z + dz);
                if n.0 < x0 || n.0 > x1 || n.2 < z0 || n.2 > z1 || n.1 < y0 || n.1 > y1 {
                    panic!("air escapes the box at {n:?} (ruin {r:?})");
                }
                if at(seed, 6, n.0, n.1, n.2) == 0 && seen.insert(n) {
                    q.push_back(n);
                }
            }
        }
        assert!(
            top_reached,
            "the stairway reaches the open air above the floor"
        );
        // Under the floor level the air stays inside the footprint.
        assert!(
            seen.iter()
                .filter(|p| p.1 < r.floor)
                .all(|p| (p.0 - r.x).abs() <= 5 && (p.2 - r.z).abs() <= 5),
            "no tunnel leaves the footprint below the floor"
        );
        assert_eq!(at(seed, 6, r.x - 3, r.floor - 8, r.z - 4), STONE);
    }
}

#[test]
fn the_pinned_ruin_of_seed_7_has_its_pedestal() {
    let seed = WorldSeed::new(7);
    assert_eq!(at(seed, 6, -95, -10 - 6, -29 + 4), BRICK);
    assert_eq!(at(seed, 6, -95, -10 - 5, -29 + 4), 0);
    assert_eq!(at(seed, 6, -95, -10, -29), BRICK);
}
