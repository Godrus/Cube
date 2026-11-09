//! Export/import views and the stable content hash (CORE-11).
//!
//! A view is plain data that a persistence layer can serialise; Core defines no file format.
//! The content hash covers the content only (structure, transforms, bounds, cell payloads), not
//! the revisions, and does not depend on how channels are encoded internally.
//!
//! Hash algorithm `CH1` (decided under DEC-001): FNV-1a with the 128-bit offset basis and prime
//! over a canonical byte stream. All integers are little-endian, collections are in a fixed order,
//! every variable part is length- or tag-prefixed. It is a content fingerprint for tests and
//! change detection, not a cryptographic hash and not the region stamp.

use crate::asset::GeometryAsset;
use crate::cell::{BrickBounds, CellPayload, CellState, CellStore, ChannelId};
use crate::commit::AssetState;
use crate::coord::{BrickCoord, LocalPos, check_level};
use crate::error::CoreError;
use crate::ids::{AssetId, LayerId};
use crate::layer::{Layer, LayerData, LayerKind};
use crate::revision::{Epoch, Revision};
use crate::transform::GeometryTransform;

/// One committed cell of a layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellView {
    /// Refinement level.
    pub level: i8,
    /// Brick coordinate at that level.
    pub brick: BrickCoord,
    /// Committed revision of the cell.
    pub revision: Revision,
    /// The payload, `None` for "known empty".
    pub payload: Option<CellPayload>,
}

/// One layer with its committed cells in ascending `(level, brick)` order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayerView {
    /// Layer id.
    pub id: LayerId,
    /// Representation.
    pub kind: LayerKind,
    /// Placement.
    pub transform: GeometryTransform,
    /// Bounds, if the layer is bounded.
    pub bounds: Option<BrickBounds>,
    /// Layer revision.
    pub revision: Revision,
    /// Committed cells (empty for mesh layers until they have a page store).
    pub cells: Vec<CellView>,
}

/// A whole asset as plain data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetView {
    /// Asset id.
    pub id: AssetId,
    /// Epoch of the asset.
    pub epoch: Epoch,
    /// Asset revision.
    pub revision: Revision,
    /// Layers in creation order.
    pub layers: Vec<LayerView>,
}

/// 128-bit content fingerprint (algorithm `CH1`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContentHash(u128);

impl ContentHash {
    /// Algorithm identifier of this hash.
    pub const ALGORITHM: &'static str = "CH1";
    /// The 128-bit value.
    pub const fn value(self) -> u128 {
        self.0
    }
}

const FNV_OFFSET: u128 = 0x6c62_272e_07bb_0142_62b8_2175_6295_c58d;
const FNV_PRIME: u128 = 0x0000_0000_0100_0000_0000_0000_0000_013b;

struct Fnv(u128);

impl Fnv {
    fn bytes(&mut self, b: &[u8]) {
        for x in b {
            self.0 = (self.0 ^ u128::from(*x)).wrapping_mul(FNV_PRIME);
        }
    }
    fn u8(&mut self, v: u8) {
        self.bytes(&[v]);
    }
    fn i64(&mut self, v: i64) {
        self.bytes(&v.to_le_bytes());
    }
    fn u64(&mut self, v: u64) {
        self.bytes(&v.to_le_bytes());
    }
    fn u16(&mut self, v: u16) {
        self.bytes(&v.to_le_bytes());
    }
}

fn kind_code(k: LayerKind) -> u8 {
    match k {
        LayerKind::Voxel => 1,
        LayerKind::Sdf => 2,
        LayerKind::Mesh => 3,
    }
}

fn channel_tag(id: ChannelId) -> (u8, u8) {
    match id {
        ChannelId::Material => (1, 0),
        ChannelId::Density => (2, 0),
        ChannelId::Attribute(n) => (3, n),
    }
}

fn hash_payload(h: &mut Fnv, p: &CellPayload) {
    match p {
        CellPayload::Channels(set) => {
            h.u8(1);
            h.u64(set.len() as u64);
            for (id, ch) in set.iter() {
                let (t, n) = channel_tag(id);
                h.u8(t);
                h.u8(n);
                // Encoding-independent: always the 4096 values.
                for p in LocalPos::all() {
                    h.u16(ch.get(p));
                }
            }
        }
        CellPayload::MeshPage(page) => {
            h.u8(2);
            h.u64(page.bytes().len() as u64);
            h.bytes(page.bytes());
        }
    }
}

