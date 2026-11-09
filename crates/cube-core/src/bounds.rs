//! Spatial extent of geometry (CORE-15, contract C-05 GeometryBounds, Provisional: pending GOV-16).
//!
//! Bounds are integer boxes in the voxel grid of one level: the primary form is the layer-local
//! box, and applying a layer transform yields the world box. Because transforms are limited to the
//! 24 axis rotations, a power-of-two scale and an integer translation (C-04), the transformed box
//! is again an exact axis-aligned box, so no separate OBB or bounding-sphere type is needed.
//!
//! The model is the half-open box `[min, max)` of [`Region`]; `Empty` is a distinct state, not a
//! degenerate box. Every operation is exact integer arithmetic: overflow is an error
//! ([`CoreError::Overflow`]), never a wrap or a clamp. Core imposes no world-size limit; the
//! representable range is the `i64` voxel grid.
//!
//! "Tight versus conservative" (open in C-05) is decided here for growth only: operations never
//! shrink a box, so a bound may be larger than the data but never smaller.

use crate::coord::{BRICK_EDGE, BrickCoord, VoxelPos, check_level};
use crate::error::CoreError;
use crate::region::Region;
use crate::transform::GeometryTransform;

/// A bound on the extent of some geometry at one refinement level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GeometryBounds {
    /// Contains no voxel (the identity of [`GeometryBounds::union`]).
    Empty,
    /// A non-empty half-open box. Constructed only through this module, so it is never degenerate.
    Box(Region),
}

impl GeometryBounds {
    /// Bounds of a region; an empty region gives `Empty`.
    pub const fn from_region(region: Region) -> Self {
        if region.is_empty() {
            Self::Empty
        } else {
            Self::Box(region)
        }
    }

    /// Bounds of the voxels of one brick at `level`. Fails if the brick is not addressable.
    pub fn from_brick(level: i8, brick: BrickCoord) -> Result<Self, CoreError> {
        Self::from_brick_range(level, brick, brick)
    }

    /// Bounds of all bricks in the inclusive range `lo..=hi` at `level`.
    pub fn from_brick_range(level: i8, lo: BrickCoord, hi: BrickCoord) -> Result<Self, CoreError> {
        check_level(level)?;
        if lo.x > hi.x || lo.y > hi.y || lo.z > hi.z {
            return Err(CoreError::InvalidRegion);
        }
        let min = lo.origin()?;
        let top = hi.origin()?;
        let up = |v: i64| v.checked_add(BRICK_EDGE).ok_or(CoreError::Overflow);
        let max = VoxelPos::new(up(top.x)?, up(top.y)?, up(top.z)?);
        Ok(Self::Box(Region::new(level, min, max)?))
    }

    /// `true` for `Empty`.
    pub const fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    /// The box, `None` for `Empty`.
    pub const fn region(&self) -> Option<Region> {
        match self {
            Self::Empty => None,
            Self::Box(r) => Some(*r),
        }
    }

    /// Level of the box, `None` for `Empty` (which has no level).
    pub const fn level(&self) -> Option<i8> {
        match self {
            Self::Empty => None,
            Self::Box(r) => Some(r.level()),
        }
    }

    /// `true` if the voxel `p` (at the box's level) lies inside.
    pub const fn contains_voxel(&self, p: VoxelPos) -> bool {
        match self {
            Self::Empty => false,
            Self::Box(r) => r.contains(p),
        }
    }

    /// `true` if every voxel of `other` lies inside `self`. `Empty` is contained in everything;
    /// a non-empty box is never contained in `Empty`; boxes of different levels are not comparable
    /// and give `false` (convert with [`GeometryBounds::to_level`] first).
    pub fn contains(&self, other: &Self) -> bool {
        match (self, other) {
            (_, Self::Empty) => true,
            (Self::Empty, _) => false,
            (Self::Box(a), Self::Box(b)) => {
                a.level() == b.level()
                    && a.min().x <= b.min().x
                    && a.min().y <= b.min().y
                    && a.min().z <= b.min().z
                    && a.max().x >= b.max().x
                    && a.max().y >= b.max().y
                    && a.max().z >= b.max().z
            }
        }
    }

