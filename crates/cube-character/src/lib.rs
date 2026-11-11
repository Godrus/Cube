//! `cube-character` (CHAR-01): the first high-detail, unsmoothed voxel character asset.
//!
//! One asset, one deterministic sculpt, a rig of parts, three LODs (coarsened voxel data, each
//! meshed with flat-shaded greedy quads). Engine neutral: an adapter copies the meshes into scene
//! nodes. Nothing here touches world state or gameplay; the authoritative collision of the player
//! stays the existing 0.6 x 1.8 body (see [`COLLISION_BOX`]).

pub mod grid;
pub mod mesh;
pub mod sculpt;
pub mod traveler;

use cube_core::ids::AssetId;
use grid::Grid;
pub use mesh::{MeshData, exposed_faces, mesh_face_area, mesh_grid};
pub use traveler::{PALETTE_LEN, PartId, palette};

/// Stable asset id of the traveller (hash of `cube.character.traveler.v1`, fixed forever).
pub const TRAVELER_ASSET_ID: AssetId = AssetId::new(0x5452_4156_454C_0001);

/// Default voxels per model unit of LOD0 (chosen from the density comparison in the docs).
pub const DEFAULT_RES: u32 = 80;

/// Authoritative collision representation: one box, `[min, max]` in model space. The detailed
/// voxels are never colliders.
pub const COLLISION_BOX: [[f32; 3]; 2] = [[-0.3, 0.0, -0.3], [0.3, 1.8, 0.3]];

/// One level of detail of one part.
#[derive(Debug, Clone)]
pub struct PartLod {
    /// Voxels per model unit at this level.
    pub res: f32,
    /// Voxel data.
    pub grid: Grid,
    /// Flat-shaded greedy mesh relative to the part pivot.
    pub mesh: MeshData,
}

/// One rig part with its LOD chain.
#[derive(Debug, Clone)]
pub struct Part {
    /// Identity.
    pub id: PartId,
    /// LOD0, LOD1, LOD2.
    pub lods: [PartLod; 3],
}

/// The built character asset.
#[derive(Debug, Clone)]
pub struct CharacterAsset {
    /// Voxels per unit at LOD0.
    pub res: u32,
    /// Parts in [`PartId::ALL`] order.
    pub parts: Vec<Part>,
}

/// Coarsening thresholds (children of eight) for LOD1 and LOD2; lower keeps thin parts.
const LOD_THRESHOLDS: [u32; 2] = [3, 2];

impl CharacterAsset {
    /// Builds the asset at `res` voxels per unit (must be divisible by 4 for the LOD chain).
    pub fn build(res: u32) -> Self {
        assert!(
            res >= 8 && res.is_multiple_of(4),
            "res must be a multiple of 4 and >= 8"
        );
        let parts = PartId::ALL
            .iter()
            .map(|&id| {
                let g0 = sculpt::rasterise(&id.ops(), res as f32, 0x51);
                let g1 = g0.coarsen(LOD_THRESHOLDS[0]);
                let g2 = g1.coarsen(LOD_THRESHOLDS[1]);
                let make = |g: Grid, r: f32| PartLod {
                    res: r,
                    mesh: mesh_grid(&g, r, id.mesh_origin(), true),
                    grid: g,
                };
                let r = res as f32;
                Part {
                    id,
                    lods: [make(g0, r), make(g1, r / 2.0), make(g2, r / 4.0)],
                }
            })
            .collect();
        Self { res, parts }
    }

    /// The stable asset id.
    pub const fn id(&self) -> AssetId {
        TRAVELER_ASSET_ID
    }

    /// Occupied voxels of all parts at a LOD.
    pub fn voxel_count(&self, lod: usize) -> usize {
        self.parts.iter().map(|p| p.lods[lod].grid.count()).sum()
    }

    /// Triangles of all parts at a LOD.
    pub fn triangle_count(&self, lod: usize) -> usize {
        self.parts
            .iter()
            .map(|p| p.lods[lod].mesh.triangle_count())
            .sum()
    }

    /// Vertices of all parts at a LOD.
    pub fn vertex_count(&self, lod: usize) -> usize {
        self.parts
            .iter()
            .map(|p| p.lods[lod].mesh.vertex_count())
            .sum()
    }

    /// CPU mesh memory of all parts at a LOD, in bytes.
    pub fn mesh_bytes(&self, lod: usize) -> usize {
        self.parts.iter().map(|p| p.lods[lod].mesh.bytes()).sum()
    }

    /// Voxel memory of all parts at a LOD, in bytes.
    pub fn grid_bytes(&self, lod: usize) -> usize {
        self.parts
            .iter()
            .map(|p| p.lods[lod].grid.cells.len())
            .sum()
    }

    /// Model-space bounds `[min, max]` of the occupied voxels at a LOD, all parts.
    pub fn bounds(&self, lod: usize) -> [[f32; 3]; 2] {
        let mut lo = [f32::MAX; 3];
        let mut hi = [f32::MIN; 3];
        for p in self.parts.iter().filter(|p| p.id != PartId::Sword) {
            // the sword is sculpted around its own grip and is excluded from body bounds
            let l = &p.lods[lod];
            let g = &l.grid;
            for z in 0..g.dims[2] as i32 {
                for y in 0..g.dims[1] as i32 {
                    for x in 0..g.dims[0] as i32 {
                        let c = [x + g.origin[0], y + g.origin[1], z + g.origin[2]];
                        if g.get(c) == 0 {
                            continue;
                        }
                        for i in 0..3 {
                            lo[i] = lo[i].min(c[i] as f32 / l.res);
                            hi[i] = hi[i].max((c[i] + 1) as f32 / l.res);
                        }
                    }
                }
            }
        }
        [lo, hi]
    }

    /// FNV-1a hash of every LOD0 grid (identity of the content; changes if the sculpt changes).
    pub fn content_hash(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |b: u8| {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        };
        for p in &self.parts {
            p.id.name().bytes().for_each(&mut eat);
            for v in p.lods[0]
                .grid
                .origin
                .iter()
                .chain(&p.lods[0].grid.dims.map(|d| d as i32))
            {
                v.to_le_bytes().into_iter().for_each(&mut eat);
            }
            p.lods[0].grid.cells.iter().copied().for_each(&mut eat);
        }
        h
    }
}

#[cfg(test)]
mod tests;
