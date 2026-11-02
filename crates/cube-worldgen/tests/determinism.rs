//! Hash-based determinism tests of the world-generation spike. They compare full chunk content
//! (every voxel and material, seed, coordinate and generator version), not voxel counts.

use std::collections::HashSet;

use cube_worldgen::generator::{stable_hash_cells, ChunkGenerator, TerrainGenerator};
use cube_worldgen::{
    ChunkCoord, StreamConfig, WorldSeed, WorldStreamer, CHUNK_SIZE, GENERATOR_VERSION,
};

const SEED: WorldSeed = WorldSeed(0xC0FFEE);

fn streamer(seed: WorldSeed, radius: i32) -> WorldStreamer {
    WorldStreamer::new(
        seed,
        Box::new(TerrainGenerator),
        StreamConfig {
            load_radius: radius,
            ..StreamConfig::default()
        },
    )
}

fn hash(seed: WorldSeed, c: ChunkCoord) -> u64 {
    TerrainGenerator
        .generate(seed, c)
        .unwrap()
        .stable_hash(seed)
}

fn coords(r: i32) -> Vec<ChunkCoord> {
    let mut v = Vec::new();
    for z in -r..=r {
        for x in -r..=r {
            v.push(ChunkCoord::new(x, z));
        }
    }
    v
}

/// Pinned values: a failure means the terrain function, the chunk layout or the hash definition
/// changed. If that is intentional, bump `GENERATOR_VERSION` and re-pin (it invalidates saved worlds).
#[test]
fn pinned_hashes_detect_accidental_generator_changes() {
    assert_eq!(GENERATOR_VERSION, 1);
    let got = [
        hash(WorldSeed(12345), ChunkCoord::new(0, 0)),
        hash(WorldSeed(12345), ChunkCoord::new(-1, -1)),
        hash(WorldSeed(0xC0FFEE), ChunkCoord::new(3, -2)),
        hash(WorldSeed(0), ChunkCoord::new(1_000_000, -1_000_000)),
    ];
    let expected: [u64; 4] = [
        0xa549d2470c8cf22e,
        0x13a765e0117086a2,
        0x555a7b32536b065b,
        0x6cb17e76e3d9ede4,
    ];
    assert_eq!(got, expected, "{got:#x?}");
}

#[test]
fn same_seed_and_coordinate_give_the_same_hash() {
    for c in coords(6) {
        let first = hash(SEED, c);
        for _ in 0..3 {
            assert_eq!(first, hash(SEED, c), "chunk {c:?}");
        }
    }
}

#[test]
fn hash_does_not_depend_on_load_order() {
    let all = coords(3);
    // Reference: generator alone, in natural order.
    let reference: Vec<u64> = all.iter().map(|&c| hash(SEED, c)).collect();

    // Reverse order and a shuffled order through the generator.
    let mut shuffled: Vec<usize> = (0..all.len()).collect();
    shuffled.sort_by_key(|&i| cube_worldgen::noise::mix(i as u64 * 7919 + 13));
    for &i in shuffled.iter().rev() {
        assert_eq!(reference[i], hash(SEED, all[i]));
    }

    // Through streamers that reach the same area along different paths.
    let mut direct = streamer(SEED, 3);
    direct.set_center_world(8, 8);
    direct.pump_all();

    let mut walked = streamer(SEED, 3);
    for (cx, cz) in [(400, 400), (-300, 50), (8, 1200), (8, 8)] {
        walked.set_center_world(cx, cz);
        walked.pump_all();
    }

    let mut stepped = streamer(SEED, 3);
    for i in (0..=24).rev() {
        stepped.set_center_world(8 + i * CHUNK_SIZE, 8 - i * CHUNK_SIZE);
        stepped.pump(5);
    }
    stepped.set_center_world(8, 8);
    stepped.pump_all();

    for (i, &c) in all.iter().enumerate() {
        for (name, w) in [
            ("direct", &direct),
            ("walked", &walked),
            ("stepped", &stepped),
        ] {
            assert_eq!(
                w.chunk_stable_hash(c),
                Some(reference[i]),
                "{name} streamer, chunk {c:?}"
            );
        }
    }
}

#[test]
fn different_seeds_give_different_hashes() {
    let seeds: Vec<WorldSeed> = (0..64u64)
        .map(|i| WorldSeed(cube_worldgen::noise::mix(i)))
        .collect();
    for c in [ChunkCoord::new(0, 0), ChunkCoord::new(-5, 9)] {
        let hashes: HashSet<u64> = seeds.iter().map(|&s| hash(s, c)).collect();
        assert_eq!(hashes.len(), seeds.len(), "chunk {c:?}");
    }
    // Neighbouring seeds as well.
    assert_ne!(
        hash(WorldSeed(1), ChunkCoord::new(0, 0)),
        hash(WorldSeed(2), ChunkCoord::new(0, 0))
    );
}

