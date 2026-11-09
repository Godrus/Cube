//! PROC-01: snapshots with apron against a live asset (immutability under commits, apron
//! correctness, explicit absent-data states, negative and extreme coordinates).
#![cfg(not(loom))]

mod common;

use common::*;
use cube_core::{CellState, LayerId};
use cube_processing::{LimitKind, ProcessingError, Snapshot, SnapshotRequest, stamp};

fn req(layer: u64, lo: (i64, i64, i64), hi: (i64, i64, i64), apron: u32) -> SnapshotRequest {
    SnapshotRequest::for_bricks(
        LayerId::new(layer),
        0,
        bc(lo.0, lo.1, lo.2),
        bc(hi.0, hi.1, hi.2),
        apron,
    )
    .unwrap()
}

#[test]
fn a_snapshot_is_immutable_under_later_commits() {
    let w = world();
    w.put(L_VOXEL, bc(0, 0, 0), 5);
    let r = req(L_VOXEL, (0, 0, 0), (0, 0, 0), 1);
    let snap = Snapshot::capture(&w.reader, &r, None).unwrap();
    let before = snap.dependency().clone();
    let content = |s: &Snapshot| -> Vec<(i64, i64, i64, bool)> {
        s.cells()
            .map(|(b, c)| (b.x, b.y, b.z, matches!(c, CellState::Present(_))))
            .collect()
    };
    let before_cells = content(&snap);

    w.put(L_VOXEL, bc(0, 0, 0), 9);
    w.put(L_VOXEL, bc(1, 0, 0), 7);

    assert_eq!(content(&snap), before_cells);
    assert_eq!(snap.dependency(), &before);
    let fresh = Snapshot::capture(&w.reader, &r, None).unwrap();
    assert!(!fresh.dependency().same_inputs(&before));
    assert_ne!(content(&fresh), before_cells);
}

#[test]
fn a_snapshot_is_one_consistent_asset_revision() {
    let w = world();
    for i in 0..5 {
        w.put(L_VOXEL, bc(i, 0, 0), 1);
    }
    let snap = Snapshot::capture(&w.reader, &req(L_VOXEL, (0, 0, 0), (4, 0, 0), 0), None).unwrap();
    // The stamp's order is the asset revision of the state it was read from.
    assert_eq!(snap.dependency().order().raw(), 5);
    assert_eq!(snap.dependency().inputs().len(), 5);
    assert!(snap.dependency().inputs().iter().all(|(_, r)| r.is_some()));
}

#[test]
fn apron_bricks_report_every_distinct_state_and_never_merge_them() {
    let w = world();
    w.put(L_BOUNDED, bc(0, 0, 0), 1); // present, centre
    w.empty(L_BOUNDED, bc(1, 0, 0)); // loaded and empty
    // (-1,0,0) was never loaded; brick 2 is outside the layer bounds [-2, 2).
    let snap =
        Snapshot::capture(&w.reader, &req(L_BOUNDED, (0, 0, 0), (1, 0, 0), 1), None).unwrap();
    assert!(matches!(
        snap.cell(bc(0, 0, 0)).unwrap(),
        CellState::Present(_)
    ));
    assert!(matches!(
        snap.cell(bc(1, 0, 0)).unwrap(),
        CellState::KnownEmpty
    ));
    assert!(matches!(
        snap.cell(bc(-1, 0, 0)).unwrap(),
        CellState::NotLoaded
    ));
    assert!(matches!(
        snap.cell(bc(2, 0, 0)).unwrap(),
        CellState::OutsideBounds
    ));
    assert!(matches!(
        snap.cell(bc(3, 0, 0)),
        Err(ProcessingError::OutsideSnapshot(_))
    ));
    // Bounds of the layer are [-2, 2): brick 2 is outside the bounds, not "not loaded".
    let edge =
        Snapshot::capture(&w.reader, &req(L_BOUNDED, (1, 0, 0), (1, 0, 0), 1), None).unwrap();
    assert!(matches!(
        edge.cell(bc(2, 0, 0)).unwrap(),
        CellState::OutsideBounds
    ));
    assert!(edge.is_core(bc(1, 0, 0)) && !edge.is_core(bc(2, 0, 0)));
    assert_eq!(edge.core_cells().count(), 1);
    assert_eq!(edge.brick_count(), 27);
}

#[test]
fn reading_outside_the_captured_area_is_an_error_not_empty() {
    let w = world();
    let snap = Snapshot::capture(&w.reader, &req(L_VOXEL, (0, 0, 0), (0, 0, 0), 1), None).unwrap();
    assert!(snap.cell(bc(1, 1, 1)).is_ok());
    assert!(matches!(
        snap.cell(bc(2, 0, 0)),
        Err(ProcessingError::OutsideSnapshot(k)) if k.brick() == bc(2, 0, 0)
    ));
    assert!(snap.neighbour(bc(0, 0, 0), [1, -1, 0]).is_ok());
    assert!(snap.neighbour(bc(0, 0, 0), [2, 0, 0]).is_err());
}

