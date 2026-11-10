//! Parameter schema of the mesh operation kinds (owned by this backend; parameters are opaque to
//! the Editing API). Schema version 1. Coordinates are absolute, in 1/256 voxel unless stated.
//!
//! Layout (little endian): `[version u8][kind-specific fields]`.
//!
//! * `ADD_GEOMETRY`, tag 1 box: `min x,y,z i64, max x,y,z i64` in **voxels**, `material u16`: the
//!   12 triangles of the box, outward counter-clockwise; tag 2 triangles: `count u32` (at most
//!   [`MAX_ADD_TRIANGLES`]) then per triangle `3 x (x,y,z i64)` and `material u16`.
//! * `REMOVE_GEOMETRY`: no fields; removes every triangle whose centroid lies in the write region.
//! * `SET_MATERIAL`: `material u16`; sets the material of every triangle whose centroid lies in
//!   the write region.

use cube_editing_api::{EditingError, KindId, Parameters, SchemaVersion, kinds};

use crate::page::MAX_COORD;

/// Schema version written and understood.
pub const SCHEMA: SchemaVersion = SchemaVersion(1);
/// Most triangles one add operation may carry.
pub const MAX_ADD_TRIANGLES: usize = 4096;

/// A triangle in absolute coordinates, 1/256 voxel.
pub type AbsTri = ([[i64; 3]; 3], u16);

/// A decoded mesh operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MeshOp {
    /// Add triangles.
    Add(Vec<AbsTri>),
    /// Remove the triangles whose centroid is in the write region.
    Remove,
    /// Set the material of the triangles whose centroid is in the write region.
    SetMaterial(u16),
}

fn params(bytes: Vec<u8>) -> Parameters {
    Parameters {
        schema: Some(SCHEMA),
        bytes,
    }
}

/// Parameters of `ADD_GEOMETRY` for an axis-aligned box `[min, max)` in voxels.
pub fn add_box(min: [i64; 3], max: [i64; 3], material: u16) -> Parameters {
    let mut b = vec![1, 1];
    for v in min.iter().chain(&max) {
        b.extend(v.to_le_bytes());
    }
    b.extend(material.to_le_bytes());
    params(b)
}

/// Parameters of `ADD_GEOMETRY` for explicit triangles (1/256 voxel).
pub fn add_triangles(tris: &[AbsTri]) -> Parameters {
    let mut b = vec![1, 2];
    b.extend((tris.len() as u32).to_le_bytes());
    for (v, m) in tris {
        for p in v {
            for c in p {
                b.extend(c.to_le_bytes());
            }
        }
        b.extend(m.to_le_bytes());
    }
    params(b)
}

/// Parameters of `REMOVE_GEOMETRY`.
pub fn remove() -> Parameters {
    params(vec![1])
}

/// Parameters of `SET_MATERIAL`.
pub fn set_material(material: u16) -> Parameters {
    let mut b = vec![1];
    b.extend(material.to_le_bytes());
    params(b)
}

fn bad(why: &'static str) -> EditingError {
    EditingError::InvalidOperation(why)
}

struct Reader<'a>(&'a [u8]);

impl Reader<'_> {
    fn take<const N: usize>(&mut self) -> Result<[u8; N], EditingError> {
        if self.0.len() < N {
            return Err(bad("truncated mesh parameters"));
        }
        let (a, rest) = self.0.split_at(N);
        self.0 = rest;
        Ok(a.try_into().expect("length checked"))
    }
    fn u8(&mut self) -> Result<u8, EditingError> {
        Ok(self.take::<1>()?[0])
    }
    fn u16(&mut self) -> Result<u16, EditingError> {
        Ok(u16::from_le_bytes(self.take()?))
    }
    fn u32(&mut self) -> Result<u32, EditingError> {
        Ok(u32::from_le_bytes(self.take()?))
    }
    fn i64(&mut self) -> Result<i64, EditingError> {
        Ok(i64::from_le_bytes(self.take()?))
    }
    fn p3(&mut self) -> Result<[i64; 3], EditingError> {
        Ok([self.i64()?, self.i64()?, self.i64()?])
    }
    fn done(&self) -> Result<(), EditingError> {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err(bad("trailing bytes in mesh parameters"))
        }
    }
}

/// The twelve triangles of the box, outward counter-clockwise, in 1/256 voxel.
fn box_tris(min: [i64; 3], max: [i64; 3], material: u16) -> Result<Vec<AbsTri>, EditingError> {
    let mut lo = [0i64; 3];
    let mut hi = [0i64; 3];
    for a in 0..3 {
        if min[a] >= max[a] {
            return Err(bad("empty box"));
        }
        lo[a] = min[a]
            .checked_mul(256)
            .ok_or(bad("box coordinate too large"))?;
        hi[a] = max[a]
            .checked_mul(256)
            .ok_or(bad("box coordinate too large"))?;
    }
    let c = |x: usize, y: usize, z: usize| {
        [
            if x == 0 { lo[0] } else { hi[0] },
            if y == 0 { lo[1] } else { hi[1] },
            if z == 0 { lo[2] } else { hi[2] },
        ]
    };
    // Each face as a counter-clockwise quad seen from outside, split into two triangles.
    let quads = [
        [c(1, 0, 0), c(1, 1, 0), c(1, 1, 1), c(1, 0, 1)], // +x
        [c(0, 0, 0), c(0, 0, 1), c(0, 1, 1), c(0, 1, 0)], // -x
        [c(0, 1, 0), c(0, 1, 1), c(1, 1, 1), c(1, 1, 0)], // +y
        [c(0, 0, 0), c(1, 0, 0), c(1, 0, 1), c(0, 0, 1)], // -y
        [c(0, 0, 1), c(1, 0, 1), c(1, 1, 1), c(0, 1, 1)], // +z
        [c(0, 0, 0), c(0, 1, 0), c(1, 1, 0), c(1, 0, 0)], // -z
    ];
    let mut out = Vec::with_capacity(12);
    for q in quads {
        out.push(([q[0], q[1], q[2]], material));
        out.push(([q[0], q[2], q[3]], material));
    }
    Ok(out)
}

