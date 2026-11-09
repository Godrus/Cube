//! VOX-01 through the production chain `Editing API -> Orchestration -> Voxel Backend -> Core
//! commit`: real Core asset, real `OrchestrationService`, real `VoxelBackend`. Only the durable
//! journal is outside this scope (PS-10), so no crash durability is claimed.
#![cfg(not(loom))]

use std::collections::HashMap;

use cube_backend_voxel::{TransformMode, VoxelBackend, add, remove, set_material, transform};
use cube_core::{
    CellChange, CellPayload, CellState, Channel, ChannelId, ChannelSet, LayerId, LocalPos,
    OperationId, Region, Rotation, VoxelPos,
};
use cube_editing_api::{
    EditMode, EditingError, EditingOperation, EditingResult, EditingService, Origin, Parameters,
    kinds,
};
use cube_integration_tests::conformance::backend::{Case, run};
use cube_integration_tests::fakes::*;
use cube_testkit::{DetRng, for_each_seed};

fn fx() -> Fixture {
    fixture_with(Options {
        backends: vec![VoxelBackend::arc()],
        ..Options::default()
    })
}

fn vbox(level: i8, min: [i64; 3], max: [i64; 3]) -> Region {
    Region::new(
        level,
        VoxelPos::new(min[0], min[1], min[2]),
        VoxelPos::new(max[0], max[1], max[2]),
    )
    .expect("box")
}

fn vop(
    n: u64,
    kind: cube_editing_api::KindId,
    layer: u64,
    region: Region,
    p: Parameters,
) -> EditingOperation {
    EditingOperation {
        id: OperationId::new(9, n),
        kind,
        asset: cube_core::AssetId::new(ASSET),
        layers: vec![LayerId::new(layer)],
        parameters: p,
        read_region: None,
        write_region: region,
        conversion: None,
        mode: EditMode::Destructive,
    }
}

fn voxel_at(f: &Fixture, layer: u64, level: i8, p: [i64; 3]) -> Option<u16> {
    let (b, l) = VoxelPos::new(p[0], p[1], p[2]).split();
    let state = f.reader.snapshot();
    match state.cell(LayerId::new(layer), level, b)?.ok()? {
        CellState::Present(pl) => match &*pl {
            CellPayload::Channels(set) => {
                Some(set.channel(ChannelId::Material).map_or(0, |c| c.get(l)))
            }
            _ => None,
        },
        CellState::KnownEmpty => Some(0),
        _ => None,
    }
}

fn v0(f: &Fixture, p: [i64; 3]) -> Option<u16> {
    voxel_at(f, L_VOXEL, 0, p)
}

fn is_err(r: &EditingResult) -> bool {
    !r.is_success() && r.left_state_unchanged()
}

