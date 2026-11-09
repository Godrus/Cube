//! TEST-08 (stress part): Processing under concurrent commits, cancellation storms and queue
//! pressure. The model-checked part is in `processing_loom.rs`.
//!
//! Invariants checked: no stale value is ever visible as current; a snapshot is never torn;
//! every task ends in exactly one terminal state; nothing is published by a cancelled, failed or
//! stale task; limits are respected; random dependency graphs are ordered or reported as cyclic.
#![cfg(not(loom))]

mod common;

use std::collections::BTreeSet;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use common::*;
use cube_processing::{
    CancelReason, ChangeMask, ComputeContext, DerivedKind, Executor, ExecutorConfig, LimitKind,
    ManualClock, Payload, Priority, PriorityClass, ProcessingError, Provider, ProviderFailure,
    ProviderRegistry, RegistryError, TargetStatus, TaskOutcome, TaskState,
};
use cube_testkit::{DetRng, for_each_seed};

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

fn visible() -> Priority {
    Priority::new(PriorityClass::Visible)
}

fn tagged(p: &Payload) -> Tagged {
    *p.downcast_ref::<Tagged>().expect("Tagged payload")
}

#[test]
fn stress_no_stale_value_is_ever_visible_and_no_snapshot_is_torn() {
    const COMMITS: u16 = 300;
    let w = Arc::new(world());
    let (a, b) = (bc(0, 0, 0), bc(1, 0, 0));
    w.put_pair(a, b, 1);
    let e = executor(&w, vec![SumProvider::plain(1)], ExecutorConfig::default());
    let workers = e.spawn_workers(4);
    let done = Arc::new(AtomicBool::new(false));
    let t = target(1, L_VOXEL, 0, 0, 0);

    // One writer: pairs are always equal, so a torn snapshot shows as an odd sum.
    let writer = {
        let (w, e) = (Arc::clone(&w), e.clone());
        std::thread::spawn(move || {
            for m in 2..=COMMITS {
                let out = w.put_pair(a, b, m);
                e.notify_commit(&out.dirty);
            }
        })
    };
    // One submitter keeps recomputation going.
    let submitter = {
        let (e, done) = (e.clone(), Arc::clone(&done));
        std::thread::spawn(move || {
            while !done.load(Ordering::SeqCst) {
                let _ = e.submit(t, visible());
                std::thread::yield_now();
            }
        })
    };
    // Readers: a current value must carry the revision the centre brick had when it was read.
    let violations = Arc::new(AtomicUsize::new(0));
    let seen = Arc::new(AtomicUsize::new(0));
    let readers: Vec<_> = (0..3)
        .map(|_| {
            let (w, e, done) = (Arc::clone(&w), e.clone(), Arc::clone(&done));
            let (violations, seen) = (Arc::clone(&violations), Arc::clone(&seen));
            std::thread::spawn(move || {
                while !done.load(Ordering::SeqCst) {
                    let before = w.rev(a).unwrap_or(0);
                    let got = e.get(t).unwrap();
                    let after = w.rev(a).unwrap_or(0);
                    if let Some(p) = got {
                        seen.fetch_add(1, Ordering::SeqCst);
                        let v = tagged(&p);
                        let rev = v.revision.unwrap_or(0);
                        let fresh = before <= rev && rev <= after;
                        let whole = v.present == 2 && v.sum.is_multiple_of(2);
                        if !(fresh && whole) {
                            violations.fetch_add(1, Ordering::SeqCst);
                            eprintln!("violation: {v:?} before={before} after={after}");
                        }
                    }
                }
            })
        })
        .collect();
    writer.join().unwrap();
    done.store(true, Ordering::SeqCst);
    submitter.join().unwrap();
    for r in readers {
        r.join().unwrap();
    }
    assert_eq!(violations.load(Ordering::SeqCst), 0);

    // Quiesce: after the last commit one more computation gives exactly the final state.
    let h = loop {
        match e.submit(t, visible()) {
            Ok(h) => break h,
            Err(ProcessingError::ResourceUnavailable(_)) => std::thread::yield_now(),
            Err(other) => panic!("{other}"),
        }
    };
    h.wait();
    let last = loop {
        if let Some(p) = e.get(t).unwrap() {
            break tagged(&p);
        }
        let _ = e.submit(t, visible()).map(|h| h.wait());
    };
    assert_eq!(last.sum, 2 * u64::from(COMMITS));
    assert_eq!(e.query(t), Ok(TargetStatus::Valid));
    workers.stop();
    eprintln!("current values observed: {}", seen.load(Ordering::SeqCst));
}

