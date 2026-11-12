//! Surface Nets provider (SDF-02; provider ownership pending O-1, so provisional like every
//! provider of PS-5).
//!
//! The mesh of ONE brick from a Processing snapshot with an apron of one brick. A sample is the
//! field value at a voxel centre; a *cell* is the cube between eight neighbouring samples and
//! holds one vertex if the sign of the field changes inside it (inside = negative). The brick
//! owns every edge whose lower sample lies in the brick and emits the quad around it. The four
//! cells of a quad can lie in the lower neighbours (hence the apron), and a vertex depends only
//! on the eight samples of its cell, so two bricks compute the same boundary vertex bit for bit:
//! **no cracks at brick joins** (checked by merging bricks and testing that every edge is shared
//! by exactly two triangles).
//!
//! Positions are exact fixed-point values (1/256 voxel, `i128`, so there is no coordinate limit
//! below the grid end). Normals come from the eight samples of the cell only ([`crate::gradient`]),
//! which keeps seams identical too. Absent data is explicit: a neighbouring brick that is not
//! loaded (or outside the bounds) counts as outside and the result says so
//! (`unloaded_neighbours`); a brick that failed to load, or the centre brick not being loaded,
//! is a failure, never an empty mesh. Output order is fixed, so results are deterministic.

use std::collections::BTreeMap;
use std::sync::Arc;

use cube_core::{BRICK_EDGE, BrickCoord, CellPayload, CellState, ChannelId, LocalPos};
use cube_processing::{
    ChangeMask, ComputeContext, DerivedKind, Payload, Provider, ProviderFailure, Snapshot,
};

use crate::field::OUTSIDE;
use crate::gradient::cell_normal_q14;

/// Derived kind of the SDF surface mesh. The SDF backend owns ids `0xB200_0000..0xB201_0000`.
pub const SDF_MESH_KIND: DerivedKind = DerivedKind(0xB200_0001);

/// One mesh vertex.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vertex {
    /// World position in 1/256 voxel.
    pub pos: [i128; 3],
    /// Unit normal in Q14 (16384 = 1.0), pointing out of the solid.
    pub normal: [i16; 3],
    /// Material of the first solid corner that has one (0 if none).
    pub material: u16,
}

/// The surface of one brick.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceMesh {
    /// The brick the mesh belongs to.
    pub key: cube_core::CellKey,
    /// Vertices in cell order.
    pub vertices: Vec<Vertex>,
    /// Counter-clockwise triangles seen from outside.
    pub triangles: Vec<[u32; 3]>,
    /// `true` if a neighbouring brick was not loaded (or outside the bounds) and was treated as
    /// outside.
    pub unloaded_neighbours: bool,
}

const N: usize = 18; // samples per axis: -1..=16

fn at(x: i64, y: i64, z: i64) -> usize {
    (x + 1) as usize + N * ((y + 1) as usize + N * (z + 1) as usize)
}

/// Density and material of the centre brick and its apron, index `-1..=16` on each axis.
pub(crate) struct Field {
    pub(crate) d: Vec<i16>,
    pub(crate) m: Vec<u16>,
    pub(crate) unloaded: bool,
}

pub(crate) fn gather(snapshot: &Snapshot, centre: BrickCoord) -> Result<Field, String> {
    let mut f = Field {
        d: vec![OUTSIDE; N * N * N],
        m: vec![0; N * N * N],
        unloaded: false,
    };
    for dz in -1i64..=1 {
        for dy in -1i64..=1 {
            for dx in -1i64..=1 {
                let (Some(x), Some(y), Some(z)) = (
                    centre.x.checked_add(dx),
                    centre.y.checked_add(dy),
                    centre.z.checked_add(dz),
                ) else {
                    continue; // beyond the end of the grid: nothing there, outside
                };
                let b = BrickCoord::new(x, y, z);
                let state = snapshot
                    .cell(b)
                    .map_err(|e| format!("snapshot read: {e}"))?;
                let set = match state {
                    CellState::Present(p) => match &**p {
                        CellPayload::Channels(set) => Some(set),
                        CellPayload::MeshPage(_) => {
                            return Err(format!("brick {b:?} holds a mesh page"));
                        }
                    },
                    CellState::KnownEmpty => None,
                    CellState::NotLoaded | CellState::OutsideBounds => {
                        if b != centre {
                            f.unloaded = true;
                        }
                        None
                    }
                    CellState::LoadFailure(r) => {
                        return Err(format!("brick {b:?} failed to load ({r:?})"));
                    }
                };
                let Some(set) = set else { continue };
                let dens = set.channel(ChannelId::Density);
                let mat = set.channel(ChannelId::Material);
                // Copy the part of this brick that falls into -1..=16 around the centre.
                for l in LocalPos::all() {
                    let g = [
                        dx * 16 + i64::from(l.x()),
                        dy * 16 + i64::from(l.y()),
                        dz * 16 + i64::from(l.z()),
                    ];
                    if g.iter().all(|v| (-1..=16).contains(v)) {
                        let i = at(g[0], g[1], g[2]);
                        f.d[i] = dens.map_or(OUTSIDE, |c| (c.get(l) as u16) as i16);
                        // Mesh pages carry 16-bit materials (wire format, MAT-02 widens it): a wider
                        // material is a controlled failure, never a silent truncation.
                        f.m[i] = u16::try_from(mat.map_or(0, |c| c.get(l))).map_err(|_| {
                            "material index above 65535: mesh format is 16-bit (MAT-02)".to_owned()
                        })?;
                    }
                }
            }
        }
    }
    Ok(f)
}

/// The Surface Nets provider.
#[derive(Debug, Default, Clone, Copy)]
pub struct SurfaceNetsProvider;