#[test]
fn hash_covers_coordinate_voxels_and_generator_version() {
    // Different coordinates (all distinct over a 13x13 area).
    let hashes: HashSet<u64> = coords(6).iter().map(|&c| hash(SEED, c)).collect();
    assert_eq!(hashes.len(), 13 * 13);

    let c = ChunkCoord::new(2, 2);
    let chunk = TerrainGenerator.generate(SEED, c).unwrap();
    let cells = |chunk: &cube_worldgen::Chunk| -> Vec<(i32, i32, i32, u16)> {
        let mut v = Vec::new();
        for y in 0..32u8 {
            for z in 0..16u8 {
                for x in 0..16u8 {
                    let m = chunk.material_at(x, y, z);
                    if m != 0 {
                        v.push((
                            c.origin_x() + x as i32,
                            y as i32,
                            c.origin_z() + z as i32,
                            m,
                        ));
                    }
                }
            }
        }
        v
    };
    let base = stable_hash_cells(SEED, GENERATOR_VERSION, c, &mut cells(&chunk));
    assert_eq!(base, chunk.stable_hash(SEED));

    // One changed material changes the hash.
    let mut edited = cells(&chunk);
    edited[100].3 = 4;
    assert_ne!(
        base,
        stable_hash_cells(SEED, GENERATOR_VERSION, c, &mut edited)
    );
    // One missing voxel changes the hash.
    let mut missing = cells(&chunk);
    missing.pop();
    assert_ne!(
        base,
        stable_hash_cells(SEED, GENERATOR_VERSION, c, &mut missing)
    );
    // The generator version is part of the hash.
    assert_ne!(
        base,
        stable_hash_cells(SEED, GENERATOR_VERSION + 1, c, &mut cells(&chunk))
    );
    // The same cells under another coordinate or seed hash differently.
    assert_ne!(
        base,
        stable_hash_cells(
            SEED,
            GENERATOR_VERSION,
            ChunkCoord::new(2, 3),
            &mut cells(&chunk)
        )
    );
    assert_ne!(
        base,
        stable_hash_cells(WorldSeed(1), GENERATOR_VERSION, c, &mut cells(&chunk))
    );
    // Input order does not matter.
    let mut reversed = cells(&chunk);
    reversed.reverse();
    assert_eq!(
        base,
        stable_hash_cells(SEED, GENERATOR_VERSION, c, &mut reversed)
    );
}

/// A chunk generated alone and the same chunk inside a streamed area (stored in Geometry Core and read
/// back) are identical voxel for voxel, including the border columns shared with neighbours.
#[test]
fn neighbouring_chunks_are_identical_alone_and_inside_an_area() {
    for seed in [WorldSeed(1), SEED, WorldSeed(u64::MAX)] {
        for centre in [(8, 8), (-8, -8), (-200, 330)] {
            let mut area = streamer(seed, 2);
            area.set_center_world(centre.0, centre.1);
            area.pump_all();
            let active = area.active_chunks();
            assert!(active.len() >= 25);
            for c in active {
                let alone = hash(seed, c);
                assert_eq!(area.chunk_stable_hash(c), Some(alone), "chunk {c:?}");
                // Full voxel comparison, not only the hash.
                let generated = TerrainGenerator.generate(seed, c).unwrap();
                let stored = area.chunk_cells(c).unwrap();
                assert_eq!(stored.len(), generated.voxel_count());
                for (p, m) in stored {
                    let lx = (p.x - c.origin_x()) as u8;
                    let lz = (p.z - c.origin_z()) as u8;
                    assert_eq!(generated.material_at(lx, p.y as u8, lz), m);
                }
            }
        }
    }
}

/// Each chunk's border columns equal the columns the neighbour sees through the pure height function:
/// the voxel at a chunk edge is the same whichever chunk generates it.
#[test]
fn border_columns_do_not_depend_on_the_generating_chunk() {
    let a = ChunkCoord::new(-1, 0);
    let b = ChunkCoord::new(0, 0);
    let (ca, cb) = (
        TerrainGenerator.generate(SEED, a).unwrap(),
        TerrainGenerator.generate(SEED, b).unwrap(),
    );
    for lz in 0..16u8 {
        let wz = lz as i32;
        // Height at the last column of `a` and first of `b` matches the height function.
        for (chunk, lx, wx) in [(&ca, 15u8, -1), (&cb, 0u8, 0)] {
            let h = TerrainGenerator::height_at(SEED, wx, wz);
            for y in 0..32u8 {
                assert_eq!(
                    chunk.material_at(lx, y, lz),
                    TerrainGenerator::material_for(y as i32, h)
                );
            }
        }
    }
}
