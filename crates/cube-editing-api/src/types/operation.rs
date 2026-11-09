//! E-03 EditingOperation.

use std::collections::BTreeSet;

use cube_core::{AssetId, LayerId, OperationId, Region};

use crate::error::EditingError;

/// Identifier of an operation kind. Kinds are extensible: the module that defines a kind owns its
/// id and parameter schema and registers it with the Editing API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct KindId(pub u32);

/// Version of a kind's parameter schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SchemaVersion(pub u16);

/// Ids of the kinds named by the contract. Stable and never reused; ids below 1000 are reserved
/// for these, other modules use ids from 1000 up.
pub mod kinds {
    use super::KindId;

    /// Set the material of voxels.
    pub const SET_MATERIAL: KindId = KindId(1);
    /// Add geometry.
    pub const ADD_GEOMETRY: KindId = KindId(2);
    /// Remove geometry.
    pub const REMOVE_GEOMETRY: KindId = KindId(3);
    /// Transform geometry.
    pub const TRANSFORM_GEOMETRY: KindId = KindId(4);
    /// Boolean operations (the concrete operation is in the parameters).
    pub const BOOLEAN: KindId = KindId(5);
    /// Sculpt tools.
    pub const SCULPT: KindId = KindId(6);
    /// Resample to another resolution.
    pub const RESAMPLE: KindId = KindId(7);
    /// Bake non-destructive state.
    pub const BAKE: KindId = KindId(8);
    /// Composite operations.
    pub const COMPOSITE: KindId = KindId(9);
}

/// Typed parameters of a kind, serialised by the owner of the kind. Opaque to the Editing API.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Parameters {
    /// Schema version the bytes follow.
    pub schema: Option<SchemaVersion>,
    /// The serialised parameters.
    pub bytes: Vec<u8>,
}

/// Whether a change of geometry type is permitted for the operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConversionPolicy {
    /// No conversion: mixed types are rejected.
    Forbid,
    /// Only exact conversions are allowed.
    AllowLossless,
    /// Lossy conversions are allowed.
    AllowLossy,
}

/// Destructive edits change the data; non-destructive ones add modifier state (E-08).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EditMode {
    /// The authoritative data changes.
    Destructive,
    /// Modifier state changes; geometry is derived.
    NonDestructive,
}

/// Declarative intent of one edit. Immutable once submitted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditingOperation {
    /// Identity of the operation (C-17).
    pub id: OperationId,
    /// Kind of the operation.
    pub kind: KindId,
    /// Asset the operation targets.
    pub asset: AssetId,
    /// Layers the operation targets.
    pub layers: Vec<LayerId>,
    /// Parameters, typed per kind.
    pub parameters: Parameters,
    /// Region the operation reads, if it reads beyond its write region.
    pub read_region: Option<Region>,
    /// Upper bound of what the operation may change; the actual changes are reported by the
    /// commit as dirty regions.
    pub write_region: Region,
    /// Conversion policy, `None` = `Forbid`.
    pub conversion: Option<ConversionPolicy>,
    /// Destructive or non-destructive.
    pub mode: EditMode,
}

impl EditingOperation {
    /// Checks the shape of the operation (not its meaning, which belongs to the kind's owner):
    /// at least one target layer, no layer twice, a non-empty write region, and read and write
    /// regions at the same level.
    pub fn validate(&self) -> Result<(), EditingError> {
        if self.layers.is_empty() {
            return Err(EditingError::InvalidOperation("no target layer"));
        }
        let unique: BTreeSet<_> = self.layers.iter().collect();
        if unique.len() != self.layers.len() {
            return Err(EditingError::InvalidOperation("duplicate target layer"));
        }
        if self.write_region.is_empty() {
            return Err(EditingError::InvalidOperation("empty write region"));
        }
        if let Some(r) = &self.read_region
            && r.level() != self.write_region.level()
        {
            return Err(EditingError::InvalidResolution);
        }
        Ok(())
    }

