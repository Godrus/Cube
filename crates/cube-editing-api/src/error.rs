//! Errors of the Editing API (contracts E-03..E-06, E-09). Codes 4000..5000 are never reused.

use std::fmt;

use cube_core::{ErrorCategory, ErrorCode, Severity};

use crate::types::{KindId, TransactionId, TransactionState};

/// A failure reported through the Editing API. Variants are the controlled errors the contracts
/// name; there is no catch-all.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum EditingError {
    /// The expected revisions of a transaction no longer hold.
    RevisionConflict,
    /// Another transaction holds or changed the same resources.
    TransactionConflict,
    /// The actor may not do this. Checked before any revision validation; leaves no state.
    PermissionDenied,
    /// Processing was cancelled while the operation needed it.
    ProcessingCancelled,
    /// Processing failed while the operation needed it.
    ProcessingFailed,
    /// The target representation does not support the operation kind.
    UnsupportedGeometryOperation(KindId),
    /// The operation mixes geometry types and carries no conversion policy that allows it.
    IncompatibleGeometryTypes,
    /// A conversion is required but not permitted by the operation's policy.
    ConversionRequired,
    /// A resolution (level) is invalid for the operation.
    InvalidResolution,
    /// The operation kind is not registered (controlled error for unknown kinds, E-03).
    UnknownKind(KindId),
    /// The operation is malformed (empty target list, duplicate layer, empty write region, ...).
    InvalidOperation(&'static str),
    /// The transaction is unknown or already released.
    UnknownTransaction(TransactionId),
    /// The call is not allowed in the transaction's current state.
    InvalidTransactionState {
        /// The transaction.
        transaction: TransactionId,
        /// Its state when the call was made.
        state: TransactionState,
    },
    /// Durable storage failed.
    PersistenceFailure,
    /// A stored item was not found.
    NotFound,
}

impl fmt::Display for EditingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RevisionConflict => f.write_str("revision conflict"),
            Self::TransactionConflict => f.write_str("transaction conflict"),
            Self::PermissionDenied => f.write_str("permission denied"),
            Self::ProcessingCancelled => f.write_str("processing cancelled"),
            Self::ProcessingFailed => f.write_str("processing failed"),
            Self::UnsupportedGeometryOperation(k) => {
                write!(f, "operation kind {} is not supported here", k.0)
            }
            Self::IncompatibleGeometryTypes => f.write_str("incompatible geometry types"),
            Self::ConversionRequired => f.write_str("conversion required"),
            Self::InvalidResolution => f.write_str("invalid resolution"),
            Self::UnknownKind(k) => write!(f, "unknown operation kind {}", k.0),
            Self::InvalidOperation(why) => write!(f, "invalid operation: {why}"),
            Self::UnknownTransaction(t) => write!(f, "unknown transaction {}", t.0),
            Self::InvalidTransactionState { transaction, state } => {
                write!(f, "transaction {} is {state:?}", transaction.0)
            }
            Self::PersistenceFailure => f.write_str("persistence failure"),
            Self::NotFound => f.write_str("not found"),
        }
    }
}

impl std::error::Error for EditingError {}

/// Name of the module that owns these codes.
pub const ERROR_MODULE: &str = "editing-api";

impl EditingError {
    /// Stable code (range 4000..5000 belongs to the Editing API).
    pub const fn code(&self) -> ErrorCode {
        ErrorCode(match self {
            Self::RevisionConflict => 4001,
            Self::TransactionConflict => 4002,
            Self::PermissionDenied => 4003,
            Self::ProcessingCancelled => 4004,
            Self::ProcessingFailed => 4005,
            Self::UnsupportedGeometryOperation(_) => 4006,
            Self::IncompatibleGeometryTypes => 4007,
            Self::ConversionRequired => 4008,
            Self::InvalidResolution => 4009,
            Self::UnknownKind(_) => 4010,
            Self::InvalidOperation(_) => 4011,
            Self::UnknownTransaction(_) => 4012,
            Self::InvalidTransactionState { .. } => 4013,
            Self::PersistenceFailure => 4014,
            Self::NotFound => 4015,
        })
    }

    /// Severity: persistence failures are critical, everything else is an ordinary error.
    pub const fn severity(&self) -> Severity {
        match self {
            Self::PersistenceFailure => Severity::Critical,
            _ => Severity::Error,
        }
    }

    /// Category of the error.
    pub const fn category(&self) -> ErrorCategory {
        match self {
            Self::RevisionConflict
            | Self::TransactionConflict
            | Self::InvalidTransactionState { .. } => ErrorCategory::Conflict,
            Self::PersistenceFailure | Self::ProcessingFailed => ErrorCategory::Capacity,
            _ => ErrorCategory::Validation,
        }
    }
}
