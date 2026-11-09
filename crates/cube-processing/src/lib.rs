//! Cube Geometry Processing foundation (PS-2).
//!
//! Layer 1, directly above Core: it runs derived-data computations (meshing, collision, ...)
//! that other modules register as providers, on immutable snapshots, and publishes their results
//! only while their inputs are still current. It contains no editing or game logic, never writes
//! to Core, and has no async runtime.
//!
//! * [`snapshot`]: immutable brick neighbourhood with apron and its input stamp (PROC-01).
//! * [`provider`], [`graph`]: provider interface, registry and dependency graph (PROC-02).
//! * [`task`], [`executor`]: task state machine, priorities and the executor (PROC-03).
//! * [`cancel`]: cancellation tokens and deadlines (PROC-04).
//! * [`slot`]: derived-data slots with stale-result rejection (PROC-05).
//! * [`caps`], [`bounds`]: resource caps and the brick-bounds slot (PROC-06).
//!
//! Contracts PR-01..PR-08 are Draft; this crate implements the provisional subset TC-01
//! (see `docs/production/processing-foundation-contract.md`).

pub mod bounds;
pub mod cancel;
pub mod caps;
pub mod error;
pub mod executor;
pub mod graph;
pub mod provider;
pub mod slot;
pub mod snapshot;
mod sync;
pub mod task;
#[cfg(all(test, not(loom)))]
mod testing;

pub use bounds::{BOUNDS_KIND, BoundsProvider, union_all};
pub use cancel::{CancelReason, CancellationToken, Clock, ManualClock, Registration};
pub use caps::{BRICK_OVERHEAD_BYTES, Caps, PinBudget, PinReservation, PinUsage};
pub use error::{ERROR_MODULE, LimitKind, ProcessingError};
#[cfg(not(loom))]
pub use executor::Workers;
pub use executor::{Executor, ExecutorConfig, Stats, TargetStatus};
pub use graph::{DependencyGraph, ProviderRegistry, RegistryError};
pub use provider::{
    ChangeMask, ComputeContext, DerivedKind, DerivedTarget, MAX_APRON, Payload, Provider,
    ProviderFailure,
};
pub use slot::{DerivedSlot, PublishOutcome, Published, SlotStatus};
pub use snapshot::{
    DependencyRevision, HARD_MAX_SNAPSHOT_BRICKS, Snapshot, SnapshotRequest, stamp,
};
pub use task::{Priority, PriorityClass, TaskHandle, TaskId, TaskOutcome, TaskState};
