//! Greedy provider: material-aware merging of coplanar faces (VOX-03; pending O-1).
//!
//! `faces` is the oracle; greedy is an optimisation candidate. The quads of a greedy mesh cover
//! exactly the unit faces of the faces mesh, no more and no less (tested as set equality), every
//! quad has one material, and the result is deterministic (fixed scan and sort order).
//!
//! **Seam rule** (`SeamRule::UnitRing`): a quad never crosses a brick border, and faces on the
//! outermost ring of a brick's face plane are emitted as unit quads, so every vertex position on
//! a brick border exists on both sides of it and no quad edge on a border contains a foreign
//! vertex. Interior quads (cells 1..=14 in both plane axes) are merged greedily.
//!
//! **Known limit (honest scope):** T-junctions *inside* a brick between an interior merged quad
//! and the unit ring around it remain possible; they are not claimed to be fixed here. The
//! renderer or a later pass may fix them; the correctness oracle is coverage, not topology.
//! Greedy is not the default renderer: that needs a decision and measurements (GOV-13, DG-1).

use std::sync::Arc;

use cube_core::{BRICK_EDGE, CellKey, VoxelPos};
use cube_processing::{
    ChangeMask, ComputeContext, DerivedKind, Payload, Provider, ProviderFailure,
};

use crate::faces::{Dir, FACES_KIND, Face, FacesMesh};

/// Derived kind of the greedy mesh.
pub const GREEDY_KIND: DerivedKind = DerivedKind(0xB100_0002);

/// How quads behave at brick borders.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeamRule {
    /// Border ring cells stay unit quads; merging happens only inside the ring.
    UnitRing,
}

/// A rectangle of coplanar faces with one material.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Quad {
    /// Direction of the normal.
    pub dir: Dir,
    /// The voxel whose face is the minimum corner of the rectangle.
    pub origin: VoxelPos,
    /// Extent along the first in-plane axis (in voxels, >= 1).
    pub u_len: u32,
    /// Extent along the second in-plane axis (in voxels, >= 1).
    pub v_len: u32,
    /// Material.
    pub material: u16,
}

/// The quads of one brick.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GreedyMesh {
    /// The brick.
    pub key: CellKey,
    /// Quads in the fixed order.
    pub quads: Vec<Quad>,
    /// As in the faces mesh.
    pub unloaded_neighbours: bool,
    /// The seam rule applied.
    pub seam: SeamRule,
}

/// In-plane axes `(u, v)` and the normal axis for a direction.
const fn axes(d: Dir) -> (usize, usize, usize) {
    match d.axis() {
        0 => (1, 2, 0),
        1 => (0, 2, 1),
        _ => (0, 1, 2),
    }
}

fn coord(p: VoxelPos, a: usize) -> i64 {
    [p.x, p.y, p.z][a]
}

/// Merges the faces of one brick into quads. `origin` is the voxel minimum of the brick.
pub fn merge(faces: &[Face], origin: VoxelPos) -> Vec<Quad> {
    let edge = BRICK_EDGE as usize;
    let mut quads = Vec::new();
    for d in Dir::ALL {
        let (ua, va, na) = axes(d);
        // Group faces of this direction by slice along the normal.
        let mut slices: std::collections::BTreeMap<i64, Vec<&Face>> = Default::default();
        for f in faces.iter().filter(|f| f.dir == d) {
            slices.entry(coord(f.voxel, na)).or_default().push(f);
        }
        for (slice, fs) in slices {
            let mut grid = vec![0u16; edge * edge];
            for f in &fs {
                let u = (coord(f.voxel, ua) - [origin.x, origin.y, origin.z][ua]) as usize;
                let v = (coord(f.voxel, va) - [origin.x, origin.y, origin.z][va]) as usize;
                grid[v * edge + u] = f.material;
            }
            let mut done = vec![false; edge * edge];
            let interior =
                |u: usize, v: usize| (1..edge - 1).contains(&u) && (1..edge - 1).contains(&v);
            for v in 0..edge {
                for u in 0..edge {
                    let m = grid[v * edge + u];
                    if m == 0 || done[v * edge + u] {
                        continue;
                    }
                    let (mut w, mut h) = (1usize, 1usize);
                    if interior(u, v) {
                        while u + w < edge
                            && interior(u + w, v)
                            && grid[v * edge + u + w] == m
                            && !done[v * edge + u + w]
                        {
                            w += 1;
                        }
                        'grow: while v + h < edge {
                            for k in 0..w {
                                if !interior(u + k, v + h)
                                    || grid[(v + h) * edge + u + k] != m
                                    || done[(v + h) * edge + u + k]
                                {
                                    break 'grow;
                                }
                            }
                            h += 1;
                        }
                    }
                    for dv in 0..h {
                        for du in 0..w {
                            done[(v + dv) * edge + u + du] = true;
                        }
                    }
                    let mut o = [0i64; 3];
                    o[ua] = [origin.x, origin.y, origin.z][ua] + u as i64;
                    o[va] = [origin.x, origin.y, origin.z][va] + v as i64;
                    o[na] = slice;
                    quads.push(Quad {
                        dir: d,
                        origin: VoxelPos::new(o[0], o[1], o[2]),
                        u_len: w as u32,
                        v_len: h as u32,
                        material: m,
                    });
                }
            }
        }
    }
    quads.sort();
    quads
}

/// Expands quads back to unit faces (for coverage checks and for consumers that need them).
pub fn expand(quads: &[Quad]) -> Vec<Face> {
    let mut out = Vec::new();
    for q in quads {
        let (ua, va, _) = axes(q.dir);
        for dv in 0..i64::from(q.v_len) {
            for du in 0..i64::from(q.u_len) {
                let mut p = [q.origin.x, q.origin.y, q.origin.z];
                p[ua] += du;
                p[va] += dv;
                out.push(Face {
                    voxel: VoxelPos::new(p[0], p[1], p[2]),
                    dir: q.dir,
                    material: q.material,
                });
            }
        }
    }
    out.sort();
    out
}

/// The greedy provider. It consumes the faces mesh of the same brick as a derived input, so the
/// two can never disagree about which faces exist.
#[derive(Debug, Default, Clone, Copy)]
pub struct GreedyProvider;

impl Provider for GreedyProvider {
    fn kind(&self) -> DerivedKind {
        GREEDY_KIND
    }
    fn name(&self) -> &str {
        "voxel-greedy"
    }
    fn version(&self) -> u32 {
        1
    }
    fn consumes(&self) -> ChangeMask {
        ChangeMask::NONE
    }
    fn depends_on(&self) -> &[DerivedKind] {
        &[FACES_KIND]
    }
    fn compute(&self, ctx: &ComputeContext<'_>) -> Result<Payload, ProviderFailure> {
        let key = ctx.target.key;
        let input = ctx
            .input(FACES_KIND)
            .ok_or_else(|| ProviderFailure::Failed("faces input missing".to_owned()))?;
        let faces = input
            .downcast_ref::<FacesMesh>()
            .ok_or_else(|| ProviderFailure::Failed("faces input has the wrong type".to_owned()))?;
        ctx.token.check()?;
        let origin = key
            .brick()
            .origin()
            .map_err(|e| ProviderFailure::Failed(e.to_string()))?;
        Ok(Arc::new(GreedyMesh {
            key,
            quads: merge(&faces.faces, origin),
            unloaded_neighbours: faces.unloaded_neighbours,
            seam: SeamRule::UnitRing,
        }))
    }
}
