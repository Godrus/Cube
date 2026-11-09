//! Expected-token validation, conflict results and idempotent retry by OperationId (ORCH-03).

use std::collections::{HashMap, HashSet, VecDeque};

use cube_core::{
    AssetState, BrickCoord, CellKey, Conflict, ExpectedRevision, LayerId, OperationId, ResourceId,
    RevisionToken, WriteSet,
};
use cube_editing_api::{
    ConflictResult, ConflictScope, EditingError, EditingOperation, EditingResult, Suggestion,
};

/// The revision a write to this cell must expect: whatever the base state has.
pub(crate) fn expected_of(
    base: &AssetState,
    layer: LayerId,
    level: i8,
    brick: BrickCoord,
) -> ExpectedRevision {
    base.cell_token(layer, level, brick)
        .map_or(ExpectedRevision::Absent, |t| {
            ExpectedRevision::At(t.value())
        })
}

/// Checks the write sets a backend returned against the operation that asked for them: layers
/// must be targets, levels must be the region's level, every cell must lie inside the declared
/// write region (an upper bound of the change, E-03), and a cell may appear only once.
pub fn check_write_sets(op: &EditingOperation, sets: &[WriteSet]) -> Result<(), EditingError> {
    let region = &op.write_region;
    let Some((lo, hi)) = region.brick_span() else {
        return Err(EditingError::InvalidOperation("empty write region"));
    };
    let mut seen = std::collections::BTreeSet::new();
    for set in sets {
        if !op.layers.contains(&set.layer) {
            return Err(EditingError::InvalidOperation(
                "write set for a layer that is not a target",
            ));
        }
        for w in &set.writes {
            if w.level != region.level() {
                return Err(EditingError::InvalidResolution);
            }
            let b = w.brick;
            let inside = b.x >= lo.x
                && b.x <= hi.x
                && b.y >= lo.y
                && b.y <= hi.y
                && b.z >= lo.z
                && b.z <= hi.z;
            if !inside {
                return Err(EditingError::InvalidOperation(
                    "write outside the declared write region",
                ));
            }
            if !seen.insert((set.layer, w.level, w.brick)) {
                return Err(EditingError::InvalidOperation("cell written twice"));
            }
        }
    }
    Ok(())
}

/// Checks the revisions a caller expects against `current`: asset, layer and cell tokens are
/// supported; a token that is not equal to the live one (older, newer, incomparable, or for a
/// resource that does not exist) is a revision conflict.
pub fn check_expected_tokens(
    current: &AssetState,
    expected: &[RevisionToken],
) -> Result<(), EditingError> {
    for t in expected {
        let live = match t.resource() {
            ResourceId::Asset(a) if a == current.asset().id() => Some(current.asset_token()),
            ResourceId::Layer(l) => current.layer_token(l),
            ResourceId::Cell(k) => current.cell_token(k.layer(), k.level(), k.brick()),
            _ => return Err(EditingError::InvalidOperation("unsupported expected token")),
        };
        match live {
            Some(l) if t.compare(&l) == cube_core::Comparison::Equal => {}
            _ => return Err(EditingError::RevisionConflict),
        }
    }
    Ok(())
}

/// Builds the conflict report for the cells Core rejected: exactly those cells, in Core's order,
/// with the expected and current tokens of the cells that existed on both sides.
pub fn conflict_result(current: &AssetState, conflicts: &[Conflict]) -> ConflictResult {
    let epoch = current.asset_token().epoch();
    let mut conflicting = Vec::new();
    let mut expected = Vec::new();
    let mut now = Vec::new();
    for c in conflicts {
        let Ok(key) = CellKey::new(c.layer, c.level, c.brick) else {
            continue;
        };
        let res = ResourceId::Cell(key);
        conflicting.push(res);
        if let (ExpectedRevision::At(e), Some(a)) = (c.expected, c.actual) {
            expected.push(RevisionToken::new(res, epoch, e));
            now.push(RevisionToken::new(res, epoch, a));
        }
    }
    ConflictResult {
        scope: ConflictScope::Brick,
        conflicting,
        expected,
        current: now,
        holder: None,
        suggestion: Suggestion::Retry,
    }
}

/// What the retry table keeps of a committed operation: a fingerprint of its content (not the
/// content itself, so the table can also be rebuilt from the durable journal) and its result.
#[derive(Debug, Clone)]
pub(crate) struct Recorded {
    pub fingerprint: u64,
    pub result: EditingResult,
}

/// Results of committed operations by `OperationId`, bounded, so that a retried request never
/// applies twice.
///
/// Eviction is safe, not just bounded: when an entry is dropped, the highest dropped sequence of
/// its scope is remembered (the *watermark*). An id at or below its scope's watermark that is
/// not in the table is refused as [`Replay::Expired`], because the table can no longer tell
/// whether it was applied. This relies on the contract that operation sequences increase per
/// scope; a client that reuses low sequences after the window moved on is refused, never applied
/// twice. Ids whose outcome is undecided after a recovery are *blocked* until resolved.
#[derive(Debug)]
pub(crate) struct RetryTable {
    capacity: usize,
    order: VecDeque<OperationId>,
    map: HashMap<OperationId, Recorded>,
    watermark: HashMap<u64, u64>,
    blocked: HashSet<OperationId>,
}

