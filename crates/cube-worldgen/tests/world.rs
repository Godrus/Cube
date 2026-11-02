//! Behavioural tests of the world-generation spike (generator, streamer, world-state-v0).

use std::cell::Cell;
use std::rc::Rc;

use cube_core::{BrickCoord, VoxelPos};
use cube_worldgen::generator::{ChunkGenerator, TerrainGenerator};
use cube_worldgen::state_v0;
use cube_worldgen::{
    Chunk, ChunkCoord, ChunkState, GenError, StreamConfig, WorldError, WorldSeed, WorldStreamer,
    CHUNK_HEIGHT, CHUNK_SIZE, MAT_AIR, MAT_GRASS, MAT_PLACED, MAX_CHUNK_ABS,
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

fn cells_hash(w: &WorldStreamer, c: ChunkCoord) -> u64 {
    let mut h = 0xCBF2_9CE4_8422_2325u64;
    for (p, m) in w.chunk_cells(c).expect("ready chunk") {
        for v in [p.x as u64, p.y as u64, p.z as u64, m as u64] {
            h ^= v;
            h = h.wrapping_mul(0x0000_0100_0000_01B3);
        }
    }
    h
}

/// Highest non-air voxel of a generated chunk column.
fn column_top(chunk: &Chunk, lx: u8, lz: u8) -> i32 {
    (0..CHUNK_HEIGHT)
        .rev()
        .find(|&y| chunk.material_at(lx, y as u8, lz) != MAT_AIR)
        .expect("column has terrain")
}

#[test]
fn same_seed_and_coordinate_give_identical_chunk() {
    for c in [
        ChunkCoord::new(0, 0),
        ChunkCoord::new(3, -2),
        ChunkCoord::new(-9, 14),
    ] {
        let a = TerrainGenerator.generate(SEED, c).unwrap();
        let b = TerrainGenerator.generate(SEED, c).unwrap();
        assert_eq!(a, b);
        assert_eq!(a.content_hash(), b.content_hash());
        assert!(a.voxel_count() > 0);
    }
}

#[test]
fn different_seeds_give_different_terrain() {
    let c = ChunkCoord::new(2, 2);
    let a = TerrainGenerator.generate(WorldSeed(1), c).unwrap();
    let b = TerrainGenerator.generate(WorldSeed(2), c).unwrap();
    assert_ne!(a.content_hash(), b.content_hash());
    let differing = (0..16)
        .flat_map(|z| (0..16).map(move |x| (x, z)))
        .filter(|&(x, z)| column_top(&a, x, z) != column_top(&b, x, z))
        .count();
    assert!(differing > 64, "only {differing} of 256 columns differ");
}

#[test]
fn negative_coordinates_work_and_use_floor_division() {
    assert_eq!(ChunkCoord::from_world(-1, -16), ChunkCoord::new(-1, -1));
    assert_eq!(ChunkCoord::from_world(-17, 15), ChunkCoord::new(-2, 0));
    for c in [
        ChunkCoord::new(-1, -1),
        ChunkCoord::new(-5, -7),
        ChunkCoord::new(-MAX_CHUNK_ABS, MAX_CHUNK_ABS),
    ] {
        let ch = TerrainGenerator.generate(SEED, c).unwrap();
        for (lx, lz) in [(0u8, 0u8), (15, 15), (7, 3)] {
            let expect = TerrainGenerator::height_at(
                SEED,
                c.origin_x() + lx as i32,
                c.origin_z() + lz as i32,
            );
            assert_eq!(column_top(&ch, lx, lz), expect);
            assert_eq!(ch.material_at(lx, expect as u8, lz), MAT_GRASS);
        }
    }
    let mut w = streamer(SEED, 1);
    w.set_center_world(-1, -1);
    w.pump_all();
    assert!(w.active_chunks().contains(&ChunkCoord::new(-2, -2)));
    // Bricks of a negative chunk sit at negative brick coordinates.
    assert!(w
        .asset()
        .layer(w.layer())
        .unwrap()
        .storage()
        .get(BrickCoord::new(-1, 0, -1))
        .is_some());
}

#[test]
fn out_of_world_chunk_is_refused() {
    let r = TerrainGenerator.generate(SEED, ChunkCoord::new(MAX_CHUNK_ABS + 1, 0));
    assert!(matches!(r, Err(GenError::OutOfWorld(_))));
}

#[test]
fn repeated_updates_never_duplicate_generation() {
    let mut w = streamer(SEED, 1);
    let first = w.set_center_world(0, 0);
    assert_eq!(first.queued, 9);
    let again = w.set_center_world(0, 0);
    assert_eq!(
        again.queued, 0,
        "already queued chunks must not be queued twice"
    );
    assert_eq!(w.queue_len(), 9);
    w.pump_all();
    let bricks = w.asset().layer(w.layer()).unwrap().storage().len();
    let rev = w.asset().asset_token();
    let third = w.set_center_world(5, 5);
    assert_eq!(third.queued, 0);
    assert_eq!(w.pump(100).generated, 0);
    assert_eq!(w.asset().layer(w.layer()).unwrap().storage().len(), bricks);
    assert_eq!(
        w.asset().asset_token(),
        rev,
        "no re-generation means no new commit"
    );
    assert_eq!(w.active_chunks().len(), 9);
}

#[test]
fn neighbouring_chunks_agree_on_their_borders() {
    for seed in [WorldSeed(0), SEED, WorldSeed(u64::MAX)] {
        for (ax, az) in [(0, 0), (-1, -1), (-4, 3), (7, -9)] {
            let a = ChunkCoord::new(ax, az);
            let east = ChunkCoord::new(ax + 1, az);
            let south = ChunkCoord::new(ax, az + 1);
            let (ca, ce, cs) = (
                TerrainGenerator.generate(seed, a).unwrap(),
                TerrainGenerator.generate(seed, east).unwrap(),
                TerrainGenerator.generate(seed, south).unwrap(),
            );
            for i in 0..16u8 {
                // Each side reproduces the world height function exactly, so there is no seam by construction...
                let (ha, he) = (column_top(&ca, 15, i), column_top(&ce, 0, i));
                assert_eq!(
                    ha,
                    TerrainGenerator::height_at(seed, a.origin_x() + 15, a.origin_z() + i as i32)
                );
                assert_eq!(
                    he,
                    TerrainGenerator::height_at(seed, east.origin_x(), east.origin_z() + i as i32)
                );
                // ...and the step across the border is within the terrain's own slope.
                assert!((ha - he).abs() <= 3, "x border step {} at {a:?}", ha - he);
                let (ha, hs) = (column_top(&ca, i, 15), column_top(&cs, i, 0));
                assert!((ha - hs).abs() <= 3, "z border step {} at {a:?}", ha - hs);
            }
        }
    }
}

#[test]
fn terrain_layers_are_grass_dirt_stone() {
    let ch = TerrainGenerator
        .generate(SEED, ChunkCoord::new(0, 0))
        .unwrap();
    let h = column_top(&ch, 4, 4);
    assert_eq!(ch.material_at(4, h as u8, 4), MAT_GRASS);
    assert_eq!(ch.material_at(4, (h - 1) as u8, 4), cube_worldgen::MAT_DIRT);
    assert_eq!(ch.material_at(4, 0, 4), cube_worldgen::MAT_STONE);
    assert_eq!(ch.material_at(4, (h + 1) as u8, 4), MAT_AIR);
}

#[test]
fn unloading_removes_chunks_from_the_active_set_and_core() {
    let mut w = streamer(SEED, 1);
    w.set_center_world(0, 0);
    w.pump_all();
    assert_eq!(w.active_chunks().len(), 9);
    w.take_delta();
    // Move 4 chunks east: everything left of x = 2 is beyond radius + margin of 2 around x = 4.
    let s = w.set_center_world(4 * CHUNK_SIZE, 0);
    assert!(s.unloaded > 0);
    let d = w.take_delta();
    assert!(d.removed.contains(&ChunkCoord::new(0, 0)));
    assert_eq!(w.state(ChunkCoord::new(0, 0)), ChunkState::Unloaded);
    assert!(!w.active_chunks().contains(&ChunkCoord::new(0, 0)));
    assert!(w
        .asset()
        .layer(w.layer())
        .unwrap()
        .storage()
        .get(BrickCoord::new(0, 0, 0))
        .is_none());
    assert!(w.chunk_cells(ChunkCoord::new(0, 0)).is_none());
}

#[test]
fn reloading_restores_identical_content() {
    let mut w = streamer(SEED, 1);
    w.set_center_world(0, 0);
    w.pump_all();
    let c = ChunkCoord::new(0, 0);
    let before = cells_hash(&w, c);
    w.set_center_world(10 * CHUNK_SIZE, 0);
    w.pump_all();
    assert_eq!(w.state(c), ChunkState::Unloaded);
    w.set_center_world(0, 0);
    w.pump_all();
    assert_eq!(w.state(c), ChunkState::Ready);
    assert_eq!(cells_hash(&w, c), before);
}

#[test]
fn hysteresis_keeps_chunks_inside_the_unload_margin() {
    let mut w = streamer(SEED, 1);
    w.set_center_world(0, 0);
    w.pump_all();
    let s = w.set_center_world(CHUNK_SIZE, 0); // one chunk east: old west column is at distance 2 = radius + margin
    assert_eq!(s.unloaded, 0);
    w.pump_all();
    assert_eq!(w.active_chunks().len(), 12);
}

struct FailAt {
    target: ChunkCoord,
    failing: Rc<Cell<bool>>,
}

impl ChunkGenerator for FailAt {
    fn generate(&self, seed: WorldSeed, c: ChunkCoord) -> Result<Chunk, GenError> {
        if c == self.target && self.failing.get() {
            return Err(GenError::Injected("injected failure"));
        }
        TerrainGenerator.generate(seed, c)
    }
}

#[test]
fn a_failing_chunk_does_not_break_the_others() {
    let failing = Rc::new(Cell::new(true));
    let bad = ChunkCoord::new(1, 0);
    let mut w = WorldStreamer::new(
        SEED,
        Box::new(FailAt {
            target: bad,
            failing: failing.clone(),
        }),
        StreamConfig {
            load_radius: 1,
            ..StreamConfig::default()
        },
    );
    w.set_center_world(0, 0);
    let s = w.pump_all();
    assert_eq!((s.generated, s.failed), (8, 1));
    assert_eq!(w.state(bad), ChunkState::Failed);
    assert!(matches!(w.failure(bad), Some(GenError::Injected(_))));
    assert_eq!(w.active_chunks().len(), 8);
    assert!(w.chunk_cells(bad).is_none());
    // A moved centre does not silently retry a failed chunk that is still wanted.
    w.set_center_world(0, 0);
    assert_eq!(w.queue_len(), 0);
    // Retrying while still failing keeps the others intact; after the cause is gone it recovers.
    assert_eq!(w.retry_failed(), 1);
    w.pump_all();
    assert_eq!(w.state(bad), ChunkState::Failed);
    failing.set(false);
    assert_eq!(w.retry_failed(), 1);
    w.pump_all();
    assert_eq!(w.state(bad), ChunkState::Ready);
    assert_eq!(w.active_chunks().len(), 9);
}

#[test]
fn generation_does_not_depend_on_load_order() {
    let mut a = streamer(SEED, 2);
    a.set_center_world(0, 0);
    a.pump_all();

    // Same final area reached by walking in from far away, one chunk per pump, other direction.
    let mut b = streamer(SEED, 2);
    for step in (0..=8).rev() {
        b.set_center_world(-step * CHUNK_SIZE * 2, 0);
        b.pump(1);
    }
    b.set_center_world(0, 0);
    b.pump_all();

    let shared: Vec<ChunkCoord> = a
        .active_chunks()
        .into_iter()
        .filter(|c| b.active_chunks().contains(c))
        .collect();
    assert!(
        shared.len() >= 20,
        "expected the 5x5 area to be shared, got {}",
        shared.len()
    );
    for c in shared {
        assert_eq!(
            cells_hash(&a, c),
            cells_hash(&b, c),
            "chunk {c:?} differs by load order"
        );
    }
}

#[test]
fn queue_is_bounded_and_catches_up_on_later_updates() {
    let mut w = WorldStreamer::new(
        SEED,
        Box::new(TerrainGenerator),
        StreamConfig {
            load_radius: 2,
            unload_margin: 1,
            max_queue: 4,
        },
    );
    let s = w.set_center_world(0, 0);
    assert_eq!((s.queued, s.deferred), (4, 21));
    assert_eq!(w.queue_len(), 4);
    // Nearest chunks first: the centre is generated before the ring.
    w.pump(1);
    assert_eq!(w.state(ChunkCoord::new(0, 0)), ChunkState::Ready);
    for _ in 0..10 {
        w.set_center_world(0, 0);
        w.pump_all();
    }
    assert_eq!(w.active_chunks().len(), 25);
    assert_eq!(w.counts().queued, 0);
}

#[test]
fn state_machine_transitions_are_ordered() {
    let mut w = streamer(SEED, 0);
    w.enable_trace();
    w.set_center_world(0, 0);
    w.pump_all();
    w.set_center_world(100 * CHUNK_SIZE, 0);
    let c = ChunkCoord::new(0, 0);
    let seq: Vec<ChunkState> = w
        .trace()
        .iter()
        .filter(|(k, _)| *k == c)
        .map(|(_, s)| *s)
        .collect();
    assert_eq!(
        seq,
        vec![
            ChunkState::Queued,
            ChunkState::Generating,
            ChunkState::Ready,
            ChunkState::Unloaded
        ]
    );
    // A queued chunk that goes out of range before generation is dropped without ever generating.
    let mut w = streamer(SEED, 1);
    w.set_center_world(0, 0);
    let s = w.set_center_world(50 * CHUNK_SIZE, 0);
    assert_eq!(s.dropped_from_queue, 9);
    assert_eq!(w.pump(100).generated, 9); // the new area, not the old one
    assert_eq!(w.state(c), ChunkState::Unloaded);
}

#[test]
fn edits_survive_unload_and_are_applied_to_regenerated_chunks() {
    let mut w = streamer(SEED, 1);
    w.set_center_world(0, 0);
    w.pump_all();
    let p = VoxelPos::new(3, 25, 4); // in the air above the terrain, top brick
    assert_eq!(w.voxel(p), Some(MAT_AIR));
    assert_eq!(w.set_voxel(p, MAT_PLACED), Ok(true));
    assert_eq!(w.set_voxel(p, MAT_PLACED), Ok(false));
    assert_eq!(w.take_modified(), vec![ChunkCoord::new(0, 0)]);
    assert_eq!(w.voxel(p), Some(MAT_PLACED));
    let dug = VoxelPos::new(5, 0, 5);
    assert_eq!(w.set_voxel(dug, MAT_AIR), Ok(true));

    w.set_center_world(40 * CHUNK_SIZE, 0);
    w.pump_all();
    assert_eq!(w.voxel(p), None);
    assert_eq!(w.edit_count(), 2);
    w.set_center_world(0, 0);
    w.pump_all();
    assert_eq!(w.voxel(p), Some(MAT_PLACED));
    assert_eq!(w.voxel(dug), Some(MAT_AIR));
}

#[test]
fn editing_rules() {
    let mut w = streamer(SEED, 0);
    w.set_center_world(0, 0);
    assert_eq!(
        w.set_voxel(VoxelPos::new(0, 5, 0), 1),
        Err(WorldError::ChunkNotReady(ChunkCoord::new(0, 0)))
    );
    w.pump_all();
    assert_eq!(
        w.set_voxel(VoxelPos::new(0, CHUNK_HEIGHT, 0), 1),
        Err(WorldError::OutOfBounds)
    );
    assert_eq!(
        w.set_voxel(VoxelPos::new(20, 5, 0), 1),
        Err(WorldError::ChunkNotReady(ChunkCoord::new(1, 0)))
    );
}

#[test]
fn active_voxel_count_matches_cells() {
    let mut w = streamer(SEED, 1);
    w.set_center_world(0, 0);
    w.pump_all();
    let total: usize = w
        .active_chunks()
        .iter()
        .map(|c| w.chunk_cells(*c).unwrap().len())
        .sum();
    assert_eq!(w.active_voxels(), total);
    assert!(total > 9 * 256 * 6);
}

fn tmp(name: &str) -> std::path::PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("cube_wstate_{}_{}", std::process::id(), name));
    p
}

