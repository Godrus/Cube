//! Resource caps (PROC-06, OD-R4 pending GOV-24; values are provisional TC-01 constants).
//!
//! The caps limit what *Processing* may hold at once: bricks and bytes pinned by running
//! snapshots, and how long a task may live (snapshot lifetime). They are limits of the machine,
//! never limits of the world: a world of any size is processed brick by brick within the budget.
//!
//! A budget is reserved before a snapshot is captured (by brick count, so an oversized request is
//! refused without touching any data) and topped up with the measured size in bytes after capture.
//! Reservations are released by `Drop`, so every path (success, failure, cancellation, panic)
//! returns them.

use std::sync::Arc;

use crate::error::LimitKind;
use crate::sync::{Mutex, lock};

/// Bookkeeping bytes counted per pinned brick on top of its payload (provisional estimate).
pub const BRICK_OVERHEAD_BYTES: u64 = 64;

/// Limits on pinned snapshot data and task lifetime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Caps {
    /// Most bricks pinned by all running snapshots together.
    pub max_pinned_bricks: u64,
    /// Most bytes (estimated from payload sizes) pinned by all running snapshots together.
    pub max_pinned_bytes: u64,
    /// Longest life of a task from submission, in milliseconds on the executor's clock; the task
    /// is cancelled with `DeadlineExceeded` when it is exceeded. `None` = no limit.
    pub task_ttl_ms: Option<u64>,
}

impl Caps {
    /// No effective limits (hard safety limits of snapshots still apply).
    pub const fn unlimited() -> Self {
        Self {
            max_pinned_bricks: u64::MAX,
            max_pinned_bytes: u64::MAX,
            task_ttl_ms: None,
        }
    }

    /// Synthetic mobile budget used by tests and by the platform-independent CI: small pins, a
    /// short life. Not a measurement of any device (device validation is DG-1, outside PS-2).
    pub const fn synthetic_mobile() -> Self {
        Self {
            max_pinned_bricks: 256,
            max_pinned_bytes: 8 * 1024 * 1024,
            task_ttl_ms: Some(5_000),
        }
    }
}

impl Default for Caps {
    fn default() -> Self {
        Self::unlimited()
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct Used {
    bricks: u64,
    bytes: u64,
    peak_bricks: u64,
    peak_bytes: u64,
}

/// Shared counters of pinned data.
#[derive(Debug)]
pub struct PinBudget {
    caps: Caps,
    used: Mutex<Used>,
}

/// Pinned amount and its high-water marks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PinUsage {
    /// Bricks pinned now.
    pub bricks: u64,
    /// Bytes pinned now.
    pub bytes: u64,
    /// Most bricks ever pinned at once.
    pub peak_bricks: u64,
    /// Most bytes ever pinned at once.
    pub peak_bytes: u64,
}

impl PinBudget {
    /// A budget with the given caps.
    pub fn new(caps: Caps) -> Arc<Self> {
        Arc::new(Self {
            caps,
            used: Mutex::new(Used::default()),
        })
    }

    /// The caps.
    pub fn caps(&self) -> &Caps {
        &self.caps
    }

    /// `true` if a request for `bricks` could ever fit (when nothing else is pinned).
    pub fn fits_ever(&self, bricks: u128) -> bool {
        bricks <= u128::from(self.caps.max_pinned_bricks)
    }

    /// Reserves `bricks` if they fit now, otherwise returns `None` (the caller waits and retries).
    pub fn try_reserve(self: &Arc<Self>, bricks: u64) -> Option<PinReservation> {
        let mut u = lock(&self.used);
        let total = u.bricks.checked_add(bricks)?;
        if total > self.caps.max_pinned_bricks {
            return None;
        }
        u.bricks = total;
        u.peak_bricks = u.peak_bricks.max(total);
        Some(PinReservation {
            budget: Arc::clone(self),
            bricks,
            bytes: 0,
        })
    }

