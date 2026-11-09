//! Minimal voxel LOD provider (VOX-05; owner: Voxel Agent, DEC-089).
//!
//! One coarser level per brick: every 2x2x2 group of voxels becomes one coarse voxel (an 8^3
//! grid). A coarse voxel is solid if **any** child is solid (presence is never lost, so thin
//! structures survive; this is the conservative rule for collision and silhouettes), and its
//! material is the most frequent non-air child material, ties to the smaller material index
//! (deterministic). The result is derived data: the authoritative voxels are never touched and
//! there is no write path from a LOD back to Core. Mixed-resolution neighbours (a coarse brick
//! next to a fine one) belong to PS-9 and are not handled here; every brick is coarsened
//! independently, so borders stay consistent between equally coarsened bricks.
//!
//! A brick that is not loaded is a failure, never an all-air LOD.

use std::sync::Arc;

use cube_core::{CellKey, CellPayload, CellState, ChannelId, LocalPos};
use cube_processing::{
    ChangeMask, ComputeContext, DerivedKind, Payload, Provider, ProviderFailure,
};

/// Derived kind of the one-step LOD.
pub const LOD_KIND: DerivedKind = DerivedKind(0xB100_0004);

/// Edge of the coarse grid.
pub const LOD_EDGE: usize = 8;

/// The coarse materials of one brick, `cells[x + 8*(y + 8*z)]`, 0 = air.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LodBrick {
    /// The brick.
    pub key: CellKey,
    /// 512 coarse voxels.
    pub cells: Vec<u16>,
}

/// Coarsens a full-resolution material grid (`mat[x + 16*(y + 16*z)]`).
pub fn coarsen(mat: &[u16]) -> Vec<u16> {
    let mut out = vec![0u16; LOD_EDGE * LOD_EDGE * LOD_EDGE];
    for z in 0..LOD_EDGE {
        for y in 0..LOD_EDGE {
            for x in 0..LOD_EDGE {
                let mut counts: Vec<(u16, u8)> = Vec::new();
                for dz in 0..2 {
                    for dy in 0..2 {
                        for dx in 0..2 {
                            let m = mat[(2 * x + dx) + 16 * ((2 * y + dy) + 16 * (2 * z + dz))];
                            if m == 0 {
                                continue;
                            }
                            match counts.iter_mut().find(|(k, _)| *k == m) {
                                Some((_, n)) => *n += 1,
                                None => counts.push((m, 1)),
                            }
                        }
                    }
                }
                // Most frequent first, smaller material index on ties.
                counts.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
                out[x + LOD_EDGE * (y + LOD_EDGE * z)] = counts.first().map_or(0, |c| c.0);
            }
        }
    }
    out
}

/// The LOD provider.
#[derive(Debug, Default, Clone, Copy)]
pub struct LodProvider;

impl Provider for LodProvider {
    fn kind(&self) -> DerivedKind {
        LOD_KIND
    }
    fn name(&self) -> &str {
        "voxel-lod"
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
        let cells = match state {
            CellState::KnownEmpty | CellState::OutsideBounds => vec![0; LOD_EDGE.pow(3)],
            CellState::NotLoaded => {
                return Err(ProviderFailure::Failed(
                    "brick is not loaded: LOD unknown".to_owned(),
                ));
            }
            CellState::LoadFailure(_) => {
                return Err(ProviderFailure::Failed("brick failed to load".to_owned()));
            }
            CellState::Present(p) => match &**p {
                CellPayload::Channels(set) => {
                    ctx.token.check()?;
                    let mut mat = vec![0u16; cube_core::BRICK_VOLUME];
                    if let Some(ch) = set.channel(ChannelId::Material) {
                        for l in LocalPos::all() {
                            mat[usize::from(l.x())
                                + 16 * (usize::from(l.y()) + 16 * usize::from(l.z()))] = ch.get(l);
                        }
                    }
                    coarsen(&mat)
                }
                CellPayload::MeshPage(_) => vec![0; LOD_EDGE.pow(3)],
            },
        };
        Ok(Arc::new(LodBrick { key, cells }))
    }
}
