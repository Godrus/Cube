//! Cube SDF backend (PS-5, SDF-01..03): a quantised signed distance field behind the Editing
//! API port. It is an alternative representation of geometry, never a required mechanic of
//! building, and there is **no implicit conversion** between voxel, SDF and mesh layers.
//!
//! * [`field`]: the field convention and quantisation; [`params`]: the operation parameters.
//! * [`ops`] (SDF-01): `SdfBackend`, add (union), remove (subtract) and material assignment as
//!   write sets. The backend never commits and holds no authoritative state.
//! * [`surface_nets`] (SDF-02): the Surface Nets mesh provider with an apron of one brick.
//! * [`gradient`], [`collision`] (SDF-03): normals from the field gradient and solid boxes.
//!
//! Convention: **negative inside**, positive outside; stored in the `Density` channel as the raw
//! bits of an `i16` with 256 steps per voxel (see [`field::SCALE`]); a `KnownEmpty` brick is
//! entirely outside; a brick that is not loaded is never guessed.

pub mod collision;
pub mod field;
pub mod gradient;
pub mod ops;
pub mod params;
pub mod surface_nets;

pub use collision::{SDF_COLLISION_KIND, SdfBox, SdfCollision, SdfCollisionProvider};
pub use field::{OUTSIDE, SCALE, Shape, quantise};
pub use gradient::{gradient_at, normal_q14};
pub use ops::{Limits, SdfBackend};
pub use params::{SdfOp, add, decode, remove, set_material};
pub use surface_nets::{SDF_MESH_KIND, SurfaceMesh, SurfaceNetsProvider, Vertex};
