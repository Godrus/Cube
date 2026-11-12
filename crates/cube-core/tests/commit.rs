//! Behaviour of the commit gate and the mutation boundary through the public API only.

use std::collections::HashMap;
use std::sync::Arc;
use std::thread;

use cube_core::{
    ActorKind, AssetHost, AssetId, BrickBounds, BrickCoord, CellChange, CellPayload, CellState,
    CellStore, CellWrite, ChangeType, Channel, ChannelId, ChannelSet, CommitBatch, CommitError,
    Comparison, CoreError, ExpectedRevision, GeometryAsset, GeometryTransform, Handles, Layer,
    LayerData, LayerId, LayerKind, MaterialIndex, OperationId, Revision, SharedAsset, WriteSet,
};
use cube_testkit::{DetRng, for_each_seed};

const L_VOXEL: u64 = 1;
const L_SDF: u64 = 2;
const L_MESH: u64 = 3;
const L_BOUNDED: u64 = 4;

fn host() -> AssetHost {
    let bounds = BrickBounds::new(BrickCoord::new(-2, -2, -2), BrickCoord::new(2, 2, 2)).unwrap();
    let layers = vec![
        Layer::new(
            LayerId::new(L_VOXEL),
            GeometryTransform::IDENTITY,
            LayerData::empty(LayerKind::Voxel),
        ),
        Layer::new(
            LayerId::new(L_SDF),
            GeometryTransform::IDENTITY,
            LayerData::empty(LayerKind::Sdf),
        ),
        Layer::new(
            LayerId::new(L_MESH),
            GeometryTransform::IDENTITY,
            LayerData::empty(LayerKind::Mesh),
        ),
        Layer::new(
            LayerId::new(L_BOUNDED),
            GeometryTransform::IDENTITY,
            LayerData::Voxel(CellStore::with_bounds(bounds)),
        ),
    ];
    AssetHost::new(GeometryAsset::new(AssetId::new(1), layers).unwrap())
}

fn mat(m: u16) -> CellPayload {
    CellPayload::Channels(ChannelSet::uniform_material(MaterialIndex::from_raw(
        u32::from(m),
    )))
}
fn b(x: i64, y: i64, z: i64) -> BrickCoord {
    BrickCoord::new(x, y, z)
}
fn write(
    level: i8,
    brick: BrickCoord,
    expected: ExpectedRevision,
    change: CellChange,
) -> CellWrite {
    CellWrite {
        level,
        brick,
        expected,
        change,
    }
}
fn batch(layer: u64, writes: Vec<CellWrite>) -> CommitBatch {
    CommitBatch {
        sets: vec![WriteSet {
            layer: LayerId::new(layer),
            writes,
        }],
    }
}
fn op(n: u64) -> OperationId {
    OperationId::new(1, n)
}
fn put_absent(brick: BrickCoord, m: u16) -> CellWrite {
    write(0, brick, ExpectedRevision::Absent, CellChange::Put(mat(m)))
}
fn state(r: &SharedAsset, layer: u64, brick: BrickCoord) -> CellState {
    r.snapshot()
        .cell(LayerId::new(layer), 0, brick)
        .unwrap()
        .unwrap()
}

#[test]
fn first_commit_sets_revisions_and_reports_exact_dirty_region() {
    let h = host();
    let r = h.reader();
    let Handles { user, .. } = h.into_handles();
    let out = user
        .commit(op(1), &batch(L_VOXEL, vec![put_absent(b(-1, 0, 2), 5)]))
        .unwrap();
    assert!(out.changed);
    assert_eq!(out.asset_token.value(), Revision::new(1));
    assert_eq!(out.layer_tokens.len(), 1);
    assert_eq!(out.layer_tokens[0].value(), Revision::new(1));
    assert_eq!(out.cell_tokens[0].value(), Revision::new(1));
    let d = out.dirty[0];
    assert_eq!(
        (d.region.min().x, d.region.min().y, d.region.min().z),
        (-16, 0, 32)
    );
    assert_eq!(
        (d.region.max().x, d.region.max().y, d.region.max().z),
        (0, 16, 48)
    );
    assert_eq!(
        (d.actor, d.operation, d.change, d.source),
        (
            ActorKind::User,
            op(1),
            ChangeType::Geometry,
            out.asset_token
        )
    );
    assert!(matches!(
        state(&r, L_VOXEL, b(-1, 0, 2)),
        CellState::Present(_)
    ));
    assert_eq!(state(&r, L_VOXEL, b(0, 0, 0)), CellState::NotLoaded);
}

