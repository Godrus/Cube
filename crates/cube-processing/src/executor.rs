//! The Processing executor (PROC-03/PROC-04/PROC-05 wiring, PR-03/PR-04/PR-05/PR-06).
//!
//! It runs provider computations on snapshots and publishes their results into derived slots.
//! It has no async runtime: [`Executor::run_one`] executes one task on the calling thread, and
//! [`Executor::spawn_workers`] runs that loop on plain threads. It never writes to Core: its only
//! access is the read-only `SharedAsset` (enforced by `cargo xtask archcheck`).
//!
//! # Guarantees
//!
//! * **Snapshot isolation:** a task computes from one immutable [`Snapshot`].
//! * **Stale never published:** the only way into a slot is [`DerivedSlot::publish`], which
//!   compares inputs under the slot lock (see `slot`). A task that finishes after its inputs
//!   changed ends as `Stale`.
//! * **Errors are errors:** a failing or panicking provider ends the task as `Failed`; nothing
//!   is published and no empty value stands in for it.
//! * **Cancellation:** at every stage (queued, before capture, during compute, before publish).
//!   Cancelling after the publish step began has no effect: the task is then terminal.
//! * **Dependencies:** a task for a kind with derived inputs waits for tasks producing them;
//!   if one of them fails, is cancelled or goes stale, the waiting task is cancelled.
//! * **Determinism:** for equal state and equal provider outputs the published values are equal
//!   whatever the thread count; the *order* in which independent tasks run is fixed only for a
//!   single caller of `run_one` (priority, then aging, then task id).
//! * **No hidden work:** a result rejected as stale or a slot invalidated by a commit is not
//!   recomputed automatically; the owner of the data resubmits.

use std::any::Any;
use std::collections::BTreeMap;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;
#[cfg(not(loom))]
use std::sync::PoisonError;

use cube_core::{DirtyRegion, SharedAsset};

use crate::cancel::{CancelReason, CancellationToken, Clock};
use crate::error::{LimitKind, ProcessingError};
use crate::graph::DependencyGraph;
use crate::provider::{
    ComputeContext, DerivedKind, DerivedTarget, Payload, Provider, ProviderFailure,
};
use crate::slot::{DerivedSlot, PublishOutcome, SlotStatus};
use crate::snapshot::{DependencyRevision, Snapshot, SnapshotRequest, stamp};
use crate::sync::{Condvar, Mutex, lock};
use crate::task::{
    Priority, PriorityClass, TaskHandle, TaskId, TaskOutcome, TaskShared, TaskState, Wake,
};

type Slot = DerivedSlot<dyn Any + Send + Sync>;

/// Limits and policy of an executor. Values are provisional constants (TC-01, tunable): they are
/// resource limits of the machine, never limits of the game world.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutorConfig {
    /// Most tasks running at the same time.
    pub max_running: usize,
    /// Most tasks of each class running at the same time (index = class rank).
    pub class_running_limits: [usize; PriorityClass::COUNT],
    /// Most tasks waiting; a full queue rejects new tasks (they are not silently dropped).
    pub max_queued: usize,
    /// Scheduling passes after which a waiting task is promoted by one class (starvation guard).
    pub aging_ticks: u64,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            max_running: 4,
            class_running_limits: [4, 4, 3, 2, 1],
            max_queued: 4096,
            aging_ticks: 64,
        }
    }
}

/// Validity of one derived item as seen by [`Executor::query`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetStatus {
    /// Never computed.
    Missing,
    /// A value exists and is current.
    Valid,
    /// A value exists but newer inputs exist.
    Stale,
    /// A task for it is queued or running.
    Computing,
}

/// Counters for tests and diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stats {
    /// Tasks waiting.
    pub queued: usize,
    /// Tasks running.
    pub running: usize,
}

struct Entry {
    task: Arc<TaskShared>,
    priority: Priority,
    enqueued: u64,
    deps: Vec<Arc<TaskShared>>,
}

struct Sched {
    queue: Vec<Entry>,
    running: usize,
    running_by_class: [usize; PriorityClass::COUNT],
    active: BTreeMap<DerivedTarget, Arc<TaskShared>>,
    next_id: u64,
    tick: u64,
    shutdown: bool,
}

struct Inner {
    asset: SharedAsset,
    graph: DependencyGraph,
    config: ExecutorConfig,
    clock: Arc<dyn Clock>,
    slots: Mutex<BTreeMap<DerivedTarget, Arc<Slot>>>,
    sched: Mutex<Sched>,
    cv: Condvar,
}

