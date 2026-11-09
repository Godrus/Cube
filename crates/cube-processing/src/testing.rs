//! Helpers for the unit tests of this crate (no commit handles: those live in
//! `cube-integration-tests`, the only place allowed to build a live asset).

use cube_core::{AssetId, BrickCoord, CellKey, Epoch, LayerId, Revision};

use crate::snapshot::{DependencyRevision, SnapshotRequest};

/// A stamp of brick `(0,0,0)` of layer 1 for asset revision `order` and brick revision `rev`.
pub(crate) fn dep(order: u64, rev: Option<u64>) -> DependencyRevision {
    let key = CellKey::new(LayerId::new(1), 0, BrickCoord::new(0, 0, 0)).unwrap();
    DependencyRevision::for_tests(
        AssetId::new(1),
        Epoch::INITIAL,
        Revision::new(order),
        SnapshotRequest::for_brick(key, 0),
        vec![(BrickCoord::new(0, 0, 0), rev.map(Revision::new))],
    )
}
