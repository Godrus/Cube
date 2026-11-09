//! Dirty regions: a record of data that actually changed (contract C-18).

use crate::ids::{AssetId, LayerId};
use crate::operation::{ActorKind, OperationId};
use crate::region::Region;
use crate::revision::RevisionToken;

/// Kind of change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChangeType {
    /// Occupancy or shape changed.
    Geometry,
    /// Only the material channel changed.
    Material,
    /// Only auxiliary attribute channels changed.
    Attribute,
}

/// One changed brick: never produced for a write that did not change the data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirtyRegion {
    /// Asset of the change.
    pub asset: AssetId,
    /// Layer of the change.
    pub layer: LayerId,
    /// Voxel box of the brick, at the brick's level.
    pub region: Region,
    /// Classification of the change.
    pub change: ChangeType,
    /// Asset token produced by the commit that made the change.
    pub source: RevisionToken,
    /// Operation that caused it.
    pub operation: OperationId,
    /// Origin of the change.
    pub actor: ActorKind,
}
