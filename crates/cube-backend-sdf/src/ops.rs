//! The SDF backend (SDF-01): union, subtraction and material assignment as write sets.
//!
//! It implements `GeometryBackend` and nothing else: it reads through the overlay, returns
//! `WriteSet`s and never commits. Deterministic: the same operation on the same overlay gives
//! identical write sets, bricks ascending. There is no world-size limit; an operation is bounded
//! by [`Limits`] and a larger one is refused, not truncated.
//!
//! Safety rules (nothing is lost or invented silently):
//!
//! * a brick that is **not loaded** is an error as soon as the operation could change it; a
//!   brick the shape cannot reach (farther than the field's reach) is neither read nor written;
//! * a brick outside the layer bounds or one that failed to load is an error; a mesh page in
//!   an SDF layer is `IncompatibleGeometryTypes` (no hidden conversion);
//! * a `KnownEmpty` brick is entirely outside (positive); a loaded brick without a `Density`
//!   channel is read as entirely outside, its other channels are carried over;
//! * a brick that ends up entirely far outside (every sample at least 16 voxels from a
//!   surface), without material and without other channels, becomes `KnownEmpty`; a loaded brick the operation did not change produces no write.

use std::collections::BTreeMap;
use std::sync::Arc;

use cube_core::{
    BRICK_VOLUME, BrickCoord, CellChange, CellKey, CellPayload, CellState, CellWrite, Channel,
    ChannelId, ChannelSet, ExpectedRevision, LayerId, LayerKind, LocalPos, Region, VoxelPos,
    WriteSet,
};
use cube_editing_api::{
    BackendCapabilities, EditingError, EditingOperation, GeometryBackend, OverlayRead,
    ValidationReport, kinds,
};
use cube_processing::Snapshot;

use crate::field::{OUTSIDE, SCALE, Shape};

/// A brick whose every sample is at least this far outside (16 voxels) holds no surface and
/// no information that a later operation could need: it is stored as `KnownEmpty`.
const FAR: i16 = (16 * SCALE) as i16;
use crate::params::{SdfOp, decode};

/// Limits protecting memory; configuration, not world limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    /// Most bricks one operation may touch.
    pub max_bricks: u64,
}

impl Default for Limits {
    fn default() -> Self {
        Self { max_bricks: 4096 }
    }
}

fn invalid(why: &'static str) -> EditingError {
    EditingError::InvalidOperation(why)
}

fn idx(l: LocalPos) -> usize {
    usize::from(l.x()) + 16 * (usize::from(l.y()) + 16 * usize::from(l.z()))
}

/// The SDF backend.
#[derive(Debug)]
pub struct SdfBackend {
    caps: BackendCapabilities,
    limits: Limits,
}

impl SdfBackend {
    /// A backend with default limits.
    pub fn new() -> Self {
        Self::with_limits(Limits::default())
    }

