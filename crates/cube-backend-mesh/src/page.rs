//! The mesh page (MSH-01/02): format, canonical form, validation and normals.
//!
//! A page holds the triangles whose centroid lies in one brick. Positions are `i32` in 1/256
//! voxel **relative to the brick origin** (a triangle may stick out of the brick, up to
//! `+-2^30`). Layout (little endian), version 1:
//! `[1 u8][nverts u32][ntris u32][verts: 3 x i32 each][tris: 3 x u32 indices each][materials: u16 per tri]`.
//!
//! Canonical form: vertices sorted by position without duplicates, no unused vertices,
//! triangles sorted by their rotated-to-smallest vertex triple and material, so the same set of
//! triangles always gives the same bytes whatever the history of edits was.
//!
//! Validation rules (typed errors): known version, exact length, counts within [`MAX_TRIANGLES`],
//! every index in range, no degenerate triangle (repeated vertex or zero area), no duplicate
//! triangle, every edge is walked at most once in each direction, i.e. at most two triangles share it and
//! they traverse it in opposite directions (consistent winding). Pages are not required to be
//! closed: the other half of a surface can live in the next page.

use std::collections::BTreeMap;

/// Most triangles in one page (budget).
pub const MAX_TRIANGLES: usize = 65_536;
/// Largest relative coordinate magnitude, in 1/256 voxel.
pub const MAX_COORD: i64 = 1 << 30;

/// A triangle with absolute-in-page positions (1/256 voxel, relative to the brick origin) and a
/// material (0 = none).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tri {
    /// Counter-clockwise seen from outside.
    pub v: [[i32; 3]; 3],
    /// Material.
    pub material: u16,
}

impl Tri {
    /// The triangle with its vertices rotated so that the smallest comes first (same winding).
    pub fn canonical(self) -> Tri {
        let mut best = self.v;
        for r in 1..3 {
            let rot = [self.v[r], self.v[(r + 1) % 3], self.v[(r + 2) % 3]];
            if rot < best {
                best = rot;
            }
        }
        Tri {
            v: best,
            material: self.material,
        }
    }

    /// Twice the area vector (cross product) in 1/256 voxel squared.
    pub fn cross(&self) -> [i128; 3] {
        let a = self.v[0].map(i128::from);
        let b = self.v[1].map(i128::from);
        let c = self.v[2].map(i128::from);
        let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let w = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        [
            u[1] * w[2] - u[2] * w[1],
            u[2] * w[0] - u[0] * w[2],
            u[0] * w[1] - u[1] * w[0],
        ]
    }
}

/// Why a page is not valid.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum MeshError {
    /// Shorter or longer than its counts say.
    Length,
    /// Unknown page version.
    Version(u8),
    /// More triangles than the budget.
    TooLarge,
    /// A triangle index is not a vertex.
    IndexOutOfRange {
        /// The triangle.
        triangle: usize,
    },
    /// A triangle with a repeated vertex or zero area.
    Degenerate {
        /// The triangle.
        triangle: usize,
    },
    /// The same triangle twice.
    Duplicate {
        /// The triangle.
        triangle: usize,
    },
    /// Two triangles traverse a shared edge in the same direction (this also catches an edge
    /// used by more than two triangles, which must repeat a direction).
    InconsistentWinding,
    /// A coordinate beyond the allowed magnitude.
    CoordinateRange,
}

impl std::fmt::Display for MeshError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid mesh page: {self:?}")
    }
}

impl std::error::Error for MeshError {}

/// A decoded page: triangles in canonical order.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Page {
    /// Triangles, canonical.
    pub tris: Vec<Tri>,
}

fn rd32(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}

impl Page {
    /// Builds a page from triangles: canonical order, and the page rules are checked.
    pub fn from_tris(tris: Vec<Tri>) -> Result<Page, MeshError> {
        let mut t: Vec<Tri> = tris.into_iter().map(Tri::canonical).collect();
        t.sort_unstable();
        let p = Page { tris: t };
        p.validate()?;
        Ok(p)
    }

