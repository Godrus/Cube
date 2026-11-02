//! Integer addressing (draft C-07). Base brick is 16^3 (AU-1.0 section 6).
//! The local index order is provisional and private to this crate.

pub const BRICK_SIZE: i32 = 16;
pub const BRICK_VOLUME: usize = 4096;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VoxelPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BrickCoord {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// Position inside a brick, each component in `0..16`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocalPos {
    pub x: u8,
    pub y: u8,
    pub z: u8,
}

impl VoxelPos {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// Floor-division split, correct for negative coordinates.
    pub fn split(self) -> (BrickCoord, LocalPos) {
        let b = BrickCoord {
            x: self.x.div_euclid(BRICK_SIZE),
            y: self.y.div_euclid(BRICK_SIZE),
            z: self.z.div_euclid(BRICK_SIZE),
        };
        let l = LocalPos {
            x: self.x.rem_euclid(BRICK_SIZE) as u8,
            y: self.y.rem_euclid(BRICK_SIZE) as u8,
            z: self.z.rem_euclid(BRICK_SIZE) as u8,
        };
        (b, l)
    }
}

impl BrickCoord {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// World-voxel position of local voxel `l` of this brick.
    pub fn join(self, l: LocalPos) -> VoxelPos {
        VoxelPos {
            x: self.x * BRICK_SIZE + l.x as i32,
            y: self.y * BRICK_SIZE + l.y as i32,
            z: self.z * BRICK_SIZE + l.z as i32,
        }
    }
}

/// Provisional index order (`x + 16*(y + 16*z)`); callers must not rely on it.
pub(crate) fn local_index(l: LocalPos) -> usize {
    l.x as usize + 16 * (l.y as usize + 16 * l.z as usize)
}

pub(crate) fn local_from_index(i: usize) -> LocalPos {
    LocalPos {
        x: (i % 16) as u8,
        y: ((i / 16) % 16) as u8,
        z: (i / 256) as u8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negative_coordinates_floor() {
        let (b, l) = VoxelPos::new(-1, -16, -17).split();
        assert_eq!(b, BrickCoord::new(-1, -1, -2));
        assert_eq!(l, LocalPos { x: 15, y: 0, z: 15 });
    }

    #[test]
    fn split_join_roundtrip_over_boundaries() {
        for v in [-33, -32, -17, -16, -15, -1, 0, 1, 15, 16, 17, 31, 32] {
            let p = VoxelPos::new(v, -v, v ^ 5);
            let (b, l) = p.split();
            assert_eq!(b.join(l), p);
        }
    }

    #[test]
    fn index_roundtrip() {
        for i in 0..4096 {
            assert_eq!(local_index(local_from_index(i)), i);
        }
    }
}
