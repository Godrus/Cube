//! E-05 EditingResult.

use cube_core::{DirtyRegion, OperationId, RevisionToken};

use super::conflict::ConflictResult;
use crate::error::EditingError;
use crate::storage::DeltaId;

/// The data of a successful operation. Contains no memory references.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Success {
    /// The operation.
    pub operation: OperationId,
    /// Authoritative tokens after the change.
    pub new_tokens: Vec<RevisionToken>,
    /// Regions that actually changed.
    pub dirty: Vec<DirtyRegion>,
    /// Ids the operation produced (new layers, assets, ...), as raw values.
    pub produced_ids: Vec<u64>,
    /// The delta recorded for the change, if one was recorded.
    pub delta: Option<DeltaId>,
}

/// Outcome of an operation or a commit. There is no partial result: an operation whose contract
/// declares partial results would need its own variant (none does).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum EditingResult {
    /// The change happened.
    Success(Success),
    /// The operation is malformed or invalid for the target; nothing changed.
    ValidationFailure(EditingError),
    /// Revisions or locks conflict; nothing changed.
    Conflict(ConflictResult),
    /// The target does not support the operation kind; nothing changed.
    Unsupported(EditingError),
    /// A conversion is needed and not permitted; nothing changed.
    ConversionRequired(EditingError),
    /// A resource limit or a Processing failure stopped it; nothing changed.
    ResourceFailure(EditingError),
    /// Cancelled before the commit started; nothing changed.
    Cancelled,
    /// The change WAS applied but its durable journal confirmation failed. Not a success: the
    /// caller must retry with the same operation id, which completes the confirmation without
    /// applying anything twice.
    AppliedUnconfirmed(Success),
}

impl EditingResult {
    /// `true` for `Success`.
    pub const fn is_success(&self) -> bool {
        matches!(self, Self::Success(_))
    }

    /// `true` if the authoritative state is guaranteed unchanged (not `Success` and not
    /// `AppliedUnconfirmed`).
    pub const fn left_state_unchanged(&self) -> bool {
        !matches!(self, Self::Success(_) | Self::AppliedUnconfirmed(_))
    }
}

/// Progress of an operation as `GetStatus` reports it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperationStatus {
    /// Accepted, not yet executed.
    Pending,
    /// Executing.
    Running,
    /// Finished with `Success`.
    Succeeded,
    /// Finished without changing state.
    Failed,
    /// Cancelled.
    Cancelled,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_success_changes_state() {
        let ok = EditingResult::Success(Success {
            operation: OperationId::new(1, 1),
            new_tokens: vec![],
            dirty: vec![],
            produced_ids: vec![],
            delta: None,
        });
        assert!(ok.is_success() && !ok.left_state_unchanged());
        for r in [
            EditingResult::ValidationFailure(EditingError::InvalidResolution),
            EditingResult::Unsupported(EditingError::UnknownKind(super::super::KindId(1))),
            EditingResult::ConversionRequired(EditingError::ConversionRequired),
            EditingResult::ResourceFailure(EditingError::ProcessingFailed),
            EditingResult::Cancelled,
        ] {
            assert!(r.left_state_unchanged() && !r.is_success());
        }
    }
}
