//! VOX-03 closing criterion: the greedy mesh has **no T-junctions**, checked on the triangle mesh
//! (not by coverage alone), across brick borders, in negative space, for random worlds, and the
//! incremental rebuild equals a full rebuild. Results are CI-host statements.
#![cfg(not(loom))]

use std::collections::HashMap;
use std::sync::Arc;

use cube_backend_voxel::{
    FACES_KIND, FacesMesh, FacesProvider, GREEDY_KIND, GreedyMesh, GreedyProvider, Quad, TriMesh,
    VoxelBackend, mesh_t_junctions, set_material, t_junctions, triangulate,
};
use cube_core::{BrickCoord, CellChange, CellKey, LayerId, OperationId, Region, VoxelPos};
use cube_editing_api::{EditMode, EditingOperation, EditingResult, kinds};
use cube_integration_tests::fakes::*;
use cube_processing::{
    DerivedTarget, Executor, ExecutorConfig, ManualClock, Priority, PriorityClass, ProviderRegistry,
};
use cube_testkit::{DetRng, for_each_seed};

fn fx() -> Fixture {
    let f = fixture_with(Options {
        backends: vec![VoxelBackend::arc()],
        ..Options::default()
    });
    let mut n = 0;
    for x in -1..=1 {
        for y in -1..=1 {
            for z in -1..=1 {
                n += 1;
                f.system
                    .materialise(
                        OperationId::new(5, n),
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

fn set_box(f: &Fixture, n: u64, lo: [i64; 3], hi: [i64; 3], m: u16) -> EditingResult {
    f.run(vec![EditingOperation {
        id: OperationId::new(11, n),
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

fn target(kind: cube_processing::DerivedKind, b: BrickCoord) -> DerivedTarget {
    DerivedTarget {
        kind,
        key: CellKey::new(LayerId::new(L_VOXEL), 0, b).unwrap(),
    }
}

fn build_all(e: &Executor) {
    for b in bricks() {
        e.submit(
            target(GREEDY_KIND, b),
            Priority::new(PriorityClass::Visible),
        )
        .unwrap();
    }
    e.run_until_idle();
}

fn greedy(e: &Executor, b: BrickCoord) -> GreedyMesh {
    e.get(target(GREEDY_KIND, b))
        .unwrap()
        .expect("current")
        .downcast_ref::<GreedyMesh>()
        .unwrap()
        .clone()
}

fn faces(e: &Executor, b: BrickCoord) -> FacesMesh {
    e.get(target(FACES_KIND, b))
        .unwrap()
        .expect("current")
        .downcast_ref::<FacesMesh>()
        .unwrap()
        .clone()
}

/// The unit quads of a faces mesh.
fn unit_quads(m: &FacesMesh) -> Vec<Quad> {
    m.faces
        .iter()
        .map(|f| Quad {
            dir: f.dir,
            origin: f.voxel,
            u_len: 1,
            v_len: 1,
            material: f.material,
        })
        .collect()
}

/// Checks a whole set of bricks: no T-junction in the union of their triangle meshes, exact
/// coverage (area equals eight half-voxel units per unit face), and the same for each brick.
fn check_world(e: &Executor, what: &str) -> usize {
    let mut meshes: Vec<TriMesh> = Vec::new();
    let mut unit_faces = 0usize;
    for b in bricks() {
        let g = greedy(e, b);
        let f = faces(e, b);
        let t = triangulate(&g.quads);
        assert_eq!(mesh_t_junctions(&t), 0, "{what}: brick {b:?} alone");
        assert_eq!(
            t.double_area(),
            8 * f.faces.len() as i128,
            "{what}: area of {b:?}"
        );
        unit_faces += f.faces.len();
        meshes.push(t);
    }
    let all = TriMesh::union(&meshes.iter().collect::<Vec<_>>());
    assert_eq!(
        mesh_t_junctions(&all),
        0,
        "{what}: T-junction across brick borders"
    );
    assert_eq!(
        all.double_area(),
        8 * unit_faces as i128,
        "{what}: total coverage"
    );
    // The reference: the faces mesh triangulated, also free of T-junctions (the checker agrees
    // with the baseline renderer).
    let reference: Vec<TriMesh> = bricks()
        .into_iter()
        .map(|b| triangulate(&unit_quads(&faces(e, b))))
        .collect();
    let r = TriMesh::union(&reference.iter().collect::<Vec<_>>());
    assert_eq!(mesh_t_junctions(&r), 0, "{what}: faces baseline");
    assert_eq!(r.double_area(), all.double_area());
    meshes.iter().map(|m| m.triangles.len()).sum()
}

/// What a careless renderer would do: two triangles per quad, edge vertices ignored.
fn naive(quads: &[Quad]) -> TriMesh {
    // Triangulate each quad alone, so no foreign vertex is ever added to its polygon.
    let parts: Vec<TriMesh> = quads
        .iter()
        .map(|q| triangulate(std::slice::from_ref(q)))
        .collect();
    TriMesh::union(&parts.iter().collect::<Vec<_>>())
}

#[test]
fn the_minimal_fixture_has_t_junctions_as_quads_and_none_as_triangles() {
    // A 16 x 16 slab in one material: a 14 x 14 interior quad inside a ring of unit quads.
    let f = fx();
    let e = executor(&f);
    assert!(set_box(&f, 1, [0, 0, 5], [16, 16, 6], 1).is_success());
    build_all(&e);
    let g = greedy(&e, bc(0, 0, 0));
    assert!(
        g.quads.iter().any(|q| q.u_len == 14),
        "a big interior quad exists"
    );
    assert!(
        !t_junctions(&g.quads).is_empty(),
        "the quad list has foreign vertices on the edges of the big quad"
    );
    let tris = check_world(&e, "slab");
    let naive_mesh = naive(&g.quads);
    assert!(
        mesh_t_junctions(&naive_mesh) > 0,
        "the checker catches a mesh that ignores the edge vertices"
    );
    // The fan costs triangles but the merge still pays: fewer vertices than the faces mesh.
    let reference = triangulate(&unit_quads(&faces(&e, bc(0, 0, 0))));
    assert!(triangulate(&g.quads).vertices.len() <= reference.vertices.len());
    assert!(tris > 0);
}

#[test]
fn a_step_and_a_plateau_across_brick_borders_have_no_t_junctions() {
    let f = fx();
    let e = executor(&f);
    // Terraces that straddle x = 0 and z = 0 (negative coordinates on one side).
    assert!(set_box(&f, 1, [-12, 0, -12], [12, 4, 12], 1).is_success());
    assert!(set_box(&f, 2, [-6, 4, -6], [7, 7, 7], 2).is_success());
    assert!(set_box(&f, 3, [-2, 7, -2], [3, 9, 3], 3).is_success());
    build_all(&e);
    check_world(&e, "terraces");
}

#[test]
fn random_worlds_have_no_t_junctions_and_the_checker_is_not_vacuous() {
    let mut naive_failures = 0;
    for_each_seed(0x7A11, 12, |seed| {
        let f = fx();
        let e = executor(&f);
        let mut rng = DetRng::new(seed);
        for n in 0..6 {
            let lo = [
                rng.range_i64(-16, 24),
                rng.range_i64(-16, 24),
                rng.range_i64(-16, 24),
            ];
            let hi = [
                (lo[0] + 1 + rng.below(14) as i64).min(32),
                (lo[1] + 1 + rng.below(14) as i64).min(32),
                (lo[2] + 1 + rng.below(14) as i64).min(32),
            ];
            let m = rng.below(4) as u16;
            assert!(set_box(&f, n + 1, lo, hi, m).is_success());
        }
        build_all(&e);
        check_world(&e, &format!("seed {seed:#x}"));
        let all: Vec<Quad> = bricks()
            .into_iter()
            .flat_map(|b| greedy(&e, b).quads)
            .collect();
        if mesh_t_junctions(&naive(&all)) > 0 {
            naive_failures += 1;
        }
    });
    assert!(
        naive_failures > 0,
        "the checker must fail on a naive triangulation somewhere"
    );
}

#[test]
fn an_incremental_rebuild_equals_a_full_rebuild_and_stays_free_of_t_junctions() {
    let f = fx();
    let e = executor(&f);
    assert!(set_box(&f, 1, [-8, -8, -8], [20, 12, 20], 1).is_success());
    build_all(&e);
    check_world(&e, "before");
    // Carve across a brick join and add a different material, then rebuild everything that the
    // executor says is stale.
    assert!(set_box(&f, 2, [-3, 2, 5], [9, 6, 18], 0).is_success());
    assert!(set_box(&f, 3, [10, 0, -4], [14, 3, 2], 2).is_success());
    build_all(&e);
    check_world(&e, "after");
    let fresh = executor(&f);
    build_all(&fresh);
    for b in bricks() {
        assert_eq!(greedy(&e, b).quads, greedy(&fresh, b).quads, "{b:?}");
        assert_eq!(
            triangulate(&greedy(&e, b).quads),
            triangulate(&greedy(&fresh, b).quads)
        );
    }
}

#[test]
fn borders_never_hold_a_merged_quad_and_quads_never_cross_bricks() {
    let f = fx();
    let e = executor(&f);
    assert!(set_box(&f, 1, [-16, -16, -16], [32, 32, 32], 1).is_success());
    assert!(set_box(&f, 2, [0, 0, 0], [16, 16, 16], 0).is_success());
    build_all(&e);
    let by_key: HashMap<BrickCoord, GreedyMesh> =
        bricks().into_iter().map(|b| (b, greedy(&e, b))).collect();
    for (b, m) in by_key {
        let o = b.origin().unwrap();
        for q in &m.quads {
            let (n, lo) = match q.dir.axis() {
                0 => (q.origin.x, o.x),
                1 => (q.origin.y, o.y),
                _ => (q.origin.z, o.z),
            };
            let local = n - lo;
            let border = match q.dir {
                cube_backend_voxel::Dir::PosX
                | cube_backend_voxel::Dir::PosY
                | cube_backend_voxel::Dir::PosZ => local == 15,
                _ => local == 0,
            };
            if border {
                assert_eq!((q.u_len, q.v_len), (1, 1), "{q:?} in {b:?}");
            }
        }
    }
}
