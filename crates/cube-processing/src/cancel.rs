//! Cooperative cancellation and deadlines (PROC-04, PR-05).
//!
//! A token is a one-way latch: once cancelled it stays cancelled and keeps its first reason.
//! Child tokens are cancelled with their parent. Callbacks run exactly once, outside any lock, and
//! never after their [`Registration`] was dropped. Deadlines are evaluated against a [`Clock`]
//! port (Processing never reads the system clock itself).
//!
//! **Latency bound:** cancellation is cooperative. A running provider observes it at its next
//! `is_cancelled()` / `check()` call, so the latency is the longest gap between two checks that
//! the provider promises (the contract asks for at most one brick of work). A deadline becomes
//! visible at the first check after the clock passes it.

use std::sync::{Arc, Weak};

use crate::sync::{AtomicBool, Mutex, Ordering, lock};

/// Why a token was cancelled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CancelReason {
    /// Explicit request by the submitter.
    Requested,
    /// The inputs changed: a newer state makes this work obsolete.
    Superseded,
    /// A task this one depends on was cancelled.
    DependencyCancelled,
    /// A task this one depends on failed.
    DependencyFailed,
    /// The deadline (maximum age) passed.
    DeadlineExceeded,
    /// A resource limit forced cancellation.
    ResourceLimit,
    /// The executor is shutting down.
    Shutdown,
}

/// Port: time source in milliseconds on a monotonic scale chosen by the host.
pub trait Clock: Send + Sync {
    /// Current time in milliseconds.
    fn now_ms(&self) -> u64;
}

/// Manually advanced clock for tests and deterministic hosts.
#[derive(Debug, Default)]
pub struct ManualClock {
    now: std::sync::atomic::AtomicU64,
}

impl ManualClock {
    /// A clock at `ms`.
    pub fn new(ms: u64) -> Self {
        Self {
            now: std::sync::atomic::AtomicU64::new(ms),
        }
    }
    /// Moves the clock forward by `ms` (saturating).
    pub fn advance(&self, ms: u64) {
        use std::sync::atomic::Ordering as O;
        let _ = self
            .now
            .fetch_update(O::SeqCst, O::SeqCst, |v| Some(v.saturating_add(ms)));
    }
}

impl Clock for ManualClock {
    fn now_ms(&self) -> u64 {
        self.now.load(std::sync::atomic::Ordering::SeqCst)
    }
}

type Callback = Box<dyn FnOnce(CancelReason) + Send>;

struct Inner {
    reason: Option<CancelReason>,
    next_id: u64,
    callbacks: Vec<(u64, Callback)>,
    children: Vec<Weak<State>>,
}

struct Deadline {
    clock: Arc<dyn Clock>,
    at_ms: u64,
}

struct State {
    flag: AtomicBool,
    inner: Mutex<Inner>,
    deadline: Option<Deadline>,
}

impl State {
    fn new(deadline: Option<Deadline>) -> Arc<Self> {
        Arc::new(Self {
            flag: AtomicBool::new(false),
            inner: Mutex::new(Inner {
                reason: None,
                next_id: 0,
                callbacks: Vec::new(),
                children: Vec::new(),
            }),
            deadline,
        })
    }

    fn cancel(self: &Arc<Self>, reason: CancelReason) -> bool {
        let (callbacks, children) = {
            let mut g = lock(&self.inner);
            if g.reason.is_some() {
                return false;
            }
            g.reason = Some(reason);
            self.flag.store(true, Ordering::SeqCst);
            (
                std::mem::take(&mut g.callbacks),
                std::mem::take(&mut g.children),
            )
        };
        for (_, cb) in callbacks {
            cb(reason);
        }
        for child in children.into_iter().filter_map(|w| w.upgrade()) {
            child.cancel(reason);
        }
        true
    }
}

/// A cancellation latch shared by all its clones.
#[derive(Clone)]
pub struct CancellationToken {
    state: Arc<State>,
}

