//! Algorithm "terrain-v3": terrain-v2 plus caves and ores (frozen once released: any change of
//! output needs a new version). The surface, biomes and heights are exactly those of v2; only
//! the stone under the ground changes. Integer arithmetic only.
//!
//! * **Tunnels:** two 3D value noises (lattice 28); a voxel is carved where both are near zero,
//!   which makes winding tubes where two "sheets" cross.
//! * **Chambers:** a third noise (lattice 36) above a high threshold carves rare rooms.
//! * **Depth rule:** nothing is carved within 6 voxels of the surface, except in *mouth* regions
//!   (a 2D noise, lattice 40, above a high threshold) where carving reaches the surface itself:
//!   there tunnels open to the open air (sinkholes and cave mouths). Nothing is carved below
//!   [`CAVE_FLOOR`], so the deep bricks stay uniform stone.
//! * **Ores:** coal (small veins from 5 voxels deep) and iron (rarer, from 16 voxels deep)
//!   replace stone only, never air, never the surface layers, and only above [`ORE_FLOOR`].

use cube_core::{BRICK_VOLUME, WorldSeed};

use crate::v1::{mix, smooth, value_noise};
use crate::v2::{self, Biome};
use crate::{BrickContent, materials};

/// Nothing is carved below this height.
pub const CAVE_FLOOR: i64 = -96;
/// No ore below this height (so bricks below the v2 uniform-stone line stay uniform).
pub const ORE_FLOOR: i64 = -100;
/// Highest possible surface (voxels), as in v2.
const MAX_AMP: i64 = 100;
const STONE_DEPTH: i64 = 4;
/// Tunnel half width in noise units (of 32768).
const TUNNEL: i128 = 3_600;
const CHAMBER_MIN: i128 = 23_500;
const MOUTH_MIN: i128 = 24_000;
const COAL_MIN: i128 = 22_000;
const IRON_MIN: i128 = 24_500;

fn lattice3(seed: u64, n: u64, ix: i64, iy: i64, iz: i64) -> i128 {
    let h = mix(seed
        ^ mix(n.wrapping_add(0x7f4a_7c15_9e37_79b9))
        ^ mix(mix(ix as u64) ^ mix((iy as u64).rotate_left(21)) ^ (iz as u64).rotate_left(42)));
    i128::from((h >> 48) as i64) - 32768
}

/// Smoothed 3D value noise in `[-32768, 32767]`.
fn noise3(seed: u64, n: u64, x: i64, y: i64, z: i64, spacing: i64) -> i128 {
    let p = |v: i64| (v.div_euclid(spacing), v.rem_euclid(spacing));
    let (ix, fx) = p(x);
    let (iy, fy) = p(y);
    let (iz, fz) = p(z);
    let t = |f: i64| smooth(i128::from(f) * 65_536 / i128::from(spacing));
    let (sx, sy, sz) = (t(fx), t(fy), t(fz));
    let v = |dx: i64, dy: i64, dz: i64| lattice3(seed, n, ix + dx, iy + dy, iz + dz);
    let lerp = |a: i128, b: i128, s: i128| a + ((b - a) * s).div_euclid(65_536);
    let x00 = lerp(v(0, 0, 0), v(1, 0, 0), sx);
    let x10 = lerp(v(0, 1, 0), v(1, 1, 0), sx);
    let x01 = lerp(v(0, 0, 1), v(1, 0, 1), sx);
    let x11 = lerp(v(0, 1, 1), v(1, 1, 1), sx);
    let y0 = lerp(x00, x10, sy);
    let y1 = lerp(x01, x11, sy);
    lerp(y0, y1, sz)
}

/// `true` if the voxel `(x, y, z)` is cave air, given the column's mouth flag.
fn carved(seed: u64, x: i64, y: i64, z: i64, depth: i64, mouth: bool) -> bool {
    if y < CAVE_FLOOR || depth < if mouth { 0 } else { 6 } {
        return false;
    }
    if noise3(seed, 32, x, y, z, 36) > CHAMBER_MIN {
        return true;
    }
    noise3(seed, 30, x, y, z, 28).abs() < TUNNEL && noise3(seed, 31, x, y, z, 28).abs() < TUNNEL
}

fn mouth_at(seed: u64, x: i64, z: i64) -> bool {
    value_noise(seed, 33, x, z, 40) > MOUTH_MIN
}

fn ore_at(seed: u64, x: i64, y: i64, z: i64, depth: i64) -> u16 {
    if y < ORE_FLOOR || depth < 5 {
        return materials::STONE;
    }
    if depth >= 16 && noise3(seed, 35, x, y, z, 9) > IRON_MIN {
        return materials::IRON;
    }
    if noise3(seed, 34, x, y, z, 12) > COAL_MIN {
        return materials::COAL;
    }
    materials::STONE
}

/// Material of the voxel below the surface of a column (`h`, `biome`), already known solid.
fn solid(seed: u64, x: i64, y: i64, z: i64, h: i64, biome: Biome, mouth: bool) -> u16 {
    let depth = h - y;
    let base = v2::surface_material(biome, depth);
    if carved(seed, x, y, z, depth, mouth) {
        return 0;
    }
    if base == materials::STONE {
        ore_at(seed, x, y, z, depth)
    } else {
        base
    }
}

/// `true` if the surface voxel of the column `(x, z)` is cave air (a cave mouth).
pub(crate) fn is_open_at_surface(seed: u64, x: i64, z: i64) -> bool {
    let (h, b) = v2::column(seed, x, z);
    solid(seed, x, h, z, h, b, mouth_at(seed, x, z)) == 0
}

/// Material of one voxel.
pub fn material(seed: WorldSeed, x: i64, y: i64, z: i64) -> u16 {
    let (h, b) = v2::column(seed.raw(), x, z);
    if y > h {
        return 0;
    }
    solid(seed.raw(), x, y, z, h, b, mouth_at(seed.raw(), x, z))
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
            let (x, z) = (ox + i64::from(lx), oz + i64::from(lz));
            let (h, b) = v2::column(seed.raw(), x, z);
            let mouth = mouth_at(seed.raw(), x, z);
            for ly in 0..16u8 {
                let y = oy + i64::from(ly);
                let i = usize::from(lx) + 16 * (usize::from(ly) + 16 * usize::from(lz));
                d[i] = if y > h {
                    0
                } else {
                    solid(seed.raw(), x, y, z, h, b, mouth)
                };
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