/// Handle to an executor; clones share it.
#[derive(Clone)]
pub struct Executor {
    inner: Arc<Inner>,
}

impl std::fmt::Debug for Executor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Executor")
            .field("stats", &self.stats())
            .finish_non_exhaustive()
    }
}

impl Executor {
    /// Creates an executor over a frozen graph, reading `asset`.
    pub fn new(
        asset: SharedAsset,
        graph: DependencyGraph,
        config: ExecutorConfig,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            inner: Arc::new(Inner {
                asset,
                graph,
                config,
                clock,
                slots: Mutex::new(BTreeMap::new()),
                sched: Mutex::new(Sched {
                    queue: Vec::new(),
                    running: 0,
                    running_by_class: [0; PriorityClass::COUNT],
                    active: BTreeMap::new(),
                    next_id: 1,
                    tick: 0,
                    shutdown: false,
                }),
                cv: Condvar::new(),
            }),
        }
    }

    /// The clock port in use.
    pub fn clock(&self) -> &Arc<dyn Clock> {
        &self.inner.clock
    }

    /// The configuration.
    pub fn config(&self) -> &ExecutorConfig {
        &self.inner.config
    }

    /// Submits the computation of `target`. If a task for the same target is already queued or
    /// running, that task is returned (its priority is raised if `priority` is higher): equal
    /// requests are coalesced. Tasks for the derived inputs of the provider are submitted too,
    /// unless their results are current.
    pub fn submit(
        &self,
        target: DerivedTarget,
        priority: Priority,
    ) -> Result<TaskHandle, ProcessingError> {
        let mut s = lock(&self.inner.sched);
        let shared = self.inner.submit_locked(&mut s, target, priority)?;
        drop(s);
        self.inner.cv.notify_all();
        Ok(self.handle(shared))
    }

    fn handle(&self, shared: Arc<TaskShared>) -> TaskHandle {
        TaskHandle {
            shared,
            waker: self.inner.clone() as Arc<dyn Wake>,
        }
    }

    /// Tells the executor that a commit changed `dirty`. Every derived item whose inputs include a
    /// dirty brick is told its new live inputs (so older results can no longer be published) and
    /// running tasks that read the old inputs are cancelled as superseded. Returns the affected
    /// items in a deterministic order. Items are not recomputed (see the module documentation).
    pub fn notify_commit(&self, dirty: &[DirtyRegion]) -> Vec<DerivedTarget> {
        let targets = self.inner.graph.invalidated_by(dirty);
        let mut by_kind: BTreeMap<DerivedKind, Vec<DerivedTarget>> = BTreeMap::new();
        for t in &targets {
            by_kind.entry(t.kind).or_default().push(*t);
        }
        // Upstream kinds first: the live stamp of a derived item depends on its inputs' slots.
        for kind in self.inner.graph.order() {
            for t in by_kind.get(kind).into_iter().flatten() {
                self.inner.invalidate_one(*t);
            }
        }
        targets
    }

    /// Validity of `target`.
    pub fn query(&self, target: DerivedTarget) -> Result<TargetStatus, ProcessingError> {
        let slot = self.inner.refreshed_slot(target)?;
        let computing = lock(&self.inner.sched)
            .active
            .get(&target)
            .is_some_and(|t| !t.state().is_terminal());
        Ok(if computing {
            TargetStatus::Computing
        } else {
            match slot.status() {
                SlotStatus::Missing => TargetStatus::Missing,
                SlotStatus::Valid => TargetStatus::Valid,
                SlotStatus::Stale => TargetStatus::Stale,
            }
        })
    }

    /// The current payload of `target`; `None` if missing or stale (a stale value is never
    /// returned here).
    pub fn get(&self, target: DerivedTarget) -> Result<Option<Payload>, ProcessingError> {
        let slot = self.inner.refreshed_slot(target)?;
        Ok(slot.current().map(|p| payload_of(&p.value)))
    }

    /// Queue and running counters.
    pub fn stats(&self) -> Stats {
        let s = lock(&self.inner.sched);
        Stats {
            queued: s
                .queue
                .iter()
                .filter(|e| !e.task.state().is_terminal())
                .count(),
            running: s.running,
        }
    }

    /// Runs one eligible task on the calling thread. Returns `false` if none is eligible (queue
    /// empty, dependencies pending, or concurrency limits reached).
    pub fn run_one(&self) -> bool {
        let entry = {
            let mut s = lock(&self.inner.sched);
            self.inner.pick(&mut s)
        };
        match entry {
            Some(e) => {
                self.inner.execute(e);
                true
            }
            None => false,
        }
    }

    /// Calls [`Executor::run_one`] until nothing is eligible. Returns the number of tasks run.
    pub fn run_until_idle(&self) -> usize {
        let mut n = 0;
        while self.run_one() {
            n += 1;
        }
        n
    }

    /// Stops accepting tasks, cancels everything queued and running (reason `Shutdown`) and
    /// wakes the workers so they exit.
    pub fn shutdown(&self) {
        let mut s = lock(&self.inner.sched);
        s.shutdown = true;
        for e in &s.queue {
            e.task.cancel_queued(CancelReason::Shutdown);
        }
        for t in s.active.values() {
            t.token.cancel(CancelReason::Shutdown);
        }
        self.inner.sweep(&mut s);
        drop(s);
        self.inner.cv.notify_all();
    }

    /// Starts `n` worker threads running tasks until [`Executor::shutdown`].
    #[cfg(not(loom))]
    pub fn spawn_workers(&self, n: usize) -> Workers {
        let threads = (0..n)
            .map(|_| {
                let inner = Arc::clone(&self.inner);
                std::thread::spawn(move || inner.worker_loop())
            })
            .collect();
        Workers {
            executor: self.clone(),
            threads,
        }
    }
}

