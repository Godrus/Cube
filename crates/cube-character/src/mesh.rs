//! Face-culled greedy meshing with flat normals: every face stays a flat axis-aligned quad.

use crate::grid::Grid;

/// CPU mesh of one part at one LOD (positions relative to the part pivot, in model units).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MeshData {
    /// xyz per vertex.
    pub positions: Vec<f32>,
    /// Flat face normal xyz per vertex.
    pub normals: Vec<f32>,
    /// Material per vertex.
    pub materials: Vec<u8>,
    /// Triangle indices, counter-clockwise seen from outside.
    pub indices: Vec<u32>,
}

impl MeshData {
    /// Vertex count.
    pub fn vertex_count(&self) -> usize {
        self.materials.len()
    }
    /// Triangle count.
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }
    /// Memory of the CPU buffers in bytes.
    pub fn bytes(&self) -> usize {
        self.positions.len() * 4
            + self.normals.len() * 4
            + self.materials.len()
            + self.indices.len() * 4
    }
}

/// Number of exposed unit faces of the grid (the reference for "no missing faces").
pub fn exposed_faces(g: &Grid) -> usize {
    let mut n = 0;
    for z in 0..g.dims[2] as i32 {
        for y in 0..g.dims[1] as i32 {
            for x in 0..g.dims[0] as i32 {
                let p = [x + g.origin[0], y + g.origin[1], z + g.origin[2]];
                if g.get(p) == 0 {
                    continue;
                }
                for a in 0..3 {
                    for s in [-1, 1] {
                        let mut q = p;
                        q[a] += s;
                        if g.get(q) == 0 {
                            n += 1;
                        }
                    }
                }
            }
        }
    }
    n
}

/// Meshes a grid. `res` is voxels per model unit; `pivot` is subtracted from every position.
/// With `greedy` false every exposed face is its own quad (used to cross-check the merge).
pub fn mesh_grid(g: &Grid, res: f32, pivot: [f32; 3], greedy: bool) -> MeshData {
    let mut m = MeshData::default();
    let d = g.dims.map(|v| v as usize);
    for a in 0..3 {
        let (u, v) = ((a + 1) % 3, (a + 2) % 3);
        let (du, dv) = (d[u], d[v]);
        for dir in [-1i32, 1] {
            let mut mask = vec![0u8; du * dv];
            for s in 0..d[a] {
                for j in 0..dv {
                    for i in 0..du {
                        let mut p = [0i32; 3];
                        p[a] = s as i32 + g.origin[a];
                        p[u] = i as i32 + g.origin[u];
                        p[v] = j as i32 + g.origin[v];
                        let c = g.get(p);
                        let mut q = p;
                        q[a] += dir;
                        mask[i + j * du] = if c != 0 && g.get(q) == 0 { c } else { 0 };
                    }
                }
                for j in 0..dv {
                    let mut i = 0;
                    while i < du {
                        let c = mask[i + j * du];
                        if c == 0 {
                            i += 1;
                            continue;
                        }
                        let (mut w, mut h) = (1, 1);
                        if greedy {
                            while i + w < du && mask[i + w + j * du] == c {
                                w += 1;
                            }
                            'grow: while j + h < dv {
                                for k in 0..w {
                                    if mask[i + k + (j + h) * du] != c {
                                        break 'grow;
                                    }
                                }
                                h += 1;
                            }
                        }
                        for jj in 0..h {
                            for ii in 0..w {
                                mask[i + ii + (j + jj) * du] = 0;
                            }
                        }
                        let plane = (s as i32 + g.origin[a] + i32::from(dir > 0)) as f32;
                        let corner = |cu: usize, cv: usize| -> [f32; 3] {
                            let mut p = [0.0f32; 3];
                            p[a] = plane;
                            p[u] = (i + cu) as f32 + g.origin[u] as f32;
                            p[v] = (j + cv) as f32 + g.origin[v] as f32;
                            [0, 1, 2].map(|k| p[k] / res - pivot[k])
                        };
                        let mut quad = [corner(0, 0), corner(w, 0), corner(w, h), corner(0, h)];
                        if dir < 0 {
                            quad.reverse();
                        }
                        let base = m.vertex_count() as u32;
                        let mut n = [0.0f32; 3];
                        n[a] = dir as f32;
                        for p in quad {
                            m.positions.extend_from_slice(&p);
                            m.normals.extend_from_slice(&n);
                            m.materials.push(c);
                        }
                        m.indices.extend_from_slice(&[
                            base,
                            base + 1,
                            base + 2,
                            base,
                            base + 2,
                            base + 3,
                        ]);
                        i += w;
                    }
                }
            }
        }
    }
    m
}

/// Total surface area, in unit faces of the grid, covered by the mesh quads.
pub fn mesh_face_area(m: &MeshData, res: f32) -> usize {
    let mut area = 0.0f64;
    for q in m.indices.chunks_exact(6) {
        let p = |i: u32| {
            let o = 3 * i as usize;
            [m.positions[o], m.positions[o + 1], m.positions[o + 2]]
        };
        let (a, b, c) = (p(q[0]), p(q[1]), p(q[2]));
        let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let e2 = [c[0] - b[0], c[1] - b[1], c[2] - b[2]];
        let cr = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        area += f64::from((cr[0] * cr[0] + cr[1] * cr[1] + cr[2] * cr[2]).sqrt());
    }
    (area * f64::from(res) * f64::from(res) + 0.5) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_voxel_has_six_flat_quads() {
        let mut g = Grid::new([0, 0, 0], [1, 1, 1]);
        g.set([0, 0, 0], 1);
        let m = mesh_grid(&g, 1.0, [0.0; 3], true);
        assert_eq!(m.triangle_count(), 12);
        assert_eq!(mesh_face_area(&m, 1.0), 6);
    }

    #[test]
    fn greedy_merges_but_covers_same_area() {
        let mut g = Grid::new([0, 0, 0], [4, 4, 4]);
        for z in 0..4 {
            for y in 0..4 {
                for x in 0..4 {
                    g.set([x, y, z], 2);
                }
            }
        }
        let a = mesh_grid(&g, 4.0, [0.0; 3], true);
        let b = mesh_grid(&g, 4.0, [0.0; 3], false);
        assert_eq!(a.triangle_count(), 12);
        assert_eq!(b.triangle_count(), 6 * 16 * 2);
        assert_eq!(mesh_face_area(&a, 4.0), exposed_faces(&g));
        assert_eq!(mesh_face_area(&b, 4.0), exposed_faces(&g));
    }
}
