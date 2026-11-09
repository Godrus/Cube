//! Cube Geometry Core: the passive data model of the world (PS-1).
//!
//! This crate has no dependencies, knows nothing about any engine or game content, and contains
//! no game rules. Implemented so far: coordinates and keys (CORE-02) and the layer model (CORE-01).

pub mod asset;
pub mod coord;
pub mod error;
pub mod ids;
pub mod key;
pub mod layer;
pub mod material;
pub mod region;
pub mod transform;

pub use asset::GeometryAsset;
pub use coord::{BRICK_EDGE, BRICK_VOLUME, BrickCoord, LocalPos, VoxelPos};
pub use error::CoreError;
pub use ids::{AssetId, LayerId};
pub use key::CellKey;
pub use layer::{CellStore, Layer, LayerData, LayerKind, PageStore};
pub use material::MaterialIndex;
pub use region::Region;
pub use transform::{GeometryTransform, Rotation};
