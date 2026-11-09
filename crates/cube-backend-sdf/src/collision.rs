//! Collision provider of the SDF backend (SDF-03; provisional like every provider of PS-5).
//!
//! Solid boxes of one brick: a voxel is solid where its field value is negative; solid voxels
//! are merged into runs along x per row, in the fixed order (z, y, x). At most 16 x 16 x 8 = 2048
//! boxes per brick (the budget); a brick that is not loaded, or failed to load, is a failure,
//! never "empty". Neighbouring bricks are not needed (apron 0): collision of a voxel does not
//! depend on its neighbours. The box set follows the sampled field, so it differs from the
//! smooth Surface Nets surface by at most half a voxel (documented tolerance).

use std::sync::Arc;

use cube_core::{BrickCoord, CellPayload, CellState, ChannelId, LocalPos};
use cube_processing::{
    ChangeMask, ComputeContext, DerivedKind, Payload, Provider, ProviderFailure,
};

/// Derived kind of the SDF collision boxes.
pub const SDF_COLLISION_KIND: DerivedKind = DerivedKind(0xB200_0002);

/// A solid box in voxel coordinates, `[min, max)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SdfBox {
    /// Lower corner.
    pub min: [i64; 3],
    /// Upper corner (exclusive).
    pub max: [i64; 3],
}

/// The solid boxes of one brick.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SdfCollision {
    /// The brick.
    pub key: cube_core::CellKey,
    /// Boxes in the fixed order.
    pub boxes: Vec<SdfBox>,
}

/// The provider.
#[derive(Debug, Default, Clone, Copy)]
pub struct SdfCollisionProvider;

impl Provider for SdfCollisionProvider {
    fn kind(&self) -> DerivedKind {
        SDF_COLLISION_KIND
    }
    fn name(&self) -> &str {
        "sdf-collision"
    }
    fn version(&self) -> u32 {
        1
    }
    fn consumes(&self) -> ChangeMask {
        ChangeMask::GEOMETRY
    }
    fn compute(&self, ctx: &ComputeContext<'_>) -> Result<Payload, ProviderFailure> {
        let key = ctx.target.key;
        let centre: BrickCoord = key.brick();
        let state = ctx
            .snapshot
            .cell(centre)
            .map_err(|e| ProviderFailure::Failed(e.to_string()))?;
        let set = match state {
            CellState::NotLoaded => {
                return Err(ProviderFailure::Failed("brick is not loaded".to_owned()));
            }
            CellState::LoadFailure(_) => {
                return Err(ProviderFailure::Failed("brick failed to load".to_owned()));
            }
            CellState::OutsideBounds | CellState::KnownEmpty => {
                return Ok(Arc::new(SdfCollision { key, boxes: vec![] }));
            }
            CellState::Present(p) => match &**p {
                CellPayload::Channels(set) => set,
                CellPayload::MeshPage(_) => {
                    return Err(ProviderFailure::Failed(
                        "brick holds a mesh page".to_owned(),
                    ));
                }
            },
        };
        let origin = centre
            .origin()
            .map_err(|e| ProviderFailure::Failed(e.to_string()))?;
        let Some(dens) = set.channel(ChannelId::Density) else {
            return Ok(Arc::new(SdfCollision { key, boxes: vec![] }));
        };
        let solid =
            |x: u8, y: u8, z: u8| LocalPos::new(x, y, z).is_ok_and(|l| (dens.get(l) as i16) < 0);
        let mut boxes = Vec::new();
        for z in 0..16u8 {
            ctx.token.check()?;
            for y in 0..16u8 {
                let mut x = 0u8;
                while x < 16 {
                    if !solid(x, y, z) {
                        x += 1;
                        continue;
                    }
                    let start = x;
                    while x < 16 && solid(x, y, z) {
                        x += 1;
                    }
                    let min = [
                        origin.x + i64::from(start),
                        origin.y + i64::from(y),
                        origin.z + i64::from(z),
                    ];
                    boxes.push(SdfBox {
                        min,
                        max: [origin.x + i64::from(x), min[1] + 1, min[2] + 1],
                    });
                }
            }
        }
        Ok(Arc::new(SdfCollision { key, boxes }))
    }
}
