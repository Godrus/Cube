//! PS-4 through the real path (ORCH-01..ORCH-05): real Core asset, real `OrchestrationService`,
//! real single user commit path. Lifecycle, atomicity, conflicts, idempotent retry, observers,
//! authorization, dispatch and write-set checks.
#![cfg(not(loom))]

use std::sync::Arc;

use cube_core::{ActorKind, ResourceId};
use cube_editing_api::{
    EditingError, EditingResult, EditingService, Origin, TransactionHandle, TransactionState, kinds,
};
use cube_integration_tests::fakes::*;
use cube_testkit::{DetRng, for_each_seed};

type Script = Box<
    dyn Fn(&cube_editing_api::EditingOperation) -> Result<Vec<cube_core::WriteSet>, EditingError>
        + Send
        + Sync,
>;

fn is_invalid(r: &EditingResult, why: &str) -> bool {
    matches!(r, EditingResult::ValidationFailure(EditingError::InvalidOperation(w)) if w.contains(why))
}

#[test]
fn a_user_edit_goes_through_the_whole_path_and_changes_the_world_once() {
    let f = fixture();
    let before = f.revision();
    let r = f.run(vec![op(
        1,
        kinds::SET_MATERIAL,
        L_VOXEL,
        bc(0, 0, 0),
        bc(1, 0, 0),
        9,
    )]);
    let EditingResult::Success(s) = r else {
        panic!("{r:?}")
    };
    assert_eq!(f.material(bc(0, 0, 0)), Some(9));
    assert_eq!(f.material(bc(1, 0, 0)), Some(9));
    assert_eq!(f.revision(), before + 1, "one transaction is one commit");
    assert_eq!(s.dirty.len(), 2);
    assert!(
        s.new_tokens
            .iter()
            .any(|t| matches!(t.resource(), ResourceId::Asset(_)))
    );
    // The change is attributed to the user path.
    let (total, log) = f.reader.commit_log();
    assert_eq!(total, 1);
    assert!(log.iter().all(|e| e.actor == ActorKind::User));
}

#[test]
fn nothing_is_authoritative_before_commit_and_rollback_and_cancel_change_nothing() {
    let f = fixture();
    let h = f.svc.begin(ctx(Origin::Server), spec()).unwrap();
    assert!(f.svc.execute(&h, set(1, bc(0, 0, 0), 5)).is_success());
    assert_eq!((f.material(bc(0, 0, 0)), f.commits()), (None, 0));
    f.svc.rollback(&h).unwrap();
    assert_eq!((f.material(bc(0, 0, 0)), f.commits()), (None, 0));
    // The handle is released: no second use.
    assert!(matches!(
        f.svc.commit(&h),
        EditingResult::ValidationFailure(EditingError::UnknownTransaction(_))
    ));
    assert!(matches!(
        f.svc.rollback(&h),
        Err(EditingError::UnknownTransaction(_))
    ));

    let h = f.svc.begin(ctx(Origin::LocalClient), spec()).unwrap();
    f.svc.execute(&h, set(2, bc(0, 0, 0), 5));
    f.svc.cancel(set(2, bc(0, 0, 0), 5).id).unwrap();
    assert_eq!((f.material(bc(0, 0, 0)), f.commits()), (None, 0));
    assert_eq!(
        f.svc.cancel(set(2, bc(0, 0, 0), 5).id),
        Err(EditingError::NotFound)
    );
    assert_eq!(
        f.svc.cancel(set(99, bc(0, 0, 0), 5).id),
        Err(EditingError::NotFound)
    );
}

#[test]
fn later_operations_of_a_transaction_see_the_staged_state_of_earlier_ones() {
    // Two operations on one brick: the last write wins and the commit is still one commit.
    let f = fixture();
    let r = f.run(vec![
        set(1, bc(0, 0, 0), 3),
        set(2, bc(0, 0, 0), 4),
        set(3, bc(5, 5, 5), 6),
    ]);
    assert!(r.is_success(), "{r:?}");
    assert_eq!(
        (f.material(bc(0, 0, 0)), f.material(bc(5, 5, 5))),
        (Some(4), Some(6))
    );
    assert_eq!(f.commits(), 1);
}

