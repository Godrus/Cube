//! Axis-aligned voxel region (draft C-06, AABB only). `min` inclusive, `max` exclusive.

use crate::coord::{BrickCoord, VoxelPos, BRICK_SIZE};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Region {
    pub min: VoxelPos,
    pub max: VoxelPos,
}

impl Region {
    pub const fn new(min: VoxelPos, max: VoxelPos) -> Self {
        Self { min, max }
    }

    pub fn is_empty(&self) -> bool {
        self.max.x <= self.min.x || self.max.y <= self.min.y || self.max.z <= self.min.z
    }

    /// Number of voxels; 0 for empty regions.
    pub fn volume(&self) -> u64 {
        if self.is_empty() {
            return 0;
        }
        (self.max.x as i64 - self.min.x as i64) as u64
            * (self.max.y as i64 - self.min.y as i64) as u64
            * (self.max.z as i64 - self.min.z as i64) as u64
    }

    pub fn contains(&self, p: VoxelPos) -> bool {
        p.x >= self.min.x
            && p.x < self.max.x
            && p.y >= self.min.y
            && p.y < self.max.y
            && p.z >= self.min.z
            && p.z < self.max.z
    }

    pub fn intersection(&self, o: &Region) -> Option<Region> {
        let r = Region {
            min: VoxelPos::new(
                self.min.x.max(o.min.x),
                self.min.y.max(o.min.y),
                self.min.z.max(o.min.z),
            ),
            max: VoxelPos::new(
                self.max.x.min(o.max.x),
                self.max.y.min(o.max.y),
                self.max.z.min(o.max.z),
            ),
        };
        if r.is_empty() {
            None
        } else {
            Some(r)
        }
    }

    pub fn union_bounds(&self, o: &Region) -> Region {
        if self.is_empty() {
            return *o;
        }
        if o.is_empty() {
            return *self;
        }
        Region {
            min: VoxelPos::new(
                self.min.x.min(o.min.x),
                self.min.y.min(o.min.y),
                self.min.z.min(o.min.z),
            ),
            max: VoxelPos::new(
                self.max.x.max(o.max.x),
                self.max.y.max(o.max.y),
                self.max.z.max(o.max.z),
            ),
        }
    }

    /// Bricks intersecting this region, in a deterministic (z, y, x) order.
    pub fn brick_coords(&self) -> Vec<BrickCoord> {
        if self.is_empty() {
            return Vec::new();
        }
        let lo = self.min.split().0;
        let hi = VoxelPos::new(self.max.x - 1, self.max.y - 1, self.max.z - 1)
            .split()
            .0;
        let mut v = Vec::new();
        for z in lo.z..=hi.z {
            for y in lo.y..=hi.y {
                for x in lo.x..=hi.x {
                    v.push(BrickCoord::new(x, y, z));
                }
            }
        }
        v
    }

    /// World-voxel region covered by a brick.
    pub fn of_brick(b: BrickCoord) -> Region {
        Region {
            min: VoxelPos::new(b.x * BRICK_SIZE, b.y * BRICK_SIZE, b.z * BRICK_SIZE),
            max: VoxelPos::new(
                (b.x + 1) * BRICK_SIZE,
                (b.y + 1) * BRICK_SIZE,
                (b.z + 1) * BRICK_SIZE,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn volume_and_empty() {
        let r = Region::new(VoxelPos::new(0, 0, 0), VoxelPos::new(2, 3, 4));
        assert_eq!(r.volume(), 24);
        assert!(Region::new(VoxelPos::new(1, 1, 1), VoxelPos::new(1, 5, 5)).is_empty());
    }

    #[test]
    fn bricks_across_negative_boundary() {
        let r = Region::new(VoxelPos::new(-1, 0, 0), VoxelPos::new(1, 1, 1));
        assert_eq!(
            r.brick_coords(),
            vec![BrickCoord::new(-1, 0, 0), BrickCoord::new(0, 0, 0)]
        );
    }

    #[test]
    fn exact_brick_has_one_brick() {
        let r = Region::of_brick(BrickCoord::new(-1, 0, 2));
        assert_eq!(r.brick_coords(), vec![BrickCoord::new(-1, 0, 2)]);
    }
}
