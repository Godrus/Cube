//! Synchronisation primitives, switched to `loom` under `--cfg loom` so the concurrency-critical
//! code can be model-checked (TEST-08). Shared data uses `std::sync::Arc` in both builds.

#[cfg(loom)]
pub(crate) use loom::sync::atomic::{AtomicBool, Ordering};
#[cfg(loom)]
pub(crate) use loom::sync::{Condvar, Mutex, MutexGuard};
#[cfg(not(loom))]
pub(crate) use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(not(loom))]
pub(crate) use std::sync::{Condvar, Mutex, MutexGuard};

use std::sync::PoisonError;

/// Locks a mutex, ignoring poisoning: no critical section of this crate leaves data half-updated,
/// and provider code never runs while a lock is held.
pub(crate) fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}
