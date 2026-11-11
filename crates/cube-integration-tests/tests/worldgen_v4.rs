//! WGN2-03: "terrain-v4" (v3 plus trees). Pinned hashes, brick equals the voxel oracle (also for
//! trees that cross brick borders), order and thread independence, terrain untouched, tree
//! density per biome, spacing, shape, no tree on a cave mouth, and the pinned forest tree of
//! seed 7 used by the Godot smoke. CI host only (DG-2).
#![cfg(not(loom))]

use std::collections::BTreeMap;

use cube_core::{BrickCoord, GeneratorVersion, WorldSeed};
use cube_worldgen::materials::{LEAVES, WOOD};
use cube_worldgen::v2::Biome;
use cube_worldgen::v4::{Tree, tree_of_cell};
use cube_worldgen::{biome_at, generate_brick, surface_height, voxel_at};

const SEED: WorldSeed = WorldSeed::new(42);

fn v(n: u32) -> GeneratorVersion {
    GeneratorVersion::new(n)
}

fn hash_of(seed: WorldSeed, b: BrickCoord) -> u64 {
    generate_brick(seed, v(4), b).unwrap().hash(seed, v(4))
}

/// Pinned for generator version 4 and seed 42. A change here means the output of "terrain-v4"
/// changed: that needs a new generator version, never an edit of these numbers.
const GOLDEN: [((i64, i64, i64), u64); 9] = [
    ((0, 0, 0), 16930795638696635195),
    ((-1, -1, -1), 11648949329969250139),
    ((3, -2, -7), 11769960036584283894),
    ((0, -4, 0), 10463037234213491787),
    ((1_000_000, 0, -1_000_000), 11026281523361703316),
    ((0, 20, 0), 8558015598741407791),
    ((6, -1, -5), 2510578200569320872),
    ((-9, -1, 8), 433028431782765666),
    ((-5, -1, -1), 5521418572519778759),
];

#[test]
fn the_same_seed_and_version_always_give_the_same_pinned_bricks() {
    for ((x, y, z), want) in GOLDEN {
        assert_eq!(
            hash_of(SEED, BrickCoord::new(x, y, z)),
            want,
            "pinned hash of terrain-v4 changed at {x},{y},{z}"
        );
    }
}

fn trees(seed: u64, range: i64) -> Vec<Tree> {
    let mut out = Vec::new();
    for cx in -range..range {
        for cz in -range..range {
            out.extend(tree_of_cell(seed, cx, cz));
        }
    }
    out
}

#[test]
fn bricks_equal_the_voxel_function_also_where_trees_cross_brick_borders() {
    let seed = WorldSeed::new(7);
    let mut checked = 0;
    for t in trees(7, 30).iter().take(400) {
        // Crown reaches 2 columns around the trunk: take every brick that touches it.
        let (b0, b1) = (
            BrickCoord::new(
                (t.x - 2).div_euclid(16),
                (t.ground + 1).div_euclid(16),
                (t.z - 2).div_euclid(16),
            ),
            BrickCoord::new(
                (t.x + 2).div_euclid(16),
                (t.ground + t.trunk + 2).div_euclid(16),
                (t.z + 2).div_euclid(16),
            ),
        );
        // Only bricks that really straddle a border are interesting; check them all, cheaply.
        if b0 == b1 {
            continue;
        }
        for bx in b0.x..=b1.x {
            for by in b0.y..=b1.y {
                for bz in b0.z..=b1.z {
                    let b = BrickCoord::new(bx, by, bz);
                    let o = b.origin().unwrap();
                    let g = generate_brick(seed, v(4), b).unwrap().materials();
                    for (i, m) in g.iter().enumerate() {
                        let (lx, ly, lz) =
                            ((i % 16) as i64, ((i / 16) % 16) as i64, (i / 256) as i64);
                        assert_eq!(
                            *m,
                            voxel_at(seed, v(4), o.x + lx, o.y + ly, o.z + lz).unwrap(),
                            "brick {b:?} voxel {lx},{ly},{lz} (tree {t:?})"
                        );
                    }
                    checked += 1;
                }
            }
        }
        if checked > 60 {
            break;
        }
    }
    assert!(
        checked > 20,
        "enough border-crossing trees were checked: {checked}"
    );
}

