//! Sparse brick storage of one layer (contract C-08, core slice).
//!
//! Reads are public. Writes are crate-private: authoritative state changes only through the
//! commit boundary, which arrives with the next tasks.

use std::collections::BTreeMap;
use std::sync::Arc;

use super::state::{CellPayload, CellState, LoadFailureReason};
use crate::coord::{BrickCoord, check_level};
use crate::error::CoreError;
use crate::region::Region;
use crate::revision::Revision;

/// Inclusive-exclusive box of base-level (level 0) brick coordinates a layer may contain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BrickBounds {
    min: BrickCoord,
    max: BrickCoord,
}

impl BrickBounds {
    /// Creates bounds `[min, max)`; fails if `min > max` on some axis.
    pub fn new(min: BrickCoord, max: BrickCoord) -> Result<Self, CoreError> {
        if min.x > max.x || min.y > max.y || min.z > max.z {
            return Err(CoreError::InvalidRegion);
        }
        Ok(Self { min, max })
    }

    /// Inclusive lower corner (base-level bricks).
    pub fn min(&self) -> BrickCoord {
        self.min
    }

    /// Exclusive upper corner (base-level bricks).
    pub fn max(&self) -> BrickCoord {
        self.max
    }

    fn contains(&self, b: BrickCoord) -> bool {
        b.x >= self.min.x
            && b.x < self.max.x
            && b.y >= self.min.y
            && b.y < self.max.y
            && b.z >= self.min.z
            && b.z < self.max.z
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Stored {
    Present(Arc<CellPayload>),
    KnownEmpty,
    Failed(LoadFailureReason),
}

/// Sparse map from `(level, brick)` to cell content, with optional layer bounds.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CellStore {
    bounds: Option<BrickBounds>,
    cells: BTreeMap<(i8, BrickCoord), Stored>,
    revisions: BTreeMap<(i8, BrickCoord), Revision>,
}

impl CellStore {
    /// An unbounded, empty store (every cell is `NotLoaded`).
    pub fn new() -> Self {
        Self::default()
    }

    /// An empty store limited to `bounds` (cells outside are `OutsideBounds`).
    pub fn with_bounds(bounds: BrickBounds) -> Self {
        Self {
            bounds: Some(bounds),
            cells: BTreeMap::new(),
            revisions: BTreeMap::new(),
        }
    }

    /// State of the brick at `(level, brick)`. Fails only for a reserved level.
    ///
    /// A brick of level `L` lies inside the base-level brick `brick >> L`, which is what the bounds
    /// are checked against. The bounds take precedence over stored data.
    pub fn get(&self, level: i8, brick: BrickCoord) -> Result<CellState, CoreError> {
        check_level(level)?;
        if let Some(b) = &self.bounds {
            let shift = u32::from(level.unsigned_abs()).min(63);
            let base = BrickCoord::new(brick.x >> shift, brick.y >> shift, brick.z >> shift);
            if !b.contains(base) {
                return Ok(CellState::OutsideBounds);
            }
        }
        Ok(match self.cells.get(&(level, brick)) {
            None => CellState::NotLoaded,
            Some(Stored::KnownEmpty) => CellState::KnownEmpty,
            Some(Stored::Present(p)) => CellState::Present(Arc::clone(p)),
            Some(Stored::Failed(r)) => CellState::LoadFailure(*r),
        })
    }

    /// Revision of the brick, `None` if it was never committed. A revision is never reused or lost.
    pub fn revision(&self, level: i8, brick: BrickCoord) -> Option<Revision> {
        self.revisions.get(&(level, brick)).copied()
    }

    /// Whether `(level, brick)` lies inside the layer bounds (always true when unbounded).
    pub fn in_bounds(&self, level: i8, brick: BrickCoord) -> bool {
        match &self.bounds {
            None => true,
            Some(b) => {
                let shift = u32::from(level.unsigned_abs()).min(63);
                b.contains(BrickCoord::new(
                    brick.x >> shift,
                    brick.y >> shift,
                    brick.z >> shift,
                ))
            }
        }
    }

    /// The layer bounds, if bounded.
    pub fn bounds(&self) -> Option<BrickBounds> {
        self.bounds
    }

    /// Number of stored cells (any stored state).
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    /// `true` if nothing is stored.
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    /// Stored cells in ascending `(level, x, y, z)` order, as `(level, brick, state)`.
    pub fn iter(&self) -> impl Iterator<Item = (i8, BrickCoord, CellState)> + '_ {
        self.cells.iter().map(|((l, b), s)| (*l, *b, to_state(s)))
    }

