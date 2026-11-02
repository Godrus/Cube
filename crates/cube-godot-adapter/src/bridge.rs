//! Pure-Rust bridge: everything the Godot adapter does, without any Godot type.
//!
//! This is the "Client Adapter Interface" of the spike: plain data in, plain data out
//! (`ChangeBatch`). The adapter owns no geometry truth beyond the `Editor` it wraps, which is
//! the same Rust runtime a headless server would use. A future Rust-native client adapter can
//! use this module unchanged.

use std::path::Path;

use cube_core::{AssetId, GeometryAsset, LayerId, Region, VoxelPos};
use cube_editing::{EditOp, EditOutcome, Editor};
use cube_processing::ProcessingSnapshot;

/// Display limit for one batch (occupied cells). Keeps a single call bounded for the client.
pub const MAX_BATCH_CELLS: usize = 262_144;
/// Largest fill the adapter accepts (client display limit, stricter than the editing limit).
pub const MAX_FILL_VOLUME: u64 = 1 << 16;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BridgeError {
    pub code: String,
    pub message: String,
}

impl BridgeError {
    fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.to_string(),
            message: message.into(),
        }
    }
}

impl std::fmt::Display for BridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.code, self.message)
    }
}

/// What the renderer needs after a change.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ChangeBatch {
    pub epoch: u32,
    pub asset_revision: u64,
    pub layer_revision: u64,
    /// True: replace the whole client mirror (load). False: clear each `dirty` region, then add `cells`.
    pub reset: bool,
    /// `[min_x, min_y, min_z, max_x, max_y, max_z]` (max exclusive).
    pub dirty: Vec<[i32; 6]>,
    /// Flat `x, y, z, material` for occupied voxels inside the dirty regions.
    pub cells: Vec<i32>,
    pub changed_voxels: u64,
}

pub struct RuntimeBridge {
    editor: Editor,
    layer: LayerId,
}

impl Default for RuntimeBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeBridge {
    pub fn new() -> Self {
        let mut editor = Editor::new(GeometryAsset::new(AssetId(1)));
        let layer = editor.add_voxel_layer(0.1);
        Self { editor, layer }
    }

    pub fn can_undo(&self) -> bool {
        self.editor.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.editor.can_redo()
    }

    pub fn fill_region(&mut self, r: [i32; 6], material: u16) -> Result<ChangeBatch, BridgeError> {
        let region = to_region(r);
        if region.volume() > MAX_FILL_VOLUME {
            return Err(BridgeError::new(
                "ADPT-0001",
                "fill region exceeds the client display limit",
            ));
        }
        let out = self
            .editor
            .execute(EditOp::FillRegion {
                layer: self.layer,
                region,
                material,
            })
            .map_err(|e| BridgeError::new(e.code(), e.to_string()))?;
        self.batch(out)
    }

    pub fn undo(&mut self) -> Result<ChangeBatch, BridgeError> {
        let out = self
            .editor
            .undo()
            .map_err(|e| BridgeError::new(e.code(), e.to_string()))?;
        self.batch(out)
    }

    pub fn redo(&mut self) -> Result<ChangeBatch, BridgeError> {
        let out = self
            .editor
            .redo()
            .map_err(|e| BridgeError::new(e.code(), e.to_string()))?;
        self.batch(out)
    }

    pub fn save(&self, path: &Path) -> Result<(), BridgeError> {
        cube_persistence::save_asset(path, self.editor.asset())
            .map_err(|e| BridgeError::new(e.code(), e.to_string()))
    }

    /// Loads into a temporary asset first; the live state is replaced only if everything works
    /// and the result fits the display limit. History is dropped on success.
    pub fn load(&mut self, path: &Path) -> Result<ChangeBatch, BridgeError> {
        let asset = cube_persistence::load_asset(path)
            .map_err(|e| BridgeError::new(e.code(), e.to_string()))?;
        let layer = asset
            .layers()
            .first()
            .map(|l| l.id())
            .ok_or_else(|| BridgeError::new("ADPT-0002", "loaded asset has no layer"))?;
        let mut cells = Vec::new();
        let mut dirty = Vec::new();
        if let Some(bounds) = layer_bounds(&asset, layer) {
            let snap = ProcessingSnapshot::capture(&asset, layer, bounds)
                .map_err(|e| BridgeError::new(e.code(), e.to_string()))?;
            flatten(&snap, &mut cells);
            dirty.push(from_region(bounds));
        }
        if cells.len() / 4 > MAX_BATCH_CELLS {
            return Err(BridgeError::new(
                "ADPT-0003",
                "loaded asset exceeds the client display limit",
            ));
        }
        let batch = ChangeBatch {
            epoch: asset.epoch(),
            asset_revision: asset.asset_token().value,
            layer_revision: asset.layer_token(layer).map(|t| t.value).unwrap_or(0),
            reset: true,
            dirty,
            cells,
            changed_voxels: 0,
        };
        self.layer = layer;
        self.editor.replace_asset(asset);
        Ok(batch)
    }

    fn batch(&self, out: EditOutcome) -> Result<ChangeBatch, BridgeError> {
        let mut cells = Vec::new();
        let mut dirty = Vec::new();
        for d in &out.dirty {
            let snap = ProcessingSnapshot::capture(self.editor.asset(), d.layer, d.region)
                .map_err(|e| BridgeError::new(e.code(), e.to_string()))?;
            flatten(&snap, &mut cells);
            dirty.push(from_region(d.region));
        }
        if cells.len() / 4 > MAX_BATCH_CELLS {
            return Err(BridgeError::new(
                "ADPT-0003",
                "change exceeds the client display limit",
            ));
        }
        Ok(ChangeBatch {
            epoch: out.asset_token.epoch,
            asset_revision: out.asset_token.value,
            layer_revision: out.layer_token.value,
            reset: false,
            dirty,
            cells,
            changed_voxels: out.changed_voxels,
        })
    }
}

fn flatten(snap: &ProcessingSnapshot, out: &mut Vec<i32>) {
    for c in snap.voxel_cells() {
        out.extend_from_slice(&[c.pos.x, c.pos.y, c.pos.z, c.material as i32]);
    }
}

fn to_region(r: [i32; 6]) -> Region {
    Region::new(
        VoxelPos::new(r[0], r[1], r[2]),
        VoxelPos::new(r[3], r[4], r[5]),
    )
}

fn from_region(r: Region) -> [i32; 6] {
    [r.min.x, r.min.y, r.min.z, r.max.x, r.max.y, r.max.z]
}

fn layer_bounds(asset: &GeometryAsset, layer: LayerId) -> Option<Region> {
    let l = asset.layer(layer).ok()?;
    l.storage()
        .bricks_sorted()
        .iter()
        .map(|(c, _)| Region::of_brick(*c))
        .reduce(|a, b| a.union_bounds(&b))
}