#[test]
fn a_failed_commit_leaves_no_half_applied_state() {
    let f = fixture();
    let h = f.svc.begin(ctx(Origin::Server), spec()).unwrap();
    // One transaction touches a free brick and a brick that someone else changes meanwhile.
    f.svc.execute(&h, set(1, bc(0, 0, 0), 1));
    f.svc.execute(&h, set(2, bc(1, 0, 0), 1));
    f.svc.execute(&h, set(3, bc(2, 0, 0), 1));
    assert!(f.run(vec![set(10, bc(1, 0, 0), 50)]).is_success());
    let before = f.commits();
    let r = f.svc.commit(&h);
    let EditingResult::Conflict(c) = r else {
        panic!("{r:?}")
    };
    assert_eq!(c.conflicting, vec![ResourceId::Cell(f.key(bc(1, 0, 0)))]);
    assert_eq!(f.commits(), before, "the rejected commit changed nothing");
    assert_eq!(f.material(bc(0, 0, 0)), None);
    assert_eq!(f.material(bc(2, 0, 0)), None);
    assert_eq!(f.material(bc(1, 0, 0)), Some(50));
}

#[test]
fn conflict_matrix() {
    let f = fixture();
    let key = |x| ResourceId::Cell(f.key(bc(x, 0, 0)));
    // (a) disjoint bricks: both succeed.
    let a = f.svc.begin(ctx(Origin::Server), spec()).unwrap();
    let b = f.svc.begin(ctx(Origin::Server), spec()).unwrap();
    f.svc.execute(&a, set(1, bc(0, 0, 0), 1));
    f.svc.execute(&b, set(2, bc(1, 0, 0), 2));
    assert!(f.svc.commit(&a).is_success() && f.svc.commit(&b).is_success());

    // (b) same existing brick: the second committer conflicts, exactly on that brick.
    let a = f.svc.begin(ctx(Origin::Server), spec()).unwrap();
    let b = f.svc.begin(ctx(Origin::Server), spec()).unwrap();
    f.svc.execute(&a, set(3, bc(0, 0, 0), 3));
    f.svc.execute(&b, set(4, bc(0, 0, 0), 4));
    assert!(f.svc.commit(&a).is_success());
    let EditingResult::Conflict(c) = f.svc.commit(&b) else {
        panic!("expected a conflict")
    };
    assert_eq!(c.conflicting, vec![key(0)]);
    assert_eq!((c.expected.len(), c.current.len()), (1, 1));
    assert!(c.tokens_pair_up());
    assert!(c.expected[0].value() < c.current[0].value());
    assert_eq!(f.material(bc(0, 0, 0)), Some(3));

    // (c) a brick that did not exist when the transaction began but exists at commit.
    let a = f.svc.begin(ctx(Origin::Server), spec()).unwrap();
    f.svc.execute(&a, set(5, bc(7, 0, 0), 1));
    assert!(f.run(vec![set(6, bc(7, 0, 0), 2)]).is_success());
    let EditingResult::Conflict(c) = f.svc.commit(&a) else {
        panic!("expected a conflict")
    };
    assert_eq!(c.conflicting, vec![key(7)]);
    assert!(c.expected.is_empty(), "an absent expectation has no token");

    // (d) several bricks: exactly the conflicting ones are named.
    let a = f.svc.begin(ctx(Origin::Server), spec()).unwrap();
    f.svc.execute(
        &a,
        op(
            7,
            kinds::SET_MATERIAL,
            L_VOXEL,
            bc(10, 0, 0),
            bc(12, 0, 0),
            1,
        ),
    );
    assert!(f.run(vec![set(8, bc(10, 0, 0), 2)]).is_success());
    assert!(f.run(vec![set(9, bc(12, 0, 0), 2)]).is_success());
    let EditingResult::Conflict(c) = f.svc.commit(&a) else {
        panic!("expected a conflict")
    };
    assert_eq!(c.conflicting, vec![key(10), key(12)]);

    // (e) stale expectations at begin are a revision conflict before anything is staged.
    let stale = f
        .reader
        .snapshot()
        .cell_token(cube_core::LayerId::new(L_VOXEL), 0, bc(0, 0, 0))
        .unwrap();
    assert!(f.run(vec![set(11, bc(0, 0, 0), 8)]).is_success());
    let mut sp = spec();
    sp.expected = vec![stale];
    assert_eq!(
        f.svc.begin(ctx(Origin::Server), sp).unwrap_err(),
        EditingError::RevisionConflict
    );
    let mut sp = spec();
    sp.expected = vec![
        f.reader
            .snapshot()
            .cell_token(cube_core::LayerId::new(L_VOXEL), 0, bc(0, 0, 0))
            .unwrap(),
    ];
    assert!(f.svc.begin(ctx(Origin::Server), sp).is_ok());
}

