//! Numeric ids local to a world. They are not persistent content identity (that lives above Core).

/// Identifier of a geometry asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AssetId(u64);

/// Identifier of a layer inside an asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LayerId(u64);

impl AssetId {
    /// Wraps a raw value.
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }
    /// The raw value.
    pub const fn raw(self) -> u64 {
        self.0
    }
}

impl LayerId {
    /// Wraps a raw value.
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }
    /// The raw value.
    pub const fn raw(self) -> u64 {
        self.0
    }
}