impl std::fmt::Debug for CancellationToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CancellationToken")
            .field("reason", &self.reason())
            .finish()
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

impl CancellationToken {
    /// A fresh, uncancelled token without deadline.
    pub fn new() -> Self {
        Self {
            state: State::new(None),
        }
    }

    /// A fresh token that cancels itself with [`CancelReason::DeadlineExceeded`] at the first check
    /// at or after `at_ms` on `clock`.
    pub fn with_deadline(clock: Arc<dyn Clock>, at_ms: u64) -> Self {
        Self {
            state: State::new(Some(Deadline { clock, at_ms })),
        }
    }

    /// A child token: cancelled whenever this token is (and keeps the deadline, if any). Cancelling
    /// the child does not affect the parent.
    pub fn child(&self) -> Self {
        let deadline = self.state.deadline.as_ref().map(|d| Deadline {
            clock: Arc::clone(&d.clock),
            at_ms: d.at_ms,
        });
        let child = State::new(deadline);
        let parent_reason = {
            let mut g = lock(&self.state.inner);
            match g.reason {
                Some(r) => Some(r),
                None => {
                    g.children.retain(|w| w.strong_count() > 0);
                    g.children.push(Arc::downgrade(&child));
                    None
                }
            }
        };
        if let Some(r) = parent_reason {
            child.cancel(r);
        }
        Self { state: child }
    }

    /// Requests cancellation. Returns `true` if this call performed the cancellation, `false` if
    /// the token was already cancelled (the first reason is kept).
    pub fn cancel(&self, reason: CancelReason) -> bool {
        self.state.cancel(reason)
    }

    fn poll_deadline(&self) {
        if let Some(d) = &self.state.deadline
            && !self.state.flag.load(Ordering::SeqCst)
            && d.clock.now_ms() >= d.at_ms
        {
            self.state.cancel(CancelReason::DeadlineExceeded);
        }
    }

    /// `true` once cancelled (also evaluates the deadline).
    pub fn is_cancelled(&self) -> bool {
        self.poll_deadline();
        self.state.flag.load(Ordering::SeqCst)
    }

    /// The reason, `None` while not cancelled (also evaluates the deadline).
    pub fn reason(&self) -> Option<CancelReason> {
        self.poll_deadline();
        lock(&self.state.inner).reason
    }

    /// `Err(reason)` if cancelled, for use with `?` inside providers.
    pub fn check(&self) -> Result<(), CancelReason> {
        match self.reason() {
            Some(r) => Err(r),
            None => Ok(()),
        }
    }

    /// Registers `callback` to run once on cancellation. If the token is already cancelled it runs
    /// immediately on the calling thread. Dropping the returned [`Registration`] before the
    /// cancellation unregisters the callback, so it never runs afterwards.
    pub fn register(&self, callback: impl FnOnce(CancelReason) + Send + 'static) -> Registration {
        let reason = {
            let mut g = lock(&self.state.inner);
            match g.reason {
                None => {
                    let id = g.next_id;
                    g.next_id += 1;
                    g.callbacks.push((id, Box::new(callback)));
                    return Registration {
                        state: Arc::downgrade(&self.state),
                        id,
                    };
                }
                Some(r) => r,
            }
        };
        callback(reason);
        Registration {
            state: Weak::new(),
            id: u64::MAX,
        }
    }
}

/// Guard of a registered callback; dropping it unregisters the callback if it has not run.
#[must_use = "dropping the registration unregisters the callback"]
pub struct Registration {
    state: Weak<State>,
    id: u64,
}

