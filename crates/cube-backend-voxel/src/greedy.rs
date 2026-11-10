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
//! **No T-junctions in the triangle mesh (PS-5).** A merged rectangle next to unit quads
//! necessarily has foreign vertices on its edges (the corners of its neighbours). That is fine
//! for a *quad list* and wrong for a *mesh*, so the mesh is built by [`triangulate`], which adds
//! every foreign edge vertex to the polygon of the merged quad (a fan around the quad centre),
//! so every vertex on an edge is a vertex of both triangles that meet there. Two rules make the
//! set of vertices on the edges of a quad complete inside one brick:
//!
//! 1. quads in a plane that lies on a brick border are never merged (the other side of that
//!    plane is the neighbouring brick, whose perpendicular faces would touch the quad edge
//!    from outside);
//! 2. interior quads merge only inside the ring (cells 1..=14), so every face that touches an
//!    edge of a merged quad belongs to the same brick.
//!
//! [`mesh_t_junctions`] is the independent checker; the tests run it over whole multi-brick
//! worlds, not only per brick. Coverage equality with faces is necessary but is not what proves
//! this. Greedy is not the default renderer: that needs a decision and measurements (GOV-13).

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
            // A plane on the brick border is never merged (rule 1).
            let local = (slice - [origin.x, origin.y, origin.z][na]) as usize;
            let on_border = match d {
                Dir::PosX | Dir::PosY | Dir::PosZ => local == edge - 1,
                _ => local == 0,
            };
            let interior = |u: usize, v: usize| {
                !on_border && (1..edge - 1).contains(&u) && (1..edge - 1).contains(&v)
            };
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

/// The four corner points of a quad in lattice coordinates (the face plane is at `slice + 1`
/// for positive directions and at `slice` for negative ones).
fn corners(q: &Quad) -> [[i64; 3]; 4] {
    let (ua, va, na) = axes(q.dir);
    let plane = coord(q.origin, na)
        + match q.dir {
            Dir::PosX | Dir::PosY | Dir::PosZ => 1,
            _ => 0,
        };
    let base = [q.origin.x, q.origin.y, q.origin.z];
    let mut out = [[0i64; 3]; 4];
    for (k, (du, dv)) in [(0, 0), (1, 0), (1, 1), (0, 1)].into_iter().enumerate() {
        let mut p = [0i64; 3];
        p[na] = plane;
        p[ua] = base[ua] + du * i64::from(q.u_len);
        p[va] = base[va] + dv * i64::from(q.v_len);
        out[k] = p;
    }
    out
}

/// Lattice points strictly inside the edges of `q`.
fn edge_points(q: &Quad) -> Vec<[i64; 3]> {
    let c = corners(q);
    let mut out = Vec::new();
    for k in 0..4 {
        let (a, b) = (c[k], c[(k + 1) % 4]);
        let axis = (0..3).find(|i| a[*i] != b[*i]).expect("edges have length");
        let (lo, hi) = (a[axis].min(b[axis]), a[axis].max(b[axis]));
        for t in lo + 1..hi {
            let mut p = a;
            p[axis] = t;
            out.push(p);
        }
    }
    out
}

/// Every (quad, foreign vertex strictly inside one of its edges) in `quads`: the T-junctions.
/// Independent of how the quads were made, so it can check a whole world of bricks at once.
pub fn t_junctions(quads: &[Quad]) -> Vec<(Quad, [i64; 3])> {
    let vertices: std::collections::BTreeSet<[i64; 3]> = quads.iter().flat_map(corners).collect();
    let mut out = Vec::new();
    for q in quads {
        for p in edge_points(q) {
            if vertices.contains(&p) {
                out.push((*q, p));
            }
        }
    }
    out
}

/// Indexed triangles of a greedy mesh. Positions are in **half voxels** (twice the lattice
/// coordinate), so quad centres are exact integers.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TriMesh {
    /// Unique positions, ascending.
    pub vertices: Vec<[i64; 3]>,
    /// Triangles, counter-clockwise seen from outside, with direction and material.
    pub triangles: Vec<([u32; 3], Dir, u16)>,
}

impl TriMesh {
    /// The union of several meshes: vertices with equal positions become one (how a renderer
    /// sees neighbouring bricks).
    pub fn union(meshes: &[&TriMesh]) -> TriMesh {
        let mut pos: std::collections::BTreeSet<[i64; 3]> = Default::default();
        for m in meshes {
            pos.extend(m.vertices.iter().copied());
        }
        let vertices: Vec<[i64; 3]> = pos.into_iter().collect();
        let index: std::collections::BTreeMap<[i64; 3], u32> = vertices
            .iter()
            .enumerate()
            .map(|(i, p)| (*p, i as u32))
            .collect();
        let mut triangles = Vec::new();
        for m in meshes {
            for (t, d, mat) in &m.triangles {
                triangles.push((t.map(|i| index[&m.vertices[i as usize]]), *d, *mat));
            }
        }
        TriMesh {
            vertices,
            triangles,
        }
    }

