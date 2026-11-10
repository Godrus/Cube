//! Thin Godot 4 adapter (ADP-01..03): translation only.
//!
//! * engine input -> [`cube_client::Controls`] (done by the caller of [`CubeWorld::frame`]);
//! * [`cube_client::MeshBuffers`] -> engine arrays and one `MeshInstance3D` per brick;
//! * the session pose -> numbers the scene's camera copies.
//!
//! No world state, no generator, no editing, persistence or history logic, no commit and no child
//! process: the world lives behind a [`SessionPort`] that the composition root (`cube-app`) opens.
//! Failures never cross the FFI boundary as panics: every entry point reports `false`, `-1` or an
//! empty result and logs the reason with `godot_error!`.

use std::collections::HashMap;

use cube_client::{CellKeyExt as _, Controls, FrameOut, SessionPort};
use godot::classes::base_material_3d::Flags;
use godot::classes::mesh::{ArrayType, PrimitiveType};
use godot::classes::{ArrayMesh, INode3D, MeshInstance3D, Node3D, StandardMaterial3D};
use godot::prelude::*;

fn make_session(dir: &str, seed: u64, create: bool) -> Result<Box<dyn SessionPort>, String> {
    cube_app::port::open_first_world(std::path::Path::new(dir), seed, create)
}

/// Vertex colour of a material index (placeholder palette, no textures).
pub fn material_colour(m: u16) -> [f32; 3] {
    match m {
        1 => [0.45, 0.45, 0.48],
        2 => [0.45, 0.30, 0.18],
        3 => [0.25, 0.60, 0.20],
        4 => [0.85, 0.80, 0.55],
        5 => [0.80, 0.25, 0.20],
        _ => [0.80, 0.20, 0.80],
    }
}

/// The node a scene adds to show and edit a world.
#[derive(GodotClass)]
#[class(base = Node3D)]
pub struct CubeWorld {
    base: Base<Node3D>,
    session: Option<Box<dyn SessionPort>>,
    bricks: HashMap<[i64; 3], Gd<MeshInstance3D>>,
    material: Option<Gd<StandardMaterial3D>>,
}

#[godot_api]
impl INode3D for CubeWorld {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            base,
            session: None,
            bricks: HashMap::new(),
            material: None,
        }
    }
}

#[godot_api]
impl CubeWorld {
    /// Opens (or creates) the world in `dir`. Returns `false` and logs the reason on failure.
    #[func]
    fn open_world(&mut self, dir: GString, seed: i64, create: bool) -> bool {
        self.close();
        match make_session(&dir.to_string(), seed as u64, create) {
            Ok(s) => {
                self.session = Some(s);
                let mut m = StandardMaterial3D::new_gd();
                m.set_flag(Flags::ALBEDO_FROM_VERTEX_COLOR, true);
                self.material = Some(m);
                true
            }
            Err(e) => {
                godot_error!("CubeWorld.open_world: {e}");
                false
            }
        }
    }

    /// Runs one frame. Directions are -1, 0 or 1; `look_*` are radians; returns the number of
    /// bricks changed in the scene, or -1 if no world is open or the frame failed.
    #[func]
    fn frame(
        &mut self,
        forward: i32,
        right: i32,
        up: i32,
        look_yaw: f32,
        look_pitch: f32,
        dt: f32,
    ) -> i32 {
        let controls = Controls {
            forward: forward.clamp(-1, 1) as i8,
            right: right.clamp(-1, 1) as i8,
            up: up.clamp(-1, 1) as i8,
            look_yaw,
            look_pitch,
            dt,
        };
        let Some(s) = self.session.as_mut() else {
            return -1;
        };
        match s.frame(&controls) {
            Ok(out) => self.apply(out),
            Err(e) => {
                godot_error!("CubeWorld.frame: {e}");
                -1
            }
        }
    }

    /// Removes (`material == 0`) or places a block of `material` at the crosshair.
    #[func]
    fn edit(&mut self, material: i32) -> bool {
        let Some(s) = self.session.as_mut() else {
            return false;
        };
        let place = (material > 0).then_some(material.min(i32::from(u16::MAX)) as u16);
        match s.edit(place) {
            Ok(hit) => hit,
            Err(e) => {
                godot_error!("CubeWorld.edit: {e}");
                false
            }
        }
    }