impl Drop for Registration {
    fn drop(&mut self) {
        if let Some(s) = self.state.upgrade() {
            lock(&s.inner).callbacks.retain(|(i, _)| *i != self.id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering as O};

    #[test]
    fn latch_keeps_the_first_reason() {
        let t = CancellationToken::new();
        assert!(!t.is_cancelled() && t.reason().is_none() && t.check().is_ok());
        assert!(t.cancel(CancelReason::Requested));
        assert!(!t.cancel(CancelReason::Shutdown));
        assert_eq!(t.reason(), Some(CancelReason::Requested));
        assert_eq!(t.check(), Err(CancelReason::Requested));
        assert!(t.clone().is_cancelled(), "clones share state");
    }

    #[test]
    fn parent_cancels_children_but_not_the_reverse() {
        let p = CancellationToken::new();
        let c = p.child();
        let gc = c.child();
        c.cancel(CancelReason::Requested);
        assert!(gc.is_cancelled() && !p.is_cancelled());
        let c2 = p.child();
        p.cancel(CancelReason::Superseded);
        assert_eq!(c2.reason(), Some(CancelReason::Superseded));
        let late = p.child();
        assert_eq!(late.reason(), Some(CancelReason::Superseded));
    }

    #[test]
    fn callbacks_run_once_immediately_when_already_cancelled_and_never_after_drop() {
        let n = Arc::new(AtomicUsize::new(0));
        let t = CancellationToken::new();
        let n1 = Arc::clone(&n);
        let _r1 = t.register(move |_| {
            n1.fetch_add(1, O::SeqCst);
        });
        let n2 = Arc::clone(&n);
        drop(t.register(move |_| {
            n2.fetch_add(100, O::SeqCst);
        }));
        t.cancel(CancelReason::Requested);
        t.cancel(CancelReason::Requested);
        assert_eq!(n.load(O::SeqCst), 1);
        let n3 = Arc::clone(&n);
        let _r3 = t.register(move |r| {
            assert_eq!(r, CancelReason::Requested);
            n3.fetch_add(10, O::SeqCst);
        });
        assert_eq!(n.load(O::SeqCst), 11);
    }

    #[test]
    fn callbacks_of_children_run_when_the_parent_cancels() {
        let n = Arc::new(AtomicUsize::new(0));
        let p = CancellationToken::new();
        let c = p.child();
        let n1 = Arc::clone(&n);
        let _r = c.register(move |_| {
            n1.fetch_add(1, O::SeqCst);
        });
        p.cancel(CancelReason::Shutdown);
        assert_eq!(n.load(O::SeqCst), 1);
    }

    #[test]
    fn callback_may_use_the_token_without_deadlock() {
        let t = CancellationToken::new();
        let t2 = t.clone();
        let _r = t.register(move |_| {
            assert!(t2.is_cancelled());
            let _extra = t2.register(|_| {});
        });
        t.cancel(CancelReason::Requested);
    }

    #[test]
    fn deadline_is_checked_against_the_clock_and_inherited_by_children() {
        let clock = Arc::new(ManualClock::new(100));
        let t = CancellationToken::with_deadline(clock.clone(), 150);
        let c = t.child();
        assert!(!t.is_cancelled() && !c.is_cancelled());
        clock.advance(49);
        assert!(!t.is_cancelled());
        clock.advance(1);
        assert_eq!(c.reason(), Some(CancelReason::DeadlineExceeded));
        assert_eq!(t.reason(), Some(CancelReason::DeadlineExceeded));
    }

    #[test]
    fn explicit_cancel_wins_over_a_later_deadline() {
        let clock = Arc::new(ManualClock::new(0));
        let t = CancellationToken::with_deadline(clock.clone(), 10);
        t.cancel(CancelReason::Requested);
        clock.advance(100);
        assert_eq!(t.reason(), Some(CancelReason::Requested));
    }

    #[test]
    fn concurrent_cancels_have_exactly_one_winner() {
        let t = CancellationToken::new();
        let wins = Arc::new(AtomicUsize::new(0));
        let hs: Vec<_> = (0..8)
            .map(|_| {
                let t = t.clone();
                let w = Arc::clone(&wins);
                std::thread::spawn(move || {
                    if t.cancel(CancelReason::Requested) {
                        w.fetch_add(1, O::SeqCst);
                    }
                })
            })
            .collect();
        for h in hs {
            h.join().unwrap();
        }
        assert_eq!(wins.load(O::SeqCst), 1);
    }
}