    /// Smallest box containing both. `Empty` is the identity. Different levels are an error.
    pub fn union(&self, other: &Self) -> Result<Self, CoreError> {
        match (self, other) {
            (Self::Empty, o) | (o, Self::Empty) => Ok(*o),
            (Self::Box(a), Self::Box(b)) => {
                if a.level() != b.level() {
                    return Err(CoreError::InvalidLevel(b.level()));
                }
                let min = VoxelPos::new(
                    a.min().x.min(b.min().x),
                    a.min().y.min(b.min().y),
                    a.min().z.min(b.min().z),
                );
                let max = VoxelPos::new(
                    a.max().x.max(b.max().x),
                    a.max().y.max(b.max().y),
                    a.max().z.max(b.max().z),
                );
                Ok(Self::Box(Region::new(a.level(), min, max)?))
            }
        }
    }

    /// Overlap of both; `Empty` if they do not overlap. Different levels are an error.
    pub fn intersection(&self, other: &Self) -> Result<Self, CoreError> {
        match (self, other) {
            (Self::Empty, _) | (_, Self::Empty) => Ok(Self::Empty),
            (Self::Box(a), Self::Box(b)) => Ok(a.intersection(b)?.map_or(Self::Empty, Self::Box)),
        }
    }

    /// The box grown by `by` voxels on every side (conservative growth, e.g. for an apron).
    /// `Empty` stays `Empty`. Fails with `Overflow` if a corner leaves the `i64` range.
    pub fn grow(&self, by: u64) -> Result<Self, CoreError> {
        let Self::Box(r) = self else {
            return Ok(Self::Empty);
        };
        let by = i64::try_from(by).map_err(|_| CoreError::Overflow)?;
        let sub = |v: i64| v.checked_sub(by).ok_or(CoreError::Overflow);
        let add = |v: i64| v.checked_add(by).ok_or(CoreError::Overflow);
        let min = VoxelPos::new(sub(r.min().x)?, sub(r.min().y)?, sub(r.min().z)?);
        let max = VoxelPos::new(add(r.max().x)?, add(r.max().y)?, add(r.max().z)?);
        Ok(Self::Box(Region::new(r.level(), min, max)?))
    }

    /// The box translated by `delta` voxels. Fails with `Overflow` if a corner leaves the range.
    pub fn translate(&self, delta: [i64; 3]) -> Result<Self, CoreError> {
        let Self::Box(r) = self else {
            return Ok(Self::Empty);
        };
        let mv = |p: VoxelPos| -> Result<VoxelPos, CoreError> {
            let f = |a: i64, d: i64| a.checked_add(d).ok_or(CoreError::Overflow);
            Ok(VoxelPos::new(
                f(p.x, delta[0])?,
                f(p.y, delta[1])?,
                f(p.z, delta[2])?,
            ))
        };
        Ok(Self::Box(Region::new(
            r.level(),
            mv(r.min())?,
            mv(r.max())?,
        )?))
    }

    /// The same extent expressed at `level`. Going finer scales the corners (may overflow); going
    /// coarser rounds outward (min floors, max ceils), so the result always contains the original
    /// extent and never shrinks.
    pub fn to_level(&self, level: i8) -> Result<Self, CoreError> {
        check_level(level)?;
        let Self::Box(r) = self else {
            return Ok(Self::Empty);
        };
        let from = r.level();
        if level >= from {
            let min = r.min().convert_level(from, level)?;
            let max = r.max().convert_level(from, level)?;
            return Ok(Self::Box(Region::new(level, min, max)?));
        }
        let d = u32::from((from - level).unsigned_abs()).min(63);
        let floor = |v: i64| v >> d;
        // ceil(v / 2^d) as floor((v - 1) / 2^d) + 1; `r` is non-empty so `v - 1` cannot underflow.
        let ceil = |v: i64| ((v - 1) >> d) + 1;
        let min = VoxelPos::new(floor(r.min().x), floor(r.min().y), floor(r.min().z));
        let max = VoxelPos::new(ceil(r.max().x), ceil(r.max().y), ceil(r.max().z));
        Ok(Self::Box(Region::new(level, min, max)?))
    }