impl AssetView {
    /// The content fingerprint of this view (revisions and epoch are not part of it).
    pub fn content_hash(&self) -> ContentHash {
        let mut h = Fnv(FNV_OFFSET);
        h.u64(self.id.raw());
        h.u64(self.layers.len() as u64);
        for l in &self.layers {
            h.u64(l.id.raw());
            h.u8(kind_code(l.kind));
            let t = l.transform.translation();
            for v in t {
                h.i64(v);
            }
            h.u8(l.transform.rotation().index());
            h.u8(l.transform.scale_shift() as u8);
            match &l.bounds {
                None => h.u8(0),
                Some(b) => {
                    h.u8(1);
                    for v in [b.min(), b.max()] {
                        h.i64(v.x);
                        h.i64(v.y);
                        h.i64(v.z);
                    }
                }
            }
            h.u64(l.cells.len() as u64);
            for c in &l.cells {
                h.u8(c.level as u8);
                h.i64(c.brick.x);
                h.i64(c.brick.y);
                h.i64(c.brick.z);
                match &c.payload {
                    None => h.u8(0),
                    Some(p) => hash_payload(&mut h, p),
                }
            }
        }
        ContentHash(h.0)
    }
}

impl AssetState {
    /// Exports the state as plain data. Failed or unloaded cells are not content and are skipped.
    pub fn export_view(&self) -> AssetView {
        let layers = self
            .asset()
            .layers()
            .iter()
            .map(|l| {
                let (store, bounds): (Option<&CellStore>, _) = match l.data() {
                    LayerData::Voxel(s) | LayerData::Sdf(s) => (Some(s), s.bounds()),
                    LayerData::Mesh(_) => (None, None),
                };
                let cells = store
                    .map(|s| {
                        s.iter()
                            .filter_map(|(level, brick, state)| {
                                let payload = match state {
                                    CellState::Present(p) => Some((*p).clone()),
                                    CellState::KnownEmpty => None,
                                    _ => return None,
                                };
                                let revision = s.revision(level, brick)?;
                                Some(CellView {
                                    level,
                                    brick,
                                    revision,
                                    payload,
                                })
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                LayerView {
                    id: l.id(),
                    kind: l.kind(),
                    transform: l.transform(),
                    bounds,
                    revision: self
                        .layer_token(l.id())
                        .map_or(Revision::INITIAL, |t| t.value()),
                    cells,
                }
            })
            .collect();
        AssetView {
            id: self.asset().id(),
            epoch: self.asset_token().epoch(),
            revision: self.asset_token().value(),
            layers,
        }
    }
}

/// Rebuilds the asset structure and state parts from a view (used by `AssetHost::from_view`).
pub(crate) fn state_parts(
    view: &AssetView,
) -> Result<(GeometryAsset, Vec<(LayerId, Revision)>), CoreError> {
    let mut layers = Vec::new();
    let mut revisions = Vec::new();
    for l in &view.layers {
        let data = match l.kind {
            LayerKind::Mesh => {
                if !l.cells.is_empty() {
                    return Err(CoreError::InvalidView);
                }
                LayerData::empty(LayerKind::Mesh)
            }
            kind => {
                let mut store = match l.bounds {
                    Some(b) => CellStore::with_bounds(b),
                    None => CellStore::new(),
                };
                let mut seen = std::collections::BTreeSet::new();
                for c in &l.cells {
                    check_level(c.level)?;
                    if !seen.insert((c.level, c.brick)) {
                        return Err(CoreError::DuplicateKey);
                    }
                    if !store.in_bounds(c.level, c.brick) {
                        return Err(CoreError::InvalidView);
                    }
                    if let Some(CellPayload::MeshPage(_)) = &c.payload {
                        return Err(CoreError::InvalidView);
                    }
                    store.commit_cell(
                        c.level,
                        c.brick,
                        c.payload.clone().map(std::sync::Arc::new),
                        c.revision,
                    );
                }
                if kind == LayerKind::Sdf {
                    LayerData::Sdf(store)
                } else {
                    LayerData::Voxel(store)
                }
            }
        };
        layers.push(Layer::new(l.id, l.transform, data));
        revisions.push((l.id, l.revision));
    }
    Ok((GeometryAsset::new(view.id, layers)?, revisions))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fnv_constants_match_the_published_128_bit_parameters() {
        assert_eq!(
            FNV_OFFSET,
            144_066_263_297_769_815_596_495_629_667_062_367_629
        );
        assert_eq!(FNV_PRIME, 309_485_009_821_345_068_724_781_371);
    }

    #[test]
    fn hash_of_nothing_is_the_offset_basis_and_bytes_change_it() {
        let h = Fnv(FNV_OFFSET);
        assert_eq!(h.0, FNV_OFFSET);
        let mut a = Fnv(FNV_OFFSET);
        a.u8(1);
        let mut b = Fnv(FNV_OFFSET);
        b.u8(2);
        assert_ne!(a.0, b.0);
        assert_ne!(a.0, FNV_OFFSET);
    }
}
