//! Gradient and normals of the field (SDF-03; provisional like every provider of PS-5).
//!
//! The field is negative inside, so the gradient points out of the solid and `-gradient` into
//! it. Two helpers:
//!
//! * [`gradient_at`] is the general central difference over any sampler (for physics and tools
//!   that sample the committed field through a snapshot);
//! * [`cell_normal_q14`] computes a vertex normal from the eight samples of one cell only, which
//!   is what keeps normals identical on both sides of a brick join.
//!
//! **Tolerance (documented, tested):** the field is quantised to 1/256 voxel, so a normal
//! derived from it deviates from the analytic one by at most about `1/256` of a radian per
//! voxel of curvature radius plus the Q14 rounding (`2^-14`); the tests accept 0.06 rad on a
//! sphere of radius 6 voxels. A zero gradient (flat saturated region) yields the zero normal,
//! never a made-up direction.

use cube_core::VoxelPos;

use crate::field::SCALE;

/// Central-difference gradient of `sample` at `p`, in field units per voxel (a unit-length
/// gradient of a true distance field has length `SCALE`).
pub fn gradient_at(sample: &dyn Fn(VoxelPos) -> i16, p: VoxelPos) -> [f64; 3] {
    let d = |dx: i64, dy: i64, dz: i64| -> f64 {
        let q = |s: i64| {
            VoxelPos::new(
                p.x.saturating_add(dx * s),
                p.y.saturating_add(dy * s),
                p.z.saturating_add(dz * s),
            )
        };
        (f64::from(sample(q(1))) - f64::from(sample(q(-1)))) / 2.0
    };
    [d(1, 0, 0), d(0, 1, 0), d(0, 0, 1)]
}

/// Normalises a gradient to Q14 (16384 = 1.0); the zero vector stays zero.
pub fn normal_q14(g: [f64; 3]) -> [i16; 3] {
    let len = (g[0] * g[0] + g[1] * g[1] + g[2] * g[2]).sqrt();
    if len == 0.0 {
        return [0, 0, 0];
    }
    g.map(|v| (v / len * 16384.0).round() as i16)
}

/// Normal of a cell from its eight corner samples (bit 0 = x, bit 1 = y, bit 2 = z).
pub fn cell_normal_q14(d: [i16; 8]) -> [i16; 3] {
    let mut g = [0.0f64; 3];
    for (k, v) in d.iter().enumerate() {
        for (axis, gv) in g.iter_mut().enumerate() {
            let sign = if k & (1 << axis) != 0 { 1.0 } else { -1.0 };
            *gv += sign * f64::from(*v);
        }
    }
    normal_q14(g)
}

/// Field units per voxel of a true distance field (for converting gradients to lengths).
pub const UNIT: i32 = SCALE;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::Shape;

    #[test]
    fn the_gradient_of_a_sphere_points_outward_with_unit_length() {
        let s = Shape::Sphere {
            centre: VoxelPos::new(0, 0, 0),
            radius_16: 16 * 6,
        };
        let f = |p: VoxelPos| s.sample(p);
        let g = gradient_at(&f, VoxelPos::new(8, 0, 0));
        let len = (g[0] * g[0] + g[1] * g[1] + g[2] * g[2]).sqrt();
        assert!((len / f64::from(UNIT) - 1.0).abs() < 0.05, "{len}");
        assert!(g[0] > 0.0);
    }

    #[test]
    fn normals_match_the_analytic_sphere_within_the_documented_tolerance() {
        let s = Shape::Sphere {
            centre: VoxelPos::new(0, 0, 0),
            radius_16: 16 * 6,
        };
        let f = |p: VoxelPos| s.sample(p);
        for p in [
            VoxelPos::new(5, 1, 0),
            VoxelPos::new(-6, 2, 1),
            VoxelPos::new(1, -5, 3),
        ] {
            let n = normal_q14(gradient_at(&f, p));
            let c = [p.x as f64 + 0.5, p.y as f64 + 0.5, p.z as f64 + 0.5];
            let l = (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt();
            let dot: f64 = (0..3).map(|i| f64::from(n[i]) / 16384.0 * c[i] / l).sum();
            assert!(dot.clamp(-1.0, 1.0).acos() < 0.06, "{p:?} dot {dot}");
        }
        assert_eq!(normal_q14([0.0; 3]), [0, 0, 0]);
    }
}
