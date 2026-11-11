//! WGN2-01: "terrain-v2" (biomes). Pinned hashes, brick equals the voxel oracle, order and
//! thread independence, the four biomes occur in sensible shares inside the finite demo world,
//! borders have no cliffs, old worlds (version 1) are untouched, and the version is chosen by the
//! world record, never silently. CI host only (cross-device equality is DG-2 and is not claimed).
#![cfg(not(loom))]

use std::collections::BTreeSet;

use cube_core::{BrickCoord, GeneratorVersion, WorldSeed};
use cube_worldgen::v2::Biome;
use cube_worldgen::{GenError, biome_at, generate_brick, surface_height, v1, voxel_at};

const SEED: WorldSeed = WorldSeed::new(42);

fn v2() -> GeneratorVersion {
    GeneratorVersion::new(2)
}

fn hash_of(seed: WorldSeed, b: BrickCoord) -> u64 {
    generate_brick(seed, v2(), b).unwrap().hash(seed, v2())
}

/// Pinned for generator version 2 and seed 42. A change here means the output of "terrain-v2"
/// changed: that needs a new generator version, never an edit of these numbers.
const GOLDEN: [((i64, i64, i64), u64); 6] = [
    ((0, 0, 0), 13340824106666550557),
    ((-1, -1, -1), 17099397580470469661),
    ((3, 0, -7), 13109602164261723168),
    ((0, -4, 0), 6805442890240415446),
    ((1_000_000, 0, -1_000_000), 15893929880290415242),
    ((0, 20, 0), 11684020089214205961),
];

#[test]
fn the_same_seed_and_version_always_give_the_same_pinned_bricks() {
    for ((x, y, z), want) in GOLDEN {
        assert_eq!(
            hash_of(SEED, BrickCoord::new(x, y, z)),
            want,
            "pinned hash of terrain-v2 changed at {x},{y},{z}"
        );
    }
}

#[test]
fn a_brick_equals_the_voxel_function_across_borders_and_negative_space() {
    for b in [
        BrickCoord::new(0, 0, 0),
        BrickCoord::new(-1, -2, -1),
        BrickCoord::new(-7, 3, 5),
        BrickCoord::new(12, -4, -9),
    ] {
        let g = generate_brick(SEED, v2(), b).unwrap();
        let mats = g.materials();
        let o = b.origin().unwrap();
        for (i, m) in mats.iter().enumerate() {
            let (lx, ly, lz) = ((i % 16) as i64, ((i / 16) % 16) as i64, (i / 256) as i64);
            assert_eq!(
                *m,
                voxel_at(SEED, v2(), o.x + lx, o.y + ly, o.z + lz).unwrap(),
                "brick {b:?} voxel {lx},{ly},{lz}"
            );
        }
    }
}

#[test]
fn generation_does_not_depend_on_order_or_threads() {
    let bricks: Vec<BrickCoord> = (-3..3)
        .flat_map(|x| (-2..2).flat_map(move |y| (-3..3).map(move |z| BrickCoord::new(x, y, z))))
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

struct Survey {
    seen: BTreeSet<Biome>,
    counts: [u32; 4],
    lo: i64,
    hi: i64,
    maxslope: i64,
    borders: u32,
    cliffs: u32,
}

/// Biome and height of every 16th column of the finite demo world (+-48 bricks).
fn survey(seed: WorldSeed) -> Survey {
    let mut s = Survey {
        seen: BTreeSet::new(),
        counts: [0; 4],
        lo: i64::MAX,
        hi: i64::MIN,
        maxslope: 0,
        borders: 0,
        cliffs: 0,
    };
    for ix in -48..48 {
        for iz in -48..48 {
            let (x, z) = (ix * 16, iz * 16);
            let b = biome_at(seed, v2(), x, z).unwrap();
            s.seen.insert(b);
            s.counts[b as usize] += 1;
            let h = surface_height(seed, v2(), x, z).unwrap();
            let hx = surface_height(seed, v2(), x + 1, z).unwrap();
            s.lo = s.lo.min(h);
            s.hi = s.hi.max(h);
            s.maxslope = s.maxslope.max((h - hx).abs());
            if biome_at(seed, v2(), x + 1, z).unwrap() != b {
                s.borders += 1;
                if (h - hx).abs() > 2 {
                    s.cliffs += 1;
                }
            }
        }
    }
    s
}

#[test]
fn all_four_biomes_occur_in_the_demo_world_in_sensible_shares_without_cliffs() {
    let s = survey(SEED);
    assert_eq!(
        s.seen.len(),
        4,
        "plains, forest, desert, tundra: {:?}",
        s.counts
    );
    let total: u32 = s.counts.iter().sum();
    for (i, c) in s.counts.iter().enumerate() {
        assert!(
            *c * 100 >= total * 8,
            "biome {i} covers under 8%: {:?}",
            s.counts
        );
    }
    assert!((-101..=100).contains(&s.lo) && (-101..=100).contains(&s.hi));
    assert!(
        s.maxslope <= 3,
        "steepest step between neighbours: {}",
        s.maxslope
    );
    assert!(s.borders > 20, "borders exist: {}", s.borders);
    assert_eq!(s.cliffs, 0, "the land does not jump at a biome border");
    for seed in [1, 7, 99, 12345] {
        let o = survey(WorldSeed::new(seed));
        assert!(o.seen.len() >= 3, "seed {seed}: {:?}", o.seen);
    }
}

#[test]
fn surface_materials_follow_the_biome() {
    use cube_worldgen::materials::{DIRT, GRASS, MOSS, SAND, SNOW, STONE};
    let mut checked = BTreeSet::new();
    for ix in -48..48 {
        for iz in -48..48 {
            let (x, z) = (ix * 16 + 3, iz * 16 + 5);
            let b = biome_at(SEED, v2(), x, z).unwrap();
            let h = surface_height(SEED, v2(), x, z).unwrap();
            let top = voxel_at(SEED, v2(), x, h, z).unwrap();
            let above = voxel_at(SEED, v2(), x, h + 1, z).unwrap();
            let deep = voxel_at(SEED, v2(), x, h - 8, z).unwrap();
            let want = match b {
                Biome::Desert => SAND,
                Biome::Tundra => SNOW,
                Biome::Forest => MOSS,
                Biome::Plains => GRASS,
            };
            assert_eq!((top, above, deep), (want, 0, STONE), "{b:?} at {x},{z}");
            if b != Biome::Desert {
                assert_eq!(voxel_at(SEED, v2(), x, h - 2, z).unwrap(), DIRT);
            }
            checked.insert(b);
        }
    }
    assert_eq!(checked.len(), 4);
}

#[test]
fn version_one_worlds_are_untouched_and_the_version_is_explicit() {
    for (x, z) in [(0, 0), (-5, 9), (300, -200)] {
        assert_eq!(
            surface_height(SEED, GeneratorVersion::new(1), x, z).unwrap(),
            v1::height(SEED, x, z)
        );
        assert_eq!(
            biome_at(SEED, GeneratorVersion::new(1), x, z).unwrap(),
            Biome::Plains
        );
    }
    // The same brick differs between the versions (a v1 world must never be read as v2).
    let b = BrickCoord::new(40, -2, 40);
    let one = GeneratorVersion::new(1);
    let h1 = generate_brick(SEED, one, b).unwrap().hash(SEED, one);
    assert_ne!(h1, hash_of(SEED, b));
    assert_eq!(
        generate_brick(SEED, GeneratorVersion::new(5), b).unwrap_err(),
        GenError::GeneratorVersionUnknown(GeneratorVersion::new(5))
    );
}
