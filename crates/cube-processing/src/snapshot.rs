//! Immutable snapshot of a brick neighbourhood with an apron (PROC-01, PR-01, TC-01 pin-only).
//!
//! A snapshot pins the *payload handles* (`Arc`) of the bricks it covers; it never copies voxel
//! data and never holds the whole asset state alive. It is taken from one consistent
//! `AssetState`, so every brick in it belongs to the same committed asset revision, and later
//! commits cannot change it: Core states are immutable, and a snapshot has no mutating API and
//! no way to reach a commit handle (enforced by `cargo xtask archcheck`).
//!
//! Absent data is explicit. Each brick keeps the five Core read states (`Present`, `KnownEmpty`,
//! `NotLoaded`, `OutsideBounds`, `LoadFailure`); the snapshot never turns "not loaded" or "failed"
//! into "empty". Reading a brick outside the captured area is an error, not an empty answer.

use std::collections::BTreeMap;

use cube_core::{
    AssetId, AssetState, BrickCoord, CellKey, CellState, Epoch, LayerData, LayerId, Region,
    Revision, RevisionToken, SharedAsset,
};

use crate::error::{LimitKind, ProcessingError};
use crate::provider::DerivedKind;

/// Absolute safety limit on the bricks of one snapshot or stamp, independent of the configurable
/// pin budget. It only protects against a request that would never fit in memory; it is not a
/// world limit (the world itself is unbounded).
pub const HARD_MAX_SNAPSHOT_BRICKS: u128 = 1 << 22;

/// What to capture: a box of bricks (the core) of one layer and level, plus an apron of
/// neighbouring bricks on every side.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SnapshotRequest {
    layer: LayerId,
    level: i8,
    core_lo: BrickCoord,
    core_hi: BrickCoord,
    apron: u32,
}

impl SnapshotRequest {
    /// Core is the inclusive brick range `lo..=hi`.
    pub fn for_bricks(
        layer: LayerId,
        level: i8,
        lo: BrickCoord,
        hi: BrickCoord,
        apron: u32,
    ) -> Result<Self, ProcessingError> {
        cube_core::coord::check_level(level)?;
        if lo.x > hi.x || lo.y > hi.y || lo.z > hi.z {
            return Err(ProcessingError::InvalidRequest(
                cube_core::CoreError::InvalidRegion,
            ));
        }
        Ok(Self {
            layer,
            level,
            core_lo: lo,
            core_hi: hi,
            apron,
        })
    }

    /// Core is the single brick `key`.
    pub fn for_brick(key: CellKey, apron: u32) -> Self {
        Self {
            layer: key.layer(),
            level: key.level(),
            core_lo: key.brick(),
            core_hi: key.brick(),
            apron,
        }
    }

    /// Core is every brick touched by `region` (which must be non-empty).
    pub fn for_region(
        layer: LayerId,
        region: &Region,
        apron: u32,
    ) -> Result<Self, ProcessingError> {
        let (lo, hi) = region.brick_span().ok_or(ProcessingError::InvalidRequest(
            cube_core::CoreError::InvalidRegion,
        ))?;
        Self::for_bricks(layer, region.level(), lo, hi, apron)
    }

    /// Layer.
    pub const fn layer(&self) -> LayerId {
        self.layer
    }
    /// Level.
    pub const fn level(&self) -> i8 {
        self.level
    }
    /// Apron width in bricks.
    pub const fn apron(&self) -> u32 {
        self.apron
    }
    /// Inclusive core range.
    pub const fn core_range(&self) -> (BrickCoord, BrickCoord) {
        (self.core_lo, self.core_hi)
    }

    /// Inclusive range of the whole captured area. Near the ends of the `i64` brick grid the
    /// apron is clipped to the grid: bricks that do not exist cannot be neighbours.
    pub fn captured_range(&self) -> (BrickCoord, BrickCoord) {
        let a = i64::from(self.apron);
        (
            BrickCoord::new(
                self.core_lo.x.saturating_sub(a),
                self.core_lo.y.saturating_sub(a),
                self.core_lo.z.saturating_sub(a),
            ),
            BrickCoord::new(
                self.core_hi.x.saturating_add(a),
                self.core_hi.y.saturating_add(a),
                self.core_hi.z.saturating_add(a),
            ),
        )
    }

    /// Number of bricks in the captured area (core plus apron). Exact; computed without capturing.
    pub fn brick_count(&self) -> u128 {
        let (lo, hi) = self.captured_range();
        let n = |a: i64, b: i64| (i128::from(b) - i128::from(a) + 1) as u128;
        // Each factor is at most 2^64, so the product of three fits u128 only below 2^128:
        // saturate instead of wrapping for astronomically large requests.
        n(lo.x, hi.x)
            .saturating_mul(n(lo.y, hi.y))
            .saturating_mul(n(lo.z, hi.z))
    }

