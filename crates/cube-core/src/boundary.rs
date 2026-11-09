//! Mutation boundary (CORE-08, invariant MB-1, ACR-004 part A / DEC-058).
//!
//! Authoritative geometry changes only through one of three capability handles. A handle can only
//! be obtained from the [`AssetHost`], which is created together with the asset and consumed when
//! it hands out the handles, so an unrelated crate has no value of these types and no way to make
//! one. [`SharedAsset`] (the read side) has no mutating method at all. The handle decides the
//! [`ActorKind`] of the commit; a caller cannot choose it.

use crate::asset::GeometryAsset;
use crate::commit::{
    CellChange, CommitBatch, CommitError, CommitOutcome, MaterialiseOutcome, SharedAsset,
};
use crate::coord::BrickCoord;
use crate::ids::LayerId;
use crate::operation::{ActorKind, OperationId};
use crate::view::AssetView;

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

    /// Restores an asset from an exported view, keeping its revisions and epoch. The commit log
    /// starts empty: it records commits made through the handles of this host.
    pub fn from_view(view: &AssetView) -> Result<Self, crate::error::CoreError> {
        let (asset, revisions) = crate::view::state_parts(view)?;
        let state = crate::commit::AssetState::from_parts(
            asset,
            view.epoch,
            view.revision,
            revisions.into_iter().collect(),
        );
        Ok(Self {
            shared: SharedAsset::from_state(state),
        })
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

impl SystemCommit {
    /// Writes generated content into a cell that is not loaded; does nothing if it is already
    /// loaded (idempotent, never overwrites a user edit). Atomic with respect to other commits.
    pub fn materialise(
        &self,
        operation: OperationId,
        layer: LayerId,
        level: i8,
        brick: BrickCoord,
        change: CellChange,
    ) -> Result<MaterialiseOutcome, CommitError> {
        self.shared
            .materialise(operation, layer, level, brick, change)
    }

    /// Unloads a loaded cell without changing any revision; the caller guarantees the data is
    /// persisted. Returns `false` if the cell was not loaded.
    pub fn evict(&self, layer: LayerId, level: i8, brick: BrickCoord) -> Result<bool, CommitError> {
        self.shared.evict(layer, level, brick)
    }
}

impl RecoveryCommit {
    /// Replays persisted changes under their original operation ids (a commit with the
    /// `Recovery` actor).
    pub fn restore(
        &self,
        operation: OperationId,
        batch: &CommitBatch,
    ) -> Result<CommitOutcome, CommitError> {
        self.shared.commit(ActorKind::Recovery, operation, batch)
    }
}
