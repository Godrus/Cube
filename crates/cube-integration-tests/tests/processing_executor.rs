//! PROC-02..PROC-05 behaviour of the executor against a live asset: dependency ordering,
//! scheduling, cancellation at every stage, error reporting, stale rejection and invalidation.
#![cfg(not(loom))]

mod common;

use std::sync::{Arc, Mutex};

use common::*;
use cube_processing::{
    CancelReason, ChangeMask, ComputeContext, DerivedKind, Executor, ExecutorConfig, LimitKind,
    ManualClock, Payload, Priority, PriorityClass, ProcessingError, Provider, ProviderFailure,
    ProviderRegistry, TargetStatus, TaskOutcome, TaskState,
};

fn executor(w: &World, providers: Vec<Arc<dyn Provider>>, cfg: ExecutorConfig) -> Executor {
    let mut r = ProviderRegistry::new();
    for p in providers {
        r.register(p).unwrap();
    }
    Executor::new(
        w.reader.clone(),
        r.freeze().unwrap(),
        cfg,
        Arc::new(ManualClock::new(0)),
    )
}

fn prio(c: PriorityClass) -> Priority {
    Priority::new(c)
}

fn tagged(p: &Payload) -> Tagged {
    *p.downcast_ref::<Tagged>().expect("Tagged payload")
}

type Log = Arc<Mutex<Vec<(i64, i64, i64)>>>;

/// Records which brick it computed, in order.
struct Recorder {
    kind: u32,
    log: Log,
}

impl Provider for Recorder {
    fn kind(&self) -> DerivedKind {
        DerivedKind(self.kind)
    }
    fn name(&self) -> &str {
        "recorder"
    }
    fn version(&self) -> u32 {
        1
    }
    fn consumes(&self) -> ChangeMask {
        ChangeMask::ALL
    }
    fn compute(&self, ctx: &ComputeContext<'_>) -> Result<Payload, ProviderFailure> {
        let b = ctx.target.key.brick();
        self.log.lock().unwrap().push((b.x, b.y, b.z));
        Ok(Arc::new(()))
    }
}

fn recorder_executor(cfg: ExecutorConfig) -> (World, Executor, Log) {
    let w = world();
    let log = Arc::new(Mutex::new(Vec::new()));
    let e = executor(
        &w,
        vec![Arc::new(Recorder {
            kind: 1,
            log: Arc::clone(&log),
        })],
        cfg,
    );
    (w, e, log)
}

#[test]
fn a_task_computes_publishes_and_the_result_is_current() {
    let w = world();
    w.put(L_VOXEL, bc(0, 0, 0), 5);
    w.put(L_VOXEL, bc(1, 0, 0), 7);
    let e = executor(&w, vec![SumProvider::plain(1)], ExecutorConfig::default());
    let t = target(1, L_VOXEL, 0, 0, 0);
    assert_eq!(e.query(t), Ok(TargetStatus::Missing));
    let h = e.submit(t, prio(PriorityClass::Visible)).unwrap();
    assert_eq!(h.state(), TaskState::Queued);
    assert_eq!(e.query(t), Ok(TargetStatus::Computing));
    assert_eq!(e.run_until_idle(), 1);
    assert_eq!(h.outcome(), Some(TaskOutcome::Published { sequence: 1 }));
    assert_eq!(h.state(), TaskState::Completed);
    assert_eq!(e.query(t), Ok(TargetStatus::Valid));
    let v = tagged(&e.get(t).unwrap().unwrap());
    assert_eq!((v.sum, v.present, v.revision), (12, 2, Some(1)));
}

#[test]
fn a_queued_task_captures_the_state_when_it_runs_not_when_it_was_submitted() {
    let w = world();
    w.put(L_VOXEL, bc(0, 0, 0), 1);
    let e = executor(&w, vec![SumProvider::plain(1)], ExecutorConfig::default());
    let t = target(1, L_VOXEL, 0, 0, 0);
    e.submit(t, prio(PriorityClass::Visible)).unwrap();
    w.put(L_VOXEL, bc(0, 0, 0), 2);
    e.run_until_idle();
    let v = tagged(&e.get(t).unwrap().unwrap());
    assert_eq!((v.sum, v.revision), (2, Some(2)));
}