impl Provider for SurfaceNetsProvider {
    fn kind(&self) -> DerivedKind {
        SDF_MESH_KIND
    }
    fn name(&self) -> &str {
        "sdf-surface-nets"
    }
    fn version(&self) -> u32 {
        1
    }
    fn consumes(&self) -> ChangeMask {
        ChangeMask::GEOMETRY.or(ChangeMask::MATERIAL)
    }
    fn apron(&self) -> u32 {
        1
    }
    fn compute(&self, ctx: &ComputeContext<'_>) -> Result<Payload, ProviderFailure> {
        let key = ctx.target.key;
        let centre = key.brick();
        let state = ctx
            .snapshot
            .cell(centre)
            .map_err(|e| ProviderFailure::Failed(e.to_string()))?;
        match state {
            CellState::NotLoaded => {
                return Err(ProviderFailure::Failed("brick is not loaded".to_owned()));
            }
            CellState::LoadFailure(_) => {
                return Err(ProviderFailure::Failed("brick failed to load".to_owned()));
            }
            CellState::OutsideBounds | CellState::KnownEmpty => {
                return Ok(Arc::new(SurfaceMesh {
                    key,
                    vertices: vec![],
                    triangles: vec![],
                    unloaded_neighbours: false,
                }));
            }
            CellState::Present(_) => {}
        }
        let origin = centre
            .origin()
            .map_err(|e| ProviderFailure::Failed(e.to_string()))?;
        let f = gather(ctx.snapshot, centre).map_err(ProviderFailure::Failed)?;
        let inside = |x: i64, y: i64, z: i64| f.d[at(x, y, z)] < 0;

        // Vertices are created on demand, ordered by cell.
        let mut cells: BTreeMap<(i64, i64, i64), u32> = BTreeMap::new();
        let mut needed: Vec<(i64, i64, i64)> = Vec::new();
        let mut tris: Vec<[(i64, i64, i64); 4]> = Vec::new();
        let mut flips: Vec<bool> = Vec::new();
        for z in 0..BRICK_EDGE {
            ctx.token.check()?;
            for y in 0..BRICK_EDGE {
                for x in 0..BRICK_EDGE {
                    let s = [x, y, z];
                    for a in 0..3usize {
                        let mut t = s;
                        t[a] += 1;
                        let (si, ti) = (inside(s[0], s[1], s[2]), inside(t[0], t[1], t[2]));
                        if si == ti {
                            continue;
                        }
                        let (b, c) = ((a + 1) % 3, (a + 2) % 3);
                        let off = |u: i64, v: i64| {
                            let mut p = s;
                            p[b] -= u;
                            p[c] -= v;
                            (p[0], p[1], p[2])
                        };
                        let q = [off(0, 0), off(1, 0), off(1, 1), off(0, 1)];
                        for cell in q {
                            needed.push(cell);
                        }
                        tris.push(q);
                        flips.push(!si);
                    }
                }
            }
        }
        needed.sort_unstable();
        needed.dedup();
        let mut vertices = Vec::with_capacity(needed.len());
        for (i, c) in needed.iter().enumerate() {
            cells.insert(*c, i as u32);
            vertices.push(vertex(&f, origin_of(origin), *c));
        }
        let mut triangles = Vec::with_capacity(tris.len() * 2);
        for (q, flip) in tris.iter().zip(&flips) {
            let mut v = [cells[&q[0]], cells[&q[1]], cells[&q[2]], cells[&q[3]]];
            if *flip {
                v.reverse();
            }
            triangles.push([v[0], v[1], v[2]]);
            triangles.push([v[0], v[2], v[3]]);
        }
        Ok(Arc::new(SurfaceMesh {
            key,
            vertices,
            triangles,
            unloaded_neighbours: f.unloaded,
        }))
    }
}

fn origin_of(o: cube_core::VoxelPos) -> [i128; 3] {
    [i128::from(o.x), i128::from(o.y), i128::from(o.z)]
}

/// The vertex of the cell whose lower corner is local sample `c`.
fn vertex(f: &Field, origin: [i128; 3], c: (i64, i64, i64)) -> Vertex {
    let corner = |k: usize| {
        let (dx, dy, dz) = ((k & 1) as i64, ((k >> 1) & 1) as i64, ((k >> 2) & 1) as i64);
        at(c.0 + dx, c.1 + dy, c.2 + dz)
    };
    let d: [i16; 8] = std::array::from_fn(|k| f.d[corner(k)]);
    // Edge crossings: pairs of corners that differ in exactly one bit.
    let mut sum = [0.0f64; 3];
    let mut n = 0.0f64;
    for k in 0..8usize {
        for bit in 0..3 {
            if k & (1 << bit) != 0 {
                continue;
            }
            let j = k | (1 << bit);
            let (a, b) = (d[k], d[j]);
            if (a < 0) != (b < 0) {
                let t = f64::from(a) / (f64::from(a) - f64::from(b));
                let mut p = [(k & 1) as f64, ((k >> 1) & 1) as f64, ((k >> 2) & 1) as f64];
                p[bit] += t;
                for i in 0..3 {
                    sum[i] += p[i];
                }
                n += 1.0;
            }
        }
    }
    let mut pos = [0i128; 3];
    for i in 0..3 {
        let local = [c.0, c.1, c.2][i];
        // Sample centres sit half a voxel above the lattice.
        let frac = ((sum[i] / n + 0.5) * 256.0).round() as i128;
        pos[i] = (origin[i] + i128::from(local)) * 256 + frac;
    }
    let material = (0..8)
        .filter(|k| d[*k] < 0)
        .map(|k| f.m[corner(k)])
        .find(|m| *m != 0)
        .unwrap_or(0);
    Vertex {
        pos,
        normal: cell_normal_q14(d),
        material,
    }
}