#[test]
fn conflict_is_all_or_nothing_and_lists_every_conflicting_cell() {
    let h = host();
    let r = h.reader();
    let user = h.into_handles().user;
    user.commit(op(1), &batch(L_VOXEL, vec![put_absent(b(0, 0, 0), 1)]))
        .unwrap();
    let before = r.snapshot();
    // One good write and two conflicting ones (wrong expectation on an existing cell, wrong At on a missing one).
    let bad = batch(
        L_VOXEL,
        vec![
            put_absent(b(5, 5, 5), 9),
            write(
                0,
                b(0, 0, 0),
                ExpectedRevision::Absent,
                CellChange::Put(mat(2)),
            ),
            write(
                0,
                b(7, 7, 7),
                ExpectedRevision::At(Revision::new(3)),
                CellChange::KnownEmpty,
            ),
        ],
    );
    let Err(CommitError::Conflict(c)) = user.commit(op(2), &bad) else {
        panic!("expected conflict")
    };
    assert_eq!(c.len(), 2);
    assert_eq!(c[0].actual, Some(Revision::new(1)));
    assert_eq!(c[1].actual, None);
    let after = r.snapshot();
    assert_eq!(*before, *after, "a rejected commit changes nothing");
    assert_eq!(state(&r, L_VOXEL, b(5, 5, 5)), CellState::NotLoaded);
}

#[test]
fn multi_layer_batch_is_atomic_across_layers() {
    let h = host();
    let r = h.reader();
    let user = h.into_handles().user;
    user.commit(op(1), &batch(L_SDF, vec![put_absent(b(0, 0, 0), 1)]))
        .unwrap();
    let both = CommitBatch {
        sets: vec![
            WriteSet {
                layer: LayerId::new(L_VOXEL),
                writes: vec![put_absent(b(0, 0, 0), 7)],
            },
            WriteSet {
                layer: LayerId::new(L_SDF),
                writes: vec![put_absent(b(0, 0, 0), 8)],
            },
        ],
    };
    assert!(matches!(
        user.commit(op(2), &both),
        Err(CommitError::Conflict(_))
    ));
    assert_eq!(
        state(&r, L_VOXEL, b(0, 0, 0)),
        CellState::NotLoaded,
        "the valid half was not applied"
    );
    // The same batch with a correct expectation commits both layers in one step: asset +1, each layer +1.
    let ok = CommitBatch {
        sets: vec![
            WriteSet {
                layer: LayerId::new(L_VOXEL),
                writes: vec![put_absent(b(0, 0, 0), 7)],
            },
            WriteSet {
                layer: LayerId::new(L_SDF),
                writes: vec![write(
                    0,
                    b(0, 0, 0),
                    ExpectedRevision::At(Revision::new(1)),
                    CellChange::Put(mat(8)),
                )],
            },
        ],
    };
    let out = user.commit(op(3), &ok).unwrap();
    assert_eq!(out.asset_token.value(), Revision::new(2));
    assert_eq!(out.layer_tokens.len(), 2);
}

#[test]
fn snapshots_are_isolated_from_later_commits() {
    let h = host();
    let r = h.reader();
    let user = h.into_handles().user;
    user.commit(op(1), &batch(L_VOXEL, vec![put_absent(b(0, 0, 0), 1)]))
        .unwrap();
    let snap = r.snapshot();
    let token = snap.asset_token();
    user.commit(
        op(2),
        &batch(
            L_VOXEL,
            vec![write(
                0,
                b(0, 0, 0),
                ExpectedRevision::At(Revision::new(1)),
                CellChange::Put(mat(2)),
            )],
        ),
    )
    .unwrap();
    assert_eq!(snap.asset_token(), token);
    let CellState::Present(p) = snap
        .cell(LayerId::new(L_VOXEL), 0, b(0, 0, 0))
        .unwrap()
        .unwrap()
    else {
        panic!()
    };
    assert_eq!(*p, mat(1), "the old snapshot still sees the old payload");
    assert_eq!(
        r.snapshot().asset_token().compare(&token),
        Comparison::Newer
    );
}