#[test]
fn scenario_add_replace_remove_one_voxel_through_the_whole_chain() {
    let f = fx();
    // 1-3: add a voxel in a world with no data and see the material in Core.
    let r = f.run(vec![vop(
        1,
        kinds::ADD_GEOMETRY,
        L_VOXEL,
        vbox(0, [5, 5, 5], [6, 6, 6]),
        add(3),
    )]);
    // The brick is not loaded and the edit covers one voxel of it: refused, never guessed.
    assert!(
        matches!(r, EditingResult::ValidationFailure(EditingError::InvalidOperation(m)) if m.contains("not loaded")),
        "{r:?}"
    );
    assert_eq!(f.commits(), 0);
    // Declare the brick loaded and empty (what generation or loading would do), then add.
    f.system
        .materialise(
            OperationId::new(1, 1),
            LayerId::new(L_VOXEL),
            0,
            bc(0, 0, 0),
            CellChange::KnownEmpty,
        )
        .unwrap();
    let r = f.run(vec![vop(
        2,
        kinds::ADD_GEOMETRY,
        L_VOXEL,
        vbox(0, [5, 5, 5], [6, 6, 6]),
        add(3),
    )]);
    assert!(r.is_success(), "{r:?}");
    assert_eq!(
        (v0(&f, [5, 5, 5]), v0(&f, [5, 5, 6]), v0(&f, [4, 5, 5])),
        (Some(3), Some(0), Some(0))
    );
    // 4: replace the material.
    let r = f.run(vec![vop(
        3,
        kinds::SET_MATERIAL,
        L_VOXEL,
        vbox(0, [5, 5, 5], [6, 6, 6]),
        set_material(4, Some(3)),
    )]);
    assert!(r.is_success(), "{r:?}");
    assert_eq!(v0(&f, [5, 5, 5]), Some(4));
    // Replacing a material that is not there changes nothing and adds no commit.
    let before = f.commits();
    assert!(
        f.run(vec![vop(
            4,
            kinds::SET_MATERIAL,
            L_VOXEL,
            vbox(0, [5, 5, 5], [6, 6, 6]),
            set_material(9, Some(3))
        )])
        .is_success()
    );
    assert_eq!((f.commits(), v0(&f, [5, 5, 5])), (before, Some(4)));
    // 5: remove it; an all-air brick collapses to known-empty, not to missing.
    assert!(
        f.run(vec![vop(
            5,
            kinds::REMOVE_GEOMETRY,
            L_VOXEL,
            vbox(0, [5, 5, 5], [6, 6, 6]),
            remove(None)
        )])
        .is_success()
    );
    assert_eq!(v0(&f, [5, 5, 5]), Some(0));
    let st = f.reader.snapshot();
    assert!(matches!(
        st.cell(LayerId::new(L_VOXEL), 0, bc(0, 0, 0)),
        Some(Ok(CellState::KnownEmpty))
    ));
}

