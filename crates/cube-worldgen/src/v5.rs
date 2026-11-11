//! Algorithm "terrain-v5": terrain-v4 plus the first places of interest, ruins (frozen once
//! released: any change of output needs a new version). Everything outside a ruin is exactly v4.
//! Integer arithmetic only.
//!
//! The world is cut into regions of 40 x 40 columns. A region holds at most one ruin (one region
//! in three; none in the four regions around the origin, so the first minutes are not decided by
//! it): its centre lies 8..=31 columns into the region, so the 11 x 11 footprint never leaves
//! its region and a voxel depends on one region only. A site is refused (deterministically)
//! where the ground inside the footprint varies by more than 5 voxels.
//!
//! A ruin is a red-brick floor at the height of the ground in the middle, levelled: the ground
//! above it is cut away, gaps below it are filled with stone, trees are cleared. A ring wall
//! (stone and red brick, 1 to 3 high, with gaps) stands on the floor edge with an open door on
//! the south side, a pillar of 4 on each corner, and a two-high altar of stone in the middle.
//! The creature den of the ruin is two voxels off the middle of the floor (the host places one
//! creature there; the altar stands in the middle).

use cube_core::WorldSeed;

use crate::v1::mix;
use crate::v2;
use crate::v4;
use crate::{BrickContent, materials};
use cube_core::BRICK_VOLUME;

/// Columns per region.
const REGION: i64 = 40;
/// Half width of the footprint (the footprint is `2 * HALF + 1` columns wide).
const HALF: i64 = 5;
/// Largest ground variation allowed inside a footprint.
const MAX_SLOPE: i64 = 5;
/// Voxels below the floor that may be filled and above it that may be cleared.
const BELOW: i64 = 8;
const ABOVE: i64 = 20;

/// A ruin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ruin {
    /// Centre column x.
    pub x: i64,
    /// Centre column z.
    pub z: i64,
    /// Height of the floor (its top voxel).
    pub floor: i64,
}

/// The ruin of region `(rx, rz)`, if it has one.
pub fn ruin_of_region(seed: u64, rx: i64, rz: i64) -> Option<Ruin> {
    if (-2..=1).contains(&rx) && (-2..=1).contains(&rz) {
        return None;
    }
    let h = mix(mix(seed ^ 0x201A_5EED) ^ mix(rx as u64) ^ mix((rz as u64).rotate_left(23)));
    if !h.is_multiple_of(3) {
        return None;
    }
    let x = rx * REGION + 8 + ((h >> 8) % 24) as i64;
    let z = rz * REGION + 8 + ((h >> 16) % 24) as i64;
    let (floor, _) = v2::column(seed, x, z);
    let (mut lo, mut hi) = (floor, floor);
    for dx in (-HALF..=HALF).step_by(5) {
        for dz in (-HALF..=HALF).step_by(5) {
            let (g, _) = v2::column(seed, x + dx, z + dz);
            lo = lo.min(g);
            hi = hi.max(g);
        }
    }
    (hi - lo <= MAX_SLOPE).then_some(Ruin { x, z, floor })
}