#[test]
fn writes_that_change_nothing_move_no_token_and_make_no_dirty_region() {
    let h = host();
    let r = h.reader();
    let user = h.into_handles().user;
    user.commit(
        op(1),
        &batch(
            L_VOXEL,
            vec![
                put_absent(b(0, 0, 0), 1),
                write(
                    0,
                    b(1, 0, 0),
                    ExpectedRevision::Absent,
                    CellChange::KnownEmpty,
                ),
            ],
        ),
    )
    .unwrap();
    let before = r.snapshot();
    let same = batch(
        L_VOXEL,
        vec![
            write(
                0,
                b(0, 0, 0),
                ExpectedRevision::At(Revision::new(1)),
                CellChange::Put(mat(1)),
            ),
            write(
                0,
                b(1, 0, 0),
                ExpectedRevision::At(Revision::new(1)),
                CellChange::KnownEmpty,
            ),
        ],
    );
    let out = user.commit(op(2), &same).unwrap();
    assert!(
        !out.changed
            && out.dirty.is_empty()
            && out.cell_tokens.is_empty()
            && out.layer_tokens.is_empty()
    );
    assert_eq!(*before, *r.snapshot());
    assert_eq!(r.commit_log().0, 1, "no log entry for a no-op");
    // An empty batch is a no-op as well.
    assert!(!user.commit(op(3), &CommitBatch::default()).unwrap().changed);
}

#[test]
fn revision_increments_are_exactly_plus_one_per_scope() {
    let h = host();
    let user = h.into_handles().user;
    let many: Vec<CellWrite> = (0..5).map(|i| put_absent(b(i, 0, 0), 1)).collect();
    let out = user.commit(op(1), &batch(L_VOXEL, many)).unwrap();
    assert_eq!(out.asset_token.value(), Revision::new(1));
    assert_eq!(
        out.layer_tokens[0].value(),
        Revision::new(1),
        "+1 per layer however many cells changed"
    );
    assert_eq!(out.cell_tokens.len(), 5);
    let again = user
        .commit(
            op(2),
            &batch(
                L_VOXEL,
                vec![write(
                    0,
                    b(2, 0, 0),
                    ExpectedRevision::At(Revision::new(1)),
                    CellChange::KnownEmpty,
                )],
            ),
        )
        .unwrap();
    assert_eq!(
        (
            again.asset_token.value(),
            again.layer_tokens[0].value(),
            again.cell_tokens[0].value()
        ),
        (Revision::new(2), Revision::new(2), Revision::new(2))
    );
}

#[test]
fn change_type_distinguishes_material_attribute_and_geometry() {
    let h = host();
    let user = h.into_handles().user;
    let base = ChannelSet::new()
        .with(ChannelId::Material, Channel::constant(1))
        .with(ChannelId::Attribute(0), Channel::constant(0));
    user.commit(
        op(1),
        &batch(
            L_VOXEL,
            vec![write(
                0,
                b(0, 0, 0),
                ExpectedRevision::Absent,
                CellChange::Put(CellPayload::Channels(base.clone())),
            )],
        ),
    )
    .unwrap();
    let change = |n: u64, rev: u64, set: ChannelSet| {
        user.commit(
            op(n),
            &batch(
                L_VOXEL,
                vec![write(
                    0,
                    b(0, 0, 0),
                    ExpectedRevision::At(Revision::new(rev)),
                    CellChange::Put(CellPayload::Channels(set)),
                )],
            ),
        )
        .unwrap()
        .dirty[0]
            .change
    };
    let mat_only = base.clone().with(ChannelId::Material, Channel::constant(2));
    assert_eq!(change(2, 1, mat_only.clone()), ChangeType::Material);
    let attr_only = mat_only
        .clone()
        .with(ChannelId::Attribute(0), Channel::constant(9));
    assert_eq!(change(3, 2, attr_only.clone()), ChangeType::Attribute);
    let both = attr_only
        .with(ChannelId::Material, Channel::constant(3))
        .with(ChannelId::Attribute(0), Channel::constant(1));
    assert_eq!(change(4, 3, both), ChangeType::Geometry);
}

