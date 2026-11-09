//! Parameter schema of the voxel operation kinds (owned by this backend, E-03: parameters are
//! opaque to the Editing API). Schema version 1. Material `0` is air.
//!
//! Layout (little endian): `[version u8][kind-specific fields]`
//!
//! * `SET_MATERIAL`: `material u16, has_filter u8, filter u16`; sets every voxel of the write
//!   region to `material`, or only the voxels currently equal to `filter` (replace material).
//! * `ADD_GEOMETRY`: `material u16`; fills only the air voxels of the write region.
//! * `REMOVE_GEOMETRY`: `has_filter u8, filter u16`; clears the voxels (to air), or only those
//!   currently equal to `filter`.
//! * `TRANSFORM_GEOMETRY`: `dest x,y,z i64, rotation u8, mode u8`; copies (`mode 0`) or moves
//!   (`mode 1`) the voxels of the read region so that their rotated box starts at `dest`.

use cube_core::{Rotation, VoxelPos};
use cube_editing_api::{EditingError, KindId, Parameters, SchemaVersion, kinds};

/// Schema version written and understood.
pub const SCHEMA: SchemaVersion = SchemaVersion(1);

/// Copy or move.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformMode {
    /// The source stays.
    Copy,
    /// The source is cleared.
    Move,
}

/// A decoded voxel operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoxelOp {
    /// Set or replace a material.
    Set {
        /// The new material.
        material: u16,
        /// Replace only voxels with this material.
        only_where: Option<u16>,
    },
    /// Fill air voxels.
    Add {
        /// Material to fill with.
        material: u16,
    },
    /// Clear voxels.
    Remove {
        /// Clear only voxels with this material.
        only_where: Option<u16>,
    },
    /// Copy or move a region.
    Transform {
        /// Where the rotated box starts.
        dest: VoxelPos,
        /// Axis rotation applied to the copied box.
        rotation: Rotation,
        /// Copy or move.
        mode: TransformMode,
    },
}

fn params(bytes: Vec<u8>) -> Parameters {
    Parameters {
        schema: Some(SCHEMA),
        bytes,
    }
}

fn filter(v: Option<u16>) -> [u8; 3] {
    let f = v.unwrap_or(0).to_le_bytes();
    [u8::from(v.is_some()), f[0], f[1]]
}

/// Parameters of `SET_MATERIAL`.
pub fn set_material(material: u16, only_where: Option<u16>) -> Parameters {
    let mut b = vec![1];
    b.extend(material.to_le_bytes());
    b.extend(filter(only_where));
    params(b)
}

/// Parameters of `ADD_GEOMETRY`.
pub fn add(material: u16) -> Parameters {
    let mut b = vec![1];
    b.extend(material.to_le_bytes());
    params(b)
}

/// Parameters of `REMOVE_GEOMETRY`.
pub fn remove(only_where: Option<u16>) -> Parameters {
    let mut b = vec![1];
    b.extend(filter(only_where));
    params(b)
}

/// Parameters of `TRANSFORM_GEOMETRY`.
pub fn transform(dest: VoxelPos, rotation: Rotation, mode: TransformMode) -> Parameters {
    let mut b = vec![1];
    for v in [dest.x, dest.y, dest.z] {
        b.extend(v.to_le_bytes());
    }
    b.push(rotation.index());
    b.push(match mode {
        TransformMode::Copy => 0,
        TransformMode::Move => 1,
    });
    params(b)
}

fn bad(why: &'static str) -> EditingError {
    EditingError::InvalidOperation(why)
}

struct Reader<'a>(&'a [u8]);

impl Reader<'_> {
    fn take<const N: usize>(&mut self) -> Result<[u8; N], EditingError> {
        if self.0.len() < N {
            return Err(bad("truncated voxel parameters"));
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
    fn i64(&mut self) -> Result<i64, EditingError> {
        Ok(i64::from_le_bytes(self.take()?))
    }
    fn filter(&mut self) -> Result<Option<u16>, EditingError> {
        let has = self.u8()?;
        let v = self.u16()?;
        match has {
            0 => Ok(None),
            1 => Ok(Some(v)),
            _ => Err(bad("bad filter flag")),
        }
    }
    fn done(&self) -> Result<(), EditingError> {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err(bad("trailing bytes in voxel parameters"))
        }
    }
}