#[test]
fn retrying_an_operation_id_never_applies_twice() {
    let f = fixture();
    let first = f.run(vec![set(1, bc(0, 0, 0), 5), set(2, bc(1, 0, 0), 5)]);
    assert!(first.is_success());
    let (rev, commits) = (f.revision(), f.commits());
    // The client lost the answer and sends the same operations again, several times.
    for _ in 0..3 {
        let again = f.run(vec![set(1, bc(0, 0, 0), 5), set(2, bc(1, 0, 0), 5)]);
        assert_eq!(again, first, "the recorded result is returned");
    }
    assert_eq!((f.revision(), f.commits()), (rev, commits));
    // Other writers changed the world meanwhile: a retry still does not re-apply.
    assert!(f.run(vec![set(3, bc(0, 0, 0), 77)]).is_success());
    let again = f.run(vec![set(1, bc(0, 0, 0), 5), set(2, bc(1, 0, 0), 5)]);
    assert_eq!(again, first);
    assert_eq!(f.material(bc(0, 0, 0)), Some(77));
}

#[test]
fn an_operation_id_is_taken_only_by_a_commit_that_applied() {
    let f = fixture();
    let h = f.svc.begin(ctx(Origin::Server), spec()).unwrap();
    f.svc.execute(&h, set(1, bc(0, 0, 0), 1));
    assert!(f.run(vec![set(2, bc(0, 0, 0), 2)]).is_success());
    assert!(matches!(f.svc.commit(&h), EditingResult::Conflict(_)));
    // The conflicted id is free again: retrying it applies normally.
    assert!(f.run(vec![set(1, bc(0, 0, 0), 1)]).is_success());
    assert_eq!(f.material(bc(0, 0, 0)), Some(1));
    // Reusing a committed id with different content is rejected, nothing changes.
    let before = f.commits();
    let r = f.run(vec![set(1, bc(0, 0, 0), 99)]);
    assert!(is_invalid(&r, "reused with different content"), "{r:?}");
    assert_eq!(f.commits(), before);
}

#[test]
fn replayed_and_new_operations_cannot_be_mixed_in_one_transaction() {
    let f = fixture();
    assert!(f.run(vec![set(1, bc(0, 0, 0), 5)]).is_success());
    let h = f.svc.begin(ctx(Origin::Server), spec()).unwrap();
    assert!(f.svc.execute(&h, set(1, bc(0, 0, 0), 5)).is_success());
    assert!(is_invalid(
        &f.svc.execute(&h, set(2, bc(1, 0, 0), 5)),
        "mixing"
    ));
    let h2 = f.svc.begin(ctx(Origin::Server), spec()).unwrap();
    assert!(f.svc.execute(&h2, set(3, bc(1, 0, 0), 5)).is_success());
    assert!(is_invalid(
        &f.svc.execute(&h2, set(1, bc(0, 0, 0), 5)),
        "mixing"
    ));
    assert!(is_invalid(
        &f.svc.execute(&h2, set(3, bc(1, 0, 0), 5)),
        "duplicate operation"
    ));
}

#[test]
fn observers_are_told_in_order_after_the_state_is_committed() {
    let f = fixture();
    let h = f.svc.begin(ctx(Origin::Server), spec()).unwrap();
    f.svc.execute(&h, set(1, bc(0, 0, 0), 1));
    let rolled = f.svc.begin(ctx(Origin::Server), spec()).unwrap();
    f.svc.execute(&rolled, set(2, bc(1, 0, 0), 1));
    assert!(f.svc.commit(&h).is_success());
    f.svc.rollback(&rolled).unwrap();
    assert!(f.run(vec![set(3, bc(2, 0, 0), 1)]).is_success());
    let log = f.log.lock().unwrap().clone();
    // Per commit: dirty sink first, then observers in registration order; rollbacks reach the
    // observers in the same order, and nothing is delivered twice.
    assert_eq!(
        log,
        vec![
            "sink:1",
            "o1:commit:1:1",
            "o2:commit:1:1",
            "o1:rollback:2",
            "o2:rollback:2",
            "sink:1",
            "o1:commit:3:2",
            "o2:commit:3:2",
        ]
    );
    // Observers see the post-commit state, never the state before it.
    assert_eq!(*f.obs1.seen_revisions.lock().unwrap(), vec![1, 2]);
}

