//! Algorithm "terrain-v1" (frozen once released: any change of output needs a new version).
//!
//! The height of the ground at column `(x, z)` is `floor(sum of octaves / 256)` where an octave
//! is integer value noise on a lattice with the given spacing, smoothed with the integer
//! smoothstep `3t^2 - 2t^3` in 16-bit fixed point and scaled to `amplitude` voxels (result in
//! 1/256 voxel). A voxel `(x, y, z)` is solid iff `y <= height(x, z)`; its material depends on
//! the depth below the surface. All arithmetic is integer; division rounds towards negative
//! infinity so negative coordinates need no special case.

use cube_core::{BRICK_VOLUME, WorldSeed};

use crate::{BrickContent, materials};

/// `(lattice spacing in voxels, amplitude in voxels)` per octave.
const OCTAVES: [(i64, i64); 3] = [(128, 40), (32, 10), (8, 3)];
/// Sum of amplitudes: the height always lies in `[-MAX_AMP - 1, MAX_AMP]`.
const MAX_AMP: i64 = 53;
/// Surface depth from which the material is stone.
const STONE_DEPTH: i64 = 4;

pub(crate) fn mix(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// Lattice value in `[-32768, 32767]`.
fn lattice(seed: u64, octave: u64, ix: i64, iz: i64) -> i128 {
    let h = mix(seed
        ^ mix(octave.wrapping_add(0x9e37_79b9_7f4a_7c15))
        ^ mix(mix(ix as u64) ^ (iz as u64).rotate_left(32)));
    i128::from((h >> 48) as i64) - 32768
}

/// Smoothstep of `t` in `0..=65536`, result in `0..=65536`.
pub(crate) fn smooth(t: i128) -> i128 {
    (t * t * (3 * 65_536 - 2 * t)).div_euclid(65_536 * 65_536)
}

/// Smoothed value noise at column `(x, z)` on a lattice with the given spacing, in
/// `[-32768, 32767]` (shared with later versions; the arithmetic must never change).
pub(crate) fn value_noise(seed: u64, n: u64, x: i64, z: i64, spacing: i64) -> i128 {
    let (ix, fx) = (x.div_euclid(spacing), x.rem_euclid(spacing));
    let (iz, fz) = (z.div_euclid(spacing), z.rem_euclid(spacing));
    let sx = smooth(i128::from(fx) * 65_536 / i128::from(spacing));
    let sz = smooth(i128::from(fz) * 65_536 / i128::from(spacing));
    let v = |dx: i64, dz: i64| lattice(seed, n, ix + dx, iz + dz);
    let a = v(0, 0) + ((v(1, 0) - v(0, 0)) * sx).div_euclid(65_536);
    let b = v(0, 1) + ((v(1, 1) - v(0, 1)) * sx).div_euclid(65_536);
    a + ((b - a) * sz).div_euclid(65_536)
}

/// One octave at column `(x, z)`, in 1/256 voxel.
fn octave(seed: u64, n: u64, x: i64, z: i64, spacing: i64, amp: i64) -> i128 {
    let h = value_noise(seed, n, x, z, spacing);
    (h * i128::from(amp) * 256).div_euclid(32_768)
}

/// Ground height at column `(x, z)`: the highest solid voxel.
pub fn height(seed: WorldSeed, x: i64, z: i64) -> i64 {
    let sum: i128 = OCTAVES
        .iter()
        .enumerate()
        .map(|(n, (s, a))| octave(seed.raw(), n as u64, x, z, *s, *a))
        .sum();
    sum.div_euclid(256) as i64
}

fn material_at_depth(depth: i64) -> u16 {
    match depth {
        0 => materials::GRASS,
        1..=3 => materials::DIRT,
        _ => materials::STONE,
    }
}

/// Material of one voxel.
pub fn material(seed: WorldSeed, x: i64, y: i64, z: i64) -> u16 {
    let h = height(seed, x, z);
    if y > h { 0 } else { material_at_depth(h - y) }
}

/// Content of the brick whose origin is `(ox, oy, oz)`.
pub fn brick(seed: WorldSeed, ox: i64, oy: i64, oz: i64) -> BrickContent {
    // Whole bricks above every possible surface or deep below it need no per-voxel work.
    if oy > MAX_AMP {
        return BrickContent::Empty;
    }
    if oy + 15 < -MAX_AMP - 1 - STONE_DEPTH {
        return BrickContent::Uniform(materials::STONE);
    }
    let mut d = Box::new([0u16; BRICK_VOLUME]);
    for lz in 0..16u8 {
        for lx in 0..16u8 {
            let h = height(seed, ox + i64::from(lx), oz + i64::from(lz));
            for ly in 0..16u8 {
                let y = oy + i64::from(ly);
                // The order of `LocalPos::all()`: x fastest, then y, then z.
                let i = usize::from(lx) + 16 * (usize::from(ly) + 16 * usize::from(lz));
                d[i] = if y > h { 0 } else { material_at_depth(h - y) };
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
