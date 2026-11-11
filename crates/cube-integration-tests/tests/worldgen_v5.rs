//! WGN2-04: "terrain-v5" (v4 plus ruins). Pinned hashes, bricks equal the voxel oracle (also for
//! ruins that cross brick borders), order and thread independence, everything outside a
//! footprint is v4, the ruin has the intended shape, none near the origin, the pinned ruin of
//! seed 7 used by the Godot smoke. CI host only (DG-2).
#![cfg(not(loom))]

use cube_core::{BrickCoord, GeneratorVersion, WorldSeed};
use cube_worldgen::materials::{BRICK, STONE};
use cube_worldgen::{generate_brick, ruin_of_region, surface_height, voxel_at};

const SEED: WorldSeed = WorldSeed::new(42);

fn v(n: u32) -> GeneratorVersion {
    GeneratorVersion::new(n)
}

fn hash_of(seed: WorldSeed, b: BrickCoord) -> u64 {
    generate_brick(seed, v(5), b).unwrap().hash(seed, v(5))
}

/// Pinned for generator version 5 and seed 42. A change here means the output of "terrain-v5"
/// changed: that needs a new generator version, never an edit of these numbers.
const GOLDEN: [((i64, i64, i64), u64); 9] = [
    ((0, 0, 0), 279037331002125898),
    ((-1, -1, -1), 2786509039926900522),
    ((3, -2, -7), 9239576380591165735),
    ((0, -4, 0), 2984563070589569978),
    ((1_000_000, 0, -1_000_000), 15640916208598626365),
    ((0, 20, 0), 1935841348454470494),
    ((6, -1, -5), 15011320178530105913),
    ((-9, -1, 8), 5427858038982346675),
    ((-5, -1, -1), 3300830266177123222),
];

#[test]
fn the_same_seed_and_version_always_give_the_same_pinned_bricks() {
    for ((x, y, z), want) in GOLDEN {
        assert_eq!(
            hash_of(SEED, BrickCoord::new(x, y, z)),
            want,
            "pinned hash of terrain-v5 changed at {x},{y},{z}"
        );
    }
    let s7 = WorldSeed::new(7);
    assert_eq!(
        generate_brick(s7, v(5), BrickCoord::new(-6, -1, -2))
            .unwrap()
            .hash(s7, v(5)),
        1181774230944056262,
        "the pinned ruin brick of seed 7"
    );
}

fn ruins(seed: WorldSeed, range: i64) -> Vec<cube_worldgen::v5::Ruin> {
    let mut out = Vec::new();
    for rx in -range..range {
        for rz in -range..range {
            out.extend(ruin_of_region(seed, v(5), rx, rz).unwrap());
        }
    }
    out
}

#[test]
fn bricks_equal_the_voxel_function_across_ruin_borders() {
    let seed = WorldSeed::new(7);
    let mut checked = 0;
    for r in ruins(seed, 12).iter().take(25) {
        for bx in (r.x - 5).div_euclid(16)..=(r.x + 5).div_euclid(16) {
            for by in (r.floor - 8).div_euclid(16)..=(r.floor + 20).div_euclid(16) {
                for bz in (r.z - 5).div_euclid(16)..=(r.z + 5).div_euclid(16) {
                    let b = BrickCoord::new(bx, by, bz);
                    let o = b.origin().unwrap();
                    let g = generate_brick(seed, v(5), b).unwrap().materials();
                    for (i, m) in g.iter().enumerate() {
                        let (lx, ly, lz) =
                            ((i % 16) as i64, ((i / 16) % 16) as i64, (i / 256) as i64);
                        assert_eq!(
                            *m,
                            voxel_at(seed, v(5), o.x + lx, o.y + ly, o.z + lz).unwrap(),
                            "brick {b:?} voxel {lx},{ly},{lz} (ruin {r:?})"
                        );
                    }
                    checked += 1;
                }
            }
        }
    }
    assert!(checked > 50, "{checked} bricks around ruins checked");
}

#[test]
fn generation_does_not_depend_on_order_or_threads() {
    let seed = WorldSeed::new(7);
    let bricks: Vec<BrickCoord> = (-8..-3)
        .flat_map(|x| (-2..1).flat_map(move |y| (-4..0).map(move |z| BrickCoord::new(x, y, z))))
        .collect();
    let h = |b: &BrickCoord| generate_brick(seed, v(5), *b).unwrap().hash(seed, v(5));
    let forward: Vec<u64> = bricks.iter().map(h).collect();
    let mut reversed: Vec<u64> = bricks.iter().rev().map(h).collect();
    reversed.reverse();
    assert_eq!(forward, reversed);
    let threaded: Vec<u64> = std::thread::scope(|s| {
        let hs: Vec<_> = bricks
            .chunks(bricks.len().div_ceil(4))
            .map(|c| s.spawn(move || c.iter().map(h).collect::<Vec<_>>()))
            .collect();
        hs.into_iter().flat_map(|j| j.join().unwrap()).collect()
    });
    assert_eq!(forward, threaded);
}

