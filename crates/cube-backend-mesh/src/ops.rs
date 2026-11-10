//! The mesh backend (MSH-01): add, remove and set material as write sets on mesh pages.
//!
//! It implements `GeometryBackend` and nothing else: it reads through the overlay, returns
//! `WriteSet`s and never commits. Deterministic and history independent: a page is always
//! written in the canonical form of [`crate::page`], so the same set of triangles gives the same
//! bytes. A triangle belongs to the page of the brick that contains its centroid and every
//! triangle touched by an operation must have its centroid inside the operation's write region
//! (an added triangle outside it is an error, never a silent write elsewhere).
//!
//! Safety rules: a page that is not loaded is an error when the operation could change it (a
//! new triangle for an unloaded page, removal or material change in an unloaded page); a
//! `KnownEmpty` page is an empty page; a voxel/SDF payload in a mesh layer is
//! `IncompatibleGeometryTypes` (no hidden conversion); a page that would exceed the triangle
//! budget, or violate the page rules (degenerate, duplicate or inconsistently wound triangles),
//! is refused with a typed reason and nothing is written. A page that becomes empty is stored as
//! `KnownEmpty`; an unchanged page produces no write.

use std::collections::BTreeMap;
use std::sync::Arc;

use cube_core::{
    BrickCoord, CellChange, CellKey, CellPayload, CellState, CellWrite, ExpectedRevision, LayerId,
    LayerKind, MeshPage, Region, VoxelPos, WriteSet,
};
use cube_editing_api::{
    BackendCapabilities, EditingError, EditingOperation, GeometryBackend, OverlayRead,
    ValidationReport, kinds,
};
use cube_processing::Snapshot;

use crate::page::{Page, Tri};
use crate::params::{AbsTri, MeshOp, decode};

/// Limits protecting memory; configuration, not world limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    /// Most pages one operation may touch.
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

/// The mesh backend.
#[derive(Debug)]
pub struct MeshBackend {
    caps: BackendCapabilities,
    limits: Limits,
}

impl MeshBackend {
    /// A backend with default limits.
    pub fn new() -> Self {
        Self::with_limits(Limits::default())
    }

    /// A backend with explicit limits.
    pub fn with_limits(limits: Limits) -> Self {
        Self {
            caps: BackendCapabilities {
                layer_kind: LayerKind::Mesh,
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

    /// Validates every present page of a region of a mesh layer with the page rules (MSH-02).
    /// Pages that are not loaded are unknown, not invalid. (The port's `validate` carries no
    /// layer, see TD-2.)
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
                    CellPayload::MeshPage(m) => {
                        if Page::decode(m.bytes()).is_err() {
                            report.invalid.push(key);
                        }
                    }
                    CellPayload::Channels(_) => report.invalid.push(key),
                },
                CellState::KnownEmpty | CellState::NotLoaded | CellState::OutsideBounds => {}
                CellState::LoadFailure(_) => report.invalid.push(key),
            }
        }
        report.invalid.sort();
        Ok(report)
    }
}

impl Default for MeshBackend {
    fn default() -> Self {
        Self::new()
    }
}

/// Brick that contains the centroid of an absolute triangle, and the triangle relative to that
/// brick's origin.
fn place(t: &AbsTri) -> Result<(BrickCoord, VoxelPos, Tri), EditingError> {
    let (v, material) = t;
    let sum = |a: usize| i128::from(v[0][a]) + i128::from(v[1][a]) + i128::from(v[2][a]);
    let vox = |a: usize| -> Result<i64, EditingError> {
        i64::try_from(sum(a).div_euclid(3).div_euclid(256))
            .map_err(|_| invalid("triangle outside the coordinate range"))
    };
    let p = VoxelPos::new(vox(0)?, vox(1)?, vox(2)?);
    let (b, _) = p.split();
    let o = b
        .origin()
        .map_err(|_| invalid("triangle outside the coordinate range"))?;
    let rel = |q: [i64; 3]| -> Result<[i32; 3], EditingError> {
        let r = |c: i64, oc: i64| i128::from(c) - i128::from(oc) * 256;
        let f = |c: i64, oc: i64| {
            i32::try_from(r(c, oc)).map_err(|_| invalid("triangle too large for a page"))
        };
        Ok([f(q[0], o.x)?, f(q[1], o.y)?, f(q[2], o.z)?])
    };
    Ok((
        b,
        p,
        Tri {
            v: [rel(v[0])?, rel(v[1])?, rel(v[2])?],
            material: *material,
        },
    ))
}