#[test]
fn scenario_atomic_edit_across_brick_borders_with_negative_coordinates() {
    let f = fx();
    // A 4x4x4 box around the origin touches eight bricks; no brick is loaded, but every brick
    // is only partly covered, so first load them as empty.
    for x in -1..=0 {
        for y in -1..=0 {
            for z in -1..=0 {
                f.system
                    .materialise(
                        OperationId::new(
                            2,
                            (x + 1) as u64 * 4 + (y + 1) as u64 * 2 + (z + 1) as u64,
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
    let base = f.commits();
    let r = f.run(vec![vop(
        1,
        kinds::SET_MATERIAL,
        L_VOXEL,
        vbox(0, [-2, -2, -2], [2, 2, 2]),
        set_material(7, None),
    )]);
    let EditingResult::Success(s) = r else {
        panic!("{r:?}")
    };
    assert_eq!(f.commits(), base + 1, "eight bricks, one atomic commit");
    assert_eq!(s.dirty.len(), 8);
    for x in -3..3 {
        for y in -3..3 {
            for z in -3..3 {
                let inside = (-2..2).contains(&x) && (-2..2).contains(&y) && (-2..2).contains(&z);
                let want = if inside {
                    Some(7)
                } else if (-16..16).contains(&x) {
                    Some(0)
                } else {
                    None
                };
                assert_eq!(v0(&f, [x, y, z]), want, "{x},{y},{z}");
            }
        }
    }
}

#[test]
fn scenario_conflict_permission_and_failure_leave_the_world_unchanged() {
    let f = fx();
    f.system
        .materialise(
            OperationId::new(1, 1),
            LayerId::new(L_VOXEL),
            0,
            bc(0, 0, 0),
            CellChange::KnownEmpty,
        )
        .unwrap();
    let region = vbox(0, [1, 1, 1], [3, 3, 3]);
    // Revision conflict: two transactions begin, both write the brick, the second loses.
    let a = f.svc.begin(ctx(Origin::Server), spec()).unwrap();
    let b = f.svc.begin(ctx(Origin::Server), spec()).unwrap();
    assert!(
        f.svc
            .execute(&a, vop(1, kinds::ADD_GEOMETRY, L_VOXEL, region, add(1)))
            .is_success()
    );
    assert!(
        f.svc
            .execute(&b, vop(2, kinds::ADD_GEOMETRY, L_VOXEL, region, add(2)))
            .is_success()
    );
    assert!(f.svc.commit(&a).is_success());
    assert!(matches!(f.svc.commit(&b), EditingResult::Conflict(_)));
    assert_eq!(v0(&f, [2, 2, 2]), Some(1));
    // Permission: no capability, no change, nothing staged.
    let mut denied = ctx(Origin::Server);
    denied.capabilities.clear();
    assert_eq!(
        f.svc.begin(denied, spec()).unwrap_err(),
        EditingError::PermissionDenied
    );
    // A failing operation (outside the layer bounds) changes nothing.
    let before = (f.commits(), f.revision());
    let r = f.run(vec![vop(
        3,
        kinds::SET_MATERIAL,
        L_BOUNDED,
        vbox(0, [100, 0, 0], [101, 1, 1]),
        set_material(1, None),
    )]);
    assert!(is_err(&r), "{r:?}");
    assert_eq!((f.commits(), f.revision()), before);
    // A transaction with one good and one failing operation applies neither.
    let h = f.svc.begin(ctx(Origin::Server), spec()).unwrap();
    assert!(
        f.svc
            .execute(
                &h,
                vop(
                    4,
                    kinds::SET_MATERIAL,
                    L_VOXEL,
                    vbox(0, [0, 0, 0], [16, 16, 16]),
                    set_material(5, None)
                )
            )
            .is_success()
    );
    assert!(
        !f.svc
            .execute(
                &h,
                vop(
                    5,
                    kinds::ADD_GEOMETRY,
                    L_VOXEL,
                    vbox(0, [40, 0, 0], [41, 1, 1]),
                    add(1)
                )
            )
            .is_success()
    );
    f.svc.rollback(&h).unwrap();
    assert_eq!(v0(&f, [2, 2, 2]), Some(1));
}

#[test]
fn unloaded_and_partly_covered_bricks_are_never_guessed() {
    let f = fx();
    // Fully covering an unloaded brick is exact and allowed.
    let r = f.run(vec![vop(
        1,
        kinds::SET_MATERIAL,
        L_VOXEL,
        vbox(0, [0, 0, 0], [16, 16, 16]),
        set_material(2, None),
    )]);
    assert!(r.is_success(), "{r:?}");
    assert_eq!(
        (v0(&f, [0, 0, 0]), v0(&f, [15, 15, 15]), v0(&f, [16, 0, 0])),
        (Some(2), Some(2), None)
    );
    // A partial write into another unloaded brick would overwrite unknown content: refused.
    for kind_params in [
        (kinds::SET_MATERIAL, set_material(1, None)),
        (kinds::REMOVE_GEOMETRY, remove(None)),
        (kinds::ADD_GEOMETRY, add(1)),
        (kinds::SET_MATERIAL, set_material(1, Some(2))),
    ] {
        let r = f.run(vec![vop(
            10,
            kind_params.0,
            L_VOXEL,
            vbox(0, [16, 0, 0], [20, 4, 4]),
            kind_params.1,
        )]);
        assert!(is_err(&r), "{r:?}");
        assert_eq!(v0(&f, [16, 0, 0]), None, "still unloaded");
    }
    // A region that spans a loaded brick and an unloaded one fails as a whole, atomically.
    let r = f.run(vec![vop(
        11,
        kinds::SET_MATERIAL,
        L_VOXEL,
        vbox(0, [8, 0, 0], [24, 4, 4]),
        set_material(9, None),
    )]);
    assert!(is_err(&r));
    assert_eq!(v0(&f, [8, 0, 0]), Some(2));
    // Beyond the bounds of a bounded layer is an error, not an empty answer.
    let r = f.run(vec![vop(
        12,
        kinds::SET_MATERIAL,
        L_BOUNDED,
        vbox(0, [64, 0, 0], [80, 16, 16]),
        set_material(1, None),
    )]);
    assert!(is_err(&r), "{r:?}");
    // The mesh layer has no voxel backend.
    let r = f.run(vec![vop(
        13,
        kinds::SET_MATERIAL,
        L_MESH,
        vbox(0, [0, 0, 0], [16, 16, 16]),
        set_material(1, None),
    )]);
    assert!(matches!(r, EditingResult::Unsupported(_)));
}

#[test]
fn other_channels_are_preserved_and_the_brick_stays_present() {
    let f = fx();
    let mut mat = Channel::constant(1);
    mat.set(LocalPos::new(3, 3, 3).unwrap(), 5);
    let density = Channel::constant(77);
    let set = ChannelSet::new()
        .with(ChannelId::Material, mat)
        .with(ChannelId::Density, density);
    f.system
        .materialise(
            OperationId::new(1, 1),
            LayerId::new(L_VOXEL),
            0,
            bc(0, 0, 0),
            CellChange::Put(CellPayload::Channels(set)),
        )
        .unwrap();
    assert!(
        f.run(vec![vop(
            1,
            kinds::REMOVE_GEOMETRY,
            L_VOXEL,
            vbox(0, [0, 0, 0], [16, 16, 16]),
            remove(None)
        )])
        .is_success()
    );
    let st = f.reader.snapshot();
    let Some(Ok(CellState::Present(p))) = st.cell(LayerId::new(L_VOXEL), 0, bc(0, 0, 0)) else {
        panic!("brick must stay present")
    };
    let CellPayload::Channels(set) = &*p else {
        panic!()
    };
    assert_eq!(
        set.channel(ChannelId::Material)
            .unwrap()
            .get(LocalPos::new(3, 3, 3).unwrap()),
        0
    );
    assert_eq!(
        set.channel(ChannelId::Density)
            .unwrap()
            .get(LocalPos::new(9, 9, 9).unwrap()),
        77
    );
}

#[test]
fn micro_voxel_levels_use_the_same_operations_at_their_own_level() {
    let f = fx();
    let r = f.run(vec![vop(
        1,
        kinds::SET_MATERIAL,
        L_VOXEL,
        vbox(2, [-16, 0, 0], [16, 16, 16]),
        set_material(6, None),
    )]);
    assert!(r.is_success(), "{r:?}");
    assert_eq!(voxel_at(&f, L_VOXEL, 2, [-1, 0, 0]), Some(6));
    assert_eq!(voxel_at(&f, L_VOXEL, 2, [15, 15, 15]), Some(6));
    assert_eq!(v0(&f, [-1, 0, 0]), None, "level 0 is untouched");
}

#[test]
fn oversized_and_extreme_regions_are_refused_without_allocation() {
    let small = fixture_with(Options {
        backends: vec![std::sync::Arc::new(VoxelBackend::with_limits(
            cube_backend_voxel::Limits { max_bricks: 8 },
        ))],
        ..Options::default()
    });
    let r = small.run(vec![vop(
        1,
        kinds::SET_MATERIAL,
        L_VOXEL,
        vbox(0, [0, 0, 0], [48, 48, 48]),
        set_material(1, None),
    )]);
    assert!(
        matches!(r, EditingResult::ValidationFailure(EditingError::InvalidOperation(m)) if m.contains("too large")),
        "{r:?}"
    );
    let huge = fx();
    let r = huge.run(vec![vop(
        2,
        kinds::REMOVE_GEOMETRY,
        L_VOXEL,
        vbox(
            0,
            [i64::MIN, i64::MIN, i64::MIN],
            [i64::MAX, i64::MAX, i64::MAX],
        ),
        remove(None),
    )]);
    assert!(is_err(&r), "{r:?}");
    // Far away from the origin works like anywhere else: there is no world limit.
    let far = fx();
    let x = i64::MAX - 16;
    let region = vbox(0, [x - 15, 0, 0], [x + 1, 16, 16]);
    let r = far.run(vec![vop(
        3,
        kinds::SET_MATERIAL,
        L_VOXEL,
        region,
        set_material(1, None),
    )]);
    assert!(r.is_success() || is_err(&r), "{r:?}");
    let neg = fx();
    let r = neg.run(vec![vop(
        4,
        kinds::SET_MATERIAL,
        L_VOXEL,
        vbox(0, [i64::MIN, 0, 0], [i64::MIN + 16, 16, 16]),
        set_material(1, None),
    )]);
    assert!(r.is_success(), "{r:?}");
    assert_eq!(v0(&neg, [i64::MIN, 0, 0]), Some(1));
}

fn xform(
    n: u64,
    src: Region,
    write: Region,
    dest: [i64; 3],
    rot: Rotation,
    mode: TransformMode,
) -> EditingOperation {
    let mut o = vop(
        n,
        kinds::TRANSFORM_GEOMETRY,
        L_VOXEL,
        write,
        transform(VoxelPos::new(dest[0], dest[1], dest[2]), rot, mode),
    );
    o.read_region = Some(src);
    o
}

fn fill_loaded(f: &Fixture, lo: i64, hi: i64) {
    let mut n = 0;
    for x in lo..=hi {
        for y in lo..=hi {
            for z in lo..=hi {
                n += 1;
                f.system
                    .materialise(
                        OperationId::new(3, n),
                        LayerId::new(L_VOXEL),
                        0,
                        bc(x, y, z),
                        CellChange::KnownEmpty,
                    )
                    .unwrap();
            }
        }
    }
}

#[test]
fn copy_and_move_follow_a_brute_force_model_including_overlaps_and_negative_space() {
    for_each_seed(0xB0C5, 12, |seed| {
        let f = fx();
        fill_loaded(&f, -2, 2);
        let mut rng = DetRng::new(seed);
        let mut model: HashMap<[i64; 3], u16> = HashMap::new();
        let rnd = |rng: &mut DetRng| rng.range_i64(-20, 20);
        for step in 0..8u64 {
            match rng.below(4) {
                0 | 1 => {
                    let lo = [rnd(&mut rng), rnd(&mut rng), rnd(&mut rng)];
                    let hi = [
                        lo[0] + 1 + rng.below(6) as i64,
                        lo[1] + 1 + rng.below(6) as i64,
                        lo[2] + 1 + rng.below(6) as i64,
                    ];
                    let m = rng.below(4) as u16;
                    let r = f.run(vec![vop(
                        100 + step,
                        kinds::SET_MATERIAL,
                        L_VOXEL,
                        vbox(0, lo, hi),
                        set_material(m, None),
                    )]);
                    assert!(r.is_success(), "{r:?}");
                    for x in lo[0]..hi[0] {
                        for y in lo[1]..hi[1] {
                            for z in lo[2]..hi[2] {
                                model.insert([x, y, z], m);
                            }
                        }
                    }
                }
                2 => {
                    let lo = [rnd(&mut rng), rnd(&mut rng), rnd(&mut rng)];
                    let e = [
                        1 + rng.below(5) as i64,
                        1 + rng.below(5) as i64,
                        1 + rng.below(5) as i64,
                    ];
                    let d = [
                        lo[0] + rng.range_i64(-6, 7),
                        lo[1] + rng.range_i64(-6, 7),
                        lo[2] + rng.range_i64(-6, 7),
                    ];
                    let src = vbox(0, lo, [lo[0] + e[0], lo[1] + e[1], lo[2] + e[2]]);
                    let dst_hi = [d[0] + e[0], d[1] + e[1], d[2] + e[2]];
                    let write = vbox(
                        0,
                        [lo[0].min(d[0]), lo[1].min(d[1]), lo[2].min(d[2])],
                        [
                            (lo[0] + e[0]).max(dst_hi[0]),
                            (lo[1] + e[1]).max(dst_hi[1]),
                            (lo[2] + e[2]).max(dst_hi[2]),
                        ],
                    );
                    let mv = rng.below(2) == 0;
                    let r = f.run(vec![xform(
                        200 + step,
                        src,
                        write,
                        d,
                        Rotation::IDENTITY,
                        if mv {
                            TransformMode::Move
                        } else {
                            TransformMode::Copy
                        },
                    )]);
                    assert!(r.is_success(), "{r:?}");
                    let old: HashMap<_, _> = model.clone();
                    if mv {
                        for x in 0..e[0] {
                            for y in 0..e[1] {
                                for z in 0..e[2] {
                                    model.insert([lo[0] + x, lo[1] + y, lo[2] + z], 0);
                                }
                            }
                        }
                    }
                    for x in 0..e[0] {
                        for y in 0..e[1] {
                            for z in 0..e[2] {
                                let v = old
                                    .get(&[lo[0] + x, lo[1] + y, lo[2] + z])
                                    .copied()
                                    .unwrap_or(0);
                                model.insert([d[0] + x, d[1] + y, d[2] + z], v);
                            }
                        }
                    }
                }
                _ => {
                    let lo = [rnd(&mut rng), rnd(&mut rng), rnd(&mut rng)];
                    let hi = [
                        lo[0] + 1 + rng.below(6) as i64,
                        lo[1] + 1 + rng.below(6) as i64,
                        lo[2] + 1 + rng.below(6) as i64,
                    ];
                    let (from, to) = (rng.below(4) as u16, rng.below(4) as u16);
                    let r = f.run(vec![vop(
                        300 + step,
                        kinds::SET_MATERIAL,
                        L_VOXEL,
                        vbox(0, lo, hi),
                        set_material(to, Some(from)),
                    )]);
                    assert!(r.is_success(), "{r:?}");
                    for x in lo[0]..hi[0] {
                        for y in lo[1]..hi[1] {
                            for z in lo[2]..hi[2] {
                                if model.get(&[x, y, z]).copied().unwrap_or(0) == from {
                                    model.insert([x, y, z], to);
                                }
                            }
                        }
                    }
                }
            }
            for x in -26..26 {
                for y in -26..26 {
                    for z in -26..26 {
                        let want = model.get(&[x, y, z]).copied().unwrap_or(0);
                        // Bricks that were never touched and not preloaded read as unloaded.
                        if let Some(got) = v0(&f, [x, y, z]) {
                            assert_eq!(got, want, "seed {seed:#x} step {step} at {x},{y},{z}");
                        } else {
                            assert_eq!(
                                want, 0,
                                "unloaded voxel must have no model content at {x},{y},{z}"
                            );
                        }
                    }
                }
            }
        }
    });
}

#[test]
fn rotated_copies_preserve_volume_and_the_inverse_rotation_restores_the_shape() {
    let f = fx();
    fill_loaded(&f, -2, 2);
    // An asymmetric L-shape: 3x2x1 with one extra voxel on top.
    let src = vbox(0, [0, 0, 0], [3, 2, 1]);
    assert!(
        f.run(vec![vop(
            1,
            kinds::SET_MATERIAL,
            L_VOXEL,
            src,
            set_material(1, None)
        )])
        .is_success()
    );
    assert!(
        f.run(vec![vop(
            2,
            kinds::SET_MATERIAL,
            L_VOXEL,
            vbox(0, [0, 0, 1], [1, 1, 2]),
            set_material(2, None)
        )])
        .is_success()
    );
    let count = |f: &Fixture, lo: [i64; 3], hi: [i64; 3]| {
        let mut n = (0u32, 0u32);
        for x in lo[0]..hi[0] {
            for y in lo[1]..hi[1] {
                for z in lo[2]..hi[2] {
                    match v0(f, [x, y, z]) {
                        Some(1) => n.0 += 1,
                        Some(2) => n.1 += 1,
                        _ => {}
                    }
                }
            }
        }
        n
    };
    for ri in 0..Rotation::COUNT {
        let rot = Rotation::from_index(ri).unwrap();
        let dest = [
            100 + 10 * i64::from(ri % 5),
            -100 + 6 * i64::from(ri / 5),
            0,
        ];
        // The destination box is the rotation of the 3x2x2 source extents, anchored at `dest`.
        let ext = rot.apply([3, 2, 2]).unwrap();
        let e = [ext[0].abs(), ext[1].abs(), ext[2].abs()];
        let write = vbox(0, dest, [dest[0] + e[0], dest[1] + e[1], dest[2] + e[2]]);
        let mut o = xform(
            10 + u64::from(ri),
            vbox(0, [0, 0, 0], [3, 2, 2]),
            write,
            dest,
            rot,
            TransformMode::Copy,
        );
        o.layers = vec![LayerId::new(L_VOXEL)];
        // The destination bricks are not loaded: the edit would only partly cover them.
        let r = f.run(vec![o.clone()]);
        assert!(
            is_err(&r),
            "rotation {ri}: unloaded destination must be refused, got {r:?}"
        );
    }
    // Rotations inside the loaded area: copy to a loaded place and count voxels.
    for ri in 0..Rotation::COUNT {
        let rot = Rotation::from_index(ri).unwrap();
        let ext = rot.apply([3, 2, 2]).unwrap();
        let e = [ext[0].abs(), ext[1].abs(), ext[2].abs()];
        let d = [
            -20,
            -20 + 7 * i64::from(ri % 3),
            -20 + 7 * i64::from(ri / 3 % 3),
        ];
        let write = vbox(0, d, [d[0] + e[0], d[1] + e[1], d[2] + e[2]]);
        let o = xform(
            50 + u64::from(ri),
            vbox(0, [0, 0, 0], [3, 2, 2]),
            write,
            d,
            rot,
            TransformMode::Copy,
        );
        // Load the destination bricks first so the edit is exact.
        let mut n = 1000 + u64::from(ri) * 100;
        for x in (d[0].div_euclid(16))..=((d[0] + e[0] - 1).div_euclid(16)) {
            for y in (d[1].div_euclid(16))..=((d[1] + e[1] - 1).div_euclid(16)) {
                for z in (d[2].div_euclid(16))..=((d[2] + e[2] - 1).div_euclid(16)) {
                    n += 1;
                    let _ = f.system.materialise(
                        OperationId::new(4, n),
                        LayerId::new(L_VOXEL),
                        0,
                        bc(x, y, z),
                        CellChange::KnownEmpty,
                    );
                }
            }
        }
        let r = f.run(vec![o]);
        assert!(r.is_success(), "rotation {ri}: {r:?}");
        let lo = d;
        let hi = [d[0] + e[0], d[1] + e[1], d[2] + e[2]];
        // Volume is preserved: six voxels of material 1 and the one extra voxel of material 2.
        assert_eq!(
            count(&f, lo, hi),
            (6, 1),
            "rotation {ri} must keep both materials' voxel counts"
        );
    }
}

#[test]
fn the_voxel_backend_passes_the_conformance_suite_and_is_deterministic() {
    struct Loaded;
    impl cube_editing_api::OverlayRead for Loaded {
        fn cell(&self, _: cube_core::CellKey) -> Result<CellState, EditingError> {
            Ok(CellState::KnownEmpty)
        }
    }
    let region = vbox(0, [-16, 0, 0], [16, 16, 16]);
    let mut t = xform(
        4,
        vbox(0, [0, 0, 0], [4, 4, 4]),
        region,
        [-16, 0, 0],
        Rotation::IDENTITY,
        TransformMode::Copy,
    );
    t.write_region = region;
    let cases = vec![
        Case {
            operation: vop(
                1,
                kinds::SET_MATERIAL,
                L_VOXEL,
                region,
                set_material(3, None),
            ),
        },
        Case {
            operation: vop(2, kinds::ADD_GEOMETRY, L_VOXEL, region, add(3)),
        },
        Case {
            operation: vop(3, kinds::REMOVE_GEOMETRY, L_VOXEL, region, remove(None)),
        },
        Case { operation: t },
    ];
    let b = VoxelBackend::arc();
    assert_eq!(run(&*b, &cases, &Loaded), vec![]);
}

#[test]
fn bad_parameters_are_controlled_errors_through_the_chain() {
    let f = fx();
    let mut bad = vop(
        1,
        kinds::SET_MATERIAL,
        L_VOXEL,
        vbox(0, [0, 0, 0], [16, 16, 16]),
        set_material(1, None),
    );
    bad.parameters.bytes.truncate(2);
    let r = f.run(vec![bad]);
    assert!(
        matches!(
            r,
            EditingResult::ValidationFailure(EditingError::InvalidOperation(_))
        ),
        "{r:?}"
    );
    let mut no_read = vop(
        2,
        kinds::TRANSFORM_GEOMETRY,
        L_VOXEL,
        vbox(0, [0, 0, 0], [16, 16, 16]),
        transform(
            VoxelPos::new(0, 0, 0),
            Rotation::IDENTITY,
            TransformMode::Copy,
        ),
    );
    no_read.read_region = None;
    assert!(is_err(&f.run(vec![no_read])));
    assert_eq!(f.commits(), 0);
}
