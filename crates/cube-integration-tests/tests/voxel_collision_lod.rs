//! VOX-05: voxel collision boxes and the one-step LOD, against a brute-force voxel model, with
//! real edits through Orchestration and real Processing (stale rejection, determinism).
#![cfg(not(loom))]

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use cube_backend_voxel::{
    COLLISION_KIND, CollisionBoxes, CollisionProvider, LOD_EDGE, LOD_KIND, LodBrick, LodProvider,
    VoxelBackend, coarsen, merge_boxes, set_material,
};
use cube_core::{BrickCoord, CellChange, CellKey, LayerId, OperationId, Region, VoxelPos};
use cube_editing_api::{EditMode, EditingOperation, EditingResult, kinds};
use cube_integration_tests::fakes::*;
use cube_processing::{
    DerivedKind, DerivedTarget, Executor, ExecutorConfig, ManualClock, Priority, PriorityClass,
    ProviderRegistry, TargetStatus, TaskOutcome,
};
use cube_testkit::{DetRng, for_each_seed};

fn fx() -> Fixture {
    let f = fixture_with(Options {
        backends: vec![VoxelBackend::arc()],
        ..Options::default()
    });
    for x in -1..=1 {
        for y in -1..=1 {
            for z in -1..=1 {
                f.system
                    .materialise(
                        OperationId::new(
                            6,
                            (x + 1) as u64 * 9 + (y + 1) as u64 * 3 + (z + 1) as u64,
                        ),
                        LayerId::new(L_VOXEL),
                        0,
                        bc(x, y, z),
                        CellChange::KnownEmpty,
                    )
                    .unwrap();
            }
        }
    }
    f
}

fn executor(f: &Fixture) -> Executor {
    let mut r = ProviderRegistry::new();
    r.register(Arc::new(CollisionProvider)).unwrap();
    r.register(Arc::new(LodProvider)).unwrap();
    let e = Executor::new(
        f.reader.clone(),
        r.freeze().unwrap(),
        ExecutorConfig::default(),
        Arc::new(ManualClock::new(0)),
    );
    f.sink.forward_to(Arc::new(e.clone()));
    e
}

fn set_box(f: &Fixture, n: u64, lo: [i64; 3], hi: [i64; 3], m: u16) -> EditingResult {
    f.run(vec![EditingOperation {
        id: OperationId::new(31, n),
        kind: kinds::SET_MATERIAL,
        asset: cube_core::AssetId::new(ASSET),
        layers: vec![LayerId::new(L_VOXEL)],
        parameters: set_material(m, None),
        read_region: None,
        write_region: Region::new(
            0,
            VoxelPos::new(lo[0], lo[1], lo[2]),
            VoxelPos::new(hi[0], hi[1], hi[2]),
        )
        .unwrap(),
        conversion: None,
        mode: EditMode::Destructive,
    }])
}

fn t(kind: DerivedKind, b: BrickCoord) -> DerivedTarget {
    DerivedTarget {
        kind,
        key: CellKey::new(LayerId::new(L_VOXEL), 0, b).unwrap(),
    }
}

fn bricks() -> Vec<BrickCoord> {
    let mut v = Vec::new();
    for x in -1..=1 {
        for y in -1..=1 {
            for z in -1..=1 {
                v.push(bc(x, y, z));
            }
        }
    }
    v
}

fn build(e: &Executor, kind: DerivedKind) {
    for b in bricks() {
        e.submit(t(kind, b), Priority::new(PriorityClass::Visible))
            .unwrap();
    }
    e.run_until_idle();
}

fn boxes_of(e: &Executor, b: BrickCoord) -> CollisionBoxes {
    e.get(t(COLLISION_KIND, b))
        .unwrap()
        .expect("current")
        .downcast_ref::<CollisionBoxes>()
        .unwrap()
        .clone()
}

fn lod_of(e: &Executor, b: BrickCoord) -> LodBrick {
    e.get(t(LOD_KIND, b))
        .unwrap()
        .expect("current")
        .downcast_ref::<LodBrick>()
        .unwrap()
        .clone()
}

