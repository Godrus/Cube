//! PROC-06: resource caps (pinned bricks, pinned bytes, task lifetime), the synthetic mobile
//! budget and the brick-bounds slot. Cap values are provisional (TC-01); the tests pin the
//! behaviour of the mechanism, not the numbers.
#![cfg(not(loom))]

mod common;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use common::*;
use cube_core::{BrickCoord, ChangeType, GeometryBounds};
use cube_processing::{
    BOUNDS_KIND, BoundsProvider, CancelReason, Caps, ChangeMask, ComputeContext, DerivedKind,
    DerivedTarget, Executor, ExecutorConfig, LimitKind, ManualClock, Payload, Priority,
    PriorityClass, ProcessingError, Provider, ProviderFailure, ProviderRegistry, TargetStatus,
    TaskOutcome, TaskState, union_all,
};

fn visible() -> Priority {
    Priority::new(PriorityClass::Visible)
}

fn executor_with(
    w: &World,
    providers: Vec<Arc<dyn Provider>>,
    cfg: ExecutorConfig,
    clock: &Arc<ManualClock>,
) -> Executor {
    let mut r = ProviderRegistry::new();
    for p in providers {
        r.register(p).unwrap();
    }
    Executor::new(w.reader.clone(), r.freeze().unwrap(), cfg, clock.clone())
}

fn executor(w: &World, providers: Vec<Arc<dyn Provider>>, cfg: ExecutorConfig) -> Executor {
    executor_with(w, providers, cfg, &Arc::new(ManualClock::new(0)))
}

fn caps(bricks: u64, bytes: u64, ttl: Option<u64>) -> Caps {
    Caps {
        max_snapshot_bricks: u64::MAX,
        max_snapshot_bytes: u64::MAX,
        max_active_snapshots: u64::MAX,
        max_pinned_bricks: bricks,
        max_pinned_bytes: bytes,
        task_ttl_ms: ttl,
    }
}

fn cfg(c: Caps) -> ExecutorConfig {
    ExecutorConfig {
        caps: c,
        ..ExecutorConfig::default()
    }
}

#[test]
fn a_request_that_can_never_fit_is_refused_at_submit() {
    let w = world();
    let e = executor(
        &w,
        vec![SumProvider::plain(1)], // apron 1: 27 bricks
        cfg(caps(26, u64::MAX, None)),
    );
    assert_eq!(
        e.submit(target(1, L_VOXEL, 0, 0, 0), visible())
            .unwrap_err(),
        ProcessingError::ResourceUnavailable(LimitKind::PinnedBricks)
    );
    assert_eq!(e.stats().queued, 0);
    let e = executor(
        &w,
        vec![SumProvider::plain(1)],
        cfg(caps(27, u64::MAX, None)),
    );
    assert!(e.submit(target(1, L_VOXEL, 0, 0, 0), visible()).is_ok());
}

#[test]
fn the_brick_budget_makes_tasks_that_do_not_fit_together_wait() {
    let w = world();
    let entered = Gate::new();
    let release = Gate::new();
    let e = executor(
        &w,
        vec![SumProvider::paused(1, &entered, &release, true)],
        cfg(caps(27, u64::MAX, None)), // exactly one apron-1 snapshot at a time
    );
    let h1 = e.submit(target(1, L_VOXEL, 0, 0, 0), visible()).unwrap();
    let h2 = e.submit(target(1, L_VOXEL, 5, 0, 0), visible()).unwrap();
    let runner = {
        let e = e.clone();
        std::thread::spawn(move || e.run_one())
    };
    entered.wait();
    assert_eq!(e.pinned().bricks, 27);
    assert!(
        !e.run_one(),
        "the second snapshot does not fit while the first is pinned"
    );
    assert_eq!(h2.state(), TaskState::Queued);
    release.open();
    assert!(runner.join().unwrap());
    assert!(e.run_one(), "it runs as soon as the pins are returned");
    assert_eq!(h1.state(), TaskState::Completed);
    assert_eq!(h2.state(), TaskState::Completed);
    let u = e.pinned();
    assert_eq!((u.bricks, u.bytes, u.peak_bricks), (0, 0, 27));
}

