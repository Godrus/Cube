//! `CubeCharacter` (CHAR-01): copies the engine-neutral character asset into engine meshes.
//!
//! Translation only: the sculpt, the LODs and the meshing live in `cube-character`. GDScript builds
//! the rig (one `MeshInstance3D` per part) once from these meshes; animation moves nodes, never
//! geometry.

use std::time::Instant;

use cube_character::{CharacterAsset, DEFAULT_RES, PartId, palette};
use godot::classes::ArrayMesh;
use godot::classes::mesh::{ArrayType, PrimitiveType};
use godot::prelude::*;

/// The traveller character asset as seen by the scene.
#[derive(GodotClass)]
#[class(init, base = RefCounted)]
pub struct CubeCharacter {
    asset: Option<CharacterAsset>,
    build_ms: f64,
}

#[godot_api]
impl CubeCharacter {
    /// Builds the asset at `res` voxels per unit (0 = default). Returns `false` on a bad value.
    #[func]
    fn build(&mut self, res: i32) -> bool {
        let r = if res == 0 { DEFAULT_RES as i32 } else { res };
        if r < 8 || r % 4 != 0 || r > 128 {
            godot_error!("CubeCharacter.build: res must be a multiple of 4 in 8..=128, got {r}");
            return false;
        }
        let t = Instant::now();
        self.asset = Some(CharacterAsset::build(r as u32));
        self.build_ms = t.elapsed().as_secs_f64() * 1000.0;
        true
    }

    /// Number of rig parts (0 before `build`).
    #[func]
    fn part_count(&self) -> i32 {
        self.asset.as_ref().map_or(0, |a| a.parts.len() as i32)
    }

    /// Stable part name.
    #[func]
    fn part_name(&self, i: i32) -> GString {
        self.part(i).map(|p| p.id.name()).unwrap_or("").into()
    }

    /// Index of the parent part, -1 for the root.
    #[func]
    fn part_parent(&self, i: i32) -> i32 {
        self.part(i)
            .and_then(|p| p.id.parent())
            .and_then(|pp| PartId::ALL.iter().position(|&x| x == pp))
            .map_or(-1, |v| v as i32)
    }

    /// Joint position of the part in model space.
    #[func]
    fn part_pivot(&self, i: i32) -> Vector3 {
        self.part(i).map_or(Vector3::ZERO, |p| {
            let v = p.id.pivot();
            Vector3::new(v[0], v[1], v[2])
        })
    }

    /// Flat-shaded mesh of a part at a LOD (0, 1 or 2); vertex colours carry the material.
    #[func]
    fn part_mesh(&self, i: i32, lod: i32) -> Gd<ArrayMesh> {
        let mut mesh = ArrayMesh::new_gd();
        let Some(p) = self.part(i) else {
            return mesh;
        };
        let m = &p.lods[lod.clamp(0, 2) as usize].mesh;
        if m.vertex_count() == 0 {
            return mesh;
        }
        let pal = palette();
        let mut positions = PackedVector3Array::new();
        let mut normals = PackedVector3Array::new();
        let mut colours = PackedColorArray::new();
        let mut indices = PackedInt32Array::new();
        for v in 0..m.vertex_count() {
            positions.push(Vector3::new(
                m.positions[3 * v],
                m.positions[3 * v + 1],
                m.positions[3 * v + 2],
            ));
            normals.push(Vector3::new(
                m.normals[3 * v],
                m.normals[3 * v + 1],
                m.normals[3 * v + 2],
            ));
            let c = pal[(m.materials[v] as usize).min(pal.len() - 1)];
            colours.push(Color::from_rgb(c[0], c[1], c[2]));
        }
        // Godot treats clockwise triangles as front faces; the buffers are counter-clockwise.
        for t in m.indices.chunks_exact(3) {
            indices.push(t[0] as i32);
            indices.push(t[2] as i32);
            indices.push(t[1] as i32);
        }
        let mut arrays = VarArray::new();
        arrays.resize(ArrayType::MAX.ord() as usize, &Variant::nil());
        arrays.set(ArrayType::VERTEX.ord() as usize, &positions.to_variant());
        arrays.set(ArrayType::NORMAL.ord() as usize, &normals.to_variant());
        arrays.set(ArrayType::COLOR.ord() as usize, &colours.to_variant());
        arrays.set(ArrayType::INDEX.ord() as usize, &indices.to_variant());
        mesh.add_surface_from_arrays(PrimitiveType::TRIANGLES, &arrays);
        mesh
    }

    /// Asset statistics for one LOD: voxels, triangles, vertices, mesh and grid bytes, build ms,
    /// voxels per unit, asset id, content hash (as text) and the body height in units.
    #[func]
    fn stats(&self, lod: i32) -> VarDictionary {
        let mut d = VarDictionary::new();
        let Some(a) = &self.asset else {
            return d;
        };
        let l = lod.clamp(0, 2) as usize;
        d.set("res", a.parts[0].lods[l].res);
        d.set("voxels", a.voxel_count(l) as i64);
        d.set("triangles", a.triangle_count(l) as i64);
        d.set("vertices", a.vertex_count(l) as i64);
        d.set("mesh_bytes", a.mesh_bytes(l) as i64);
        d.set("grid_bytes", a.grid_bytes(l) as i64);
        d.set("build_ms", self.build_ms);
        d.set("asset_id", a.id().raw() as i64);
        d.set("content_hash", format!("{:016x}", a.content_hash()));
        d.set("height", a.bounds(l)[1][1]);
        d.set("parts", a.parts.len() as i64);
        d
    }
}

impl CubeCharacter {
    fn part(&self, i: i32) -> Option<&cube_character::Part> {
        self.asset.as_ref()?.parts.get(usize::try_from(i).ok()?)
    }
}
