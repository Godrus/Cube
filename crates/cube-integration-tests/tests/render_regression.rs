//! RND-01 (software form, CI host): pixel identity of the greedy mesh and the faces baseline.
//!
//! A tiny exact rasteriser draws the triangle meshes orthographically along every axis in both
//! directions with a depth buffer; the images of the greedy mesh (merged quads, fan triangles) and
//! of the faces mesh (unit quads) must be identical, pixel for pixel, for random worlds across
//! brick borders. This is a statement about geometry on the CI host; it says nothing about any
//! GPU, driver or Godot (the Godot picture is checked by the smoke run, not here).
#![cfg(not(loom))]

use std::sync::Arc;

use cube_backend_voxel::{
    FACES_KIND, FacesMesh, FacesProvider, GREEDY_KIND, GreedyMesh, GreedyProvider, Quad, TriMesh,
    VoxelBackend, set_material, triangulate,
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

const SIZE: i64 = 96 * 2; // pixels per side: one pixel per half voxel over 96 voxels
const MIN: i64 = -48; // voxel coordinate of pixel 0

/// Renders along `axis` (looking towards negative if `from_positive`). A pixel holds the
/// material of the nearest triangle covering its centre (0 = nothing).
fn render(mesh: &TriMesh, axis: usize, from_positive: bool) -> Vec<u16> {
    let (ua, va) = match axis {
        0 => (1, 2),
        1 => (0, 2),
        _ => (0, 1),
    };
    let mut img = vec![0u16; (SIZE * SIZE) as usize];
    let mut depth = vec![i64::MIN; (SIZE * SIZE) as usize];
    for (t, _, material) in &mesh.triangles {
        // Doubled half-voxel coordinates, relative to the image corner.
        let p: Vec<[i64; 3]> = t
            .iter()
            .map(|i| mesh.vertices[*i as usize].map(|c| 2 * c))
            .collect();
        let origin = 2 * 2 * MIN;
        let sx = |q: &[i64; 3]| q[ua] - origin;
        let sy = |q: &[i64; 3]| q[va] - origin;
        let near = |q: &[i64; 3]| if from_positive { q[axis] } else { -q[axis] };
        // Only triangles perpendicular to the view axis cover pixels.
        if p[0][axis] != p[1][axis] || p[1][axis] != p[2][axis] {
            continue;
        }
        let d = near(&p[0]);
        let (minx, maxx) = (
            p.iter().map(sx).min().unwrap(),
            p.iter().map(sx).max().unwrap(),
        );
        let (miny, maxy) = (
            p.iter().map(sy).min().unwrap(),
            p.iter().map(sy).max().unwrap(),
        );
        let edge = |a: &[i64; 3], b: &[i64; 3], x: i64, y: i64| {
            (sx(b) - sx(a)) * (y - sy(a)) - (sy(b) - sy(a)) * (x - sx(a))
        };
        let mut p = p;
        let mut area = edge(&p[0], &p[1], sx(&p[2]), sy(&p[2]));
        if area == 0 {
            continue;
        }
        if area < 0 {
            p.swap(1, 2);
            area = -area;
        }
        // Top-left fill rule: a sample exactly on an edge belongs to exactly one of the two
        // triangles that share it (fan spokes pass through sample points).
        let owns = |a: &[i64; 3], b: &[i64; 3]| {
            let (dx, dy) = (sx(b) - sx(a), sy(b) - sy(a));
            dy < 0 || (dy == 0 && dx > 0)
        };
        let own = [owns(&p[1], &p[2]), owns(&p[2], &p[0]), owns(&p[0], &p[1])];
        for py in (miny / 2).max(0)..=(maxy / 2).min(SIZE - 1) {
            for px in (minx / 2).max(0)..=(maxx / 2).min(SIZE - 1) {
                // One pixel = 2 quarter voxels; the sample point is odd, vertices are even.
                let (x, y) = (2 * px + 1, 2 * py + 1);
                let w = [
                    edge(&p[1], &p[2], x, y),
                    edge(&p[2], &p[0], x, y),
                    edge(&p[0], &p[1], x, y),
                ];
                let inside = (0..3).all(|k| w[k] > 0 || (w[k] == 0 && own[k]));
                if inside {
                    let i = (py * SIZE + px) as usize;
                    if d > depth[i] {
                        depth[i] = d;
                        img[i] = *material;
                    }
                }
            }
        }
    }
    img
}

fn world_meshes(e: &Executor) -> (TriMesh, TriMesh) {
    let g: Vec<TriMesh> = bricks()
        .into_iter()
        .map(|b| triangulate(&greedy(e, b).quads))
        .collect();
    let f: Vec<TriMesh> = bricks()
        .into_iter()
        .map(|b| triangulate(&unit_quads(&faces(e, b))))
        .collect();
    (
        TriMesh::union(&g.iter().collect::<Vec<_>>()),
        TriMesh::union(&f.iter().collect::<Vec<_>>()),
    )
}

#[test]
fn greedy_and_faces_render_to_identical_pictures_from_every_side() {
    let mut covered = 0usize;
    for_each_seed(0x2E2D, 6, |seed| {
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
            assert!(set_box(&f, n + 1, lo, hi, 1 + rng.below(3) as u16).is_success());
        }
        build_all(&e);
        let (greedy_mesh, faces_mesh) = world_meshes(&e);
        for axis in 0..3 {
            for from_positive in [true, false] {
                let a = render(&greedy_mesh, axis, from_positive);
                let b = render(&faces_mesh, axis, from_positive);
                assert_eq!(a, b, "seed {seed:#x} axis {axis} positive {from_positive}");
                covered += a.iter().filter(|m| **m != 0).count();
            }
        }
    });
    assert!(
        covered > 10_000,
        "the pictures are not empty: {covered} pixels"
    );
}

#[test]
fn the_rasteriser_sees_a_missing_triangle() {
    let f = fx();
    let e = executor(&f);
    assert!(set_box(&f, 1, [0, 0, 0], [8, 8, 8], 1).is_success());
    build_all(&e);
    let (g, faces_mesh) = world_meshes(&e);
    let mut broken = g.clone();
    for t in &mut broken.triangles {
        if t.1 == cube_backend_voxel::Dir::PosZ {
            t.2 = 9; // a wrong material on the top faces
        }
    }
    assert_ne!(render(&broken, 2, true), render(&faces_mesh, 2, true));
    assert_eq!(render(&g, 2, true), render(&faces_mesh, 2, true));
}
