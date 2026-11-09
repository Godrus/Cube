//! Cell payload and the five distinct read states of a cell (contracts C-08, C-09).

use std::sync::Arc;

use super::channel::ChannelSet;

/// Opaque mesh page payload for `Mesh` layers. Core does not interpret the bytes.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MeshPage {
    bytes: Vec<u8>,
}

impl MeshPage {
    /// Wraps raw bytes.
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }
    /// The raw bytes.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// Data of one present cell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CellPayload {
    /// Voxel/SDF channels of a brick.
    Channels(ChannelSet),
    /// A mesh page.
    MeshPage(MeshPage),
}

/// Why a cell could not be loaded. Failures are never reported as empty space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum LoadFailureReason {
    /// The stored data was not found although it was expected.
    Missing,
    /// The stored data failed validation.
    Corrupt,
    /// The storage layer reported an I/O error.
    Io,
}

/// What a reader learns about a cell. The five states are never merged:
/// *empty and loaded* (`KnownEmpty`) is different from *not loaded yet* (`NotLoaded`),
/// from *beyond the layer's bounds* (`OutsideBounds`) and from *failed to load* (`LoadFailure`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CellState {
    /// Loaded data (shared, immutable once committed).
    Present(Arc<CellPayload>),
    /// Loaded and known to contain nothing.
    KnownEmpty,
    /// Inside the bounds but not loaded.
    NotLoaded,
    /// Outside the bounds of the layer.
    OutsideBounds,
    /// Loading was attempted and failed.
    LoadFailure(LoadFailureReason),
}

impl CellState {
    /// `true` for `Present` and `KnownEmpty`: the content of the cell is known.
    pub const fn is_loaded(&self) -> bool {
        matches!(self, Self::Present(_) | Self::KnownEmpty)
    }

    /// The payload if present.
    pub fn payload(&self) -> Option<&CellPayload> {
        match self {
            Self::Present(p) => Some(p),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_present_and_known_empty_are_loaded() {
        let present =
            CellState::Present(Arc::new(CellPayload::MeshPage(MeshPage::new(vec![1, 2]))));
        assert!(present.is_loaded() && CellState::KnownEmpty.is_loaded());
        assert!(!CellState::NotLoaded.is_loaded());
        assert!(!CellState::OutsideBounds.is_loaded());
        assert!(!CellState::LoadFailure(LoadFailureReason::Io).is_loaded());
    }

    #[test]
    fn states_are_pairwise_distinct() {
        let all = [
            CellState::KnownEmpty,
            CellState::NotLoaded,
            CellState::OutsideBounds,
            CellState::LoadFailure(LoadFailureReason::Missing),
            CellState::LoadFailure(LoadFailureReason::Corrupt),
            CellState::Present(Arc::new(CellPayload::Channels(ChannelSet::new()))),
        ];
        for (i, a) in all.iter().enumerate() {
            for (j, b) in all.iter().enumerate() {
                assert_eq!(a == b, i == j, "{a:?} vs {b:?}");
            }
        }
    }

    #[test]
    fn payload_accessor_and_mesh_bytes() {
        let page = MeshPage::new(vec![9, 8, 7]);
        assert_eq!(page.bytes(), [9, 8, 7]);
        let s = CellState::Present(Arc::new(CellPayload::MeshPage(page.clone())));
        assert_eq!(s.payload(), Some(&CellPayload::MeshPage(page)));
        assert_eq!(CellState::KnownEmpty.payload(), None);
    }
}
