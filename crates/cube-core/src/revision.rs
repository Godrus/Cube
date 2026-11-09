//! Revisions, tokens and region stamps (CORE-05, contract C-16; DEC-059..DEC-062).
//!
//! A revision is a monotonic counter scoped to one resource. It never wraps: reaching the end of
//! the range is a critical error. Tokens of different resources or epochs are *incomparable*, and
//! every consumer must treat incomparable as stale.

use crate::error::CoreError;
use crate::ids::{AssetId, LayerId};
use crate::key::CellKey;

/// Generation of a resource's history; changes when it is re-created or recovered from older state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Epoch(u32);

impl Epoch {
    /// The first epoch.
    pub const INITIAL: Self = Self(0);
    /// Epoch of client-side predicted revisions; only comparable with itself.
    pub const LOCAL_PREDICTION: Self = Self(u32::MAX);

    /// Wraps a raw epoch value.
    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }
    /// The raw value.
    pub const fn raw(self) -> u32 {
        self.0
    }
    /// The next epoch; fails instead of wrapping or entering the reserved prediction value.
    pub const fn next(self) -> Result<Self, CoreError> {
        if self.0 >= u32::MAX - 1 {
            Err(CoreError::RevisionOverflow)
        } else {
            Ok(Self(self.0 + 1))
        }
    }
}

/// Monotonic revision counter value (width fixed by DEC-060: `u64`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Revision(u64);

impl Revision {
    /// Revision of a freshly created resource.
    pub const INITIAL: Self = Self(0);

    /// Wraps a raw value.
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }
    /// The raw value.
    pub const fn raw(self) -> u64 {
        self.0
    }
    /// `self + 1`; overflow is a critical error and never wraps.
    pub const fn next(self) -> Result<Self, CoreError> {
        match self.0.checked_add(1) {
            Some(v) => Ok(Self(v)),
            None => Err(CoreError::RevisionOverflow),
        }
    }
}

/// Identifier of a derived-data slot owned by Processing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DerivedSlotId(pub u64);

/// What a token is about; the scope is implied by the variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceId {
    /// A geometry asset.
    Asset(AssetId),
    /// A layer.
    Layer(LayerId),
    /// One brick.
    Cell(CellKey),
    /// A region, identified by its stamp.
    Region(RegionStamp),
    /// A derived-data slot.
    Derived(DerivedSlotId),
}

/// Result of comparing two tokens. There is deliberately no boolean comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Comparison {
    /// Same resource, epoch and value: unchanged.
    Equal,
    /// `self` is newer than `other`.
    Newer,
    /// `self` is older than `other` (stale).
    Older,
    /// Different resource or epoch: cannot be ordered; treat as stale.
    Incomparable,
}

/// A revision of one resource in one epoch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RevisionToken {
    resource: ResourceId,
    epoch: Epoch,
    value: Revision,
}

impl RevisionToken {
    /// Creates a token.
    pub const fn new(resource: ResourceId, epoch: Epoch, value: Revision) -> Self {
        Self {
            resource,
            epoch,
            value,
        }
    }
    /// The resource.
    pub const fn resource(&self) -> ResourceId {
        self.resource
    }
    /// The epoch.
    pub const fn epoch(&self) -> Epoch {
        self.epoch
    }
    /// The revision value.
    pub const fn value(&self) -> Revision {
        self.value
    }

    /// The token with the value increased by one; fails on overflow.
    pub const fn bumped(self) -> Result<Self, CoreError> {
        match self.value.next() {
            Ok(v) => Ok(Self { value: v, ..self }),
            Err(e) => Err(e),
        }
    }

    /// Compares with `other`; tokens are comparable only if resource and epoch are equal.
    pub fn compare(&self, other: &Self) -> Comparison {
        if self.resource != other.resource || self.epoch != other.epoch {
            return Comparison::Incomparable;
        }
        match self.value.cmp(&other.value) {
            std::cmp::Ordering::Equal => Comparison::Equal,
            std::cmp::Ordering::Greater => Comparison::Newer,
            std::cmp::Ordering::Less => Comparison::Older,
        }
    }
}

/// Version of the stamp algorithm; a stamp of another version is never read as "unchanged".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StampAlgorithm(u8);

impl StampAlgorithm {
    /// Version 1: dependency-free 128-bit mixing over the sorted `(CellKey, revision)` pairs.
    pub const V1: Self = Self(1);

    /// The version number.
    pub const fn version(self) -> u8 {
        self.0
    }
}

/// Outcome of comparing two stamps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StampCompare {
    /// Same algorithm, count and hash: *probably* the same set of revisions (a cache hint only;
    /// authoritative checks verify per-cell revisions exactly, policy P-3).
    ProbablySame,
    /// Same algorithm and the sets are certainly different.
    Different,
    /// Different algorithm versions: no statement can be made; treat as changed.
    Incomparable,
}

