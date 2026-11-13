//! System generation (materialise / evict) and recovery restore through the handles (CORE-09).

use cube_core::{
    ActorKind, AssetHost, AssetId, BrickBounds, BrickCoord, CellChange, CellPayload, CellState,
    CellStore, CellWrite, ChannelSet, CommitBatch, CommitError, ExpectedRevision, GeometryAsset,
    GeometryTransform, Layer, LayerData, LayerId, LayerKind, MaterialIndex, MaterialiseOutcome,
    OperationId, Revision, WriteSet,
};

const L: u64 = 1;
const LB: u64 = 2;

fn host() -> AssetHost {
    let bounds = BrickBounds::new(BrickCoord::new(0, 0, 0), BrickCoord::new(4, 4, 4)).unwrap();
    let layers = vec![
        Layer::new(
            LayerId::new(L),
            GeometryTransform::IDENTITY,
            LayerData::empty(LayerKind::Voxel),
        ),
        Layer::new(
            LayerId::new(LB),
            GeometryTransform::IDENTITY,
            LayerData::Voxel(CellStore::with_bounds(bounds)),
        ),
        Layer::new(
            LayerId::new(3),
            GeometryTransform::IDENTITY,
            LayerData::empty(LayerKind::Mesh),
        ),
    ];
    AssetHost::new(GeometryAsset::new(AssetId::new(1), layers).unwrap())
}
fn mat(m: u16) -> CellChange {
    CellChange::Put(CellPayload::Channels(ChannelSet::uniform_material(
        MaterialIndex::from_raw(u32::from(m)),
    )))
}
fn b(x: i64) -> BrickCoord {
    BrickCoord::new(x, 0, 0)
}
fn op(n: u64) -> OperationId {
    OperationId::new(9, n)
}

#[test]
fn materialise_writes_once_and_is_idempotent() {
    let h = host();
    let r = h.reader();
    let sys = h.into_handles().system;
    let MaterialiseOutcome::Written(out) = sys
        .materialise(op(1), LayerId::new(L), 0, b(0), mat(1))
        .unwrap()
    else {
        panic!("expected a write")
    };
    assert_eq!(out.dirty[0].actor, ActorKind::System);
    assert_eq!(r.snapshot().asset_token().value(), Revision::new(1));
    let again = sys
        .materialise(op(2), LayerId::new(L), 0, b(0), mat(1))
        .unwrap();
    assert_eq!(again, MaterialiseOutcome::AlreadyLoaded);
    assert_eq!(
        r.snapshot().asset_token().value(),
        Revision::new(1),
        "a repeated materialise changes nothing"
    );
    assert_eq!(r.commit_log().0, 1);
}

#[test]
fn materialise_never_overwrites_a_user_edit() {
    let h = host();
    let r = h.reader();
    let handles = h.into_handles();
    let w = |m| CommitBatch {
        sets: vec![WriteSet {
            layer: LayerId::new(L),
            writes: vec![CellWrite {
                level: 0,
                brick: b(0),
                expected: ExpectedRevision::Absent,
                change: mat(m),
            }],
        }],
    };
    handles.user.commit(op(1), &w(7)).unwrap();
    assert_eq!(
        handles
            .system
            .materialise(op(2), LayerId::new(L), 0, b(0), mat(1))
            .unwrap(),
        MaterialiseOutcome::AlreadyLoaded
    );
    let CellState::Present(p) = r
        .snapshot()
        .cell(LayerId::new(L), 0, b(0))
        .unwrap()
        .unwrap()
    else {
        panic!()
    };
    assert_eq!(
        *p,
        CellPayload::Channels(ChannelSet::uniform_material(MaterialIndex::from_raw(7)))
    );
}

#[test]
fn evict_unloads_without_touching_revisions_and_materialise_reloads_with_plus_one() {
    let h = host();
    let r = h.reader();
    let sys = h.into_handles().system;
    sys.materialise(op(1), LayerId::new(L), 0, b(3), mat(5))
        .unwrap();
    let before = r.snapshot();
    assert_eq!(sys.evict(LayerId::new(L), 0, b(3)), Ok(true));
    let after = r.snapshot();
    assert_eq!(
        after.cell(LayerId::new(L), 0, b(3)).unwrap().unwrap(),
        CellState::NotLoaded
    );
    assert_eq!(
        after.asset_token(),
        before.asset_token(),
        "eviction moves no asset token"
    );
    assert_eq!(
        after.cell_token(LayerId::new(L), 0, b(3)),
        before.cell_token(LayerId::new(L), 0, b(3)),
        "the cell's revision survives eviction"
    );
    assert!(
        matches!(
            before.cell(LayerId::new(L), 0, b(3)).unwrap().unwrap(),
            CellState::Present(_)
        ),
        "old snapshots are unaffected"
    );
    assert_eq!(
        r.commit_log().0,
        1,
        "no log entry without a revision increment"
    );
    assert_eq!(
        sys.evict(LayerId::new(L), 0, b(3)),
        Ok(false),
        "evicting an unloaded cell is a no-op"
    );
    // Reloading continues the revision sequence: 1 -> 2, never reused.
    let MaterialiseOutcome::Written(out) = sys
        .materialise(op(2), LayerId::new(L), 0, b(3), mat(5))
        .unwrap()
    else {
        panic!()
    };
    assert_eq!(out.cell_tokens[0].value(), Revision::new(2));
    assert_eq!(r.commit_log().0, 2);
}

