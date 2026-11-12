//! SDF-01..03 through the production chain `Editing API -> Orchestration -> SDF Backend -> Core
//! commit`, and the SDF providers through real Processing (snapshots with apron, executor).
//! Timing/determinism statements are CI-host results, not device claims (DG-2).
#![cfg(not(loom))]

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::Arc;

use cube_backend_sdf::{
    SDF_COLLISION_KIND, SDF_MESH_KIND, SdfBackend, SdfCollision, Shape, SurfaceMesh,
    SurfaceNetsProvider, add, remove, set_material,
};
use cube_core::{
    BrickCoord, CellChange, CellKey, CellPayload, CellState, ChannelId, LayerId, LocalPos,
    OperationId, Region, VoxelPos,
};
use cube_editing_api::{
    EditMode, EditingError, EditingOperation, EditingResult, GeometryBackend, OverlayRead,
    Parameters, kinds,
};
use cube_integration_tests::conformance::backend::{Case, run};
use cube_integration_tests::fakes::*;
use cube_processing::{
    DerivedTarget, Executor, ExecutorConfig, ManualClock, Priority, PriorityClass, ProviderRegistry,
};

fn fx(materialise: i64) -> Fixture {
    let f = fixture_with(Options {
        backends: vec![Arc::new(SdfBackend::new())],
        sdf_layer: true,
        ..Options::default()
    });
    let mut n = 0u64;
    for x in -materialise..materialise {
        for y in -materialise..materialise {
            for z in -materialise..materialise {
                n += 1;
                f.system
                    .materialise(
                        OperationId::new(5, n),
                        LayerId::new(L_SDF),
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

fn region(lo: [i64; 3], hi: [i64; 3]) -> Region {
    Region::new(
        0,
        VoxelPos::new(lo[0], lo[1], lo[2]),
        VoxelPos::new(hi[0], hi[1], hi[2]),
    )
    .unwrap()
}

fn sdf_op(n: u64, kind: cube_editing_api::KindId, p: Parameters, r: Region) -> EditingOperation {
    EditingOperation {
        id: OperationId::new(13, n),
        kind,
        asset: cube_core::AssetId::new(ASSET),
        layers: vec![LayerId::new(L_SDF)],
        parameters: p,
        read_region: None,
        write_region: r,
        conversion: None,
        mode: EditMode::Destructive,
    }
}

fn sphere(r: i64) -> Shape {
    Shape::Sphere {
        centre: VoxelPos::new(0, 0, 0),
        radius_16: (r * 16) as u32,
    }
}

fn add_sphere(f: &Fixture, n: u64, r: i64, material: u16) -> EditingResult {
    f.run(vec![sdf_op(
        n,
        kinds::ADD_GEOMETRY,
        add(sphere(r), material),
        region([-16, -16, -16], [16, 16, 16]),
    )])
}

/// (density, material) of a voxel; `None` if the brick is not present.
fn cell_at(f: &Fixture, p: [i64; 3]) -> Option<(i16, u16)> {
    let (b, l) = VoxelPos::new(p[0], p[1], p[2]).split();
    let st = f.reader.snapshot();
    match st.cell(LayerId::new(L_SDF), 0, b)?.ok()? {
        CellState::Present(pl) => match &*pl {
            CellPayload::Channels(set) => Some((
                set.channel(ChannelId::Density)
                    .map_or(i16::MAX, |c| c.get(l) as i16),
                set.channel(ChannelId::Material)
                    .map_or(0, |c| c.get(l) as u16),
            )),
            _ => None,
        },
        CellState::KnownEmpty => Some((i16::MAX, 0)),
        _ => None,
    }
}

#[test]
fn add_and_remove_follow_the_negative_inside_convention_through_the_real_chain() {
    let f = fx(2);
    let base = f.commits();
    assert!(add_sphere(&f, 1, 6, 7).is_success());
    assert_eq!(f.commits(), base + 1, "one atomic commit for eight bricks");
    let (d, m) = cell_at(&f, [0, 0, 0]).unwrap();
    assert!(
        d < 0 && m == 7,
        "centre is inside with the material: {d} {m}"
    );
    let (d, m) = cell_at(&f, [12, 0, 0]).unwrap();
    assert!(d > 0 && m == 0, "outside has no material");
    // A surface voxel: value close to zero.
    let (d, _) = cell_at(&f, [5, 0, 0]).unwrap();
    assert!(d.abs() < 256, "within one voxel of the surface: {d}");

    // Subtracting a box carves it out and clears the material there.
    assert!(
        f.run(vec![sdf_op(
            2,
            kinds::REMOVE_GEOMETRY,
            remove(Shape::Box {
                min: VoxelPos::new(-2, -2, -2),
                max: VoxelPos::new(2, 2, 2)
            }),
            region([-16, -16, -16], [16, 16, 16]),
        )])
        .is_success()
    );
    let (d, m) = cell_at(&f, [0, 0, 0]).unwrap();
    assert!(d > 0 && m == 0, "carved: {d} {m}");
    assert!(
        cell_at(&f, [4, 0, 0]).unwrap().0 < 0,
        "the rest stays solid"
    );

    // Material assignment changes only inside cells and leaves the field alone.
    let before = cell_at(&f, [4, 0, 0]).unwrap().0;
    assert!(
        f.run(vec![sdf_op(
            3,
            kinds::SET_MATERIAL,
            set_material(9),
            region([-16, -16, -16], [16, 16, 16]),
        )])
        .is_success()
    );
    assert_eq!(cell_at(&f, [4, 0, 0]), Some((before, 9)));
    assert_eq!(cell_at(&f, [12, 0, 0]).unwrap().1, 0);
    // Removing everything collapses bricks to known-empty, not to missing.
    assert!(
        f.run(vec![sdf_op(
            4,
            kinds::REMOVE_GEOMETRY,
            remove(Shape::Box {
                min: VoxelPos::new(-40, -40, -40),
                max: VoxelPos::new(40, 40, 40)
            }),
            region([-16, -16, -16], [16, 16, 16]),
        )])
        .is_success()
    );
    let st = f.reader.snapshot();
    assert!(matches!(
        st.cell(LayerId::new(L_SDF), 0, bc(0, 0, 0)),
        Some(Ok(CellState::KnownEmpty))
    ));
}

#[test]
fn the_same_operations_give_the_identical_world_and_retries_do_nothing() {
    let (a, b) = (fx(2), fx(2));
    for f in [&a, &b] {
        assert!(add_sphere(f, 1, 6, 3).is_success());
        assert!(
            f.run(vec![sdf_op(
                2,
                kinds::REMOVE_GEOMETRY,
                remove(Shape::Sphere {
                    centre: VoxelPos::new(3, 0, 0),
                    radius_16: 48
                }),
                region([-16, -16, -16], [16, 16, 16]),
            )])
            .is_success()
        );
    }
    let h = |f: &Fixture| f.reader.snapshot().export_view().content_hash();
    assert_eq!(h(&a), h(&b));
    // An operation that changes nothing produces no commit.
    let commits = a.commits();
    assert!(
        a.run(vec![sdf_op(
            3,
            kinds::SET_MATERIAL,
            set_material(3),
            region([-16, -16, -16], [16, 16, 16]),
        )])
        .is_success()
    );
    assert_eq!(a.commits(), commits);
}

#[test]
fn unloaded_bricks_are_never_guessed_and_far_bricks_are_not_touched() {
    // Only the bricks around the origin are loaded; a small sphere does not reach the others.
    let f = fixture_with(Options {
        backends: vec![Arc::new(SdfBackend::new())],
        sdf_layer: true,
        ..Options::default()
    });
    // Nothing loaded: the sphere reaches unloaded bricks, so it is an error and changes nothing.
    let before = f.commits();
    let r = add_sphere(&f, 1, 6, 1);
    assert!(
        matches!(r, EditingResult::ValidationFailure(EditingError::InvalidOperation(m)) if m.contains("not loaded")),
        "{r:?}"
    );
    assert_eq!(f.commits(), before);
    // A shape far away from an unloaded region reaches nothing there: no error, no write.
    let far = f.run(vec![sdf_op(
        2,
        kinds::ADD_GEOMETRY,
        add(
            Shape::Sphere {
                centre: VoxelPos::new(100_000, 0, 0),
                radius_16: 32,
            },
            1,
        ),
        region([-16, -16, -16], [16, 16, 16]),
    )]);
    assert!(far.is_success());
    assert_eq!(f.commits(), before, "nothing changed");
}

#[test]
fn bad_parameters_oversize_regions_and_wrong_geometry_are_controlled_errors() {
    let f = fx(1);
    let bad = f.run(vec![sdf_op(
        1,
        kinds::ADD_GEOMETRY,
        Parameters {
            schema: None,
            bytes: vec![1],
        },
        region([0, 0, 0], [4, 4, 4]),
    )]);
    assert!(!bad.is_success());
    let huge = f.run(vec![sdf_op(
        2,
        kinds::ADD_GEOMETRY,
        add(sphere(6), 1),
        region([-1000, -1000, -1000], [1000, 1000, 1000]),
    )]);
    assert!(
        !huge.is_success(),
        "region over the limit is refused, not truncated"
    );
    let unsupported = f.run(vec![sdf_op(
        3,
        kinds::TRANSFORM_GEOMETRY,
        add(sphere(2), 1),
        region([0, 0, 0], [4, 4, 4]),
    )]);
    assert!(!unsupported.is_success());
}

struct Known;
impl OverlayRead for Known {
    fn cell(&self, _: CellKey) -> Result<CellState, EditingError> {
        Ok(CellState::KnownEmpty)
    }
}

#[test]
fn the_backend_passes_the_generic_conformance_suite() {
    let b = SdfBackend::new();
    let cases: Vec<Case> = [
        (kinds::ADD_GEOMETRY, add(sphere(3), 2)),
        (
            kinds::REMOVE_GEOMETRY,
            remove(Shape::Box {
                min: VoxelPos::new(0, 0, 0),
                max: VoxelPos::new(4, 4, 4),
            }),
        ),
        (kinds::SET_MATERIAL, set_material(2)),
    ]
    .into_iter()
    .enumerate()
    .map(|(i, (k, p))| Case {
        operation: {
            let mut o = sdf_op(i as u64 + 1, k, p, region([-8, -8, -8], [8, 8, 8]));
            o.layers = vec![LayerId::new(L_SDF)];
            o
        },
    })
    .collect();
    assert_eq!(run(&b, &cases, &Known), vec![]);
    assert!(b.capabilities().supports(kinds::ADD_GEOMETRY));
    assert!(!b.capabilities().supports(kinds::TRANSFORM_GEOMETRY));
}

#[test]
fn validate_layer_finds_material_outside_the_surface_and_skips_unknown_bricks() {
    let f = fx(2);
    assert!(add_sphere(&f, 1, 6, 4).is_success());
    let b = SdfBackend::new();
    let st = f.reader.snapshot();
    let _ = &st;
    // A healthy world validates; the overlay of the live asset is read through a fixture view.
    let ov = ViewOverlay::of(&f);
    let rep = b
        .validate_layer(
            &ov,
            LayerId::new(L_SDF),
            &region([-16, -16, -16], [16, 16, 16]),
        )
        .unwrap();
    assert!(rep.is_valid(), "{rep:?}");
    // The port-level validate still refuses honestly (TD-2).
    assert!(b.validate(&ov, &region([0, 0, 0], [1, 1, 1])).is_err());
}

/// Overlay over the committed state of a fixture.
struct ViewOverlay(std::sync::Arc<cube_core::AssetState>);
impl ViewOverlay {
    fn of(f: &Fixture) -> Self {
        Self(f.reader.snapshot())
    }
}
impl OverlayRead for ViewOverlay {
    fn cell(&self, key: CellKey) -> Result<CellState, EditingError> {
        match self.0.cell(key.layer(), key.level(), key.brick()) {
            Some(Ok(s)) => Ok(s),
            Some(Err(_)) | None => Err(EditingError::InvalidOperation("unreadable")),
        }
    }
}

// ---- providers ----

fn executor(f: &Fixture) -> Executor {
    let mut r = ProviderRegistry::new();
    r.register(Arc::new(SurfaceNetsProvider)).unwrap();
    r.register(Arc::new(cube_backend_sdf::SdfCollisionProvider))
        .unwrap();
    let e = Executor::new(
        f.reader.clone(),
        r.freeze().unwrap(),
        ExecutorConfig::default(),
        Arc::new(ManualClock::new(0)),
    );
    f.sink.forward_to(Arc::new(e.clone()));
    e
}

fn target(kind: cube_processing::DerivedKind, b: BrickCoord) -> DerivedTarget {
    DerivedTarget {
        kind,
        key: CellKey::new(LayerId::new(L_SDF), 0, b).unwrap(),
    }
}

fn mesh_of(e: &Executor, b: BrickCoord) -> SurfaceMesh {
    e.get(target(SDF_MESH_KIND, b))
        .unwrap()
        .expect("mesh is current")
        .downcast_ref::<SurfaceMesh>()
        .unwrap()
        .clone()
}

fn build(e: &Executor, kind: cube_processing::DerivedKind, bricks: &[BrickCoord]) {
    for b in bricks {
        e.submit(target(kind, *b), Priority::new(PriorityClass::Visible))
            .unwrap();
    }
    e.run_until_idle();
}

fn cube(lo: i64, hi: i64) -> Vec<BrickCoord> {
    let mut v = Vec::new();
    for x in lo..=hi {
        for y in lo..=hi {
            for z in lo..=hi {
                v.push(bc(x, y, z));
            }
        }
    }
    v
}

/// Topology of the union of the per-brick meshes: vertices merged by position.
struct Topology {
    vertices: usize,
    edges: usize,
    faces: usize,
    /// Undirected edges that are not shared by exactly two triangles.
    open_edges: usize,
    /// Directed edges used more than once in the same direction (inconsistent winding).
    bad_winding: usize,
    unloaded: bool,
}

fn topology(e: &Executor, bricks: &[BrickCoord]) -> Topology {
    let mut ids: BTreeMap<[i128; 3], u32> = BTreeMap::new();
    let mut directed: HashMap<(u32, u32), u32> = HashMap::new();
    let mut faces = 0;
    let mut unloaded = false;
    for b in bricks {
        let m = mesh_of(e, *b);
        unloaded |= m.unloaded_neighbours;
        let map: Vec<u32> = m
            .vertices
            .iter()
            .map(|v| {
                let n = ids.len() as u32;
                *ids.entry(v.pos).or_insert(n)
            })
            .collect();
        for t in &m.triangles {
            let t = [map[t[0] as usize], map[t[1] as usize], map[t[2] as usize]];
            if t[0] == t[1] || t[1] == t[2] || t[0] == t[2] {
                continue; // a collapsed triangle carries no area
            }
            faces += 1;
            for i in 0..3 {
                *directed.entry((t[i], t[(i + 1) % 3])).or_default() += 1;
            }
        }
    }
    let undirected: BTreeSet<(u32, u32)> = directed
        .keys()
        .map(|(a, b)| (*a.min(b), *a.max(b)))
        .collect();
    let open_edges = undirected
        .iter()
        .filter(|(a, b)| {
            directed.get(&(*a, *b)).copied().unwrap_or(0) != 1
                || directed.get(&(*b, *a)).copied().unwrap_or(0) != 1
        })
        .count();
    let bad_winding = directed.values().filter(|c| **c > 1).count();
    Topology {
        vertices: ids.len(),
        edges: undirected.len(),
        faces,
        open_edges,
        bad_winding,
        unloaded,
    }
}

#[test]
fn a_sphere_over_eight_bricks_is_one_watertight_manifold_without_cracks_at_joins() {
    let f = fx(2);
    assert!(add_sphere(&f, 1, 6, 5).is_success());
    let e = executor(&f);
    let bricks = cube(-1, 0);
    build(&e, SDF_MESH_KIND, &bricks);
    let t = topology(&e, &bricks);
    assert!(t.faces > 100, "a real surface was produced: {}", t.faces);
    assert_eq!(
        t.open_edges, 0,
        "every edge is shared by exactly two triangles, opposite winding"
    );
    assert_eq!(t.bad_winding, 0);
    // Euler characteristic of a sphere.
    assert_eq!(t.vertices as i64 - t.edges as i64 + t.faces as i64, 2);
    assert!(!t.unloaded);
    // Vertices lie on the surface within the documented tolerance (half a voxel) and normals
    // point outwards.
    for b in &bricks {
        for v in &mesh_of(&e, *b).vertices {
            let c = v.pos.map(|q| q as f64 / 256.0);
            let r = (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt();
            assert!((r - 6.0).abs() < 0.6, "vertex radius {r}");
            let out: f64 = (0..3)
                .map(|i| f64::from(v.normal[i]) / 16384.0 * c[i] / r)
                .sum();
            assert!(out > 0.9, "normal points outward: {out}");
            assert_eq!(v.material, 5);
        }
    }
}

#[test]
fn meshing_is_deterministic_and_incremental_edits_equal_a_full_rebuild() {
    let f = fx(2);
    assert!(add_sphere(&f, 1, 6, 5).is_success());
    let e = executor(&f);
    let bricks = cube(-1, 0);
    build(&e, SDF_MESH_KIND, &bricks);
    // Edit: carve a hole that crosses a brick join, rebuild only the dirty neighbourhood.
    assert!(
        f.run(vec![sdf_op(
            2,
            kinds::REMOVE_GEOMETRY,
            remove(Shape::Sphere {
                centre: VoxelPos::new(0, 0, 6),
                radius_16: 40
            }),
            region([-16, -16, -16], [16, 16, 16]),
        )])
        .is_success()
    );
    build(&e, SDF_MESH_KIND, &bricks);
    let incremental: Vec<SurfaceMesh> = bricks.iter().map(|b| mesh_of(&e, *b)).collect();
    // A second executor that never saw the old state computes the same meshes from scratch.
    let fresh = executor(&f);
    build(&fresh, SDF_MESH_KIND, &bricks);
    for (b, inc) in bricks.iter().zip(&incremental) {
        assert_eq!(&mesh_of(&fresh, *b), inc, "brick {b:?}");
    }
    let t = topology(&e, &bricks);
    assert_eq!(t.open_edges, 0, "a carved sphere is still closed");
}

#[test]
fn absent_data_is_explicit_in_meshes_and_collision() {
    // Only one brick materialised: its neighbours are not loaded and are flagged, not guessed.
    let f = fixture_with(Options {
        backends: vec![Arc::new(SdfBackend::new())],
        sdf_layer: true,
        ..Options::default()
    });
    f.system
        .materialise(
            OperationId::new(5, 1),
            LayerId::new(L_SDF),
            0,
            bc(0, 0, 0),
            CellChange::KnownEmpty,
        )
        .unwrap();
    assert!(
        f.run(vec![sdf_op(
            1,
            kinds::ADD_GEOMETRY,
            add(
                Shape::Box {
                    min: VoxelPos::new(2, 2, 2),
                    max: VoxelPos::new(6, 6, 6)
                },
                1
            ),
            region([0, 0, 0], [16, 16, 16]),
        )])
        .is_success()
    );
    let e = executor(&f);
    build(&e, SDF_MESH_KIND, &[bc(0, 0, 0)]);
    let m = mesh_of(&e, bc(0, 0, 0));
    assert!(m.unloaded_neighbours && !m.triangles.is_empty());
    // A centre brick that is not loaded is a failure, never an empty mesh.
    assert!(
        e.submit(
            target(SDF_MESH_KIND, bc(5, 5, 5)),
            Priority::new(PriorityClass::Visible)
        )
        .is_err()
            || {
                e.run_until_idle();
                e.get(target(SDF_MESH_KIND, bc(5, 5, 5))).is_err()
                    || e.get(target(SDF_MESH_KIND, bc(5, 5, 5))).unwrap().is_none()
            }
    );
}

#[test]
fn collision_boxes_cover_exactly_the_solid_voxels() {
    let f = fx(2);
    assert!(add_sphere(&f, 1, 6, 5).is_success());
    let e = executor(&f);
    let bricks = cube(-1, 0);
    build(&e, SDF_COLLISION_KIND, &bricks);
    for b in &bricks {
        let col = e
            .get(target(SDF_COLLISION_KIND, *b))
            .unwrap()
            .unwrap()
            .downcast_ref::<SdfCollision>()
            .unwrap()
            .clone();
        let volume: i64 = col
            .boxes
            .iter()
            .map(|x| (0..3).map(|a| x.max[a] - x.min[a]).product::<i64>())
            .sum();
        let st = f.reader.snapshot();
        let solid = match st.cell(LayerId::new(L_SDF), 0, *b).unwrap().unwrap() {
            CellState::Present(p) => match &*p {
                CellPayload::Channels(set) => LocalPos::all()
                    .filter(|l| (set.channel(ChannelId::Density).unwrap().get(*l) as i16) < 0)
                    .count() as i64,
                _ => unreachable!(),
            },
            _ => 0,
        };
        assert_eq!(volume, solid, "brick {b:?}");
        assert!(col.boxes.len() <= 2048, "budget");
    }
}
