//! GDExtension adapter (experimental spike).
//!
//! Thin layer: it converts Godot arguments to plain values, calls `bridge::RuntimeBridge`
//! (pure Rust), and hands the results back as Godot arrays. It stores no geometry truth of its
//! own and exposes no Godot type to any other Cube crate.

pub mod bridge;
pub mod world_bridge;

use std::path::Path;

use godot::prelude::*;

use bridge::{BridgeError, ChangeBatch, RuntimeBridge};
use world_bridge::WorldBridge;

struct CubeExtension;

#[gdextension]
unsafe impl ExtensionLibrary for CubeExtension {}

/// Godot-facing handle to the Rust runtime. Results of the last call are read through getters.
#[derive(GodotClass)]
#[class(base=RefCounted)]
pub struct CubeRuntime {
    bridge: RuntimeBridge,
    last: ChangeBatch,
    last_error: String,
    base: Base<RefCounted>,
}

#[godot_api]
impl IRefCounted for CubeRuntime {
    fn init(base: Base<RefCounted>) -> Self {
        Self {
            bridge: RuntimeBridge::new(),
            last: ChangeBatch::default(),
            last_error: String::new(),
            base,
        }
    }
}

impl CubeRuntime {
    fn finish(&mut self, r: Result<ChangeBatch, BridgeError>) -> bool {
        match r {
            Ok(b) => {
                self.last = b;
                self.last_error.clear();
                true
            }
            Err(e) => {
                self.last_error = e.to_string();
                false
            }
        }
    }
}

#[godot_api]
impl CubeRuntime {
    #[func]
    #[allow(clippy::too_many_arguments)]
    fn fill_region(
        &mut self,
        x0: i32,
        y0: i32,
        z0: i32,
        x1: i32,
        y1: i32,
        z1: i32,
        material: i32,
    ) -> bool {
        let Ok(m) = u16::try_from(material) else {
            self.last_error = "ADPT-0004 material out of range".to_string();
            return false;
        };
        let r = self.bridge.fill_region([x0, y0, z0, x1, y1, z1], m);
        self.finish(r)
    }

    #[func]
    fn undo(&mut self) -> bool {
        let r = self.bridge.undo();
        self.finish(r)
    }

    #[func]
    fn redo(&mut self) -> bool {
        let r = self.bridge.redo();
        self.finish(r)
    }

    #[func]
    fn save(&mut self, path: GString) -> bool {
        match self.bridge.save(Path::new(&path.to_string())) {
            Ok(()) => {
                self.last_error.clear();
                true
            }
            Err(e) => {
                self.last_error = e.to_string();
                false
            }
        }
    }

    #[func]
    fn load(&mut self, path: GString) -> bool {
        let r = self.bridge.load(Path::new(&path.to_string()));
        self.finish(r)
    }

    #[func]
    fn can_undo(&self) -> bool {
        self.bridge.can_undo()
    }

    #[func]
    fn can_redo(&self) -> bool {
        self.bridge.can_redo()
    }

    #[func]
    fn last_error(&self) -> GString {
        GString::from(self.last_error.as_str())
    }

    /// Flat `x, y, z, material` of occupied voxels in the dirty regions of the last change.
    #[func]
    fn last_cells(&self) -> PackedInt32Array {
        PackedInt32Array::from(self.last.cells.as_slice())
    }

    /// Flat `min_x, min_y, min_z, max_x, max_y, max_z` per dirty region (max exclusive).
    #[func]
    fn last_dirty(&self) -> PackedInt32Array {
        let flat: Vec<i32> = self.last.dirty.iter().flatten().copied().collect();
        PackedInt32Array::from(flat.as_slice())
    }

    #[func]
    fn last_reset(&self) -> bool {
        self.last.reset
    }

    #[func]
    fn asset_revision(&self) -> i64 {
        self.last.asset_revision as i64
    }

    #[func]
    fn layer_revision(&self) -> i64 {
        self.last.layer_revision as i64
    }