fn random_world(f: &Fixture, rng: &mut DetRng, edits: u64) -> HashMap<[i64; 3], u16> {
    let mut model = HashMap::new();
    for n in 0..edits {
        let lo = [
            rng.range_i64(-16, 28),
            rng.range_i64(-16, 28),
            rng.range_i64(-16, 28),
        ];
        let hi = [
            (lo[0] + 1 + rng.below(9) as i64).min(32),
            (lo[1] + 1 + rng.below(9) as i64).min(32),
            (lo[2] + 1 + rng.below(9) as i64).min(32),
        ];
        let m = rng.below(4) as u16;
        for x in lo[0]..hi[0] {
            for y in lo[1]..hi[1] {
                for z in lo[2]..hi[2] {
                    model.insert([x, y, z], m);
                }
            }
        }
        assert!(set_box(f, n + 1, lo, hi, m).is_success());
    }
    model
}

fn solid_cells(bx: &CollisionBoxes) -> Vec<[i64; 3]> {
    let mut out = Vec::new();
    for b in &bx.boxes {
        for x in b.min.x..b.max.x {
            for y in b.min.y..b.max.y {
                for z in b.min.z..b.max.z {
                    out.push([x, y, z]);
                }
            }
        }
    }
    out
}

#[test]
fn collision_boxes_are_disjoint_and_cover_exactly_the_solid_voxels() {
    for_each_seed(0xC011, 10, |seed| {
        let f = fx();
        let e = executor(&f);
        let model = random_world(&f, &mut DetRng::new(seed), 5);
        build(&e, COLLISION_KIND);
        for b in bricks() {
            let bx = boxes_of(&e, b);
            let cells = solid_cells(&bx);
            let set: HashSet<_> = cells.iter().copied().collect();
            assert_eq!(
                cells.len(),
                set.len(),
                "boxes overlap (seed {seed:#x}, brick {b:?})"
            );
            let want: HashSet<[i64; 3]> = model
                .iter()
                .filter(|(p, m)| **m != 0 && VoxelPos::new(p[0], p[1], p[2]).brick() == b)
                .map(|(p, _)| *p)
                .collect();
            assert_eq!(set, want, "seed {seed:#x} brick {b:?}");
            // Boxes stay inside their brick and are in ascending order.
            let o = b.origin().unwrap();
            assert!(bx.boxes.iter().all(|x| x.min.x >= o.x
                && x.max.x <= o.x + 16
                && x.min.y >= o.y
                && x.max.y <= o.y + 16
                && x.min.z >= o.z
                && x.max.z <= o.z + 16));
            assert!(bx.boxes.windows(2).all(|w| w[0] < w[1]));
        }
    });
}

#[test]
fn a_solid_slab_is_one_box_and_the_checkerboard_worst_case_is_bounded() {
    let f = fx();
    let e = executor(&f);
    assert!(set_box(&f, 1, [0, 0, 0], [16, 16, 16], 1).is_success());
    build(&e, COLLISION_KIND);
    assert_eq!(boxes_of(&e, bc(0, 0, 0)).boxes.len(), 1);
    assert_eq!(
        boxes_of(&e, bc(1, 0, 0)).boxes.len(),
        0,
        "a loaded empty brick has no boxes"
    );
    let checker: Vec<bool> = (0..4096)
        .map(|i| (i % 16 + (i / 16) % 16 + i / 256) % 2 == 0)
        .collect();
    let boxes = merge_boxes(&checker, VoxelPos::new(0, 0, 0));
    assert_eq!(boxes.len(), 2048);
    assert!(boxes.len() <= 4096, "documented budget");
}

