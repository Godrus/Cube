//! Integer addressing (contract C-07, Provisional PC-1, DEC-064).
//!
//! Voxel and brick coordinates are `i64`. A brick has 16 voxels per axis. Negative coordinates use
//! Euclidean (floor) division, so `-1` belongs to brick `-1`, local `15`. The local index order
//! inside a brick is private to Core and is not exposed here.

use crate::error::CoreError;

/// Base brick edge in voxels (a power of two).
pub const BRICK_EDGE: i64 = 16;
/// `log2(BRICK_EDGE)`.
pub const BRICK_EDGE_LOG2: u32 = 4;
/// Voxels per brick.
pub const BRICK_VOLUME: usize = 4096;
/// Largest shift that is defined for an `i64` coordinate conversion to a finer level.
const MAX_LEVEL_SHIFT: u32 = 62;

/// Validates a refinement level: level 0 is the base resolution, positive levels are finer by a
/// factor of `2^level` per axis, negative levels are reserved (DEC-064).
pub const fn check_level(level: i8) -> Result<(), CoreError> {
    if level < 0 {
        Err(CoreError::InvalidLevel(level))
    } else {
        Ok(())
    }
}

/// Position of a voxel in the voxel grid of some level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VoxelPos {
    /// X coordinate.
    pub x: i64,
    /// Y coordinate.
    pub y: i64,
    /// Z coordinate.
    pub z: i64,
}

/// Position of a brick in the brick grid of some level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BrickCoord {
    /// X coordinate.
    pub x: i64,
    /// Y coordinate.
    pub y: i64,
    /// Z coordinate.
    pub z: i64,
}

/// Position of a voxel inside its brick; every component is in `0..16`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LocalPos {
    x: u8,
    y: u8,
    z: u8,
}

impl LocalPos {
    /// Creates a local position; fails if a component is not in `0..16`.
    pub const fn new(x: u8, y: u8, z: u8) -> Result<Self, CoreError> {
        if x < 16 && y < 16 && z < 16 {
            Ok(Self { x, y, z })
        } else {
            Err(CoreError::InvalidLocalCoord)
        }
    }
    /// X component.
    pub const fn x(self) -> u8 {
        self.x
    }
    /// Y component.
    pub const fn y(self) -> u8 {
        self.y
    }
    /// Z component.
    pub const fn z(self) -> u8 {
        self.z
    }
}

impl VoxelPos {
    /// Creates a position.
    pub const fn new(x: i64, y: i64, z: i64) -> Self {
        Self { x, y, z }
    }

    /// Splits into the containing brick and the local position (Euclidean division; never overflows).
    pub const fn split(self) -> (BrickCoord, LocalPos) {
        let b = BrickCoord {
            x: self.x.div_euclid(BRICK_EDGE),
            y: self.y.div_euclid(BRICK_EDGE),
            z: self.z.div_euclid(BRICK_EDGE),
        };
        // rem_euclid(16) is always in 0..16, so the narrowing is lossless.
        let l = LocalPos {
            x: self.x.rem_euclid(BRICK_EDGE) as u8,
            y: self.y.rem_euclid(BRICK_EDGE) as u8,
            z: self.z.rem_euclid(BRICK_EDGE) as u8,
        };
        (b, l)
    }

    /// Brick containing this voxel.
    pub const fn brick(self) -> BrickCoord {
        self.split().0
    }

    /// Converts the position from the voxel grid of level `from` to that of level `to`.
    ///
    /// Going finer maps to the lowest sub-voxel (exact, may overflow); going coarser floors.
    /// So `coarse -> fine -> coarse` is the identity, while `fine -> coarse -> fine` is not.
    pub fn convert_level(self, from: i8, to: i8) -> Result<Self, CoreError> {
        check_level(from)?;
        check_level(to)?;
        if to >= from {
            let d = u32::try_from(to - from).map_err(|_| CoreError::Overflow)?;
            if d > MAX_LEVEL_SHIFT {
                return Err(CoreError::Overflow);
            }
            let f = 1i64 << d;
            Ok(Self {
                x: self.x.checked_mul(f).ok_or(CoreError::Overflow)?,
                y: self.y.checked_mul(f).ok_or(CoreError::Overflow)?,
                z: self.z.checked_mul(f).ok_or(CoreError::Overflow)?,
            })
        } else {
            let d = u32::try_from(from - to)
                .map_err(|_| CoreError::Overflow)?
                .min(63);
            // Arithmetic shift right is floor division by 2^d, also for negative numbers.
            Ok(Self {
                x: self.x >> d,
                y: self.y >> d,
                z: self.z >> d,
            })
        }
    }
}

impl BrickCoord {
    /// Creates a brick coordinate.
    pub const fn new(x: i64, y: i64, z: i64) -> Self {
        Self { x, y, z }
    }

    /// Voxel position of the lowest corner of this brick; fails if it is not representable.
    pub fn origin(self) -> Result<VoxelPos, CoreError> {
        self.join(LocalPos { x: 0, y: 0, z: 0 })
    }