    /// Current usage and peaks.
    pub fn usage(&self) -> PinUsage {
        let u = lock(&self.used);
        PinUsage {
            bricks: u.bricks,
            bytes: u.bytes,
            peak_bricks: u.peak_bricks,
            peak_bytes: u.peak_bytes,
        }
    }
}

/// A reservation of pinned bricks and bytes; dropping it returns both.
#[derive(Debug)]
#[must_use = "dropping the reservation releases the budget"]
pub struct PinReservation {
    budget: Arc<PinBudget>,
    bricks: u64,
    bytes: u64,
}

impl PinReservation {
    /// Adds the measured `bytes` of the captured snapshot. Fails (adding nothing) if the byte cap
    /// would be exceeded.
    pub fn add_bytes(&mut self, bytes: u64) -> Result<(), LimitKind> {
        let mut u = lock(&self.budget.used);
        let total = u.bytes.checked_add(bytes).ok_or(LimitKind::PinnedBytes)?;
        if total > self.budget.caps.max_pinned_bytes {
            return Err(LimitKind::PinnedBytes);
        }
        u.bytes = total;
        u.peak_bytes = u.peak_bytes.max(total);
        self.bytes += bytes;
        Ok(())
    }

    /// Bricks held by this reservation.
    pub fn bricks(&self) -> u64 {
        self.bricks
    }
    /// Bytes held by this reservation.
    pub fn bytes(&self) -> u64 {
        self.bytes
    }
}

impl Drop for PinReservation {
    fn drop(&mut self) {
        let mut u = lock(&self.budget.used);
        u.bricks -= self.bricks;
        u.bytes -= self.bytes;
    }
}

#[cfg(all(test, not(loom)))]
mod tests {
    use super::*;

    fn budget(bricks: u64, bytes: u64) -> Arc<PinBudget> {
        PinBudget::new(Caps {
            max_pinned_bricks: bricks,
            max_pinned_bytes: bytes,
            task_ttl_ms: None,
        })
    }

    #[test]
    fn reservations_respect_the_brick_cap_and_release_on_drop() {
        let b = budget(10, 1000);
        let r1 = b.try_reserve(6).unwrap();
        assert!(b.try_reserve(5).is_none());
        let r2 = b.try_reserve(4).unwrap();
        assert!(b.try_reserve(1).is_none());
        assert_eq!(b.usage().bricks, 10);
        drop(r1);
        assert_eq!(b.usage().bricks, 4);
        assert!(b.try_reserve(6).is_some());
        drop(r2);
        assert_eq!(b.usage().bricks, 0);
        assert_eq!(b.usage().peak_bricks, 10);
    }

    #[test]
    fn bytes_are_capped_across_reservations_and_failed_adds_change_nothing() {
        let b = budget(100, 1000);
        let mut r1 = b.try_reserve(1).unwrap();
        let mut r2 = b.try_reserve(1).unwrap();
        r1.add_bytes(600).unwrap();
        assert_eq!(r2.add_bytes(401), Err(LimitKind::PinnedBytes));
        assert_eq!(r2.bytes(), 0);
        r2.add_bytes(400).unwrap();
        assert_eq!(b.usage().bytes, 1000);
        drop(r1);
        assert_eq!(b.usage().bytes, 400);
        drop(r2);
        let u = b.usage();
        assert_eq!((u.bricks, u.bytes, u.peak_bytes), (0, 0, 1000));
    }

    #[test]
    fn overflowing_requests_are_refused_not_wrapped() {
        let b = budget(u64::MAX, u64::MAX);
        let _r = b.try_reserve(u64::MAX).unwrap();
        assert!(b.try_reserve(1).is_none());
        let mut r = b.try_reserve(0).unwrap();
        r.add_bytes(u64::MAX).unwrap();
        assert_eq!(r.add_bytes(1), Err(LimitKind::PinnedBytes));
    }

    #[test]
    fn fits_ever_compares_with_the_total_cap() {
        let b = budget(27, 10);
        assert!(b.fits_ever(27) && !b.fits_ever(28));
        assert!(!b.fits_ever(u128::MAX));
    }

    #[test]
    fn synthetic_mobile_is_smaller_than_unlimited() {
        let m = Caps::synthetic_mobile();
        assert!(m.max_pinned_bricks < Caps::unlimited().max_pinned_bricks);
        assert!(m.task_ttl_ms.is_some());
        assert_eq!(Caps::default(), Caps::unlimited());
    }

    #[test]
    fn concurrent_reservations_never_exceed_the_cap() {
        let b = budget(8, u64::MAX);
        let hs: Vec<_> = (0..8)
            .map(|_| {
                let b = Arc::clone(&b);
                std::thread::spawn(move || {
                    for _ in 0..500 {
                        if let Some(r) = b.try_reserve(3) {
                            assert!(b.usage().bricks <= 8);
                            drop(r);
                        }
                    }
                })
            })
            .collect();
        for h in hs {
            h.join().unwrap();
        }
        let u = b.usage();
        assert_eq!(u.bricks, 0);
        assert!(u.peak_bricks <= 8);
    }
}
