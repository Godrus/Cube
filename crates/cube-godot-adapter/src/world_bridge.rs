//! Pure-Rust bridge for the world-generation spike (no Godot types), tested without Godot.
//!
//! Wraps `cube_worldgen::WorldStreamer`. The adapter owns no terrain truth: voxels live in the
//! streamer's Geometry Core asset; the bridge only converts positions, pumps generation and prepares
//! flat arrays for the renderer.

use std::path::Path;

use cube_core::VoxelPos;
use cube_worldgen::generator::TerrainGenerator;
use cube_worldgen::state_v0;
use cube_worldgen::{
    ChunkCoord, ChunkState, PumpSummary, SectionCoord, StateCounts, StreamConfig, UpdateSummary,
    WorldSeed, WorldStreamer, BRICKS_PER_CHUNK, CHUNK_HEIGHT, MAT_PLACED,
};

pub const MAX_RADIUS: i32 = 8;
/// Floats per rendered cube: 12 for a 3x4 transform + 4 for the colour (Godot `MultiMesh` buffer layout).
pub const FLOATS_PER_INSTANCE: usize = 16;

/// Visible-face mesh of one chunk (experimental renderer). Four vertices and six indices per face;
/// flat arrays ready to become Godot mesh arrays. Faces whose neighbouring voxel is occupied are not
/// emitted. A neighbour in a chunk that is not loaded counts as air; below the world (`y < 0`) counts as
/// solid, so the unreachable bottom faces are never drawn.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChunkMesh {
    /// `x, y, z` per vertex.
    pub positions: Vec<f32>,
    /// `x, y, z` per vertex.
    pub normals: Vec<f32>,
    /// `r, g, b, a` per vertex.
    pub colors: Vec<f32>,
    pub indices: Vec<u32>,
    /// Drawable elements: unit faces for the `faces` mesh, merged quads for the `greedy` mesh.
    pub faces: usize,
    /// Unit voxel faces covered (equals `faces` for the `faces` mesh; sum of quad areas for `greedy`).
    pub area: usize,
}

/// A greedy quad with the vertices of its boundary polygon (corners plus the extra points required to
/// avoid T-junctions) and its triangulation (indices into `vertices`, clockwise seen from outside like
/// every other face, no degenerate triangle, `vertices.len() - 2` triangles).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuadMesh {
    pub quad: Quad,
    pub vertices: Vec<[i32; 3]>,
    pub triangles: Vec<[usize; 3]>,
}

/// One merged rectangle of coplanar voxel faces of one material (greedy mesh). `voxel` is the voxel
/// with the smallest coordinates among the covered voxels; `size` is the extent in voxels along the
/// face's `u` and `v` axes (see `FACES`: both are positive unit axes with `u x v = normal`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Quad {
    pub normal: [i32; 3],
    pub voxel: [i32; 3],
    pub size: [i32; 2],
    pub material: u16,
}

/// One unit voxel face: the voxel, the outward normal and the material of the voxel.
pub type UnitFace = ([i32; 3], [i32; 3], u16);

impl Quad {
    /// The four corners in emission order `p, p+v*h, p+u*w+v*h, p+u*w` (clockwise from outside).
    pub fn corners(&self) -> [[i32; 3]; 4] {
        let (_, u, v, o) = FACES
            .iter()
            .copied()
            .find(|f| f.0 == self.normal)
            .expect("normal is one of the six axis directions");
        let p0 = [
            self.voxel[0] + o[0],
            self.voxel[1] + o[1],
            self.voxel[2] + o[2],
        ];
        let at = |a: i32, b: i32| {
            [
                p0[0] + a * u[0] * self.size[0] + b * v[0] * self.size[1],
                p0[1] + a * u[1] * self.size[0] + b * v[1] * self.size[1],
                p0[2] + a * u[2] * self.size[0] + b * v[2] * self.size[1],
            ]
        };
        [at(0, 0), at(0, 1), at(1, 1), at(1, 0)]
    }

    /// The positive `u` and `v` axes (as unit vectors) of the face this quad lies on.
    pub fn axes(&self) -> ([i32; 3], [i32; 3]) {
        let (_, u, v, _) = FACES
            .iter()
            .copied()
            .find(|f| f.0 == self.normal)
            .expect("normal is one of the six axis directions");
        (u, v)
    }

    /// Unit faces covered by this quad.
    pub fn unit_faces(&self) -> Vec<UnitFace> {
        let (u, v) = self.axes();
        let mut out = Vec::with_capacity((self.size[0] * self.size[1]) as usize);
        for j in 0..self.size[1] {
            for i in 0..self.size[0] {
                let mut p = self.voxel;
                for k in 0..3 {
                    p[k] += i * u[k] + j * v[k];
                }
                out.push((p, self.normal, self.material));
            }
        }
        out
    }
}