#[test]
fn unknown_is_never_free_space_and_collision_follows_edits_without_serving_stale_data() {
    let f = fx();
    let e = executor(&f);
    let h = e
        .submit(
            t(COLLISION_KIND, bc(7, 7, 7)),
            Priority::new(PriorityClass::Visible),
        )
        .unwrap();
    e.run_until_idle();
    assert!(
        matches!(h.outcome(), Some(TaskOutcome::Failed(_))),
        "an unloaded brick has no collision answer"
    );
    assert!(e.get(t(COLLISION_KIND, bc(7, 7, 7))).unwrap().is_none());

    assert!(set_box(&f, 1, [2, 2, 2], [6, 6, 6], 1).is_success());
    build(&e, COLLISION_KIND);
    let target = t(COLLISION_KIND, bc(0, 0, 0));
    assert_eq!(e.query(target), Ok(TargetStatus::Valid));
    // Carve a hole: the derived data is stale at once and recomputes to the new truth.
    assert!(set_box(&f, 2, [3, 3, 3], [5, 5, 5], 0).is_success());
    assert_eq!(e.query(target), Ok(TargetStatus::Stale));
    assert!(e.get(target).unwrap().is_none());
    e.submit(target, Priority::new(PriorityClass::Visible))
        .unwrap();
    e.run_until_idle();
    let cells: HashSet<_> = solid_cells(&boxes_of(&e, bc(0, 0, 0)))
        .into_iter()
        .collect();
    assert_eq!(cells.len(), 4 * 4 * 4 - 2 * 2 * 2);
    assert!(!cells.contains(&[3, 3, 3]) && cells.contains(&[2, 2, 2]));
}

#[test]
fn lod_equals_the_brute_force_coarsening_never_loses_presence_and_leaves_core_untouched() {
    for_each_seed(0x10D, 8, |seed| {
        let f = fx();
        let e = executor(&f);
        let model = random_world(&f, &mut DetRng::new(seed), 5);
        let hash_before = f.reader.snapshot().export_view().content_hash();
        build(&e, LOD_KIND);
        for b in bricks() {
            let lod = lod_of(&e, b);
            assert_eq!(lod.cells.len(), LOD_EDGE.pow(3));
            let o = b.origin().unwrap();
            for z in 0..8usize {
                for y in 0..8usize {
                    for x in 0..8usize {
                        let mut counts: HashMap<u16, u8> = HashMap::new();
                        let mut any = false;
                        for (dx, dy, dz) in [
                            (0, 0, 0),
                            (1, 0, 0),
                            (0, 1, 0),
                            (1, 1, 0),
                            (0, 0, 1),
                            (1, 0, 1),
                            (0, 1, 1),
                            (1, 1, 1),
                        ] {
                            let p = [
                                o.x + 2 * x as i64 + dx,
                                o.y + 2 * y as i64 + dy,
                                o.z + 2 * z as i64 + dz,
                            ];
                            let m = model.get(&p).copied().unwrap_or(0);
                            if m != 0 {
                                any = true;
                                *counts.entry(m).or_default() += 1;
                            }
                        }
                        let mut best: Vec<_> = counts.into_iter().collect();
                        best.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
                        let want = best.first().map_or(0, |c| c.0);
                        let got = lod.cells[x + 8 * (y + 8 * z)];
                        assert_eq!(got, want, "seed {seed:#x} brick {b:?} cell {x},{y},{z}");
                        assert_eq!(got != 0, any, "presence must survive coarsening");
                    }
                }
            }
        }
        // Derived only: computing LODs changed no authoritative data.
        assert_eq!(
            f.reader.snapshot().export_view().content_hash(),
            hash_before
        );
    });
    let mut one = vec![0u16; 4096];
    one[5 + 16 * (7 + 16 * 9)] = 3;
    assert_eq!(
        coarsen(&one).iter().filter(|m| **m != 0).count(),
        1,
        "a single voxel survives"
    );
}

#[test]
fn collision_and_lod_are_deterministic_for_any_worker_count() {
    let run = |workers: usize| {
        let f = fx();
        let e = executor(&f);
        random_world(&f, &mut DetRng::new(0x5EED), 4);
        let w = (workers > 0).then(|| e.spawn_workers(workers));
        let hs: Vec<_> = bricks()
            .into_iter()
            .flat_map(|b| [t(COLLISION_KIND, b), t(LOD_KIND, b)])
            .map(|x| e.submit(x, Priority::new(PriorityClass::Visible)).unwrap())
            .collect();
        if w.is_none() {
            e.run_until_idle();
        }
        for h in &hs {
            assert!(matches!(h.wait(), TaskOutcome::Published { .. }));
        }
        let out: Vec<_> = bricks()
            .into_iter()
            .map(|b| (boxes_of(&e, b), lod_of(&e, b)))
            .collect();
        if let Some(w) = w {
            w.stop();
        }
        out
    };
    let a = run(0);
    assert_eq!(a, run(1));
    assert_eq!(a, run(4));
}