    /// Stored cells whose brick intersects `region` (same level), in ascending order.
    pub fn iter_region(
        &self,
        region: &Region,
    ) -> impl Iterator<Item = (BrickCoord, CellState)> + '_ {
        let level = region.level();
        let span = region.brick_span();
        self.cells
            .range(
                (level, BrickCoord::new(i64::MIN, i64::MIN, i64::MIN))
                    ..=(level, BrickCoord::new(i64::MAX, i64::MAX, i64::MAX)),
            )
            .filter(move |((_, b), _)| {
                span.is_some_and(|(lo, hi)| {
                    b.x >= lo.x
                        && b.x <= hi.x
                        && b.y >= lo.y
                        && b.y <= hi.y
                        && b.z >= lo.z
                        && b.z <= hi.z
                })
            })
            .map(|((_, b), s)| (*b, to_state(s)))
    }

    /// Applies one committed write with its new revision (`None` payload = known empty).
    pub(crate) fn commit_cell(
        &mut self,
        level: i8,
        brick: BrickCoord,
        payload: Option<Arc<CellPayload>>,
        rev: Revision,
    ) {
        let stored = match payload {
            Some(p) => Stored::Present(p),
            None => Stored::KnownEmpty,
        };
        self.cells.insert((level, brick), stored);
        self.revisions.insert((level, brick), rev);
    }

    /// Stores a present cell (crate-private: reached through the commit boundary).
    #[allow(dead_code)] // first caller arrives with the commit task
    pub(crate) fn put(
        &mut self,
        level: i8,
        brick: BrickCoord,
        payload: CellPayload,
    ) -> Result<(), CoreError> {
        check_level(level)?;
        self.cells
            .insert((level, brick), Stored::Present(Arc::new(payload)));
        Ok(())
    }

    /// Stores "known empty" (crate-private).
    #[allow(dead_code)]
    pub(crate) fn put_known_empty(
        &mut self,
        level: i8,
        brick: BrickCoord,
    ) -> Result<(), CoreError> {
        check_level(level)?;
        self.cells.insert((level, brick), Stored::KnownEmpty);
        Ok(())
    }

    /// Records a load failure (crate-private).
    #[allow(dead_code)]
    pub(crate) fn put_failure(
        &mut self,
        level: i8,
        brick: BrickCoord,
        reason: LoadFailureReason,
    ) -> Result<(), CoreError> {
        check_level(level)?;
        self.cells.insert((level, brick), Stored::Failed(reason));
        Ok(())
    }

    /// Forgets a cell: it becomes `NotLoaded` again (crate-private; used by eviction).
    pub(crate) fn remove(&mut self, level: i8, brick: BrickCoord) {
        self.cells.remove(&(level, brick));
    }
}

fn to_state(s: &Stored) -> CellState {
    match s {
        Stored::Present(p) => CellState::Present(Arc::clone(p)),
        Stored::KnownEmpty => CellState::KnownEmpty,
        Stored::Failed(r) => CellState::LoadFailure(*r),
    }
}

#[cfg(test)]
mod tests {
    use super::super::channel::ChannelSet;
    use super::*;
    use crate::coord::VoxelPos;
    use crate::material::MaterialIndex;

    fn b(x: i64, y: i64, z: i64) -> BrickCoord {
        BrickCoord::new(x, y, z)
    }
    fn payload(m: u16) -> CellPayload {
        CellPayload::Channels(ChannelSet::uniform_material(MaterialIndex::from_raw(m)))
    }

    #[test]
    fn empty_loaded_is_not_the_same_as_not_loaded() {
        let mut s = CellStore::new();
        assert_eq!(s.get(0, b(0, 0, 0)), Ok(CellState::NotLoaded));
        s.put_known_empty(0, b(0, 0, 0)).unwrap();
        assert_eq!(s.get(0, b(0, 0, 0)), Ok(CellState::KnownEmpty));
        assert_eq!(s.get(0, b(1, 0, 0)), Ok(CellState::NotLoaded));
        s.remove(0, b(0, 0, 0));
        assert_eq!(s.get(0, b(0, 0, 0)), Ok(CellState::NotLoaded));
    }