#[test]
fn generation_does_not_depend_on_order_or_threads() {
    let bricks: Vec<BrickCoord> = (-4..4)
        .flat_map(|x| (-2..1).flat_map(move |y| (-4..4).map(move |z| BrickCoord::new(x, y, z))))
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
fn terrain_is_exactly_v3_and_trees_only_fill_air_above_the_ground() {
    let mut tree_voxels = 0;
    for x in (-200..200).step_by(2) {
        for z in (-200..200).step_by(2) {
            let h = surface_height(SEED, v(4), x, z).unwrap();
            for y in (h - 12)..=(h + 12) {
                let (a, b) = (
                    voxel_at(SEED, v(3), x, y, z).unwrap(),
                    voxel_at(SEED, v(4), x, y, z).unwrap(),
                );
                if a != b {
                    assert_eq!(a, 0, "terrain is never replaced: {a}->{b} at {x},{y},{z}");
                    assert!(b == WOOD || b == LEAVES, "only wood and leaves are added");
                    tree_voxels += 1;
                }
            }
        }
    }
    assert!(tree_voxels > 100, "there are trees: {tree_voxels}");
}

#[test]
fn trees_have_a_density_per_biome_a_spacing_a_shape_and_stand_on_solid_ground() {
    let seed = 42;
    let (mut cells, mut counts): ([u32; 4], [u32; 4]) = ([0; 4], [0; 4]);
    let all = trees(seed, 128);
    for cx in -128..128 {
        for cz in -128..128 {
            let b = biome_at(SEED, v(4), cx * 6 + 3, cz * 6 + 3).unwrap();
            cells[b as usize] += 1;
        }
    }
    let mut by_cell: BTreeMap<(i64, i64), Tree> = BTreeMap::new();
    for t in &all {
        let b = biome_at(SEED, v(4), t.x, t.z).unwrap();
        counts[b as usize] += 1;
        by_cell.insert((t.x.div_euclid(6), t.z.div_euclid(6)), *t);
        assert!(matches!(t.trunk, 4..=6));
        // Stands on solid ground, the trunk is wood, the crown has leaves, the top is open.
        assert_ne!(
            voxel_at(SEED, v(4), t.x, t.ground, t.z).unwrap(),
            0,
            "ground under {t:?}"
        );
        for dy in 1..=t.trunk {
            assert_eq!(voxel_at(SEED, v(4), t.x, t.ground + dy, t.z).unwrap(), WOOD);
        }
        assert_eq!(
            voxel_at(SEED, v(4), t.x, t.ground + t.trunk + 2, t.z).unwrap(),
            LEAVES
        );
        assert_eq!(
            voxel_at(SEED, v(4), t.x, t.ground + t.trunk + 3, t.z).unwrap(),
            0
        );
    }
    let pct = |i: usize| f64::from(counts[i]) * 100.0 / f64::from(cells[i].max(1));
    assert_eq!(counts[Biome::Desert as usize], 0, "no tree in the desert");
    assert!(
        (40.0..=60.0).contains(&pct(Biome::Forest as usize)),
        "forest {:.1}%",
        pct(Biome::Forest as usize)
    );
    assert!(
        (3.0..=11.0).contains(&pct(Biome::Plains as usize)),
        "plains {:.1}%",
        pct(Biome::Plains as usize)
    );
    assert!(
        (5.0..=16.0).contains(&pct(Biome::Tundra as usize)),
        "tundra {:.1}%",
        pct(Biome::Tundra as usize)
    );
    // Spacing: two trunks are at least 3 columns apart (in x or z).
    for ((cx, cz), t) in &by_cell {
        if let Some(n) = by_cell.get(&(cx + 1, *cz)) {
            assert!(n.x - t.x >= 3);
        }
        if let Some(n) = by_cell.get(&(*cx, cz + 1)) {
            assert!(n.z - t.z >= 3);
        }
    }
}

/// The forest tree of seed 7 that the real-Godot smoke looks at: trunk column (-76, -11), the
/// ground at -10, five voxels of wood.
#[test]
fn the_pinned_forest_tree_of_seed_7_exists() {
    let s = WorldSeed::new(7);
    assert_eq!(biome_at(s, v(4), -76, -11).unwrap(), Biome::Forest);
    assert_eq!(surface_height(s, v(4), -76, -11).unwrap(), -10);
    for y in -9..=-5 {
        assert_eq!(
            voxel_at(s, v(4), -76, y, -11).unwrap(),
            WOOD,
            "trunk at {y}"
        );
    }
    assert_ne!(voxel_at(s, v(4), -76, -10, -11).unwrap(), 0);
    assert_eq!(voxel_at(s, v(4), -76, -3, -11).unwrap(), LEAVES);
}
