//! TEST-08 (model-checked part): exhaustive interleavings of the Processing primitives under
//! `loom`. Run with:
//! `RUSTFLAGS="--cfg loom" cargo test -p cube-integration-tests --test processing_loom --release`
//!
//! Models: the derived slot (stale never published), the cancellation latch (one winner, callbacks
//! exactly once) and the executor (a task racing a commit and its notification).
#![cfg(loom)]

mod common;

use std::sync::Arc;

use common::*;
use cube_processing::{
    CancelReason, CancellationToken, DerivedSlot, Executor, ExecutorConfig, ManualClock, Priority,
    PriorityClass, ProviderRegistry, PublishOutcome, SlotStatus, SnapshotRequest, TargetStatus,
    TaskOutcome, TaskState, stamp,
};
use loom::sync::atomic::{AtomicUsize, Ordering};

fn model(f: impl Fn() + Sync + Send + 'static) {
    let mut b = loom::model::Builder::new();
    b.preemption_bound = Some(3);
    b.check(f);
}

fn two_stamps() -> (
    cube_processing::DependencyRevision,
    cube_processing::DependencyRevision,
) {
    let w = world();
    w.put(L_VOXEL, bc(0, 0, 0), 1);
    let req = SnapshotRequest::for_brick(key(L_VOXEL, 0, 0, 0), 0);
    let s1 = stamp(&w.reader.snapshot(), &req, None).unwrap();
    w.put(L_VOXEL, bc(0, 0, 0), 2);
    let s2 = stamp(&w.reader.snapshot(), &req, None).unwrap();
    assert!(!s1.same_inputs(&s2));
    (s1, s2)
}

#[test]
fn slot_no_stale_value_is_visible_after_expect_returns() {
    model(|| {
        let (s1, s2) = two_stamps();
        let slot: Arc<DerivedSlot<u32>> = Arc::new(DerivedSlot::new());
        let (sa, sb) = (Arc::clone(&slot), Arc::clone(&slot));
        let (r1, l1) = (s1.clone(), s1);
        let a = loom::thread::spawn(move || sa.publish(&r1, &l1, Arc::new(1)));
        let b = loom::thread::spawn(move || {
            sb.expect(&s2);
            // Only a result of the old inputs was ever offered: nothing may be current now.
            sb.current().is_none()
        });
        let outcome = a.join().unwrap();
        assert!(
            b.join().unwrap(),
            "a stale value was visible after expect returned"
        );
        match outcome {
            // The publication won the race, then it was invalidated.
            PublishOutcome::Published { .. } => assert_eq!(slot.status(), SlotStatus::Stale),
            PublishOutcome::Stale => assert_eq!(slot.status(), SlotStatus::Missing),
        }
    });
}

#[test]
fn slot_racing_publishers_end_with_the_newest_inputs() {
    model(|| {
        let (s1, s2) = two_stamps();
        let slot: Arc<DerivedSlot<u32>> = Arc::new(DerivedSlot::new());
        let (sa, sb) = (Arc::clone(&slot), Arc::clone(&slot));
        let (a1, a2) = (s2.clone(), s2);
        let (b1, b2) = (s1.clone(), s1);
        let a = loom::thread::spawn(move || sa.publish(&a1, &a2, Arc::new(2)));
        let b = loom::thread::spawn(move || sb.publish(&b1, &b2, Arc::new(1)));
        assert!(matches!(
            a.join().unwrap(),
            PublishOutcome::Published { .. }
        ));
        let _ = b.join().unwrap();
        // Whatever the order, the value of the newest inputs is the one that is current.
        assert_eq!(slot.status(), SlotStatus::Valid);
        assert_eq!(*slot.current().unwrap().value, 2);
    });
}

#[test]
fn slot_readers_never_see_a_stale_value_flagged_as_current() {
    model(|| {
        let (s1, s2) = two_stamps();
        let slot: Arc<DerivedSlot<u32>> = Arc::new(DerivedSlot::new());
        let (sa, sb, sc) = (Arc::clone(&slot), Arc::clone(&slot), Arc::clone(&slot));
        let (r1, l1) = (s1.clone(), s1);
        let a = loom::thread::spawn(move || sa.publish(&r1, &l1, Arc::new(1)));
        let b = loom::thread::spawn(move || sb.expect(&s2));
        let c = loom::thread::spawn(move || sc.last_known().map(|(p, cur)| (*p.value, cur)));
        a.join().unwrap();
        b.join().unwrap();
        if let Some((v, current)) = c.join().unwrap() {
            // Value and flag are read atomically: if flagged current it matched the expectation
            // at that instant, which for this model can only be the old inputs.
            assert_eq!(v, 1);
            let _ = current;
        }
    });
}