impl ChunkMesh {
    pub fn vertex_count(&self) -> usize {
        self.positions.len() / 3
    }

    /// Mesh of greedy quads with their triangulations: one polygon per quad, same vertex layout and
    /// winding as the `faces` mesh. `faces` counts quads, `area` the unit faces they cover.
    pub fn from_quad_meshes(quads: &[QuadMesh]) -> ChunkMesh {
        let mut mesh = ChunkMesh::default();
        for qm in quads {
            let n = qm.quad.normal;
            let col = material_color(qm.quad.material);
            let base = (mesh.positions.len() / 3) as u32;
            for p in &qm.vertices {
                for k in 0..3 {
                    mesh.positions.push(p[k] as f32);
                    mesh.normals.push(n[k] as f32);
                }
                mesh.colors.extend_from_slice(&col);
            }
            for t in &qm.triangles {
                mesh.indices.extend_from_slice(&[
                    base + t[0] as u32,
                    base + t[1] as u32,
                    base + t[2] as u32,
                ]);
            }
            mesh.faces += 1;
            mesh.area += (qm.quad.size[0] * qm.quad.size[1]) as usize;
        }
        mesh
    }

    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Bytes of the arrays as handed to the renderer (positions, normals, colours, 32-bit indices).
    pub fn byte_size(&self) -> usize {
        (self.positions.len() + self.normals.len() + self.colors.len()) * 4 + self.indices.len() * 4
    }
}

impl ChunkMesh {
    /// Appends another mesh (indices rebased). Chunk mesh == section 0 appended with section 1.
    pub fn append(&mut self, o: &ChunkMesh) {
        let base = self.vertex_count() as u32;
        self.positions.extend_from_slice(&o.positions);
        self.normals.extend_from_slice(&o.normals);
        self.colors.extend_from_slice(&o.colors);
        self.indices.extend(o.indices.iter().map(|i| i + base));
        self.faces += o.faces;
        self.area += o.area;
    }

    /// Hash of the face list (FNV-1a 64 over integer vertex positions, normals, indices and 8-bit
    /// colours). For tests and comparisons only; not a persisted format.
    pub fn content_hash(&self) -> u64 {
        let mut h = 0xCBF2_9CE4_8422_2325u64;
        let mut feed = |v: u32| {
            for b in v.to_le_bytes() {
                h ^= b as u64;
                h = h.wrapping_mul(0x0000_0100_0000_01B3);
            }
        };
        for v in self.positions.iter().chain(self.normals.iter()) {
            feed(*v as i32 as u32);
        }
        for c in &self.colors {
            feed((c * 255.0).round() as u32);
        }
        for i in &self.indices {
            feed(*i);
        }
        h
    }
}

const PAD_X: usize = 18;
const PAD_Z: usize = 18;
/// Padded grid height: `y = -1 ..= CHUNK_HEIGHT` (one cell of padding on both ends).
const PAD_Y: usize = CHUNK_HEIGHT as usize + 2;
const SOLID_BELOW_WORLD: u16 = u16::MAX;

/// Face table: outward normal, and the two edge vectors `u`, `v` with `u x v = normal`; `origin` is the
/// corner offset inside the voxel. Corners are emitted `p, p+v, p+u+v, p+u`, which is clockwise seen
/// from outside, the front-face winding of Godot.
type Face = ([i32; 3], [i32; 3], [i32; 3], [i32; 3]);
const FACES: [Face; 6] = [
    ([1, 0, 0], [0, 1, 0], [0, 0, 1], [1, 0, 0]),
    ([-1, 0, 0], [0, 0, 1], [0, 1, 0], [0, 0, 0]),
    ([0, 1, 0], [0, 0, 1], [1, 0, 0], [0, 1, 0]),
    ([0, -1, 0], [1, 0, 0], [0, 0, 1], [0, 0, 0]),
    ([0, 0, 1], [1, 0, 0], [0, 1, 0], [0, 0, 1]),
    ([0, 0, -1], [0, 1, 0], [1, 0, 0], [0, 0, 0]),
];

fn pad_index(x: i32, y: i32, z: i32) -> usize {
    ((y + 1) as usize * PAD_Z + (z + 1) as usize) * PAD_X + (x + 1) as usize
}

/// Appends the visible faces of voxel `p` (material `m`); `hidden(normal)` says whether the neighbour
/// in that direction is occupied.
fn emit_faces(mesh: &mut ChunkMesh, p: VoxelPos, m: u16, hidden: impl Fn([i32; 3]) -> bool) {
    let col = material_color(m);
    for (n, u, v, o) in FACES {
        if hidden(n) {
            continue;
        }
        let base = (mesh.positions.len() / 3) as u32;
        let p0 = [p.x + o[0], p.y + o[1], p.z + o[2]];
        for (a, b) in [(0, 0), (0, 1), (1, 1), (1, 0)] {
            for k in 0..3 {
                mesh.positions.push((p0[k] + a * u[k] + b * v[k]) as f32);
                mesh.normals.push(n[k] as f32);
            }
            mesh.colors.extend_from_slice(&col);
        }
        mesh.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        mesh.faces += 1;
        mesh.area += 1;
    }
}