    #[test]
    fn load_failure_is_never_reported_as_empty() {
        let mut s = CellStore::new();
        s.put_failure(0, b(2, 2, 2), LoadFailureReason::Corrupt)
            .unwrap();
        assert_eq!(
            s.get(0, b(2, 2, 2)),
            Ok(CellState::LoadFailure(LoadFailureReason::Corrupt))
        );
        assert!(!s.get(0, b(2, 2, 2)).unwrap().is_loaded());
    }

    #[test]
    fn bounds_take_precedence_and_use_floor_for_negative_bricks() {
        let bounds = BrickBounds::new(b(-2, -2, -2), b(2, 2, 2)).unwrap();
        let mut s = CellStore::with_bounds(bounds);
        assert_eq!(s.get(0, b(-2, 1, 1)), Ok(CellState::NotLoaded));
        assert_eq!(s.get(0, b(-3, 0, 0)), Ok(CellState::OutsideBounds));
        assert_eq!(s.get(0, b(2, 0, 0)), Ok(CellState::OutsideBounds));
        // Data stored outside the bounds is still reported as outside.
        s.put(0, b(5, 0, 0), payload(1)).unwrap();
        assert_eq!(s.get(0, b(5, 0, 0)), Ok(CellState::OutsideBounds));
        // Level 1 brick (-1, 0, 0) lies in base brick -1; (-5, 0, 0) in base brick -3 (outside).
        assert_eq!(s.get(1, b(-1, 0, 0)), Ok(CellState::NotLoaded));
        assert_eq!(s.get(1, b(-5, 0, 0)), Ok(CellState::OutsideBounds));
        assert_eq!(
            s.get(1, b(3, 0, 0)),
            Ok(CellState::NotLoaded),
            "3 >> 1 = 1 is inside"
        );
        assert_eq!(
            s.get(1, b(4, 0, 0)),
            Ok(CellState::OutsideBounds),
            "4 >> 1 = 2 is outside"
        );
    }

    #[test]
    fn levels_are_separate_address_spaces_and_negative_levels_are_rejected() {
        let mut s = CellStore::new();
        s.put(0, b(0, 0, 0), payload(1)).unwrap();
        s.put(2, b(0, 0, 0), payload(2)).unwrap();
        assert_ne!(s.get(0, b(0, 0, 0)), s.get(2, b(0, 0, 0)));
        assert_eq!(s.get(1, b(0, 0, 0)), Ok(CellState::NotLoaded));
        assert_eq!(s.get(-1, b(0, 0, 0)), Err(CoreError::InvalidLevel(-1)));
        assert_eq!(
            s.put(-1, b(0, 0, 0), payload(1)),
            Err(CoreError::InvalidLevel(-1))
        );
    }

    #[test]
    fn present_cells_share_payloads_cheaply() {
        let mut s = CellStore::new();
        s.put(0, b(0, 0, 0), payload(7)).unwrap();
        let (CellState::Present(a), CellState::Present(c)) =
            (s.get(0, b(0, 0, 0)).unwrap(), s.get(0, b(0, 0, 0)).unwrap())
        else {
            panic!("expected present");
        };
        assert!(Arc::ptr_eq(&a, &c));
    }

    #[test]
    fn iteration_is_sorted_and_region_filtered() {
        let mut s = CellStore::new();
        for (x, y, z) in [(3, 0, 0), (-1, 5, 5), (0, 0, 1), (0, 0, 0), (-1, 5, -5)] {
            s.put(0, b(x, y, z), payload(1)).unwrap();
        }
        s.put_known_empty(1, b(0, 0, 0)).unwrap();
        let keys: Vec<(i8, BrickCoord)> = s.iter().map(|(l, c, _)| (l, c)).collect();
        let mut sorted = keys.clone();
        sorted.sort();
        assert_eq!(keys, sorted);
        assert_eq!(s.len(), 6);
        let r = Region::new(0, VoxelPos::new(-16, 0, 0), VoxelPos::new(16, 16, 32)).unwrap();
        let in_region: Vec<BrickCoord> = s.iter_region(&r).map(|(c, _)| c).collect();
        assert_eq!(in_region, vec![b(0, 0, 0), b(0, 0, 1)]);
        let empty = Region::new(0, VoxelPos::new(0, 0, 0), VoxelPos::new(0, 5, 5)).unwrap();
        assert_eq!(s.iter_region(&empty).count(), 0);
    }

    #[test]
    fn bounds_constructor_rejects_inverted_boxes() {
        assert_eq!(
            BrickBounds::new(b(1, 0, 0), b(0, 5, 5)),
            Err(CoreError::InvalidRegion)
        );
    }
}