impl Ruin {
    /// What the ruin says about the voxel `(x, y, z)`: `Some(material)` (0 = air) if the ruin
    /// decides it given the natural `base` voxel and the natural ground `ground` of its column,
    /// `None` if the voxel is not the ruin's.
    fn voxel(&self, x: i64, y: i64, z: i64, base: u16, ground: i64) -> Option<u16> {
        let (dx, dz, dy) = (x - self.x, z - self.z, y - self.floor);
        if dx.abs() > HALF || dz.abs() > HALF || !(-BELOW..=ABOVE).contains(&dy) {
            return None;
        }
        if dy < 0 {
            // Fill the gap between the natural ground and the floor with stone.
            return (base == 0 && y > ground).then_some(materials::STONE);
        }
        if dy == 0 {
            return Some(materials::BRICK);
        }
        // Above the floor: wall, pillar, altar, or clean air.
        let edge = dx.abs() == HALF || dz.abs() == HALF;
        let corner = dx.abs() == HALF && dz.abs() == HALF;
        let door = dz == HALF && dx.abs() <= 1;
        let hh = mix(mix(x as u64) ^ (z as u64).rotate_left(17) ^ 0x57A1);
        let m = if corner {
            (dy <= 4).then_some(materials::STONE)
        } else if edge && !door {
            let height = 1 + (hh % 3) as i64;
            let gap = (hh >> 8).is_multiple_of(5);
            (!gap && dy <= height).then_some(if (hh >> 12).is_multiple_of(2) {
                materials::STONE
            } else {
                materials::BRICK
            })
        } else if dx == 0 && dz == 0 {
            (dy <= 2).then_some(materials::STONE)
        } else {
            None
        };
        Some(m.unwrap_or(0))
    }
}

/// The ruin whose footprint contains the column `(x, z)`, if any.
fn ruin_at(seed: u64, x: i64, z: i64) -> Option<Ruin> {
    ruin_of_region(seed, x.div_euclid(REGION), z.div_euclid(REGION))
        .filter(|r| (x - r.x).abs() <= HALF && (z - r.z).abs() <= HALF)
}

/// Material of one voxel.
pub fn material(seed: WorldSeed, x: i64, y: i64, z: i64) -> u16 {
    let base = v4::material(seed, x, y, z);
    if let Some(r) = ruin_at(seed.raw(), x, z) {
        let (ground, _) = v2::column(seed.raw(), x, z);
        if let Some(m) = r.voxel(x, y, z, base, ground) {
            return m;
        }
    }
    base
}

/// Content of the brick whose origin is `(ox, oy, oz)`.
pub fn brick(seed: WorldSeed, ox: i64, oy: i64, oz: i64) -> BrickContent {
    let base = v4::brick(seed, ox, oy, oz);
    let (rx0, rx1) = (
        (ox - HALF).div_euclid(REGION),
        (ox + 15 + HALF).div_euclid(REGION),
    );
    let (rz0, rz1) = (
        (oz - HALF).div_euclid(REGION),
        (oz + 15 + HALF).div_euclid(REGION),
    );
    let mut ruins = Vec::new();
    for rz in rz0..=rz1 {
        for rx in rx0..=rx1 {
            ruins.extend(ruin_of_region(seed.raw(), rx, rz).filter(|r| {
                r.x + HALF >= ox
                    && r.x - HALF <= ox + 15
                    && r.z + HALF >= oz
                    && r.z - HALF <= oz + 15
                    && r.floor + ABOVE >= oy
                    && r.floor - BELOW <= oy + 15
            }));
        }
    }
    if ruins.is_empty() {
        return base;
    }
    let mut d: Box<[u16; BRICK_VOLUME]> = match base {
        BrickContent::Empty => Box::new([0; BRICK_VOLUME]),
        BrickContent::Uniform(m) => Box::new([m; BRICK_VOLUME]),
        BrickContent::Dense(d) => d,
    };
    for r in &ruins {
        for lz in 0..16u8 {
            for lx in 0..16u8 {
                let (x, z) = (ox + i64::from(lx), oz + i64::from(lz));
                if (x - r.x).abs() > HALF || (z - r.z).abs() > HALF {
                    continue;
                }
                let (ground, _) = v2::column(seed.raw(), x, z);
                for ly in 0..16u8 {
                    let i = usize::from(lx) + 16 * (usize::from(ly) + 16 * usize::from(lz));
                    if let Some(m) = r.voxel(x, oy + i64::from(ly), z, d[i], ground) {
                        d[i] = m;
                    }
                }
            }
        }
    }
    let first = d[0];
    if d.iter().all(|m| *m == first) {
        return if first == 0 {
            BrickContent::Empty
        } else {
            BrickContent::Uniform(first)
        };
    }
    BrickContent::Dense(d)
}