#[test]
fn stress_cancellation_storm_leaves_every_task_in_exactly_one_terminal_state() {
    let w = Arc::new(world());
    for x in 0..8 {
        w.put(L_VOXEL, bc(x, 0, 0), 1);
    }
    let e = executor(&w, vec![SumProvider::plain(1)], ExecutorConfig::default());
    let workers = e.spawn_workers(4);
    let handles = Arc::new(Mutex::new(Vec::new()));
    let submitters: Vec<_> = (0..4u64)
        .map(|seed| {
            let (e, handles) = (e.clone(), Arc::clone(&handles));
            std::thread::spawn(move || {
                let mut rng = DetRng::new(seed);
                for _ in 0..300 {
                    let x = rng.below(8) as i64;
                    if let Ok(h) = e.submit(target(1, L_VOXEL, x, 0, 0), visible()) {
                        if rng.below(3) == 0 {
                            h.cancel();
                        }
                        handles.lock().unwrap().push(h);
                    }
                }
            })
        })
        .collect();
    for s in submitters {
        s.join().unwrap();
    }
    let all = handles.lock().unwrap().clone();
    for h in &all {
        let o = h.wait();
        match (&o, h.state()) {
            (TaskOutcome::Published { .. }, TaskState::Completed)
            | (TaskOutcome::Stale, TaskState::Stale)
            | (TaskOutcome::Cancelled(_), TaskState::Cancelled) => {}
            (o, s) => panic!("inconsistent end: {o:?} / {s:?}"),
        }
        assert!(
            !matches!(o, TaskOutcome::Failed(_)),
            "nothing in this scenario may fail: {o:?}"
        );
    }
    workers.stop();
    assert_eq!(e.stats().running, 0);
    assert_eq!(e.stats().queued, 0);
    // A cancelled task that was the only one for its brick published nothing: every Valid slot
    // has a published task behind it.
    let published: BTreeSet<_> = all
        .iter()
        .filter(|h| matches!(h.outcome(), Some(TaskOutcome::Published { .. })))
        .map(|h| h.target())
        .collect();
    for x in 0..8 {
        let t = target(1, L_VOXEL, x, 0, 0);
        if e.query(t) == Ok(TargetStatus::Valid) {
            assert!(published.contains(&t));
        }
    }
}

#[test]
fn stress_queue_pressure_never_loses_or_duplicates_accepted_work() {
    let w = Arc::new(world());
    let cfg = ExecutorConfig {
        max_queued: 16,
        ..ExecutorConfig::default()
    };
    let e = executor(&w, vec![SumProvider::plain(1)], cfg);
    let workers = e.spawn_workers(2);
    let accepted = Arc::new(Mutex::new(Vec::new()));
    let rejected = Arc::new(AtomicUsize::new(0));
    let hs: Vec<_> = (0..4i64)
        .map(|n| {
            let (e, accepted, rejected) = (e.clone(), Arc::clone(&accepted), Arc::clone(&rejected));
            std::thread::spawn(move || {
                for i in 0..200i64 {
                    match e.submit(target(1, L_VOXEL, n, i, 0), visible()) {
                        Ok(h) => accepted.lock().unwrap().push(h),
                        Err(ProcessingError::ResourceUnavailable(LimitKind::QueueFull)) => {
                            rejected.fetch_add(1, Ordering::SeqCst);
                        }
                        Err(other) => panic!("unexpected: {other}"),
                    }
                }
            })
        })
        .collect();
    for h in hs {
        h.join().unwrap();
    }
    let accepted = accepted.lock().unwrap().clone();
    let ids: BTreeSet<_> = accepted.iter().map(|h| h.id()).collect();
    assert_eq!(
        ids.len(),
        accepted.len(),
        "distinct targets: no coalescing, no duplicates"
    );
    for h in &accepted {
        assert!(matches!(h.wait(), TaskOutcome::Published { .. }));
    }
    workers.stop();
    assert_eq!(accepted.len() + rejected.load(Ordering::SeqCst), 800);
    assert!(
        rejected.load(Ordering::SeqCst) > 0,
        "the queue limit of 16 should have been hit"
    );
}