#[test]
fn a_smaller_task_may_pass_one_that_does_not_fit_yet() {
    let w = world();
    let entered = Gate::new();
    let release = Gate::new();
    // Kind 1 pins 27 bricks (apron 1), kind 2 pins 1 (apron 0); the budget holds 28.
    let e = executor(
        &w,
        vec![
            SumProvider::paused(1, &entered, &release, true),
            Arc::new(Counting::new(2, 0)),
        ],
        cfg(caps(28, u64::MAX, None)),
    );
    let big1 = e.submit(target(1, L_VOXEL, 0, 0, 0), visible()).unwrap();
    let big2 = e.submit(target(1, L_VOXEL, 9, 0, 0), visible()).unwrap();
    let small = e
        .submit(
            target(2, L_VOXEL, 4, 0, 0),
            Priority::new(PriorityClass::Background),
        )
        .unwrap();
    let runner = {
        let e = e.clone();
        std::thread::spawn(move || e.run_one())
    };
    entered.wait();
    // The second big task cannot start (27 + 27 > 28) but the small one fits next to the first.
    assert!(e.run_one());
    assert_eq!(small.state(), TaskState::Completed);
    assert_eq!(big2.state(), TaskState::Queued);
    release.open();
    runner.join().unwrap();
    assert!(e.run_one());
    assert_eq!(big1.state(), TaskState::Completed);
    assert_eq!(big2.state(), TaskState::Completed);
}

#[test]
fn exceeding_the_byte_cap_fails_the_task_and_returns_every_pin() {
    let w = world();
    for x in 0..3 {
        w.put_dense(L_VOXEL, bc(x, 0, 0), 1);
    }
    let t = target(1, L_VOXEL, 1, 0, 0); // its neighbourhood holds three dense bricks
    let small = executor(
        &w,
        vec![SumProvider::plain(1)],
        cfg(caps(100, 20_000, None)),
    );
    let h = small.submit(t, visible()).unwrap();
    small.run_until_idle();
    assert_eq!(
        h.outcome(),
        Some(TaskOutcome::Failed(ProcessingError::ResourceUnavailable(
            LimitKind::PinnedBytes
        )))
    );
    assert_eq!(
        small.query(t),
        Ok(TargetStatus::Missing),
        "nothing was published"
    );
    let u = small.pinned();
    assert_eq!((u.bricks, u.bytes), (0, 0));

    let roomy = executor(
        &w,
        vec![SumProvider::plain(1)],
        cfg(caps(100, 200_000, None)),
    );
    let h = roomy.submit(t, visible()).unwrap();
    roomy.run_until_idle();
    assert!(matches!(h.outcome(), Some(TaskOutcome::Published { .. })));
    let u = roomy.pinned();
    assert_eq!((u.bricks, u.bytes), (0, 0));
    assert!(
        u.peak_bytes >= 3 * 8192,
        "dense bricks are counted at 2 bytes per voxel"
    );
}

#[test]
fn pins_are_returned_on_failure_cancellation_and_panic() {
    let w = world();
    let started = Gate::new();
    let spin_started = Gate::new();
    let e = executor(
        &w,
        vec![
            Arc::new(Odd {
                kind: 1,
                mode: Mode::Fail,
                started: Arc::clone(&started),
            }),
            Arc::new(Odd {
                kind: 2,
                mode: Mode::Panic,
                started: Arc::clone(&started),
            }),
            Arc::new(Odd {
                kind: 3,
                mode: Mode::SpinUntilCancelled,
                started: Arc::clone(&spin_started),
            }),
        ],
        cfg(caps(10, u64::MAX, None)),
    );
    for kind in [1, 2] {
        e.submit(target(kind, L_VOXEL, 0, 0, 0), visible()).unwrap();
        e.run_until_idle();
        assert_eq!(e.pinned().bricks, 0, "kind {kind}");
    }
    let h = e.submit(target(3, L_VOXEL, 0, 0, 0), visible()).unwrap();
    let runner = {
        let e = e.clone();
        std::thread::spawn(move || e.run_one())
    };
    spin_started.wait();
    assert_eq!(e.pinned().bricks, 1);
    h.cancel();
    runner.join().unwrap();
    assert_eq!(e.pinned().bricks, 0);
    assert_eq!(e.pinned().peak_bricks, 1);
}

