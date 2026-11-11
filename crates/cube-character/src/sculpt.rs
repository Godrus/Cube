//! Deterministic sculpting: shape operations in model units (metres) rasterised onto a lattice of
//! `res` voxels per unit. Shapes are defined continuously, so the same sculpt can be rasterised at
//! any density; the density comparison in the docs uses exactly this.

use crate::grid::Grid;

/// A solid shape in model units.
#[derive(Debug, Clone, Copy)]
pub enum Shape {
    /// Axis-aligned box.
    Box {
        /// Minimum corner.
        lo: [f32; 3],
        /// Maximum corner.
        hi: [f32; 3],
    },
    /// Superellipsoid: `|x/rx|^p + |y/ry|^p + |z/rz|^p <= 1` (p = 2 ellipsoid, large p box).
    Super {
        /// Centre.
        c: [f32; 3],
        /// Radii per axis.
        r: [f32; 3],
        /// Exponent.
        p: f32,
    },
    /// Band in the XY plane around the segment a-b, infinite in z (always used with a clip).
    Band {
        /// Segment start.
        a: [f32; 2],
        /// Segment end.
        b: [f32; 2],
        /// Half width.
        hw: f32,
    },
}

/// What an operation does to a covered voxel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Sets the material.
    Add,
    /// Sets the material only where a voxel already exists (surface colouring).
    Paint,
    /// Removes the voxel.
    Carve,
}

/// One sculpt operation; later operations win.
#[derive(Debug, Clone, Copy)]
pub struct Op {
    /// The shape.
    pub shape: Shape,
    /// Material (ignored by `Carve`).
    pub mat: u8,
    /// Effect.
    pub mode: Mode,
    /// Optional clip box `(lo, hi)`.
    pub clip: Option<([f32; 3], [f32; 3])>,
    /// Optional patchy variation: `(alt_material, fraction)`.
    pub vary: Option<(u8, f32)>,
}

impl Op {
    /// Add op.
    pub fn add(shape: Shape, mat: u8) -> Self {
        Self {
            shape,
            mat,
            mode: Mode::Add,
            clip: None,
            vary: None,
        }
    }
    /// Paint op.
    pub fn paint(shape: Shape, mat: u8) -> Self {
        Self {
            shape,
            mat,
            mode: Mode::Paint,
            clip: None,
            vary: None,
        }
    }
    /// Carve op.
    pub fn carve(shape: Shape) -> Self {
        Self {
            shape,
            mat: 0,
            mode: Mode::Carve,
            clip: None,
            vary: None,
        }
    }
    /// Adds patchy variation.
    pub fn vary(mut self, alt: u8, fraction: f32) -> Self {
        self.vary = Some((alt, fraction));
        self
    }
    /// Adds a clip box.
    pub fn clip(mut self, lo: [f32; 3], hi: [f32; 3]) -> Self {
        self.clip = Some((lo, hi));
        self
    }

    /// Model-space bounds of the op (shape intersected with the clip).
    pub fn bounds(&self) -> ([f32; 3], [f32; 3]) {
        let (mut lo, mut hi) = match self.shape {
            Shape::Box { lo, hi } => (lo, hi),
            Shape::Super { c, r, .. } => (
                [0, 1, 2].map(|i| c[i] - r[i]),
                [0, 1, 2].map(|i| c[i] + r[i]),
            ),
            Shape::Band { a, b, hw } => (
                [a[0].min(b[0]) - hw, a[1].min(b[1]) - hw, -1.0],
                [a[0].max(b[0]) + hw, a[1].max(b[1]) + hw, 1.0],
            ),
        };
        if let Some((cl, ch)) = self.clip {
            for i in 0..3 {
                lo[i] = lo[i].max(cl[i]);
                hi[i] = hi[i].min(ch[i]);
            }
        }
        (lo, hi)
    }