    /// Sum over all triangles of the axis-aligned area measure (the absolute components of the
    /// cross product), an exact integer in half-voxel units, for coverage comparisons: for
    /// axis-aligned planar triangles it is twice the area.
    pub fn double_area(&self) -> i128 {
        let mut a = 0i128;
        for (t, _, _) in &self.triangles {
            let p: [[i128; 3]; 3] = t.map(|i| self.vertices[i as usize].map(i128::from));
            let u = [p[1][0] - p[0][0], p[1][1] - p[0][1], p[1][2] - p[0][2]];
            let w = [p[2][0] - p[0][0], p[2][1] - p[0][1], p[2][2] - p[0][2]];
            let c = [
                u[1] * w[2] - u[2] * w[1],
                u[2] * w[0] - u[0] * w[2],
                u[0] * w[1] - u[1] * w[0],
            ];
            a += c[0].abs() + c[1].abs() + c[2].abs();
        }
        a
    }
}

fn gcd(a: i64, b: i64) -> i64 {
    if b == 0 { a.abs() } else { gcd(b, a % b) }
}

/// Number of (triangle edge, vertex strictly inside it) pairs: the T-junctions of a triangle
/// mesh. Zero is the contract.
pub fn mesh_t_junctions(m: &TriMesh) -> usize {
    let vs: std::collections::BTreeSet<[i64; 3]> = m.vertices.iter().copied().collect();
    let mut edges: std::collections::BTreeSet<([i64; 3], [i64; 3])> = Default::default();
    for (t, _, _) in &m.triangles {
        for k in 0..3 {
            let (a, b) = (
                m.vertices[t[k] as usize],
                m.vertices[t[(k + 1) % 3] as usize],
            );
            edges.insert((a.min(b), a.max(b)));
        }
    }
    let mut n = 0;
    for (a, b) in edges {
        let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let g = gcd(gcd(d[0], d[1]), d[2]);
        for t in 1..g {
            let p = [
                a[0] + d[0] / g * t,
                a[1] + d[1] / g * t,
                a[2] + d[2] / g * t,
            ];
            if vs.contains(&p) {
                n += 1;
            }
        }
    }
    n
}

/// Triangulates quads without T-junctions: a quad whose edges carry foreign vertices becomes a
/// fan around its centre through every boundary vertex; any other quad becomes two triangles.
/// Deterministic; covers exactly the area of the quads.
pub fn triangulate(quads: &[Quad]) -> TriMesh {
    let all: std::collections::BTreeSet<[i64; 3]> = quads.iter().flat_map(corners).collect();
    let mut positions: std::collections::BTreeSet<[i64; 3]> = Default::default();
    let mut raw: Vec<(Vec<[i64; 3]>, Option<[i64; 3]>, Dir, u16)> = Vec::new();
    for q in quads {
        let c = corners(q).map(|p| p.map(|v| 2 * v));
        // Boundary ring in (u, v) counter-clockwise order, with the foreign vertices.
        let mut ring: Vec<[i64; 3]> = Vec::new();
        for k in 0..4 {
            let (a, b) = (c[k], c[(k + 1) % 4]);
            ring.push(a);
            let axis = (0..3).find(|i| a[*i] != b[*i]).expect("edges have length");
            let step = (b[axis] - a[axis]).signum();
            let mut t = a[axis] + 2 * step;
            while t != b[axis] {
                let mut p = a;
                p[axis] = t;
                if all.contains(&p.map(|v| v / 2)) {
                    ring.push(p);
                }
                t += 2 * step;
            }
        }
        // (u, v, normal) is right handed for X and Z and left handed for Y.
        let handed = if q.dir.axis() == 1 { -1 } else { 1 };
        let want = match q.dir {
            Dir::PosX | Dir::PosY | Dir::PosZ => 1,
            _ => -1,
        };
        if handed != want {
            ring.reverse();
        }
        let centre = (ring.len() > 4).then(|| {
            [
                (c[0][0] + c[2][0]) / 2,
                (c[0][1] + c[2][1]) / 2,
                (c[0][2] + c[2][2]) / 2,
            ]
        });
        positions.extend(ring.iter().copied());
        positions.extend(centre);
        raw.push((ring, centre, q.dir, q.material));
    }
    let vertices: Vec<[i64; 3]> = positions.into_iter().collect();
    let index: std::collections::BTreeMap<[i64; 3], u32> = vertices
        .iter()
        .enumerate()
        .map(|(i, p)| (*p, i as u32))
        .collect();
    let mut triangles = Vec::new();
    for (ring, centre, dir, material) in raw {
        let id = |p: &[i64; 3]| index[p];
        match centre {
            None => {
                triangles.push(([id(&ring[0]), id(&ring[1]), id(&ring[2])], dir, material));
                triangles.push(([id(&ring[0]), id(&ring[2]), id(&ring[3])], dir, material));
            }
            Some(c) => {
                for k in 0..ring.len() {
                    triangles.push((
                        [id(&c), id(&ring[k]), id(&ring[(k + 1) % ring.len()])],
                        dir,
                        material,
                    ));
                }
            }
        }
    }
    TriMesh {
        vertices,
        triangles,
    }
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