#[test]
fn stress_a_cancelled_dependency_chain_leaves_no_partial_results() {
    let w = world();
    w.put(L_VOXEL, bc(0, 0, 0), 3);
    let e = executor(
        &w,
        vec![
            SumProvider::plain(1),
            DoubleOf::of(2, 1),
            DoubleOf::of(3, 2),
        ],
        ExecutorConfig::default(),
    );
    let mut m: u16 = 3;
    for round in 0..50 {
        let top = target(3, L_VOXEL, 0, 0, 0);
        let h3 = e.submit(top, visible()).unwrap();
        let h1 = e.submit(target(1, L_VOXEL, 0, 0, 0), visible()).unwrap();
        if round % 2 == 0 {
            h1.cancel();
            assert!(matches!(
                h3.outcome(),
                Some(TaskOutcome::Cancelled(CancelReason::DependencyCancelled))
            ));
            e.run_until_idle();
            assert!(e.get(top).unwrap().is_none());
        } else {
            e.run_until_idle();
            assert_eq!(tagged(&e.get(top).unwrap().unwrap()).sum, 4 * u64::from(m));
        }
        // Change the brick: the whole chain is invalidated and must be recomputed from scratch.
        m += 1;
        let out = w.put(L_VOXEL, bc(0, 0, 0), m);
        e.notify_commit(&out.dirty);
        assert!(matches!(
            e.query(top),
            Ok(TargetStatus::Stale | TargetStatus::Missing)
        ));
    }
}

struct Node {
    kind: u32,
    deps: Vec<DerivedKind>,
}

impl Provider for Node {
    fn kind(&self) -> DerivedKind {
        DerivedKind(self.kind)
    }
    fn name(&self) -> &str {
        "node"
    }
    fn version(&self) -> u32 {
        1
    }
    fn consumes(&self) -> ChangeMask {
        ChangeMask::ALL
    }
    fn depends_on(&self) -> &[DerivedKind] {
        &self.deps
    }
    fn compute(&self, _: &ComputeContext<'_>) -> Result<Payload, ProviderFailure> {
        Ok(Arc::new(self.kind))
    }
}

fn build(
    edges: &[(u32, u32)],
    n: u32,
) -> Result<cube_processing::DependencyGraph, Vec<RegistryError>> {
    let mut r = ProviderRegistry::new();
    for k in 0..n {
        let deps = edges
            .iter()
            .filter(|(from, _)| *from == k)
            .map(|(_, to)| DerivedKind(*to))
            .collect();
        r.register(Arc::new(Node { kind: k, deps })).unwrap();
    }
    r.freeze()
}

#[test]
fn property_random_acyclic_graphs_are_ordered_and_back_edges_are_cycles() {
    for_each_seed(0xDA6, 300, |seed| {
        let mut rng = DetRng::new(seed);
        let n = 2 + rng.below(10) as u32;
        // Edges only from a higher to a lower kind id: acyclic by construction.
        let mut edges = BTreeSet::new();
        for _ in 0..rng.below(20) {
            let a = 1 + rng.below(u64::from(n) - 1) as u32;
            let b = rng.below(u64::from(a)) as u32;
            edges.insert((a, b));
        }
        let edges: Vec<_> = edges.into_iter().collect();
        let g = build(&edges, n).expect("acyclic graph freezes");
        let pos = |k: u32| g.order().iter().position(|x| x.0 == k).unwrap();
        assert_eq!(g.order().len(), n as usize);
        for (from, to) in &edges {
            assert!(pos(*to) < pos(*from), "{to} must come before {from}");
        }
        // Order does not depend on the order edges were listed.
        let mut rev = edges.clone();
        rev.reverse();
        assert_eq!(build(&rev, n).unwrap().order(), g.order());

        // Adding the reverse of an existing edge makes a cycle that is reported.
        if let Some((from, to)) = edges.first().copied() {
            let mut cyc = edges.clone();
            cyc.push((to, from));
            let err = build(&cyc, n).unwrap_err();
            let [RegistryError::Cycle { path }] = err.as_slice() else {
                panic!("expected a single cycle error, got {err:?}");
            };
            assert_eq!(path.first(), path.last());
            // Every step of the reported path is a real dependency edge.
            let all: BTreeSet<(u32, u32)> = cyc.iter().copied().collect();
            for w2 in path.windows(2) {
                assert!(all.contains(&(w2[0].0, w2[1].0)), "{:?} is not an edge", w2);
            }
        }
    });
}
