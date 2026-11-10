//! MSH-01..03 through the production chain `Editing API -> Orchestration -> Mesh Backend -> Core
//! commit` and real Processing for the collision provider. No renderer is involved.
//! Determinism statements are CI-host results, not device claims (DG-2).
#![cfg(not(loom))]

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use cube_backend_mesh::{
    MESH_COLLISION_KIND, MeshBackend, MeshCollision, MeshCollisionProvider, Page, add_box,
    add_triangles, remove, set_material,
};
use cube_core::{
    BrickCoord, CellChange, CellKey, CellPayload, CellState, LayerId, OperationId, Region, VoxelPos,
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

fn fx(materialise: bool) -> Fixture {
    let f = fixture_with(Options {
        backends: vec![Arc::new(MeshBackend::new())],
        ..Options::default()
    });
    if materialise {
        let mut n = 0;
        for x in -1..=1 {
            for y in -1..=1 {
                for z in -1..=1 {
                    n += 1;
                    f.system
                        .materialise(
                            OperationId::new(5, n),
                            LayerId::new(L_MESH),
                            0,
                            bc(x, y, z),
                            CellChange::KnownEmpty,
                        )
                        .unwrap();
                }
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

fn big() -> Region {
    region([-16, -16, -16], [32, 32, 32])
}

fn mop(n: u64, kind: cube_editing_api::KindId, p: Parameters, r: Region) -> EditingOperation {
    EditingOperation {
        id: OperationId::new(17, n),
        kind,
        asset: cube_core::AssetId::new(ASSET),
        layers: vec![LayerId::new(L_MESH)],
        parameters: p,
        read_region: None,
        write_region: r,
        conversion: None,
        mode: EditMode::Destructive,
    }
}

fn add_cube(f: &Fixture, n: u64, min: [i64; 3], max: [i64; 3], m: u16) -> EditingResult {
    f.run(vec![mop(
        n,
        kinds::ADD_GEOMETRY,
        add_box(min, max, m),
        big(),
    )])
}

/// All triangles of the layer, absolute 1/256 voxel, from the committed pages.
fn world_tris(f: &Fixture) -> Vec<([[i128; 3]; 3], u16)> {
    let st = f.reader.snapshot();
    let mut out = Vec::new();
    for x in -1..=1 {
        for y in -1..=1 {
            for z in -1..=1 {
                let b = bc(x, y, z);
                if let Some(Ok(CellState::Present(p))) = st.cell(LayerId::new(L_MESH), 0, b) {
                    let CellPayload::MeshPage(m) = &*p else {
                        panic!("not a mesh page")
                    };
                    let page = Page::decode(m.bytes()).expect("committed pages are valid");
                    let o = b.origin().unwrap();
                    for t in page.tris {
                        out.push((
                            t.v.map(|v| {
                                [
                                    i128::from(o.x) * 256 + i128::from(v[0]),
                                    i128::from(o.y) * 256 + i128::from(v[1]),
                                    i128::from(o.z) * 256 + i128::from(v[2]),
                                ]
                            }),
                            t.material,
                        ));
                    }
                }
            }
        }
    }
    out
}

#[test]
fn a_box_over_several_pages_is_one_closed_consistently_wound_manifold() {
    let f = fx(true);
    let base = f.commits();
    assert!(add_cube(&f, 1, [4, 4, 4], [20, 12, 12], 3).is_success());
    assert_eq!(f.commits(), base + 1, "one atomic commit for all pages");
    let tris = world_tris(&f);
    assert_eq!(tris.len(), 12);
    // Union of all pages: every edge shared by exactly two triangles in opposite directions.
    let mut dir: BTreeMap<([i128; 3], [i128; 3]), u32> = BTreeMap::new();
    let mut verts = BTreeSet::new();
    for (t, m) in &tris {
        assert_eq!(*m, 3);
        for k in 0..3 {
            verts.insert(t[k]);
            *dir.entry((t[k], t[(k + 1) % 3])).or_default() += 1;
        }
    }
    assert!(dir.values().all(|c| *c == 1));
    assert!(dir.keys().all(|(a, b)| dir.contains_key(&(*b, *a))));
    let edges = dir.len() / 2;
    assert_eq!(verts.len() as i64 - edges as i64 + tris.len() as i64, 2);
    // The triangles were distributed over more than one page (brick 0 and brick 1 along x).
    let st = f.reader.snapshot();
    let pages = [bc(0, 0, 0), bc(1, 0, 0)]
        .iter()
        .filter(|b| {
            matches!(
                st.cell(LayerId::new(L_MESH), 0, **b),
                Some(Ok(CellState::Present(_)))
            )
        })
        .count();
    assert_eq!(pages, 2);
}

#[test]
fn history_does_not_matter_and_repeating_an_add_changes_nothing() {
    let (a, b) = (fx(true), fx(true));
    assert!(add_cube(&a, 1, [2, 2, 2], [6, 6, 6], 1).is_success());
    assert!(add_cube(&a, 2, [8, 8, 8], [12, 12, 12], 2).is_success());
    assert!(add_cube(&b, 1, [8, 8, 8], [12, 12, 12], 2).is_success());
    assert!(add_cube(&b, 2, [2, 2, 2], [6, 6, 6], 1).is_success());
    let h = |f: &Fixture| f.reader.snapshot().export_view().content_hash();
    assert_eq!(h(&a), h(&b), "the same triangles give the same pages");
    let commits = a.commits();
    assert!(add_cube(&a, 3, [2, 2, 2], [6, 6, 6], 1).is_success());
    assert_eq!(a.commits(), commits, "already present: no write, no commit");
}

#[test]
fn remove_and_set_material_act_on_triangles_whose_centroid_is_in_the_region() {
    let f = fx(true);
    assert!(add_cube(&f, 1, [2, 2, 2], [6, 6, 6], 1).is_success());
    assert!(add_cube(&f, 2, [20, 2, 2], [24, 6, 6], 1).is_success());
    assert_eq!(world_tris(&f).len(), 24);
    assert!(
        f.run(vec![mop(
            3,
            kinds::SET_MATERIAL,
            set_material(9),
            region([0, 0, 0], [16, 16, 16])
        )])
        .is_success()
    );
    let t = world_tris(&f);
    assert_eq!(t.iter().filter(|(_, m)| *m == 9).count(), 12);
    assert_eq!(t.iter().filter(|(_, m)| *m == 1).count(), 12);
    assert!(
        f.run(vec![mop(
            4,
            kinds::REMOVE_GEOMETRY,
            remove(),
            region([0, 0, 0], [16, 16, 16])
        )])
        .is_success()
    );
    assert_eq!(world_tris(&f).len(), 12);
    // Removing the rest collapses its pages to known-empty, not missing.
    assert!(
        f.run(vec![mop(5, kinds::REMOVE_GEOMETRY, remove(), big())])
            .is_success()
    );
    assert!(world_tris(&f).is_empty());
    let st = f.reader.snapshot();
    assert!(matches!(
        st.cell(LayerId::new(L_MESH), 0, bc(1, 0, 0)),
        Some(Ok(CellState::KnownEmpty))
    ));
}

#[test]
fn invalid_input_is_a_typed_refusal_and_changes_nothing() {
    let f = fx(true);
    let base = f.commits();
    let refused = |r: EditingResult| {
        matches!(
            r,
            EditingResult::ValidationFailure(EditingError::InvalidOperation(_))
        )
    };
    // Degenerate triangle.
    let degenerate = [([[0, 0, 0], [256, 0, 0], [512, 0, 0]], 1u16)];
    assert!(refused(f.run(vec![mop(
        1,
        kinds::ADD_GEOMETRY,
        add_triangles(&degenerate),
        big()
    )])));
    // Two triangles that traverse their shared edge in the same direction.
    let same_dir = [
        ([[0, 0, 0], [256, 0, 0], [0, 256, 0]], 1u16),
        ([[0, 0, 0], [256, 0, 0], [0, 0, 256]], 1u16),
    ];
    assert!(refused(f.run(vec![mop(
        2,
        kinds::ADD_GEOMETRY,
        add_triangles(&same_dir),
        big()
    )])));
    // A triangle whose centroid is outside the write region.
    let far = [([[0, 0, 0], [256, 0, 0], [0, 256, 0]], 1u16)];
    assert!(refused(f.run(vec![mop(
        3,
        kinds::ADD_GEOMETRY,
        add_triangles(&far),
        region([100, 100, 100], [116, 116, 116])
    )])));
    // Oversize region and malformed parameters.
    assert!(
        !f.run(vec![mop(
            4,
            kinds::REMOVE_GEOMETRY,
            remove(),
            region([-9999, -9999, -9999], [9999, 9999, 9999])
        )])
        .is_success()
    );
    let bad = Parameters {
        schema: None,
        bytes: vec![],
    };
    assert!(
        !f.run(vec![mop(5, kinds::ADD_GEOMETRY, bad, big())])
            .is_success()
    );
    assert!(
        !f.run(vec![mop(6, kinds::TRANSFORM_GEOMETRY, remove(), big())])
            .is_success()
    );
    assert_eq!(f.commits(), base);
    // Pages that are not loaded are never guessed.
    let g = fx(false);
    let r = add_cube(&g, 1, [2, 2, 2], [6, 6, 6], 1);
    assert!(
        matches!(r, EditingResult::ValidationFailure(EditingError::InvalidOperation(m)) if m.contains("not loaded")),
        "{r:?}"
    );
}

struct Known;
impl OverlayRead for Known {
    fn cell(&self, _: CellKey) -> Result<CellState, EditingError> {
        Ok(CellState::KnownEmpty)
    }
}

#[test]
fn the_backend_passes_the_generic_conformance_suite_and_validates_layers() {
    let b = MeshBackend::new();
    let cases: Vec<Case> = [
        (kinds::ADD_GEOMETRY, add_box([2, 2, 2], [6, 6, 6], 1)),
        (kinds::REMOVE_GEOMETRY, remove()),
        (kinds::SET_MATERIAL, set_material(2)),
    ]
    .into_iter()
    .enumerate()
    .map(|(i, (k, p))| Case {
        operation: mop(i as u64 + 1, k, p, region([-8, -8, -8], [16, 16, 16])),
    })
    .collect();
    assert_eq!(run(&b, &cases, &Known), vec![]);

    let f = fx(true);
    assert!(add_cube(&f, 1, [2, 2, 2], [6, 6, 6], 1).is_success());
    struct Over(std::sync::Arc<cube_core::AssetState>);
    impl OverlayRead for Over {
        fn cell(&self, k: CellKey) -> Result<CellState, EditingError> {
            match self.0.cell(k.layer(), k.level(), k.brick()) {
                Some(Ok(s)) => Ok(s),
                _ => Err(EditingError::InvalidOperation("unreadable")),
            }
        }
    }
    let ov = Over(f.reader.snapshot());
    let rep = b.validate_layer(&ov, LayerId::new(L_MESH), &big()).unwrap();
    assert!(rep.is_valid());
    assert!(
        b.validate(&ov, &big()).is_err(),
        "port-level validate refuses honestly (TD-2)"
    );
}

fn executor(f: &Fixture) -> Executor {
    let mut r = ProviderRegistry::new();
    r.register(Arc::new(MeshCollisionProvider)).unwrap();
    let e = Executor::new(
        f.reader.clone(),
        r.freeze().unwrap(),
        ExecutorConfig::default(),
        Arc::new(ManualClock::new(0)),
    );
    f.sink.forward_to(Arc::new(e.clone()));
    e
}

fn collision_of(e: &Executor, b: BrickCoord) -> MeshCollision {
    let t = DerivedTarget {
        kind: MESH_COLLISION_KIND,
        key: CellKey::new(LayerId::new(L_MESH), 0, b).unwrap(),
    };
    e.submit(t, Priority::new(PriorityClass::Visible)).unwrap();
    e.run_until_idle();
    e.get(t)
        .unwrap()
        .expect("collision is current")
        .downcast_ref::<MeshCollision>()
        .unwrap()
        .clone()
}

#[test]
fn collision_triangles_match_the_committed_page_and_a_golden_box() {
    let f = fx(true);
    assert!(add_cube(&f, 1, [2, 2, 2], [6, 6, 6], 1).is_success());
    let e = executor(&f);
    let c = collision_of(&e, bc(0, 0, 0));
    // Only +x/+y/+z/-x/-y/-z faces whose centroid is in brick 0: all twelve (box well inside).
    assert_eq!(c.triangles.len(), 12);
    assert_eq!(
        c.bounds,
        Some(([512, 512, 512], [1536, 1536, 1536])),
        "bounds in 1/256 voxel"
    );
    // Golden: a fingerprint of the triangle list pinned for this provider version.
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for t in &c.triangles {
        for v in t {
            for x in v {
                for b in x.to_le_bytes() {
                    h ^= u64::from(b);
                    h = h.wrapping_mul(0x0000_0100_0000_01b3);
                }
            }
        }
    }
    assert_eq!(
        h, GOLDEN_BOX_COLLISION,
        "provider output changed: new golden {h}"
    );
    // An edit invalidates and the rebuilt result follows it.
    assert!(
        f.run(vec![mop(2, kinds::REMOVE_GEOMETRY, remove(), big())])
            .is_success()
    );
    assert!(collision_of(&e, bc(0, 0, 0)).triangles.is_empty());
    // A page that is not loaded is a failure, never "empty".
    let t = DerivedTarget {
        kind: MESH_COLLISION_KIND,
        key: CellKey::new(LayerId::new(L_MESH), 0, bc(7, 7, 7)).unwrap(),
    };
    let r = e.submit(t, Priority::new(PriorityClass::Visible));
    e.run_until_idle();
    assert!(r.is_err() || e.get(t).map_or(true, |p| p.is_none()));
}

const GOLDEN_BOX_COLLISION: u64 = 11_716_351_056_715_993_125;