#[test]
fn a_task_older_than_the_ttl_is_cancelled_whether_it_waits_or_runs() {
    let w = world();
    let clock = Arc::new(ManualClock::new(1_000));
    let started = Gate::new();
    let log = Arc::new(Mutex::new(Vec::new()));
    let e = executor_with(
        &w,
        vec![
            Arc::new(Odd {
                kind: 1,
                mode: Mode::SpinUntilCancelled,
                started: Arc::clone(&started),
            }),
            Arc::new(Logged {
                kind: 2,
                log: Arc::clone(&log),
            }),
        ],
        cfg(caps(100, u64::MAX, Some(100))),
        &clock,
    );
    // Waiting: expires in the queue and the provider never runs.
    let waiting = e.submit(target(2, L_VOXEL, 0, 0, 0), visible()).unwrap();
    clock.advance(100);
    assert!(!e.run_one());
    assert_eq!(
        waiting.outcome(),
        Some(TaskOutcome::Cancelled(CancelReason::DeadlineExceeded))
    );
    assert!(log.lock().unwrap().is_empty());

    // Running: the provider notices the deadline at its next check.
    let t = target(1, L_VOXEL, 1, 0, 0);
    let running = e.submit(t, visible()).unwrap();
    let runner = {
        let e = e.clone();
        std::thread::spawn(move || e.run_one())
    };
    started.wait();
    assert_eq!(running.state(), TaskState::Running);
    clock.advance(100);
    runner.join().unwrap();
    assert_eq!(
        running.outcome(),
        Some(TaskOutcome::Cancelled(CancelReason::DeadlineExceeded))
    );
    assert_eq!(e.query(t), Ok(TargetStatus::Missing));
    assert_eq!(e.pinned().bricks, 0);

    // A task inside its lifetime is unaffected.
    let fresh = e.submit(target(2, L_VOXEL, 2, 0, 0), visible()).unwrap();
    clock.advance(99);
    e.run_until_idle();
    assert!(matches!(
        fresh.outcome(),
        Some(TaskOutcome::Published { .. })
    ));
}

/// Counts concurrent computations and remembers the peak.
struct Counting {
    kind: u32,
    apron: u32,
    now: AtomicUsize,
    peak: AtomicUsize,
}

impl Counting {
    fn new(kind: u32, apron: u32) -> Self {
        Self {
            kind,
            apron,
            now: AtomicUsize::new(0),
            peak: AtomicUsize::new(0),
        }
    }
}

impl Provider for Counting {
    fn kind(&self) -> DerivedKind {
        DerivedKind(self.kind)
    }
    fn name(&self) -> &str {
        "counting"
    }
    fn version(&self) -> u32 {
        1
    }
    fn consumes(&self) -> ChangeMask {
        ChangeMask::ALL
    }
    fn apron(&self) -> u32 {
        self.apron
    }
    fn compute(&self, ctx: &ComputeContext<'_>) -> Result<Payload, ProviderFailure> {
        let n = self.now.fetch_add(1, Ordering::SeqCst) + 1;
        self.peak.fetch_max(n, Ordering::SeqCst);
        for _ in 0..50 {
            std::thread::yield_now();
        }
        ctx.token.check()?;
        self.now.fetch_sub(1, Ordering::SeqCst);
        Ok(Arc::new(n))
    }
}

struct Logged {
    kind: u32,
    log: Arc<Mutex<Vec<i64>>>,
}

impl Provider for Logged {
    fn kind(&self) -> DerivedKind {
        DerivedKind(self.kind)
    }
    fn name(&self) -> &str {
        "logged"
    }
    fn version(&self) -> u32 {
        1
    }
    fn consumes(&self) -> ChangeMask {
        ChangeMask::ALL
    }
    fn compute(&self, ctx: &ComputeContext<'_>) -> Result<Payload, ProviderFailure> {
        self.log.lock().unwrap().push(ctx.target.key.brick().x);
        Ok(Arc::new(()))
    }
}

#[test]
fn the_synthetic_mobile_budget_holds_under_load() {
    let w = world();
    let counting = Arc::new(Counting::new(1, 1));
    let provider: Arc<dyn Provider> = counting.clone();
    let config = ExecutorConfig::synthetic_mobile();
    let (max_running, max_bricks, max_queued) = (
        config.max_running,
        config.caps.max_pinned_bricks,
        config.max_queued,
    );
    let e = executor(&w, vec![provider], config);
    let workers = e.spawn_workers(6); // more threads than the budget allows to run at once
    let mut handles = Vec::new();
    for i in 0..600i64 {
        let t = target(1, L_VOXEL, i % 50, i / 50, 0);
        loop {
            match e.submit(t, visible()) {
                Ok(h) => {
                    handles.push(h);
                    break;
                }
                Err(ProcessingError::ResourceUnavailable(LimitKind::QueueFull)) => {
                    std::thread::yield_now();
                }
                Err(other) => panic!("{other}"),
            }
        }
        assert!(e.stats().queued <= max_queued);
        assert!(e.stats().running <= max_running);
    }
    for h in &handles {
        assert!(matches!(h.wait(), TaskOutcome::Published { .. }));
    }
    workers.stop();
    let u = e.pinned();
    assert_eq!((u.bricks, u.bytes), (0, 0));
    assert!(u.peak_bricks <= max_bricks);
    assert!(u.peak_bricks >= 27, "tasks did pin their neighbourhoods");
    assert!(
        counting.peak.load(Ordering::SeqCst) <= max_running,
        "no more tasks ran at once than the budget allows"
    );
}