const SEC_N: usize = 18;

/// A section's own voxels and an 18^3 grid with one voxel of padding on every side.
struct SectionData {
    cells: Vec<(VoxelPos, u16)>,
    grid: Vec<u16>,
    ox: i32,
    oy: i32,
    oz: i32,
}

impl SectionData {
    fn idx(x: i32, y: i32, z: i32) -> usize {
        ((y + 1) as usize * SEC_N + (z + 1) as usize) * SEC_N + (x + 1) as usize
    }
}

/// True if the axis-aligned edge `a -> b` lies in one of the six boundary planes of the section whose
/// minimum corner is `origin` (a plane `axis = origin[axis]` or `origin[axis] + 16`, `axis` not the
/// edge direction).
pub fn edge_on_section_boundary(a: [i32; 3], b: [i32; 3], origin: [i32; 3]) -> bool {
    (0..3).any(|k| a[k] == b[k] && (a[k] == origin[k] || a[k] == origin[k] + 16))
}

/// Which extra boundary vertices a greedy quad carries (both rules are crack-free; see
/// `WorldBridge::section_quad_meshes_with`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeamRule {
    /// Corners of other quads on the edge, plus every lattice point on section boundary planes.
    Minimal,
    /// `Minimal`, plus every lattice point along an edge stretch that borders a surface of another colour
    /// or plane (a different material, a perpendicular wall, or the section boundary). Those are the
    /// stretches where two different-coloured surfaces meet on screen, so they are subdivided exactly
    /// like the one-face-per-voxel mesh and the GPU snaps them identically. Edge stretches between two
    /// coplanar quads of the same material keep only the corner vertices.
    VisibleEdges,
}

type FaceMap = std::collections::HashMap<([i32; 3], [i32; 3]), u16>;

/// True if the unit stretch `k` of edge `e` (0..4, in emission order) of quad `q` borders a surface that
/// is not a coplanar face of the same material (`faces` maps (voxel, normal) -> material of the section's
/// visible unit faces; a face outside the section is unknown and counts as different).
fn stretch_borders_other_surface(q: &Quad, e: usize, k: i32, faces: &FaceMap) -> bool {
    let (u, v) = q.axes();
    let add =
        |p: [i32; 3], d: [i32; 3], t: i32| [p[0] + t * d[0], p[1] + t * d[1], p[2] + t * d[2]];
    let (w, h) = (q.size[0], q.size[1]);
    let outside = match e {
        0 => add(add(q.voxel, v, k), u, -1),
        1 => add(add(add(q.voxel, v, h - 1), u, k), v, 1),
        2 => add(add(add(q.voxel, u, w - 1), v, h - 1 - k), u, 1),
        _ => add(add(add(q.voxel, u, w - 1 - k), v, 0), v, -1),
    };
    faces.get(&(outside, q.normal)) != Some(&q.material)
}