#[test]
fn a_failing_observer_is_isolated_and_never_undoes_the_commit() {
    let f = fixture();
    f.obs1
        .panic_on_commit
        .store(true, std::sync::atomic::Ordering::SeqCst);
    let r = f.run(vec![set(1, bc(0, 0, 0), 5)]);
    assert!(r.is_success(), "{r:?}");
    assert_eq!(f.material(bc(0, 0, 0)), Some(5));
    assert_eq!(f.svc.isolated_failures(), 1);
    // The other observer was still told.
    assert_eq!(f.obs2.committed.lock().unwrap().len(), 1);
    assert!(f.obs1.committed.lock().unwrap().is_empty());
}

#[test]
fn a_failed_or_rejected_commit_notifies_nobody_of_a_change() {
    let f = fixture();
    let h = f.svc.begin(ctx(Origin::Server), spec()).unwrap();
    f.svc.execute(&h, set(1, bc(0, 0, 0), 1));
    assert!(f.run(vec![set(2, bc(0, 0, 0), 2)]).is_success());
    f.log.lock().unwrap().clear();
    assert!(matches!(f.svc.commit(&h), EditingResult::Conflict(_)));
    let log = f.log.lock().unwrap().clone();
    assert_eq!(log, vec!["o1:rollback:1", "o2:rollback:1"]);
    assert_eq!(
        f.sink.batches.lock().unwrap().len(),
        1,
        "no dirty regions for a rejected commit"
    );
}

#[test]
fn authorization_is_checked_first_and_a_denial_leaves_no_state() {
    let f = fixture();
    let mut denied = ctx(Origin::Server);
    denied.capabilities.clear();
    // Also a stale expectation: authorization must win over revision validation.
    let mut sp = spec();
    sp.expected = vec![f.reader.snapshot().asset_token().bumped().unwrap()];
    assert_eq!(
        f.svc.begin(denied.clone(), sp).unwrap_err(),
        EditingError::PermissionDenied
    );
    // The local client origin gets nothing extra.
    let mut local = denied;
    local.origin = Origin::LocalClient;
    assert_eq!(
        f.svc.begin(local, spec()).unwrap_err(),
        EditingError::PermissionDenied
    );
    // A context scoped to another asset is denied.
    let mut other = ctx(Origin::Server);
    other.scope.assets = vec![cube_core::AssetId::new(99)];
    assert_eq!(
        f.svc.begin(other, spec()).unwrap_err(),
        EditingError::PermissionDenied
    );
    // System origin never edits through the user path.
    assert_eq!(
        f.svc.begin(ctx(Origin::System), spec()).unwrap_err(),
        EditingError::PermissionDenied
    );
    assert_eq!(f.commits(), 0);
    // And a valid context works with either user origin.
    for o in [Origin::LocalClient, Origin::Server] {
        assert!(f.svc.begin(ctx(o), spec()).is_ok());
    }
}

#[test]
fn an_operation_on_another_asset_or_with_a_bad_shape_is_rejected_before_any_backend_runs() {
    let f = fixture();
    let h = f.svc.begin(ctx(Origin::Server), spec()).unwrap();
    let mut wrong_asset = set(1, bc(0, 0, 0), 1);
    wrong_asset.asset = cube_core::AssetId::new(99);
    // Authorization says no (the scope names only asset 1).
    assert!(matches!(
        f.svc.execute(&h, wrong_asset),
        EditingResult::ValidationFailure(EditingError::PermissionDenied)
    ));
    let mut no_layers = set(2, bc(0, 0, 0), 1);
    no_layers.layers.clear();
    assert!(is_invalid(&f.svc.execute(&h, no_layers), "no target layer"));
    let mut unknown_layer = set(3, bc(0, 0, 0), 1);
    unknown_layer.layers = vec![cube_core::LayerId::new(77)];
    assert!(is_invalid(
        &f.svc.execute(&h, unknown_layer),
        "unknown layer"
    ));
    assert_eq!(
        f.svc
            .begin(ctx(Origin::Server), {
                let mut s = spec();
                s.layers = vec![cube_core::LayerId::new(77)];
                s
            })
            .unwrap_err(),
        EditingError::InvalidOperation("unknown layer")
    );
}

