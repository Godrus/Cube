//! Dirty propagation and observer notification (ORCH-04).
//!
//! After a commit the actual dirty regions go to the dirty sink (Processing invalidation) first,
//! then observers are told in registration order. Both happen after Core published the new
//! state, so observers see post-commit state only. Observers receive immutable data and have no
//! way to commit; a panicking observer is isolated and counted, it never undoes the commit or
//! stops the others. Notifications are not durable: a lost notification never loses a change
//! (reliable subscribers will need replay/resync, DEC-086).

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use cube_core::DirtyRegion;
use cube_editing_api::{GeometryDelta, TransactionObserver, TransactionSummary};

/// Where dirty regions go after a commit (Processing).
pub trait DirtySink: Send + Sync {
    /// The regions that actually changed in one commit.
    fn dirty(&self, regions: &[DirtyRegion]);
}

impl DirtySink for cube_processing::Executor {
    fn dirty(&self, regions: &[DirtyRegion]) {
        self.notify_commit(regions);
    }
}

/// Delivers notifications in a fixed order and isolates failing observers.
pub(crate) struct Publisher {
    sink: Option<Arc<dyn DirtySink>>,
    observers: Vec<Arc<dyn TransactionObserver>>,
    failures: AtomicU64,
}

impl Publisher {
    pub(crate) fn new(
        sink: Option<Arc<dyn DirtySink>>,
        observers: Vec<Arc<dyn TransactionObserver>>,
    ) -> Self {
        Self {
            sink,
            observers,
            failures: AtomicU64::new(0),
        }
    }

    pub(crate) fn failures(&self) -> u64 {
        self.failures.load(Ordering::SeqCst)
    }

    pub(crate) fn committed(&self, summary: &TransactionSummary, delta: &GeometryDelta) {
        if let Some(sink) = &self.sink
            && catch_unwind(AssertUnwindSafe(|| sink.dirty(&summary.dirty))).is_err()
        {
            self.failures.fetch_add(1, Ordering::SeqCst);
        }
        for o in &self.observers {
            if catch_unwind(AssertUnwindSafe(|| o.on_committed(summary, delta))).is_err() {
                self.failures.fetch_add(1, Ordering::SeqCst);
            }
        }
    }

    pub(crate) fn rolled_back(&self, summary: &TransactionSummary) {
        for o in &self.observers {
            if catch_unwind(AssertUnwindSafe(|| o.on_rolled_back(summary))).is_err() {
                self.failures.fetch_add(1, Ordering::SeqCst);
            }
        }
    }
}