#[test]
fn unknown_kinds_and_shutdown_are_rejected() {
    let w = world();
    let e = executor(&w, vec![SumProvider::plain(1)], ExecutorConfig::default());
    assert_eq!(
        e.submit(target(9, L_VOXEL, 0, 0, 0), prio(PriorityClass::Visible))
            .unwrap_err(),
        ProcessingError::UnknownKind(DerivedKind(9))
    );
    let h = e
        .submit(target(1, L_VOXEL, 0, 0, 0), prio(PriorityClass::Visible))
        .unwrap();
    e.shutdown();
    assert_eq!(
        h.outcome(),
        Some(TaskOutcome::Cancelled(CancelReason::Shutdown))
    );
    assert_eq!(
        e.submit(target(1, L_VOXEL, 1, 0, 0), prio(PriorityClass::Visible))
            .unwrap_err(),
        ProcessingError::ShutDown
    );
    assert!(!e.run_one());
}

#[test]
fn equal_requests_are_coalesced_and_the_higher_priority_wins() {
    let (_w, e, log) = recorder_executor(ExecutorConfig::default());
    let a = e
        .submit(target(1, L_VOXEL, 0, 0, 0), prio(PriorityClass::Background))
        .unwrap();
    let b = e
        .submit(target(1, L_VOXEL, 1, 0, 0), prio(PriorityClass::Background))
        .unwrap();
    let b2 = e
        .submit(
            target(1, L_VOXEL, 1, 0, 0),
            prio(PriorityClass::Interactive),
        )
        .unwrap();
    assert_eq!(b.id(), b2.id());
    assert_ne!(a.id(), b.id());
    assert_eq!(e.stats().queued, 2);
    e.run_until_idle();
    assert_eq!(*log.lock().unwrap(), vec![(1, 0, 0), (0, 0, 0)]);
}

#[test]
fn scheduling_orders_by_class_then_weight_then_submission() {
    let (_w, e, log) = recorder_executor(ExecutorConfig::default());
    let p = |c, w| Priority {
        class: c,
        weight: w,
    };
    let order = [
        (0, p(PriorityClass::Maintenance, 0)),
        (1, p(PriorityClass::Background, 1)),
        (2, p(PriorityClass::Background, 9)),
        (3, p(PriorityClass::Visible, 0)),
        (4, p(PriorityClass::Interactive, 0)),
        (5, p(PriorityClass::Visible, 0)),
    ];
    for (x, pr) in order {
        e.submit(target(1, L_VOXEL, x, 0, 0), pr).unwrap();
    }
    e.run_until_idle();
    let xs: Vec<i64> = log.lock().unwrap().iter().map(|b| b.0).collect();
    assert_eq!(xs, vec![4, 3, 5, 2, 1, 0]);
}

#[test]
fn aging_prevents_starvation_of_low_priority_work() {
    let cfg = ExecutorConfig {
        aging_ticks: 2,
        ..ExecutorConfig::default()
    };
    let (_w, e, log) = recorder_executor(cfg);
    e.submit(
        target(1, L_VOXEL, 0, 0, 0),
        prio(PriorityClass::Maintenance),
    )
    .unwrap();
    for x in 1..=10 {
        e.submit(
            target(1, L_VOXEL, x, 0, 0),
            prio(PriorityClass::Interactive),
        )
        .unwrap();
    }
    e.run_until_idle();
    let pos = log
        .lock()
        .unwrap()
        .iter()
        .position(|b| b.0 == 0)
        .expect("maintenance task ran");
    assert!(
        pos < 10,
        "the oldest low-priority task must overtake newer interactive work (ran at {pos})"
    );
    assert!(
        pos > 0,
        "but not before work that outranks it from the start"
    );
}

