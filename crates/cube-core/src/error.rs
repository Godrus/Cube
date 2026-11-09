//! Error type of the Geometry Core.

use std::fmt;

use crate::ids::LayerId;

/// Errors returned by Core constructors and conversions. Never panics on bad input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CoreError {
    /// A level is outside the supported range (negative levels are reserved, DEC-064).
    InvalidLevel(i8),
    /// An integer operation would leave the `i64` range (or a shift is too large).
    Overflow,
    /// A local brick coordinate is not in `0..16`.
    InvalidLocalCoord,
    /// A region has `min > max` on some axis.
    InvalidRegion,
    /// A transform parameter is out of range.
    InvalidTransform,
    /// Two layers of one asset share an id.
    DuplicateLayer(LayerId),
    /// A revision or epoch counter reached the end of its range (critical; never wraps).
    RevisionOverflow,
    /// The same key appears twice where keys must be unique.
    DuplicateKey,
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLevel(l) => write!(
                f,
                "invalid level {l} (levels are >= 0; negative levels are reserved)"
            ),
            Self::Overflow => f.write_str("integer overflow"),
            Self::InvalidLocalCoord => f.write_str("local coordinate outside 0..16"),
            Self::InvalidRegion => f.write_str("region has min greater than max"),
            Self::InvalidTransform => f.write_str("transform parameter out of range"),
            Self::RevisionOverflow => f.write_str("revision counter overflow"),
            Self::DuplicateKey => f.write_str("duplicate key"),
            Self::DuplicateLayer(id) => write!(f, "duplicate layer id {}", id.raw()),
        }
    }
}

impl std::error::Error for CoreError {}