#[test]
fn system_ports_report_precise_errors() {
    let h = host();
    let sys = h.into_handles().system;
    assert_eq!(
        sys.materialise(op(1), LayerId::new(99), 0, b(0), mat(1)),
        Err(CommitError::UnknownLayer(LayerId::new(99)))
    );
    assert_eq!(
        sys.materialise(op(2), LayerId::new(3), 0, b(0), mat(1)),
        Err(CommitError::PayloadKindMismatch(LayerId::new(3)))
    );
    assert_eq!(
        sys.materialise(op(3), LayerId::new(LB), 0, b(9), mat(1)),
        Err(CommitError::OutsideBounds {
            layer: LayerId::new(LB),
            level: 0,
            brick: b(9)
        })
    );
    assert!(
        sys.materialise(op(4), LayerId::new(L), -1, b(0), mat(1))
            .is_err()
    );
    assert_eq!(
        sys.evict(LayerId::new(99), 0, b(0)),
        Err(CommitError::UnknownLayer(LayerId::new(99)))
    );
}

#[test]
fn recovery_restore_commits_with_the_recovery_actor() {
    let h = host();
    let r = h.reader();
    let rec = h.into_handles().recovery;
    let batch = CommitBatch {
        sets: vec![WriteSet {
            layer: LayerId::new(L),
            writes: vec![CellWrite {
                level: 0,
                brick: b(1),
                expected: ExpectedRevision::Absent,
                change: mat(2),
            }],
        }],
    };
    let out = rec.restore(op(77), &batch).unwrap();
    assert_eq!(
        (out.dirty[0].actor, out.dirty[0].operation),
        (ActorKind::Recovery, op(77))
    );
    assert_eq!(r.commit_log().1[0].actor, ActorKind::Recovery);
}

#[test]
fn evict_many_unloads_in_one_step_skips_the_unloaded_and_touches_no_revision() {
    // M1-A3d: evicting a slab one brick at a time copied the layer store once per brick.
    let h = host();
    let r = h.reader();
    let sys = h.into_handles().system;
    for x in 0..5 {
        sys.materialise(
            op(x as u64 + 1),
            LayerId::new(L),
            0,
            b(x),
            mat(x as u16 + 1),
        )
        .unwrap();
    }
    let before = r.snapshot();
    let token = before.asset_token().value();
    let cell_rev = before.cell_token(LayerId::new(L), 0, b(2)).unwrap().value();
    // Two loaded, one repeated, one never loaded: three distinct cells are unloaded... two are.
    let n = sys
        .evict_many(LayerId::new(L), 0, &[b(1), b(3), b(3), b(40)])
        .unwrap();
    assert_eq!(n, 2, "only the loaded, distinct cells count");
    let after = r.snapshot();
    assert!(
        !after
            .cell(LayerId::new(L), 0, b(1))
            .unwrap()
            .unwrap()
            .is_loaded()
    );
    assert!(
        !after
            .cell(LayerId::new(L), 0, b(3))
            .unwrap()
            .unwrap()
            .is_loaded()
    );
    for x in [0, 2, 4] {
        assert!(
            after
                .cell(LayerId::new(L), 0, b(x))
                .unwrap()
                .unwrap()
                .is_loaded(),
            "brick {x} stays"
        );
    }
    assert_eq!(after.asset_token().value(), token, "no asset token moves");
    assert_eq!(
        after.cell_token(LayerId::new(L), 0, b(2)).unwrap().value(),
        cell_rev
    );
    // The snapshot taken before is unchanged, as for a single evict.
    assert!(
        before
            .cell(LayerId::new(L), 0, b(1))
            .unwrap()
            .unwrap()
            .is_loaded()
    );
    // An empty list and a list of unloaded cells do nothing.
    assert_eq!(sys.evict_many(LayerId::new(L), 0, &[]), Ok(0));
    assert_eq!(sys.evict_many(LayerId::new(L), 0, &[b(1), b(3)]), Ok(0));
}
