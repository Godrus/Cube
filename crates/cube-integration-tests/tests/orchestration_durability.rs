//! Journal boundary (DEC-086, blocker CB-1) and concurrent revision changes through the real
//! Orchestration path. The journal is a contract fake: no production crash durability is claimed
//! (that is PS-10).
#![cfg(not(loom))]

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use cube_core::OperationId;
use cube_editing_api::{
    EditingError, EditingResult, EditingService, JournalPhase, OperationStatus, Origin,
};
use cube_integration_tests::fakes::*;

fn journalled() -> Fixture {
    fixture_with(Options {
        journal: true,
        ..Options::default()
    })
}

#[test]
fn a_durable_operation_is_journalled_write_ahead_then_confirmed() {
    let f = journalled();
    let r = f.run(vec![set(1, bc(0, 0, 0), 5)]);
    assert!(r.is_success(), "{r:?}");
    assert_eq!(
        f.journal.phases(),
        vec![JournalPhase::Intent, JournalPhase::Confirmed]
    );
    let e = f.journal.entries();
    assert_eq!((e[0].seq.0, e[1].seq.0), (1, 2));
    assert_eq!(e[0].operation, OperationId::new(7, 1));
    assert!(e[0].post_tokens.is_empty() && !e[1].post_tokens.is_empty());
    assert_eq!(e[0].actor.0, 42);
}

#[test]
fn when_the_intent_cannot_be_made_durable_nothing_is_applied() {
    let f = journalled();
    f.journal.fail_intent.store(true, Ordering::SeqCst);
    let r = f.run(vec![set(1, bc(0, 0, 0), 5)]);
    assert_eq!(
        r,
        EditingResult::ResourceFailure(EditingError::PersistenceFailure)
    );
    assert_eq!((f.commits(), f.material(bc(0, 0, 0))), (0, None));
    assert!(f.journal.entries().is_empty());
    // The id was never taken: after the journal heals the same request applies.
    f.journal.fail_intent.store(false, Ordering::SeqCst);
    assert!(f.run(vec![set(1, bc(0, 0, 0), 5)]).is_success());
}

#[test]
fn a_lost_confirmation_is_never_reported_as_success_and_a_retry_completes_it_once() {
    let f = journalled();
    f.journal.fail_confirm.store(true, Ordering::SeqCst);
    let r = f.run(vec![set(1, bc(0, 0, 0), 5)]);
    let EditingResult::AppliedUnconfirmed(s) = &r else {
        panic!("expected AppliedUnconfirmed, got {r:?}")
    };
    assert!(!r.is_success() && !r.left_state_unchanged());
    // The change IS applied, observers and the dirty sink were told, and the state is visible.
    assert_eq!((f.material(bc(0, 0, 0)), f.commits()), (Some(5), 1));
    assert_eq!(f.obs1.committed.lock().unwrap().len(), 1);
    assert_eq!(f.svc.unconfirmed(), 1);
    assert_eq!(
        f.svc.status(OperationId::new(7, 1)),
        Ok(OperationStatus::Running)
    );
    assert_eq!(f.journal.phases(), vec![JournalPhase::Intent]);

    // Retrying while the journal is still down stays honest and applies nothing.
    let again = f.run(vec![set(1, bc(0, 0, 0), 5)]);
    assert_eq!(again, r);
    assert_eq!(f.commits(), 1);

    // The journal heals: the retry confirms without applying twice and now reports success.
    f.journal.fail_confirm.store(false, Ordering::SeqCst);
    let done = f.run(vec![set(1, bc(0, 0, 0), 5)]);
    assert_eq!(done, EditingResult::Success(s.clone()));
    assert_eq!(f.commits(), 1);
    assert_eq!(f.svc.unconfirmed(), 0);
    assert_eq!(
        f.journal.phases(),
        vec![JournalPhase::Intent, JournalPhase::Confirmed]
    );
    assert_eq!(
        f.svc.status(OperationId::new(7, 1)),
        Ok(OperationStatus::Succeeded)
    );
}

#[test]
fn reconcile_confirms_outstanding_operations_without_a_client_retry() {
    let f = journalled();
    f.journal.fail_confirm.store(true, Ordering::SeqCst);
    f.run(vec![set(1, bc(0, 0, 0), 5)]);
    f.run(vec![set(2, bc(1, 0, 0), 5)]);
    assert_eq!((f.svc.unconfirmed(), f.svc.reconcile()), (2, 0));
    f.journal.fail_confirm.store(false, Ordering::SeqCst);
    assert_eq!(f.svc.reconcile(), 2);
    assert_eq!(f.svc.unconfirmed(), 0);
    assert_eq!(
        f.svc.status(OperationId::new(7, 2)),
        Ok(OperationStatus::Succeeded)
    );
    // Each operation was confirmed exactly once.
    let confirmed = f
        .journal
        .entries()
        .iter()
        .filter(|e| e.phase == JournalPhase::Confirmed)
        .count();
    assert_eq!(confirmed, 2);
}

