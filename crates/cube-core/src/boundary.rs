//! Mutation boundary (CORE-08, invariant MB-1, ACR-004 part A / DEC-058).
//!
//! Authoritative geometry changes only through one of three capability handles. A handle can only
//! be obtained from the [`AssetHost`], which is created together with the asset and consumed when
//! it hands out the handles, so an unrelated crate has no value of these types and no way to make
//! one. [`SharedAsset`] (the read side) has no mutating method at all. The handle decides the
//! [`ActorKind`] of the commit; a caller cannot choose it.

use crate::asset::GeometryAsset;
use crate::commit::{CommitBatch, CommitError, CommitOutcome, SharedAsset};
use crate::operation::{ActorKind, OperationId};

/// Owner of a new asset; the only source of commit handles.
#[derive(Debug)]
pub struct AssetHost {
    shared: SharedAsset,
}

/// The three commit handles of one asset, in the hands of the composition root.
#[derive(Debug)]
pub struct Handles {
    /// For the Editing orchestration only.
    pub user: UserCommit,
    /// For the WorldGen materialiser only.
    pub system: SystemCommit,
    /// For the persistence recovery loader only.
    pub recovery: RecoveryCommit,
}

impl AssetHost {
    /// Creates the live asset from its initial structure.
    pub fn new(asset: GeometryAsset) -> Self {
        Self {
            shared: SharedAsset::new(asset),
        }
    }

    /// A read-only handle (clonable, freely shareable).
    pub fn reader(&self) -> SharedAsset {
        self.shared.clone()
    }

    /// Consumes the host and returns the commit handles.
    pub fn into_handles(self) -> Handles {
        Handles {
            user: UserCommit {
                shared: self.shared.clone(),
            },
            system: SystemCommit {
                shared: self.shared.clone(),
            },
            recovery: RecoveryCommit {
                shared: self.shared,
            },
        }
    }
}

macro_rules! handle {
    ($(#[$m:meta])* $name:ident, $actor:expr) => {
        $(#[$m])*
        #[derive(Debug)]
        pub struct $name {
            shared: SharedAsset,
        }

        impl $name {
            /// Commits `batch` atomically on behalf of this handle's actor kind.
            pub fn commit(&self, operation: OperationId, batch: &CommitBatch) -> Result<CommitOutcome, CommitError> {
                self.shared.commit($actor, operation, batch)
            }

            /// Read access to the same asset.
            pub fn reader(&self) -> SharedAsset {
                self.shared.clone()
            }
        }
    };
}

handle!(
    /// Commits user-originated changes (`ActorKind::User`).
    UserCommit,
    ActorKind::User
);
handle!(
    /// Commits system generation and eviction (`ActorKind::System`).
    SystemCommit,
    ActorKind::System
);
handle!(
    /// Commits replay of persisted history (`ActorKind::Recovery`).
    RecoveryCommit,
    ActorKind::Recovery
);