/// Boundary polygon of a quad with T-junction vertices, triangulated without degenerate triangles.
///
/// The quad `c0 c1 c2 c3` is split along the diagonal `c0 c2` into the triangles `T1 = (c0, c1, c2)` and
/// `T2 = (c2, c3, c0)`. The diagonal is interior and never needs extra vertices; the other two sides of
/// each triangle may. For a triangle `A B C` with chains `A..B` (points `a_1..a_m`) and `B..C` (points
/// `b_1..b_n`): a fan from `C` over `A, a_1, .., a_m` plus a fan from `a_m` over `B, b_1, .., b_n, C`
/// cover it exactly with `m + n + 1` non-degenerate triangles, using every boundary vertex.
fn triangulate_quad(
    q: Quad,
    corners: &std::collections::HashSet<[i32; 3]>,
    origin: [i32; 3],
    rule: SeamRule,
    faces: &FaceMap,
) -> QuadMesh {
    let c = q.corners();
    let mut poly: Vec<[i32; 3]> = Vec::new();
    // Index chains per edge, each from the start corner to the end corner (inclusive).
    let mut chains: Vec<Vec<usize>> = Vec::new();
    for e in 0..4 {
        let (a, b) = (c[e], c[(e + 1) % 4]);
        let len = (0..3).map(|k| (b[k] - a[k]).abs()).sum::<i32>();
        let dir = [
            (b[0] - a[0]).signum(),
            (b[1] - a[1]).signum(),
            (b[2] - a[2]).signum(),
        ];
        let full = edge_on_section_boundary(a, b, origin);
        let mut chain = Vec::new();
        if e == 0 {
            chain.push(poly.len());
            poly.push(a);
        } else {
            chain.push(poly.len() - 1);
        }
        for k in 1..len {
            let p = [a[0] + k * dir[0], a[1] + k * dir[1], a[2] + k * dir[2]];
            let visible = rule == SeamRule::VisibleEdges
                && (stretch_borders_other_surface(&q, e, k - 1, faces)
                    || stretch_borders_other_surface(&q, e, k, faces));
            if full || visible || corners.contains(&p) {
                chain.push(poly.len());
                poly.push(p);
            }
        }
        if e < 3 {
            chain.push(poly.len());
            poly.push(b);
        } else {
            chain.push(0); // back to c0
        }
        chains.push(chain);
    }
    let mut triangles: Vec<[usize; 3]> = Vec::new();
    // (leg A..B, leg B..C): T1 uses edges 0 and 1, T2 uses edges 2 and 3.
    for (ab, bc) in [(&chains[0], &chains[1]), (&chains[2], &chains[3])] {
        let c_apex = *bc.last().unwrap();
        // fan from C over A .. a_m (everything on A..B except the last segment to B)
        for w in ab[..ab.len() - 1].windows(2) {
            triangles.push([c_apex, w[0], w[1]]);
        }
        // fan from a_m over B .. C
        let a_m = ab[ab.len() - 2];
        for w in bc.windows(2) {
            triangles.push([a_m, w[0], w[1]]);
        }
    }
    QuadMesh {
        quad: q,
        vertices: poly,
        triangles,
    }
}

/// Seam rule used by [`WorldBridge::section_mesh_greedy`].
pub const GREEDY_SEAM_RULE: SeamRule = SeamRule::VisibleEdges;

pub struct WorldBridge {
    w: WorldStreamer,
    added: Vec<ChunkCoord>,
    removed: Vec<ChunkCoord>,
    /// Spike-local inverse-edit journal (NOT the Editing API undo): per edit batch the voxels it changed.
    undo: Vec<EditBatch>,
    redo: Vec<EditBatch>,
}

/// One user edit: `(position, material before, material after)` of every voxel that changed.
type EditBatch = Vec<(VoxelPos, u16, u16)>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadSummary {
    pub saved_active: usize,
    pub restored_active: usize,
    pub missing: usize,
    pub edits_restored: usize,
}

fn floor_i32(v: f32) -> i32 {
    if v.is_nan() {
        0
    } else {
        v.floor().clamp(-2_000_000_000.0, 2_000_000_000.0) as i32
    }
}

pub fn material_color(m: u16) -> [f32; 4] {
    match m {
        1 => [0.35, 0.75, 0.25, 1.0],
        2 => [0.45, 0.30, 0.18, 1.0],
        3 => [0.50, 0.50, 0.55, 1.0],
        MAT_PLACED => [0.95, 0.55, 0.15, 1.0],
        _ => [1.0, 0.0, 1.0, 1.0],
    }
}

impl WorldBridge {
    pub fn new(seed: u64, radius: i32) -> Self {
        let cfg = StreamConfig {
            load_radius: radius.clamp(0, MAX_RADIUS),
            ..StreamConfig::default()
        };
        Self {
            w: WorldStreamer::new(WorldSeed(seed), Box::new(TerrainGenerator), cfg),
            added: Vec::new(),
            removed: Vec::new(),
            undo: Vec::new(),
            redo: Vec::new(),
        }
    }

    pub fn seed(&self) -> u64 {
        self.w.seed().0
    }

    pub fn radius(&self) -> i32 {
        self.w.config().load_radius
    }

    /// Sets the player/camera position (world units); queues chunks around it and unloads far ones.
    pub fn set_player(&mut self, x: f32, _y: f32, z: f32) -> UpdateSummary {
        self.w.set_center_world(floor_i32(x), floor_i32(z))
    }

    pub fn pump(&mut self, budget: usize) -> PumpSummary {
        self.w.pump(budget)
    }

    pub fn pump_all(&mut self) -> PumpSummary {
        self.w.pump_all()
    }

    fn drain_delta(&mut self) {
        let d = self.w.take_delta();
        self.added.extend(d.added);
        self.removed.extend(d.removed);
    }

    /// Chunks that became Ready since the last call (the client should add them).
    pub fn take_added(&mut self) -> Vec<[i32; 2]> {
        self.drain_delta();
        std::mem::take(&mut self.added)
            .into_iter()
            .map(|c| [c.x, c.z])
            .collect()
    }

    /// Chunks that were unloaded since the last call (the client should remove them).
    pub fn take_removed(&mut self) -> Vec<[i32; 2]> {
        self.drain_delta();
        std::mem::take(&mut self.removed)
            .into_iter()
            .map(|c| [c.x, c.z])
            .collect()
    }