    /// Checks the page rules (see the module documentation).
    pub fn validate(&self) -> Result<(), MeshError> {
        if self.tris.len() > MAX_TRIANGLES {
            return Err(MeshError::TooLarge);
        }
        let mut edges: BTreeMap<([i32; 3], [i32; 3]), u32> = BTreeMap::new();
        for (i, t) in self.tris.iter().enumerate() {
            if t.v
                .iter()
                .flatten()
                .any(|c| i64::from(*c).abs() > MAX_COORD)
            {
                return Err(MeshError::CoordinateRange);
            }
            if t.v[0] == t.v[1] || t.v[1] == t.v[2] || t.v[0] == t.v[2] || t.cross() == [0; 3] {
                return Err(MeshError::Degenerate { triangle: i });
            }
            if i > 0 && self.tris[i - 1] == *t {
                return Err(MeshError::Duplicate { triangle: i });
            }
            for k in 0..3 {
                *edges.entry((t.v[k], t.v[(k + 1) % 3])).or_default() += 1;
            }
        }
        if edges.values().any(|n| *n > 1) {
            return Err(MeshError::InconsistentWinding);
        }
        Ok(())
    }

    /// Encodes the page (validates first).
    pub fn encode(&self) -> Result<Vec<u8>, MeshError> {
        self.validate()?;
        let mut verts: BTreeMap<[i32; 3], u32> = BTreeMap::new();
        for t in &self.tris {
            for v in &t.v {
                verts.insert(*v, 0);
            }
        }
        for (i, (_, id)) in verts.iter_mut().enumerate() {
            *id = i as u32;
        }
        let mut out = vec![1u8];
        out.extend((verts.len() as u32).to_le_bytes());
        out.extend((self.tris.len() as u32).to_le_bytes());
        for v in verts.keys() {
            for c in v {
                out.extend(c.to_le_bytes());
            }
        }
        for t in &self.tris {
            for v in &t.v {
                out.extend(verts[v].to_le_bytes());
            }
        }
        for t in &self.tris {
            out.extend(t.material.to_le_bytes());
        }
        Ok(out)
    }

    /// Decodes and validates a page. Never panics on hostile input.
    pub fn decode(b: &[u8]) -> Result<Page, MeshError> {
        if b.len() < 9 {
            return Err(MeshError::Length);
        }
        if b[0] != 1 {
            return Err(MeshError::Version(b[0]));
        }
        let (nv, nt) = (rd32(b, 1) as usize, rd32(b, 5) as usize);
        if nt > MAX_TRIANGLES || nv > 3 * MAX_TRIANGLES {
            return Err(MeshError::TooLarge);
        }
        let want = 9 + nv * 12 + nt * 12 + nt * 2;
        if b.len() != want {
            return Err(MeshError::Length);
        }
        let verts: Vec<[i32; 3]> = (0..nv)
            .map(|i| {
                let o = 9 + i * 12;
                [
                    rd32(b, o) as i32,
                    rd32(b, o + 4) as i32,
                    rd32(b, o + 8) as i32,
                ]
            })
            .collect();
        let mut tris = Vec::with_capacity(nt);
        for i in 0..nt {
            let o = 9 + nv * 12 + i * 12;
            let idx = [rd32(b, o), rd32(b, o + 4), rd32(b, o + 8)];
            if idx.iter().any(|v| *v as usize >= nv) {
                return Err(MeshError::IndexOutOfRange { triangle: i });
            }
            let m = 9 + nv * 12 + nt * 12 + i * 2;
            tris.push(Tri {
                v: [
                    verts[idx[0] as usize],
                    verts[idx[1] as usize],
                    verts[idx[2] as usize],
                ],
                material: u16::from_le_bytes([b[m], b[m + 1]]),
            });
        }
        let page = Page { tris };
        page.validate()?;
        // The stored form must be the canonical one: two byte strings for one mesh would make
        // content hashes depend on history.
        if page.encode()? != b {
            return Err(MeshError::Length);
        }
        Ok(page)
    }
}