#[test]
fn per_class_and_total_running_limits_are_enforced() {
    let w = world();
    w.put(L_VOXEL, bc(0, 0, 0), 1);
    let entered = Gate::new();
    let release = Gate::new();
    let cfg = ExecutorConfig {
        class_running_limits: [4, 4, 3, 2, 1],
        max_running: 4,
        ..ExecutorConfig::default()
    };
    let e = executor(
        &w,
        vec![SumProvider::paused(1, &entered, &release, true)],
        cfg,
    );
    let hold = |x| {
        e.submit(
            target(1, L_VOXEL, x, 0, 0),
            prio(PriorityClass::Maintenance),
        )
        .unwrap()
    };
    let h1 = hold(0);
    let h2 = hold(1);
    let runner = {
        let e = e.clone();
        std::thread::spawn(move || e.run_one())
    };
    entered.wait();
    assert_eq!(e.stats().running, 1);
    // Maintenance allows one at a time: the second task must wait.
    assert!(!e.run_one());
    assert_eq!(h2.state(), TaskState::Queued);
    release.open();
    assert!(runner.join().unwrap());
    assert!(e.run_one());
    assert_eq!(h1.state(), TaskState::Completed);
    assert_eq!(h2.state(), TaskState::Completed);
}

#[test]
fn a_full_queue_rejects_instead_of_dropping() {
    let cfg = ExecutorConfig {
        max_queued: 2,
        ..ExecutorConfig::default()
    };
    let (_w, e, _log) = recorder_executor(cfg);
    for x in 0..2 {
        e.submit(target(1, L_VOXEL, x, 0, 0), prio(PriorityClass::Visible))
            .unwrap();
    }
    assert_eq!(
        e.submit(target(1, L_VOXEL, 2, 0, 0), prio(PriorityClass::Visible))
            .unwrap_err(),
        ProcessingError::ResourceUnavailable(LimitKind::QueueFull)
    );
    // An equal request is not a new task and is still accepted.
    assert!(
        e.submit(target(1, L_VOXEL, 0, 0, 0), prio(PriorityClass::Visible))
            .is_ok()
    );
    e.run_one();
    assert!(
        e.submit(target(1, L_VOXEL, 2, 0, 0), prio(PriorityClass::Visible))
            .is_ok(),
        "space frees up as tasks finish"
    );
}

#[test]
fn derived_inputs_run_first_and_feed_the_dependent() {
    let w = world();
    w.put(L_VOXEL, bc(0, 0, 0), 5);
    let e = executor(
        &w,
        vec![SumProvider::plain(1), DoubleOf::of(2, 1)],
        ExecutorConfig::default(),
    );
    let t2 = target(2, L_VOXEL, 0, 0, 0);
    let t1 = target(1, L_VOXEL, 0, 0, 0);
    let h = e.submit(t2, prio(PriorityClass::Visible)).unwrap();
    assert_eq!(
        e.stats().queued,
        2,
        "the input task was submitted automatically"
    );
    assert!(e.run_one(), "the input task runs first");
    assert_eq!(h.state(), TaskState::Queued);
    assert!(e.run_one());
    assert_eq!(h.state(), TaskState::Completed);
    let s1 = tagged(&e.get(t1).unwrap().unwrap()).sum;
    let s2 = tagged(&e.get(t2).unwrap().unwrap()).sum;
    assert_eq!((s1, s2), (5, 10));
    // A current input is not recomputed for a second dependent request.
    let again = e.submit(t2, prio(PriorityClass::Visible)).unwrap();
    assert_eq!(e.stats().queued, 1);
    assert_ne!(again.id(), h.id());
}