/// Decodes the parameters of an operation of `kind`. Unknown schema versions, truncated or
/// over-long input, malformed primitives and unknown kinds are controlled errors, never panics.
pub fn decode(kind: KindId, p: &Parameters) -> Result<MeshOp, EditingError> {
    if p.schema != Some(SCHEMA) {
        return Err(bad("unsupported mesh parameter schema"));
    }
    let mut r = Reader(&p.bytes);
    if r.u8()? != 1 {
        return Err(bad("unsupported mesh parameter version"));
    }
    let op = match kind {
        k if k == kinds::ADD_GEOMETRY => match r.u8()? {
            1 => {
                let (min, max) = (r.p3()?, r.p3()?);
                MeshOp::Add(box_tris(min, max, r.u16()?)?)
            }
            2 => {
                let n = r.u32()? as usize;
                if n == 0 || n > MAX_ADD_TRIANGLES {
                    return Err(bad("triangle count out of range"));
                }
                let mut tris = Vec::with_capacity(n);
                for _ in 0..n {
                    let v = [r.p3()?, r.p3()?, r.p3()?];
                    if v.iter().flatten().any(|c| c.abs() > MAX_COORD * 1024) {
                        return Err(bad("triangle coordinate too large"));
                    }
                    tris.push((v, r.u16()?));
                }
                MeshOp::Add(tris)
            }
            _ => return Err(bad("unknown primitive")),
        },
        k if k == kinds::REMOVE_GEOMETRY => MeshOp::Remove,
        k if k == kinds::SET_MATERIAL => MeshOp::SetMaterial(r.u16()?),
        other => return Err(EditingError::UnsupportedGeometryOperation(other)),
    };
    r.done()?;
    Ok(op)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operations_round_trip_and_a_box_has_twelve_outward_triangles() {
        let MeshOp::Add(t) =
            decode(kinds::ADD_GEOMETRY, &add_box([0, 0, 0], [2, 3, 4], 5)).unwrap()
        else {
            panic!()
        };
        assert_eq!(t.len(), 12);
        // Every triangle's normal points away from the centre of the box.
        let centre = [256i128, 384, 512];
        for (v, _) in &t {
            let a = v[0].map(i128::from);
            let u = [
                v[1][0] as i128 - a[0],
                v[1][1] as i128 - a[1],
                v[1][2] as i128 - a[2],
            ];
            let w = [
                v[2][0] as i128 - a[0],
                v[2][1] as i128 - a[1],
                v[2][2] as i128 - a[2],
            ];
            let n = [
                u[1] * w[2] - u[2] * w[1],
                u[2] * w[0] - u[0] * w[2],
                u[0] * w[1] - u[1] * w[0],
            ];
            let to = [a[0] - centre[0], a[1] - centre[1], a[2] - centre[2]];
            assert!(n[0] * to[0] + n[1] * to[1] + n[2] * to[2] > 0);
        }
        let tris = [([[0, 0, 0], [256, 0, 0], [0, 256, 0]], 3u16)];
        assert_eq!(
            decode(kinds::ADD_GEOMETRY, &add_triangles(&tris)),
            Ok(MeshOp::Add(tris.to_vec()))
        );
        assert_eq!(
            decode(kinds::REMOVE_GEOMETRY, &remove()),
            Ok(MeshOp::Remove)
        );
        assert_eq!(
            decode(kinds::SET_MATERIAL, &set_material(9)),
            Ok(MeshOp::SetMaterial(9))
        );
    }

    #[test]
    fn malformed_input_is_a_controlled_error() {
        let good = add_box([0, 0, 0], [1, 1, 1], 1);
        for cut in 0..good.bytes.len() {
            let p = Parameters {
                schema: good.schema,
                bytes: good.bytes[..cut].to_vec(),
            };
            assert!(decode(kinds::ADD_GEOMETRY, &p).is_err(), "cut {cut}");
        }
        assert!(decode(kinds::ADD_GEOMETRY, &add_box([1, 0, 0], [1, 1, 1], 1)).is_err());
        assert!(
            decode(
                kinds::ADD_GEOMETRY,
                &add_box([0, 0, 0], [i64::MAX, 1, 1], 1)
            )
            .is_err()
        );
        assert!(decode(kinds::ADD_GEOMETRY, &add_triangles(&[])).is_err());
        let mut long = remove();
        long.bytes.push(7);
        assert!(decode(kinds::REMOVE_GEOMETRY, &long).is_err());
        assert_eq!(
            decode(kinds::TRANSFORM_GEOMETRY, &remove()),
            Err(EditingError::UnsupportedGeometryOperation(
                kinds::TRANSFORM_GEOMETRY
            ))
        );
    }
}
