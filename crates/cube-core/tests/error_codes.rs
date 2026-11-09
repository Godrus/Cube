//! Error envelope (CORE-07, contract C-19): stable codes, one negative test per code.

use std::collections::BTreeSet;

use cube_core::{
    AssetHost, AssetId, BrickBounds, BrickCoord, CellChange, CellStore, CellWrite, CommitBatch,
    CommitError, CoreError, ErrorCategory, GeometryAsset, GeometryTransform, Layer, LayerData,
    LayerId, LayerKind, LocalPos, OperationId, Region, Revision, Rotation, Severity, VoxelPos,
    WriteSet,
};

fn all_core_errors() -> Vec<(CoreError, u16)> {
    vec![
        (CoreError::InvalidLevel(-1), 1001),
        (CoreError::Overflow, 1002),
        (CoreError::InvalidLocalCoord, 1003),
        (CoreError::InvalidRegion, 1004),
        (CoreError::InvalidTransform, 1005),
        (CoreError::DuplicateLayer(LayerId::new(1)), 1006),
        (CoreError::RevisionOverflow, 1007),
        (CoreError::DuplicateKey, 1008),
        (CoreError::InvalidView, 1009),
    ]
}

#[test]
fn core_error_codes_are_pinned_unique_and_documented() {
    let mut seen = BTreeSet::new();
    for (e, code) in all_core_errors() {
        assert_eq!(e.code().0, code, "{e}");
        assert!(seen.insert(code), "code {code} reused");
        assert!(!e.to_string().is_empty());
    }
}

#[test]
fn every_core_error_is_actually_produced_by_the_api() {
    assert_eq!(
        VoxelPos::new(0, 0, 0).convert_level(-1, 0),
        Err(CoreError::InvalidLevel(-1))
    );
    assert_eq!(
        VoxelPos::new(i64::MAX, 0, 0).convert_level(0, 1),
        Err(CoreError::Overflow)
    );
    assert_eq!(LocalPos::new(16, 0, 0), Err(CoreError::InvalidLocalCoord));
    assert_eq!(
        BrickBounds::new(BrickCoord::new(1, 0, 0), BrickCoord::new(0, 0, 0)).err(),
        Some(CoreError::InvalidRegion)
    );
    assert_eq!(
        Region::new(0, VoxelPos::new(1, 0, 0), VoxelPos::new(0, 0, 0)).err(),
        Some(CoreError::InvalidRegion)
    );
    assert_eq!(Rotation::from_index(24), Err(CoreError::InvalidTransform));
    assert_eq!(
        GeometryTransform::new([0; 3], Rotation::IDENTITY, 100),
        Err(CoreError::InvalidTransform)
    );
    let l = || {
        Layer::new(
            LayerId::new(1),
            GeometryTransform::IDENTITY,
            LayerData::empty(LayerKind::Voxel),
        )
    };
    assert_eq!(
        GeometryAsset::new(AssetId::new(1), vec![l(), l()]).err(),
        Some(CoreError::DuplicateLayer(LayerId::new(1)))
    );
    assert_eq!(
        Revision::new(u64::MAX).next(),
        Err(CoreError::RevisionOverflow)
    );
    assert_eq!(CoreError::DuplicateKey.code().0, 1008);
    // InvalidView and DuplicateKey from the view import are covered in tests/views.rs.
}

#[test]
fn severity_and_category_are_assigned() {
    assert_eq!(CoreError::RevisionOverflow.severity(), Severity::Critical);
    assert_eq!(CoreError::Overflow.severity(), Severity::Error);
    assert_eq!(CoreError::Overflow.category(), ErrorCategory::Capacity);
    assert_eq!(CoreError::DuplicateKey.category(), ErrorCategory::Conflict);
    assert_eq!(
        CoreError::InvalidLevel(-3).category(),
        ErrorCategory::Validation
    );
    assert_eq!(cube_core::ERROR_MODULE, "core");
}

#[test]
fn commit_error_codes_are_pinned_and_wrap_core_codes() {
    let bounds = BrickBounds::new(BrickCoord::new(0, 0, 0), BrickCoord::new(1, 1, 1)).unwrap();
    let layers = vec![
        Layer::new(
            LayerId::new(1),
            GeometryTransform::IDENTITY,
            LayerData::empty(LayerKind::Voxel),
        ),
        Layer::new(
            LayerId::new(2),
            GeometryTransform::IDENTITY,
            LayerData::Voxel(CellStore::with_bounds(bounds)),
        ),
        Layer::new(
            LayerId::new(3),
            GeometryTransform::IDENTITY,
            LayerData::empty(LayerKind::Mesh),
        ),
    ];
    let user = AssetHost::new(GeometryAsset::new(AssetId::new(1), layers).unwrap())
        .into_handles()
        .user;
    let w = |layer: u64, level: i8, x: i64, expected| CommitBatch {
        sets: vec![WriteSet {
            layer: LayerId::new(layer),
            writes: vec![CellWrite {
                level,
                brick: BrickCoord::new(x, 0, 0),
                expected,
                change: CellChange::KnownEmpty,
            }],
        }],
    };
    let op = OperationId::new(1, 1);
    let absent = cube_core::ExpectedRevision::Absent;
    user.commit(op, &w(1, 0, 0, absent)).unwrap();
    let cases: Vec<(CommitError, u16, ErrorCategory)> = vec![
        (
            user.commit(op, &w(1, 0, 0, absent)).unwrap_err(),
            2001,
            ErrorCategory::Conflict,
        ),
        (
            user.commit(op, &w(99, 0, 0, absent)).unwrap_err(),
            2002,
            ErrorCategory::Validation,
        ),
        (
            user.commit(op, &w(3, 0, 0, absent)).unwrap_err(),
            2003,
            ErrorCategory::Validation,
        ),
        (
            user.commit(op, &w(2, 0, 5, absent)).unwrap_err(),
            2004,
            ErrorCategory::Validation,
        ),
        (
            user.commit(op, &duplicate()).unwrap_err(),
            2005,
            ErrorCategory::Validation,
        ),
        (
            user.commit(op, &w(1, -1, 0, absent)).unwrap_err(),
            1001,
            ErrorCategory::Validation,
        ),
    ];
    for (e, code, cat) in cases {
        assert_eq!(
            (e.code().0, e.category(), e.severity()),
            (code, cat, Severity::Error),
            "{e:?}"
        );
    }
}

fn duplicate() -> CommitBatch {
    let write = || CellWrite {
        level: 0,
        brick: BrickCoord::new(7, 0, 0),
        expected: cube_core::ExpectedRevision::Absent,
        change: CellChange::KnownEmpty,
    };
    CommitBatch {
        sets: vec![WriteSet {
            layer: LayerId::new(1),
            writes: vec![write(), write()],
        }],
    }
}
