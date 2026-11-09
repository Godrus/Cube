//! Public data types of the Editing API (API-01, contracts E-03..E-06).
//!
//! Plain data only: no behaviour beyond construction checks and state-table lookups. Parameter
//! schemas of operation kinds belong to the modules that define the kinds (E-03); here they are
//! an opaque, versioned byte block.

mod conflict;
mod operation;
mod result;
mod transaction;

pub use conflict::{ConflictResult, ConflictScope, Suggestion};
pub use operation::{
    ConversionPolicy, EditMode, EditingOperation, KindId, Parameters, SchemaVersion, kinds,
};
pub use result::{EditingResult, OperationStatus, Success};
pub use transaction::{
    CommitPolicy, RollbackPolicy, TransactionHandle, TransactionId, TransactionSpec,
    TransactionState, TransactionSummary,
};