    fn contains(&self, p: [f32; 3]) -> bool {
        if let Some((lo, hi)) = self.clip
            && (0..3).any(|i| p[i] < lo[i] || p[i] > hi[i])
        {
            return false;
        }
        match self.shape {
            Shape::Box { lo, hi } => (0..3).all(|i| p[i] >= lo[i] && p[i] <= hi[i]),
            Shape::Super { c, r, p: e } => {
                let mut s = 0.0f32;
                for i in 0..3 {
                    s += ((p[i] - c[i]) / r[i]).abs().powf(e);
                }
                s <= 1.0
            }
            Shape::Band { a, b, hw } => {
                let (ab, ap) = ([b[0] - a[0], b[1] - a[1]], [p[0] - a[0], p[1] - a[1]]);
                let t = ((ap[0] * ab[0] + ap[1] * ab[1]) / (ab[0] * ab[0] + ab[1] * ab[1]))
                    .clamp(0.0, 1.0);
                let (dx, dy) = (ap[0] - t * ab[0], ap[1] - t * ab[1]);
                dx * dx + dy * dy <= hw * hw
            }
        }
    }
}

/// Box shape helper (sorts the corners so mirrored boxes stay valid).
pub fn bx(a: [f32; 3], b: [f32; 3]) -> Shape {
    Shape::Box {
        lo: [0, 1, 2].map(|i| a[i].min(b[i])),
        hi: [0, 1, 2].map(|i| a[i].max(b[i])),
    }
}

/// Superellipsoid helper.
pub fn sup(c: [f32; 3], r: [f32; 3], p: f32) -> Shape {
    Shape::Super { c, r, p }
}

/// Deterministic patch noise of a model-space position (patch size 3.5 cm, independent of `res`).
pub fn patch_noise(p: [f32; 3], seed: u32) -> f32 {
    const CELL: f32 = 0.035;
    let a = (p[0] / CELL).floor() as i32 as u32;
    let b = (p[1] / CELL).floor() as i32 as u32;
    let c = (p[2] / CELL).floor() as i32 as u32;
    let mut x = a.wrapping_mul(0x9E37_79B1)
        ^ b.wrapping_mul(0x85EB_CA77)
        ^ c.wrapping_mul(0xC2B2_AE3D)
        ^ seed.wrapping_mul(0x27D4_EB2F);
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^= x >> 12;
    x = x.wrapping_mul(0x297A_2D39);
    x ^= x >> 15;
    (x & 0xFFFF) as f32 / 65535.0
}

/// Lattice bounds `(lo, hi_exclusive)` that cover every `Add` op at `res`.
pub fn lattice_bounds(ops: &[Op], res: f32) -> ([i32; 3], [i32; 3]) {
    let mut lo = [f32::MAX; 3];
    let mut hi = [f32::MIN; 3];
    for op in ops.iter().filter(|o| o.mode == Mode::Add) {
        let (l, h) = op.bounds();
        for i in 0..3 {
            lo[i] = lo[i].min(l[i]);
            hi[i] = hi[i].max(h[i]);
        }
    }
    (
        [0, 1, 2].map(|i| (lo[i] * res).floor() as i32),
        [0, 1, 2].map(|i| (hi[i] * res).ceil() as i32 + 1),
    )
}

/// Rasterises the ops into a grid at `res` voxels per unit (voxel centres are sampled).
pub fn rasterise(ops: &[Op], res: f32, seed: u32) -> Grid {
    let (glo, ghi) = lattice_bounds(ops, res);
    let mut g = Grid::new(glo, ghi);
    for (n, op) in ops.iter().enumerate() {
        let (l, h) = op.bounds();
        let vlo = [0, 1, 2].map(|i| ((l[i] * res).floor() as i32).max(glo[i]));
        let vhi = [0, 1, 2].map(|i| ((h[i] * res).ceil() as i32 + 1).min(ghi[i]));
        for z in vlo[2]..vhi[2] {
            for y in vlo[1]..vhi[1] {
                for x in vlo[0]..vhi[0] {
                    let p = [
                        (x as f32 + 0.5) / res,
                        (y as f32 + 0.5) / res,
                        (z as f32 + 0.5) / res,
                    ];
                    if !op.contains(p) {
                        continue;
                    }
                    let cell = [x, y, z];
                    match op.mode {
                        Mode::Carve => g.set(cell, 0),
                        Mode::Paint if g.get(cell) == 0 => {}
                        _ => {
                            let mut m = op.mat;
                            if let Some((alt, f)) = op.vary
                                && patch_noise(p, seed.wrapping_add(n as u32)) < f
                            {
                                m = alt;
                            }
                            g.set(cell, m);
                        }
                    }
                }
            }
        }
    }
    g
}
