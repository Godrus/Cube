//! GDExtension adapter (experimental spike).
//!
//! Thin layer: it converts Godot arguments to plain values, calls `bridge::RuntimeBridge`
//! (pure Rust), and hands the results back as Godot arrays. It stores no geometry truth of its
//! own and exposes no Godot type to any other Cube crate.

pub mod bridge;

use std::path::Path;

use godot::prelude::*;

use bridge::{BridgeError, ChangeBatch, RuntimeBridge};

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
