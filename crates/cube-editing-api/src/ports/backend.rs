//! E-07 GeometryBackend and BackendCapabilities.

use std::collections::BTreeSet;

use cube_core::{CellKey, CellState, LayerKind, Region, WriteSet};
use cube_processing::Snapshot;

use crate::error::EditingError;
use crate::types::{EditingOperation, KindId};

/// What a backend can do. Kinds not listed are rejected with `UnsupportedGeometryOperation`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendCapabilities {
    /// The layer representation the backend serves.
    pub layer_kind: LayerKind,
    /// Operation kinds it can apply.
    pub kinds: BTreeSet<KindId>,
    /// `true` if it implements Boolean operations natively.
    pub native_boolean: bool,
    /// `true` if it supports resampling.
    pub resample: bool,
    /// `true` if it implements `validate`.
    pub validate: bool,
}

impl BackendCapabilities {
    /// `true` if the backend can apply `kind`.
    pub fn supports(&self, kind: KindId) -> bool {
        self.kinds.contains(&kind)
    }
}

/// Result of validating a region of a layer.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ValidationReport {
    /// Bricks found inconsistent, in ascending order. Empty = valid.
    pub invalid: Vec<CellKey>,
}

impl ValidationReport {
    /// `true` if nothing was found inconsistent.
    pub fn is_valid(&self) -> bool {
        self.invalid.is_empty()
    }
}

/// Read access to the private overlay of a transaction: the committed state plus the writes that
/// transaction has staged so far. It has no way to change anything.
pub trait OverlayRead {
    /// The state of a brick as this transaction sees it.
    fn cell(&self, key: CellKey) -> Result<CellState, EditingError>;
}

/// The only way Orchestration reaches representation-specific logic. Backends never call tools
/// or Orchestration. Mutation happens only through the returned write sets, which Orchestration
/// stages in the overlay and commits through Core's gate; a backend cannot write anything by
/// construction (it receives read-only views).
pub trait GeometryBackend: Send + Sync {
    /// What the backend supports.
    fn capabilities(&self) -> &BackendCapabilities;

    /// Computes the writes of `operation` against the overlay (and, for work done on workers, an
    /// immutable Processing snapshot). Must not convert between representations implicitly and
    /// must report only writes inside the operation's write region. Returns a controlled error
    /// instead of panicking.
    fn apply(
        &self,
        operation: &EditingOperation,
        overlay: &dyn OverlayRead,
        snapshot: Option<&Snapshot>,
    ) -> Result<Vec<WriteSet>, EditingError>;

    /// Checks the consistency of a region of a layer of this representation.
    fn validate(
        &self,
        overlay: &dyn OverlayRead,
        region: &Region,
    ) -> Result<ValidationReport, EditingError>;
}