    #[func]
    fn epoch(&self) -> i64 {
        self.last.epoch as i64
    }
}

fn flat2(v: Vec<[i32; 2]>) -> PackedInt32Array {
    let flat: Vec<i32> = v.into_iter().flatten().collect();
    PackedInt32Array::from(flat.as_slice())
}

/// Godot-facing handle to the experimental world generator (see `world_bridge`).
#[derive(GodotClass)]
#[class(base=RefCounted)]
pub struct CubeWorld {
    bridge: WorldBridge,
    last_error: String,
    base: Base<RefCounted>,
}

#[godot_api]
impl IRefCounted for CubeWorld {
    fn init(base: Base<RefCounted>) -> Self {
        Self {
            bridge: WorldBridge::new(0, 1),
            last_error: String::new(),
            base,
        }
    }
}

#[godot_api]
impl CubeWorld {
    /// Replaces the world with a new one for `seed` (loaded radius in chunks, clamped to 0..=8).
    #[func]
    fn create(&mut self, seed: i64, radius: i32) {
        self.bridge = WorldBridge::new(seed as u64, radius);
        self.last_error.clear();
    }

    #[func]
    fn seed(&self) -> i64 {
        self.bridge.seed() as i64
    }

    #[func]
    fn radius(&self) -> i32 {
        self.bridge.radius()
    }

    /// Moves the streaming centre; returns how many chunks were queued.
    #[func]
    fn set_player(&mut self, x: f32, y: f32, z: f32) -> i32 {
        self.bridge.set_player(x, y, z).queued as i32
    }

    /// Generates up to `budget` queued chunks; returns how many became ready.
    #[func]
    fn pump(&mut self, budget: i32) -> i32 {
        self.bridge.pump(budget.max(0) as usize).generated as i32
    }

    #[func]
    fn pump_all(&mut self) -> i32 {
        self.bridge.pump_all().generated as i32
    }

    #[func]
    fn queue_len(&self) -> i32 {
        self.bridge.queue_len() as i32
    }

    /// Flat `x, z` pairs of chunks that became ready since the last call.
    #[func]
    fn take_added(&mut self) -> PackedInt32Array {
        flat2(self.bridge.take_added())
    }

    /// Flat `x, z` pairs of chunks unloaded since the last call.
    #[func]
    fn take_removed(&mut self) -> PackedInt32Array {
        flat2(self.bridge.take_removed())
    }

    /// Flat `x, z` pairs of chunks edited since the last call.
    #[func]
    fn take_modified(&mut self) -> PackedInt32Array {
        flat2(self.bridge.take_modified())
    }

    /// Flat `x, z` pairs of all ready chunks.
    #[func]
    fn active_chunks(&self) -> PackedInt32Array {
        flat2(self.bridge.active_chunks())
    }

    #[func]
    fn active_count(&self) -> i32 {
        self.bridge.active_count() as i32
    }

    #[func]
    fn active_voxels(&self) -> i32 {
        self.bridge.active_voxels() as i32
    }

    #[func]
    fn chunk_voxel_count(&self, cx: i32, cz: i32) -> i32 {
        self.bridge
            .chunk_voxel_count(cx, cz)
            .map_or(-1, |n| n as i32)
    }

    /// Flat `x, y, z, material` of a ready chunk (empty if the chunk is not ready).
    #[func]
    fn chunk_voxels(&self, cx: i32, cz: i32) -> PackedInt32Array {
        let v = self.bridge.chunk_voxels(cx, cz).unwrap_or_default();
        PackedInt32Array::from(v.as_slice())
    }

    /// Ready-made `MultiMesh.buffer` (16 floats per cube) of a ready chunk; empty if not ready.
    #[func]
    fn chunk_instance_buffer(&self, cx: i32, cz: i32) -> PackedFloat32Array {
        let v = self
            .bridge
            .chunk_instance_buffer(cx, cz)
            .unwrap_or_default();
        PackedFloat32Array::from(v.as_slice())
    }

