//! Axis-aligned half-open box `[min, max)` in the voxel grid of one level (contract C-06).

use crate::coord::{BrickCoord, VoxelPos, check_level};
use crate::error::CoreError;

/// Half-open box of voxels at a stated level. Empty when `min == max` on any axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Region {
    level: i8,
    min: VoxelPos,
    max: VoxelPos,
}

impl Region {
    /// Creates a region; fails for a reserved level or if `min > max` on some axis.
    pub fn new(level: i8, min: VoxelPos, max: VoxelPos) -> Result<Self, CoreError> {
        check_level(level)?;
        if min.x > max.x || min.y > max.y || min.z > max.z {
            return Err(CoreError::InvalidRegion);
        }
        Ok(Self { level, min, max })
    }

    /// Level of the voxel grid the coordinates refer to.
    pub const fn level(&self) -> i8 {
        self.level
    }
    /// Inclusive lower corner.
    pub const fn min(&self) -> VoxelPos {
        self.min
    }
    /// Exclusive upper corner.
    pub const fn max(&self) -> VoxelPos {
        self.max
    }

    /// `true` if the region contains no voxel.
    pub const fn is_empty(&self) -> bool {
        self.min.x == self.max.x || self.min.y == self.max.y || self.min.z == self.max.z
    }

    /// `true` if `p` lies inside (the caller guarantees `p` is at the same level).
    pub const fn contains(&self, p: VoxelPos) -> bool {
        p.x >= self.min.x
            && p.x < self.max.x
            && p.y >= self.min.y
            && p.y < self.max.y
            && p.z >= self.min.z
            && p.z < self.max.z
    }

    /// Edge lengths in voxels. Always fits `u64`.
    pub fn extent(&self) -> [u64; 3] {
        let d = |a: i64, b: i64| (i128::from(b) - i128::from(a)) as u64;
        [
            d(self.min.x, self.max.x),
            d(self.min.y, self.max.y),
            d(self.min.z, self.max.z),
        ]
    }

    /// Intersection with another region of the same level; `Ok(None)` if they do not overlap.
    /// Fails if the levels differ (convert first).
    pub fn intersection(&self, other: &Self) -> Result<Option<Self>, CoreError> {
        if self.level != other.level {
            return Err(CoreError::InvalidLevel(other.level));
        }
        let min = VoxelPos::new(
            self.min.x.max(other.min.x),
            self.min.y.max(other.min.y),
            self.min.z.max(other.min.z),
        );
        let max = VoxelPos::new(
            self.max.x.min(other.max.x),
            self.max.y.min(other.max.y),
            self.max.z.min(other.max.z),
        );
        if min.x >= max.x || min.y >= max.y || min.z >= max.z {
            return Ok(None);
        }
        Ok(Some(Self {
            level: self.level,
            min,
            max,
        }))
    }

    /// Inclusive range of bricks touched by this region, or `None` if it is empty.
    pub fn brick_span(&self) -> Option<(BrickCoord, BrickCoord)> {
        if self.is_empty() {
            return None;
        }
        // max is exclusive and > min, so max - 1 cannot overflow.
        let last = VoxelPos::new(self.max.x - 1, self.max.y - 1, self.max.z - 1);
        Some((self.min.brick(), last.brick()))
    }

    /// Number of bricks touched, `None` if it does not fit `u128` (or for an empty region: `Some(0)`).
    pub fn brick_count(&self) -> Option<u128> {
        let Some((a, b)) = self.brick_span() else {
            return Some(0);
        };
        let n = |lo: i64, hi: i64| (i128::from(hi) - i128::from(lo) + 1) as u128;
        n(a.x, b.x)
            .checked_mul(n(a.y, b.y))?
            .checked_mul(n(a.z, b.z))
    }

    /// Iterates the touched bricks in lexicographic `(x, y, z)` order (z fastest); deterministic.
    pub fn bricks(&self) -> Bricks {
        match self.brick_span() {
            Some((lo, hi)) => Bricks {
                lo,
                hi,
                next: Some(lo),
            },
            None => Bricks {
                lo: BrickCoord::new(0, 0, 0),
                hi: BrickCoord::new(0, 0, 0),
                next: None,
            },
        }
    }
}

/// Iterator over the bricks of a region.
#[derive(Debug, Clone)]
pub struct Bricks {
    lo: BrickCoord,
    hi: BrickCoord,
    next: Option<BrickCoord>,
}

impl Iterator for Bricks {
    type Item = BrickCoord;