    pub fn take_modified(&mut self) -> Vec<[i32; 2]> {
        self.w
            .take_modified()
            .into_iter()
            .map(|c| [c.x, c.z])
            .collect()
    }

    /// Neighbouring chunks whose border faces changed because of an edit on a shared border.
    pub fn take_border_touched(&mut self) -> Vec<[i32; 2]> {
        self.w
            .take_border_touched()
            .into_iter()
            .map(|c| [c.x, c.z])
            .collect()
    }

    /// Mesh sections `[chunk x, brick, chunk z]` whose faces may have changed because of edits (see
    /// `WorldStreamer::take_dirty_sections` for the dirty-region rule).
    pub fn take_dirty_sections(&mut self) -> Vec<[i32; 3]> {
        self.w
            .take_dirty_sections()
            .into_iter()
            .map(|s| [s.chunk.x, s.brick as i32, s.chunk.z])
            .collect()
    }

    /// Sets one voxel to `material` and records the change in `batch`; `true` if it changed.
    fn set_recorded(&mut self, p: VoxelPos, material: u16, batch: &mut EditBatch) -> bool {
        let before = self.w.voxel(p);
        if let (Some(old), Ok(true)) = (before, self.w.set_voxel(p, material)) {
            batch.push((p, old, material));
            true
        } else {
            false
        }
    }

    fn finish_batch(&mut self, batch: EditBatch) {
        if !batch.is_empty() {
            self.undo.push(batch);
            self.redo.clear();
        }
    }

    /// Sets one voxel (Ready chunks only); `true` if it changed. One undo step.
    pub fn edit_voxel(&mut self, x: i32, y: i32, z: i32, material: u16) -> bool {
        let mut batch = Vec::new();
        let changed = self.set_recorded(VoxelPos::new(x, y, z), material, &mut batch);
        self.finish_batch(batch);
        changed
    }

    /// Reverts the last edit batch; returns how many voxels were restored (0 if nothing to undo).
    /// Voxels whose chunk was unloaded meanwhile are skipped. Spike-local helper, not the approved
    /// Editing API undo.
    pub fn undo(&mut self) -> usize {
        let Some(batch) = self.undo.pop() else {
            return 0;
        };
        let n = self.apply_batch(&batch, true);
        self.redo.push(batch);
        n
    }

    pub fn redo(&mut self) -> usize {
        let Some(batch) = self.redo.pop() else {
            return 0;
        };
        let n = self.apply_batch(&batch, false);
        self.undo.push(batch);
        n
    }

    fn apply_batch(&mut self, batch: &EditBatch, revert: bool) -> usize {
        let mut n = 0;
        let order: Vec<_> = if revert {
            batch.iter().rev().collect()
        } else {
            batch.iter().collect()
        };
        for &(p, old, new) in order {
            let target = if revert { old } else { new };
            if matches!(self.w.set_voxel(p, target), Ok(true)) {
                n += 1;
            }
        }
        n
    }

    /// Fills the inclusive box with `material` (Ready chunks only, y clamped to the world); one undo
    /// step. Returns how many voxels changed.
    pub fn fill_box(&mut self, a: [i32; 3], b: [i32; 3], material: u16) -> usize {
        let mut batch = Vec::new();
        for y in a[1].min(b[1]).max(0)..=a[1].max(b[1]).min(CHUNK_HEIGHT - 1) {
            for z in a[2].min(b[2])..=a[2].max(b[2]) {
                for x in a[0].min(b[0])..=a[0].max(b[0]) {
                    self.set_recorded(VoxelPos::new(x, y, z), material, &mut batch);
                }
            }
        }
        let n = batch.len();
        self.finish_batch(batch);
        n
    }

    pub fn undo_depth(&self) -> usize {
        self.undo.len()
    }

    pub fn redo_depth(&self) -> usize {
        self.redo.len()
    }

    pub fn active_chunks(&self) -> Vec<[i32; 2]> {
        self.w
            .active_chunks()
            .into_iter()
            .map(|c| [c.x, c.z])
            .collect()
    }

    pub fn active_count(&self) -> usize {
        self.w.counts().ready
    }

    pub fn active_voxels(&self) -> usize {
        self.w.active_voxels()
    }

    pub fn counts(&self) -> StateCounts {
        self.w.counts()
    }

    pub fn queue_len(&self) -> usize {
        self.w.queue_len()
    }

    pub fn chunk_state(&self, cx: i32, cz: i32) -> ChunkState {
        self.w.state(ChunkCoord::new(cx, cz))
    }

    pub fn chunk_voxel_count(&self, cx: i32, cz: i32) -> Option<usize> {
        self.w.chunk_cells(ChunkCoord::new(cx, cz)).map(|v| v.len())
    }