fn bounds_executor(w: &World) -> Executor {
    executor(w, vec![Arc::new(BoundsProvider)], ExecutorConfig::default())
}

fn bounds_of(e: &Executor, t: DerivedTarget) -> GeometryBounds {
    *e.get(t)
        .unwrap()
        .unwrap()
        .downcast_ref::<GeometryBounds>()
        .unwrap()
}

#[test]
fn bounds_never_turn_unloaded_or_failed_data_into_empty() {
    let w = world();
    w.put(L_VOXEL, bc(0, 0, 0), 1);
    w.empty(L_VOXEL, bc(1, 0, 0));
    w.put(L_VOXEL, bc(-3, -3, -3), 1);
    let e = bounds_executor(&w);
    let bt = |x, y, z| DerivedTarget {
        kind: BOUNDS_KIND,
        key: key(L_VOXEL, x, y, z),
    };
    let handles: Vec<_> = [bt(0, 0, 0), bt(1, 0, 0), bt(2, 0, 0), bt(-3, -3, -3)]
        .into_iter()
        .map(|t| e.submit(t, visible()).unwrap())
        .collect();
    e.run_until_idle();
    assert_eq!(
        bounds_of(&e, bt(0, 0, 0)),
        GeometryBounds::from_brick(0, bc(0, 0, 0)).unwrap()
    );
    assert_eq!(bounds_of(&e, bt(1, 0, 0)), GeometryBounds::Empty);
    assert_eq!(
        bounds_of(&e, bt(-3, -3, -3)),
        GeometryBounds::from_brick(0, bc(-3, -3, -3)).unwrap()
    );
    // Brick 2 was never loaded: the bounds are unknown, a failure, never Empty.
    assert!(matches!(
        handles[2].outcome(),
        Some(TaskOutcome::Failed(ProcessingError::ProviderFailed(m))) if m.contains("not loaded")
    ));
    assert_eq!(e.query(bt(2, 0, 0)), Ok(TargetStatus::Missing));
    assert!(e.get(bt(2, 0, 0)).unwrap().is_none());
}

#[test]
fn bounds_of_a_brick_beyond_the_layer_bounds_are_empty() {
    let w = world();
    let e = bounds_executor(&w);
    let beyond = DerivedTarget {
        kind: BOUNDS_KIND,
        key: key(L_BOUNDED, 7, 0, 0),
    };
    e.submit(beyond, visible()).unwrap();
    e.run_until_idle();
    assert_eq!(bounds_of(&e, beyond), GeometryBounds::Empty);
}

#[test]
fn bounds_follow_geometry_changes_and_material_changes_do_not_change_them() {
    let w = world();
    w.put(L_VOXEL, bc(0, 0, 0), 1);
    let e = bounds_executor(&w);
    let t = DerivedTarget {
        kind: BOUNDS_KIND,
        key: key(L_VOXEL, 0, 0, 0),
    };
    e.submit(t, visible()).unwrap();
    e.run_until_idle();
    assert_eq!(e.query(t), Ok(TargetStatus::Valid));

    let material = w.put(L_VOXEL, bc(0, 0, 0), 2);
    assert!(
        material
            .dirty
            .iter()
            .all(|d| d.change == ChangeType::Material),
        "same occupancy, different material: {:?}",
        material.dirty.iter().map(|d| d.change).collect::<Vec<_>>()
    );
    // The change mask only narrows eager invalidation; validity is decided by the per-brick
    // stamp (TC-01), so the result is conservatively stale on query and recomputes unchanged.
    assert!(!e.notify_commit(&material.dirty).contains(&t));
    assert_eq!(e.query(t), Ok(TargetStatus::Stale));
    e.submit(t, visible()).unwrap();
    e.run_until_idle();
    assert_eq!(e.query(t), Ok(TargetStatus::Valid));
    assert_eq!(
        bounds_of(&e, t),
        GeometryBounds::from_brick(0, bc(0, 0, 0)).unwrap()
    );

    let emptied = w.empty(L_VOXEL, bc(0, 0, 0));
    assert!(e.notify_commit(&emptied.dirty).contains(&t));
    assert_eq!(e.query(t), Ok(TargetStatus::Stale));
    e.submit(t, visible()).unwrap();
    e.run_until_idle();
    assert_eq!(bounds_of(&e, t), GeometryBounds::Empty);
}

