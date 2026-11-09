//! Derived-data slot with stale-result rejection (PROC-05, PR-04).
//!
//! **Invariant (stale never published):** a value becomes visible in a slot only if, at the
//! instant of the swap and under the slot's lock, its input stamp equals the slot's *expected*
//! stamp, which is the newest stamp of the live state the slot has been told about. The expected
//! stamp only moves forward (by `(epoch, asset revision)`), so once `expect` has returned for a
//! newer state, no result computed from older inputs can be published afterwards, whatever the
//! interleaving. Results whose inputs are unchanged by a newer state stay valid.
//!
//! The check and the swap are one critical section; there is no window in which a stale value is
//! stored and "fixed up" later. Readers that need only current data use [`DerivedSlot::current`];
//! [`DerivedSlot::last_known`] exposes an older value explicitly marked as stale.

use std::sync::Arc;

use crate::snapshot::DependencyRevision;
use crate::sync::{Mutex, lock};

/// Result of an attempt to publish.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublishOutcome {
    /// The value is now visible; `sequence` counts publications of this slot from 1.
    Published {
        /// Publication number.
        sequence: u64,
    },
    /// Rejected: the inputs of the value are not the current inputs. Nothing was stored.
    Stale,
}

/// State of a slot as seen by a query.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotStatus {
    /// Nothing has ever been published.
    Missing,
    /// A value is published and its inputs are the current inputs.
    Valid,
    /// A value is published but newer inputs exist.
    Stale,
}

/// A published value with its stamp.
#[derive(Debug)]
pub struct Published<T: ?Sized> {
    /// The value.
    pub value: Arc<T>,
    /// The inputs the value was computed from.
    pub dependency: DependencyRevision,
    /// Publication number of this slot.
    pub sequence: u64,
}

impl<T: ?Sized> Clone for Published<T> {
    fn clone(&self) -> Self {
        Self {
            value: Arc::clone(&self.value),
            dependency: self.dependency.clone(),
            sequence: self.sequence,
        }
    }
}

struct Inner<T: ?Sized> {
    expected: Option<DependencyRevision>,
    published: Option<Published<T>>,
    sequence: u64,
}

/// One derived value (for example the mesh of one brick) and its validity.
pub struct DerivedSlot<T: ?Sized> {
    inner: Mutex<Inner<T>>,
}

impl<T: ?Sized> Default for DerivedSlot<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ?Sized> std::fmt::Debug for DerivedSlot<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DerivedSlot")
            .field("status", &self.status())
            .finish()
    }
}

fn raise<T: ?Sized>(g: &mut Inner<T>, live: &DependencyRevision) -> bool {
    let newer = match &g.expected {
        None => true,
        Some(e) => live.order_key() > e.order_key(),
    };
    if newer {
        g.expected = Some(live.clone());
    }
    newer
}

fn is_current<T: ?Sized>(g: &Inner<T>) -> Option<bool> {
    let p = g.published.as_ref()?;
    Some(match &g.expected {
        None => true,
        Some(e) => p.dependency.same_inputs(e),
    })
}

impl<T: ?Sized> DerivedSlot<T> {
    /// An empty slot.
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(Inner {
                expected: None,
                published: None,
                sequence: 0,
            }),
        }
    }

    /// Tells the slot what the live inputs are now. Only a strictly newer stamp (by
    /// `(epoch, asset revision)`) changes the expectation; an older or equal one is ignored, so
    /// the expectation never moves backwards. Returns `true` if the expectation changed.
    pub fn expect(&self, live: &DependencyRevision) -> bool {
        raise(&mut lock(&self.inner), live)
    }

    /// Publishes `value`, computed from `result`'s inputs, if those are the current inputs.
    ///
    /// `live` must be read from the live state *after* the computation finished; it is applied as
    /// an `expect` in the same critical section, so a missed invalidation cannot let a stale
    /// value through. Returns [`PublishOutcome::Stale`] and stores nothing otherwise.
    pub fn publish(
        &self,
        result: &DependencyRevision,
        live: &DependencyRevision,
        value: Arc<T>,
    ) -> PublishOutcome {
        let mut g = lock(&self.inner);
        raise(&mut g, live);
        let current = g.expected.as_ref().is_some_and(|e| result.same_inputs(e));
        if !current {
            return PublishOutcome::Stale;
        }
        g.sequence += 1;
        let sequence = g.sequence;
        g.published = Some(Published {
            value,
            dependency: result.clone(),
            sequence,
        });
        PublishOutcome::Published { sequence }
    }

    /// Validity of the slot.
    pub fn status(&self) -> SlotStatus {
        match is_current(&lock(&self.inner)) {
            None => SlotStatus::Missing,
            Some(true) => SlotStatus::Valid,
            Some(false) => SlotStatus::Stale,
        }
    }

    /// The value, only if it is current.
    pub fn current(&self) -> Option<Published<T>> {
        let g = lock(&self.inner);
        if is_current(&g) == Some(true) {
            g.published.clone()
        } else {
            None
        }
    }

    /// The last published value and whether it is still current (for display while a newer value
    /// is being computed). Value and flag are read atomically.
    pub fn last_known(&self) -> Option<(Published<T>, bool)> {
        let g = lock(&self.inner);
        let cur = is_current(&g)?;
        g.published.clone().map(|p| (p, cur))
    }

    /// The newest stamp the slot expects, if it has been told one.
    pub fn expected(&self) -> Option<DependencyRevision> {
        lock(&self.inner).expected.clone()
    }
}