    /// Flat `x, y, z, material` of every non-air voxel of a Ready chunk.
    pub fn chunk_voxels(&self, cx: i32, cz: i32) -> Option<Vec<i32>> {
        let cells = self.w.chunk_cells(ChunkCoord::new(cx, cz))?;
        let mut out = Vec::with_capacity(cells.len() * 4);
        for (p, m) in cells {
            out.extend_from_slice(&[p.x, p.y, p.z, m as i32]);
        }
        Some(out)
    }

    /// Ready-made `MultiMesh.buffer` content: per cube a row-major 3x4 identity-rotation transform at
    /// the voxel centre followed by an RGBA colour.
    pub fn chunk_instance_buffer(&self, cx: i32, cz: i32) -> Option<Vec<f32>> {
        let cells = self.w.chunk_cells(ChunkCoord::new(cx, cz))?;
        let mut out = Vec::with_capacity(cells.len() * FLOATS_PER_INSTANCE);
        for (p, m) in cells {
            let c = material_color(m);
            out.extend_from_slice(&[
                1.0,
                0.0,
                0.0,
                p.x as f32 + 0.5,
                0.0,
                1.0,
                0.0,
                p.y as f32 + 0.5,
                0.0,
                0.0,
                1.0,
                p.z as f32 + 0.5,
                c[0],
                c[1],
                c[2],
                c[3],
            ]);
        }
        Some(out)
    }

    /// Visible-face mesh of a Ready chunk in chunk-local-free world coordinates (same space as
    /// `chunk_instance_buffer`). Reads border neighbours from the neighbouring chunks, so a chunk must
    /// be rebuilt when a neighbour is loaded, unloaded or edited next to the border.
    pub fn chunk_mesh(&self, cx: i32, cz: i32) -> Option<ChunkMesh> {
        let coord = ChunkCoord::new(cx, cz);
        let cells = self.w.chunk_cells(coord)?;
        let (ox, oz) = (coord.origin_x(), coord.origin_z());
        let mut grid = vec![0u16; PAD_X * PAD_Y * PAD_Z];
        for z in -1..=16 {
            for x in -1..=16 {
                grid[pad_index(x, -1, z)] = SOLID_BELOW_WORLD;
            }
        }
        for &(p, m) in &cells {
            grid[pad_index(p.x - ox, p.y, p.z - oz)] = m;
        }
        for y in 0..CHUNK_HEIGHT {
            for i in 0..16 {
                for (lx, lz) in [(-1, i), (16, i), (i, -1), (i, 16)] {
                    let m = self
                        .w
                        .voxel(VoxelPos::new(ox + lx, y, oz + lz))
                        .unwrap_or(0);
                    grid[pad_index(lx, y, lz)] = m;
                }
            }
        }
        let mut mesh = ChunkMesh::default();
        for &(p, m) in &cells {
            let (lx, ly, lz) = (p.x - ox, p.y, p.z - oz);
            emit_faces(&mut mesh, p, m, |n| {
                grid[pad_index(lx + n[0], ly + n[1], lz + n[2])] != 0
            });
        }
        Some(mesh)
    }

    /// Cells of one section plus a padded 18^3 grid (the section, one voxel of its neighbours on every
    /// side: other brick, neighbouring chunks, world limits). Shared by the `faces` and `greedy` builders.
    fn section_data(&self, cx: i32, bi: i32, cz: i32) -> Option<SectionData> {
        if !(0..BRICKS_PER_CHUNK as i32).contains(&bi) {
            return None;
        }
        let coord = ChunkCoord::new(cx, cz);
        let cells = self.w.section_cells(SectionCoord {
            chunk: coord,
            brick: bi as u8,
        })?;
        let (ox, oy, oz) = (coord.origin_x(), bi * 16, coord.origin_z());
        let mut d = SectionData {
            cells,
            grid: vec![0u16; SEC_N * SEC_N * SEC_N],
            ox,
            oy,
            oz,
        };
        for k in 0..d.cells.len() {
            let (p, m) = d.cells[k];
            let i = SectionData::idx(p.x - ox, p.y - oy, p.z - oz);
            d.grid[i] = m;
        }
        // Padding planes above and below inside the chunk (other brick) or the world limit.
        for z in 0..16 {
            for x in 0..16 {
                d.grid[SectionData::idx(x, -1, z)] = if oy == 0 {
                    SOLID_BELOW_WORLD
                } else {
                    self.w
                        .voxel(VoxelPos::new(ox + x, oy - 1, oz + z))
                        .unwrap_or(0)
                };
                d.grid[SectionData::idx(x, 16, z)] = if oy + 16 >= CHUNK_HEIGHT {
                    0
                } else {
                    self.w
                        .voxel(VoxelPos::new(ox + x, oy + 16, oz + z))
                        .unwrap_or(0)
                };
            }
        }
        for y in 0..16 {
            for i in 0..16 {
                for (lx, lz) in [(-1, i), (16, i), (i, -1), (i, 16)] {
                    d.grid[SectionData::idx(lx, y, lz)] = self
                        .w
                        .voxel(VoxelPos::new(ox + lx, oy + y, oz + lz))
                        .unwrap_or(0);
                }
            }
        }
        Some(d)
    }

