//! WGN2-02: "terrain-v3" (v2 plus caves and ores). Pinned hashes, brick equals the voxel oracle,
//! order and thread independence, the surface is exactly v2's, caves and ores have the intended
//! shape (shares, depth rules, floors, openings), the pinned landmarks of seed 7 used by the
//! Godot smoke still exist, and versions 1 and 2 are untouched. CI host only (DG-2).
#![cfg(not(loom))]

use cube_core::{BrickCoord, GeneratorVersion, WorldSeed};
use cube_worldgen::materials::{COAL, IRON, STONE};
use cube_worldgen::v3::{CAVE_FLOOR, ORE_FLOOR};
use cube_worldgen::{generate_brick, surface_height, voxel_at};

const SEED: WorldSeed = WorldSeed::new(42);

fn v(n: u32) -> GeneratorVersion {
    GeneratorVersion::new(n)
}

fn hash_of(seed: WorldSeed, b: BrickCoord) -> u64 {
    generate_brick(seed, v(3), b).unwrap().hash(seed, v(3))
}

/// Pinned for generator version 3 and seed 42. A change here means the output of "terrain-v3"
/// changed: that needs a new generator version, never an edit of these numbers.
const GOLDEN: [((i64, i64, i64), u64); 8] = [
    ((0, 0, 0), 15135809872681592876),
    ((-1, -1, -1), 15147678244366703340),
    ((3, -2, -7), 6140503879701094849),
    ((0, -4, 0), 5033888323634592380),
    ((1_000_000, 0, -1_000_000), 8012749684542919715),
    ((0, 20, 0), 5061845838927268664),
    ((6, -1, -5), 4401226252336210847),
    ((-9, -3, 8), 10137557113383956824),
];

#[test]
fn the_same_seed_and_version_always_give_the_same_pinned_bricks() {
    for ((x, y, z), want) in GOLDEN {
        assert_eq!(
            hash_of(SEED, BrickCoord::new(x, y, z)),
            want,
            "pinned hash of terrain-v3 changed at {x},{y},{z}"
        );
    }
}

#[test]
fn a_brick_equals_the_voxel_function_across_borders_and_negative_space() {
    for b in [
        BrickCoord::new(0, -3, 0),
        BrickCoord::new(-1, -2, -1),
        BrickCoord::new(-7, -1, 5),
        BrickCoord::new(12, -4, -9),
        BrickCoord::new(6, -1, -5),
    ] {
        let g = generate_brick(SEED, v(3), b).unwrap();
        let o = b.origin().unwrap();
        for (i, m) in g.materials().iter().enumerate() {
            let (lx, ly, lz) = ((i % 16) as i64, ((i / 16) % 16) as i64, (i / 256) as i64);
            assert_eq!(
                *m,
                voxel_at(SEED, v(3), o.x + lx, o.y + ly, o.z + lz).unwrap(),
                "brick {b:?} voxel {lx},{ly},{lz}"
            );
        }
    }
}

#[test]
fn generation_does_not_depend_on_order_or_threads() {
    let bricks: Vec<BrickCoord> = (-3..3)
        .flat_map(|x| (-6..0).flat_map(move |y| (-3..3).map(move |z| BrickCoord::new(x, y, z))))
        .collect();
    let forward: Vec<u64> = bricks.iter().map(|b| hash_of(SEED, *b)).collect();
    let mut reversed: Vec<u64> = bricks.iter().rev().map(|b| hash_of(SEED, *b)).collect();
    reversed.reverse();
    assert_eq!(forward, reversed);
    let threaded: Vec<u64> = std::thread::scope(|s| {
        let hs: Vec<_> = bricks
            .chunks(bricks.len().div_ceil(4))
            .map(|c| s.spawn(move || c.iter().map(|b| hash_of(SEED, *b)).collect::<Vec<_>>()))
            .collect();
        hs.into_iter().flat_map(|h| h.join().unwrap()).collect()
    });
    assert_eq!(forward, threaded);
}

