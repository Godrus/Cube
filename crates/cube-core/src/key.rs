//! Storage address of a brick (contract C-07).

use crate::coord::{BrickCoord, check_level};
use crate::error::CoreError;
use crate::ids::LayerId;

/// Address of one brick: `(layer, level, brick coordinate)`. The derived order is
/// `(layer, level, x, y, z)` with signed comparison, a total order usable for canonical sorting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CellKey {
    layer: LayerId,
    level: i8,
    brick: BrickCoord,
}

impl CellKey {
    /// Creates a key; fails for a reserved (negative) level.
    pub fn new(layer: LayerId, level: i8, brick: BrickCoord) -> Result<Self, CoreError> {
        check_level(level)?;
        Ok(Self {
            layer,
            level,
            brick,
        })
    }

    /// Layer of the brick.
    pub const fn layer(self) -> LayerId {
        self.layer
    }
    /// Refinement level (0 = the layer's base resolution).
    pub const fn level(self) -> i8 {
        self.level
    }
    /// Brick coordinate in the brick grid of `level`.
    pub const fn brick(self) -> BrickCoord {
        self.brick
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(l: u64, lv: i8, x: i64, y: i64, z: i64) -> CellKey {
        CellKey::new(LayerId::new(l), lv, BrickCoord::new(x, y, z)).unwrap()
    }

    #[test]
    fn rejects_reserved_levels() {
        assert_eq!(
            CellKey::new(LayerId::new(1), -1, BrickCoord::new(0, 0, 0)),
            Err(CoreError::InvalidLevel(-1))
        );
        assert!(CellKey::new(LayerId::new(1), 127, BrickCoord::new(0, 0, 0)).is_ok());
    }

    #[test]
    fn order_is_layer_level_then_signed_xyz() {
        let mut v = vec![
            key(2, 0, 0, 0, 0),
            key(1, 1, 0, 0, 0),
            key(1, 0, 1, 0, 0),
            key(1, 0, -1, 5, 5),
            key(1, 0, 0, -1, 9),
            key(1, 0, 0, 0, -1),
        ];
        v.sort();
        let expected = vec![
            key(1, 0, -1, 5, 5),
            key(1, 0, 0, -1, 9),
            key(1, 0, 0, 0, -1),
            key(1, 0, 1, 0, 0),
            key(1, 1, 0, 0, 0),
            key(2, 0, 0, 0, 0),
        ];
        assert_eq!(v, expected);
    }

    #[test]
    fn accessors_return_the_parts() {
        let k = key(7, 3, -4, 5, i64::MIN);
        assert_eq!(
            (k.layer(), k.level(), k.brick()),
            (LayerId::new(7), 3, BrickCoord::new(-4, 5, i64::MIN))
        );
    }
}
