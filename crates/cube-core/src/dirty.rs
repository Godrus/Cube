//! DirtyRegion (draft C-18): actually changed data, produced by the commit primitive.

use crate::ids::{AssetId, LayerId, OperationId};
use crate::region::Region;
use crate::revision::RevisionToken;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChangeType {
    Geometry,
    Material,
    Attribute,
    Transform,
    Topology,
    Resolution,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DirtyRegion {
    pub asset: AssetId,
    pub layer: LayerId,
    pub region: Region,
    pub change: ChangeType,
    /// Layer token after the commit that produced this region.
    pub source: RevisionToken,
    pub operation: Option<OperationId>,
}