#[test]
fn dispatch_by_capabilities_rejects_unsupported_kinds_types_and_layers() {
    let f = fixture();
    let h = f.svc.begin(ctx(Origin::Server), spec()).unwrap();
    let unsupported = op(1, kinds::SCULPT, L_VOXEL, bc(0, 0, 0), bc(0, 0, 0), 1);
    assert!(matches!(
        f.svc.execute(&h, unsupported),
        EditingResult::Unsupported(EditingError::UnsupportedGeometryOperation(k)) if k == kinds::SCULPT
    ));
    // The mesh layer has no registered backend.
    let mesh = op(2, kinds::SET_MATERIAL, L_MESH, bc(0, 0, 0), bc(0, 0, 0), 1);
    assert!(matches!(
        f.svc.execute(&h, mesh),
        EditingResult::Unsupported(_)
    ));
    // Mixed representations without a conversion policy.
    let mut mixed = set(3, bc(0, 0, 0), 1);
    mixed.layers = vec![
        cube_core::LayerId::new(L_VOXEL),
        cube_core::LayerId::new(L_MESH),
    ];
    assert!(matches!(
        f.svc.execute(&h, mixed),
        EditingResult::ValidationFailure(EditingError::IncompatibleGeometryTypes)
    ));
    // Nothing was staged by any of that.
    assert!(f.svc.commit(&h).is_success());
    assert_eq!(f.commits(), 0);
}

#[test]
fn write_sets_that_break_the_contract_are_rejected_and_stage_nothing() {
    let cases: Vec<(&str, Script)> = vec![
        (
            "outside region",
            Box::new(|_| Ok(vec![cell_write(L_VOXEL, bc(9, 9, 9), 1)])),
        ),
        (
            "not a target",
            Box::new(|_| Ok(vec![cell_write(L_BOUNDED, bc(0, 0, 0), 1)])),
        ),
        (
            "twice",
            Box::new(|_| {
                let mut s = cell_write(L_VOXEL, bc(0, 0, 0), 1);
                s.writes.push(s.writes[0].clone());
                Ok(vec![s])
            }),
        ),
    ];
    for (name, script) in cases {
        let f = fixture_with(Options {
            backends: vec![ScriptedBackend::arc(script)],
            ..Options::default()
        });
        let h = f.svc.begin(ctx(Origin::Server), spec()).unwrap();
        let r = f.svc.execute(&h, set(1, bc(0, 0, 0), 1));
        assert!(
            matches!(r, EditingResult::ValidationFailure(_)),
            "{name}: {r:?}"
        );
        assert!(f.svc.commit(&h).is_success());
        assert_eq!((f.commits(), f.material(bc(0, 0, 0))), (0, None), "{name}");
    }
    // A level other than the region's is an invalid resolution.
    let f = fixture_with(Options {
        backends: vec![ScriptedBackend::arc(|_| {
            let mut s = cell_write(L_VOXEL, bc(0, 0, 0), 1);
            s.writes[0].level = 3;
            Ok(vec![s])
        })],
        ..Options::default()
    });
    let h = f.svc.begin(ctx(Origin::Server), spec()).unwrap();
    assert!(matches!(
        f.svc.execute(&h, set(1, bc(0, 0, 0), 1)),
        EditingResult::ValidationFailure(EditingError::InvalidResolution)
    ));
}

