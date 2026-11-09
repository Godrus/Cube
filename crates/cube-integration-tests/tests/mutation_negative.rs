//! TEST-10: negative tests through the real Orchestration path. A user request cannot yield a
//! System or Recovery commit, and clients and adapters cannot commit. Compile-fail cases prove
//! the type-level boundary; `cargo xtask archcheck` (with seeded self-tests) proves the
//! source-level boundary and runs in CI; the runtime tests prove the behaviour of the real path.
#![cfg(not(loom))]

use cube_core::ActorKind;
use cube_editing_api::{EditingResult, EditingService, Origin, kinds};
use cube_integration_tests::fakes::*;

#[test]
fn bypass_attempts_do_not_compile() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui_mutation/*.rs");
}

#[test]
fn no_user_request_can_produce_a_system_or_recovery_commit() {
    let f = fixture();
    // Every origin a request may carry, with every kind of operation, many times over.
    for n in 0..30u64 {
        for origin in [Origin::LocalClient, Origin::Server, Origin::System] {
            let h = f.svc.begin(ctx(origin), spec());
            if let Ok(h) = h {
                let kind = [
                    kinds::SET_MATERIAL,
                    kinds::ADD_GEOMETRY,
                    kinds::REMOVE_GEOMETRY,
                ][(n % 3) as usize];
                f.svc.execute(
                    &h,
                    op(
                        1000 * n + origin as u64,
                        kind,
                        L_VOXEL,
                        bc(n as i64, 0, 0),
                        bc(n as i64, 0, 0),
                        1,
                    ),
                );
                let _ = f.svc.commit(&h);
            }
        }
    }
    let (total, log) = f.reader.commit_log();
    assert!(total > 0);
    assert!(
        log.iter().all(|e| e.actor == ActorKind::User),
        "the user path can only produce user commits"
    );
}

#[test]
fn a_system_origin_request_is_refused_not_downgraded() {
    let f = fixture();
    assert_eq!(
        f.svc.begin(ctx(Origin::System), spec()).unwrap_err(),
        cube_editing_api::EditingError::PermissionDenied
    );
    assert_eq!(f.commits(), 0);
}

#[test]
fn a_client_without_the_service_has_no_route_to_the_world() {
    // What a client holds is a reader: it can look, and the only mutating entry is the service.
    let f = fixture();
    let before = f.revision();
    let reader = f.reader.clone();
    let _look = reader.snapshot();
    assert_eq!(f.revision(), before);
    assert!(f.run(vec![set(1, bc(0, 0, 0), 1)]).is_success());
    assert_eq!(reader.snapshot().asset_token().value().raw(), before + 1);
}

#[test]
fn failed_user_requests_never_leave_a_commit_of_any_actor() {
    let f = fixture();
    let h = f.svc.begin(ctx(Origin::Server), spec()).unwrap();
    f.svc.execute(&h, set(1, bc(0, 0, 0), 1));
    assert!(f.run(vec![set(2, bc(0, 0, 0), 2)]).is_success());
    let before = f.commits();
    assert!(matches!(f.svc.commit(&h), EditingResult::Conflict(_)));
    assert_eq!(f.commits(), before);
}
