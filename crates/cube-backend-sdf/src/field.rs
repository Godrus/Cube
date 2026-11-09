//! Field convention, quantisation and the analytic shapes of the operations.
//!
//! Distances are computed in `f64` from exact integer inputs with only `+ - * /` and `sqrt`
//! (all correctly rounded by IEEE 754), then rounded once to the quantised grid, so results are
//! identical on every platform that implements IEEE 754 arithmetic. This is a determinism
//! argument for CI hosts, not a device claim (DG-2).

use cube_core::VoxelPos;

/// Quantisation steps per voxel.
pub const SCALE: i32 = 256;
/// The saturated "far outside" value; `-OUTSIDE` is the saturated "deep inside". `i16::MIN` is
/// never produced (the range is symmetric).
pub const OUTSIDE: i16 = i16::MAX;
/// Distance, in voxels, beyond which a shape no longer changes the quantised field.
pub const REACH: i64 = (OUTSIDE as i64 / SCALE as i64) + 2;

/// Quantises a distance in voxels, saturating symmetrically.
pub fn quantise(distance: f64) -> i16 {
    let q = (distance * f64::from(SCALE)).round();
    if q >= f64::from(OUTSIDE) {
        OUTSIDE
    } else if q <= -f64::from(OUTSIDE) {
        -OUTSIDE
    } else {
        q as i16
    }
}

/// An analytic shape in voxel-lattice coordinates. The sample point of a voxel is its centre.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    /// Centre at a lattice point; `radius_16` in sixteenths of a voxel.
    Sphere {
        /// Centre.
        centre: VoxelPos,
        /// Radius in 1/16 voxel.
        radius_16: u32,
    },
    /// Axis-aligned box `[min, max)` on the lattice.
    Box {
        /// Lower corner.
        min: VoxelPos,
        /// Upper corner (exclusive).
        max: VoxelPos,
    },
}

fn axis(v: VoxelPos) -> [i128; 3] {
    [i128::from(v.x), i128::from(v.y), i128::from(v.z)]
}

impl Shape {
    /// Smallest and largest lattice coordinate (inclusive) that can differ from "far outside",
    /// per axis; `None` if the shape is empty or malformed.
    pub fn influence(&self) -> Option<([i128; 3], [i128; 3])> {
        let reach = i128::from(REACH);
        let (lo, hi) = match *self {
            Shape::Sphere { centre, radius_16 } => {
                let c = axis(centre);
                let r = i128::from(radius_16) / 16 + 1;
                (
                    [c[0] - r, c[1] - r, c[2] - r],
                    [c[0] + r, c[1] + r, c[2] + r],
                )
            }
            Shape::Box { min, max } => (axis(min), axis(max)),
        };
        (lo[0] <= hi[0] && lo[1] <= hi[1] && lo[2] <= hi[2]).then_some((
            [lo[0] - reach, lo[1] - reach, lo[2] - reach],
            [hi[0] + reach, hi[1] + reach, hi[2] + reach],
        ))
    }

    /// Quantised signed distance at the centre of voxel `p`.
    pub fn sample(&self, p: VoxelPos) -> i16 {
        let far = i128::from(REACH) * 2 + 4_194_304; // anything farther saturates
        // Work in half voxels so that centres (x + 1/2) and box corners are exact integers.
        let p2 = [
            2 * i128::from(p.x) + 1,
            2 * i128::from(p.y) + 1,
            2 * i128::from(p.z) + 1,
        ];
        match *self {
            Shape::Sphere { centre, radius_16 } => {
                let c2 = axis(centre).map(|v| 2 * v);
                let d = [p2[0] - c2[0], p2[1] - c2[1], p2[2] - c2[2]];
                if d.iter().any(|v| v.abs() > far) {
                    return OUTSIDE;
                }
                let s = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]) as f64;
                quantise(s.sqrt() / 2.0 - f64::from(radius_16) / 16.0)
            }
            Shape::Box { min, max } => {
                let (lo2, hi2) = (axis(min).map(|v| 2 * v), axis(max).map(|v| 2 * v));
                let mut outside = 0.0f64;
                let mut inside = f64::NEG_INFINITY;
                for a in 0..3 {
                    let centre2 = (lo2[a] + hi2[a]) / 2;
                    let half2 = (hi2[a] - lo2[a]) / 2;
                    let q = (p2[a] - centre2).abs() - half2;
                    if q > far {
                        return OUTSIDE;
                    }
                    let qf = q as f64 / 2.0;
                    if qf > 0.0 {
                        outside += qf * qf;
                    }
                    inside = inside.max(qf);
                }
                if outside > 0.0 {
                    quantise(outside.sqrt())
                } else {
                    quantise(inside.min(0.0))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quantisation_is_symmetric_saturating_and_never_i16_min() {
        assert_eq!(quantise(0.0), 0);
        assert_eq!(quantise(1.0), 256);
        assert_eq!(quantise(-1.0), -256);
        assert_eq!(quantise(1e9), OUTSIDE);
        assert_eq!(quantise(-1e9), -OUTSIDE);
        assert!(quantise(-1e300) > i16::MIN);
    }

    #[test]
    fn a_sphere_is_negative_inside_positive_outside_and_saturates_far_away() {
        let s = Shape::Sphere {
            centre: VoxelPos::new(0, 0, 0),
            radius_16: 16 * 5,
        };
        assert!(s.sample(VoxelPos::new(0, 0, 0)) < 0);
        assert!(s.sample(VoxelPos::new(9, 9, 9)) > 0);
        assert_eq!(s.sample(VoxelPos::new(i64::MAX, 0, 0)), OUTSIDE);
        assert_eq!(
            s.sample(VoxelPos::new(i64::MIN, i64::MIN, i64::MIN)),
            OUTSIDE
        );
    }

    #[test]
    fn a_box_is_exact_on_faces_and_deep_inside_is_negative() {
        let b = Shape::Box {
            min: VoxelPos::new(0, 0, 0),
            max: VoxelPos::new(10, 10, 10),
        };
        // The centre of voxel 4 is at 4.5: the nearest face (at 0) is 4.5 away.
        assert_eq!(b.sample(VoxelPos::new(4, 4, 4)), -(4.5 * 256.0) as i16);
        assert_eq!(b.sample(VoxelPos::new(-1, 4, 4)), (0.5 * 256.0) as i16);
        assert!(b.sample(VoxelPos::new(100, 100, 100)) > 0);
    }
}
