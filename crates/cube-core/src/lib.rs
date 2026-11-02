//! Cube Geometry Core (experimental spike, draft contracts C-01..C-19).
//!
//! Standard library only. No engine, UI, network, persistence or processing
//! dependency. Single voxel layer type, one brick storage implementation.
//!
//! Not modelled in this spike: SDF/Mesh layers, transforms, bounds beyond a
//! region type, channels beyond one `u16` material channel, refinement,
//! junctions, composite data, `NotLoaded`/`LoadFailure`/`OutsideBounds`
//! brick states (the enum exists, storage never produces them).

pub mod asset;
pub mod brick;
pub mod coord;
pub mod dirty;
pub mod error;
pub mod ids;
pub mod region;
pub mod revision;

pub use asset::{
    AssetExport, BrickPins, BrickWrite, CommitOutcome, ExpectedBrick, GeometryAsset, LayerExport,
    VoxelLayer, WriteSet,
};
pub use brick::{Brick, BrickData, BrickLookup, BrickStorage};
pub use coord::{BrickCoord, LocalPos, VoxelPos, BRICK_SIZE, BRICK_VOLUME};
pub use dirty::{ChangeType, DirtyRegion};
pub use error::CoreError;
pub use ids::{AssetId, LayerId, OperationId};
pub use region::Region;
pub use revision::{BrickKey, ResourceId, RevisionOrder, RevisionToken};
