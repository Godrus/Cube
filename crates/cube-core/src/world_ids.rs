//! World identity value types (CORE-16, support for ACR-004 parts B and C).
//!
//! Plain newtypes with no behaviour: they carry no generation or persistence logic (that lives in
//! WorldGen and Persistence). Equality, ordering and hashing are derived from the raw integer, so
//! comparison is exact and platform independent. The byte form is fixed-width little-endian so a
//! persistence layer can store it without Core defining any file format.

/// Seed of a world's procedural base. Opaque: its meaning belongs to the generator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WorldSeed(u64);

impl WorldSeed {
    /// Wraps a raw seed.
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }
    /// The raw seed.
    pub const fn raw(self) -> u64 {
        self.0
    }
    /// Fixed-width little-endian bytes.
    pub const fn to_le_bytes(self) -> [u8; 8] {
        self.0.to_le_bytes()
    }
    /// Inverse of [`WorldSeed::to_le_bytes`].
    pub const fn from_le_bytes(bytes: [u8; 8]) -> Self {
        Self(u64::from_le_bytes(bytes))
    }
}

/// Version of the world generator that produced (or will reproduce) a world's untouched base.
/// Versions are compared by value only; whether two versions are compatible is not Core's concern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GeneratorVersion(u32);

impl GeneratorVersion {
    /// Wraps a raw version number.
    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }
    /// The raw version number.
    pub const fn raw(self) -> u32 {
        self.0
    }
    /// Fixed-width little-endian bytes.
    pub const fn to_le_bytes(self) -> [u8; 4] {
        self.0.to_le_bytes()
    }
    /// Inverse of [`GeneratorVersion::to_le_bytes`].
    pub const fn from_le_bytes(bytes: [u8; 4]) -> Self {
        Self(u32::from_le_bytes(bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_roundtrips_raw_and_bytes_for_extremes() {
        for raw in [0, 1, u64::MAX, 0x0123_4567_89AB_CDEF] {
            let s = WorldSeed::new(raw);
            assert_eq!(s.raw(), raw);
            assert_eq!(WorldSeed::from_le_bytes(s.to_le_bytes()), s);
        }
        assert_eq!(WorldSeed::new(1).to_le_bytes(), [1, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn version_roundtrips_and_orders_by_value() {
        for raw in [0, 1, u32::MAX] {
            let v = GeneratorVersion::new(raw);
            assert_eq!(v.raw(), raw);
            assert_eq!(GeneratorVersion::from_le_bytes(v.to_le_bytes()), v);
        }
        assert!(GeneratorVersion::new(1) < GeneratorVersion::new(2));
        assert_eq!(GeneratorVersion::new(1).to_le_bytes(), [1, 0, 0, 0]);
    }

    #[test]
    fn distinct_seeds_are_distinct_and_equal_seeds_hash_alike() {
        use std::collections::HashSet;
        let set: HashSet<WorldSeed> = [1, 2, 2, 3].into_iter().map(WorldSeed::new).collect();
        assert_eq!(set.len(), 3);
    }
}