    /// Checks the hard safety limit and, if given, a configured limit on the number of bricks.
    pub fn check_size(&self, max_bricks: Option<u64>) -> Result<(), ProcessingError> {
        let n = self.brick_count();
        let limit = max_bricks.map_or(HARD_MAX_SNAPSHOT_BRICKS, |m| {
            u128::from(m).min(HARD_MAX_SNAPSHOT_BRICKS)
        });
        if n > limit {
            Err(ProcessingError::ResourceUnavailable(
                LimitKind::PinnedBricks,
            ))
        } else {
            Ok(())
        }
    }
}

/// Exact input stamp of a derived result (PR-02, TC-01: per-brick stamps).
///
/// Two stamps have the *same inputs* when they describe the same asset history (epoch), the same
/// request, the same provider (version and parameters) and every brick has the same committed
/// revision. The asset revision at capture time (`order`) is carried only to order stamps of one
/// history; it is not part of equality of inputs, so a commit to an unrelated brick does not
/// invalidate a result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyRevision {
    asset: AssetId,
    epoch: Epoch,
    order: Revision,
    request: SnapshotRequest,
    inputs: Vec<(BrickCoord, Option<Revision>)>,
    provider_version: u32,
    parameters_hash: u64,
    derived: Vec<(DerivedKind, u64)>,
}

impl DependencyRevision {
    /// The same stamp bound to a provider version and parameters hash.
    pub fn with_provider(mut self, version: u32, parameters_hash: u64) -> Self {
        self.provider_version = version;
        self.parameters_hash = parameters_hash;
        self
    }

    #[cfg(all(test, not(loom)))]
    pub(crate) fn for_tests(
        asset: AssetId,
        epoch: Epoch,
        order: Revision,
        request: SnapshotRequest,
        inputs: Vec<(BrickCoord, Option<Revision>)>,
    ) -> Self {
        Self {
            asset,
            epoch,
            order,
            request,
            inputs,
            provider_version: 0,
            parameters_hash: 0,
            derived: Vec::new(),
        }
    }

    /// The same stamp that also depends on derived inputs: `(kind, publication sequence)` of each
    /// derived input; sequence 0 means "that input is not current". Result and live stamps of a
    /// derived-from-derived item are built the same way, so a republished input invalidates it.
    pub fn with_derived(mut self, derived: Vec<(DerivedKind, u64)>) -> Self {
        self.derived = derived;
        self
    }

    /// Derived inputs and their publication sequences.
    pub fn derived(&self) -> &[(DerivedKind, u64)] {
        &self.derived
    }

    /// Asset.
    pub const fn asset(&self) -> AssetId {
        self.asset
    }
    /// Epoch of the asset history.
    pub const fn epoch(&self) -> Epoch {
        self.epoch
    }
    /// Asset revision at capture time (ordering only).
    pub const fn order(&self) -> Revision {
        self.order
    }
    /// The captured request.
    pub const fn request(&self) -> &SnapshotRequest {
        &self.request
    }
    /// Revision of every captured brick, in ascending brick order; `None` = never committed.
    pub fn inputs(&self) -> &[(BrickCoord, Option<Revision>)] {
        &self.inputs
    }
    /// Provider version.
    pub const fn provider_version(&self) -> u32 {
        self.provider_version
    }
    /// Provider parameters hash.
    pub const fn parameters_hash(&self) -> u64 {
        self.parameters_hash
    }

    /// `(epoch, asset revision, sum of derived input sequences)`: the order of stamps of one
    /// item. Epochs, revisions and publication sequences only grow, so the key only grows; an
    /// input that stops being current (sequence 0) only happens together with a newer revision.
    pub fn order_key(&self) -> (u32, Revision, u64) {
        let d = self
            .derived
            .iter()
            .fold(0u64, |a, (_, s)| a.saturating_add(*s));
        (self.epoch.raw(), self.order, d)
    }

    /// `true` if both describe exactly the same inputs (see the type documentation).
    pub fn same_inputs(&self, other: &Self) -> bool {
        self.asset == other.asset
            && self.epoch == other.epoch
            && self.request == other.request
            && self.provider_version == other.provider_version
            && self.parameters_hash == other.parameters_hash
            && self.derived == other.derived
            && self.inputs == other.inputs
    }
}

fn each_brick(lo: BrickCoord, hi: BrickCoord, mut f: impl FnMut(BrickCoord)) {
    for x in lo.x..=hi.x {
        for y in lo.y..=hi.y {
            for z in lo.z..=hi.z {
                f(BrickCoord::new(x, y, z));
            }
        }
    }
}

fn check_layer(state: &AssetState, layer: LayerId) -> Result<(), ProcessingError> {
    match state.asset().layer(layer) {
        None => Err(ProcessingError::UnknownLayer(layer)),
        Some(l) => match l.data() {
            LayerData::Voxel(_) | LayerData::Sdf(_) => Ok(()),
            LayerData::Mesh(_) => Err(ProcessingError::UnsupportedLayer(layer)),
        },
    }
}

