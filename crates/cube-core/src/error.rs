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
    /// An imported view is inconsistent (cells outside bounds, cells on a mesh layer, wrong payload kind).
    InvalidView,
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
            Self::InvalidView => f.write_str("inconsistent asset view"),
            Self::DuplicateLayer(id) => write!(f, "duplicate layer id {}", id.raw()),
        }
    }
}

impl std::error::Error for CoreError {}

/// Stable numeric code of an error. Codes are never reused or renumbered (contract C-19).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ErrorCode(pub u16);

/// How serious an error is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// An expected, recoverable failure (bad input, conflict).
    Error,
    /// A critical failure that must not be ignored (for example counter overflow).
    Critical,
}

/// Broad class of an error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// The input is invalid.
    Validation,
    /// The request conflicts with the current state.
    Conflict,
    /// A numeric or capacity limit was reached.
    Capacity,
}

/// Name of the module that owns these codes.
pub const ERROR_MODULE: &str = "core";

impl CoreError {
    /// Stable code (range 1000..2000 belongs to Core errors).
    pub const fn code(&self) -> ErrorCode {
        ErrorCode(match self {
            Self::InvalidLevel(_) => 1001,
            Self::Overflow => 1002,
            Self::InvalidLocalCoord => 1003,
            Self::InvalidRegion => 1004,
            Self::InvalidTransform => 1005,
            Self::DuplicateLayer(_) => 1006,
            Self::RevisionOverflow => 1007,
            Self::DuplicateKey => 1008,
            Self::InvalidView => 1009,
        })
    }

    /// Severity: only a revision overflow is critical.
    pub const fn severity(&self) -> Severity {
        match self {
            Self::RevisionOverflow => Severity::Critical,
            _ => Severity::Error,
        }
    }

    /// Category of the error.
    pub const fn category(&self) -> ErrorCategory {
        match self {
            Self::Overflow | Self::RevisionOverflow => ErrorCategory::Capacity,
            Self::DuplicateLayer(_) | Self::DuplicateKey => ErrorCategory::Conflict,
            _ => ErrorCategory::Validation,
        }
    }
}