/// Outcome of looking an operation up.
#[derive(Debug, Clone)]
pub(crate) enum Replay {
    /// Never committed and not covered by the watermark: execute normally.
    Fresh,
    /// Committed earlier with identical content: answer with the recorded result.
    Same(EditingResult),
    /// The id was used for different content.
    Mismatch,
    /// The id is older than the retry window and unknown: refused (it may have been applied).
    Expired,
    /// The outcome of an earlier attempt is undecided after a recovery: refused until resolved.
    Blocked,
}

impl RetryTable {
    pub(crate) fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            order: VecDeque::new(),
            map: HashMap::new(),
            watermark: HashMap::new(),
            blocked: HashSet::new(),
        }
    }

    pub(crate) fn lookup(&self, op: &EditingOperation) -> Replay {
        if self.blocked.contains(&op.id) {
            return Replay::Blocked;
        }
        match self.map.get(&op.id) {
            Some(r) if r.fingerprint == op.fingerprint() => Replay::Same(r.result.clone()),
            Some(_) => Replay::Mismatch,
            None => match self.watermark.get(&op.id.scope()) {
                Some(w) if op.id.sequence() <= *w => Replay::Expired,
                _ => Replay::Fresh,
            },
        }
    }

    pub(crate) fn record(&mut self, op: EditingOperation, result: EditingResult) {
        self.record_fingerprint(op.id, op.fingerprint(), result);
    }

    pub(crate) fn record_fingerprint(
        &mut self,
        id: OperationId,
        fingerprint: u64,
        result: EditingResult,
    ) {
        let new = self
            .map
            .insert(
                id,
                Recorded {
                    fingerprint,
                    result,
                },
            )
            .is_none();
        if new {
            self.order.push_back(id);
            while self.order.len() > self.capacity {
                if let Some(old) = self.order.pop_front() {
                    self.map.remove(&old);
                    let w = self.watermark.entry(old.scope()).or_insert(0);
                    *w = (*w).max(old.sequence());
                }
            }
        }
    }

    pub(crate) fn block(&mut self, id: OperationId) {
        self.blocked.insert(id);
    }

    pub(crate) fn unblock(&mut self, id: OperationId) {
        self.blocked.remove(&id);
    }

    pub(crate) fn len(&self) -> usize {
        self.map.len()
    }

    pub(crate) fn result_of(&self, id: OperationId) -> Option<&EditingResult> {
        self.map.get(&id).map(|r| &r.result)
    }

    pub(crate) fn update_result(&mut self, id: OperationId, result: EditingResult) {
        if let Some(r) = self.map.get_mut(&id) {
            r.result = result;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cube_core::{AssetId, CellChange, CellWrite, Region, VoxelPos};
    use cube_editing_api::{EditMode, Parameters, kinds};

    fn op(id: u64) -> EditingOperation {
        EditingOperation {
            id: OperationId::new(1, id),
            kind: kinds::SET_MATERIAL,
            asset: AssetId::new(1),
            layers: vec![LayerId::new(1)],
            parameters: Parameters::default(),
            read_region: None,
            write_region: Region::new(0, VoxelPos::new(-16, 0, 0), VoxelPos::new(16, 16, 16))
                .unwrap(),
            conversion: None,
            mode: EditMode::Destructive,
        }
    }

    fn w(x: i64) -> CellWrite {
        CellWrite {
            level: 0,
            brick: BrickCoord::new(x, 0, 0),
            expected: ExpectedRevision::Absent,
            change: CellChange::KnownEmpty,
        }
    }

    fn set(layer: u64, writes: Vec<CellWrite>) -> WriteSet {
        WriteSet {
            layer: LayerId::new(layer),
            writes,
        }
    }

    #[test]
    fn write_sets_must_stay_inside_targets_levels_and_region() {
        let o = op(1);
        assert_eq!(check_write_sets(&o, &[set(1, vec![w(-1), w(0)])]), Ok(()));
        for bad in [
            set(2, vec![w(0)]),
            set(1, vec![w(1)]),
            set(1, vec![w(-2)]),
            set(1, vec![w(0), w(0)]),
        ] {
            assert!(matches!(
                check_write_sets(&o, &[bad]),
                Err(EditingError::InvalidOperation(_))
            ));
        }
        let mut other_level = w(0);
        other_level.level = 1;
        assert_eq!(
            check_write_sets(&o, &[set(1, vec![other_level])]),
            Err(EditingError::InvalidResolution)
        );
        // The same cell in two sets of one result is also a duplicate.
        assert!(check_write_sets(&o, &[set(1, vec![w(0)]), set(1, vec![w(0)])]).is_err());
    }

    #[test]
    fn retry_table_distinguishes_fresh_same_and_reused_ids() {
        let mut t = RetryTable::new(2);
        let r = EditingResult::Cancelled;
        assert!(matches!(t.lookup(&op(1)), Replay::Fresh));
        t.record(op(1), r.clone());
        assert!(matches!(t.lookup(&op(1)), Replay::Same(_)));
        let mut changed = op(1);
        changed.mode = EditMode::NonDestructive;
        assert!(matches!(t.lookup(&changed), Replay::Mismatch));
    }

    #[test]
    fn retry_table_is_bounded_and_drops_the_oldest() {
        let mut t = RetryTable::new(2);
        for i in 1..=3 {
            t.record(op(i), EditingResult::Cancelled);
        }
        assert!(
            matches!(t.lookup(&op(1)), Replay::Expired),
            "a dropped id must not look fresh: it may have been applied"
        );
        assert!(matches!(t.lookup(&op(2)), Replay::Same(_)));
        assert!(matches!(t.lookup(&op(3)), Replay::Same(_)));
        assert!(t.result_of(OperationId::new(1, 3)).is_some());
    }
}
