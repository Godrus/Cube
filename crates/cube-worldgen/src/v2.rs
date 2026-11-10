//! Algorithm "terrain-v2": biomes (frozen once released: any change of output needs a new
//! version). Integer arithmetic only, like v1; the shared value noise of v1 is reused as is.
//!
//! Two slow climate fields, temperature and moisture (lattice spacing 192 and 160, so several biomes fit in the finite demo world), pick the
//! biome of a column: **Tundra** (cold: snow on top, high hills), **Desert** (hot and dry: sand,
//! low dunes), **Forest** (wet: dark moss, dirt) and **Plains** (the rest). The boundaries are
//! ragged (a fine noise of +-1500 is added to the climate before the thresholds), so a border
//! is never a straight line. The *height* does not follow the biome label but the continuous
//! climate (cold raises, hot and dry lowers, wet raises a little) and a separate relief field
//! (spacing 256) that scales the large hills 0.125..1.375 times: there is no cliff at a biome
//! border, the land changes smoothly while the surface material changes at the border.
//! A voxel is solid iff `y <= height(x, z)`; its material depends on the biome and the depth.

use cube_core::{BRICK_VOLUME, WorldSeed};

use crate::v1::value_noise;
use crate::{BrickContent, materials};

/// The biomes of version 2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Biome {
    /// Grassland.
    Plains,
    /// Wet grassland.
    Forest,
    /// Hot and dry: sand.
    Desert,
    /// Cold: snow.
    Tundra,
}

/// Highest possible surface (voxels); the height always lies in `[-MAX_AMP - 1, MAX_AMP]`.
const MAX_AMP: i64 = 100;
/// Surface depth from which the material is stone (grass, dirt and sand layers are shallower).
const STONE_DEPTH: i64 = 4;

/// Temperature and moisture of a column in `[-32768, 32767]` each.
fn climate(seed: u64, x: i64, z: i64) -> (i128, i128) {
    (
        value_noise(seed, 20, x, z, 192),
        value_noise(seed, 21, x, z, 160),
    )
}

fn clamp(v: i128, lo: i128, hi: i128) -> i128 {
    v.max(lo).min(hi)
}

/// The biome of the column `(x, z)`.
pub fn biome(seed: WorldSeed, x: i64, z: i64) -> Biome {
    let (t, m) = climate(seed.raw(), x, z);
    biome_of(seed.raw(), x, z, t, m)
}

fn biome_of(seed: u64, x: i64, z: i64, t: i128, m: i128) -> Biome {
    let j = value_noise(seed, 22, x, z, 16) * 1500 / 32_768;
    let (t, m) = (t + j, m + j);
    if t < -12_000 {
        Biome::Tundra
    } else if t > 8_000 && m < 0 {
        Biome::Desert
    } else if m > 6_000 {
        Biome::Forest
    } else {
        Biome::Plains
    }
}

/// Height of the ground at column `(x, z)`: the highest solid voxel.
pub fn height(seed: WorldSeed, x: i64, z: i64) -> i64 {
    column(seed.raw(), x, z).0
}

/// `(height, biome)` of a column, computing the climate once.
fn column(seed: u64, x: i64, z: i64) -> (i64, Biome) {
    let (t, m) = climate(seed, x, z);
    let cold = clamp(-t - 6_000, 0, 12_000);
    let dry_hot = clamp(t - 6_000, 0, 12_000).min(clamp(-m, 0, 12_000));
    let wet = clamp(m - 4_000, 0, 12_000);
    // Multiplier of the big hills over 128: 64 (desert) .. 196 (cold and wet).
    let mult = 128 + 52 * cold / 12_000 - 64 * dry_hot / 12_000 + 16 * wet / 12_000;
    // Relief field: 16 .. 176 (over 128).
    let relief = 96 + value_noise(seed, 10, x, z, 256) * 80 / 32_768;
    // Big hills in 1/256 voxel: amplitude 40 * relief/128 * mult/128.
    let big = value_noise(seed, 0, x, z, 128) * (40 * 256) * relief * mult / (32_768 * 128 * 128);
    let mid = value_noise(seed, 1, x, z, 32) * 10 * 256 / 32_768;
    let fine = value_noise(seed, 2, x, z, 8) * 3 * 256 / 32_768;
    let h = (big + mid + fine).div_euclid(256) as i64;
    (h, biome_of(seed, x, z, t, m))
}

fn material_at(biome: Biome, depth: i64) -> u16 {
    match (biome, depth) {
        (Biome::Desert, 0..=3) => materials::SAND,
        (Biome::Tundra, 0) => materials::SNOW,
        (Biome::Tundra, 1..=3) => materials::DIRT,
        (Biome::Plains, 0) => materials::GRASS,
        (Biome::Forest, 0) => materials::MOSS,
        (Biome::Plains | Biome::Forest, 1..=3) => materials::DIRT,
        _ => materials::STONE,
    }
}

/// Material of one voxel.
pub fn material(seed: WorldSeed, x: i64, y: i64, z: i64) -> u16 {
    let (h, b) = column(seed.raw(), x, z);
    if y > h { 0 } else { material_at(b, h - y) }
}

/// Content of the brick whose origin is `(ox, oy, oz)`.
pub fn brick(seed: WorldSeed, ox: i64, oy: i64, oz: i64) -> BrickContent {
    if oy > MAX_AMP {
        return BrickContent::Empty;
    }
    if oy + 15 < -MAX_AMP - 1 - STONE_DEPTH {
        return BrickContent::Uniform(materials::STONE);
    }
    let mut d = Box::new([0u16; BRICK_VOLUME]);
    for lz in 0..16u8 {
        for lx in 0..16u8 {
            let (h, b) = column(seed.raw(), ox + i64::from(lx), oz + i64::from(lz));
            for ly in 0..16u8 {
                let y = oy + i64::from(ly);
                let i = usize::from(lx) + 16 * (usize::from(ly) + 16 * usize::from(lz));
                d[i] = if y > h { 0 } else { material_at(b, h - y) };
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