    /// A backend with explicit limits.
    pub fn with_limits(limits: Limits) -> Self {
        Self {
            caps: BackendCapabilities {
                layer_kind: LayerKind::Sdf,
                kinds: [
                    kinds::SET_MATERIAL,
                    kinds::ADD_GEOMETRY,
                    kinds::REMOVE_GEOMETRY,
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
            _ => Err(invalid("region too large")),
        }
    }

    /// Checks the consistency of a region of an SDF layer: no forbidden `i16::MIN` density, no
    /// material outside the surface, no mesh page. Bricks that are not loaded are unknown, not
    /// invalid, and are skipped. (The port's `validate` carries no layer, see TD-2.)
    pub fn validate_layer(
        &self,
        overlay: &dyn OverlayRead,
        layer: LayerId,
        region: &Region,
    ) -> Result<ValidationReport, EditingError> {
        self.check_size(region)?;
        let mut report = ValidationReport::default();
        for brick in region.bricks() {
            let key = CellKey::new(layer, region.level(), brick)
                .map_err(|_| EditingError::InvalidResolution)?;
            match overlay.cell(key)? {
                CellState::Present(p) => match &*p {
                    CellPayload::Channels(set) => {
                        let dens = set.channel(ChannelId::Density);
                        let mat = set.channel(ChannelId::Material);
                        let bad = LocalPos::all().any(|l| {
                            let d = dens.map_or(OUTSIDE, |c| c.get(l) as i16);
                            let m = mat.map_or(0, |c| c.get(l));
                            d == i16::MIN || (d >= 0 && m != 0)
                        });
                        if bad {
                            report.invalid.push(key);
                        }
                    }
                    CellPayload::MeshPage(_) => report.invalid.push(key),
                },
                CellState::KnownEmpty | CellState::NotLoaded | CellState::OutsideBounds => {}
                CellState::LoadFailure(_) => report.invalid.push(key),
            }
        }
        report.invalid.sort();
        Ok(report)
    }
}

impl Default for SdfBackend {
    fn default() -> Self {
        Self::new()
    }
}

struct Buf {
    dens: Vec<i16>,
    mat: Vec<u32>,
    others: Vec<(ChannelId, Channel)>,
    changed: bool,
}

fn load(
    overlay: &dyn OverlayRead,
    layer: LayerId,
    level: i8,
    brick: BrickCoord,
) -> Result<Buf, EditingError> {
    let key = CellKey::new(layer, level, brick).map_err(|_| EditingError::InvalidResolution)?;
    match overlay.cell(key)? {
        CellState::Present(p) => match &*p {
            CellPayload::Channels(set) => {
                let mut dens = vec![OUTSIDE; BRICK_VOLUME];
                let mut mat = vec![0u32; BRICK_VOLUME];
                if let Some(c) = set.channel(ChannelId::Density) {
                    for l in LocalPos::all() {
                        dens[idx(l)] = (c.get(l) as u16) as i16;
                    }
                }
                if let Some(c) = set.channel(ChannelId::Material) {
                    for l in LocalPos::all() {
                        mat[idx(l)] = c.get(l);
                    }
                }
                let others = set
                    .iter()
                    .filter(|(id, _)| *id != ChannelId::Material && *id != ChannelId::Density)
                    .map(|(id, c)| (id, c.clone()))
                    .collect();
                Ok(Buf {
                    dens,
                    mat,
                    others,
                    changed: false,
                })
            }
            CellPayload::MeshPage(_) => Err(EditingError::IncompatibleGeometryTypes),
        },
        CellState::KnownEmpty => Ok(Buf {
            dens: vec![OUTSIDE; BRICK_VOLUME],
            mat: vec![0; BRICK_VOLUME],
            others: vec![],
            changed: false,
        }),
        CellState::NotLoaded => Err(invalid("brick is not loaded")),
        CellState::OutsideBounds => Err(invalid("outside the layer bounds")),
        CellState::LoadFailure(_) => Err(invalid("brick failed to load")),
    }
}

fn channel_of(values: &[u32]) -> Channel {
    let first = values[0];
    let mut ch = Channel::constant(first);
    for (l, v) in LocalPos::all().zip(values) {
        if *v != first {
            ch.set(l, *v);
        }
    }
    ch.compact();
    ch
}

fn finish(bufs: BTreeMap<BrickCoord, Buf>, level: i8) -> Vec<CellWrite> {
    let mut out = Vec::new();
    for (brick, b) in bufs {
        if !b.changed {
            continue;
        }
        let all_outside = b.dens.iter().all(|d| *d >= FAR);
        let no_material = b.mat.iter().all(|m| *m == 0);
        let change = if all_outside && no_material && b.others.is_empty() {
            CellChange::KnownEmpty
        } else {
            let raw: Vec<u32> = b.dens.iter().map(|d| u32::from(*d as u16)).collect();
            let mut set = ChannelSet::new().with(ChannelId::Density, channel_of(&raw));
            if !no_material {
                set = set.with(ChannelId::Material, channel_of(&b.mat));
            }
            for (id, c) in b.others {
                set = set.with(id, c);
            }
            CellChange::Put(CellPayload::Channels(set))
        };
        out.push(CellWrite {
            level,
            brick,
            expected: ExpectedRevision::Absent,
            change,
        });
    }
    out
}

/// Inclusive lattice range of brick `b` intersected with `[min, max)` and `influence`.
fn clip(
    b: BrickCoord,
    region: &Region,
    influence: Option<&([i128; 3], [i128; 3])>,
) -> Option<([i64; 3], [i64; 3])> {
    let o = [
        i128::from(b.x) * 16,
        i128::from(b.y) * 16,
        i128::from(b.z) * 16,
    ];
    let (rmin, rmax) = (region.min(), region.max());
    let rmin = [i128::from(rmin.x), i128::from(rmin.y), i128::from(rmin.z)];
    let rmax = [i128::from(rmax.x), i128::from(rmax.y), i128::from(rmax.z)];
    let mut lo = [0i128; 3];
    let mut hi = [0i128; 3];
    for a in 0..3 {
        lo[a] = o[a].max(rmin[a]);
        hi[a] = (o[a] + 15).min(rmax[a] - 1);
        if let Some((imin, imax)) = influence {
            lo[a] = lo[a].max(imin[a]);
            hi[a] = hi[a].min(imax[a]);
        }
        if lo[a] > hi[a] {
            return None;
        }
    }
    Some((lo.map(|v| v as i64), hi.map(|v| v as i64)))
}

impl GeometryBackend for SdfBackend {
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
        let shape: Option<Shape> = match op {
            SdfOp::Add { shape, .. } | SdfOp::Remove { shape } => Some(shape),
            SdfOp::SetMaterial { .. } => None,
        };
        let influence = shape.as_ref().and_then(Shape::influence);
        let mut sets = Vec::new();
        for layer in &o.layers {
            let mut bufs: BTreeMap<BrickCoord, Buf> = BTreeMap::new();
            for brick in region.bricks() {
                let infl = if shape.is_some() {
                    match &influence {
                        Some(i) => Some(i),
                        None => continue,
                    }
                } else {
                    None
                };
                let Some((lo, hi)) = clip(brick, &region, infl) else {
                    continue;
                };
                let mut buf = load(overlay, *layer, level, brick)?;
                for z in lo[2]..=hi[2] {
                    for y in lo[1]..=hi[1] {
                        for x in lo[0]..=hi[0] {
                            let p = VoxelPos::new(x, y, z);
                            let i = idx(p.split().1);
                            let old = buf.dens[i];
                            match op {
                                SdfOp::Add { shape, material } => {
                                    let s = shape.sample(p);
                                    if s < old {
                                        buf.dens[i] = s;
                                        buf.changed = true;
                                    }
                                    if s < 0 && material != 0 && buf.mat[i] != u32::from(material) {
                                        buf.mat[i] = u32::from(material);
                                        buf.changed = true;
                                    }
                                }
                                SdfOp::Remove { shape } => {
                                    let s = shape.sample(p);
                                    let new = old.max(-s);
                                    if new != old {
                                        buf.dens[i] = new;
                                        buf.changed = true;
                                    }
                                    if new >= 0 && buf.mat[i] != 0 {
                                        buf.mat[i] = 0;
                                        buf.changed = true;
                                    }
                                }
                                SdfOp::SetMaterial { material } => {
                                    if old < 0 && buf.mat[i] != u32::from(material) {
                                        buf.mat[i] = u32::from(material);
                                        buf.changed = true;
                                    }
                                }
                            }
                        }
                    }
                }
                bufs.insert(brick, buf);
            }
            let writes = finish(bufs, level);
            if !writes.is_empty() {
                sets.push(WriteSet {
                    layer: *layer,
                    writes,
                });
            }
        }
        Ok(sets)
    }

    /// The port's `validate` carries no layer (TD-2), so the backend does not advertise it and
    /// answers with a controlled error; use [`SdfBackend::validate_layer`].
    fn validate(
        &self,
        _overlay: &dyn OverlayRead,
        _region: &Region,
    ) -> Result<ValidationReport, EditingError> {
        Err(invalid(
            "validation needs a layer, which the port does not carry",
        ))
    }
}
