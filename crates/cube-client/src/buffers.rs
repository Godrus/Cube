//! Mesh buffers for an engine (ADP preparation): plain arrays an adapter copies into engine
//! arrays. The client computes **no geometry**: the buffers are a flat re-layout of the faces
//! (or triangles) a provider already produced.
//!
//! Positions are relative to the brick origin (so they are small integers or halves, exact in
//! `f32`); the adapter places the brick node at `origin`. Faces become four vertices and two
//! triangles each (counter-clockwise seen from outside) with the face normal and the material
//! index per vertex; the baseline renderer is faces (GOV-13). A triangle mesh from the greedy
//! provider goes through [`MeshBuffers::from_trimesh`].

use cube_backend_voxel::{Dir, FacesMesh, TriMesh};

/// Flat buffers of one brick.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MeshBuffers {
    /// World position of the brick origin, in voxels.
    pub origin: [i64; 3],
    /// Vertex positions relative to `origin`, xyz.
    pub positions: Vec<f32>,
    /// Vertex normals, xyz.
    pub normals: Vec<f32>,
    /// Material index per vertex (0 is never emitted).
    pub materials: Vec<u16>,
    /// Triangle indices.
    pub indices: Vec<u32>,
}

impl MeshBuffers {
    /// Number of vertices.
    pub fn vertex_count(&self) -> usize {
        self.materials.len()
    }

    /// Number of triangles.
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Whether the buffers are consistent (lengths agree, indices in range).
    pub fn is_valid(&self) -> bool {
        let n = self.vertex_count();
        self.positions.len() == 3 * n
            && self.normals.len() == 3 * n
            && self.indices.len().is_multiple_of(3)
            && self.indices.iter().all(|i| (*i as usize) < n)
    }

    /// Buffers of a faces mesh.
    pub fn from_faces(mesh: &FacesMesh) -> Self {
        let b = mesh.key.brick();
        let o = b.origin().map_or([0, 0, 0], |p| [p.x, p.y, p.z]);
        let mut out = MeshBuffers {
            origin: o,
            ..Default::default()
        };
        for f in &mesh.faces {
            let local = [f.voxel.x - o[0], f.voxel.y - o[1], f.voxel.z - o[2]];
            let (n, quad) = face_quad(f.dir);
            let base = out.vertex_count() as u32;
            for c in quad {
                for a in 0..3 {
                    out.positions.push((local[a] + c[a]) as f32);
                    out.normals.push(n[a]);
                }
                out.materials.push(f.material);
            }
            out.indices
                .extend([base, base + 1, base + 2, base, base + 2, base + 3]);
        }
        out
    }

    /// Buffers of a triangle mesh (half-voxel positions, as produced by the greedy provider's
    /// triangulation). `origin` is the brick origin; the normal is the face direction.
    pub fn from_trimesh(mesh: &TriMesh, origin: [i64; 3]) -> Self {
        let mut out = MeshBuffers {
            origin,
            ..Default::default()
        };
        for (t, d, m) in &mesh.triangles {
            let (n, _) = face_quad(*d);
            for i in t {
                let p = mesh.vertices[*i as usize];
                for a in 0..3 {
                    out.positions.push((p[a] - 2 * origin[a]) as f32 / 2.0);
                    out.normals.push(n[a]);
                }
                out.materials.push(*m);
                out.indices.push(out.vertex_count() as u32 - 1);
            }
        }
        out
    }
}

/// Normal and the four corners (relative to the voxel minimum) of a face, counter-clockwise
/// seen from outside.
fn face_quad(d: Dir) -> ([f32; 3], [[i64; 3]; 4]) {
    match d {
        Dir::PosX => (
            [1.0, 0.0, 0.0],
            [[1, 0, 0], [1, 1, 0], [1, 1, 1], [1, 0, 1]],
        ),
        Dir::NegX => (
            [-1.0, 0.0, 0.0],
            [[0, 0, 0], [0, 0, 1], [0, 1, 1], [0, 1, 0]],
        ),
        Dir::PosY => (
            [0.0, 1.0, 0.0],
            [[0, 1, 0], [0, 1, 1], [1, 1, 1], [1, 1, 0]],
        ),
        Dir::NegY => (
            [0.0, -1.0, 0.0],
            [[0, 0, 0], [1, 0, 0], [1, 0, 1], [0, 0, 1]],
        ),
        Dir::PosZ => (
            [0.0, 0.0, 1.0],
            [[0, 0, 1], [1, 0, 1], [1, 1, 1], [0, 1, 1]],
        ),
        Dir::NegZ => (
            [0.0, 0.0, -1.0],
            [[0, 0, 0], [0, 1, 0], [1, 1, 0], [1, 0, 0]],
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cube_backend_voxel::Face;
    use cube_core::{CellKey, LayerId, VoxelPos};

    fn mesh(faces: Vec<Face>) -> FacesMesh {
        FacesMesh {
            key: CellKey::new(LayerId::new(1), 0, cube_core::BrickCoord::new(-1, 0, 2)).unwrap(),
            faces,
            unloaded_neighbours: false,
        }
    }

    #[test]
    fn every_face_becomes_four_vertices_and_two_outward_triangles() {
        let o = cube_core::BrickCoord::new(-1, 0, 2).origin().unwrap();
        let faces: Vec<Face> = Dir::ALL
            .iter()
            .map(|d| Face {
                voxel: VoxelPos::new(o.x + 3, o.y + 4, o.z + 5),
                dir: *d,
                material: 9,
            })
            .collect();
        let b = MeshBuffers::from_faces(&mesh(faces));
        assert!(b.is_valid());
        assert_eq!((b.vertex_count(), b.triangle_count()), (24, 12));
        assert_eq!(b.origin, [o.x, o.y, o.z]);
        assert!(b.materials.iter().all(|m| *m == 9));
        // Relative positions are small and exact; every triangle winds counter-clockwise
        // around its normal.
        assert!(b.positions.iter().all(|p| (0.0..=16.0).contains(p)));
        for t in b.indices.chunks(3) {
            let p = |i: u32| -> [f32; 3] {
                let k = 3 * i as usize;
                [b.positions[k], b.positions[k + 1], b.positions[k + 2]]
            };
            let n = |i: u32| -> [f32; 3] {
                let k = 3 * i as usize;
                [b.normals[k], b.normals[k + 1], b.normals[k + 2]]
            };
            let (a, bb, c) = (p(t[0]), p(t[1]), p(t[2]));
            let u = [bb[0] - a[0], bb[1] - a[1], bb[2] - a[2]];
            let w = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let cross = [
                u[1] * w[2] - u[2] * w[1],
                u[2] * w[0] - u[0] * w[2],
                u[0] * w[1] - u[1] * w[0],
            ];
            let nn = n(t[0]);
            let dot = cross[0] * nn[0] + cross[1] * nn[1] + cross[2] * nn[2];
            assert!(dot > 0.0, "triangle winds against its normal: {t:?}");
        }
    }

    #[test]
    fn an_empty_mesh_gives_empty_valid_buffers() {
        let b = MeshBuffers::from_faces(&mesh(vec![]));
        assert!(b.is_valid() && b.vertex_count() == 0);
    }
}