#[cfg(all(test, not(loom)))]
mod tests {
    use super::*;
    use crate::testing::dep;

    #[test]
    fn publishes_when_inputs_match_and_rejects_when_they_do_not() {
        let s: DerivedSlot<u32> = DerivedSlot::new();
        assert_eq!(s.status(), SlotStatus::Missing);
        let d1 = dep(1, Some(1));
        assert_eq!(
            s.publish(&d1, &d1, Arc::new(10)),
            PublishOutcome::Published { sequence: 1 }
        );
        assert_eq!(s.status(), SlotStatus::Valid);
        assert_eq!(*s.current().unwrap().value, 10);

        let d2 = dep(2, Some(2));
        assert!(s.expect(&d2));
        assert_eq!(s.status(), SlotStatus::Stale);
        assert!(s.current().is_none());
        let (old, cur) = s.last_known().unwrap();
        assert_eq!((*old.value, cur), (10, false));
        // A late result for the old inputs is rejected and changes nothing.
        assert_eq!(s.publish(&d1, &d2, Arc::new(11)), PublishOutcome::Stale);
        assert_eq!(*s.last_known().unwrap().0.value, 10);
        // The new inputs are accepted.
        assert_eq!(
            s.publish(&d2, &d2, Arc::new(12)),
            PublishOutcome::Published { sequence: 2 }
        );
        assert_eq!(s.status(), SlotStatus::Valid);
    }

    #[test]
    fn expectation_never_moves_backwards() {
        let s: DerivedSlot<u32> = DerivedSlot::new();
        let (d1, d2, d3) = (dep(1, Some(1)), dep(3, Some(3)), dep(2, Some(2)));
        assert!(s.expect(&d2));
        assert!(!s.expect(&d1));
        assert!(!s.expect(&d3));
        assert!(!s.expect(&d2));
        assert_eq!(s.expected().unwrap().order(), d2.order());
    }

    #[test]
    fn a_newer_state_with_unchanged_inputs_keeps_the_value_valid() {
        let s: DerivedSlot<u32> = DerivedSlot::new();
        let d1 = dep(1, Some(1));
        s.publish(&d1, &d1, Arc::new(1));
        let d2 = dep(2, Some(1)); // the asset moved on, this brick did not change
        assert!(s.expect(&d2));
        assert_eq!(s.status(), SlotStatus::Valid);
        assert!(s.current().is_some());
    }

    #[test]
    fn publish_applies_the_live_stamp_so_a_missed_invalidation_cannot_leak() {
        let s: DerivedSlot<u32> = DerivedSlot::new();
        let (d1, d2) = (dep(1, Some(1)), dep(2, Some(2)));
        // Nobody called expect(d2); the live stamp passed to publish catches it.
        assert_eq!(s.publish(&d1, &d2, Arc::new(1)), PublishOutcome::Stale);
        assert_eq!(s.status(), SlotStatus::Missing);
    }

    #[test]
    fn a_result_that_is_newer_than_the_expectation_is_rejected_not_adopted() {
        let s: DerivedSlot<u32> = DerivedSlot::new();
        let (d1, d2) = (dep(1, Some(1)), dep(2, Some(2)));
        s.expect(&d1);
        // `live` is not newer than the expectation, the result claims newer inputs: not accepted
        // because publication never raises the expectation to the result's own stamp.
        assert_eq!(s.publish(&d2, &d1, Arc::new(1)), PublishOutcome::Stale);
    }

    #[test]
    fn unsized_payloads_are_supported() {
        let s: DerivedSlot<dyn std::any::Any + Send + Sync> = DerivedSlot::new();
        let d = dep(1, Some(1));
        let v: Arc<dyn std::any::Any + Send + Sync> = Arc::new(7u8);
        assert!(matches!(
            s.publish(&d, &d, v),
            PublishOutcome::Published { .. }
        ));
        let got = s.current().unwrap().value;
        assert_eq!(got.downcast_ref::<u8>(), Some(&7));
    }
}
