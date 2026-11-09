//! Layer model (CORE-01): a layer is a transformable container of one representation.

mod kind;

pub use kind::{CellStore, LayerData, LayerKind, PageStore};

use crate::ids::LayerId;
use crate::transform::GeometryTransform;

/// One layer of a geometry asset. Fields are private and there is no mutating API: changes to
/// authoritative data go through the commit boundary (a later task), never through a setter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layer {
    id: LayerId,
    transform: GeometryTransform,
    data: LayerData,
}

impl Layer {
    /// Creates a layer.
    pub fn new(id: LayerId, transform: GeometryTransform, data: LayerData) -> Self {
        Self {
            id,
            transform,
            data,
        }
    }

    /// Layer id.
    pub const fn id(&self) -> LayerId {
        self.id
    }
    /// Placement of the layer.
    pub const fn transform(&self) -> GeometryTransform {
        self.transform
    }
    /// Payload.
    pub const fn data(&self) -> &LayerData {
        &self.data
    }
    /// Representation tag.
    pub const fn kind(&self) -> LayerKind {
        self.data.kind()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layer_reports_its_parts() {
        let l = Layer::new(
            LayerId::new(3),
            GeometryTransform::IDENTITY,
            LayerData::empty(LayerKind::Sdf),
        );
        assert_eq!(l.id(), LayerId::new(3));
        assert_eq!(l.kind(), LayerKind::Sdf);
        assert_eq!(l.transform(), GeometryTransform::IDENTITY);
        assert!(matches!(l.data(), LayerData::Sdf(_)));
    }
}
