//! VOX-02..VOX-04: the faces oracle, greedy merging and the dirty rule, with real edits through
//! Orchestration and real Processing (snapshots with apron, executor, stale rejection).
//! Timing at the end is a CI-host report, not a device measurement.
#![cfg(not(loom))]

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use cube_backend_voxel::{
    Dir, FACES_KIND, Face, FacesMesh, FacesProvider, GREEDY_KIND, GreedyMesh, GreedyProvider,
    VoxelBackend, expand, rebuild_set, set_material,
};
use cube_core::{BrickCoord, CellChange, CellKey, LayerId, OperationId, Region, VoxelPos};
use cube_editing_api::{EditMode, EditingOperation, EditingResult, kinds};
use cube_integration_tests::fakes::*;
use cube_processing::{
    DerivedTarget, Executor, ExecutorConfig, ManualClock, Priority, PriorityClass,
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
                            5,
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
    r.register(Arc::new(FacesProvider)).unwrap();
    r.register(Arc::new(GreedyProvider)).unwrap();
    let e = Executor::new(
        f.reader.clone(),
        r.freeze().unwrap(),
        ExecutorConfig::default(),
        Arc::new(ManualClock::new(0)),
    );
    f.sink.forward_to(Arc::new(e.clone()));
    e
}

fn region(lo: [i64; 3], hi: [i64; 3]) -> Region {
    Region::new(
        0,
        VoxelPos::new(lo[0], lo[1], lo[2]),
        VoxelPos::new(hi[0], hi[1], hi[2]),
    )
    .unwrap()
}

fn set_box(f: &Fixture, n: u64, lo: [i64; 3], hi: [i64; 3], m: u16) -> EditingResult {
    f.run(vec![EditingOperation {
        id: OperationId::new(11, n),
        kind: kinds::SET_MATERIAL,
        asset: cube_core::AssetId::new(ASSET),
        layers: vec![LayerId::new(L_VOXEL)],
        parameters: set_material(m, None),
        read_region: None,
        write_region: region(lo, hi),
        conversion: None,
        mode: EditMode::Destructive,
    }])
}

fn target(kind: cube_processing::DerivedKind, b: BrickCoord) -> DerivedTarget {
    DerivedTarget {
        kind,
        key: CellKey::new(LayerId::new(L_VOXEL), 0, b).unwrap(),
    }
}

fn prio() -> Priority {
    Priority::new(PriorityClass::Visible)
}