/// The input stamp of `req` in `state`, without pinning any payload. Used to learn what the
/// *current* state looks like (invalidation, publication check).
pub fn stamp(
    state: &AssetState,
    req: &SnapshotRequest,
    max_bricks: Option<u64>,
) -> Result<DependencyRevision, ProcessingError> {
    req.check_size(max_bricks)?;
    check_layer(state, req.layer)?;
    let (lo, hi) = req.captured_range();
    let mut inputs = Vec::new();
    each_brick(lo, hi, |b| {
        let rev = state.cell_token(req.layer, req.level, b).map(|t| t.value());
        inputs.push((b, rev));
    });
    let token: RevisionToken = state.asset_token();
    Ok(DependencyRevision {
        asset: state.asset().id(),
        epoch: token.epoch(),
        order: token.value(),
        request: *req,
        inputs,
        provider_version: 0,
        parameters_hash: 0,
        derived: Vec::new(),
    })
}

/// Immutable view of a brick neighbourhood at one asset revision.
#[derive(Debug)]
pub struct Snapshot {
    request: SnapshotRequest,
    cells: BTreeMap<BrickCoord, CellState>,
    dependency: DependencyRevision,
}

static OUTSIDE_GRID: CellState = CellState::OutsideBounds;

impl Snapshot {
    /// Captures `req` from the latest committed state of `asset`. `max_bricks` is the configured
    /// limit on the area (checked before anything is pinned).
    pub fn capture(
        asset: &SharedAsset,
        req: &SnapshotRequest,
        max_bricks: Option<u64>,
    ) -> Result<Self, ProcessingError> {
        Self::capture_state(&asset.snapshot(), req, max_bricks)
    }

    /// Captures `req` from an explicit committed state.
    pub fn capture_state(
        state: &AssetState,
        req: &SnapshotRequest,
        max_bricks: Option<u64>,
    ) -> Result<Self, ProcessingError> {
        let dependency = stamp(state, req, max_bricks)?;
        let (lo, hi) = req.captured_range();
        let mut cells = BTreeMap::new();
        let mut failure = None;
        each_brick(lo, hi, |b| match state.cell(req.layer, req.level, b) {
            Some(Ok(c)) => {
                cells.insert(b, c);
            }
            Some(Err(e)) => failure = Some(ProcessingError::from(e)),
            None => failure = Some(ProcessingError::UnsupportedLayer(req.layer)),
        });
        if let Some(e) = failure {
            return Err(e);
        }
        Ok(Self {
            request: *req,
            cells,
            dependency,
        })
    }

    /// The request this snapshot satisfies.
    pub const fn request(&self) -> &SnapshotRequest {
        &self.request
    }
    /// The input stamp (revisions of every captured brick).
    pub const fn dependency(&self) -> &DependencyRevision {
        &self.dependency
    }
    /// Number of pinned bricks (core plus apron).
    pub fn brick_count(&self) -> usize {
        self.cells.len()
    }
    /// `true` if `b` is in the core (not only in the apron).
    pub fn is_core(&self, b: BrickCoord) -> bool {
        let (lo, hi) = self.request.core_range();
        b.x >= lo.x && b.x <= hi.x && b.y >= lo.y && b.y <= hi.y && b.z >= lo.z && b.z <= hi.z
    }

    /// State of a brick in the captured area. A brick outside it is `OutsideSnapshot`: the
    /// snapshot does not know anything about it, which is different from "empty" or "not loaded".
    pub fn cell(&self, b: BrickCoord) -> Result<&CellState, ProcessingError> {
        self.cells.get(&b).ok_or_else(|| {
            match CellKey::new(self.request.layer, self.request.level, b) {
                Ok(k) => ProcessingError::OutsideSnapshot(k),
                Err(e) => ProcessingError::from(e),
            }
        })
    }

    /// State of the brick at offset `d` from `b`. If the neighbour does not exist because the
    /// coordinate would leave the `i64` grid it is `OutsideBounds`; if it exists but is outside
    /// the captured area (offset larger than the apron) the call fails.
    pub fn neighbour(&self, b: BrickCoord, d: [i64; 3]) -> Result<&CellState, ProcessingError> {
        match (
            b.x.checked_add(d[0]),
            b.y.checked_add(d[1]),
            b.z.checked_add(d[2]),
        ) {
            (Some(x), Some(y), Some(z)) => self.cell(BrickCoord::new(x, y, z)),
            _ => Ok(&OUTSIDE_GRID),
        }
    }

    /// Every captured brick in ascending order, core and apron.
    pub fn cells(&self) -> impl Iterator<Item = (BrickCoord, &CellState)> {
        self.cells.iter().map(|(b, c)| (*b, c))
    }

