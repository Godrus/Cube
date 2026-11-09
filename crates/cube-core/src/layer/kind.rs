//! Representation tag and layer payload (contracts C-02, C-03).

use crate::cell::CellStore;

/// Which representation a layer holds. A closed set: a new representation needs an ACR, it is not
/// an extension point (DEC-053 note; plan section 5.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LayerKind {
    /// Cubic voxels (ordinary and micro-voxels differ only by level).
    Voxel,
    /// Signed-distance field.
    Sdf,
    /// Polygon mesh pages.
    Mesh,
}

/// Sparse store of mesh pages for `Mesh` layers (contents arrive with a later task).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PageStore {
    _private: (),
}

/// Payload of a layer: one variant per representation. No voxel-only type appears in the layer API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayerData {
    /// Voxel cells.
    Voxel(CellStore),
    /// SDF cells.
    Sdf(CellStore),
    /// Mesh pages.
    Mesh(PageStore),
}

impl LayerData {
    /// The representation tag of this payload; it cannot disagree with the payload.
    pub const fn kind(&self) -> LayerKind {
        match self {
            Self::Voxel(_) => LayerKind::Voxel,
            Self::Sdf(_) => LayerKind::Sdf,
            Self::Mesh(_) => LayerKind::Mesh,
        }
    }

    /// An empty payload of the given kind.
    pub fn empty(kind: LayerKind) -> Self {
        match kind {
            LayerKind::Voxel => Self::Voxel(CellStore::default()),
            LayerKind::Sdf => Self::Sdf(CellStore::default()),
            LayerKind::Mesh => Self::Mesh(PageStore::default()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_always_matches_the_payload() {
        for kind in [LayerKind::Voxel, LayerKind::Sdf, LayerKind::Mesh] {
            assert_eq!(LayerData::empty(kind).kind(), kind);
        }
    }
}
