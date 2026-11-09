//! Cube Geometry Core: the passive data model of the world (PS-1).
//!
//! This crate has no dependencies, knows nothing about any engine or game content, and contains
//! no game rules. Implemented so far: coordinates and keys (CORE-02), the layer model (CORE-01) the cell model
//! with sparse brick storage (CORE-03), revisions (CORE-05), operation ids (CORE-07), the commit
//! gate (CORE-06) and the mutation boundary (CORE-08).

pub mod asset;
pub mod boundary;
pub mod cell;
pub mod commit;
pub mod coord;
pub mod dirty;
pub mod error;
pub mod ids;
pub mod key;
pub mod layer;
pub mod material;
pub mod operation;
pub mod region;
pub mod revision;
mod rotation;
pub mod transform;

pub use asset::GeometryAsset;
pub use boundary::{AssetHost, Handles, RecoveryCommit, SystemCommit, UserCommit};
pub use cell::{
    BrickBounds, CellPayload, CellState, CellStore, Channel, ChannelId, ChannelSet,
    LoadFailureReason, MeshPage,
};
pub use commit::{
    AssetState, CellChange, CellWrite, CommitBatch, CommitError, CommitLogEntry, CommitOutcome,
    Conflict, ExpectedRevision, MaterialiseOutcome, SharedAsset, WriteSet,
};
pub use coord::{BRICK_EDGE, BRICK_VOLUME, BrickCoord, LocalPos, VoxelPos};
pub use dirty::{ChangeType, DirtyRegion};
pub use error::CoreError;
pub use ids::{AssetId, LayerId};
pub use key::CellKey;
pub use layer::{Layer, LayerData, LayerKind, PageStore};
pub use material::MaterialIndex;
pub use operation::{ActorKind, OperationId};
pub use region::Region;
pub use revision::{
    Comparison, DerivedSlotId, Epoch, RegionStamp, ResourceId, Revision, RevisionToken,
    StampAlgorithm, StampCompare,
};
pub use transform::{GeometryTransform, Rotation};
