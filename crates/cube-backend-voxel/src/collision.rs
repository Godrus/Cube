//! Collision provider for voxels (VOX-05; owner: Voxel Agent, DEC-089).
//!
//! For one brick it produces disjoint axis-aligned boxes whose union is exactly the set of solid
//! voxels (material != 0) of that brick, found by a deterministic scan (x, then y, then z
//! extension). Boxes never cross a brick border; a consumer that needs a larger body merges
//! across bricks itself. Collision depends only on the brick's own voxels, so the provider needs
//! no apron and a change invalidates only that brick.
//!
//! Unknown is never empty: a brick that is not loaded (or failed to load) is a **failure**, so
//! a consumer cannot treat it as free space; a loaded empty brick has no boxes.
//!
//! Budget (documented): at most 4096 boxes per brick (checkerboard worst case); typical built
//! shapes produce a handful.

use std::sync::Arc;

use cube_core::{BRICK_EDGE, CellKey, CellPayload, CellState, ChannelId, LocalPos, VoxelPos};
use cube_processing::{
    ChangeMask, ComputeContext, DerivedKind, Payload, Provider, ProviderFailure,
};

/// Derived kind of the collision boxes.
pub const COLLISION_KIND: DerivedKind = DerivedKind(0xB100_0003);

/// A solid box `[min, max)` in voxel coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SolidBox {
    /// Inclusive minimum corner.
    pub min: VoxelPos,
    /// Exclusive maximum corner.
    pub max: VoxelPos,
}

/// The collision data of one brick.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollisionBoxes {
    /// The brick.
    pub key: CellKey,
    /// Disjoint boxes in ascending order.
    pub boxes: Vec<SolidBox>,
}

/// Merges the solid cells of a 16^3 grid (`solid[x + 16*(y + 16*z)]`) into boxes at `origin`.
pub fn merge_boxes(solid: &[bool], origin: VoxelPos) -> Vec<SolidBox> {
    let e = BRICK_EDGE as usize;
    let at = |x: usize, y: usize, z: usize| solid[x + e * (y + e * z)];
    let mut used = vec![false; solid.len()];
    let mut out = Vec::new();
    for z in 0..e {
        for y in 0..e {
            for x in 0..e {
                if !at(x, y, z) || used[x + e * (y + e * z)] {
                    continue;
                }
                let free = |x: usize, y: usize, z: usize| at(x, y, z) && !used[x + e * (y + e * z)];
                let mut dx = 1;
                while x + dx < e && free(x + dx, y, z) {
                    dx += 1;
                }
                let mut dy = 1;
                'y: while y + dy < e {
                    for i in 0..dx {
                        if !free(x + i, y + dy, z) {
                            break 'y;
                        }
                    }
                    dy += 1;
                }
                let mut dz = 1;
                'z: while z + dz < e {
                    for j in 0..dy {
                        for i in 0..dx {
                            if !free(x + i, y + j, z + dz) {
                                break 'z;
                            }
                        }
                    }
                    dz += 1;
                }
                for k in 0..dz {
                    for j in 0..dy {
                        for i in 0..dx {
                            used[(x + i) + e * ((y + j) + e * (z + k))] = true;
                        }
                    }
                }
                let (x, y, z) = (x as i64, y as i64, z as i64);
                out.push(SolidBox {
                    min: VoxelPos::new(origin.x + x, origin.y + y, origin.z + z),
                    max: VoxelPos::new(
                        origin.x + x + dx as i64,
                        origin.y + y + dy as i64,
                        origin.z + z + dz as i64,
                    ),
                });
            }
        }
    }
    out.sort();
    out
}

/// The collision provider.
#[derive(Debug, Default, Clone, Copy)]
pub struct CollisionProvider;

impl Provider for CollisionProvider {
    fn kind(&self) -> DerivedKind {
        COLLISION_KIND
    }
    fn name(&self) -> &str {
        "voxel-collision"
    }
    fn version(&self) -> u32 {
        1
    }
    fn consumes(&self) -> ChangeMask {
        ChangeMask::GEOMETRY.or(ChangeMask::MATERIAL)
    }
    fn compute(&self, ctx: &ComputeContext<'_>) -> Result<Payload, ProviderFailure> {
        let key = ctx.target.key;
        let state = ctx
            .snapshot
            .cell(key.brick())
            .map_err(|e| ProviderFailure::Failed(e.to_string()))?;
        let origin = key
            .brick()
            .origin()
            .map_err(|e| ProviderFailure::Failed(e.to_string()))?;
        let boxes = match state {
            CellState::KnownEmpty | CellState::OutsideBounds => vec![],
            CellState::NotLoaded => {
                return Err(ProviderFailure::Failed(
                    "brick is not loaded: collision unknown".to_owned(),
                ));
            }
            CellState::LoadFailure(_) => {
                return Err(ProviderFailure::Failed("brick failed to load".to_owned()));
            }
            CellState::Present(p) => match &**p {
                CellPayload::Channels(set) => {
                    ctx.token.check()?;
                    let mut solid = vec![false; cube_core::BRICK_VOLUME];
                    if let Some(ch) = set.channel(ChannelId::Material) {
                        for l in LocalPos::all() {
                            solid[usize::from(l.x())
                                + 16 * (usize::from(l.y()) + 16 * usize::from(l.z()))] =
                                ch.get(l) != 0;
                        }
                    }
                    merge_boxes(&solid, origin)
                }
                CellPayload::MeshPage(_) => vec![],
            },
        };
        Ok(Arc::new(CollisionBoxes { key, boxes }))
    }
}