fn all_bricks() -> Vec<BrickCoord> {
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

fn faces_of(e: &Executor, b: BrickCoord) -> FacesMesh {
    e.get(target(FACES_KIND, b))
        .unwrap()
        .expect("faces are current")
        .downcast_ref::<FacesMesh>()
        .unwrap()
        .clone()
}

fn greedy_of(e: &Executor, b: BrickCoord) -> GreedyMesh {
    e.get(target(GREEDY_KIND, b))
        .unwrap()
        .expect("greedy is current")
        .downcast_ref::<GreedyMesh>()
        .unwrap()
        .clone()
}

fn build_all(e: &Executor, kind: cube_processing::DerivedKind) {
    for b in all_bricks() {
        e.submit(target(kind, b), prio()).unwrap();
    }
    e.run_until_idle();
}

/// Brute-force definition of the faces of one brick over a voxel model (outside = air).
fn oracle(model: &HashMap<[i64; 3], u16>, b: BrickCoord) -> Vec<Face> {
    let at = |p: [i64; 3]| model.get(&p).copied().unwrap_or(0);
    let mut out = Vec::new();
    for z in 0..16 {
        for y in 0..16 {
            for x in 0..16 {
                let p = [b.x * 16 + x, b.y * 16 + y, b.z * 16 + z];
                let m = at(p);
                if m == 0 {
                    continue;
                }
                for d in Dir::ALL {
                    let s = d.step();
                    if at([p[0] + s[0], p[1] + s[1], p[2] + s[2]]) == 0 {
                        out.push(Face {
                            voxel: VoxelPos::new(p[0], p[1], p[2]),
                            dir: d,
                            material: m,
                        });
                    }
                }
            }
        }
    }
    out
}

fn random_edit(rng: &mut DetRng, model: &mut HashMap<[i64; 3], u16>) -> ([i64; 3], [i64; 3], u16) {
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
    (lo, hi, m)
}

#[test]
fn faces_equal_the_brute_force_definition_across_brick_borders_and_negative_space() {
    for_each_seed(0xFACE, 10, |seed| {
        let f = fx();
        let e = executor(&f);
        let mut rng = DetRng::new(seed);
        let mut model = HashMap::new();
        for n in 0..5 {
            let (lo, hi, m) = random_edit(&mut rng, &mut model);
            assert!(set_box(&f, n + 1, lo, hi, m).is_success());
        }
        build_all(&e, FACES_KIND);
        for b in all_bricks() {
            let mesh = faces_of(&e, b);
            assert_eq!(mesh.faces, oracle(&model, b), "seed {seed:#x} brick {b:?}");
        }
    });
}

#[test]
fn a_border_between_two_solid_voxels_has_no_faces_and_a_border_to_air_has_exactly_one() {
    let f = fx();
    let e = executor(&f);
    // Solid voxels on both sides of the x = 16 border, and one more solid voxel next to air.
    assert!(set_box(&f, 1, [15, 0, 0], [17, 1, 1], 1).is_success());
    build_all(&e, FACES_KIND);
    let (a, b) = (faces_of(&e, bc(0, 0, 0)), faces_of(&e, bc(1, 0, 0)));
    assert!(
        !a.faces
            .iter()
            .any(|f| f.voxel.x == 15 && f.dir == Dir::PosX),
        "hidden across the border"
    );
    assert!(
        !b.faces
            .iter()
            .any(|f| f.voxel.x == 16 && f.dir == Dir::NegX)
    );
    assert_eq!(
        a.faces.len() + b.faces.len(),
        10,
        "two joined cubes show 5 faces each"
    );
    // The material of one side changes: the shared border stays hidden (solid next to solid).
    assert!(set_box(&f, 2, [16, 0, 0], [17, 1, 1], 2).is_success());
    e.run_until_idle();
    for br in [bc(0, 0, 0), bc(1, 0, 0)] {
        e.submit(target(FACES_KIND, br), prio()).unwrap();
    }
    e.run_until_idle();
    assert!(
        !faces_of(&e, bc(0, 0, 0))
            .faces
            .iter()
            .any(|f| f.voxel.x == 15 && f.dir == Dir::PosX)
    );
    // Removing one side exposes exactly the border face on the other.
    assert!(set_box(&f, 3, [16, 0, 0], [17, 1, 1], 0).is_success());
    e.submit(target(FACES_KIND, bc(0, 0, 0)), prio()).unwrap();
    e.run_until_idle();
    assert!(
        faces_of(&e, bc(0, 0, 0))
            .faces
            .iter()
            .any(|f| f.voxel.x == 15 && f.dir == Dir::PosX)
    );
}

#[test]
fn unloaded_neighbours_are_reported_and_an_unloaded_centre_is_a_failure() {
    let f = fx();
    let e = executor(&f);
    // A voxel on the border of the loaded area: its neighbour brick (2,0,0) is not loaded.
    assert!(set_box(&f, 1, [31, 0, 0], [32, 1, 1], 1).is_success());
    e.submit(target(FACES_KIND, bc(1, 0, 0)), prio()).unwrap();
    e.run_until_idle();
    let m = faces_of(&e, bc(1, 0, 0));
    assert!(m.unloaded_neighbours);
    assert!(
        m.faces.iter().any(|f| f.dir == Dir::PosX),
        "unloaded counts as air, flagged"
    );
    // The centre brick itself not loaded: a failure, never an empty mesh.
    let h = e.submit(target(FACES_KIND, bc(5, 5, 5)), prio()).unwrap();
    e.run_until_idle();
    assert!(
        matches!(h.outcome(), Some(TaskOutcome::Failed(_))),
        "{:?}",
        h.outcome()
    );
    assert!(e.get(target(FACES_KIND, bc(5, 5, 5))).unwrap().is_none());
    // An empty loaded brick is an empty mesh, which is different.
    e.submit(target(FACES_KIND, bc(0, 1, 0)), prio()).unwrap();
    e.run_until_idle();
    assert!(faces_of(&e, bc(0, 1, 0)).faces.is_empty());
}

#[test]
fn greedy_covers_exactly_the_faces_and_obeys_the_seam_rule() {
    for_each_seed(0x6EED, 8, |seed| {
        let f = fx();
        let e = executor(&f);
        let mut rng = DetRng::new(seed);
        let mut model = HashMap::new();
        for n in 0..4 {
            let (lo, hi, m) = random_edit(&mut rng, &mut model);
            assert!(set_box(&f, n + 1, lo, hi, m).is_success());
        }
        build_all(&e, GREEDY_KIND);
        for b in all_bricks() {
            let faces = faces_of(&e, b);
            let mesh = greedy_of(&e, b);
            let mut want = faces.faces.clone();
            want.sort();
            // Coverage: the same unit faces with the same materials, no more, no less.
            assert_eq!(expand(&mesh.quads), want, "seed {seed:#x} brick {b:?}");
            assert!(mesh.quads.len() <= faces.faces.len());
            let o = b.origin().unwrap();
            for q in &mesh.quads {
                // A quad stays inside its brick and one slice.
                let ax = |p: VoxelPos, a: usize| [p.x, p.y, p.z][a];
                for a in 0..3 {
                    assert!((ax(o, a)..ax(o, a) + 16).contains(&ax(q.origin, a)));
                }
                // Seam rule: a quad that touches the ring of its plane is a unit quad.
                let (u, v) = match q.dir.axis() {
                    0 => (1, 2),
                    1 => (0, 2),
                    _ => (0, 1),
                };
                let (lu, lv) = (ax(q.origin, u) - ax(o, u), ax(q.origin, v) - ax(o, v));
                let touches_ring = lu == 0
                    || lv == 0
                    || lu + i64::from(q.u_len) == 16
                    || lv + i64::from(q.v_len) == 16;
                if touches_ring {
                    assert_eq!((q.u_len, q.v_len), (1, 1), "{q:?}");
                }
            }
        }
    });
}

#[test]
fn greedy_merges_uniform_interiors_and_splits_at_material_changes() {
    let f = fx();
    let e = executor(&f);
    // A 16x16 slab at z = 5 in one material: the top has a 14x14 interior quad plus a unit ring.
    assert!(set_box(&f, 1, [0, 0, 5], [16, 16, 6], 1).is_success());
    build_all(&e, GREEDY_KIND);
    let g = greedy_of(&e, bc(0, 0, 0));
    let top: Vec<_> = g.quads.iter().filter(|q| q.dir == Dir::PosZ).collect();
    assert_eq!(
        top.len(),
        1 + 60,
        "one interior quad and 60 unit ring quads"
    );
    assert!(top.iter().any(|q| (q.u_len, q.v_len) == (14, 14)));
    // Two materials side by side inside the interior must not merge into one quad.
    assert!(set_box(&f, 2, [8, 0, 5], [16, 16, 6], 2).is_success());
    e.submit(target(GREEDY_KIND, bc(0, 0, 0)), prio()).unwrap();
    e.run_until_idle();
    let g = greedy_of(&e, bc(0, 0, 0));
    assert!(g.quads.iter().all(|q| q.material == 1 || q.material == 2));
    assert!(g.quads.iter().any(|q| q.material == 1 && q.u_len > 1));
    assert!(g.quads.iter().any(|q| q.material == 2 && q.u_len > 1));
}

#[test]
fn derived_meshes_are_deterministic_and_independent_of_the_worker_count() {
    let run = |workers: usize| {
        let f = fx();
        let e = executor(&f);
        let mut rng = DetRng::new(0xDE7);
        let mut model = HashMap::new();
        for n in 0..4 {
            let (lo, hi, m) = random_edit(&mut rng, &mut model);
            assert!(set_box(&f, n + 1, lo, hi, m).is_success());
        }
        let w = (workers > 0).then(|| e.spawn_workers(workers));
        let hs: Vec<_> = all_bricks()
            .into_iter()
            .map(|b| e.submit(target(GREEDY_KIND, b), prio()).unwrap())
            .collect();
        if w.is_none() {
            e.run_until_idle();
        }
        for h in &hs {
            assert!(matches!(h.wait(), TaskOutcome::Published { .. }));
        }
        let out: Vec<(FacesMesh, GreedyMesh)> = all_bricks()
            .into_iter()
            .map(|b| (faces_of(&e, b), greedy_of(&e, b)))
            .collect();
        if let Some(w) = w {
            w.stop();
        }
        out
    };
    let single = run(0);
    assert_eq!(single, run(1));
    assert_eq!(single, run(4));
}

#[test]
fn derived_data_goes_stale_after_a_core_commit_and_a_late_result_is_never_published() {
    let f = fx();
    let e = executor(&f);
    assert!(set_box(&f, 1, [3, 3, 3], [5, 5, 5], 1).is_success());
    build_all(&e, GREEDY_KIND);
    let t = target(GREEDY_KIND, bc(0, 0, 0));
    assert_eq!(e.query(t), Ok(TargetStatus::Valid));
    // An edit through Orchestration reaches the executor through the dirty sink.
    assert!(set_box(&f, 2, [3, 3, 3], [4, 4, 4], 0).is_success());
    assert_eq!(e.query(t), Ok(TargetStatus::Stale));
    assert!(
        e.get(t).unwrap().is_none(),
        "a stale mesh is never served as current"
    );
    e.submit(t, prio()).unwrap();
    e.run_until_idle();
    let mut model = HashMap::new();
    for x in 3..5 {
        for y in 3..5 {
            for z in 3..5 {
                model.insert([x, y, z], 1u16);
            }
        }
    }
    model.insert([3, 3, 3], 0);
    let mesh = greedy_of(&e, bc(0, 0, 0));
    let mut want = oracle(&model, bc(0, 0, 0));
    want.sort();
    assert_eq!(expand(&mesh.quads), want);
}

#[test]
fn incremental_rebuild_by_the_dirty_rule_equals_a_full_rebuild() {
    for_each_seed(0x1C0, 6, |seed| {
        let f = fx();
        let e = executor(&f);
        let mut rng = DetRng::new(seed);
        let mut model: HashMap<[i64; 3], u16> = HashMap::new();
        build_all(&e, FACES_KIND);
        let mut meshes: BTreeMap<BrickCoord, FacesMesh> = all_bricks()
            .into_iter()
            .map(|b| (b, faces_of(&e, b)))
            .collect();
        for n in 0..8 {
            let (lo, hi, m) = random_edit(&mut rng, &mut model);
            let EditingResult::Success(s) = set_box(&f, n + 1, lo, hi, m) else {
                panic!("edit failed")
            };
            // Recompute ONLY what the dirty rule names; everything else keeps its old mesh.
            for k in rebuild_set(&s.dirty) {
                if let Some(old) = meshes.get_mut(&k.brick()) {
                    e.submit(target(FACES_KIND, k.brick()), prio()).unwrap();
                    e.run_until_idle();
                    *old = faces_of(&e, k.brick());
                }
            }
            for b in all_bricks() {
                assert_eq!(
                    meshes[&b].faces,
                    oracle(&model, b),
                    "seed {seed:#x} step {n} brick {b:?}"
                );
            }
        }
    });
}

#[test]
fn ci_host_timing_report_for_faces_and_greedy() {
    // Not a gate and not a device measurement: a dense, noisy world on the CI host.
    let f = fx();
    let e = executor(&f);
    let mut rng = DetRng::new(7);
    for n in 0..60u64 {
        let lo = [
            rng.range_i64(-16, 28),
            rng.range_i64(-16, 28),
            rng.range_i64(-16, 28),
        ];
        let hi = [
            lo[0] + 1 + rng.below(3) as i64,
            lo[1] + 1 + rng.below(3) as i64,
            lo[2] + 1 + rng.below(3) as i64,
        ];
        assert!(set_box(&f, n + 1, lo, hi.map(|h| h.min(32)), 1 + (n % 3) as u16).is_success());
    }
    let t0 = std::time::Instant::now();
    build_all(&e, FACES_KIND);
    let faces_time = t0.elapsed();
    let t1 = std::time::Instant::now();
    build_all(&e, GREEDY_KIND);
    let greedy_time = t1.elapsed();
    let (nf, nq): (usize, usize) = all_bricks()
        .iter()
        .map(|b| (faces_of(&e, *b).faces.len(), greedy_of(&e, *b).quads.len()))
        .fold((0, 0), |a, x| (a.0 + x.0, a.1 + x.1));
    eprintln!(
        "CI-host report (debug build, 27 bricks): faces {faces_time:?} -> {nf} faces; greedy {greedy_time:?} -> {nq} quads"
    );
    assert!(nq <= nf && nf > 0);
}