#[test]
fn a_rejected_commit_is_journalled_as_aborted_and_a_dead_journal_cannot_hide_it() {
    let f = journalled();
    let h = f.svc.begin(ctx(Origin::Server), spec()).unwrap();
    f.svc.execute(&h, set(1, bc(0, 0, 0), 1));
    assert!(f.run(vec![set(2, bc(0, 0, 0), 2)]).is_success());
    assert!(matches!(f.svc.commit(&h), EditingResult::Conflict(_)));
    assert_eq!(
        f.journal.phases(),
        vec![
            JournalPhase::Intent,
            JournalPhase::Confirmed,
            JournalPhase::Intent,
            JournalPhase::Aborted
        ]
    );
    // Even if the abort record cannot be written the caller still gets the conflict.
    let h = f.svc.begin(ctx(Origin::Server), spec()).unwrap();
    f.svc.execute(&h, set(3, bc(0, 0, 0), 1));
    assert!(f.run(vec![set(4, bc(0, 0, 0), 9)]).is_success());
    f.journal.fail_abort.store(true, Ordering::SeqCst);
    assert!(matches!(f.svc.commit(&h), EditingResult::Conflict(_)));
    // Sequence numbers are monotonic and gapless.
    let seqs: Vec<u64> = f.journal.entries().iter().map(|e| e.seq.0).collect();
    assert_eq!(seqs, (1..=seqs.len() as u64).collect::<Vec<_>>());
}

#[test]
fn an_unjournalled_service_makes_no_durability_claim_and_writes_no_records() {
    let f = fixture();
    assert!(f.run(vec![set(1, bc(0, 0, 0), 5)]).is_success());
    assert!(f.journal.entries().is_empty());
}

#[test]
fn concurrent_transactions_never_lose_an_update_and_notifications_follow_commit_order() {
    const THREADS: usize = 6;
    const ROUNDS: u64 = 40;
    let f = Arc::new(fixture());
    let applied = Arc::new(AtomicUsize::new(0));
    let conflicts = Arc::new(AtomicUsize::new(0));
    let handles: Vec<_> = (0..THREADS as u64)
        .map(|t| {
            let (f, applied, conflicts) =
                (Arc::clone(&f), Arc::clone(&applied), Arc::clone(&conflicts));
            std::thread::spawn(move || {
                for r in 0..ROUNDS {
                    // Everyone fights over brick (0,0,0) and also owns a private brick.
                    let base = 1000 * (t + 1) + 2 * r;
                    let h = f.svc.begin(ctx(Origin::Server), spec()).unwrap();
                    assert!(
                        f.svc
                            .execute(&h, set(base, bc(0, 0, 0), (t + 1) as u16))
                            .is_success()
                    );
                    assert!(
                        f.svc
                            .execute(&h, set(base + 1, bc(t as i64 + 1, 0, 0), r as u16 + 1))
                            .is_success()
                    );
                    match f.svc.commit(&h) {
                        EditingResult::Success(_) => {
                            applied.fetch_add(1, Ordering::SeqCst);
                        }
                        EditingResult::Conflict(c) => {
                            conflicts.fetch_add(1, Ordering::SeqCst);
                            // Only the contested brick can conflict: private bricks never do.
                            assert_eq!(c.conflicting.len(), 1);
                        }
                        other => panic!("{other:?}"),
                    }
                }
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
    let (applied, conflicts) = (
        applied.load(Ordering::SeqCst),
        conflicts.load(Ordering::SeqCst),
    );
    assert_eq!(applied + conflicts, THREADS * ROUNDS as usize);
    assert!(applied >= 1);
    // Every success is exactly one Core commit; every conflict changed nothing.
    assert_eq!(f.commits(), applied as u64);
    assert_eq!(f.revision(), applied as u64);
    // Notifications arrive once per commit, in the order Core committed them.
    let seen = f.obs1.committed.lock().unwrap().clone();
    let (_, log) = f.reader.commit_log();
    assert_eq!(seen.len(), applied);
    assert_eq!(seen, log.iter().map(|e| e.operation).collect::<Vec<_>>());
    assert_eq!(
        *f.obs1.committed.lock().unwrap(),
        *f.obs2.committed.lock().unwrap()
    );
    // The contested brick holds the value of the LAST committed transaction: no lost update.
    let last = seen.last().unwrap();
    let writer_thread = (last.sequence() / 1000) as u16;
    assert_eq!(f.material(bc(0, 0, 0)), Some(u32::from(writer_thread)));
}