    /// Voxel position of local voxel `l` of this brick; fails if it is not representable.
    pub fn join(self, l: LocalPos) -> Result<VoxelPos, CoreError> {
        let one = |b: i64, l: u8| {
            b.checked_mul(BRICK_EDGE)
                .and_then(|v| v.checked_add(i64::from(l)))
                .ok_or(CoreError::Overflow)
        };
        Ok(VoxelPos {
            x: one(self.x, l.x)?,
            y: one(self.y, l.y)?,
            z: one(self.z, l.z)?,
        })
    }

    /// `true` if every voxel of this brick has a representable position.
    pub fn is_addressable(self) -> bool {
        self.origin().is_ok()
            && self
                .join(LocalPos {
                    x: 15,
                    y: 15,
                    z: 15,
                })
                .is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negative_coordinates_use_floor_division() {
        let (b, l) = VoxelPos::new(-1, -16, -17).split();
        assert_eq!(b, BrickCoord::new(-1, -1, -2));
        assert_eq!((l.x(), l.y(), l.z()), (15, 0, 15));
        let (b, l) = VoxelPos::new(0, 15, 16).split();
        assert_eq!(b, BrickCoord::new(0, 0, 1));
        assert_eq!((l.x(), l.y(), l.z()), (0, 15, 0));
    }

    #[test]
    fn split_handles_the_extremes_without_overflow() {
        let (b, l) = VoxelPos::new(i64::MIN, i64::MAX, 0).split();
        assert_eq!(b.x, i64::MIN / 16);
        assert_eq!(l.x(), 0);
        assert_eq!(b.y, i64::MAX / 16);
        assert_eq!(l.y(), 15);
    }

    #[test]
    fn join_is_the_inverse_of_split_at_the_extremes() {
        for v in [
            i64::MIN,
            i64::MIN + 1,
            -17,
            -16,
            -1,
            0,
            1,
            15,
            16,
            i64::MAX - 1,
            i64::MAX,
        ] {
            let p = VoxelPos::new(v, v, v);
            let (b, l) = p.split();
            assert_eq!(b.join(l), Ok(p), "v = {v}");
        }
    }

    #[test]
    fn join_reports_overflow_instead_of_wrapping() {
        let far = BrickCoord::new(i64::MAX, 0, 0);
        assert_eq!(far.origin(), Err(CoreError::Overflow));
        assert!(!far.is_addressable());
        let edge = BrickCoord::new(i64::MAX / 16, i64::MIN / 16, 0);
        assert!(edge.is_addressable());
        let beyond = BrickCoord::new(i64::MAX / 16 + 1, 0, 0);
        assert!(!beyond.is_addressable());
    }

    #[test]
    fn local_pos_rejects_out_of_range() {
        assert!(LocalPos::new(15, 15, 15).is_ok());
        assert_eq!(LocalPos::new(16, 0, 0), Err(CoreError::InvalidLocalCoord));
        assert_eq!(LocalPos::new(0, 255, 0), Err(CoreError::InvalidLocalCoord));
    }

    #[test]
    fn level_conversion_floors_and_round_trips_upward() {
        let p = VoxelPos::new(-3, 5, 0);
        let fine = p.convert_level(0, 2).unwrap();
        assert_eq!(fine, VoxelPos::new(-12, 20, 0));
        assert_eq!(fine.convert_level(2, 0), Ok(p));
        // finer: multiply; coarser: floor (also for negatives).
        assert_eq!(
            VoxelPos::new(-1, 1, 2).convert_level(0, 1),
            Ok(VoxelPos::new(-2, 2, 4))
        );
        assert_eq!(
            VoxelPos::new(-1, 1, 2).convert_level(1, 0),
            Ok(VoxelPos::new(-1, 0, 1))
        );
        assert_eq!(
            VoxelPos::new(-2, -3, 3).convert_level(1, 0),
            Ok(VoxelPos::new(-1, -2, 1))
        );
        assert_eq!(p.convert_level(3, 3), Ok(p));
    }

    #[test]
    fn level_conversion_rejects_reserved_levels_and_overflow() {
        let p = VoxelPos::new(1, 0, 0);
        assert_eq!(p.convert_level(-1, 0), Err(CoreError::InvalidLevel(-1)));
        assert_eq!(p.convert_level(0, -5), Err(CoreError::InvalidLevel(-5)));
        assert_eq!(p.convert_level(0, 63), Err(CoreError::Overflow));
        assert_eq!(p.convert_level(0, 127), Err(CoreError::Overflow));
        assert_eq!(
            VoxelPos::new(i64::MAX, 0, 0).convert_level(0, 1),
            Err(CoreError::Overflow)
        );
        assert_eq!(p.convert_level(0, 62), Ok(VoxelPos::new(1 << 62, 0, 0)));
        // Coarsening by the largest possible difference is defined.
        assert_eq!(
            VoxelPos::new(i64::MIN, -1, 5).convert_level(127, 0),
            Ok(VoxelPos::new(-1, -1, 0))
        );
    }
}