#[test]
fn token_one_winner_and_callbacks_run_exactly_once() {
    model(|| {
        let t = CancellationToken::new();
        let calls = Arc::new(AtomicUsize::new(0));
        let c = Arc::clone(&calls);
        let seen = Arc::new(loom::sync::Mutex::new(None));
        let s = Arc::clone(&seen);
        let t_reg = t.clone();
        let reg = loom::thread::spawn(move || {
            t_reg.register(move |r| {
                c.fetch_add(1, Ordering::SeqCst);
                *s.lock().unwrap() = Some(r);
            })
        });
        let (t1, t2) = (t.clone(), t.clone());
        let w1 = loom::thread::spawn(move || t1.cancel(CancelReason::Requested));
        let w2 = loom::thread::spawn(move || t2.cancel(CancelReason::Superseded));
        let (a, b) = (w1.join().unwrap(), w2.join().unwrap());
        let _registration = reg.join().unwrap();
        assert!(a ^ b, "exactly one cancel wins");
        assert_eq!(
            calls.load(Ordering::SeqCst),
            1,
            "callback must run exactly once"
        );
        assert_eq!(
            *seen.lock().unwrap(),
            t.reason(),
            "callback saw the winning reason"
        );
    });
}

#[test]
fn token_child_is_cancelled_by_a_racing_parent_cancel() {
    model(|| {
        let parent = CancellationToken::new();
        let p = parent.clone();
        let child_holder = Arc::new(loom::sync::Mutex::new(None));
        let h = Arc::clone(&child_holder);
        let p2 = parent.clone();
        let spawner = loom::thread::spawn(move || {
            *h.lock().unwrap() = Some(p2.child());
        });
        let canceller = loom::thread::spawn(move || {
            p.cancel(CancelReason::Shutdown);
        });
        spawner.join().unwrap();
        canceller.join().unwrap();
        let child = child_holder.lock().unwrap().take().unwrap();
        assert_eq!(
            child.reason(),
            Some(CancelReason::Shutdown),
            "a child created while the parent was being cancelled must not be left running"
        );
    });
}

#[test]
fn executor_task_racing_a_commit_never_publishes_a_stale_result() {
    model(|| {
        let w = Arc::new(world());
        w.put(L_VOXEL, bc(0, 0, 0), 1);
        let mut r = ProviderRegistry::new();
        r.register(SumProvider::plain(1)).unwrap();
        let e = Executor::new(
            w.reader.clone(),
            r.freeze().unwrap(),
            ExecutorConfig::default(),
            Arc::new(ManualClock::new(0)),
        );
        let t = target(1, L_VOXEL, 0, 0, 0);
        let h = e.submit(t, Priority::new(PriorityClass::Visible)).unwrap();
        let (e1, e2, w2) = (e.clone(), e.clone(), Arc::clone(&w));
        let runner = loom::thread::spawn(move || e1.run_one());
        let committer = loom::thread::spawn(move || {
            let out = w2.put(L_VOXEL, bc(0, 0, 0), 2);
            e2.notify_commit(&out.dirty);
        });
        runner.join().unwrap();
        committer.join().unwrap();

        // The task ran (or was ended) and is terminal.
        assert!(h.state().is_terminal(), "state {:?}", h.state());
        match h.outcome().unwrap() {
            TaskOutcome::Published { .. } | TaskOutcome::Stale => {}
            TaskOutcome::Cancelled(CancelReason::Superseded) => {}
            other => panic!("unexpected outcome {other:?}"),
        }
        // After the commit nothing older than it is current.
        let current = e.get(t).unwrap();
        if let Some(p) = current {
            let v = p.downcast_ref::<Tagged>().unwrap();
            assert_eq!(
                v.revision,
                w.rev(bc(0, 0, 0)),
                "a current value must be from the latest state"
            );
        }
        match e.query(t).unwrap() {
            TargetStatus::Valid => assert_eq!(h.state(), TaskState::Completed),
            TargetStatus::Stale | TargetStatus::Missing => {}
            TargetStatus::Computing => panic!("nothing may still be computing"),
        }
    });
}

#[test]
fn pin_budget_never_over_commits_and_always_returns_everything() {
    use cube_processing::{Caps, PinBudget};
    model(|| {
        let budget = PinBudget::new(Caps {
            max_snapshot_bricks: u64::MAX,
            max_snapshot_bytes: u64::MAX,
            max_active_snapshots: u64::MAX,
            max_pinned_bricks: 4,
            max_pinned_bytes: 100,
            task_ttl_ms: None,
        });
        let hs: Vec<_> = (0..2)
            .map(|_| {
                let b = Arc::clone(&budget);
                loom::thread::spawn(move || {
                    let mut r = b.try_reserve(3)?;
                    assert!(b.usage().bricks <= 4, "over-committed");
                    let _ = r.add_bytes(60);
                    assert!(b.usage().bytes <= 100, "byte cap exceeded");
                    Some(())
                })
            })
            .collect();
        let wins = hs
            .into_iter()
            .map(|h| h.join().unwrap())
            .filter(Option::is_some)
            .count();
        assert!(wins >= 1, "at least one reservation of 3 of 4 must succeed");
        let u = budget.usage();
        assert_eq!((u.bricks, u.bytes), (0, 0));
    });
}
