//! Cube mesh backend (PS-5, MSH-01..03): authoritative polygon geometry in mesh pages behind
//! the Editing API port. No renderer, no Godot, no implicit conversion to or from voxels or
//! SDF.
//!
//! * [`page`] (MSH-02): the page format of this backend (Core stores the bytes opaquely), the
//!   validator with typed errors, and normal recalculation.
//! * [`params`], [`ops`] (MSH-01): `MeshBackend`: add a box or triangles, remove, set material,
//!   as write sets. A triangle belongs to the page of the brick that contains its centroid;
//!   triangles are **not clipped** at brick borders.
//! * [`collision`] (MSH-03): the triangle collision provider of a page.
//!
//! The backend never commits and holds no authoritative state.

pub mod collision;
pub mod ops;
pub mod page;
pub mod params;

pub use collision::{MESH_COLLISION_KIND, MeshCollision, MeshCollisionProvider};
pub use ops::{Limits, MeshBackend};
pub use page::{MAX_TRIANGLES, MeshError, Page, Tri, normals_q14};
pub use params::{MeshOp, add_box, add_triangles, remove, set_material};
