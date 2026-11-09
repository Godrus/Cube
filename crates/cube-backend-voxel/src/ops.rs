//! The voxel backend (VOX-01): set, replace, fill, clear and copy/move of regions as write sets.
//!
//! It implements `GeometryBackend` and nothing else: it reads through the overlay, returns
//! `WriteSet`s and never commits (Orchestration stages and commits them, with expected revisions
//! of its own). It is deterministic: the same operation on the same overlay gives identical
//! write sets, bricks ascending. All coordinates are `i64` voxels at the region's level (so
//! finer micro-voxel levels use the same code); negative coordinates follow Core's floor
//! division. There is no world-size limit: an operation is bounded by [`Limits`] (memory), and
//! a larger one is refused, not truncated.
//!
//! Current scope: voxel material channel; other channels are preserved untouched. Selective
//! micro-voxel detailing (promoting part of a brick to a finer level) is a later step: the
//! interfaces are level-aware, so it is additive.

use std::sync::Arc;

use cube_core::{LayerKind, Region, VoxelPos, WriteSet};
use cube_editing_api::{
    BackendCapabilities, EditingError, EditingOperation, GeometryBackend, OverlayRead,
    ValidationReport, kinds,
};
use cube_processing::Snapshot;

use crate::edit::{Editor, Limits};
use crate::params::{TransformMode, VoxelOp, decode};

/// The voxel backend.
#[derive(Debug)]
pub struct VoxelBackend {
    caps: BackendCapabilities,
    limits: Limits,
}

impl VoxelBackend {
    /// A backend with default limits.
    pub fn new() -> Self {
        Self::with_limits(Limits::default())
    }

    /// A backend with explicit limits.
    pub fn with_limits(limits: Limits) -> Self {
        Self {
            caps: BackendCapabilities {
                layer_kind: LayerKind::Voxel,
                kinds: [
                    kinds::SET_MATERIAL,
                    kinds::ADD_GEOMETRY,
                    kinds::REMOVE_GEOMETRY,
                    kinds::TRANSFORM_GEOMETRY,
                ]
                .into_iter()
                .collect(),
                native_boolean: false,
                resample: false,
                validate: false,
            },
            limits,
        }
    }

    /// As a shareable port.
    pub fn arc() -> Arc<dyn GeometryBackend> {
        Arc::new(Self::new())
    }

    fn check_size(&self, r: &Region) -> Result<(), EditingError> {
        match r.brick_count() {
            Some(n) if n <= u128::from(self.limits.max_bricks) => Ok(()),
            _ => Err(EditingError::InvalidOperation("region too large")),
        }
    }
}

impl Default for VoxelBackend {
    fn default() -> Self {
        Self::new()
    }
}

fn box_of(r: &Region) -> (VoxelPos, VoxelPos) {
    (r.min(), r.max())
}

/// Rotated, translated destination of the source box: `(min, max)` exclusive, and the mapping of
/// a source cell offset to a destination cell.
struct Mapping {
    rotation: cube_core::Rotation,
    bmin: [i64; 3],
    dest: VoxelPos,
    extent: [i64; 3],
}

impl Mapping {
    fn new(
        src: &Region,
        dest: VoxelPos,
        rotation: cube_core::Rotation,
    ) -> Result<Self, EditingError> {
        let overflow =
            |_| EditingError::InvalidOperation("transform overflows the coordinate range");
        let e = src.extent();
        let ext = [
            i64::try_from(e[0]).map_err(overflow)?,
            i64::try_from(e[1]).map_err(overflow)?,
            i64::try_from(e[2]).map_err(overflow)?,
        ];
        let a = rotation
            .apply([0, 0, 0])
            .map_err(|_| EditingError::InvalidOperation("rotation overflow"))?;
        let b = rotation
            .apply(ext)
            .map_err(|_| EditingError::InvalidOperation("rotation overflow"))?;
        let bmin = [a[0].min(b[0]), a[1].min(b[1]), a[2].min(b[2])];
        let bmax = [a[0].max(b[0]), a[1].max(b[1]), a[2].max(b[2])];
        Ok(Self {
            rotation,
            bmin,
            dest,
            extent: [bmax[0] - bmin[0], bmax[1] - bmin[1], bmax[2] - bmin[2]],
        })
    }

    /// Destination box `[min, max)`.
    fn dest_box(&self) -> Result<(VoxelPos, VoxelPos), EditingError> {
        let add = |a: i64, b: i64| {
            a.checked_add(b).ok_or(EditingError::InvalidOperation(
                "destination leaves the coordinate range",
            ))
        };
        let min = self.dest;
        let max = VoxelPos::new(
            add(min.x, self.extent[0])?,
            add(min.y, self.extent[1])?,
            add(min.z, self.extent[2])?,
        );
        Ok((min, max))
    }