#[test]
fn bounds_of_many_bricks_combine_exactly_and_conservatively() {
    let w = world();
    let bricks = [bc(-2, 0, 0), bc(0, 0, 0), bc(3, 1, 0)];
    for b in bricks {
        w.put(L_VOXEL, b, 1);
    }
    let e = bounds_executor(&w);
    let ts: Vec<_> = bricks
        .iter()
        .map(|b| DerivedTarget {
            kind: BOUNDS_KIND,
            key: key(L_VOXEL, b.x, b.y, b.z),
        })
        .collect();
    for t in &ts {
        e.submit(*t, visible()).unwrap();
    }
    e.run_until_idle();
    let hull = union_all(ts.iter().map(|t| bounds_of(&e, *t))).unwrap();
    for b in bricks {
        assert!(hull.contains(&GeometryBounds::from_brick(0, b).unwrap()));
    }
    let r = hull.region().unwrap();
    assert_eq!(
        (r.min().x, r.max().x, r.min().y, r.max().y),
        (-32, 64, 0, 32),
        "negative bricks extend the hull downwards exactly"
    );
    let _ = BrickCoord::new(0, 0, 0);
}

#[test]
fn per_snapshot_and_active_snapshot_budgets_refuse_with_a_named_limit() {
    let w = world();
    // One snapshot of 27 bricks against a per-snapshot limit of 26: refused at submit.
    let tight = Caps {
        max_snapshot_bricks: 26,
        ..Caps::unlimited()
    };
    let e = executor(&w, vec![SumProvider::plain(1)], cfg(tight));
    assert_eq!(
        e.submit(target(1, L_VOXEL, 0, 0, 0), visible())
            .unwrap_err(),
        ProcessingError::ResourceUnavailable(LimitKind::SnapshotBricks)
    );

    // Per-snapshot byte budget: a task whose neighbourhood is too heavy fails, diagnosably.
    for x in 0..3 {
        w.put_dense(L_VOXEL, bc(x, 0, 0), 1);
    }
    let heavy = Caps {
        max_snapshot_bytes: 20_000,
        ..Caps::unlimited()
    };
    let e = executor(&w, vec![SumProvider::plain(1)], cfg(heavy));
    let h = e.submit(target(1, L_VOXEL, 1, 0, 0), visible()).unwrap();
    e.run_until_idle();
    assert_eq!(
        h.outcome(),
        Some(TaskOutcome::Failed(ProcessingError::ResourceUnavailable(
            LimitKind::SnapshotBytes
        )))
    );
    assert_eq!(e.pinned().snapshots, 0);

    // Active snapshots: one at a time even though the executor could run more tasks.
    let entered = Gate::new();
    let release = Gate::new();
    let one = Caps {
        max_active_snapshots: 1,
        ..Caps::unlimited()
    };
    let e = executor(
        &w,
        vec![SumProvider::paused(1, &entered, &release, true)],
        cfg(one),
    );
    let h1 = e.submit(target(1, L_VOXEL, 0, 0, 0), visible()).unwrap();
    let h2 = e.submit(target(1, L_VOXEL, 9, 0, 0), visible()).unwrap();
    let runner = {
        let e = e.clone();
        std::thread::spawn(move || e.run_one())
    };
    entered.wait();
    assert_eq!(e.pinned().snapshots, 1);
    assert!(!e.run_one());
    assert_eq!(h2.state(), TaskState::Queued);
    release.open();
    runner.join().unwrap();
    assert!(e.run_one());
    assert_eq!(h1.state(), TaskState::Completed);
    assert_eq!(h2.state(), TaskState::Completed);
    let u = e.pinned();
    assert_eq!((u.snapshots, u.peak_snapshots), (0, 1));
}