#[test]
fn a_failed_input_cancels_its_dependents_and_nothing_is_published() {
    let w = world();
    let started = Gate::new();
    let e = executor(
        &w,
        vec![
            Arc::new(Odd {
                kind: 1,
                mode: Mode::Fail,
                started,
            }),
            DoubleOf::of(2, 1),
        ],
        ExecutorConfig::default(),
    );
    let t1 = target(1, L_VOXEL, 0, 0, 0);
    let t2 = target(2, L_VOXEL, 0, 0, 0);
    let dep = e.submit(t2, prio(PriorityClass::Visible)).unwrap();
    e.run_until_idle();
    let input = e.submit(t1, prio(PriorityClass::Visible)).unwrap();
    assert!(matches!(
        dep.outcome(),
        Some(TaskOutcome::Cancelled(CancelReason::DependencyFailed))
    ));
    e.run_until_idle();
    assert_eq!(
        input.outcome(),
        Some(TaskOutcome::Failed(ProcessingError::ProviderFailed(
            "boom".into()
        )))
    );
    assert_eq!(e.query(t1), Ok(TargetStatus::Missing));
    assert_eq!(e.query(t2), Ok(TargetStatus::Missing));
    assert!(e.get(t1).unwrap().is_none() && e.get(t2).unwrap().is_none());
}

#[test]
fn a_panicking_provider_is_a_failure_and_the_executor_keeps_working() {
    let w = world();
    let started = Gate::new();
    let e = executor(
        &w,
        vec![
            Arc::new(Odd {
                kind: 1,
                mode: Mode::Panic,
                started,
            }),
            SumProvider::plain(2),
        ],
        ExecutorConfig::default(),
    );
    let bad = e
        .submit(target(1, L_VOXEL, 0, 0, 0), prio(PriorityClass::Visible))
        .unwrap();
    let good = e
        .submit(target(2, L_VOXEL, 0, 0, 0), prio(PriorityClass::Visible))
        .unwrap();
    assert_eq!(e.run_until_idle(), 2);
    assert_eq!(
        bad.outcome(),
        Some(TaskOutcome::Failed(ProcessingError::ProviderFailed(
            "provider panicked".into()
        )))
    );
    assert_eq!(good.state(), TaskState::Completed);
    assert_eq!(
        e.stats(),
        cube_processing::Stats {
            queued: 0,
            running: 0
        }
    );
}

#[test]
fn cancelling_a_queued_task_ends_it_at_once_and_it_never_runs() {
    let (_w, e, log) = recorder_executor(ExecutorConfig::default());
    let h = e
        .submit(target(1, L_VOXEL, 0, 0, 0), prio(PriorityClass::Visible))
        .unwrap();
    h.cancel();
    assert_eq!(h.state(), TaskState::Cancelled);
    assert_eq!(
        h.outcome(),
        Some(TaskOutcome::Cancelled(CancelReason::Requested))
    );
    assert!(!e.run_one());
    assert!(log.lock().unwrap().is_empty());
    assert_eq!(e.stats().queued, 0);
    assert_eq!(
        e.query(target(1, L_VOXEL, 0, 0, 0)),
        Ok(TargetStatus::Missing)
    );
}

#[test]
fn cancelling_a_running_task_stops_it_cooperatively() {
    let w = world();
    let started = Gate::new();
    let e = executor(
        &w,
        vec![Arc::new(Odd {
            kind: 1,
            mode: Mode::SpinUntilCancelled,
            started: Arc::clone(&started),
        })],
        ExecutorConfig::default(),
    );
    let t = target(1, L_VOXEL, 0, 0, 0);
    let h = e.submit(t, prio(PriorityClass::Visible)).unwrap();
    let runner = {
        let e = e.clone();
        std::thread::spawn(move || e.run_one())
    };
    started.wait();
    assert_eq!(h.state(), TaskState::Running);
    h.cancel();
    runner.join().unwrap();
    assert_eq!(
        h.outcome(),
        Some(TaskOutcome::Cancelled(CancelReason::Requested))
    );
    assert_eq!(e.query(t), Ok(TargetStatus::Missing));
    // Cancelling again, or after the end, changes nothing.
    h.cancel();
    assert_eq!(h.state(), TaskState::Cancelled);
}