#[test]
fn only_footprints_differ_from_v4_and_none_is_near_the_origin() {
    let seed = WorldSeed::new(7);
    let all = ruins(seed, 19);
    assert!(all.len() > 100, "ruins in the finite world: {}", all.len());
    for r in &all {
        assert!(
            r.x.abs() > 50 || r.z.abs() > 50,
            "too close to the origin: {r:?}"
        );
    }
    let inside = |x: i64, z: i64| {
        all.iter()
            .any(|r| (x - r.x).abs() <= 5 && (z - r.z).abs() <= 5)
    };
    let mut differing = 0;
    for x in (-300..300).step_by(1) {
        for z in (-300..300).step_by(4) {
            let h = surface_height(seed, v(5), x, z).unwrap();
            for y in [h - 9, h - 1, h, h + 1, h + 4, h + 12] {
                let (a, b) = (
                    voxel_at(seed, v(4), x, y, z).unwrap(),
                    voxel_at(seed, v(5), x, y, z).unwrap(),
                );
                if a != b {
                    assert!(
                        inside(x, z),
                        "v5 differs from v4 outside a footprint at {x},{y},{z}"
                    );
                    differing += 1;
                }
            }
        }
    }
    assert!(differing > 50, "ruins change voxels: {differing}");
}

#[test]
fn a_ruin_has_a_levelled_floor_a_door_pillars_an_altar_and_no_trees() {
    let seed = WorldSeed::new(7);
    let m = |x, y, z| voxel_at(seed, v(5), x, y, z).unwrap();
    for r in ruins(seed, 19) {
        for dx in -4..=4 {
            for dz in -4..=4 {
                assert_eq!(
                    m(r.x + dx, r.floor, r.z + dz),
                    BRICK,
                    "floor at {dx},{dz} of {r:?}"
                );
                // Clear above the floor inside the walls except the altar.
                if (dx, dz) != (0, 0) {
                    assert_eq!(m(r.x + dx, r.floor + 5, r.z + dz), 0, "open to the sky");
                }
            }
        }
        for dy in 1..=4 {
            for (dx, dz) in [(5, 5), (-5, 5), (5, -5), (-5, -5)] {
                assert_eq!(
                    m(r.x + dx, r.floor + dy, r.z + dz),
                    STONE,
                    "corner pillar {dy}"
                );
            }
        }
        assert_eq!(m(r.x, r.floor + 1, r.z), STONE, "altar");
        assert_eq!(m(r.x, r.floor + 2, r.z), STONE, "altar");
        assert_eq!(m(r.x, r.floor + 3, r.z), 0);
        for dx in -1..=1 {
            for dy in 1..=3 {
                assert_eq!(
                    m(r.x + dx, r.floor + dy, r.z + 5),
                    0,
                    "the door on the south side"
                );
            }
        }
        // Standing room above the floor in the middle ring is free for a creature.
        assert_eq!(m(r.x + 2, r.floor + 1, r.z + 2), 0);
        assert_eq!(m(r.x + 2, r.floor + 2, r.z + 2), 0);
    }
}

/// The ruin of seed 7 that the real-Godot smoke visits: centre (-95, -29), floor -10.
#[test]
fn the_pinned_ruin_of_seed_7_exists() {
    let s = WorldSeed::new(7);
    let r = ruin_of_region(s, v(5), (-95_i64).div_euclid(40), (-29_i64).div_euclid(40))
        .unwrap()
        .expect("a ruin");
    assert_eq!((r.x, r.z, r.floor), (-95, -29, -10));
    assert_eq!(voxel_at(s, v(5), -95, -10, -29).unwrap(), BRICK);
    assert_eq!(voxel_at(s, v(5), -95, -9, -29).unwrap(), STONE);
}

#[test]
fn versions_one_to_four_know_no_ruins() {
    for n in 1..=4 {
        for rx in -6..6 {
            for rz in -6..6 {
                assert_eq!(ruin_of_region(SEED, v(n), rx, rz).unwrap(), None);
            }
        }
    }
}
