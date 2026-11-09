//! Cube voxel backend (PS-5): real voxel editing behind the Editing API port.
//!
//! * [`ops`] (VOX-01): `VoxelBackend`, set/replace/fill/clear/copy/move of regions as write sets.
//! * [`params`]: the parameter schema of the operation kinds this backend owns.
//! * [`edit`]: the brick editor and its safety rules (no silent data loss).
//!
//! The backend never commits and holds no authoritative state; Orchestration owns the commit.

pub mod edit;
pub mod ops;
pub mod params;

pub use edit::Limits;
pub use ops::VoxelBackend;
pub use params::{TransformMode, VoxelOp, add, decode, remove, set_material, transform};