/// Worker threads of an executor; dropping or calling [`Workers::stop`] shuts the executor down
/// and joins them.
#[cfg(not(loom))]
pub struct Workers {
    executor: Executor,
    threads: Vec<std::thread::JoinHandle<()>>,
}

#[cfg(not(loom))]
impl Workers {
    /// Shuts the executor down and waits for every worker to exit.
    pub fn stop(mut self) {
        self.join_all();
    }

    fn join_all(&mut self) {
        self.executor.shutdown();
        for t in self.threads.drain(..) {
            let _ = t.join();
        }
    }
}

#[cfg(not(loom))]
impl Drop for Workers {
    fn drop(&mut self) {
        self.join_all();
    }
}

fn payload_of(v: &Arc<dyn Any + Send + Sync>) -> Payload {
    Arc::clone(v)
}

impl Wake for Inner {
    fn wake(&self) {
        let mut s = lock(&self.sched);
        self.sweep(&mut s);
        drop(s);
        self.cv.notify_all();
    }
}

impl Inner {
    fn provider(&self, kind: DerivedKind) -> Result<&Arc<dyn Provider>, ProcessingError> {
        self.graph
            .provider(kind)
            .ok_or(ProcessingError::UnknownKind(kind))
    }

    fn slot(&self, target: DerivedTarget) -> Arc<Slot> {
        Arc::clone(lock(&self.slots).entry(target).or_default())
    }

    fn existing_slot(&self, target: DerivedTarget) -> Option<Arc<Slot>> {
        lock(&self.slots).get(&target).cloned()
    }

    /// Stamp of the live state for `target`, without derived inputs.
    fn state_stamp(&self, target: DerivedTarget) -> Result<DependencyRevision, ProcessingError> {
        let p = self.provider(target.kind)?;
        let req = SnapshotRequest::for_brick(target.key, p.apron());
        Ok(stamp(&self.asset.snapshot(), &req, None)?
            .with_provider(p.version(), p.parameters_hash()))
    }

    /// Publication sequences of the derived inputs of `target` that are current now (0 if not).
    fn derived_sequences(
        &self,
        target: DerivedTarget,
    ) -> Result<Vec<(DerivedKind, u64)>, ProcessingError> {
        let p = self.provider(target.kind)?;
        let mut out = Vec::new();
        for dk in p.depends_on() {
            let dt = DerivedTarget {
                kind: *dk,
                key: target.key,
            };
            let seq = self.refreshed_slot(dt)?.current().map_or(0, |c| c.sequence);
            out.push((*dk, seq));
        }
        Ok(out)
    }

    /// Full live stamp of `target`: live state plus the current derived inputs.
    fn live_stamp(&self, target: DerivedTarget) -> Result<DependencyRevision, ProcessingError> {
        Ok(self
            .state_stamp(target)?
            .with_derived(self.derived_sequences(target)?))
    }

