//! Task model: states, priorities and the handle (PROC-03, PR-03/PR-06).
//!
//! State machine (terminal states are final):
//!
//! ```text
//! Created -> Queued -> Running -> Completed | Failed | Cancelled | Stale
//!               \-> Cancelled            (cancelled or made obsolete while waiting)
//! Created -> Failed | Cancelled          (rejected before it was queued)
//! ```
//!
//! `Completed` means *published*. `Stale` means the work finished but its result was rejected
//! because newer inputs exist; nothing was published.

use std::sync::{Arc, PoisonError};

use crate::cancel::{CancelReason, CancellationToken};
use crate::error::ProcessingError;
use crate::provider::DerivedTarget;
use crate::snapshot::DependencyRevision;
use crate::sync::{Condvar, Mutex, lock};

/// Identifier of a task, unique per executor, increasing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TaskId(pub u64);

/// Lifecycle state of a task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskState {
    /// Accepted but not yet queued.
    Created,
    /// Waiting for a worker or for its dependencies.
    Queued,
    /// Being executed.
    Running,
    /// Finished and published.
    Completed,
    /// Finished with an error; nothing was published.
    Failed,
    /// Stopped by cancellation; nothing was published.
    Cancelled,
    /// Finished but rejected as stale; nothing was published.
    Stale,
}

impl TaskState {
    /// `true` for the four final states.
    pub const fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Failed | Self::Cancelled | Self::Stale
        )
    }

    /// `true` if the state machine allows `self -> to`.
    pub const fn can_become(self, to: Self) -> bool {
        use TaskState::*;
        matches!(
            (self, to),
            (Created, Queued | Failed | Cancelled)
                | (Queued, Running | Cancelled)
                | (Running, Completed | Failed | Cancelled | Stale)
        )
    }
}

/// Scheduling class (PR-06), highest first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PriorityClass {
    /// Directly caused by a user action.
    Interactive,
    /// Needed for what is on screen.
    Visible,
    /// Needed by gameplay.
    Gameplay,
    /// Normal background work.
    Background,
    /// Housekeeping.
    Maintenance,
}

impl PriorityClass {
    /// Number of classes.
    pub const COUNT: usize = 5;
    /// Index `0..COUNT`, 0 = highest.
    pub const fn rank(self) -> usize {
        self as usize
    }
    /// The class with rank `r` (clamped to the lowest).
    pub const fn from_rank(r: usize) -> Self {
        match r {
            0 => Self::Interactive,
            1 => Self::Visible,
            2 => Self::Gameplay,
            3 => Self::Background,
            _ => Self::Maintenance,
        }
    }
}

/// Class plus weight within the class (larger = sooner, for example nearer to the camera).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Priority {
    /// Scheduling class.
    pub class: PriorityClass,
    /// Weight within the class.
    pub weight: u32,
}

impl Priority {
    /// A priority of `class` with weight 0.
    pub const fn new(class: PriorityClass) -> Self {
        Self { class, weight: 0 }
    }
    /// `true` if `self` is scheduled before `other` ignoring aging.
    pub fn outranks(&self, other: &Self) -> bool {
        (self.class, std::cmp::Reverse(self.weight))
            < (other.class, std::cmp::Reverse(other.weight))
    }
}

/// How a task ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskOutcome {
    /// The result was published as publication number `sequence` of its slot.
    Published {
        /// Publication number.
        sequence: u64,
    },
    /// The result was rejected: newer inputs exist.
    Stale,
    /// The task failed; carries the error.
    Failed(ProcessingError),
    /// The task was cancelled for the given reason.
    Cancelled(CancelReason),
}

impl TaskOutcome {
    const fn state(&self) -> TaskState {
        match self {
            Self::Published { .. } => TaskState::Completed,
            Self::Stale => TaskState::Stale,
            Self::Failed(_) => TaskState::Failed,
            Self::Cancelled(_) => TaskState::Cancelled,
        }
    }
}

struct TaskInner {
    state: TaskState,
    outcome: Option<TaskOutcome>,
    captured: Option<DependencyRevision>,
}

pub(crate) trait Wake: Send + Sync {
    fn wake(&self);
}

pub(crate) struct TaskShared {
    pub(crate) id: TaskId,
    pub(crate) target: DerivedTarget,
    pub(crate) token: CancellationToken,
    inner: Mutex<TaskInner>,
    cv: Condvar,
}

impl TaskShared {
    pub(crate) fn new(id: TaskId, target: DerivedTarget, token: CancellationToken) -> Self {
        Self {
            id,
            target,
            token,
            inner: Mutex::new(TaskInner {
                state: TaskState::Created,
                outcome: None,
                captured: None,
            }),
            cv: Condvar::new(),
        }
    }

    pub(crate) fn state(&self) -> TaskState {
        lock(&self.inner).state
    }

    pub(crate) fn outcome(&self) -> Option<TaskOutcome> {
        lock(&self.inner).outcome.clone()
    }

    pub(crate) fn captured(&self) -> Option<DependencyRevision> {
        lock(&self.inner).captured.clone()
    }

