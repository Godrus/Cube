//! Integer placement of a layer (contract C-04, decision D-T / DEC-065).
//!
//! Authoritative geometry state contains no floating point: translation is integral, rotation is
//! one of the 24 axis-aligned rotations, scale is a power of two. The continuous pose of a moving
//! body is entity/physics state and never lives here.

use crate::error::CoreError;

/// Largest supported `|scale_shift|`; keeps `2^shift` inside `i64`.
pub const MAX_SCALE_SHIFT: i8 = 62;

/// One of the 24 proper axis-aligned rotations, by index; `0` is the identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rotation(u8);

impl Rotation {
    /// Number of axis-aligned rotations.
    pub const COUNT: u8 = 24;
    /// The identity rotation.
    pub const IDENTITY: Self = Self(0);

    /// Creates a rotation from its index; fails if the index is not in `0..24`.
    pub const fn from_index(index: u8) -> Result<Self, CoreError> {
        if index < Self::COUNT {
            Ok(Self(index))
        } else {
            Err(CoreError::InvalidTransform)
        }
    }

    /// The index in `0..24`.
    pub const fn index(self) -> u8 {
        self.0
    }
}

/// Placement of a layer: translation, axis rotation and power-of-two scale (`2^scale_shift`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GeometryTransform {
    translation: [i64; 3],
    rotation: Rotation,
    scale_shift: i8,
}

impl GeometryTransform {
    /// No translation, no rotation, scale 1.
    pub const IDENTITY: Self = Self {
        translation: [0; 3],
        rotation: Rotation::IDENTITY,
        scale_shift: 0,
    };

    /// Creates a transform; fails if `|scale_shift| > 62`.
    pub const fn new(
        translation: [i64; 3],
        rotation: Rotation,
        scale_shift: i8,
    ) -> Result<Self, CoreError> {
        if scale_shift > MAX_SCALE_SHIFT || scale_shift < -MAX_SCALE_SHIFT {
            return Err(CoreError::InvalidTransform);
        }
        Ok(Self {
            translation,
            rotation,
            scale_shift,
        })
    }

    /// Translation in voxels of the layer's base level.
    pub const fn translation(&self) -> [i64; 3] {
        self.translation
    }
    /// Axis rotation.
    pub const fn rotation(&self) -> Rotation {
        self.rotation
    }
    /// Scale is `2^scale_shift`.
    pub const fn scale_shift(&self) -> i8 {
        self.scale_shift
    }
}

impl Default for GeometryTransform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotation_index_bounds() {
        assert!(Rotation::from_index(0).is_ok() && Rotation::from_index(23).is_ok());
        assert_eq!(Rotation::from_index(24), Err(CoreError::InvalidTransform));
        assert_eq!(Rotation::from_index(255), Err(CoreError::InvalidTransform));
    }

    #[test]
    fn scale_shift_bounds() {
        assert!(GeometryTransform::new([0; 3], Rotation::IDENTITY, 62).is_ok());
        assert!(GeometryTransform::new([0; 3], Rotation::IDENTITY, -62).is_ok());
        assert_eq!(
            GeometryTransform::new([0; 3], Rotation::IDENTITY, 63),
            Err(CoreError::InvalidTransform)
        );
        assert_eq!(
            GeometryTransform::new([0; 3], Rotation::IDENTITY, i8::MIN),
            Err(CoreError::InvalidTransform)
        );
    }

    #[test]
    fn identity_is_the_default_and_keeps_extreme_translations() {
        assert_eq!(GeometryTransform::default(), GeometryTransform::IDENTITY);
        let t = GeometryTransform::new(
            [i64::MIN, 0, i64::MAX],
            Rotation::from_index(7).unwrap(),
            -3,
        )
        .unwrap();
        assert_eq!(
            (t.translation(), t.rotation().index(), t.scale_shift()),
            ([i64::MIN, 0, i64::MAX], 7, -3)
        );
    }
}
