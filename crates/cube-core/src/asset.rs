//! Geometry asset: an immutable container of layers (contract C-01, core slice).

use std::collections::BTreeSet;

use crate::error::CoreError;
use crate::ids::{AssetId, LayerId};
use crate::layer::Layer;

/// A geometry asset. Layers are fixed at construction in this task; structural changes become
/// commits with the commit boundary (later tasks), so there is deliberately no mutating method.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeometryAsset {
    id: AssetId,
    layers: Vec<Layer>,
}

impl GeometryAsset {
    /// Creates an asset; fails if two layers share an id. Layer order is preserved.
    pub fn new(id: AssetId, layers: Vec<Layer>) -> Result<Self, CoreError> {
        let mut seen = BTreeSet::new();
        for l in &layers {
            if !seen.insert(l.id()) {
                return Err(CoreError::DuplicateLayer(l.id()));
            }
        }
        Ok(Self { id, layers })
    }

    /// Asset id.
    pub const fn id(&self) -> AssetId {
        self.id
    }

    /// Layers in creation order.
    pub fn layers(&self) -> &[Layer] {
        &self.layers
    }

    /// Looks a layer up by id.
    pub fn layer(&self, id: LayerId) -> Option<&Layer> {
        self.layers.iter().find(|l| l.id() == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer::{LayerData, LayerKind};
    use crate::transform::GeometryTransform;

    fn layer(id: u64, kind: LayerKind) -> Layer {
        Layer::new(
            LayerId::new(id),
            GeometryTransform::IDENTITY,
            LayerData::empty(kind),
        )
    }

    #[test]
    fn keeps_order_and_finds_layers() {
        let a = GeometryAsset::new(
            AssetId::new(1),
            vec![
                layer(9, LayerKind::Voxel),
                layer(2, LayerKind::Mesh),
                layer(5, LayerKind::Sdf),
            ],
        )
        .unwrap();
        let ids: Vec<u64> = a.layers().iter().map(|l| l.id().raw()).collect();
        assert_eq!(ids, [9, 2, 5]);
        assert_eq!(
            a.layer(LayerId::new(2)).map(Layer::kind),
            Some(LayerKind::Mesh)
        );
        assert!(a.layer(LayerId::new(3)).is_none());
        assert_eq!(a.id(), AssetId::new(1));
    }

    #[test]
    fn rejects_duplicate_layer_ids() {
        let r = GeometryAsset::new(
            AssetId::new(1),
            vec![layer(4, LayerKind::Voxel), layer(4, LayerKind::Sdf)],
        );
        assert_eq!(r, Err(CoreError::DuplicateLayer(LayerId::new(4))));
    }

    #[test]
    fn empty_asset_is_valid() {
        assert!(
            GeometryAsset::new(AssetId::new(0), vec![])
                .unwrap()
                .layers()
                .is_empty()
        );
    }
}
