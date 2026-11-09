//! Opaque material index (DEC-066).
//!
//! Core stores this value per cell and knows nothing about what it means: no names, no packs,
//! no persistent content ids. The index is local to a world; persistent identity is resolved above Core
//! through the world's id tables. The width is declared in the save-format header.

/// Opaque per-world material index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MaterialIndex(u16);

impl MaterialIndex {
    /// Width of the index in bits; a persisted format declares this value in its header.
    pub const WIDTH_BITS: u32 = 16;

    /// Wraps a raw index.
    pub const fn from_raw(raw: u16) -> Self {
        Self(raw)
    }

    /// The raw index.
    pub const fn raw(self) -> u16 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_every_raw_value_at_the_edges() {
        for raw in [0, 1, 255, 256, u16::MAX - 1, u16::MAX] {
            assert_eq!(MaterialIndex::from_raw(raw).raw(), raw);
        }
        assert_eq!(MaterialIndex::WIDTH_BITS, u16::BITS);
    }
}