/// Decodes the parameters of an operation of `kind`. Unknown schema versions, truncated or
/// over-long input and unknown kinds are controlled errors, never panics.
pub fn decode(kind: KindId, p: &Parameters) -> Result<VoxelOp, EditingError> {
    if p.schema != Some(SCHEMA) {
        return Err(bad("unsupported voxel parameter schema"));
    }
    let mut r = Reader(&p.bytes);
    if r.u8()? != 1 {
        return Err(bad("unsupported voxel parameter version"));
    }
    let op = match kind {
        k if k == kinds::SET_MATERIAL => VoxelOp::Set {
            material: r.u16()?,
            only_where: r.filter()?,
        },
        k if k == kinds::ADD_GEOMETRY => VoxelOp::Add { material: r.u16()? },
        k if k == kinds::REMOVE_GEOMETRY => VoxelOp::Remove {
            only_where: r.filter()?,
        },
        k if k == kinds::TRANSFORM_GEOMETRY => {
            let dest = VoxelPos::new(r.i64()?, r.i64()?, r.i64()?);
            let rotation = Rotation::from_index(r.u8()?).map_err(|_| bad("bad rotation index"))?;
            let mode = match r.u8()? {
                0 => TransformMode::Copy,
                1 => TransformMode::Move,
                _ => return Err(bad("bad transform mode")),
            };
            VoxelOp::Transform {
                dest,
                rotation,
                mode,
            }
        }
        other => return Err(EditingError::UnsupportedGeometryOperation(other)),
    };
    r.done()?;
    Ok(op)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_operation_round_trips() {
        let rot = Rotation::from_index(7).unwrap();
        let cases = [
            (
                kinds::SET_MATERIAL,
                set_material(9, None),
                VoxelOp::Set {
                    material: 9,
                    only_where: None,
                },
            ),
            (
                kinds::SET_MATERIAL,
                set_material(9, Some(2)),
                VoxelOp::Set {
                    material: 9,
                    only_where: Some(2),
                },
            ),
            (
                kinds::ADD_GEOMETRY,
                add(65535),
                VoxelOp::Add { material: 65535 },
            ),
            (
                kinds::REMOVE_GEOMETRY,
                remove(Some(0)),
                VoxelOp::Remove {
                    only_where: Some(0),
                },
            ),
            (
                kinds::REMOVE_GEOMETRY,
                remove(None),
                VoxelOp::Remove { only_where: None },
            ),
            (
                kinds::TRANSFORM_GEOMETRY,
                transform(
                    VoxelPos::new(i64::MIN, -5, i64::MAX),
                    rot,
                    TransformMode::Move,
                ),
                VoxelOp::Transform {
                    dest: VoxelPos::new(i64::MIN, -5, i64::MAX),
                    rotation: rot,
                    mode: TransformMode::Move,
                },
            ),
        ];
        for (k, p, want) in cases {
            assert_eq!(decode(k, &p), Ok(want));
        }
    }

    #[test]
    fn malformed_input_is_a_controlled_error() {
        let good = set_material(1, None);
        let mut long = good.clone();
        long.bytes.push(0);
        let mut wrong_version = good.clone();
        wrong_version.bytes[0] = 2;
        let mut no_schema = good.clone();
        no_schema.schema = None;
        let mut bad_flag = good.clone();
        bad_flag.bytes[3] = 7;
        for p in [
            long,
            wrong_version,
            no_schema,
            bad_flag,
            Parameters::default(),
        ] {
            assert!(matches!(
                decode(kinds::SET_MATERIAL, &p),
                Err(EditingError::InvalidOperation(_))
            ));
        }
        for cut in 0..good.bytes.len() {
            let mut t = good.clone();
            t.bytes.truncate(cut);
            assert!(decode(kinds::SET_MATERIAL, &t).is_err());
        }
        assert_eq!(
            decode(kinds::SCULPT, &good),
            Err(EditingError::UnsupportedGeometryOperation(kinds::SCULPT))
        );
        let mut bad_rot = transform(
            VoxelPos::new(0, 0, 0),
            Rotation::IDENTITY,
            TransformMode::Copy,
        );
        let n = bad_rot.bytes.len();
        bad_rot.bytes[n - 2] = 99;
        assert!(decode(kinds::TRANSFORM_GEOMETRY, &bad_rot).is_err());
    }
}
