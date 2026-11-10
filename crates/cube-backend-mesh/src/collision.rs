//! Collision provider of the mesh backend (MSH-03; provisional like every provider of PS-5).
//!
//! The collision triangles of one page, in absolute 1/256 voxel coordinates (`i128`, no limit
//! below the grid end), with their bounding box, in the canonical triangle order, so the result
//! is deterministic and golden-testable. Budget: at most [`crate::MAX_TRIANGLES`] triangles per
//! page (the page limit itself). A page that is not loaded, failed to load, is not a valid page
//! or belongs to a voxel layer is a failure, never an empty result.

use std::sync::Arc;

use cube_core::{CellPayload, CellState};
use cube_processing::{
    ChangeMask, ComputeContext, DerivedKind, Payload, Provider, ProviderFailure,
};

use crate::page::Page;

/// Derived kind of the mesh collision triangles. The mesh backend owns ids
/// `0xB300_0000..0xB301_0000`.
pub const MESH_COLLISION_KIND: DerivedKind = DerivedKind(0xB300_0001);

/// Collision data of one page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeshCollision {
    /// The brick.
    pub key: cube_core::CellKey,
    /// Triangles, absolute 1/256 voxel.
    pub triangles: Vec<[[i128; 3]; 3]>,
    /// Smallest and largest coordinate over all vertices; `None` for an empty page.
    pub bounds: Option<([i128; 3], [i128; 3])>,
}

/// The provider.
#[derive(Debug, Default, Clone, Copy)]
pub struct MeshCollisionProvider;

impl Provider for MeshCollisionProvider {
    fn kind(&self) -> DerivedKind {
        MESH_COLLISION_KIND
    }
    fn name(&self) -> &str {
        "mesh-collision"
    }
    fn version(&self) -> u32 {
        1
    }
    fn consumes(&self) -> ChangeMask {
        ChangeMask::GEOMETRY
    }
    fn compute(&self, ctx: &ComputeContext<'_>) -> Result<Payload, ProviderFailure> {
        let key = ctx.target.key;
        let state = ctx
            .snapshot
            .cell(key.brick())
            .map_err(|e| ProviderFailure::Failed(e.to_string()))?;
        let fail = |m: &str| ProviderFailure::Failed(m.to_owned());
        let page = match state {
            CellState::NotLoaded => return Err(fail("page is not loaded")),
            CellState::LoadFailure(_) => return Err(fail("page failed to load")),
            CellState::OutsideBounds | CellState::KnownEmpty => Page::default(),
            CellState::Present(p) => match &**p {
                CellPayload::MeshPage(m) => {
                    Page::decode(m.bytes()).map_err(|e| ProviderFailure::Failed(e.to_string()))?
                }
                CellPayload::Channels(_) => return Err(fail("brick is not a mesh page")),
            },
        };
        ctx.token.check()?;
        let o = key
            .brick()
            .origin()
            .map_err(|e| ProviderFailure::Failed(e.to_string()))?;
        let o = [
            i128::from(o.x) * 256,
            i128::from(o.y) * 256,
            i128::from(o.z) * 256,
        ];
        let mut bounds: Option<([i128; 3], [i128; 3])> = None;
        let triangles: Vec<[[i128; 3]; 3]> = page
            .tris
            .iter()
            .map(|t| {
                t.v.map(|v| {
                    let p = [
                        o[0] + i128::from(v[0]),
                        o[1] + i128::from(v[1]),
                        o[2] + i128::from(v[2]),
                    ];
                    let b = bounds.get_or_insert((p, p));
                    for (i, c) in p.iter().enumerate() {
                        b.0[i] = b.0[i].min(*c);
                        b.1[i] = b.1[i].max(*c);
                    }
                    p
                })
            })
            .collect();
        Ok(Arc::new(MeshCollision {
            key,
            triangles,
            bounds,
        }))
    }
}