    fn next(&mut self) -> Option<BrickCoord> {
        let cur = self.next?;
        // Bricks are at most i64::MAX / 16 in magnitude when derived from voxels, so +1 cannot overflow.
        self.next = if cur.z < self.hi.z {
            Some(BrickCoord::new(cur.x, cur.y, cur.z + 1))
        } else if cur.y < self.hi.y {
            Some(BrickCoord::new(cur.x, cur.y + 1, self.lo.z))
        } else if cur.x < self.hi.x {
            Some(BrickCoord::new(cur.x + 1, self.lo.y, self.lo.z))
        } else {
            None
        };
        Some(cur)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(min: [i64; 3], max: [i64; 3]) -> Region {
        Region::new(
            0,
            VoxelPos::new(min[0], min[1], min[2]),
            VoxelPos::new(max[0], max[1], max[2]),
        )
        .unwrap()
    }

    #[test]
    fn rejects_inverted_boxes_and_reserved_levels() {
        assert_eq!(
            Region::new(0, VoxelPos::new(1, 0, 0), VoxelPos::new(0, 5, 5)),
            Err(CoreError::InvalidRegion)
        );
        assert_eq!(
            Region::new(-1, VoxelPos::new(0, 0, 0), VoxelPos::new(1, 1, 1)),
            Err(CoreError::InvalidLevel(-1))
        );
    }

    #[test]
    fn half_open_containment_and_emptiness() {
        let a = r([-2, 0, 0], [2, 1, 1]);
        assert!(a.contains(VoxelPos::new(-2, 0, 0)) && a.contains(VoxelPos::new(1, 0, 0)));
        assert!(!a.contains(VoxelPos::new(2, 0, 0)) && !a.contains(VoxelPos::new(0, 1, 0)));
        assert!(r([3, 3, 3], [3, 9, 9]).is_empty());
        assert!(!a.is_empty());
        assert_eq!(a.extent(), [4, 1, 1]);
    }

    #[test]
    fn extent_of_the_full_range_fits() {
        let all = r(
            [i64::MIN, i64::MIN, i64::MIN],
            [i64::MAX, i64::MAX, i64::MAX],
        );
        assert_eq!(all.extent(), [u64::MAX; 3]);
    }

    #[test]
    fn intersection_cases() {
        let a = r([0, 0, 0], [10, 10, 10]);
        assert_eq!(
            a.intersection(&r([5, -5, 0], [20, 5, 3])).unwrap(),
            Some(r([5, 0, 0], [10, 5, 3]))
        );
        assert_eq!(
            a.intersection(&r([10, 0, 0], [20, 10, 10])).unwrap(),
            None,
            "touching boxes do not overlap"
        );
        let other_level = Region::new(1, VoxelPos::new(0, 0, 0), VoxelPos::new(1, 1, 1)).unwrap();
        assert_eq!(
            a.intersection(&other_level),
            Err(CoreError::InvalidLevel(1))
        );
    }

    #[test]
    fn brick_span_crosses_zero_correctly() {
        let a = r([-1, 0, 15], [1, 16, 17]);
        assert_eq!(
            a.brick_span(),
            Some((BrickCoord::new(-1, 0, 0), BrickCoord::new(0, 0, 1)))
        );
        assert_eq!(a.brick_count(), Some(4));
        let v: Vec<_> = a.bricks().collect();
        assert_eq!(
            v,
            vec![
                BrickCoord::new(-1, 0, 0),
                BrickCoord::new(-1, 0, 1),
                BrickCoord::new(0, 0, 0),
                BrickCoord::new(0, 0, 1)
            ]
        );
    }

    #[test]
    fn empty_region_has_no_bricks() {
        let e = r([4, 4, 4], [4, 8, 8]);
        assert_eq!(e.brick_span(), None);
        assert_eq!(e.brick_count(), Some(0));
        assert_eq!(e.bricks().count(), 0);
    }

    #[test]
    fn extreme_regions_do_not_overflow() {
        let all = r([i64::MIN, 0, 0], [i64::MAX, 1, 1]);
        let (lo, hi) = all.brick_span().unwrap();
        assert_eq!((lo.x, hi.x), (i64::MIN / 16, i64::MAX / 16));
        assert!(all.brick_count().is_some());
        let huge = r([i64::MIN; 3], [i64::MAX; 3]);
        assert_eq!(huge.brick_count(), None, "three full axes exceed u128");
        // The iterator must not be advanced over that range; only its first items are checked.
        assert_eq!(
            huge.bricks().next(),
            Some(BrickCoord::new(i64::MIN / 16, i64::MIN / 16, i64::MIN / 16))
        );
    }
}