    /// Stable 64-bit fingerprint of the operation's content (FNV-1a over an explicit field
    /// encoding, independent of `Debug` output and of the compiler). Equal operations have equal
    /// fingerprints; it is used to tell a retry from an operation id reused with other content.
    /// Not cryptographic.
    pub fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut put = |b: &[u8]| {
            for x in b {
                h ^= u64::from(*x);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        };
        put(&self.id.scope().to_le_bytes());
        put(&self.id.sequence().to_le_bytes());
        put(&self.kind.0.to_le_bytes());
        put(&self.asset.raw().to_le_bytes());
        put(&(self.layers.len() as u64).to_le_bytes());
        for l in &self.layers {
            put(&l.raw().to_le_bytes());
        }
        put(&[u8::from(self.parameters.schema.is_some())]);
        put(&self.parameters.schema.map_or(0, |s| s.0).to_le_bytes());
        put(&(self.parameters.bytes.len() as u64).to_le_bytes());
        put(&self.parameters.bytes);
        let region = |put: &mut dyn FnMut(&[u8]), r: &Region| {
            put(&[r.level() as u8]);
            for v in [
                r.min().x,
                r.min().y,
                r.min().z,
                r.max().x,
                r.max().y,
                r.max().z,
            ] {
                put(&v.to_le_bytes());
            }
        };
        match &self.read_region {
            None => put(&[0]),
            Some(r) => {
                put(&[1]);
                region(&mut put, r);
            }
        }
        region(&mut put, &self.write_region);
        put(&[match self.conversion {
            None => 0,
            Some(ConversionPolicy::Forbid) => 1,
            Some(ConversionPolicy::AllowLossless) => 2,
            Some(ConversionPolicy::AllowLossy) => 3,
        }]);
        put(&[match self.mode {
            EditMode::Destructive => 0,
            EditMode::NonDestructive => 1,
        }]);
        h
    }

    /// The conversion policy in force (`Forbid` when none is given).
    pub fn conversion_policy(&self) -> ConversionPolicy {
        self.conversion.unwrap_or(ConversionPolicy::Forbid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cube_core::VoxelPos;

    fn op() -> EditingOperation {
        EditingOperation {
            id: OperationId::new(1, 1),
            kind: kinds::SET_MATERIAL,
            asset: AssetId::new(1),
            layers: vec![LayerId::new(1)],
            parameters: Parameters::default(),
            read_region: None,
            write_region: Region::new(0, VoxelPos::new(0, 0, 0), VoxelPos::new(1, 1, 1)).unwrap(),
            conversion: None,
            mode: EditMode::Destructive,
        }
    }

    #[test]
    fn a_well_formed_operation_validates() {
        assert_eq!(op().validate(), Ok(()));
        assert_eq!(op().conversion_policy(), ConversionPolicy::Forbid);
    }

    #[test]
    fn malformed_operations_are_rejected_with_a_reason() {
        let mut o = op();
        o.layers.clear();
        assert!(matches!(
            o.validate(),
            Err(EditingError::InvalidOperation(_))
        ));
        let mut o = op();
        o.layers = vec![LayerId::new(1), LayerId::new(1)];
        assert!(matches!(
            o.validate(),
            Err(EditingError::InvalidOperation(_))
        ));
        let mut o = op();
        o.write_region = Region::new(0, VoxelPos::new(0, 0, 0), VoxelPos::new(0, 1, 1)).unwrap();
        assert!(matches!(
            o.validate(),
            Err(EditingError::InvalidOperation(_))
        ));
        let mut o = op();
        o.read_region =
            Some(Region::new(1, VoxelPos::new(0, 0, 0), VoxelPos::new(1, 1, 1)).unwrap());
        assert_eq!(o.validate(), Err(EditingError::InvalidResolution));
    }

    #[test]
    fn negative_and_extreme_regions_are_valid_targets() {
        let mut o = op();
        o.write_region = Region::new(
            0,
            VoxelPos::new(i64::MIN, -5, -1),
            VoxelPos::new(i64::MIN + 1, 0, 0),
        )
        .unwrap();
        assert_eq!(o.validate(), Ok(()));
    }

    #[test]
    fn kind_ids_are_pinned() {
        let ids = [
            kinds::SET_MATERIAL,
            kinds::ADD_GEOMETRY,
            kinds::REMOVE_GEOMETRY,
            kinds::TRANSFORM_GEOMETRY,
            kinds::BOOLEAN,
            kinds::SCULPT,
            kinds::RESAMPLE,
            kinds::BAKE,
            kinds::COMPOSITE,
        ];
        assert_eq!(ids.map(|k| k.0), [1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }
}