    /// Destination cell of the source cell at offset `i` from the source minimum.
    fn cell(&self, i: [i64; 3]) -> Result<VoxelPos, EditingError> {
        let rot = |v: [i64; 3]| {
            self.rotation
                .apply(v)
                .map_err(|_| EditingError::InvalidOperation("rotation overflow"))
        };
        let a = rot(i)?;
        let b = rot([i[0] + 1, i[1] + 1, i[2] + 1])?;
        let m = [a[0].min(b[0]), a[1].min(b[1]), a[2].min(b[2])];
        Ok(VoxelPos::new(
            self.dest.x + (m[0] - self.bmin[0]),
            self.dest.y + (m[1] - self.bmin[1]),
            self.dest.z + (m[2] - self.bmin[2]),
        ))
    }
}

fn covers(outer: &Region, min: VoxelPos, max: VoxelPos) -> bool {
    outer.min().x <= min.x
        && outer.min().y <= min.y
        && outer.min().z <= min.z
        && outer.max().x >= max.x
        && outer.max().y >= max.y
        && outer.max().z >= max.z
}

impl GeometryBackend for VoxelBackend {
    fn capabilities(&self) -> &BackendCapabilities {
        &self.caps
    }

    fn apply(
        &self,
        o: &EditingOperation,
        overlay: &dyn OverlayRead,
        _snapshot: Option<&Snapshot>,
    ) -> Result<Vec<WriteSet>, EditingError> {
        if !self.caps.supports(o.kind) {
            return Err(EditingError::UnsupportedGeometryOperation(o.kind));
        }
        let op = decode(o.kind, &o.parameters)?;
        let region = o.write_region;
        self.check_size(&region)?;
        let level = region.level();
        let mut sets = Vec::new();
        for layer in &o.layers {
            let mut ed = Editor::new(overlay, *layer, level);
            let (min, max) = box_of(&region);
            match op {
                VoxelOp::Set {
                    material,
                    only_where,
                } => match only_where {
                    None => ed.apply_box(min, max, false, &|_| Some(material))?,
                    Some(w) => ed.apply_box(min, max, true, &|c| (c == w).then_some(material))?,
                },
                VoxelOp::Add { material } => {
                    ed.apply_box(min, max, true, &|c| (c == 0).then_some(material))?;
                }
                VoxelOp::Remove { only_where } => match only_where {
                    None => ed.apply_box(min, max, false, &|_| Some(0))?,
                    Some(w) => ed.apply_box(min, max, true, &|c| (c == w).then_some(0))?,
                },
                VoxelOp::Transform {
                    dest,
                    rotation,
                    mode,
                } => {
                    let src = o.read_region.ok_or(EditingError::InvalidOperation(
                        "transform needs a read region",
                    ))?;
                    if src.level() != level {
                        return Err(EditingError::InvalidResolution);
                    }
                    self.check_size(&src)?;
                    let m = Mapping::new(&src, dest, rotation)?;
                    let (dmin, dmax) = m.dest_box()?;
                    let dest_region = Region::new(level, dmin, dmax)
                        .map_err(|_| EditingError::InvalidOperation("bad destination"))?;
                    self.check_size(&dest_region)?;
                    if !covers(&region, dmin, dmax)
                        || (mode == TransformMode::Move && !covers(&region, src.min(), src.max()))
                    {
                        return Err(EditingError::InvalidOperation(
                            "write region does not cover the destination",
                        ));
                    }
                    let e = src.extent();
                    let voxels = u128::from(e[0]) * u128::from(e[1]) * u128::from(e[2]);
                    if voxels > u128::from(self.limits.max_bricks) * 4096 {
                        return Err(EditingError::InvalidOperation("region too large"));
                    }
                    // Read the whole source first: overlapping copies must see the old state.
                    let mut cells = Vec::with_capacity(voxels as usize);
                    for z in 0..e[2] as i64 {
                        for y in 0..e[1] as i64 {
                            for x in 0..e[0] as i64 {
                                let p = VoxelPos::new(
                                    src.min().x + x,
                                    src.min().y + y,
                                    src.min().z + z,
                                );
                                cells.push(ed.get(p)?);
                            }
                        }
                    }
                    if mode == TransformMode::Move {
                        let (smin, smax) = box_of(&src);
                        ed.apply_box(smin, smax, false, &|_| Some(0))?;
                    }
                    let mut it = cells.into_iter();
                    for z in 0..e[2] as i64 {
                        for y in 0..e[1] as i64 {
                            for x in 0..e[0] as i64 {
                                let v = it.next().expect("one value per cell");
                                ed.set(m.cell([x, y, z])?, v)?;
                            }
                        }
                    }
                }
            }
            let writes = ed.finish()?;
            if !writes.is_empty() {
                sets.push(WriteSet {
                    layer: *layer,
                    writes,
                });
            }
        }
        Ok(sets)
    }

    /// The port's `validate` receives a region but no layer, so this backend cannot tell which
    /// layer to inspect; it therefore does not advertise validation (`validate: false`) and
    /// answers with a controlled error instead of claiming validity. Needs a port change
    /// (layer in the call) before it can be implemented.
    fn validate(
        &self,
        _overlay: &dyn OverlayRead,
        _region: &Region,
    ) -> Result<ValidationReport, EditingError> {
        Err(EditingError::InvalidOperation(
            "validation needs a layer, which the port does not carry",
        ))
    }
}
