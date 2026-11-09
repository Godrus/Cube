//! Exact integer arithmetic for rotations and transforms (CORE-04, decision D-T / DEC-065).
//!
//! The 24 proper axis-aligned rotations are signed permutation matrices with determinant +1,
//! enumerated in a fixed order (axis permutations lexicographically, then sign masks ascending);
//! index 0 is the identity. No floating point is involved anywhere.

use crate::error::CoreError;
use crate::transform::{GeometryTransform, MAX_SCALE_SHIFT, Rotation};

type Matrix = [[i8; 3]; 3];

const PERMS: [[usize; 3]; 6] = [
    [0, 1, 2],
    [0, 2, 1],
    [1, 0, 2],
    [1, 2, 0],
    [2, 0, 1],
    [2, 1, 0],
];

/// Parity of a permutation: `1` even, `-1` odd.
const fn parity(p: [usize; 3]) -> i8 {
    let mut inv = 0;
    let mut i = 0;
    while i < 3 {
        let mut j = i + 1;
        while j < 3 {
            if p[i] > p[j] {
                inv += 1;
            }
            j += 1;
        }
        i += 1;
    }
    if inv % 2 == 0 { 1 } else { -1 }
}

const fn build() -> [Matrix; 24] {
    let mut out = [[[0i8; 3]; 3]; 24];
    let mut n = 0;
    let mut pi = 0;
    while pi < 6 {
        let mut mask = 0;
        while mask < 8 {
            let s = [
                if mask & 1 == 0 { 1i8 } else { -1 },
                if mask & 2 == 0 { 1i8 } else { -1 },
                if mask & 4 == 0 { 1i8 } else { -1 },
            ];
            if parity(PERMS[pi]) * s[0] * s[1] * s[2] == 1 {
                let mut r = 0;
                while r < 3 {
                    out[n][r][PERMS[pi][r]] = s[r];
                    r += 1;
                }
                n += 1;
            }
            mask += 1;
        }
        pi += 1;
    }
    out
}

const MATRICES: [Matrix; 24] = build();

fn mul(a: &Matrix, b: &Matrix) -> Matrix {
    let mut m = [[0i8; 3]; 3];
    for (i, row) in m.iter_mut().enumerate() {
        for (j, cell) in row.iter_mut().enumerate() {
            *cell = (0..3).map(|k| a[i][k] * b[k][j]).sum();
        }
    }
    m
}

fn transpose(a: &Matrix) -> Matrix {
    let mut m = [[0i8; 3]; 3];
    for (i, row) in m.iter_mut().enumerate() {
        for (j, cell) in row.iter_mut().enumerate() {
            *cell = a[j][i];
        }
    }
    m
}

fn index_of(m: &Matrix) -> Rotation {
    let i = MATRICES
        .iter()
        .position(|x| x == m)
        .expect("a product of rotations is a rotation");
    Rotation::from_index(i as u8).expect("index below 24")
}

impl Rotation {
    /// Composition: the rotation that applies `other` first, then `self`.
    pub fn compose(self, other: Self) -> Self {
        index_of(&mul(
            &MATRICES[usize::from(self.index())],
            &MATRICES[usize::from(other.index())],
        ))
    }

    /// The inverse rotation.
    pub fn inverse(self) -> Self {
        index_of(&transpose(&MATRICES[usize::from(self.index())]))
    }

    /// Rotates an integer vector; fails only if a negation would overflow (`i64::MIN`).
    pub fn apply(self, v: [i64; 3]) -> Result<[i64; 3], CoreError> {
        let m = &MATRICES[usize::from(self.index())];
        let mut out = [0i64; 3];
        for (o, row) in out.iter_mut().zip(m) {
            let c = row
                .iter()
                .position(|x| *x != 0)
                .expect("every row has one entry");
            *o = if row[c] > 0 {
                v[c]
            } else {
                v[c].checked_neg().ok_or(CoreError::Overflow)?
            };
        }
        Ok(out)
    }
}

impl GeometryTransform {
    /// Maps a point of the layer's space to the parent space: rotate, scale by `2^scale_shift`
    /// (shift left, exact; a negative shift is not supported here), then translate. Fails on
    /// overflow or for a negative scale shift.
    pub fn apply(&self, p: [i64; 3]) -> Result<[i64; 3], CoreError> {
        let s = self.scale_shift();
        if s < 0 {
            return Err(CoreError::InvalidTransform);
        }
        let r = self.rotation().apply(p)?;
        let t = self.translation();
        let mut out = [0i64; 3];
        for i in 0..3 {
            let scaled = r[i].checked_mul(1i64 << s).ok_or(CoreError::Overflow)?;
            out[i] = scaled.checked_add(t[i]).ok_or(CoreError::Overflow)?;
        }
        Ok(out)
    }

