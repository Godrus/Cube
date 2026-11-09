//! E-12 TransactionObserver.

use crate::storage::GeometryDelta;
use crate::types::TransactionSummary;

/// Post-commit sink, implemented by History (and optionally networking). Observers cannot veto or
/// mutate: both methods return nothing, so a failing observer is reported and logged by the
/// caller and never undoes a commit. Delivery is ordered per asset, after the commit gate is
/// released; whether delivery is durable before the acknowledgement is an open decision.
pub trait TransactionObserver: Send + Sync {
    /// A transaction committed with this delta.
    fn on_committed(&self, summary: &TransactionSummary, delta: &GeometryDelta);
    /// A transaction was rolled back, failed or was cancelled; nothing changed.
    fn on_rolled_back(&self, summary: &TransactionSummary);
}
