//! Cube voxel backend (PS-5): real voxel editing behind the Editing API port.
//!
//! * [`ops`] (VOX-01): `VoxelBackend`, set/replace/fill/clear/copy/move of regions as write sets.
//! * [`params`]: the parameter schema of the operation kinds this backend owns.
//! * [`faces`] (VOX-02): the faces provider, the reference oracle of meshing.
//! * [`greedy`] (VOX-03): material-aware greedy merging with a seam rule.
//! * [`sections`] (VOX-04): which brick meshes a commit makes dirty.
//! * [`edit`]: the brick editor and its safety rules (no silent data loss).
//!
//! The backend never commits and holds no authoritative state; Orchestration owns the commit.

pub mod edit;
pub mod faces;
pub mod greedy;
pub mod ops;
pub mod params;
pub mod sections;

pub use edit::Limits;
pub use faces::{Dir, FACES_KIND, Face, FacesMesh, FacesProvider};
pub use greedy::{GREEDY_KIND, GreedyMesh, GreedyProvider, Quad, SeamRule, expand, merge};
pub use ops::VoxelBackend;
pub use params::{TransformMode, VoxelOp, add, decode, remove, set_material, transform};
pub use sections::rebuild_set;
