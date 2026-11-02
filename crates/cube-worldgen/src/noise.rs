//! Integer-only deterministic value noise. No floating point, so results are identical on every
//! platform. Noise is a pure function of (seed, salt, world x, world z): it never depends on chunk
//! boundaries, which is what keeps neighbouring chunks seamless.

const FIXED: i64 = 1024;

/// SplitMix64 finaliser.
pub fn mix(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Random lattice value in `0..=255`.
fn lattice(seed: u64, salt: u64, ix: i32, iz: i32) -> i64 {
    let h = mix(seed ^ salt.wrapping_mul(0xD6E8_FEB8_6659_FD93));
    let h = mix(h ^ (ix as u32 as u64));
    let h = mix(h ^ ((iz as u32 as u64) << 32));
    (h >> 56) as i64
}

/// Smoothstep on a `0..=FIXED` parameter, result in `0..=FIXED`.
fn smooth(t: i64) -> i64 {
    t * t * (3 * FIXED - 2 * t) / (FIXED * FIXED)
}

/// Smooth value noise in `0..=255` with lattice spacing `cell` (> 0) in voxels.
pub fn value_noise(seed: u64, salt: u64, wx: i32, wz: i32, cell: i32) -> i32 {
    let ix = wx.div_euclid(cell);
    let iz = wz.div_euclid(cell);
    let tx = smooth(wx.rem_euclid(cell) as i64 * FIXED / cell as i64);
    let tz = smooth(wz.rem_euclid(cell) as i64 * FIXED / cell as i64);
    let v00 = lattice(seed, salt, ix, iz);
    let v10 = lattice(seed, salt, ix + 1, iz);
    let v01 = lattice(seed, salt, ix, iz + 1);
    let v11 = lattice(seed, salt, ix + 1, iz + 1);
    let a = v00 * (FIXED - tx) + v10 * tx;
    let b = v01 * (FIXED - tx) + v11 * tx;
    ((a * (FIXED - tz) + b * tz) / (FIXED * FIXED)) as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_and_determinism() {
        for x in (-200..200).step_by(7) {
            for z in (-200..200).step_by(11) {
                let a = value_noise(42, 1, x, z, 32);
                assert!((0..=255).contains(&a));
                assert_eq!(a, value_noise(42, 1, x, z, 32));
            }
        }
    }

    #[test]
    fn continuous_across_lattice_and_zero() {
        // Neighbouring samples never jump by more than the slope bound (255 over a 16 cell, smoothstep max slope 1.5).
        for x in -100..100 {
            let d = (value_noise(7, 2, x, 5, 16) - value_noise(7, 2, x + 1, 5, 16)).abs();
            assert!(d <= 26, "jump {d} at x={x}");
        }
    }

    #[test]
    fn seed_changes_values() {
        let same = (0..64)
            .filter(|&x| value_noise(1, 1, x, 3, 16) == value_noise(2, 1, x, 3, 16))
            .count();
        assert!(same < 32);
    }
}