#[test]
fn negative_coordinates_are_addressed_like_positive_ones() {
    let w = world();
    w.put(L_VOXEL, bc(-1, -1, -1), 3);
    w.put(L_VOXEL, bc(-5, 0, 0), 4);
    let snap = Snapshot::capture(
        &w.reader,
        &req(L_VOXEL, (-1, -1, -1), (-1, -1, -1), 1),
        None,
    )
    .unwrap();
    assert!(matches!(
        snap.cell(bc(-1, -1, -1)).unwrap(),
        CellState::Present(_)
    ));
    assert_eq!(
        snap.neighbour(bc(-1, -1, -1), [1, 1, 1])
            .map(|c| c.is_loaded()),
        Ok(false)
    );
    let far = Snapshot::capture(&w.reader, &req(L_VOXEL, (-5, 0, 0), (-5, 0, 0), 0), None).unwrap();
    assert!(matches!(
        far.cell(bc(-5, 0, 0)).unwrap(),
        CellState::Present(_)
    ));
}

#[test]
fn the_ends_of_the_grid_are_handled_without_overflow() {
    let w = world();
    let r = SnapshotRequest::for_bricks(
        LayerId::new(L_VOXEL),
        0,
        bc(i64::MAX, i64::MIN, 0),
        bc(i64::MAX, i64::MIN, 0),
        1,
    )
    .unwrap();
    let snap = Snapshot::capture(&w.reader, &r, None).unwrap();
    assert_eq!(snap.brick_count(), 2 * 2 * 3);
    // The neighbour beyond the grid does not exist: OutsideBounds, never a wrapped coordinate.
    assert!(matches!(
        snap.neighbour(bc(i64::MAX, i64::MIN, 0), [1, 0, 0])
            .unwrap(),
        CellState::OutsideBounds
    ));
    assert!(matches!(
        snap.neighbour(bc(i64::MAX, i64::MIN, 0), [0, -1, 0])
            .unwrap(),
        CellState::OutsideBounds
    ));
}

#[test]
fn unknown_and_unsupported_layers_and_oversized_requests_fail_cleanly() {
    let w = world();
    assert!(matches!(
        Snapshot::capture(&w.reader, &req(99, (0, 0, 0), (0, 0, 0), 0), None),
        Err(ProcessingError::UnknownLayer(_))
    ));
    assert!(matches!(
        Snapshot::capture(&w.reader, &req(L_MESH, (0, 0, 0), (0, 0, 0), 0), None),
        Err(ProcessingError::UnsupportedLayer(_))
    ));
    let big = req(L_VOXEL, (0, 0, 0), (9, 9, 9), 0);
    assert_eq!(
        Snapshot::capture(&w.reader, &big, Some(999)).unwrap_err(),
        ProcessingError::ResourceUnavailable(LimitKind::PinnedBricks)
    );
    assert!(Snapshot::capture(&w.reader, &big, Some(1000)).is_ok());
}

#[test]
fn stamps_change_only_for_bricks_in_the_captured_area() {
    let w = world();
    w.put(L_VOXEL, bc(0, 0, 0), 1);
    let r = req(L_VOXEL, (0, 0, 0), (0, 0, 0), 1);
    let s1 = stamp(&w.reader.snapshot(), &r, None).unwrap();
    w.put(L_VOXEL, bc(10, 10, 10), 1); // far away
    let s2 = stamp(&w.reader.snapshot(), &r, None).unwrap();
    assert!(s1.same_inputs(&s2));
    assert!(s2.order_key() > s1.order_key());
    w.put(L_VOXEL, bc(1, 1, 1), 1); // inside the apron
    let s3 = stamp(&w.reader.snapshot(), &r, None).unwrap();
    assert!(!s2.same_inputs(&s3));
    // The first commit of a brick that was never committed also changes the stamp.
    w.put(L_VOXEL, bc(-1, 0, 0), 1);
    let s4 = stamp(&w.reader.snapshot(), &r, None).unwrap();
    assert!(!s3.same_inputs(&s4));
}

#[test]
fn snapshots_can_be_read_from_many_threads_while_commits_continue() {
    let w = std::sync::Arc::new(world());
    w.put(L_VOXEL, bc(0, 0, 0), 1);
    let snap = std::sync::Arc::new(
        Snapshot::capture(&w.reader, &req(L_VOXEL, (0, 0, 0), (0, 0, 0), 1), None).unwrap(),
    );
    let expected = snap.dependency().clone();
    let readers: Vec<_> = (0..4)
        .map(|_| {
            let s = std::sync::Arc::clone(&snap);
            std::thread::spawn(move || {
                for _ in 0..200 {
                    assert!(matches!(
                        s.cell(bc(0, 0, 0)).unwrap(),
                        CellState::Present(_)
                    ));
                    assert_eq!(s.brick_count(), 27);
                }
            })
        })
        .collect();
    for m in 2..60 {
        w.put(L_VOXEL, bc(0, 0, 0), m);
    }
    for h in readers {
        h.join().unwrap();
    }
    assert_eq!(snap.dependency(), &expected);
}