    /// The slot of `target`, told the live inputs first.
    fn refreshed_slot(&self, target: DerivedTarget) -> Result<Arc<Slot>, ProcessingError> {
        let live = self.live_stamp(target)?;
        let slot = self.slot(target);
        slot.expect(&live);
        Ok(slot)
    }

    fn invalidate_one(&self, target: DerivedTarget) {
        let Ok(state) = self.state_stamp(target) else {
            return;
        };
        if let Some(slot) = self.existing_slot(target)
            && let Ok(live) = self.live_stamp(target)
        {
            slot.expect(&live);
        }
        let task = lock(&self.sched).active.get(&target).cloned();
        if let Some(t) = task
            && t.state() == TaskState::Running
            && let Some(c) = t.captured()
            && !c.same_inputs(&state)
        {
            t.token.cancel(CancelReason::Superseded);
        }
    }

    fn submit_locked(
        &self,
        s: &mut Sched,
        target: DerivedTarget,
        priority: Priority,
    ) -> Result<Arc<TaskShared>, ProcessingError> {
        if s.shutdown {
            return Err(ProcessingError::ShutDown);
        }
        let provider = Arc::clone(self.provider(target.kind)?);
        if let Some(t) = s.active.get(&target).map(Arc::clone)
            && !t.state().is_terminal()
        {
            for e in s.queue.iter_mut().filter(|e| e.task.id == t.id) {
                if priority.outranks(&e.priority) {
                    e.priority = priority;
                }
            }
            return Ok(t);
        }
        let mut deps = Vec::new();
        for dk in provider.depends_on() {
            let dt = DerivedTarget {
                kind: *dk,
                key: target.key,
            };
            let current = self.refreshed_slot(dt)?.current().is_some();
            if !current {
                deps.push(self.submit_locked(s, dt, priority)?);
            }
        }
        let waiting = s
            .queue
            .iter()
            .filter(|e| !e.task.state().is_terminal())
            .count();
        if waiting >= self.config.max_queued {
            return Err(ProcessingError::ResourceUnavailable(LimitKind::QueueFull));
        }
        let id = TaskId(s.next_id);
        s.next_id += 1;
        let task = Arc::new(TaskShared::new(id, target, CancellationToken::new()));
        task.advance(TaskState::Queued);
        s.queue.push(Entry {
            task: Arc::clone(&task),
            priority,
            enqueued: s.tick,
            deps,
        });
        s.active.insert(target, Arc::clone(&task));
        Ok(task)
    }