/// Summary of the revisions of a set of bricks (contract C-16, DEC-062).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RegionStamp {
    algorithm: StampAlgorithm,
    count: u64,
    hash: u128,
}

const SEED_A: u64 = 0x243F_6A88_85A3_08D3;
const SEED_B: u64 = 0x1319_8A2E_0370_7344;

fn mix(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

struct Lanes {
    a: u64,
    b: u64,
}

impl Lanes {
    fn absorb(&mut self, word: u64) {
        self.a = mix(self.a ^ word).wrapping_add(SEED_A);
        self.b = mix(self.b.rotate_left(23) ^ word.rotate_left(17)).wrapping_add(SEED_B);
    }
}

impl RegionStamp {
    /// Builds a stamp from `(cell, revision)` pairs in any order. The pairs are put into canonical
    /// order (by `CellKey`); the same cell twice is an error.
    pub fn from_pairs(pairs: &[(CellKey, Revision)]) -> Result<Self, CoreError> {
        let mut v: Vec<(CellKey, Revision)> = pairs.to_vec();
        v.sort_by_key(|(k, _)| *k);
        if v.windows(2).any(|w| w[0].0 == w[1].0) {
            return Err(CoreError::DuplicateKey);
        }
        let mut l = Lanes {
            a: SEED_A,
            b: SEED_B,
        };
        l.absorb(u64::from(StampAlgorithm::V1.version()));
        l.absorb(v.len() as u64);
        for (k, rev) in &v {
            l.absorb(k.layer().raw());
            l.absorb(u64::from(k.level() as u8));
            let b = k.brick();
            l.absorb(b.x as u64);
            l.absorb(b.y as u64);
            l.absorb(b.z as u64);
            l.absorb(rev.raw());
        }
        Ok(Self {
            algorithm: StampAlgorithm::V1,
            count: v.len() as u64,
            hash: (u128::from(mix(l.a)) << 64) | u128::from(mix(l.b)),
        })
    }

    /// Algorithm version of this stamp.
    pub const fn algorithm(&self) -> StampAlgorithm {
        self.algorithm
    }
    /// Number of bricks summarised.
    pub const fn count(&self) -> u64 {
        self.count
    }
    /// The 128-bit hash value.
    pub const fn hash(&self) -> u128 {
        self.hash
    }

    /// Compares two stamps (see [`StampCompare`]).
    pub fn compare(&self, other: &Self) -> StampCompare {
        if self.algorithm != other.algorithm {
            StampCompare::Incomparable
        } else if self.count == other.count && self.hash == other.hash {
            StampCompare::ProbablySame
        } else {
            StampCompare::Different
        }
    }

    /// Exact check used by authoritative paths: both sets, canonicalised, are identical.
    pub fn same_revisions_exactly(a: &[(CellKey, Revision)], b: &[(CellKey, Revision)]) -> bool {
        let canon = |s: &[(CellKey, Revision)]| {
            let mut v = s.to_vec();
            v.sort_by_key(|(k, _)| *k);
            v
        };
        canon(a) == canon(b)
    }

    #[cfg(test)]
    pub(crate) fn with_algorithm(mut self, v: u8) -> Self {
        self.algorithm = StampAlgorithm(v);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coord::BrickCoord;

    fn key(l: u64, lv: i8, x: i64, y: i64, z: i64) -> CellKey {
        CellKey::new(LayerId::new(l), lv, BrickCoord::new(x, y, z)).unwrap()
    }
    fn tok(res: ResourceId, e: u32, v: u64) -> RevisionToken {
        RevisionToken::new(res, Epoch::new(e), Revision::new(v))
    }

    #[test]
    fn revision_never_wraps() {
        assert_eq!(
            Revision::new(u64::MAX - 1).next(),
            Ok(Revision::new(u64::MAX))
        );
        assert_eq!(
            Revision::new(u64::MAX).next(),
            Err(CoreError::RevisionOverflow)
        );
        let t = tok(ResourceId::Asset(AssetId::new(1)), 0, u64::MAX);
        assert_eq!(t.bumped(), Err(CoreError::RevisionOverflow));
        assert_eq!(
            Epoch::new(u32::MAX - 2).next(),
            Ok(Epoch::new(u32::MAX - 1))
        );
        assert_eq!(
            Epoch::new(u32::MAX - 1).next(),
            Err(CoreError::RevisionOverflow),
            "never reaches the prediction value"
        );
        assert_eq!(
            Epoch::LOCAL_PREDICTION.next(),
            Err(CoreError::RevisionOverflow)
        );
    }

    #[test]
    fn same_resource_orders_by_value() {
        let r = ResourceId::Layer(LayerId::new(2));
        assert_eq!(tok(r, 0, 5).compare(&tok(r, 0, 5)), Comparison::Equal);
        assert_eq!(tok(r, 0, 6).compare(&tok(r, 0, 5)), Comparison::Newer);
        assert_eq!(tok(r, 0, 4).compare(&tok(r, 0, 5)), Comparison::Older);
        assert_eq!(tok(r, 0, 5).bumped().unwrap().value(), Revision::new(6));
    }

    #[test]
    fn different_resource_or_epoch_is_incomparable() {
        let a = ResourceId::Asset(AssetId::new(1));
        let l = ResourceId::Layer(LayerId::new(1));
        assert_eq!(
            tok(a, 0, 5).compare(&tok(l, 0, 5)),
            Comparison::Incomparable,
            "scopes are never comparable"
        );
        assert_eq!(
            tok(a, 0, 5).compare(&tok(ResourceId::Asset(AssetId::new(2)), 0, 5)),
            Comparison::Incomparable
        );
        assert_eq!(
            tok(a, 0, 9).compare(&tok(a, 1, 1)),
            Comparison::Incomparable
        );
        let predicted = RevisionToken::new(a, Epoch::LOCAL_PREDICTION, Revision::new(5));
        assert_eq!(predicted.compare(&tok(a, 0, 5)), Comparison::Incomparable);
        assert_eq!(predicted.compare(&predicted), Comparison::Equal);
        let c = ResourceId::Cell(key(1, 0, 0, 0, 0));
        assert_eq!(
            tok(c, 0, 1).compare(&tok(ResourceId::Cell(key(1, 0, 0, 0, 1)), 0, 1)),
            Comparison::Incomparable
        );
    }

    #[test]
    fn stamp_is_independent_of_input_order_and_rejects_duplicates() {
        let p = [
            (key(1, 0, 0, 0, 0), Revision::new(3)),
            (key(1, 0, -1, 0, 0), Revision::new(7)),
            (key(2, 1, 5, 5, 5), Revision::new(0)),
        ];
        let mut q = p;
        q.reverse();
        assert_eq!(
            RegionStamp::from_pairs(&p).unwrap(),
            RegionStamp::from_pairs(&q).unwrap()
        );
        let dup = [p[0], (p[0].0, Revision::new(9))];
        assert_eq!(RegionStamp::from_pairs(&dup), Err(CoreError::DuplicateKey));
    }

    #[test]
    fn stamp_changes_with_any_revision_key_or_count() {
        let base = [
            (key(1, 0, 0, 0, 0), Revision::new(3)),
            (key(1, 0, 1, 0, 0), Revision::new(7)),
        ];
        let s = RegionStamp::from_pairs(&base).unwrap();
        let rev = [(base[0].0, Revision::new(4)), base[1]];
        let moved = [(key(1, 0, 0, 0, 1), Revision::new(3)), base[1]];
        let fewer = [base[0]];
        let level = [(key(1, 1, 0, 0, 0), Revision::new(3)), base[1]];
        for other in [&rev[..], &moved[..], &fewer[..], &level[..]] {
            assert_eq!(
                s.compare(&RegionStamp::from_pairs(other).unwrap()),
                StampCompare::Different
            );
        }
        assert_eq!(
            s.compare(&RegionStamp::from_pairs(&base).unwrap()),
            StampCompare::ProbablySame
        );
        assert_eq!(s.count(), 2);
    }

    #[test]
    fn empty_stamp_is_valid_and_distinct_from_a_single_cell() {
        let e = RegionStamp::from_pairs(&[]).unwrap();
        assert_eq!(e.count(), 0);
        let one = RegionStamp::from_pairs(&[(key(0, 0, 0, 0, 0), Revision::INITIAL)]).unwrap();
        assert_eq!(e.compare(&one), StampCompare::Different);
    }

    #[test]
    fn other_algorithm_versions_are_incomparable_never_unchanged() {
        let s = RegionStamp::from_pairs(&[(key(1, 0, 0, 0, 0), Revision::new(1))]).unwrap();
        assert_eq!(s.compare(&s.with_algorithm(2)), StampCompare::Incomparable);
    }

    #[test]
    fn exact_verification_agrees_with_canonical_order() {
        let a = [
            (key(1, 0, 0, 0, 0), Revision::new(1)),
            (key(1, 0, 1, 0, 0), Revision::new(2)),
        ];
        let b = [a[1], a[0]];
        assert!(RegionStamp::same_revisions_exactly(&a, &b));
        assert!(!RegionStamp::same_revisions_exactly(&a, &[a[0]]));
    }

    #[test]
    fn golden_value_pins_algorithm_v1() {
        // If this fails the stamp algorithm changed: that needs a new StampAlgorithm version.
        let s = RegionStamp::from_pairs(&[
            (key(1, 0, 0, 0, 0), Revision::new(3)),
            (key(1, 0, -1, 2, -3), Revision::new(7)),
        ])
        .unwrap();
        assert_eq!(s.hash(), GOLDEN_V1);
    }

    const GOLDEN_V1: u128 = 165_256_310_144_090_171_886_825_555_616_110_280_119;
}