#[test]
fn caves_and_ores_only_change_stone_below_the_surface_and_the_surface_is_v2s() {
    let (mut stone, mut coal, mut iron, mut air_under, mut under) = (0u64, 0u64, 0u64, 0u64, 0u64);
    let mut open_columns = 0;
    for x in (-300..300).step_by(3) {
        for z in (-300..300).step_by(3) {
            let h = surface_height(SEED, v(3), x, z).unwrap();
            assert_eq!(h, surface_height(SEED, v(2), x, z).unwrap(), "same surface");
            if voxel_at(SEED, v(3), x, h, z).unwrap() == 0 {
                open_columns += 1;
            }
            assert_eq!(
                voxel_at(SEED, v(3), x, h + 1, z).unwrap(),
                0,
                "air stays air"
            );
            for y in (h - 60)..=h {
                let (a, b) = (
                    voxel_at(SEED, v(2), x, y, z).unwrap(),
                    voxel_at(SEED, v(3), x, y, z).unwrap(),
                );
                let depth = h - y;
                if a != b {
                    // The only differences: stone becomes air, coal or iron; at the surface layers
                    // only air (a mouth).
                    assert!(
                        b == 0 || (a == STONE && (b == COAL || b == IRON)),
                        "{a}->{b} at {x},{y},{z}"
                    );
                    if b == COAL || b == IRON {
                        assert!(
                            depth >= 5 && y >= ORE_FLOOR,
                            "ore too shallow or too deep at {x},{y},{z}"
                        );
                    }
                    if b == IRON {
                        assert!(depth >= 16, "iron only deep: {depth}");
                    }
                }
                if y < CAVE_FLOOR {
                    assert_eq!(a, b, "nothing below the cave floor changes except ore");
                }
                if depth >= 6 {
                    under += 1;
                    match b {
                        0 => air_under += 1,
                        STONE => stone += 1,
                        COAL => coal += 1,
                        IRON => iron += 1,
                        _ => {}
                    }
                }
            }
        }
    }
    assert!(
        air_under * 100 >= under * 2 && air_under * 100 <= under * 15,
        "cave share: {air_under} of {under}"
    );
    let rock = stone + coal + iron;
    assert!(
        coal * 100 >= rock && coal * 100 <= rock * 6,
        "coal share: {coal} of {rock}"
    );
    assert!(
        iron * 1000 >= rock * 3 && iron * 100 <= rock * 3,
        "iron share: {iron} of {rock}"
    );
    assert!(open_columns > 0, "some tunnels reach the open air");
}

#[test]
fn deep_bricks_stay_uniform_stone() {
    for (x, z) in [(0, 0), (5, -5), (-20, 17)] {
        let b = generate_brick(SEED, v(3), BrickCoord::new(x, -8, z)).unwrap();
        assert!(matches!(
            b.content,
            cube_worldgen::BrickContent::Uniform(STONE)
        ));
    }
}

/// The landmarks of seed 7 that the real-Godot smoke flies to: a shaft open to the sky at
/// (98, -7, -70) and a coal voxel at (0, -54, -23) seen from the air voxel at (0, -54, -24).
#[test]
fn the_pinned_landmarks_of_seed_7_exist() {
    let s = WorldSeed::new(7);
    let m = |x, y, z| voxel_at(s, v(3), x, y, z).unwrap();
    assert_eq!(surface_height(s, v(3), 98, -70).unwrap(), -7);
    for y in -10..=-7 {
        assert_eq!(m(98, y, -70), 0, "shaft at {y}");
    }
    assert_eq!(m(0, -54, -23), COAL);
    assert_eq!((m(0, -54, -24), m(0, -53, -24)), (0, 0));
}

#[test]
fn versions_one_and_two_are_untouched() {
    // The v2 pinned hashes live in worldgen_v2.rs and the v1 ones in determinism.rs; here the
    // versions must still differ from v3 where caves exist, and v2 stays solid where v3 carves.
    let (x, y, z) = (98, -9, -70);
    assert_ne!(voxel_at(WorldSeed::new(7), v(2), x, y, z).unwrap(), 0);
    assert_eq!(voxel_at(WorldSeed::new(7), v(3), x, y, z).unwrap(), 0);
}
