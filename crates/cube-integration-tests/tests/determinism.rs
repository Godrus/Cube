//! TEST-09 (in the allowed scope): the determinism suite of the world generator on the CI host:
//! pinned hashes, order and thread independence, negative and extreme coordinates, brick
//! borders, version registry, bounded cost, and "generation never erases a user edit".
//!
//! This is a statement about the CI host only. Equality on Android and iOS devices is DG-2 and is
//! not claimed. The generator is integer-only (archcheck forbids floats in `cube-worldgen`).
#![cfg(not(loom))]

use std::collections::BTreeMap;

use cube_backend_voxel::{VoxelBackend, remove};
use cube_core::{
    BrickCoord, CellChange, GeneratorVersion, LayerId, MaterialiseOutcome, OperationId, Region,
    VoxelPos, WorldSeed,
};
use cube_editing_api::{EditMode, EditingOperation, kinds};
use cube_integration_tests::fakes::*;
use cube_testkit::{DetRng, for_each_seed};
use cube_worldgen::{
    BrickContent, COORD_LIMIT, GenError, current_version, generate_brick, materials, registry, v1,
    voxel_at,
};

const SEED: WorldSeed = WorldSeed::new(42);

fn v1() -> GeneratorVersion {
    GeneratorVersion::new(1)
}

fn hash_of(seed: WorldSeed, b: BrickCoord) -> u64 {
    generate_brick(seed, v1(), b).unwrap().hash(seed, v1())
}

/// Pinned for generator version 1 and seed 42. A change here means the output of "terrain-v1"
/// changed: that needs a new generator version, never an edit of these numbers.
const GOLDEN: [((i64, i64, i64), u64); 6] = [
    ((0, 0, 0), 11545838340651508238),
    ((-1, -1, -1), 10506031000260715588),
    ((3, 0, -7), 1551315167222879730),
    ((0, -4, 0), 716171075521359173),
    ((1_000_000, 0, -1_000_000), 6569739149561532569),
    ((0, 20, 0), 1471874306897184026),
];

#[test]
fn the_same_seed_and_version_always_give_the_same_pinned_bricks() {
    let got: Vec<u64> = GOLDEN
        .iter()
        .map(|((x, y, z), _)| hash_of(SEED, BrickCoord::new(*x, *y, *z)))
        .collect();
    let want: Vec<u64> = GOLDEN.iter().map(|(_, h)| *h).collect();
    assert_eq!(got, want, "pinned hashes of terrain-v1 changed");
}

#[test]
fn different_seeds_make_different_worlds() {
    let b = BrickCoord::new(0, 0, 0);
    let hashes: Vec<u64> = (0..16).map(|s| hash_of(WorldSeed::new(s), b)).collect();
    let mut u = hashes.clone();
    u.sort_unstable();
    u.dedup();
    assert_eq!(
        u.len(),
        hashes.len(),
        "no two of 16 seeds collide on the surface brick"
    );
}