#[test]
fn backend_errors_and_panics_are_controlled_results_and_the_transaction_stays_usable() {
    let f = fixture_with(Options {
        backends: vec![ScriptedBackend::arc(|o| match o.id.sequence() {
            1 => Err(EditingError::ConversionRequired),
            2 => panic!("backend bug"),
            3 => Err(EditingError::ProcessingFailed),
            _ => Ok(vec![cell_write(L_VOXEL, bc(0, 0, 0), 4)]),
        })],
        ..Options::default()
    });
    let h = f.svc.begin(ctx(Origin::Server), spec()).unwrap();
    assert!(matches!(
        f.svc.execute(&h, set(1, bc(0, 0, 0), 1)),
        EditingResult::ConversionRequired(_)
    ));
    assert!(matches!(
        f.svc.execute(&h, set(2, bc(0, 0, 0), 1)),
        EditingResult::ResourceFailure(EditingError::ProcessingFailed)
    ));
    assert!(matches!(
        f.svc.execute(&h, set(3, bc(0, 0, 0), 1)),
        EditingResult::ResourceFailure(_)
    ));
    assert!(f.svc.execute(&h, set(4, bc(0, 0, 0), 1)).is_success());
    assert!(f.svc.commit(&h).is_success());
    assert_eq!(f.material(bc(0, 0, 0)), Some(4));
}

#[test]
fn writes_outside_layer_bounds_are_rejected_by_core_and_change_nothing() {
    let f = fixture();
    let h = f.svc.begin(ctx(Origin::Server), spec()).unwrap();
    // Bounds of the bounded layer are [-2, 2): brick 5 is outside.
    assert!(
        f.svc
            .execute(
                &h,
                op(
                    1,
                    kinds::SET_MATERIAL,
                    L_BOUNDED,
                    bc(0, 0, 0),
                    bc(0, 0, 0),
                    1
                )
            )
            .is_success()
    );
    assert!(
        f.svc
            .execute(
                &h,
                op(
                    2,
                    kinds::SET_MATERIAL,
                    L_BOUNDED,
                    bc(5, 0, 0),
                    bc(5, 0, 0),
                    1
                )
            )
            .is_success()
    );
    let r = f.svc.commit(&h);
    assert!(
        matches!(
            r,
            EditingResult::ValidationFailure(EditingError::InvalidOperation(_))
        ),
        "{r:?}"
    );
    assert_eq!(f.material_in(L_BOUNDED, bc(0, 0, 0)), None);
    assert_eq!(f.commits(), 0);
}

#[test]
fn dirty_regions_reach_the_sink_exactly_as_committed() {
    let f = fixture();
    let r = f.run(vec![op(
        1,
        kinds::SET_MATERIAL,
        L_VOXEL,
        bc(-1, 0, 0),
        bc(0, 0, 0),
        3,
    )]);
    let EditingResult::Success(s) = r else {
        panic!("{r:?}")
    };
    let batches = f.sink.batches.lock().unwrap().clone();
    assert_eq!(batches.len(), 1);
    assert_eq!(batches[0], s.dirty);
    assert_eq!(batches[0].len(), 2);
    // Writing identical content changes nothing and reports nothing dirty.
    assert!(
        f.run(vec![op(
            2,
            kinds::SET_MATERIAL,
            L_VOXEL,
            bc(-1, 0, 0),
            bc(0, 0, 0),
            3
        )])
        .is_success()
    );
    assert!(f.sink.batches.lock().unwrap()[1].is_empty());
    assert_eq!(f.commits(), 1);
}

#[test]
fn remove_and_add_geometry_round_trip_through_the_path() {
    let f = fixture();
    assert!(
        f.run(vec![op(
            1,
            kinds::ADD_GEOMETRY,
            L_VOXEL,
            bc(0, 0, 0),
            bc(0, 0, 0),
            2
        )])
        .is_success()
    );
    assert_eq!(f.material(bc(0, 0, 0)), Some(2));
    assert!(
        f.run(vec![op(
            2,
            kinds::REMOVE_GEOMETRY,
            L_VOXEL,
            bc(0, 0, 0),
            bc(0, 0, 0),
            0
        )])
        .is_success()
    );
    assert_eq!(f.material(bc(0, 0, 0)), None);
    let state = f.reader.snapshot();
    assert!(matches!(
        state.cell(cube_core::LayerId::new(L_VOXEL), 0, bc(0, 0, 0)),
        Some(Ok(cube_core::CellState::KnownEmpty))
    ));
}