    /// Only the core bricks in ascending order.
    pub fn core_cells(&self) -> impl Iterator<Item = (BrickCoord, &CellState)> + '_ {
        self.cells().filter(|(b, _)| self.is_core(*b))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lid(n: u64) -> LayerId {
        LayerId::new(n)
    }
    fn bc(x: i64, y: i64, z: i64) -> BrickCoord {
        BrickCoord::new(x, y, z)
    }

    #[test]
    fn request_validates_levels_and_ranges() {
        assert!(SnapshotRequest::for_bricks(lid(1), 0, bc(0, 0, 0), bc(1, 1, 1), 1).is_ok());
        assert!(matches!(
            SnapshotRequest::for_bricks(lid(1), -1, bc(0, 0, 0), bc(1, 1, 1), 1),
            Err(ProcessingError::InvalidRequest(_))
        ));
        assert!(matches!(
            SnapshotRequest::for_bricks(lid(1), 0, bc(2, 0, 0), bc(1, 1, 1), 0),
            Err(ProcessingError::InvalidRequest(_))
        ));
    }

    #[test]
    fn captured_range_and_count_include_the_apron_and_negative_bricks() {
        let r = SnapshotRequest::for_bricks(lid(1), 0, bc(-1, 0, 0), bc(0, 0, 0), 1).unwrap();
        assert_eq!(r.captured_range(), (bc(-2, -1, -1), bc(1, 1, 1)));
        assert_eq!(r.brick_count(), 4 * 3 * 3);
        let single = SnapshotRequest::for_brick(CellKey::new(lid(1), 2, bc(5, 5, 5)).unwrap(), 0);
        assert_eq!(single.brick_count(), 1);
    }

    #[test]
    fn apron_is_clipped_at_the_ends_of_the_grid_without_overflow() {
        let r = SnapshotRequest::for_bricks(lid(1), 0, bc(i64::MAX, 0, 0), bc(i64::MAX, 0, 0), 2)
            .unwrap();
        let (lo, hi) = r.captured_range();
        assert_eq!((lo.x, hi.x), (i64::MAX - 2, i64::MAX));
        assert_eq!(r.brick_count(), 3 * 5 * 5);
        let r = SnapshotRequest::for_bricks(lid(1), 0, bc(i64::MIN, 0, 0), bc(i64::MIN, 0, 0), 1)
            .unwrap();
        assert_eq!(r.captured_range().0.x, i64::MIN);
    }

    #[test]
    fn oversized_requests_are_rejected_before_anything_is_pinned() {
        let huge =
            SnapshotRequest::for_bricks(lid(1), 0, bc(i64::MIN, 0, 0), bc(i64::MAX, 0, 0), 0)
                .unwrap();
        assert_eq!(
            huge.check_size(None),
            Err(ProcessingError::ResourceUnavailable(
                LimitKind::PinnedBricks
            ))
        );
        let r = SnapshotRequest::for_bricks(lid(1), 0, bc(0, 0, 0), bc(3, 3, 3), 0).unwrap();
        assert!(r.check_size(Some(64)).is_ok());
        assert!(r.check_size(Some(63)).is_err());
        let all = SnapshotRequest::for_bricks(
            lid(1),
            0,
            bc(i64::MIN, i64::MIN, i64::MIN),
            bc(i64::MAX, i64::MAX, i64::MAX),
            0,
        )
        .unwrap();
        assert_eq!(
            all.brick_count(),
            u128::MAX,
            "saturates instead of wrapping"
        );
        assert!(all.check_size(None).is_err());
    }

    #[test]
    fn dependency_equality_ignores_order_but_not_inputs_or_provider() {
        let req = SnapshotRequest::for_brick(CellKey::new(lid(1), 0, bc(0, 0, 0)).unwrap(), 0);
        let mk = |order: u64, rev: Option<u64>| DependencyRevision {
            asset: AssetId::new(1),
            epoch: Epoch::INITIAL,
            order: Revision::new(order),
            request: req,
            inputs: vec![(bc(0, 0, 0), rev.map(Revision::new))],
            provider_version: 1,
            parameters_hash: 0,
            derived: Vec::new(),
        };
        assert!(mk(1, Some(1)).same_inputs(&mk(9, Some(1))));
        assert!(!mk(1, Some(1)).same_inputs(&mk(1, Some(2))));
        assert!(!mk(1, None).same_inputs(&mk(1, Some(1))));
        assert!(!mk(1, Some(1)).same_inputs(&mk(1, Some(1)).with_provider(2, 0)));
        assert!(!mk(1, Some(1)).same_inputs(&mk(1, Some(1)).with_provider(1, 7)));
        assert!(mk(1, Some(1)).order_key() < mk(2, Some(1)).order_key());
    }
}
