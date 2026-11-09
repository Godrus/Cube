//! Loom model of the commit gate (TEST-05). Run with
//! `RUSTFLAGS="--cfg loom" cargo test -p cube-core --test loom_gate --release`.
//! Without `--cfg loom` this file compiles to nothing.
#![cfg(loom)]

use std::sync::Arc;

use cube_core::{
    AssetHost, AssetId, BrickCoord, CellChange, CellPayload, CellState, CellWrite, ChannelSet,
    CommitBatch, CommitError, ExpectedRevision, GeometryAsset, GeometryTransform, Layer, LayerData,
    LayerId, LayerKind, MaterialIndex, OperationId, WriteSet,
};
use loom::thread;

fn host() -> AssetHost {
    let layer = Layer::new(
        LayerId::new(1),
        GeometryTransform::IDENTITY,
        LayerData::empty(LayerKind::Voxel),
    );
    AssetHost::new(GeometryAsset::new(AssetId::new(1), vec![layer]).unwrap())
}

fn write(m: u16) -> CommitBatch {
    let payload = CellPayload::Channels(ChannelSet::uniform_material(MaterialIndex::from_raw(m)));
    CommitBatch {
        sets: vec![WriteSet {
            layer: LayerId::new(1),
            writes: vec![CellWrite {
                level: 0,
                brick: BrickCoord::new(0, 0, 0),
                expected: ExpectedRevision::Absent,
                change: CellChange::Put(payload),
            }],
        }],
    }
}

#[test]
fn two_writers_race_exactly_one_wins_and_a_reader_never_sees_a_torn_state() {
    loom::model(|| {
        let h = host();
        let reader = h.reader();
        let user = Arc::new(h.into_handles().user);
        let (a, b) = (Arc::clone(&user), Arc::clone(&user));
        let t1 = thread::spawn(move || a.commit(OperationId::new(1, 1), &write(1)));
        let t2 = thread::spawn(move || b.commit(OperationId::new(1, 2), &write(2)));
        let r = thread::spawn(move || {
            let snap = reader.snapshot();
            let rev = snap.asset_token().value().raw();
            let cell = snap
                .cell(LayerId::new(1), 0, BrickCoord::new(0, 0, 0))
                .unwrap()
                .unwrap();
            // Either nothing is committed yet, or the revision and the cell agree.
            match rev {
                0 => assert_eq!(cell, CellState::NotLoaded),
                1 => assert!(matches!(cell, CellState::Present(_))),
                _ => panic!("impossible revision {rev}"),
            }
        });
        let (r1, r2) = (t1.join().unwrap(), t2.join().unwrap());
        r.join().unwrap();
        assert!(r1.is_ok() != r2.is_ok(), "exactly one writer wins");
        let loser = if r1.is_ok() { r2 } else { r1 };
        assert!(matches!(loser, Err(CommitError::Conflict(_))));
    });
}

#[test]
fn concurrent_writers_to_different_cells_serialise_with_dense_revisions() {
    loom::model(|| {
        let h = host();
        let reader = h.reader();
        let user = Arc::new(h.into_handles().user);
        let at = |x: i64, m: u16| {
            let payload =
                CellPayload::Channels(ChannelSet::uniform_material(MaterialIndex::from_raw(m)));
            CommitBatch {
                sets: vec![WriteSet {
                    layer: LayerId::new(1),
                    writes: vec![CellWrite {
                        level: 0,
                        brick: BrickCoord::new(x, 0, 0),
                        expected: ExpectedRevision::Absent,
                        change: CellChange::Put(payload),
                    }],
                }],
            }
        };
        let (a, b) = (Arc::clone(&user), Arc::clone(&user));
        let (ba, bb) = (at(1, 1), at(2, 2));
        let t1 = thread::spawn(move || a.commit(OperationId::new(1, 1), &ba).unwrap());
        let t2 = thread::spawn(move || b.commit(OperationId::new(1, 2), &bb).unwrap());
        let (o1, o2) = (t1.join().unwrap(), t2.join().unwrap());
        let mut revs = [o1.asset_token.value().raw(), o2.asset_token.value().raw()];
        revs.sort_unstable();
        assert_eq!(revs, [1, 2]);
        assert_eq!(reader.snapshot().asset_token().value().raw(), 2);
        assert_eq!(reader.commit_log().0, 2);
    });
}
