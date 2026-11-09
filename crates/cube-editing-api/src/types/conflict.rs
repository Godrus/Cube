//! E-06 ConflictResult.

use cube_core::{OperationId, ResourceId, RevisionToken};

/// What the conflict is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConflictScope {
    /// One or more bricks.
    Brick,
    /// A layer.
    Layer,
    /// A whole asset.
    Asset,
    /// A lock held by another operation.
    Lock,
}

/// What the caller may try next. Informative only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Suggestion {
    /// Try again with fresh tokens.
    Retry,
    /// Rebase the operation on the current state.
    Rebase,
    /// Give up.
    Abort,
}

/// Report of a conflict. The default policy is "reject and report" (OD-R3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConflictResult {
    /// Scope of the conflict.
    pub scope: ConflictScope,
    /// Exactly the resources that conflict, in a stable order.
    pub conflicting: Vec<ResourceId>,
    /// What the caller expected.
    pub expected: Vec<RevisionToken>,
    /// What is current.
    pub current: Vec<RevisionToken>,
    /// The operation holding a lock, for `Lock` conflicts.
    pub holder: Option<OperationId>,
    /// Suggested next step.
    pub suggestion: Suggestion,
}

impl ConflictResult {
    /// `true` if `expected` and `current` describe the same resources (a well-formed report).
    pub fn tokens_pair_up(&self) -> bool {
        self.expected.len() == self.current.len()
            && self
                .expected
                .iter()
                .zip(&self.current)
                .all(|(e, c)| e.resource() == c.resource())
    }
}