#[test]
fn cancellation_after_the_computation_but_before_publication_prevents_publication() {
    let w = world();
    w.put(L_VOXEL, bc(0, 0, 0), 1);
    let entered = Gate::new();
    let release = Gate::new();
    // The provider ignores its token, so only the check before publication can stop it.
    let e = executor(
        &w,
        vec![SumProvider::paused(1, &entered, &release, false)],
        ExecutorConfig::default(),
    );
    let t = target(1, L_VOXEL, 0, 0, 0);
    let h = e.submit(t, prio(PriorityClass::Visible)).unwrap();
    let runner = {
        let e = e.clone();
        std::thread::spawn(move || e.run_one())
    };
    entered.wait();
    h.cancel();
    release.open();
    runner.join().unwrap();
    assert_eq!(
        h.outcome(),
        Some(TaskOutcome::Cancelled(CancelReason::Requested))
    );
    assert_eq!(e.query(t), Ok(TargetStatus::Missing));
}

#[test]
fn cancelling_a_task_cancels_the_queued_tasks_that_wait_for_it() {
    let w = world();
    let e = executor(
        &w,
        vec![SumProvider::plain(1), DoubleOf::of(2, 1)],
        ExecutorConfig::default(),
    );
    let dependent = e
        .submit(target(2, L_VOXEL, 0, 0, 0), prio(PriorityClass::Visible))
        .unwrap();
    // The input task is coalesced: the same handle comes back.
    let input = e
        .submit(target(1, L_VOXEL, 0, 0, 0), prio(PriorityClass::Visible))
        .unwrap();
    input.cancel();
    assert_eq!(
        dependent.outcome(),
        Some(TaskOutcome::Cancelled(CancelReason::DependencyCancelled))
    );
    assert!(!e.run_one());
    assert_eq!(
        e.stats(),
        cube_processing::Stats {
            queued: 0,
            running: 0
        }
    );
}

#[test]
fn a_result_computed_from_changed_inputs_is_never_published() {
    let w = world();
    w.put(L_VOXEL, bc(0, 0, 0), 1);
    let entered = Gate::new();
    let release = Gate::new();
    let e = executor(
        &w,
        vec![SumProvider::paused(1, &entered, &release, false)],
        ExecutorConfig::default(),
    );
    let t = target(1, L_VOXEL, 0, 0, 0);
    let h = e.submit(t, prio(PriorityClass::Visible)).unwrap();
    let runner = {
        let e = e.clone();
        std::thread::spawn(move || e.run_one())
    };
    entered.wait();
    let out = w.put(L_VOXEL, bc(0, 0, 0), 2); // an input of the running task changes
    let affected = e.notify_commit(&out.dirty);
    assert!(affected.contains(&t));
    release.open();
    runner.join().unwrap();
    // The notification already cancelled the superseded task, so the check before publication
    // ends it; without that check the slot would reject the result as Stale (next test).
    assert_eq!(
        h.outcome(),
        Some(TaskOutcome::Cancelled(CancelReason::Superseded))
    );
    assert_eq!(e.query(t), Ok(TargetStatus::Missing));
    assert!(e.get(t).unwrap().is_none());
}

#[test]
fn a_missed_notification_cannot_let_a_stale_result_through() {
    let w = world();
    w.put(L_VOXEL, bc(0, 0, 0), 1);
    let entered = Gate::new();
    let release = Gate::new();
    let e = executor(
        &w,
        vec![SumProvider::paused(1, &entered, &release, false)],
        ExecutorConfig::default(),
    );
    let t = target(1, L_VOXEL, 0, 0, 0);
    let h = e.submit(t, prio(PriorityClass::Visible)).unwrap();
    let runner = {
        let e = e.clone();
        std::thread::spawn(move || e.run_one())
    };
    entered.wait();
    w.put(L_VOXEL, bc(0, 0, 0), 2); // nobody calls notify_commit
    release.open();
    runner.join().unwrap();
    assert_eq!(h.outcome(), Some(TaskOutcome::Stale));
    assert!(e.get(t).unwrap().is_none());
}