#[test]
fn only_the_user_path_yields_user_commits_and_the_three_writers_stay_distinct() {
    let f = fixture();
    assert!(f.run(vec![set(1, bc(0, 0, 0), 1)]).is_success());
    let sys = f.system.materialise(
        cube_core::OperationId::new(1, 1),
        cube_core::LayerId::new(L_VOXEL),
        0,
        bc(8, 0, 0),
        cube_core::CellChange::KnownEmpty,
    );
    assert!(sys.is_ok());
    let batch = cube_core::CommitBatch {
        sets: vec![cube_core::WriteSet {
            layer: cube_core::LayerId::new(L_VOXEL),
            writes: vec![cube_core::CellWrite {
                level: 0,
                brick: bc(9, 0, 0),
                expected: cube_core::ExpectedRevision::Absent,
                change: cube_core::CellChange::KnownEmpty,
            }],
        }],
    };
    f.recovery
        .restore(cube_core::OperationId::new(1, 2), &batch)
        .unwrap();
    assert!(f.run(vec![set(2, bc(1, 0, 0), 1)]).is_success());
    let (_, log) = f.reader.commit_log();
    let actors: Vec<_> = log.iter().map(|e| e.actor).collect();
    assert_eq!(
        actors,
        vec![
            ActorKind::User,
            ActorKind::System,
            ActorKind::Recovery,
            ActorKind::User
        ]
    );
}

#[test]
fn model_based_random_sessions_keep_the_world_equal_to_the_committed_model() {
    // Random begin/execute/commit/rollback over a few bricks with up to two open transactions.
    // Model: committed material per brick and revision per brick; a commit conflicts exactly
    // when one of its bricks changed after the transaction began.
    for_each_seed(0xC0FFEE, 60, |seed| {
        let f = fixture();
        let mut rng = DetRng::new(seed);
        let mut model: std::collections::BTreeMap<i64, (u16, u64)> = Default::default();
        let mut clock = 0u64;
        struct Open {
            h: TransactionHandle,
            began: u64,
            staged: std::collections::BTreeMap<i64, u16>,
        }
        let mut open: Vec<Open> = Vec::new();
        let mut next = 1u64;
        for _ in 0..80 {
            match rng.below(4) {
                0 if open.len() < 2 => {
                    let h = f.svc.begin(ctx(Origin::Server), spec()).unwrap();
                    open.push(Open {
                        h,
                        began: clock,
                        staged: Default::default(),
                    });
                }
                1 if !open.is_empty() => {
                    let i = rng.below(open.len() as u64) as usize;
                    let (x, m) = (rng.below(4) as i64, 1 + rng.below(9) as u16);
                    let r = f.svc.execute(&open[i].h, set(next, bc(x, 0, 0), m));
                    next += 1;
                    assert!(r.is_success(), "{r:?}");
                    open[i].staged.insert(x, m);
                }
                2 if !open.is_empty() => {
                    let i = rng.below(open.len() as u64) as usize;
                    let t = open.remove(i);
                    let conflict = t
                        .staged
                        .keys()
                        .any(|x| model.get(x).is_some_and(|(_, at)| *at > t.began));
                    let r = f.svc.commit(&t.h);
                    if t.staged.is_empty() {
                        assert!(r.is_success());
                    } else if conflict {
                        assert!(matches!(r, EditingResult::Conflict(_)), "{r:?}");
                    } else {
                        assert!(r.is_success(), "{r:?}");
                        // Identical content is not a change and keeps its revision.
                        let changed = t
                            .staged
                            .iter()
                            .any(|(x, m)| model.get(x).map(|v| v.0) != Some(*m));
                        if changed {
                            clock += 1;
                        }
                        for (x, m) in &t.staged {
                            let same = model.get(x).map(|v| v.0) == Some(*m);
                            if !same {
                                model.insert(*x, (*m, clock));
                            }
                        }
                    }
                }
                3 if !open.is_empty() => {
                    let i = rng.below(open.len() as u64) as usize;
                    let t = open.remove(i);
                    f.svc.rollback(&t.h).unwrap();
                }
                _ => {}
            }
            for x in 0..4 {
                assert_eq!(
                    f.material(bc(x, 0, 0)),
                    model.get(&x).map(|v| v.0),
                    "brick {x}"
                );
            }
            assert_eq!(f.revision(), clock);
        }
    });
}

#[test]
fn the_service_is_shareable_across_threads() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<cube_orchestration::OrchestrationService>();
    let f = fixture();
    let svc = Arc::clone(&f.svc);
    std::thread::spawn(move || svc.begin(ctx(Origin::Server), spec()).map(|_| ()))
        .join()
        .unwrap()
        .unwrap();
    assert_eq!(TransactionState::Open, TransactionState::Open);
}
