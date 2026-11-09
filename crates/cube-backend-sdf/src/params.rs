//! Parameter schema of the SDF operation kinds (owned by this backend; parameters are opaque to
//! the Editing API). Schema version 1.
//!
//! Layout (little endian): `[version u8][kind-specific fields]`; a shape is
//! `[tag u8][fields]`: tag 1 sphere `centre x,y,z i64, radius_16 u32`; tag 2 box
//! `min x,y,z i64, max x,y,z i64`.
//!
//! * `ADD_GEOMETRY`: `shape, material u16`: union with the shape, inside cells get `material`.
//! * `REMOVE_GEOMETRY`: `shape`: subtraction; cells that become outside lose their material.
//! * `SET_MATERIAL`: `material u16`: every cell currently inside the surface in the write
//!   region gets `material`; the field is untouched.

use cube_core::VoxelPos;
use cube_editing_api::{EditingError, KindId, Parameters, SchemaVersion, kinds};

use crate::field::Shape;

/// Schema version written and understood.
pub const SCHEMA: SchemaVersion = SchemaVersion(1);

/// Largest sphere radius accepted, in 1/16 voxel (65 536 voxels); the field itself only reaches
/// 128 voxels from a surface, larger work is bounded by the region limit instead.
pub const MAX_RADIUS_16: u32 = 1 << 20;

/// A decoded SDF operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SdfOp {
    /// Union with a shape.
    Add {
        /// The shape.
        shape: Shape,
        /// Material of the new inside cells (0 keeps the existing material).
        material: u16,
    },
    /// Subtraction of a shape.
    Remove {
        /// The shape.
        shape: Shape,
    },
    /// Material assignment inside the surface.
    SetMaterial {
        /// The material.
        material: u16,
    },
}

fn params(bytes: Vec<u8>) -> Parameters {
    Parameters {
        schema: Some(SCHEMA),
        bytes,
    }
}

fn put_shape(b: &mut Vec<u8>, s: Shape) {
    match s {
        Shape::Sphere { centre, radius_16 } => {
            b.push(1);
            for v in [centre.x, centre.y, centre.z] {
                b.extend(v.to_le_bytes());
            }
            b.extend(radius_16.to_le_bytes());
        }
        Shape::Box { min, max } => {
            b.push(2);
            for v in [min.x, min.y, min.z, max.x, max.y, max.z] {
                b.extend(v.to_le_bytes());
            }
        }
    }
}

/// Parameters of `ADD_GEOMETRY`.
pub fn add(shape: Shape, material: u16) -> Parameters {
    let mut b = vec![1];
    put_shape(&mut b, shape);
    b.extend(material.to_le_bytes());
    params(b)
}

/// Parameters of `REMOVE_GEOMETRY`.
pub fn remove(shape: Shape) -> Parameters {
    let mut b = vec![1];
    put_shape(&mut b, shape);
    params(b)
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
            return Err(bad("truncated SDF parameters"));
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
    fn pos(&mut self) -> Result<VoxelPos, EditingError> {
        Ok(VoxelPos::new(self.i64()?, self.i64()?, self.i64()?))
    }
    fn shape(&mut self) -> Result<Shape, EditingError> {
        match self.u8()? {
            1 => {
                let centre = self.pos()?;
                let radius_16 = self.u32()?;
                if radius_16 > MAX_RADIUS_16 {
                    return Err(bad("sphere radius too large"));
                }
                Ok(Shape::Sphere { centre, radius_16 })
            }
            2 => {
                let (min, max) = (self.pos()?, self.pos()?);
                if min.x >= max.x || min.y >= max.y || min.z >= max.z {
                    return Err(bad("empty box"));
                }
                Ok(Shape::Box { min, max })
            }
            _ => Err(bad("unknown shape")),
        }
    }
    fn done(&self) -> Result<(), EditingError> {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err(bad("trailing bytes in SDF parameters"))
        }
    }
}

/// Decodes the parameters of an operation of `kind`. Unknown schema versions, truncated or
/// over-long input, malformed shapes and unknown kinds are controlled errors, never panics.
pub fn decode(kind: KindId, p: &Parameters) -> Result<SdfOp, EditingError> {
    if p.schema != Some(SCHEMA) {
        return Err(bad("unsupported SDF parameter schema"));
    }
    let mut r = Reader(&p.bytes);
    if r.u8()? != 1 {
        return Err(bad("unsupported SDF parameter version"));
    }
    let op = match kind {
        k if k == kinds::ADD_GEOMETRY => SdfOp::Add {
            shape: r.shape()?,
            material: r.u16()?,
        },
        k if k == kinds::REMOVE_GEOMETRY => SdfOp::Remove { shape: r.shape()? },
        k if k == kinds::SET_MATERIAL => SdfOp::SetMaterial { material: r.u16()? },
        other => return Err(EditingError::UnsupportedGeometryOperation(other)),
    };
    r.done()?;
    Ok(op)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sphere() -> Shape {
        Shape::Sphere {
            centre: VoxelPos::new(i64::MIN, -5, i64::MAX),
            radius_16: 80,
        }
    }

    #[test]
    fn every_operation_round_trips() {
        let b = Shape::Box {
            min: VoxelPos::new(-3, -3, -3),
            max: VoxelPos::new(3, 3, 3),
        };
        assert_eq!(
            decode(kinds::ADD_GEOMETRY, &add(sphere(), 7)),
            Ok(SdfOp::Add {
                shape: sphere(),
                material: 7
            })
        );
        assert_eq!(
            decode(kinds::REMOVE_GEOMETRY, &remove(b)),
            Ok(SdfOp::Remove { shape: b })
        );
        assert_eq!(
            decode(kinds::SET_MATERIAL, &set_material(65535)),
            Ok(SdfOp::SetMaterial { material: 65535 })
        );
    }

    #[test]
    fn malformed_input_is_a_controlled_error() {
        let good = add(sphere(), 1);
        for cut in 0..good.bytes.len() {
            let p = Parameters {
                schema: good.schema,
                bytes: good.bytes[..cut].to_vec(),
            };
            assert!(decode(kinds::ADD_GEOMETRY, &p).is_err(), "cut {cut}");
        }
        let mut long = good.clone();
        long.bytes.push(0);
        assert!(decode(kinds::ADD_GEOMETRY, &long).is_err());
        let mut wrong = good.clone();
        wrong.schema = Some(SchemaVersion(9));
        assert!(decode(kinds::ADD_GEOMETRY, &wrong).is_err());
        let empty = Shape::Box {
            min: VoxelPos::new(0, 0, 0),
            max: VoxelPos::new(0, 5, 5),
        };
        assert!(decode(kinds::REMOVE_GEOMETRY, &remove(empty)).is_err());
        let huge = Shape::Sphere {
            centre: VoxelPos::new(0, 0, 0),
            radius_16: u32::MAX,
        };
        assert!(decode(kinds::REMOVE_GEOMETRY, &remove(huge)).is_err());
        let mut tag = remove(sphere());
        tag.bytes[1] = 9;
        assert!(decode(kinds::REMOVE_GEOMETRY, &tag).is_err());
        assert_eq!(
            decode(kinds::TRANSFORM_GEOMETRY, &good),
            Err(EditingError::UnsupportedGeometryOperation(
                kinds::TRANSFORM_GEOMETRY
            ))
        );
    }
}