    /// Face mesh of one mesh section (brick `bi` of chunk `(cx, cz)`): only the faces of the section's
    /// own voxels. `chunk_mesh(c)` equals section 0 appended with section 1 (same faces, same order).
    /// Neighbouring voxels are read from the other brick of the chunk and the neighbouring chunks.
    pub fn section_mesh(&self, cx: i32, bi: i32, cz: i32) -> Option<ChunkMesh> {
        let d = self.section_data(cx, bi, cz)?;
        let mut mesh = ChunkMesh::default();
        for &(p, m) in &d.cells {
            let (lx, ly, lz) = (p.x - d.ox, p.y - d.oy, p.z - d.oz);
            emit_faces(&mut mesh, p, m, |n| {
                d.grid[SectionData::idx(lx + n[0], ly + n[1], lz + n[2])] != 0
            });
        }
        Some(mesh)
    }

    /// Reference list of the visible unit faces of a section (voxel, normal, material), computed
    /// directly from the voxels: a face is visible iff the voxel is solid and its neighbour is not.
    pub fn section_unit_faces(&self, cx: i32, bi: i32, cz: i32) -> Option<Vec<UnitFace>> {
        let d = self.section_data(cx, bi, cz)?;
        let mut out = Vec::new();
        for &(p, m) in &d.cells {
            let (lx, ly, lz) = (p.x - d.ox, p.y - d.oy, p.z - d.oz);
            for (n, _, _, _) in FACES {
                if d.grid[SectionData::idx(lx + n[0], ly + n[1], lz + n[2])] == 0 {
                    out.push(([p.x, p.y, p.z], n, m));
                }
            }
        }
        Some(out)
    }