#[test]
fn a_running_task_with_changed_inputs_is_cancelled_as_superseded() {
    let w = world();
    w.put(L_VOXEL, bc(0, 0, 0), 1);
    let entered = Gate::new();
    let release = Gate::new();
    let e = executor(
        &w,
        vec![SumProvider::paused(1, &entered, &release, true)],
        ExecutorConfig::default(),
    );
    let t = target(1, L_VOXEL, 0, 0, 0);
    let h = e.submit(t, prio(PriorityClass::Visible)).unwrap();
    let runner = {
        let e = e.clone();
        std::thread::spawn(move || e.run_one())
    };
    entered.wait();
    let out = w.put(L_VOXEL, bc(1, 0, 0), 4); // inside the apron
    e.notify_commit(&out.dirty);
    release.open();
    runner.join().unwrap();
    assert_eq!(
        h.outcome(),
        Some(TaskOutcome::Cancelled(CancelReason::Superseded))
    );
    assert_eq!(e.query(t), Ok(TargetStatus::Missing));
}

#[test]
fn an_unrelated_commit_keeps_results_valid_and_a_related_one_invalidates_them() {
    let w = world();
    w.put(L_VOXEL, bc(0, 0, 0), 1);
    let e = executor(&w, vec![SumProvider::plain(1)], ExecutorConfig::default());
    let t = target(1, L_VOXEL, 0, 0, 0);
    e.submit(t, prio(PriorityClass::Visible)).unwrap();
    e.run_until_idle();
    assert_eq!(e.query(t), Ok(TargetStatus::Valid));

    let far = w.put(L_VOXEL, bc(10, 10, 10), 1);
    let affected = e.notify_commit(&far.dirty);
    assert!(!affected.contains(&t));
    assert_eq!(e.query(t), Ok(TargetStatus::Valid));

    let near = w.put(L_VOXEL, bc(-1, 0, 0), 3);
    assert!(e.notify_commit(&near.dirty).contains(&t));
    assert_eq!(e.query(t), Ok(TargetStatus::Stale));
    assert!(
        e.get(t).unwrap().is_none(),
        "a stale value is never returned as current"
    );

    e.submit(t, prio(PriorityClass::Visible)).unwrap();
    e.run_until_idle();
    assert_eq!(e.query(t), Ok(TargetStatus::Valid));
    assert_eq!(tagged(&e.get(t).unwrap().unwrap()).sum, 4);
}

#[test]
fn staleness_is_detected_even_without_any_notification_when_queried() {
    let w = world();
    w.put(L_VOXEL, bc(0, 0, 0), 1);
    let e = executor(&w, vec![SumProvider::plain(1)], ExecutorConfig::default());
    let t = target(1, L_VOXEL, 0, 0, 0);
    e.submit(t, prio(PriorityClass::Visible)).unwrap();
    e.run_until_idle();
    w.put(L_VOXEL, bc(0, 0, 0), 2);
    assert_eq!(e.query(t), Ok(TargetStatus::Stale));
}

#[test]
fn invalidation_follows_the_dependency_chain_and_resubmission_recomputes_both() {
    let w = world();
    w.put(L_VOXEL, bc(0, 0, 0), 5);
    let e = executor(
        &w,
        vec![SumProvider::plain(1), DoubleOf::of(2, 1)],
        ExecutorConfig::default(),
    );
    let (t1, t2) = (target(1, L_VOXEL, 0, 0, 0), target(2, L_VOXEL, 0, 0, 0));
    e.submit(t2, prio(PriorityClass::Visible)).unwrap();
    e.run_until_idle();
    let out = w.put(L_VOXEL, bc(0, 0, 0), 6);
    let affected = e.notify_commit(&out.dirty);
    assert!(affected.contains(&t1) && affected.contains(&t2));
    assert_eq!(e.query(t1), Ok(TargetStatus::Stale));
    assert_eq!(e.query(t2), Ok(TargetStatus::Stale));
    e.submit(t2, prio(PriorityClass::Visible)).unwrap();
    assert_eq!(e.run_until_idle(), 2);
    assert_eq!(tagged(&e.get(t2).unwrap().unwrap()).sum, 12);
}