    /// Applies a layer transform (rotate, scale by `2^scale_shift`, translate), giving the box in
    /// the parent space. The box is treated as a continuous interval of the lattice, so the result
    /// is exact for the supported transforms: both corners go through [`GeometryTransform::apply`]
    /// and each axis takes the smaller and larger value. Fails with `Overflow`, or
    /// `InvalidTransform` for a negative scale shift (as `GeometryTransform::apply` does).
    pub fn transform(&self, t: &GeometryTransform) -> Result<Self, CoreError> {
        let Self::Box(r) = self else {
            return Ok(Self::Empty);
        };
        let a = t.apply([r.min().x, r.min().y, r.min().z])?;
        let b = t.apply([r.max().x, r.max().y, r.max().z])?;
        let lo = VoxelPos::new(a[0].min(b[0]), a[1].min(b[1]), a[2].min(b[2]));
        let hi = VoxelPos::new(a[0].max(b[0]), a[1].max(b[1]), a[2].max(b[2]));
        Ok(Self::Box(Region::new(r.level(), lo, hi)?))
    }

    /// Inclusive range of bricks that the box touches, `None` for `Empty`.
    pub fn brick_span(&self) -> Option<(BrickCoord, BrickCoord)> {
        self.region().and_then(|r| r.brick_span())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transform::Rotation;
    use cube_testkit::{DetRng, for_each_seed};

    fn b(level: i8, min: [i64; 3], max: [i64; 3]) -> GeometryBounds {
        GeometryBounds::from_region(
            Region::new(
                level,
                VoxelPos::new(min[0], min[1], min[2]),
                VoxelPos::new(max[0], max[1], max[2]),
            )
            .unwrap(),
        )
    }

    fn rand_box(rng: &mut DetRng, level: i8) -> GeometryBounds {
        let mut lo = [0i64; 3];
        let mut hi = [0i64; 3];
        for i in 0..3 {
            lo[i] = rng.range_i64(-100, 100);
            hi[i] = lo[i] + rng.range_i64(1, 40);
        }
        b(level, lo, hi)
    }

    #[test]
    fn empty_region_is_the_empty_state() {
        assert!(b(0, [1, 1, 1], [1, 5, 5]).is_empty());
        assert!(!b(0, [0, 0, 0], [1, 1, 1]).is_empty());
        assert_eq!(GeometryBounds::Empty.level(), None);
    }

    #[test]
    fn brick_bounds_handle_negative_coordinates() {
        let g = GeometryBounds::from_brick(0, BrickCoord::new(-1, 0, 0)).unwrap();
        assert_eq!(g, b(0, [-16, 0, 0], [0, 16, 16]));
        let g = GeometryBounds::from_brick_range(
            0,
            BrickCoord::new(-2, -1, 0),
            BrickCoord::new(0, 0, 0),
        )
        .unwrap();
        assert_eq!(g, b(0, [-32, -16, 0], [16, 16, 16]));
        assert_eq!(
            g.brick_span(),
            Some((BrickCoord::new(-2, -1, 0), BrickCoord::new(0, 0, 0)))
        );
        assert_eq!(
            GeometryBounds::from_brick_range(0, BrickCoord::new(1, 0, 0), BrickCoord::new(0, 0, 0)),
            Err(CoreError::InvalidRegion)
        );
        assert_eq!(
            GeometryBounds::from_brick(-1, BrickCoord::new(0, 0, 0)),
            Err(CoreError::InvalidLevel(-1))
        );
    }

    #[test]
    fn unaddressable_bricks_overflow_instead_of_wrapping() {
        let far = BrickCoord::new(i64::MAX / 16, 0, 0);
        assert!(GeometryBounds::from_brick(0, far).is_err());
        let near = BrickCoord::new(i64::MAX / 16 - 1, 0, 0);
        assert!(GeometryBounds::from_brick(0, near).is_ok());
        assert_eq!(
            GeometryBounds::from_brick(0, BrickCoord::new(i64::MIN, 0, 0)),
            Err(CoreError::Overflow)
        );
    }

    #[test]
    fn union_and_intersection_have_the_expected_identities() {
        let a = b(0, [0, 0, 0], [4, 4, 4]);
        let e = GeometryBounds::Empty;
        assert_eq!(a.union(&e), Ok(a));
        assert_eq!(e.union(&a), Ok(a));
        assert_eq!(a.intersection(&e), Ok(e));
        assert_eq!(
            a.intersection(&b(0, [4, 0, 0], [8, 4, 4])),
            Ok(e),
            "touching boxes do not overlap"
        );
        assert_eq!(
            a.intersection(&b(0, [2, 2, 2], [9, 9, 9])),
            Ok(b(0, [2, 2, 2], [4, 4, 4]))
        );
        let other_level = b(1, [0, 0, 0], [1, 1, 1]);
        assert_eq!(a.union(&other_level), Err(CoreError::InvalidLevel(1)));
        assert_eq!(
            a.intersection(&other_level),
            Err(CoreError::InvalidLevel(1))
        );
    }

    #[test]
    fn extreme_boxes_union_without_arithmetic_overflow() {
        let a = b(0, [i64::MIN, 0, 0], [i64::MIN + 1, 1, 1]);
        let c = b(0, [i64::MAX - 1, 0, 0], [i64::MAX, 1, 1]);
        assert_eq!(a.union(&c), Ok(b(0, [i64::MIN, 0, 0], [i64::MAX, 1, 1])));
    }

    #[test]
    fn grow_is_conservative_and_checks_overflow() {
        let a = b(0, [0, 0, 0], [2, 2, 2]);
        assert_eq!(a.grow(1), Ok(b(0, [-1, -1, -1], [3, 3, 3])));
        assert_eq!(a.grow(0), Ok(a));
        assert_eq!(GeometryBounds::Empty.grow(5), Ok(GeometryBounds::Empty));
        let top = b(0, [0, 0, 0], [i64::MAX, 1, 1]);
        assert_eq!(top.grow(1), Err(CoreError::Overflow));
        assert_eq!(a.grow(u64::MAX), Err(CoreError::Overflow));
        let bottom = b(0, [i64::MIN, 0, 0], [0, 1, 1]);
        assert_eq!(bottom.grow(1), Err(CoreError::Overflow));
    }

    #[test]
    fn coarser_levels_round_outward_for_negative_and_positive_boxes() {
        // Level 1 -> level 0: 2 fine voxels per coarse voxel.
        let a = b(1, [-3, 1, 0], [-1, 4, 1]);
        let c = a.to_level(0).unwrap();
        assert_eq!(c, b(0, [-2, 0, 0], [0, 2, 1]));
        assert!(c.to_level(1).unwrap().contains(&a));
        // Finer: exact scaling.
        assert_eq!(
            b(0, [-1, 0, 0], [1, 1, 1]).to_level(2),
            Ok(b(2, [-4, 0, 0], [4, 4, 4]))
        );
        assert_eq!(a.to_level(-1), Err(CoreError::InvalidLevel(-1)));
        // A finer conversion that does not fit is an error.
        assert_eq!(
            b(0, [0, 0, 0], [i64::MAX, 1, 1]).to_level(5),
            Err(CoreError::Overflow)
        );
    }

    #[test]
    fn transform_matches_point_mapping_and_handles_flips() {
        let a = b(0, [1, 2, 3], [3, 5, 9]);
        let t = GeometryTransform::new([10, 0, -4], Rotation::IDENTITY, 1).unwrap();
        assert_eq!(a.transform(&t), Ok(b(0, [12, 4, 2], [16, 10, 14])));
        // Every rotation preserves the volume and keeps min <= max.
        for i in 0..Rotation::COUNT {
            let t = GeometryTransform::new([0; 3], Rotation::from_index(i).unwrap(), 0).unwrap();
            let w = a.transform(&t).unwrap();
            let r = w.region().unwrap();
            assert_eq!(r.extent().iter().product::<u64>(), 2 * 3 * 6);
        }
        let neg = GeometryTransform::new([0; 3], Rotation::IDENTITY, -1).unwrap();
        assert_eq!(a.transform(&neg), Err(CoreError::InvalidTransform));
        let far = GeometryTransform::new([i64::MAX, 0, 0], Rotation::IDENTITY, 0).unwrap();
        assert_eq!(a.transform(&far), Err(CoreError::Overflow));
    }

    #[test]
    fn property_union_is_commutative_associative_and_contains_both() {
        for_each_seed(0xB0_0001, 200, |seed| {
            let mut rng = DetRng::new(seed);
            let x = rand_box(&mut rng, 0);
            let y = rand_box(&mut rng, 0);
            let z = rand_box(&mut rng, 0);
            assert_eq!(x.union(&y), y.union(&x));
            assert_eq!(
                x.union(&y).unwrap().union(&z),
                x.union(&y.union(&z).unwrap())
            );
            let u = x.union(&y).unwrap();
            assert!(u.contains(&x) && u.contains(&y));
            assert_eq!(x.union(&x), Ok(x));
        });
    }

    #[test]
    fn property_intersection_is_contained_in_both_and_commutative() {
        for_each_seed(0xB0_0002, 200, |seed| {
            let mut rng = DetRng::new(seed);
            let x = rand_box(&mut rng, 0);
            let y = rand_box(&mut rng, 0);
            let i = x.intersection(&y).unwrap();
            assert_eq!(Ok(i), y.intersection(&x));
            assert!(x.contains(&i) && y.contains(&i));
        });
    }

    #[test]
    fn property_grow_only_widens_and_translate_preserves_extent() {
        for_each_seed(0xB0_0003, 200, |seed| {
            let mut rng = DetRng::new(seed);
            let x = rand_box(&mut rng, 0);
            let g = x.grow(rng.below(9)).unwrap();
            assert!(g.contains(&x));
            let d = [
                rng.range_i64(-50, 50),
                rng.range_i64(-50, 50),
                rng.range_i64(-50, 50),
            ];
            let m = x.translate(d).unwrap();
            assert_eq!(m.region().unwrap().extent(), x.region().unwrap().extent());
            assert_eq!(m.translate([-d[0], -d[1], -d[2]]), Ok(x));
        });
    }

    #[test]
    fn property_transform_contains_the_image_of_every_inside_point() {
        for_each_seed(0xB0_0004, 200, |seed| {
            let mut rng = DetRng::new(seed);
            let x = rand_box(&mut rng, 0);
            let rot = Rotation::from_index(rng.below(24) as u8).unwrap();
            let shift = rng.below(4) as i8;
            let t = GeometryTransform::new(
                [
                    rng.range_i64(-99, 99),
                    rng.range_i64(-99, 99),
                    rng.range_i64(-99, 99),
                ],
                rot,
                shift,
            )
            .unwrap();
            let w = x.transform(&t).unwrap().region().unwrap();
            let r = x.region().unwrap();
            for _ in 0..20 {
                let p = [
                    rng.range_i64(r.min().x, r.max().x),
                    rng.range_i64(r.min().y, r.max().y),
                    rng.range_i64(r.min().z, r.max().z),
                ];
                let q = t.apply(p).unwrap();
                // The image of a point of the half-open box lies in the closed transformed box.
                let inside = |v: i64, lo: i64, hi: i64| v >= lo && v <= hi;
                assert!(
                    inside(q[0], w.min().x, w.max().x)
                        && inside(q[1], w.min().y, w.max().y)
                        && inside(q[2], w.min().z, w.max().z)
                );
            }
        });
    }

    #[test]
    fn property_to_level_roundtrip_never_shrinks() {
        for_each_seed(0xB0_0005, 200, |seed| {
            let mut rng = DetRng::new(seed);
            let lvl = rng.below(4) as i8;
            let x = rand_box(&mut rng, lvl);
            let coarse = x.to_level(0).unwrap();
            assert!(coarse.to_level(lvl).unwrap().contains(&x));
            // Finer round trip is exact.
            let fine = x.to_level(lvl + 2).unwrap();
            assert_eq!(fine.to_level(lvl), Ok(x));
        });
    }

    #[test]
    fn bounds_agree_with_region_brick_enumeration() {
        let x = b(0, [-17, 0, 0], [17, 1, 1]);
        let (lo, hi) = x.brick_span().unwrap();
        assert_eq!(
            (lo, hi),
            (BrickCoord::new(-2, 0, 0), BrickCoord::new(1, 0, 0))
        );
        let back = GeometryBounds::from_brick_range(0, lo, hi).unwrap();
        assert!(back.contains(&x));
    }

    #[test]
    fn results_are_deterministic() {
        cube_testkit::assert_deterministic(|| {
            let a = b(0, [-5, 0, 3], [9, 9, 9]);
            a.grow(3)
                .unwrap()
                .to_level(2)
                .unwrap()
                .union(&a.to_level(2).unwrap())
        });
    }
}