    /// Camera position in voxel units.
    #[func]
    fn camera_position(&self) -> Vector3 {
        self.session.as_ref().map_or(Vector3::ZERO, |s| {
            let p = s.pose().pos;
            Vector3::new(p[0] as f32, p[1] as f32, p[2] as f32)
        })
    }

    /// Camera rotation as (pitch, yaw, 0) in radians, for `Node3D.rotation`.
    #[func]
    fn camera_rotation(&self) -> Vector3 {
        self.session.as_ref().map_or(Vector3::ZERO, |s| {
            let p = s.pose();
            Vector3::new(p.pitch, p.yaw, 0.0)
        })
    }

    /// Switches between walking (gravity, jump, collision) and free flight.
    #[func]
    fn set_walk(&mut self, on: bool) {
        if let Some(s) = self.session.as_mut() {
            s.set_walk(on);
        }
    }

    /// Chooses the third-person camera (only while walking).
    #[func]
    fn set_third_person(&mut self, on: bool) {
        if let Some(s) = self.session.as_mut() {
            s.set_third_person(on);
        }
    }

    /// `true` while the player walks.
    #[func]
    fn is_walking(&self) -> bool {
        self.session.as_ref().is_some_and(|s| s.walking())
    }

    /// Standing on something (walking only).
    #[func]
    fn is_grounded(&self) -> bool {
        self.session.as_ref().is_some_and(|s| s.grounded())
    }

    /// Feet position of the walking player (zero when flying).
    #[func]
    fn player_feet(&self) -> Vector3 {
        self.session
            .as_ref()
            .and_then(|s| s.player_feet())
            .map_or(Vector3::ZERO, |p| {
                Vector3::new(p[0] as f32, p[1] as f32, p[2] as f32)
            })
    }

    /// Number of bricks with geometry currently in the scene.
    #[func]
    fn brick_count(&self) -> i32 {
        self.bricks.len() as i32
    }

    /// Stops the world and frees every brick node.
    #[func]
    fn close(&mut self) {
        if let Some(mut s) = self.session.take() {
            s.shutdown();
        }
        for (_, mut n) in self.bricks.drain() {
            n.queue_free();
        }
    }
}

impl CubeWorld {
    fn apply(&mut self, out: FrameOut) -> i32 {
        let mut changed = 0;
        for k in out.remove {
            if let Some(mut n) = self.bricks.remove(&k.coords()) {
                n.queue_free();
                changed += 1;
            }
        }
        for (k, b) in out.upsert {
            let key = k.coords();
            if let Some(mut old) = self.bricks.remove(&key) {
                old.queue_free();
            }
            changed += 1;
            if b.vertex_count() == 0 {
                continue;
            }
            let mut positions = PackedVector3Array::new();
            let mut normals = PackedVector3Array::new();
            let mut colours = PackedColorArray::new();
            let mut indices = PackedInt32Array::new();
            for i in 0..b.vertex_count() {
                positions.push(Vector3::new(
                    b.positions[3 * i],
                    b.positions[3 * i + 1],
                    b.positions[3 * i + 2],
                ));
                normals.push(Vector3::new(
                    b.normals[3 * i],
                    b.normals[3 * i + 1],
                    b.normals[3 * i + 2],
                ));
                let c = material_colour(b.materials[i]);
                colours.push(Color::from_rgb(c[0], c[1], c[2]));
            }
            // Godot treats clockwise triangles as front faces; the buffers are counter-clockwise
            // seen from outside, so every triangle is swapped here (engine convention only).
            for t in b.indices.chunks_exact(3) {
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
            let mut mesh = ArrayMesh::new_gd();
            mesh.add_surface_from_arrays(PrimitiveType::TRIANGLES, &arrays);
            let mut node = MeshInstance3D::new_alloc();
            node.set_mesh(&mesh);
            if let Some(m) = &self.material {
                node.set_material_override(m);
            }
            node.set_position(Vector3::new(
                b.origin[0] as f32,
                b.origin[1] as f32,
                b.origin[2] as f32,
            ));
            self.base_mut().add_child(&node);
            self.bricks.insert(key, node);
        }
        changed
    }
}

/// The library entry point Godot loads (see `godot/cube.gdextension`). This is the only `unsafe`
/// in the project and it is the one line the binding requires.
struct CubeExtension;

#[gdextension]
unsafe impl ExtensionLibrary for CubeExtension {}
