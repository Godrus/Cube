//! E-11 EditingService (the facade).

use cube_core::OperationId;

use crate::auth::AuthorizationContext;
use crate::error::EditingError;
use crate::types::{
    EditingOperation, EditingResult, OperationStatus, TransactionHandle, TransactionSpec,
};

/// The public editing facade, implemented by Orchestration. The low-level API cannot bypass dirty
/// tracking, revision management or the dependency graph: every authoritative change goes
/// through a transaction.
///
/// Authorization is checked before revision validation, and a denial leaves no state. Async
/// variants (a job handle per Processing task) are an open decision and not part of this
/// minimal contract.
pub trait EditingService: Send + Sync {
    /// Starts a transaction under `context`.
    fn begin(
        &self,
        context: AuthorizationContext,
        spec: TransactionSpec,
    ) -> Result<TransactionHandle, EditingError>;

    /// Executes one operation inside the transaction. Nothing authoritative changes before
    /// `commit`.
    fn execute(&self, handle: &TransactionHandle, operation: EditingOperation) -> EditingResult;

    /// Commits atomically; the result carries the new tokens and the dirty regions.
    fn commit(&self, handle: &TransactionHandle) -> EditingResult;

    /// Discards the transaction; no revision changes.
    fn rollback(&self, handle: &TransactionHandle) -> Result<(), EditingError>;

    /// Cancels an operation that has not started committing.
    fn cancel(&self, operation: OperationId) -> Result<(), EditingError>;

    /// Progress of an operation.
    fn status(&self, operation: OperationId) -> Result<OperationStatus, EditingError>;
}