    /// Greedy quads of one section. For each of the six directions and each of the 16 slices along the
    /// normal, a 16x16 mask holds the material of every visible face of the section's own voxels; the
    /// mask is covered with maximal rectangles of one material (scan v outer, u inner: extend along u,
    /// then along v while the whole row matches). Rectangles never leave the section's 16x16 slice, so no
    /// quad crosses a section (brick or chunk) boundary. Order of quads: direction, slice, v, u.
    pub fn section_quads(&self, cx: i32, bi: i32, cz: i32) -> Option<Vec<Quad>> {
        let d = self.section_data(cx, bi, cz)?;
        let mut quads = Vec::new();
        for (n, u, v, _) in FACES {
            let axis = |a: [i32; 3]| a.iter().position(|&c| c != 0).unwrap();
            let (an, au, av) = (axis(n), axis(u), axis(v));
            for slice in 0..16 {
                let mut mask = [0u16; 256];
                let mut any = false;
                for iv in 0..16 {
                    for iu in 0..16 {
                        let mut l = [0i32; 3];
                        l[an] = slice;
                        l[au] = iu;
                        l[av] = iv;
                        let m = d.grid[SectionData::idx(l[0], l[1], l[2])];
                        if m != 0
                            && d.grid[SectionData::idx(l[0] + n[0], l[1] + n[1], l[2] + n[2])] == 0
                        {
                            mask[(iv * 16 + iu) as usize] = m;
                            any = true;
                        }
                    }
                }
                if !any {
                    continue;
                }
                for iv in 0..16usize {
                    let mut iu = 0usize;
                    while iu < 16 {
                        let m = mask[iv * 16 + iu];
                        if m == 0 {
                            iu += 1;
                            continue;
                        }
                        let mut w = 1;
                        while iu + w < 16 && mask[iv * 16 + iu + w] == m {
                            w += 1;
                        }
                        let mut h = 1;
                        'rows: while iv + h < 16 {
                            for k in 0..w {
                                if mask[(iv + h) * 16 + iu + k] != m {
                                    break 'rows;
                                }
                            }
                            h += 1;
                        }
                        for dv in 0..h {
                            for du in 0..w {
                                mask[(iv + dv) * 16 + iu + du] = 0;
                            }
                        }
                        let mut l = [0i32; 3];
                        l[an] = slice;
                        l[au] = iu as i32;
                        l[av] = iv as i32;
                        quads.push(Quad {
                            normal: n,
                            voxel: [l[0] + d.ox, l[1] + d.oy, l[2] + d.oz],
                            size: [w as i32, h as i32],
                            material: m,
                        });
                        iu += w;
                    }
                }
            }
        }
        Some(quads)
    }

    /// Greedy mesh of a section without T-junctions: every quad is emitted as a rectangle whose boundary
    /// carries the extra vertices of [`GREEDY_SEAM_RULE`], split along a diagonal and fan-triangulated.
    /// Same vertex layout and winding as the `faces` mesh. `faces` counts quads, `area` the unit faces
    /// they cover.
    pub fn section_mesh_greedy(&self, cx: i32, bi: i32, cz: i32) -> Option<ChunkMesh> {
        Some(ChunkMesh::from_quad_meshes(
            &self.section_quad_meshes(cx, bi, cz)?,
        ))
    }

    /// The greedy quads of a section with their conforming triangulation (see [`QuadMesh`]).
    ///
    /// T-junction rule. A vertex lying in the interior of another quad's edge makes the GPU rasterise the
    /// two sides slightly differently (pinholes), so every quad edge carries
    ///  * every integer point on it that is a corner of another quad of the same section, and
    ///  * **every** integer point on it if the edge lies in one of the section's six boundary planes
    ///    (the faces of its 16^3 brick). The neighbouring section follows the same rule on the shared
    ///    plane, so both sides get identical vertices without looking at each other: a section's mesh still
    ///    depends only on its own quads, which keeps the partial rebuild rule intact.
    ///
    /// [`SeamRule::VisibleEdges`] (the default) adds every lattice point along edge stretches that border a
    /// surface of another colour or plane. Without it the picture still differs from the one-face-per-voxel
    /// mesh in a few pixels per frame, where two different surfaces meet along an edge that is subdivided
    /// differently in the two meshes (sub-pixel snapping of the vertices).
    pub fn section_quad_meshes(&self, cx: i32, bi: i32, cz: i32) -> Option<Vec<QuadMesh>> {
        self.section_quad_meshes_with(cx, bi, cz, GREEDY_SEAM_RULE)
    }

    /// Like [`Self::section_quad_meshes`] with an explicit [`SeamRule`] (for comparison).
    pub fn section_quad_meshes_with(
        &self,
        cx: i32,
        bi: i32,
        cz: i32,
        rule: SeamRule,
    ) -> Option<Vec<QuadMesh>> {
        let quads = self.section_quads(cx, bi, cz)?;
        let mut faces = FaceMap::new();
        if rule == SeamRule::VisibleEdges {
            for q in &quads {
                for (p, n, m) in q.unit_faces() {
                    faces.insert((p, n), m);
                }
            }
        }
        let origin = [cx * 16, bi * 16, cz * 16];
        let mut corners: std::collections::HashSet<[i32; 3]> = std::collections::HashSet::new();
        for q in &quads {
            for c in q.corners() {
                corners.insert(c);
            }
        }
        Some(
            quads
                .into_iter()
                .map(|q| triangulate_quad(q, &corners, origin, rule, &faces))
                .collect(),
        )
    }

    pub fn ground_height(&self, x: i32, z: i32) -> i32 {
        TerrainGenerator::height_at(self.w.seed(), x, z)
    }

    /// Places a 3x3x3 block of `MAT_PLACED` centred on `(x, y, z)`; returns how many voxels changed.
    /// Voxels in chunks that are not Ready are skipped.
    pub fn edit_around(&mut self, x: i32, y: i32, z: i32) -> usize {
        let mut batch = Vec::new();
        for dy in -1..=1 {
            for dz in -1..=1 {
                for dx in -1..=1 {
                    let yy = (y + dy).clamp(0, CHUNK_HEIGHT - 1);
                    self.set_recorded(VoxelPos::new(x + dx, yy, z + dz), MAT_PLACED, &mut batch);
                }
            }
        }
        let changed = batch.len();
        self.finish_batch(batch);
        changed
    }

    pub fn edit_count(&self) -> usize {
        self.w.edit_count()
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        self.w.save_state(path).map_err(|e| e.to_string())
    }

    /// Replaces the world with the saved one (same seed, config, edits, chunks regenerated).
    pub fn load(&mut self, path: &Path) -> Result<LoadSummary, String> {
        let (w, rep) =
            state_v0::load(path, Box::new(TerrainGenerator)).map_err(|e| e.to_string())?;
        self.w = w;
        self.undo.clear();
        self.redo.clear();
        // The client clears everything on load and then takes all active chunks as newly added.
        self.added = self.w.active_chunks();
        self.removed.clear();
        Ok(LoadSummary {
            saved_active: rep.saved_active,
            restored_active: rep.restored_active,
            missing: rep.missing.len(),
            edits_restored: rep.edits_restored,
        })
    }

    pub fn approx_brick_bytes(&self) -> usize {
        self.w.approx_brick_bytes()
    }
}
