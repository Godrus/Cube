//! Cube Geometry Core: the passive data model of the world (PS-1).
//!
//! This crate has no dependencies, knows nothing about any engine or game content, and contains
//! no game rules. Implemented so far: coordinates and keys (CORE-02), the layer model (CORE-01) and the cell model
//! with sparse brick storage (CORE-03).

pub mod asset;
pub mod cell;
pub mod coord;
pub mod error;
pub mod ids;
pub mod key;
pub mod layer;
pub mod material;
pub mod region;
pub mod revision;
pub mod transform;

pub use asset::GeometryAsset;
pub use cell::{
    BrickBounds, CellPayload, CellState, CellStore, Channel, ChannelId, ChannelSet,
    LoadFailureReason, MeshPage,
};
pub use coord::{BRICK_EDGE, BRICK_VOLUME, BrickCoord, LocalPos, VoxelPos};
pub use error::CoreError;
pub use ids::{AssetId, LayerId};
pub use key::CellKey;
pub use layer::{Layer, LayerData, LayerKind, PageStore};
pub use material::MaterialIndex;
pub use region::Region;
pub use revision::{
    Comparison, DerivedSlotId, Epoch, RegionStamp, ResourceId, Revision, RevisionToken,
    StampAlgorithm, StampCompare,
};
pub use transform::{GeometryTransform, Rotation};
