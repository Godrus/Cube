//! Provider interface (PROC-02, PR-02/PR-07, TC-01).
//!
//! A provider computes one kind of derived data for one brick from a [`Snapshot`] (and from the
//! results of the derived kinds it declares as inputs). Providers are supplied by their owners
//! (meshing, collision, ...); Processing knows nothing about what they compute. Payloads are
//! opaque to Processing (`Arc<dyn Any + Send + Sync>`, PR-04 "payload typing" resolved provisionally).
//!
//! Registration is explicit and checked by [`crate::ProviderRegistry`]. This is not a mod system:
//! there is no discovery, no dynamic loading and no scripting.

use std::any::Any;
use std::sync::Arc;

use cube_core::{CellKey, ChangeType};

use crate::cancel::{CancelReason, CancellationToken};
use crate::snapshot::Snapshot;

/// Identifier of a kind of derived data, chosen by the module that owns the provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DerivedKind(pub u32);

/// The brick-level unit of derived data: one kind for one brick.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DerivedTarget {
    /// Kind of derived data.
    pub kind: DerivedKind,
    /// The brick the data belongs to (the core of its snapshot).
    pub key: CellKey,
}

/// Opaque result of a provider.
pub type Payload = Arc<dyn Any + Send + Sync>;

/// A set of [`ChangeType`]s a provider's cell inputs react to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChangeMask(u8);

impl ChangeMask {
    /// Reacts to nothing (the provider only consumes derived data of other providers).
    pub const NONE: Self = Self(0);
    /// Occupancy or shape changes.
    pub const GEOMETRY: Self = Self(1);
    /// Material channel changes.
    pub const MATERIAL: Self = Self(2);
    /// Attribute channel changes.
    pub const ATTRIBUTE: Self = Self(4);
    /// Every change.
    pub const ALL: Self = Self(7);

    /// Union of two masks.
    pub const fn or(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
    /// `true` if the mask reacts to `change`.
    pub const fn contains(self, change: ChangeType) -> bool {
        let bit = match change {
            ChangeType::Geometry => 1,
            ChangeType::Material => 2,
            ChangeType::Attribute => 4,
        };
        self.0 & bit != 0
    }
    /// `true` for the empty mask.
    pub const fn is_none(self) -> bool {
        self.0 == 0
    }
}

/// Why a provider did not produce a payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderFailure {
    /// The provider noticed the token was cancelled and stopped.
    Cancelled(CancelReason),
    /// The provider could not compute the result (for example it needs a brick that is not
    /// loaded). Never an empty success.
    Failed(String),
}

impl From<CancelReason> for ProviderFailure {
    fn from(r: CancelReason) -> Self {
        Self::Cancelled(r)
    }
}

/// Everything a provider may use while computing.
pub struct ComputeContext<'a> {
    /// What is being computed.
    pub target: DerivedTarget,
    /// Immutable input.
    pub snapshot: &'a Snapshot,
    /// Check it at least once per brick of work (see [`crate::cancel`]).
    pub token: &'a CancellationToken,
    /// Current payloads of the derived kinds the provider declared as inputs, for the same brick.
    pub inputs: &'a [(DerivedKind, Payload)],
}

impl ComputeContext<'_> {
    /// The input payload of `kind`, if the provider declared it.
    pub fn input(&self, kind: DerivedKind) -> Option<&Payload> {
        self.inputs.iter().find(|(k, _)| *k == kind).map(|(_, p)| p)
    }
}

/// Largest apron (in bricks) a provider may request. Provisional (TC-01: tunable constant).
pub const MAX_APRON: u32 = 8;

/// A computation of one kind of derived data.
pub trait Provider: Send + Sync {
    /// The kind this provider produces; unique in a registry.
    fn kind(&self) -> DerivedKind;
    /// Human-readable name for diagnostics.
    fn name(&self) -> &str;
    /// Version of the algorithm; a different version is a different input stamp.
    fn version(&self) -> u32;
    /// Hash of the provider's parameters; a different value is a different input stamp.
    fn parameters_hash(&self) -> u64 {
        0
    }
    /// Which changes of cell data invalidate the result.
    fn consumes(&self) -> ChangeMask;
    /// Derived kinds whose results (same brick) this provider needs.
    fn depends_on(&self) -> &[DerivedKind] {
        &[]
    }
    /// Width in bricks of the neighbourhood the provider reads (at most [`MAX_APRON`]).
    fn apron(&self) -> u32 {
        0
    }
    /// Computes the result. Must be deterministic for equal snapshots and inputs, must not block
    /// on other Processing tasks, and must check `ctx.token` regularly.
    fn compute(&self, ctx: &ComputeContext<'_>) -> Result<Payload, ProviderFailure>;
}
