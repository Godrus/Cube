//! RevisionToken (draft C-16; semantics: docs/api-contracts/revision-and-snapshot-semantics.md).
//! Value width and atomic commit algorithm are Open Decisions (OD-R1, OD-R2); `u64`/`u32` here.

use crate::coord::BrickCoord;
use crate::ids::{AssetId, LayerId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BrickKey {
    pub layer: LayerId,
    pub coord: BrickCoord,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ResourceId {
    Asset(AssetId),
    Layer(LayerId),
    Brick(BrickKey),
}

/// Result of comparing two tokens. There is deliberately no boolean compare.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RevisionOrder {
    Equal,
    /// `self` is newer than `other`.
    Newer,
    /// `self` is older than `other`.
    Older,
    /// Different resource or epoch: must be treated as stale.
    Incomparable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RevisionToken {
    pub resource: ResourceId,
    pub epoch: u32,
    pub value: u64,
}

impl RevisionToken {
    pub fn new(resource: ResourceId, epoch: u32, value: u64) -> Self {
        Self {
            resource,
            epoch,
            value,
        }
    }

    pub fn compare(&self, other: &RevisionToken) -> RevisionOrder {
        if self.resource != other.resource || self.epoch != other.epoch {
            return RevisionOrder::Incomparable;
        }
        match self.value.cmp(&other.value) {
            std::cmp::Ordering::Equal => RevisionOrder::Equal,
            std::cmp::Ordering::Greater => RevisionOrder::Newer,
            std::cmp::Ordering::Less => RevisionOrder::Older,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(res: ResourceId, epoch: u32, v: u64) -> RevisionToken {
        RevisionToken::new(res, epoch, v)
    }

    #[test]
    fn ordering_within_scope() {
        let a = ResourceId::Asset(AssetId(1));
        assert_eq!(t(a, 0, 2).compare(&t(a, 0, 2)), RevisionOrder::Equal);
        assert_eq!(t(a, 0, 3).compare(&t(a, 0, 2)), RevisionOrder::Newer);
        assert_eq!(t(a, 0, 1).compare(&t(a, 0, 2)), RevisionOrder::Older);
    }

    #[test]
    fn different_epoch_or_resource_is_incomparable() {
        let a = ResourceId::Asset(AssetId(1));
        assert_eq!(t(a, 0, 5).compare(&t(a, 1, 5)), RevisionOrder::Incomparable);
        assert_eq!(
            t(a, 0, 5).compare(&t(ResourceId::Asset(AssetId(2)), 0, 5)),
            RevisionOrder::Incomparable
        );
        assert_eq!(
            t(a, 0, 5).compare(&t(ResourceId::Layer(LayerId(1)), 0, 5)),
            RevisionOrder::Incomparable
        );
    }
}
