//! Opaque material index (DEC-066, MAT-01).
//!
//! Core stores this value per cell and knows nothing about what it means: no names, no packs,
//! no persistent content ids. The index is local to a world; persistent identity is resolved above Core
//! through the world's id tables.
//!
//! The runtime width is 32 bits (MAT-01, owner decision 2026-11-12: adaptive material width).
//! Storage is adaptive: a brick channel keeps the values in 8, 16 or 32 bits, whichever is the
//! narrowest that fits (see the channel module), so a typical brick with few materials costs one
//! byte per cell.

/// Opaque per-world material index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MaterialIndex(u32);

impl MaterialIndex {
    /// Width of the runtime index in bits. Storage and persisted formats use the narrowest of
    /// 8, 16 and 32 bits that fits the values they hold.
    pub const WIDTH_BITS: u32 = 32;

    /// Wraps a raw index.
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// The raw index.
    pub const fn raw(self) -> u32 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_every_raw_value_at_the_edges() {
        for raw in [0, 1, 255, 256, 65_535, 65_536, u32::MAX - 1, u32::MAX] {
            assert_eq!(MaterialIndex::from_raw(raw).raw(), raw);
        }
        assert_eq!(MaterialIndex::WIDTH_BITS, u32::BITS);
    }
}