#[test]
fn invalid_batches_are_rejected_with_specific_errors_and_change_nothing() {
    let h = host();
    let r = h.reader();
    let user = h.into_handles().user;
    let tok = r.snapshot().asset_token();
    let unknown = user.commit(op(1), &batch(99, vec![put_absent(b(0, 0, 0), 1)]));
    assert_eq!(unknown, Err(CommitError::UnknownLayer(LayerId::new(99))));
    let mesh = user.commit(op(2), &batch(L_MESH, vec![put_absent(b(0, 0, 0), 1)]));
    assert_eq!(
        mesh,
        Err(CommitError::PayloadKindMismatch(LayerId::new(L_MESH)))
    );
    let page = user.commit(
        op(3),
        &batch(
            L_VOXEL,
            vec![write(
                0,
                b(0, 0, 0),
                ExpectedRevision::Absent,
                CellChange::Put(CellPayload::MeshPage(Default::default())),
            )],
        ),
    );
    assert_eq!(
        page,
        Err(CommitError::PayloadKindMismatch(LayerId::new(L_VOXEL)))
    );
    let outside = user.commit(op(4), &batch(L_BOUNDED, vec![put_absent(b(2, 0, 0), 1)]));
    assert_eq!(
        outside,
        Err(CommitError::OutsideBounds {
            layer: LayerId::new(L_BOUNDED),
            level: 0,
            brick: b(2, 0, 0)
        })
    );
    let dup = user.commit(
        op(5),
        &batch(
            L_VOXEL,
            vec![put_absent(b(1, 1, 1), 1), put_absent(b(1, 1, 1), 2)],
        ),
    );
    assert_eq!(
        dup,
        Err(CommitError::DuplicateWrite {
            layer: LayerId::new(L_VOXEL),
            level: 0,
            brick: b(1, 1, 1)
        })
    );
    let level = user.commit(
        op(6),
        &batch(
            L_VOXEL,
            vec![write(
                -1,
                b(0, 0, 0),
                ExpectedRevision::Absent,
                CellChange::KnownEmpty,
            )],
        ),
    );
    assert_eq!(level, Err(CommitError::Core(CoreError::InvalidLevel(-1))));
    assert_eq!(r.snapshot().asset_token(), tok);
    assert_eq!(r.commit_log().0, 0);
    // Bounds hold for refined levels too: level-2 brick (8,0,0) is in base brick 2 (outside).
    let refined = user.commit(
        op(7),
        &batch(
            L_BOUNDED,
            vec![write(
                2,
                b(8, 0, 0),
                ExpectedRevision::Absent,
                CellChange::KnownEmpty,
            )],
        ),
    );
    assert!(matches!(refined, Err(CommitError::OutsideBounds { .. })));
    assert!(
        user.commit(
            op(8),
            &batch(
                L_BOUNDED,
                vec![write(
                    2,
                    b(7, 0, 0),
                    ExpectedRevision::Absent,
                    CellChange::KnownEmpty
                )]
            )
        )
        .unwrap()
        .changed,
        "level-2 brick 7 is in base brick 1"
    );
}

#[test]
fn handles_fix_the_actor_and_the_log_is_complete() {
    let h = host();
    let r = h.reader();
    let Handles {
        user,
        system,
        recovery,
    } = h.into_handles();
    let s = system
        .commit(op(1), &batch(L_VOXEL, vec![put_absent(b(0, 0, 0), 1)]))
        .unwrap();
    let u = user
        .commit(op(2), &batch(L_VOXEL, vec![put_absent(b(1, 0, 0), 1)]))
        .unwrap();
    let rc = recovery
        .commit(op(3), &batch(L_VOXEL, vec![put_absent(b(2, 0, 0), 1)]))
        .unwrap();
    assert_eq!(
        [s.dirty[0].actor, u.dirty[0].actor, rc.dirty[0].actor],
        [ActorKind::System, ActorKind::User, ActorKind::Recovery]
    );
    let (total, log) = r.commit_log();
    assert_eq!(
        total,
        r.snapshot().asset_token().value().raw(),
        "every asset revision increment has a log entry"
    );
    assert_eq!(
        log.iter().map(|e| e.actor).collect::<Vec<_>>(),
        [ActorKind::System, ActorKind::User, ActorKind::Recovery]
    );
    assert!(
        log.windows(2)
            .all(|w| w[0].asset_token.compare(&w[1].asset_token) == Comparison::Older)
    );
    assert_eq!(
        log.iter().map(|e| e.operation).collect::<Vec<_>>(),
        [op(1), op(2), op(3)]
    );
}

