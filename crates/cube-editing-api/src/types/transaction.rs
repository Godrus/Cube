//! E-04 EditingTransaction.

use cube_core::{AssetId, DirtyRegion, LayerId, OperationId, Region, RevisionToken};

/// Identifier of a transaction, unique per Orchestration instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TransactionId(pub u64);

/// Lifecycle of a transaction. Terminal states are final.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransactionState {
    /// Accepting operations.
    Open,
    /// The commit is running; it completes or fails as a unit and can no longer be cancelled.
    Committing,
    /// Committed.
    Committed,
    /// Rolled back; no revision changed.
    RolledBack,
    /// The commit failed; no revision changed.
    Failed,
    /// Cancelled before `Committing`; no revision changed.
    Cancelled,
}

impl TransactionState {
    /// `true` for the four final states.
    pub const fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Committed | Self::RolledBack | Self::Failed | Self::Cancelled
        )
    }

    /// `true` if `self -> to` is allowed:
    /// `Open -> Committing | RolledBack | Cancelled`, `Committing -> Committed | Failed`.
    pub const fn can_become(self, to: Self) -> bool {
        use TransactionState::*;
        matches!(
            (self, to),
            (Open, Committing | RolledBack | Cancelled) | (Committing, Committed | Failed)
        )
    }
}

/// What to do when the commit cannot be applied as planned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommitPolicy {
    /// All or nothing (default): any conflict rejects the whole transaction.
    Atomic,
}

/// What a failed operation leaves behind (Agent Spec section 9).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RollbackPolicy {
    /// State is unchanged.
    Unchanged,
}

/// Opaque reference a caller holds while a transaction is live.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransactionHandle {
    /// The transaction.
    pub id: TransactionId,
}

/// What the caller states when it begins a transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionSpec {
    /// Asset the transaction works on.
    pub asset: AssetId,
    /// Layers it may touch.
    pub layers: Vec<LayerId>,
    /// Regions it reads.
    pub read_regions: Vec<Region>,
    /// Regions it may write (upper bound).
    pub write_regions: Vec<Region>,
    /// Revisions the caller expects (optimistic concurrency); empty = take the current ones.
    pub expected: Vec<RevisionToken>,
    /// Commit policy.
    pub commit: CommitPolicy,
    /// Rollback policy.
    pub rollback: RollbackPolicy,
}

/// What observers learn about a finished transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionSummary {
    /// The transaction.
    pub transaction: TransactionId,
    /// Operations it contained, in execution order.
    pub operations: Vec<OperationId>,
    /// Asset it changed.
    pub asset: AssetId,
    /// Actor, as given by the authorization context.
    pub actor: u64,
    /// Authoritative tokens after the commit (empty for a rollback).
    pub new_tokens: Vec<RevisionToken>,
    /// Regions that actually changed.
    pub dirty: Vec<DirtyRegion>,
}

impl TransactionSpec {
    /// A spec for `asset` with the default policies and no regions yet.
    pub fn new(asset: AssetId) -> Self {
        Self {
            asset,
            layers: Vec::new(),
            read_regions: Vec::new(),
            write_regions: Vec::new(),
            expected: Vec::new(),
            commit: CommitPolicy::Atomic,
            rollback: RollbackPolicy::Unchanged,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use TransactionState::*;

    #[test]
    fn terminal_states_are_final_and_the_table_is_exact() {
        let all = [Open, Committing, Committed, RolledBack, Failed, Cancelled];
        let legal = [
            (Open, Committing),
            (Open, RolledBack),
            (Open, Cancelled),
            (Committing, Committed),
            (Committing, Failed),
        ];
        for a in all {
            for b in all {
                assert_eq!(a.can_become(b), legal.contains(&(a, b)), "{a:?} -> {b:?}");
            }
            if a.is_terminal() {
                assert!(all.iter().all(|b| !a.can_become(*b)));
            }
        }
    }

    #[test]
    fn a_committing_transaction_cannot_be_cancelled() {
        assert!(!Committing.can_become(Cancelled));
        assert!(!Committing.can_become(RolledBack));
    }
}