/// Area-weighted vertex normals in Q14, one per distinct vertex position in ascending position
/// order (MSH-02 normal recalculation). A vertex whose triangles cancel out gets the zero normal.
pub fn normals_q14(page: &Page) -> Vec<([i32; 3], [i16; 3])> {
    let mut acc: BTreeMap<[i32; 3], [i128; 3]> = BTreeMap::new();
    for t in &page.tris {
        let c = t.cross();
        for v in &t.v {
            let a = acc.entry(*v).or_insert([0; 3]);
            for i in 0..3 {
                a[i] += c[i];
            }
        }
    }
    acc.into_iter()
        .map(|(v, a)| {
            let g = a.map(|x| x as f64);
            let len = (g[0] * g[0] + g[1] * g[1] + g[2] * g[2]).sqrt();
            let n = if len == 0.0 {
                [0, 0, 0]
            } else {
                g.map(|x| (x / len * 16384.0).round() as i16)
            };
            (v, n)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tri(a: [i32; 3], b: [i32; 3], c: [i32; 3]) -> Tri {
        Tri {
            v: [a, b, c],
            material: 1,
        }
    }

    #[test]
    fn a_page_round_trips_in_canonical_form_whatever_the_input_order() {
        let t1 = tri([0, 0, 0], [256, 0, 0], [0, 256, 0]);
        let t2 = tri([0, 0, 0], [0, 256, 0], [0, 0, 256]);
        let a = Page::from_tris(vec![t1, t2]).unwrap();
        let b = Page::from_tris(vec![
            Tri {
                v: [t2.v[1], t2.v[2], t2.v[0]],
                ..t2
            },
            t1,
        ])
        .unwrap();
        assert_eq!(a, b);
        let bytes = a.encode().unwrap();
        assert_eq!(Page::decode(&bytes).unwrap(), a);
        assert_eq!(bytes, b.encode().unwrap());
    }

    #[test]
    fn invalid_pages_are_typed_errors() {
        let t = tri([0, 0, 0], [256, 0, 0], [0, 256, 0]);
        assert_eq!(
            Page::from_tris(vec![tri([0, 0, 0], [0, 0, 0], [0, 256, 0])]),
            Err(MeshError::Degenerate { triangle: 0 })
        );
        assert_eq!(
            Page::from_tris(vec![tri([0, 0, 0], [256, 0, 0], [512, 0, 0])]),
            Err(MeshError::Degenerate { triangle: 0 })
        );
        assert_eq!(
            Page::from_tris(vec![t, t]),
            Err(MeshError::Duplicate { triangle: 1 })
        );
        // Two triangles walking the shared edge in the same direction.
        let u = tri([0, 0, 0], [256, 0, 0], [0, 0, 256]);
        assert_eq!(
            Page::from_tris(vec![t, u]),
            Err(MeshError::InconsistentWinding)
        );
        // Three triangles on one edge.
        let w = tri([256, 0, 0], [0, 0, 0], [0, 0, -256]);
        let x = tri([256, 0, 0], [0, 0, 0], [0, -256, 0]);
        assert!(Page::from_tris(vec![t, w, x]).is_err());
        assert_eq!(
            Page::from_tris(vec![tri([0, 0, 0], [i32::MAX, 0, 0], [0, 256, 0])]),
            Err(MeshError::CoordinateRange)
        );
    }

    #[test]
    fn hostile_bytes_never_panic_and_a_non_canonical_encoding_is_refused() {
        let page = Page::from_tris(vec![tri([0, 0, 0], [256, 0, 0], [0, 256, 0])]).unwrap();
        let good = page.encode().unwrap();
        for cut in 0..good.len() {
            assert!(Page::decode(&good[..cut]).is_err(), "cut {cut}");
        }
        let mut long = good.clone();
        long.push(0);
        assert_eq!(Page::decode(&long), Err(MeshError::Length));
        let mut ver = good.clone();
        ver[0] = 2;
        assert_eq!(Page::decode(&ver), Err(MeshError::Version(2)));
        let mut idx = good.clone();
        let at = 9 + 3 * 12;
        idx[at..at + 4].copy_from_slice(&99u32.to_le_bytes());
        assert!(matches!(
            Page::decode(&idx),
            Err(MeshError::IndexOutOfRange { .. })
        ));
        let mut huge = good.clone();
        huge[5..9].copy_from_slice(&u32::MAX.to_le_bytes());
        assert_eq!(Page::decode(&huge), Err(MeshError::TooLarge));
        // Every single byte flip is either refused or decodes to a valid canonical page.
        for i in 0..good.len() {
            let mut b = good.clone();
            b[i] ^= 0xFF;
            if let Ok(p) = Page::decode(&b) {
                assert_eq!(p.encode().unwrap(), b);
            }
        }
    }

    #[test]
    fn normals_point_along_the_winding_and_cancelling_vertices_get_zero() {
        let p = Page::from_tris(vec![tri([0, 0, 0], [256, 0, 0], [0, 256, 0])]).unwrap();
        for (_, n) in normals_q14(&p) {
            assert_eq!(n, [0, 0, 16384]);
        }
    }
}