fn centroid_voxel(t: &Tri, origin: VoxelPos) -> VoxelPos {
    let s = |a: usize| i128::from(t.v[0][a]) + i128::from(t.v[1][a]) + i128::from(t.v[2][a]);
    let f = |a: usize, o: i64| (s(a).div_euclid(3).div_euclid(256) + i128::from(o)) as i64;
    VoxelPos::new(f(0, origin.x), f(1, origin.y), f(2, origin.z))
}

fn inside(r: &Region, p: VoxelPos) -> bool {
    let (lo, hi) = (r.min(), r.max());
    p.x >= lo.x && p.x < hi.x && p.y >= lo.y && p.y < hi.y && p.z >= lo.z && p.z < hi.z
}

fn load(
    overlay: &dyn OverlayRead,
    layer: LayerId,
    level: i8,
    brick: BrickCoord,
) -> Result<Page, EditingError> {
    let key = CellKey::new(layer, level, brick).map_err(|_| EditingError::InvalidResolution)?;
    match overlay.cell(key)? {
        CellState::Present(p) => match &*p {
            CellPayload::MeshPage(m) => {
                Page::decode(m.bytes()).map_err(|_| invalid("stored mesh page is invalid"))
            }
            CellPayload::Channels(_) => Err(EditingError::IncompatibleGeometryTypes),
        },
        CellState::KnownEmpty => Ok(Page::default()),
        CellState::NotLoaded => Err(invalid("brick is not loaded")),
        CellState::OutsideBounds => Err(invalid("outside the layer bounds")),
        CellState::LoadFailure(_) => Err(invalid("brick failed to load")),
    }
}

impl GeometryBackend for MeshBackend {
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
            // brick -> (original page, new triangles)
            let mut pages: BTreeMap<BrickCoord, (Page, Vec<Tri>)> = BTreeMap::new();
            match &op {
                MeshOp::Add(tris) => {
                    for t in tris {
                        let (b, p, rel) = place(t)?;
                        if !inside(&region, p) {
                            return Err(invalid("triangle centroid outside the write region"));
                        }
                        if let std::collections::btree_map::Entry::Vacant(e) = pages.entry(b) {
                            e.insert((load(overlay, *layer, level, b)?, Vec::new()));
                        }
                        let entry = pages.get_mut(&b).expect("inserted");
                        let rel = rel.canonical();
                        if !entry.0.tris.contains(&rel) {
                            entry.1.push(rel);
                        }
                    }
                }
                MeshOp::Remove | MeshOp::SetMaterial(_) => {
                    for brick in region.bricks() {
                        let page = load(overlay, *layer, level, brick)?;
                        pages.insert(brick, (page, Vec::new()));
                    }
                }
            }
            let mut writes = Vec::new();
            for (brick, (old, added)) in pages {
                let origin = brick
                    .origin()
                    .map_err(|_| invalid("brick outside the coordinate range"))?;
                let mut tris: Vec<Tri> = old.tris.clone();
                match &op {
                    MeshOp::Add(_) => tris.extend(added),
                    MeshOp::Remove => {
                        tris.retain(|t| !inside(&region, centroid_voxel(t, origin)));
                    }
                    MeshOp::SetMaterial(m) => {
                        for t in &mut tris {
                            if inside(&region, centroid_voxel(t, origin)) {
                                t.material = *m;
                            }
                        }
                    }
                }
                let new = Page::from_tris(tris).map_err(|e| match e {
                    crate::page::MeshError::TooLarge => invalid("mesh page exceeds the budget"),
                    crate::page::MeshError::Degenerate { .. } => invalid("degenerate triangle"),
                    crate::page::MeshError::InconsistentWinding => {
                        invalid("inconsistent winding or non-manifold edge")
                    }
                    _ => invalid("mesh page violates the page rules"),
                })?;
                if new == old {
                    continue;
                }
                let change = if new.tris.is_empty() {
                    CellChange::KnownEmpty
                } else {
                    CellChange::Put(CellPayload::MeshPage(MeshPage::new(
                        new.encode()
                            .map_err(|_| invalid("mesh page violates the page rules"))?,
                    )))
                };
                writes.push(CellWrite {
                    level,
                    brick,
                    expected: ExpectedRevision::Absent,
                    change,
                });
            }
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
    /// answers with a controlled error; use [`MeshBackend::validate_layer`].
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