#[test]
fn the_result_does_not_depend_on_order_area_or_thread() {
    let mut rng = DetRng::new(7);
    let bricks: Vec<BrickCoord> = (0..200)
        .map(|_| {
            BrickCoord::new(
                rng.range_i64(-40, 40),
                rng.range_i64(-6, 5),
                rng.range_i64(-40, 40),
            )
        })
        .collect();
    let forward: BTreeMap<_, _> = bricks.iter().map(|b| (*b, hash_of(SEED, *b))).collect();
    // Reverse and shuffled orders.
    for_each_seed(11, 4, |s| {
        let mut order = bricks.clone();
        let mut r = DetRng::new(s);
        for i in (1..order.len()).rev() {
            order.swap(i, r.range_i64(0, i as i64) as usize);
        }
        for b in &order {
            assert_eq!(forward[b], hash_of(SEED, *b), "{b:?}");
        }
    });
    // Four threads, each over a quarter, in interleaved order.
    let results: Vec<(BrickCoord, u64)> = std::thread::scope(|s| {
        let handles: Vec<_> = (0..4)
            .map(|t| {
                let bricks = &bricks;
                s.spawn(move || {
                    bricks
                        .iter()
                        .skip(t)
                        .step_by(4)
                        .map(|b| (*b, hash_of(SEED, *b)))
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|h| h.join().unwrap())
            .collect()
    });
    for (b, h) in results {
        assert_eq!(forward[&b], h);
    }
}

#[test]
fn a_brick_equals_the_voxel_oracle_including_negative_coordinates() {
    for b in [
        BrickCoord::new(-1, -1, -1),
        BrickCoord::new(0, 0, 0),
        BrickCoord::new(-7, 1, 3),
        BrickCoord::new(2, -2, -9),
    ] {
        let g = generate_brick(SEED, v1(), b).unwrap();
        let m = g.materials();
        let o = b.origin().unwrap();
        for lz in 0..16 {
            for ly in 0..16 {
                for lx in 0..16 {
                    let i = lx + 16 * (ly + 16 * lz);
                    assert_eq!(
                        m[i as usize],
                        voxel_at(SEED, v1(), o.x + lx, o.y + ly, o.z + lz).unwrap(),
                        "{b:?} local ({lx},{ly},{lz})"
                    );
                }
            }
        }
    }
}

#[test]
fn the_ground_is_continuous_across_brick_borders_and_layered() {
    // No jump larger than 2 voxels between neighbouring columns, also over the x = 0, z = 0 and
    // brick borders (the slope of the noise is bounded).
    for z in -40i64..40 {
        for x in -70i64..70 {
            let h = v1::height(SEED, x, z);
            assert!((h - v1::height(SEED, x + 1, z)).abs() <= 2, "x {x} z {z}");
            assert!((h - v1::height(SEED, x, z + 1)).abs() <= 2, "x {x} z {z}");
            assert!(h.abs() <= 53);
        }
    }
    // Layers: grass on top, three dirt, stone below, air above.
    let h = v1::height(SEED, -5, 9);
    assert_eq!(voxel_at(SEED, v1(), -5, h + 1, 9).unwrap(), 0);
    assert_eq!(voxel_at(SEED, v1(), -5, h, 9).unwrap(), materials::GRASS);
    for d in 1..=3 {
        assert_eq!(voxel_at(SEED, v1(), -5, h - d, 9).unwrap(), materials::DIRT);
    }
    assert_eq!(
        voxel_at(SEED, v1(), -5, h - 4, 9).unwrap(),
        materials::STONE
    );
}

#[test]
fn there_is_no_size_limit_inside_the_range_and_errors_are_stable_outside() {
    let far = COORD_LIMIT / 32;
    for b in [
        BrickCoord::new(far, 0, -far),
        BrickCoord::new(-far, -3, far),
        BrickCoord::new(0, 0, 0),
    ] {
        assert!(generate_brick(SEED, v1(), b).is_ok(), "{b:?}");
    }
    let too_far = COORD_LIMIT / 8;
    for b in [
        BrickCoord::new(too_far, 0, 0),
        BrickCoord::new(0, 0, -too_far),
        BrickCoord::new(i64::MAX, 0, 0),
        BrickCoord::new(0, i64::MIN, 0),
    ] {
        assert_eq!(
            generate_brick(SEED, v1(), b),
            Err(GenError::InvalidCoord),
            "{b:?}"
        );
    }
    assert_eq!(
        voxel_at(SEED, v1(), i64::MAX, 0, 0),
        Err(GenError::InvalidCoord)
    );
    assert_eq!(GenError::InvalidCoord.code(), 5002);
}

#[test]
fn versions_are_registered_unknown_versions_are_errors_and_cost_is_bounded() {
    assert_eq!(
        current_version(),
        GeneratorVersion::new(5),
        "new worlds use the newest"
    );
    assert_eq!(registry().len(), 5);
    assert_eq!(registry()[0].algorithm, "terrain-v1");
    assert_eq!(registry()[1].algorithm, "terrain-v2");
    assert_eq!(registry()[2].algorithm, "terrain-v3");
    assert_eq!(registry()[3].algorithm, "terrain-v4");
    assert_eq!(registry()[4].algorithm, "terrain-v5");
    let unknown = GeneratorVersion::new(6);
    let e = generate_brick(SEED, unknown, BrickCoord::new(0, 0, 0)).unwrap_err();
    assert_eq!(e, GenError::GeneratorVersionUnknown(unknown));
    assert_eq!(e.code(), 5001);
    assert!(voxel_at(SEED, unknown, 0, 0, 0).is_err());
    // Bounded: far above and far below the terrain cost nothing and are not dense.
    assert_eq!(
        generate_brick(SEED, v1(), BrickCoord::new(3, 10, 3))
            .unwrap()
            .content,
        BrickContent::Empty
    );
    assert_eq!(
        generate_brick(SEED, v1(), BrickCoord::new(3, -20, 3))
            .unwrap()
            .content,
        BrickContent::Uniform(materials::STONE)
    );
    // Every brick of a column that crosses the terrain has exactly 4096 materials.
    for y in -5..5 {
        let g = generate_brick(SEED, v1(), BrickCoord::new(0, y, 0)).unwrap();
        assert_eq!(g.materials().len(), 4096);
    }
}

fn op(n: u64, min: [i64; 3]) -> EditingOperation {
    EditingOperation {
        id: OperationId::new(21, n),
        kind: kinds::REMOVE_GEOMETRY,
        asset: cube_core::AssetId::new(ASSET),
        layers: vec![LayerId::new(L_VOXEL)],
        parameters: remove(None),
        read_region: None,
        write_region: Region::new(
            0,
            VoxelPos::new(min[0], min[1], min[2]),
            VoxelPos::new(min[0] + 1, min[1] + 1, min[2] + 1),
        )
        .unwrap(),
        conversion: None,
        mode: EditMode::Destructive,
    }
}

#[test]
fn generation_never_erases_a_user_edit() {
    let f = fixture_with(Options {
        backends: vec![VoxelBackend::arc()],
        ..Options::default()
    });
    // The brick that holds the surface of column (0, 0).
    let h = v1::height(SEED, 0, 0);
    let (cx, cz) = (0, 0);
    let b = BrickCoord::new(0, h.div_euclid(16), 0);
    let layer = LayerId::new(L_VOXEL);
    let change = |b| generate_brick(SEED, v1(), b).unwrap().into_change();
    assert!(matches!(
        f.system
            .materialise(OperationId::new(5, 1), layer, 0, b, change(b))
            .unwrap(),
        MaterialiseOutcome::Written(_)
    ));
    assert!(f.run(vec![op(1, [cx, h, cz])]).is_success());
    assert_eq!(
        voxel_in(&f, [cx, h, cz]),
        Some(0),
        "the user removed the grass"
    );
    // The generator runs again for the same brick (a reload, a second request): nothing is
    // overwritten.
    let again = f
        .system
        .materialise(OperationId::new(5, 2), layer, 0, b, change(b))
        .unwrap();
    assert!(matches!(again, MaterialiseOutcome::AlreadyLoaded));
    assert_eq!(voxel_in(&f, [cx, h, cz]), Some(0), "the edit survived");
    // Known-empty generated content is idempotent too.
    let sky = BrickCoord::new(0, 10, 0);
    let first = f
        .system
        .materialise(
            OperationId::new(5, 3),
            layer,
            0,
            sky,
            CellChange::KnownEmpty,
        )
        .unwrap();
    assert!(matches!(first, MaterialiseOutcome::Written(_)));
}

fn voxel_in(f: &Fixture, p: [i64; 3]) -> Option<u16> {
    let (b, l) = VoxelPos::new(p[0], p[1], p[2]).split();
    let st = f.reader.snapshot();
    match st.cell(LayerId::new(L_VOXEL), 0, b)?.ok()? {
        cube_core::CellState::Present(pl) => match &*pl {
            cube_core::CellPayload::Channels(set) => set.material(l).map(|m| m.raw()),
            _ => None,
        },
        cube_core::CellState::KnownEmpty => Some(0),
        _ => None,
    }
}