    /// Experimental face-culled renderer: an `ArrayMesh` with the visible faces of a ready chunk (one
    /// surface, vertex colours). `null` if the chunk is not ready or has no visible face.
    #[func]
    fn chunk_mesh(&self, cx: i32, cz: i32) -> Option<Gd<godot::classes::ArrayMesh>> {
        use godot::classes::mesh::{ArrayType, PrimitiveType};
        use godot::classes::ArrayMesh;
        let m = self.bridge.chunk_mesh(cx, cz)?;
        if m.faces == 0 {
            return None;
        }
        let vec3 = |f: &[f32]| -> PackedVector3Array {
            f.chunks_exact(3)
                .map(|c| Vector3::new(c[0], c[1], c[2]))
                .collect()
        };
        let colors: PackedColorArray = m
            .colors
            .chunks_exact(4)
            .map(|c| Color::from_rgba(c[0], c[1], c[2], c[3]))
            .collect();
        let indices: PackedInt32Array = m.indices.iter().map(|&i| i as i32).collect();
        let mut arrays = VarArray::new();
        arrays.resize(ArrayType::MAX.ord() as usize, &Variant::nil());
        arrays.set(
            ArrayType::VERTEX.ord() as usize,
            &vec3(&m.positions).to_variant(),
        );
        arrays.set(
            ArrayType::NORMAL.ord() as usize,
            &vec3(&m.normals).to_variant(),
        );
        arrays.set(ArrayType::COLOR.ord() as usize, &colors.to_variant());
        arrays.set(ArrayType::INDEX.ord() as usize, &indices.to_variant());
        let mut mesh = ArrayMesh::new_gd();
        mesh.add_surface_from_arrays(PrimitiveType::TRIANGLES, &arrays);
        Some(mesh)
    }

    /// Number of visible faces of a ready chunk (-1 if not ready). Cheap check for the HUD and tests.
    #[func]
    fn chunk_face_count(&self, cx: i32, cz: i32) -> i32 {
        self.bridge
            .chunk_mesh(cx, cz)
            .map_or(-1, |m| m.faces as i32)
    }

    #[func]
    fn ground_height(&self, x: i32, z: i32) -> i32 {
        self.bridge.ground_height(x, z)
    }

    /// Chunks next to an edited border that need a face-mesh rebuild (see `take_modified` for the edited ones).
    #[func]
    fn take_border_touched(&mut self) -> PackedInt32Array {
        self.bridge
            .take_border_touched()
            .into_iter()
            .flatten()
            .collect()
    }

    /// Sets one voxel to a material (4 = placed block); true if it changed.
    #[func]
    fn edit_voxel(&mut self, x: i32, y: i32, z: i32, material: i32) -> bool {
        self.bridge.edit_voxel(x, y, z, material as u16)
    }

    /// Places a 3x3x3 block around the position; returns changed voxels.
    #[func]
    fn edit_around(&mut self, x: i32, y: i32, z: i32) -> i32 {
        self.bridge.edit_around(x, y, z) as i32
    }

    #[func]
    fn edit_count(&self) -> i32 {
        self.bridge.edit_count() as i32
    }

    #[func]
    fn save(&mut self, path: GString) -> bool {
        match self.bridge.save(Path::new(&path.to_string())) {
            Ok(()) => {
                self.last_error.clear();
                true
            }
            Err(e) => {
                self.last_error = e;
                false
            }
        }
    }

    #[func]
    fn load(&mut self, path: GString) -> bool {
        match self.bridge.load(Path::new(&path.to_string())) {
            Ok(s) if s.missing == 0 => {
                self.last_error.clear();
                true
            }
            Ok(s) => {
                self.last_error =
                    format!("WORLD-LOAD {} saved chunks did not come back", s.missing);
                false
            }
            Err(e) => {
                self.last_error = e;
                false
            }
        }
    }

    #[func]
    fn last_error(&self) -> GString {
        GString::from(self.last_error.as_str())
    }
}
