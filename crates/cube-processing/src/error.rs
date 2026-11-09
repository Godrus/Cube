//! Errors of Processing. Codes 3000..4000 belong to this crate and are never reused (C-19).

use std::fmt;

use cube_core::{CellKey, CoreError, ErrorCategory, ErrorCode, LayerId, Severity};

use crate::cancel::CancelReason;
use crate::provider::DerivedKind;

/// Which resource limit was hit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LimitKind {
    /// The queue of waiting tasks is full.
    QueueFull,
    /// The request needs more pinned bricks than the budget allows.
    PinnedBricks,
    /// The request needs more pinned memory than the budget allows.
    PinnedBytes,
}

/// Failure of a Processing operation. Errors are never reported as success: a failed task has the
/// outcome `Failed`, never a published value.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ProcessingError {
    /// A request carried an invalid level, region or an out-of-range value.
    InvalidRequest(CoreError),
    /// The layer does not exist in the asset.
    UnknownLayer(LayerId),
    /// The layer has no voxel/SDF cell store (for example a mesh layer).
    UnsupportedLayer(LayerId),
    /// A brick outside the captured area (core plus apron) was read.
    OutsideSnapshot(CellKey),
    /// A resource limit prevents the request.
    ResourceUnavailable(LimitKind),
    /// No provider is registered for the kind.
    UnknownKind(DerivedKind),
    /// A provider reported a failure or panicked.
    ProviderFailed(String),
    /// A task finished because it was cancelled.
    Cancelled(CancelReason),
    /// The executor is shut down.
    ShutDown,
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest(e) => write!(f, "invalid request: {e}"),
            Self::UnknownLayer(l) => write!(f, "unknown layer {}", l.raw()),
            Self::UnsupportedLayer(l) => write!(f, "layer {} has no cell store", l.raw()),
            Self::OutsideSnapshot(k) => write!(
                f,
                "brick {:?} of layer {} is outside the captured snapshot",
                k.brick(),
                k.layer().raw()
            ),
            Self::ResourceUnavailable(k) => write!(f, "resource limit reached: {k:?}"),
            Self::UnknownKind(k) => write!(f, "no provider for derived kind {}", k.0),
            Self::ProviderFailed(m) => write!(f, "provider failed: {m}"),
            Self::Cancelled(r) => write!(f, "cancelled: {r:?}"),
            Self::ShutDown => f.write_str("executor is shut down"),
        }
    }
}

impl std::error::Error for ProcessingError {}

impl From<CoreError> for ProcessingError {
    fn from(e: CoreError) -> Self {
        Self::InvalidRequest(e)
    }
}

/// Name of the module that owns these codes.
pub const ERROR_MODULE: &str = "processing";

impl ProcessingError {
    /// Stable code (range 3000..4000 belongs to Processing).
    pub const fn code(&self) -> ErrorCode {
        ErrorCode(match self {
            Self::InvalidRequest(_) => 3001,
            Self::UnknownLayer(_) => 3002,
            Self::UnsupportedLayer(_) => 3003,
            Self::OutsideSnapshot(_) => 3004,
            Self::ResourceUnavailable(_) => 3005,
            Self::UnknownKind(_) => 3006,
            Self::ProviderFailed(_) => 3007,
            Self::Cancelled(_) => 3008,
            Self::ShutDown => 3009,
        })
    }

    /// Severity: Processing failures are recoverable.
    pub const fn severity(&self) -> Severity {
        Severity::Error
    }

    /// Category of the error.
    pub const fn category(&self) -> ErrorCategory {
        match self {
            Self::ResourceUnavailable(_) => ErrorCategory::Capacity,
            Self::Cancelled(_) | Self::ShutDown => ErrorCategory::Conflict,
            _ => ErrorCategory::Validation,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cube_core::{BrickCoord, CoreError};

    #[test]
    fn codes_are_pinned_unique_and_in_the_processing_range() {
        let key = CellKey::new(LayerId::new(1), 0, BrickCoord::new(0, 0, 0)).unwrap();
        let all = [
            (ProcessingError::InvalidRequest(CoreError::Overflow), 3001),
            (ProcessingError::UnknownLayer(LayerId::new(1)), 3002),
            (ProcessingError::UnsupportedLayer(LayerId::new(1)), 3003),
            (ProcessingError::OutsideSnapshot(key), 3004),
            (
                ProcessingError::ResourceUnavailable(LimitKind::QueueFull),
                3005,
            ),
            (ProcessingError::UnknownKind(DerivedKind(1)), 3006),
            (ProcessingError::ProviderFailed(String::new()), 3007),
            (ProcessingError::Cancelled(CancelReason::Requested), 3008),
            (ProcessingError::ShutDown, 3009),
        ];
        let mut seen = std::collections::BTreeSet::new();
        for (e, code) in &all {
            assert_eq!(e.code(), ErrorCode(*code), "{e}");
            assert!(seen.insert(*code));
            assert!((3000..4000).contains(code));
            assert!(!e.to_string().is_empty());
        }
        assert_eq!(
            ProcessingError::ResourceUnavailable(LimitKind::PinnedBytes).category(),
            ErrorCategory::Capacity
        );
    }
}
