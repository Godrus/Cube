//! The single user commit path (ORCH-05).
//!
//! This is the only file of the crate (and, by `cargo xtask archcheck`, of the whole workspace
//! outside the composition root and tests) that holds the user commit handle. Every authoritative
//! change a user request causes goes through [`CommitPath::commit`], which can only produce a
//! `User` commit: Core decides the actor kind by the handle, not by the caller.

use cube_core::{CommitBatch, CommitError, CommitOutcome, OperationId, SharedAsset, UserCommit};

/// Owner of the user commit handle of one asset.
#[derive(Debug)]
pub struct CommitPath {
    handle: UserCommit,
}

impl CommitPath {
    /// Takes the handle. It is handed over once by the composition root.
    pub fn new(handle: UserCommit) -> Self {
        Self { handle }
    }

    /// Read access to the same asset.
    pub fn reader(&self) -> SharedAsset {
        self.handle.reader()
    }

    /// Commits `batch` atomically as a user change. Crate-private: nothing outside this crate can
    /// reach it, and inside the crate only the transaction commit calls it.
    pub(crate) fn commit(
        &self,
        operation: OperationId,
        batch: &CommitBatch,
    ) -> Result<CommitOutcome, CommitError> {
        self.handle.commit(operation, batch)
    }
}