#[test]
fn a_dependent_is_stale_when_its_input_changes_outside_its_own_brick_window() {
    // Kind 2 reads no cells itself (apron 0) but depends on kind 1 (apron 1): a change in a
    // neighbour of kind 1's window must still invalidate kind 2 through the derived input.
    let w = world();
    w.put(L_VOXEL, bc(0, 0, 0), 5);
    let e = executor(
        &w,
        vec![SumProvider::plain(1), DoubleOf::of(2, 1)],
        ExecutorConfig::default(),
    );
    let (t1, t2) = (target(1, L_VOXEL, 0, 0, 0), target(2, L_VOXEL, 0, 0, 0));
    e.submit(t2, prio(PriorityClass::Visible)).unwrap();
    e.run_until_idle();
    assert_eq!(e.query(t2), Ok(TargetStatus::Valid));
    w.put(L_VOXEL, bc(1, 1, 1), 9); // inside kind 1's apron, outside kind 2's own (empty) window
    assert_eq!(e.query(t1), Ok(TargetStatus::Stale));
    assert_eq!(e.query(t2), Ok(TargetStatus::Stale));
}

#[test]
fn workers_produce_the_same_results_for_any_thread_count() {
    let run = |threads: usize| {
        let w = world();
        for x in -3..3 {
            for y in -3..3 {
                w.put(L_VOXEL, bc(x, y, 0), (x * 7 + y * 3 + 20) as u16);
            }
        }
        let e = executor(
            &w,
            vec![SumProvider::plain(1), DoubleOf::of(2, 1)],
            ExecutorConfig::default(),
        );
        let workers = e.spawn_workers(threads);
        let handles: Vec<_> = (-3..3)
            .flat_map(|x| (-3..3).map(move |y| (x, y)))
            .map(|(x, y)| {
                e.submit(target(2, L_VOXEL, x, y, 0), prio(PriorityClass::Visible))
                    .unwrap()
            })
            .collect();
        for h in &handles {
            assert!(matches!(h.wait(), TaskOutcome::Published { .. }));
        }
        let mut out = Vec::new();
        for x in -3..3 {
            for y in -3..3 {
                out.push(tagged(
                    &e.get(target(2, L_VOXEL, x, y, 0)).unwrap().unwrap(),
                ));
            }
        }
        workers.stop();
        out
    };
    let single = run(1);
    assert_eq!(single, run(4));
    assert_eq!(single, run(8));
}

#[test]
fn stopping_the_workers_cancels_pending_work_and_joins() {
    let w = world();
    let started = Gate::new();
    let e = executor(
        &w,
        vec![Arc::new(Odd {
            kind: 1,
            mode: Mode::SpinUntilCancelled,
            started: Arc::clone(&started),
        })],
        ExecutorConfig::default(),
    );
    let workers = e.spawn_workers(1);
    let running = e
        .submit(target(1, L_VOXEL, 0, 0, 0), prio(PriorityClass::Visible))
        .unwrap();
    started.wait();
    let waiting = e
        .submit(target(1, L_VOXEL, 1, 0, 0), prio(PriorityClass::Visible))
        .unwrap();
    workers.stop();
    assert_eq!(
        running.outcome(),
        Some(TaskOutcome::Cancelled(CancelReason::Shutdown))
    );
    assert_eq!(
        waiting.outcome(),
        Some(TaskOutcome::Cancelled(CancelReason::Shutdown))
    );
}