    pub(crate) fn set_captured(&self, d: DependencyRevision) {
        lock(&self.inner).captured = Some(d);
    }

    /// Moves to a non-terminal `to` if legal.
    pub(crate) fn advance(&self, to: TaskState) -> bool {
        debug_assert!(!to.is_terminal());
        let mut g = lock(&self.inner);
        if g.state.can_become(to) {
            g.state = to;
            true
        } else {
            false
        }
    }

    /// Ends the task with `outcome` if the transition is legal; a terminal task is never changed.
    pub(crate) fn finish(&self, outcome: TaskOutcome) -> bool {
        let to = outcome.state();
        let mut g = lock(&self.inner);
        if !g.state.can_become(to) {
            return false;
        }
        g.state = to;
        g.outcome = Some(outcome);
        drop(g);
        self.cv.notify_all();
        true
    }

    /// Cancels a task that has not started: `Queued -> Cancelled` (or `Created -> Cancelled`).
    /// Returns `false` if it already started or ended; a running task is stopped through its
    /// token by the worker, never ended from outside.
    pub(crate) fn cancel_queued(&self, reason: CancelReason) -> bool {
        let mut g = lock(&self.inner);
        if !matches!(g.state, TaskState::Queued | TaskState::Created) {
            return false;
        }
        self.token.cancel(reason);
        let r = self.token.reason().unwrap_or(reason);
        g.state = TaskState::Cancelled;
        g.outcome = Some(TaskOutcome::Cancelled(r));
        drop(g);
        self.cv.notify_all();
        true
    }

    fn wait(&self) -> TaskOutcome {
        let mut g = lock(&self.inner);
        loop {
            if let Some(o) = &g.outcome {
                return o.clone();
            }
            g = self.cv.wait(g).unwrap_or_else(PoisonError::into_inner);
        }
    }
}

/// The submitter's view of a task: read its status, wait for it, cancel it. Safe from any thread.
#[derive(Clone)]
pub struct TaskHandle {
    pub(crate) shared: Arc<TaskShared>,
    pub(crate) waker: Arc<dyn Wake>,
}

impl std::fmt::Debug for TaskHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaskHandle")
            .field("id", &self.shared.id)
            .field("target", &self.shared.target)
            .field("state", &self.shared.state())
            .finish()
    }
}

impl TaskHandle {
    /// The task id.
    pub fn id(&self) -> TaskId {
        self.shared.id
    }
    /// What the task computes.
    pub fn target(&self) -> DerivedTarget {
        self.shared.target
    }
    /// Current state.
    pub fn state(&self) -> TaskState {
        self.shared.state()
    }
    /// How the task ended, `None` while it is not terminal.
    pub fn outcome(&self) -> Option<TaskOutcome> {
        self.shared.outcome()
    }
    /// Requests cancellation. A queued task ends at once; a running task ends when its provider
    /// next checks the token; cancelling a terminal task does nothing.
    pub fn cancel(&self) {
        if !self.shared.cancel_queued(CancelReason::Requested) {
            self.shared.token.cancel(CancelReason::Requested);
        }
        self.waker.wake();
    }
    /// Blocks until the task is terminal and returns its outcome. Needs a worker (or another
    /// thread calling `run_one`) to make progress.
    pub fn wait(&self) -> TaskOutcome {
        self.shared.wait()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [TaskState; 7] = [
        TaskState::Created,
        TaskState::Queued,
        TaskState::Running,
        TaskState::Completed,
        TaskState::Failed,
        TaskState::Cancelled,
        TaskState::Stale,
    ];

    #[test]
    fn terminal_states_are_final() {
        for s in ALL.into_iter().filter(|s| s.is_terminal()) {
            for t in ALL {
                assert!(!s.can_become(t), "{s:?} -> {t:?} must be illegal");
            }
        }
    }

    #[test]
    fn only_the_documented_transitions_are_legal() {
        use TaskState::*;
        let legal = [
            (Created, Queued),
            (Created, Failed),
            (Created, Cancelled),
            (Queued, Running),
            (Queued, Cancelled),
            (Running, Completed),
            (Running, Failed),
            (Running, Cancelled),
            (Running, Stale),
        ];
        for a in ALL {
            for b in ALL {
                assert_eq!(a.can_become(b), legal.contains(&(a, b)), "{a:?} -> {b:?}");
            }
        }
    }

    #[test]
    fn priority_order_is_class_then_weight() {
        let p = |c, w| Priority {
            class: c,
            weight: w,
        };
        assert!(p(PriorityClass::Interactive, 0).outranks(&p(PriorityClass::Visible, 99)));
        assert!(p(PriorityClass::Visible, 5).outranks(&p(PriorityClass::Visible, 4)));
        assert!(!p(PriorityClass::Visible, 4).outranks(&p(PriorityClass::Visible, 4)));
        for r in 0..PriorityClass::COUNT {
            assert_eq!(PriorityClass::from_rank(r).rank(), r);
        }
        assert_eq!(PriorityClass::from_rank(99), PriorityClass::Maintenance);
    }
}