#[test]
fn world_state_v0_round_trip() {
    let mut w = streamer(WorldSeed(777), 1);
    w.set_center_world(-20, 35);
    w.pump_all();
    w.set_voxel(VoxelPos::new(-20, 28, 35), MAT_PLACED).unwrap();
    w.set_voxel(VoxelPos::new(-19, 3, 36), MAT_AIR).unwrap();
    let before: Vec<(ChunkCoord, u64)> = w
        .active_chunks()
        .iter()
        .map(|c| (*c, cells_hash(&w, *c)))
        .collect();
    let p = tmp("a.wst0");
    w.save_state(&p).unwrap();

    let (loaded, rep) = state_v0::load(&p, Box::new(TerrainGenerator)).unwrap();
    assert_eq!(rep.saved_active, 9);
    assert_eq!(rep.restored_active, 9);
    assert!(rep.missing.is_empty());
    assert_eq!(rep.edits_restored, 2);
    assert_eq!(loaded.seed(), WorldSeed(777));
    assert_eq!(
        loaded.asset().epoch(),
        w.asset().epoch() + 1,
        "loaded world starts a new epoch"
    );
    for (c, h) in before {
        assert_eq!(cells_hash(&loaded, c), h, "chunk {c:?}");
    }
    assert_eq!(loaded.voxel(VoxelPos::new(-20, 28, 35)), Some(MAT_PLACED));
    let _ = std::fs::remove_file(&p);
}