    /// The transform equal to applying `inner` first and then `self`
    /// (`self.compose(inner).apply(p) == self.apply(inner.apply(p))`). Exact: both scale shifts
    /// must be non-negative and the summed shift at most 62; fails on overflow otherwise.
    pub fn compose(&self, inner: &Self) -> Result<Self, CoreError> {
        let (so, si) = (self.scale_shift(), inner.scale_shift());
        if so < 0 || si < 0 {
            return Err(CoreError::InvalidTransform);
        }
        let shift = so
            .checked_add(si)
            .filter(|s| *s <= MAX_SCALE_SHIFT)
            .ok_or(CoreError::InvalidTransform)?;
        // translation = self.apply(inner.translation) because the inner point (0) maps to inner.translation.
        let inner_t = self.rotation().apply(inner.translation())?;
        let f = 1i64 << so;
        let o = self.translation();
        let mut t = [0i64; 3];
        for i in 0..3 {
            t[i] = inner_t[i]
                .checked_mul(f)
                .and_then(|v| v.checked_add(o[i]))
                .ok_or(CoreError::Overflow)?;
        }
        GeometryTransform::new(t, self.rotation().compose(inner.rotation()), shift)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cube_testkit::{DetRng, for_each_seed};

    fn rot(i: u8) -> Rotation {
        Rotation::from_index(i).unwrap()
    }

    #[test]
    fn there_are_24_distinct_proper_rotations_and_index_0_is_identity() {
        for (i, a) in MATRICES.iter().enumerate() {
            for b in &MATRICES[..i] {
                assert_ne!(a, b);
            }
        }
        assert_eq!(MATRICES[0], [[1, 0, 0], [0, 1, 0], [0, 0, 1]]);
        assert_eq!(Rotation::IDENTITY.apply([1, 2, 3]), Ok([1, 2, 3]));
    }

    #[test]
    fn rotations_form_a_group() {
        for a in 0..24 {
            assert_eq!(rot(a).compose(rot(a).inverse()), Rotation::IDENTITY);
            assert_eq!(rot(a).compose(Rotation::IDENTITY), rot(a));
            for b in 0..24 {
                for c in 0..24 {
                    assert_eq!(
                        rot(a).compose(rot(b)).compose(rot(c)),
                        rot(a).compose(rot(b).compose(rot(c)))
                    );
                }
            }
        }
    }

    #[test]
    fn apply_preserves_length_and_matches_composition() {
        for_each_seed(41, 300, |seed| {
            let mut r = DetRng::new(seed);
            let v = [
                r.range_i64(-1000, 1000),
                r.range_i64(-1000, 1000),
                r.range_i64(-1000, 1000),
            ];
            let (a, b) = (rot(r.below(24) as u8), rot(r.below(24) as u8));
            let len2 = |p: [i64; 3]| p.iter().map(|c| c * c).sum::<i64>();
            assert_eq!(len2(a.apply(v).unwrap()), len2(v));
            assert_eq!(a.compose(b).apply(v), a.apply(b.apply(v).unwrap()));
            assert_eq!(a.inverse().apply(a.apply(v).unwrap()), Ok(v));
        });
    }

    #[test]
    fn negating_the_minimum_overflows_instead_of_wrapping() {
        let flip = (0..24)
            .map(rot)
            .find(|r| r.apply([1, 0, 0]) == Ok([-1, 0, 0]))
            .unwrap();
        assert_eq!(flip.apply([i64::MIN, 0, 0]), Err(CoreError::Overflow));
        assert!(flip.apply([i64::MAX, 0, 0]).is_ok());
    }

    #[test]
    fn transform_composition_equals_sequential_application() {
        for_each_seed(42, 400, |seed| {
            let mut r = DetRng::new(seed);
            let mk = |r: &mut DetRng| {
                GeometryTransform::new(
                    [
                        r.range_i64(-500, 500),
                        r.range_i64(-500, 500),
                        r.range_i64(-500, 500),
                    ],
                    rot(r.below(24) as u8),
                    r.below(4) as i8,
                )
                .unwrap()
            };
            let (a, b) = (mk(&mut r), mk(&mut r));
            let p = [
                r.range_i64(-300, 300),
                r.range_i64(-300, 300),
                r.range_i64(-300, 300),
            ];
            let both = a.compose(&b).unwrap();
            assert_eq!(both.apply(p), a.apply(b.apply(p).unwrap()));
            assert_eq!(both.scale_shift(), a.scale_shift() + b.scale_shift());
        });
    }

    #[test]
    fn identity_is_neutral_and_composition_is_deterministic() {
        let t = GeometryTransform::new([5, -6, 7], rot(9), 2).unwrap();
        assert_eq!(GeometryTransform::IDENTITY.compose(&t), Ok(t));
        assert_eq!(t.compose(&GeometryTransform::IDENTITY), Ok(t));
        cube_testkit::assert_deterministic(|| t.compose(&t));
    }

    #[test]
    fn unsupported_or_overflowing_compositions_are_errors() {
        let down = GeometryTransform::new([0; 3], Rotation::IDENTITY, -1).unwrap();
        assert_eq!(
            down.compose(&GeometryTransform::IDENTITY),
            Err(CoreError::InvalidTransform)
        );
        assert_eq!(down.apply([1, 1, 1]), Err(CoreError::InvalidTransform));
        let big = GeometryTransform::new([0; 3], Rotation::IDENTITY, 40).unwrap();
        assert_eq!(
            big.compose(&big),
            Err(CoreError::InvalidTransform),
            "summed shift above 62"
        );
        let far = GeometryTransform::new([i64::MAX, 0, 0], Rotation::IDENTITY, 0).unwrap();
        assert_eq!(far.compose(&far), Err(CoreError::Overflow));
        assert_eq!(big.apply([i64::MAX, 0, 0]), Err(CoreError::Overflow));
    }
}