#[test]
fn exactly_one_of_many_concurrent_writers_to_the_same_cell_wins() {
    let h = host();
    let r = h.reader();
    let user = Arc::new(h.into_handles().user);
    let threads: Vec<_> = (0..8)
        .map(|i| {
            let user = Arc::clone(&user);
            thread::spawn(move || {
                user.commit(
                    op(i),
                    &batch(L_VOXEL, vec![put_absent(b(0, 0, 0), i as u16 + 1)]),
                )
            })
        })
        .collect();
    let results: Vec<_> = threads.into_iter().map(|t| t.join().unwrap()).collect();
    assert_eq!(results.iter().filter(|x| x.is_ok()).count(), 1);
    assert!(
        results
            .iter()
            .filter_map(|x| x.as_ref().err())
            .all(|e| matches!(e, CommitError::Conflict(_)))
    );
    assert_eq!(r.snapshot().asset_token().value(), Revision::new(1));
}

#[test]
fn concurrent_writers_to_different_cells_all_succeed_with_dense_revisions() {
    let h = host();
    let r = h.reader();
    let user = Arc::new(h.into_handles().user);
    let threads: Vec<_> = (0..8)
        .map(|i| {
            let user = Arc::clone(&user);
            thread::spawn(move || {
                for j in 0..20 {
                    user.commit(
                        op(i * 100 + j),
                        &batch(L_VOXEL, vec![put_absent(b(i as i64, j as i64, 0), 1)]),
                    )
                    .unwrap();
                }
            })
        })
        .collect();
    for t in threads {
        t.join().unwrap();
    }
    let snap = r.snapshot();
    assert_eq!(snap.asset_token().value(), Revision::new(160));
    assert_eq!(
        snap.layer_token(LayerId::new(L_VOXEL)).unwrap().value(),
        Revision::new(160)
    );
    let (total, log) = r.commit_log();
    assert_eq!((total, log.len()), (160, 160));
    let seqs: Vec<u64> = log.iter().map(|e| e.asset_token.value().raw()).collect();
    assert_eq!(
        seqs,
        (1..=160).collect::<Vec<u64>>(),
        "the gate serialises writers: revisions are consecutive"
    );
}

#[test]
fn random_commit_sequences_match_a_reference_model() {
    for_each_seed(31, 60, |seed| {
        let mut rng = DetRng::new(seed);
        let h = host();
        let r = h.reader();
        let user = h.into_handles().user;
        // model: brick -> (payload material or None for known-empty, revision)
        let mut model: HashMap<BrickCoord, (Option<u16>, u64)> = HashMap::new();
        let mut asset_rev = 0u64;
        for n in 0..40 {
            let brick = b(rng.range_i64(-2, 2), rng.range_i64(-2, 2), 0);
            let current = model.get(&brick).copied();
            let wrong = rng.below(5) == 0;
            let expected = match (current, wrong) {
                (None, false) => ExpectedRevision::Absent,
                (Some((_, rev)), false) => ExpectedRevision::At(Revision::new(rev)),
                (None, true) => ExpectedRevision::At(Revision::new(1)),
                (Some(_), true) => ExpectedRevision::Absent,
            };
            let new_mat = if rng.below(4) == 0 {
                None
            } else {
                Some(rng.below(4) as u16)
            };
            let change = new_mat.map_or(CellChange::KnownEmpty, |m| CellChange::Put(mat(m)));
            let res = user.commit(
                op(n),
                &batch(L_VOXEL, vec![write(0, brick, expected, change)]),
            );
            if wrong {
                assert!(matches!(res, Err(CommitError::Conflict(_))));
            } else {
                let out = res.unwrap();
                let same = current.is_some_and(|(m, _)| m == new_mat);
                assert_eq!(out.changed, !same);
                if !same {
                    asset_rev += 1;
                    model.insert(brick, (new_mat, current.map_or(1, |(_, r)| r + 1)));
                }
            }
            let snap = r.snapshot();
            assert_eq!(snap.asset_token().value().raw(), asset_rev);
            for (k, (m, rev)) in &model {
                assert_eq!(
                    snap.cell_token(LayerId::new(L_VOXEL), 0, *k)
                        .unwrap()
                        .value()
                        .raw(),
                    *rev
                );
                let st = snap.cell(LayerId::new(L_VOXEL), 0, *k).unwrap().unwrap();
                match m {
                    None => assert_eq!(st, CellState::KnownEmpty),
                    Some(v) => assert_eq!(st, CellState::Present(Arc::new(mat(*v)))),
                }
            }
        }
        assert_eq!(r.commit_log().0, asset_rev);
    });
}