    /// Removes finished entries and cancels queued tasks whose dependencies did not complete.
    fn sweep(&self, s: &mut Sched) {
        loop {
            let mut changed = false;
            let snapshot: Vec<(Arc<TaskShared>, Option<CancelReason>)> = s
                .queue
                .iter()
                .map(|e| {
                    let bad = e
                        .deps
                        .iter()
                        .map(|d| d.state())
                        .find(|st| st.is_terminal() && *st != TaskState::Completed)
                        .map(|st| match st {
                            TaskState::Failed => CancelReason::DependencyFailed,
                            TaskState::Stale => CancelReason::Superseded,
                            _ => CancelReason::DependencyCancelled,
                        });
                    (Arc::clone(&e.task), bad)
                })
                .collect();
            for (task, bad) in snapshot {
                if let Some(reason) = bad
                    && task.cancel_queued(reason)
                {
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        let finished: Vec<Arc<TaskShared>> = s
            .queue
            .iter()
            .filter(|e| e.task.state().is_terminal())
            .map(|e| Arc::clone(&e.task))
            .collect();
        s.queue.retain(|e| !e.task.state().is_terminal());
        for t in finished {
            if s.active.get(&t.target).is_some_and(|a| a.id == t.id) {
                s.active.remove(&t.target);
            }
        }
    }

    /// Chooses the next eligible task and marks it running.
    fn pick(&self, s: &mut Sched) -> Option<Entry> {
        self.sweep(s);
        if s.shutdown || s.running >= self.config.max_running {
            return None;
        }
        s.tick += 1;
        let tick = s.tick;
        let aging = self.config.aging_ticks.max(1);
        let mut candidates: Vec<(usize, u32, TaskId, usize)> = s
            .queue
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                e.task.state() == TaskState::Queued
                    && e.deps.iter().all(|d| d.state() == TaskState::Completed)
                    && s.running_by_class[e.priority.class.rank()]
                        < self.config.class_running_limits[e.priority.class.rank()]
            })
            .map(|(i, e)| {
                let promoted = usize::try_from((tick - e.enqueued) / aging).unwrap_or(usize::MAX);
                let rank = e.priority.class.rank().saturating_sub(promoted);
                (rank, u32::MAX - e.priority.weight, e.task.id, i)
            })
            .collect();
        candidates.sort();
        for (_, _, _, i) in candidates {
            if s.queue[i].task.advance(TaskState::Running) {
                let e = s.queue.remove(i);
                s.running += 1;
                s.running_by_class[e.priority.class.rank()] += 1;
                return Some(e);
            }
        }
        None
    }

    fn execute(&self, entry: Entry) {
        let outcome = self.run_task(&entry.task);
        entry.task.finish(outcome);
        let mut s = lock(&self.sched);
        s.running -= 1;
        s.running_by_class[entry.priority.class.rank()] -= 1;
        if s.active
            .get(&entry.task.target)
            .is_some_and(|a| a.id == entry.task.id)
        {
            s.active.remove(&entry.task.target);
        }
        self.sweep(&mut s);
        drop(s);
        self.cv.notify_all();
    }

    fn run_task(&self, task: &Arc<TaskShared>) -> TaskOutcome {
        match self.try_run_task(task) {
            Ok(o) => o,
            Err(e) => TaskOutcome::Failed(e),
        }
    }

    fn try_run_task(&self, task: &Arc<TaskShared>) -> Result<TaskOutcome, ProcessingError> {
        let target = task.target;
        let cancelled = |t: &Arc<TaskShared>| t.token.reason().map(TaskOutcome::Cancelled);
        let provider = Arc::clone(self.provider(target.kind)?);
        if let Some(o) = cancelled(task) {
            return Ok(o);
        }

        // Inputs: current results of the derived kinds this provider needs.
        let mut inputs: Vec<(DerivedKind, Payload)> = Vec::new();
        for dk in provider.depends_on() {
            let dt = DerivedTarget {
                kind: *dk,
                key: target.key,
            };
            match self.refreshed_slot(dt)?.current() {
                Some(p) => inputs.push((*dk, payload_of(&p.value))),
                None => return Ok(TaskOutcome::Stale),
            }
        }
        let used_sequences = self.derived_sequences(target)?;

        // Snapshot.
        let req = SnapshotRequest::for_brick(target.key, provider.apron());
        let snapshot = Snapshot::capture(&self.asset, &req, None)?;
        let own: DependencyRevision = snapshot
            .dependency()
            .clone()
            .with_provider(provider.version(), provider.parameters_hash())
            .with_derived(used_sequences);
        task.set_captured(own.clone());
        if let Some(o) = cancelled(task) {
            return Ok(o);
        }

        // Compute (provider code runs without any lock held; a panic is a failure).
        let ctx = ComputeContext {
            target,
            snapshot: &snapshot,
            token: &task.token,
            inputs: &inputs,
        };
        let computed = catch_unwind(AssertUnwindSafe(|| provider.compute(&ctx)));
        let payload = match computed {
            Err(_) => {
                return Err(ProcessingError::ProviderFailed(
                    "provider panicked".to_owned(),
                ));
            }
            Ok(Err(ProviderFailure::Cancelled(r))) => {
                return Ok(TaskOutcome::Cancelled(task.token.reason().unwrap_or(r)));
            }
            Ok(Err(ProviderFailure::Failed(m))) => return Err(ProcessingError::ProviderFailed(m)),
            Ok(Ok(p)) => p,
        };
        if let Some(o) = cancelled(task) {
            return Ok(o);
        }

        // Publish: the slot compares inputs under its lock against the live state read now.
        let live = self.live_stamp(target)?;
        let slot = self.slot(target);
        Ok(match slot.publish(&own, &live, payload) {
            PublishOutcome::Published { sequence } => TaskOutcome::Published { sequence },
            PublishOutcome::Stale => TaskOutcome::Stale,
        })
    }

    #[cfg(not(loom))]
    fn worker_loop(self: Arc<Self>) {
        loop {
            let entry = {
                let mut s = lock(&self.sched);
                loop {
                    if s.shutdown {
                        return;
                    }
                    if let Some(e) = self.pick(&mut s) {
                        break e;
                    }
                    s = self.cv.wait(s).unwrap_or_else(PoisonError::into_inner);
                }
            };
            self.execute(entry);
        }
    }
}