#[test]
fn world_state_v0_detects_corruption_and_wrong_version() {
    let mut w = streamer(SEED, 0);
    w.set_center_world(0, 0);
    w.pump_all();
    let bytes = state_v0::encode(&w);
    for i in [0usize, 9, 20, bytes.len() / 2, bytes.len() - 1] {
        let mut bad = bytes.clone();
        bad[i] ^= 0x5A;
        assert!(
            state_v0::decode(&bad, Box::new(TerrainGenerator)).is_err(),
            "flip at {i}"
        );
    }
    assert!(state_v0::decode(&bytes[..bytes.len() - 5], Box::new(TerrainGenerator)).is_err());
    assert!(state_v0::decode(&[], Box::new(TerrainGenerator)).is_err());
    // Version bump with a valid checksum must be refused, not misread.
    let mut v1 = bytes[..bytes.len() - 4].to_vec();
    v1[8] = 1;
    let crc = state_v0::crc32(&v1);
    v1.extend_from_slice(&crc.to_le_bytes());
    assert!(matches!(
        state_v0::decode(&v1, Box::new(TerrainGenerator)),
        Err(state_v0::StateError::UnsupportedVersion(1))
    ));
}

#[test]
fn world_state_v0_restores_chunks_kept_by_the_unload_margin() {
    let mut w = streamer(SEED, 1);
    w.set_center_world(0, 0);
    w.pump_all();
    w.set_center_world(CHUNK_SIZE, 0); // hysteresis keeps the old west column: 12 active chunks
    w.pump_all();
    assert_eq!(w.active_chunks().len(), 12);
    let before: Vec<(ChunkCoord, u64)> = w
        .active_chunks()
        .iter()
        .map(|c| (*c, cells_hash(&w, *c)))
        .collect();
    let bytes = state_v0::encode(&w);
    let (loaded, rep) = state_v0::decode(&bytes, Box::new(TerrainGenerator)).unwrap();
    assert_eq!((rep.saved_active, rep.restored_active), (12, 12));
    assert!(rep.missing.is_empty());
    for (c, h) in before {
        assert_eq!(cells_hash(&loaded, c), h);
    }
}
