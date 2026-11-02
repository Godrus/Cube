//! Cube Editing (experimental spike; draft E-03, E-04, E-05, E-09 at minimal level).
//!
//! One operation (`FillRegion`, which also covers set/clear voxel), executed as an atomic
//! transaction through the Core commit primitive, with a minimal delta-based undo/redo.
//! Not implemented: transaction handles/overlays across several operations, backends port,
//! authorization, conflict policy beyond reject, cancellation, async execution.

use std::sync::Arc;

use cube_core::{
    AssetId, Brick, BrickCoord, BrickWrite, CoreError, DirtyRegion, ExpectedBrick, GeometryAsset,
    LayerId, OperationId, Region, RevisionToken, VoxelPos, WriteSet,
};

/// Upper bound for one fill (4M voxels) so a bad request cannot stall the process.
pub const MAX_FILL_VOLUME: u64 = 1 << 22;
pub const DEFAULT_HISTORY_LIMIT: usize = 256;

#[derive(Clone, Copy, Debug)]
pub enum EditOp {
    /// Material 0 clears.
    FillRegion {
        layer: LayerId,
        region: Region,
        material: u16,
    },
}

#[derive(Clone, Debug)]
pub struct EditOutcome {
    pub operation: Option<OperationId>,
    pub asset_token: RevisionToken,
    pub layer_token: RevisionToken,
    pub dirty: Vec<DirtyRegion>,
    pub changed_voxels: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditError {
    Core(CoreError),
    InvalidOperation(&'static str),
    NothingToUndo,
    NothingToRedo,
}

impl EditError {
    pub fn code(&self) -> &'static str {
        match self {
            EditError::Core(e) => e.code(),
            EditError::InvalidOperation(_) => "EDIT-0001",
            EditError::NothingToUndo => "EDIT-0002",
            EditError::NothingToRedo => "EDIT-0003",
        }
    }
}

impl From<CoreError> for EditError {
    fn from(e: CoreError) -> Self {
        EditError::Core(e)
    }
}

impl std::fmt::Display for EditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EditError::Core(e) => write!(f, "{e}"),
            EditError::InvalidOperation(m) => write!(f, "{}: invalid operation: {m}", self.code()),
            EditError::NothingToUndo => write!(f, "{}: nothing to undo", self.code()),
            EditError::NothingToRedo => write!(f, "{}: nothing to redo", self.code()),
        }
    }
}

impl std::error::Error for EditError {}

/// Actual data change of one brick; before/after share storage with Core (copy-on-write).
#[derive(Clone, Debug)]
struct BrickChange {
    coord: BrickCoord,
    before: Option<Arc<Brick>>,
    after: Option<Arc<Brick>>,
    changed: Region,
}

/// Minimal GeometryDelta: what changed, plus the brick states the asset must be in for the
/// delta to be applicable in its current direction.
#[derive(Clone, Debug)]
struct Delta {
    operation: OperationId,
    layer: LayerId,
    changes: Vec<BrickChange>,
    expected_now: Vec<(BrickCoord, ExpectedBrick)>,
}

pub struct Editor {
    asset: GeometryAsset,
    undo: Vec<Delta>,
    redo: Vec<Delta>,
    next_op: u64,
    history_limit: usize,
}

impl Editor {
    pub fn new(asset: GeometryAsset) -> Self {
        Self {
            asset,
            undo: Vec::new(),
            redo: Vec::new(),
            next_op: 1,
            history_limit: DEFAULT_HISTORY_LIMIT,
        }
    }

    pub fn with_history_limit(mut self, limit: usize) -> Self {
        self.history_limit = limit;
        self
    }

    pub fn asset(&self) -> &GeometryAsset {
        &self.asset
    }

    pub fn asset_id(&self) -> AssetId {
        self.asset.id()
    }

    /// Replace the asset (load). History cannot apply to a different asset state and is dropped.
    pub fn replace_asset(&mut self, asset: GeometryAsset) {
        self.asset = asset;
        self.undo.clear();
        self.redo.clear();
    }

    pub fn add_voxel_layer(&mut self, voxel_size_m: f32) -> LayerId {
        self.asset.add_voxel_layer(voxel_size_m)
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn execute(&mut self, op: EditOp) -> Result<EditOutcome, EditError> {
        match op {
            EditOp::FillRegion {
                layer,
                region,
                material,
            } => self.fill_region(layer, region, material),
        }
    }

    fn fill_region(
        &mut self,
        layer: LayerId,
        region: Region,
        material: u16,
    ) -> Result<EditOutcome, EditError> {
        if region.is_empty() {
            return Err(EditError::InvalidOperation("empty region"));
        }
        if region.volume() > MAX_FILL_VOLUME {
            return Err(EditError::InvalidOperation("region too large"));
        }
        // Validates the layer; also gives the base state the writes are computed against.
        self.asset.layer(layer)?;

        let mut writes = Vec::new();
        let mut changes = Vec::new();
        let mut changed_voxels = 0u64;
        for coord in region.brick_coords() {
            let existing = self.asset.layer(layer)?.storage().get(coord).cloned();
            let sub = region
                .intersection(&Region::of_brick(coord))
                .expect("brick intersects region");
            let mut work = existing
                .as_deref()
                .cloned()
                .unwrap_or_else(|| Brick::constant(0));
            let mut count = 0u64;
            let mut lo = VoxelPos::new(i32::MAX, i32::MAX, i32::MAX);
            let mut hi = VoxelPos::new(i32::MIN, i32::MIN, i32::MIN);
            for z in sub.min.z..sub.max.z {
                for y in sub.min.y..sub.max.y {
                    for x in sub.min.x..sub.max.x {
                        let p = VoxelPos::new(x, y, z);
                        if work.set(p.split().1, material) {
                            count += 1;
                            lo = VoxelPos::new(lo.x.min(x), lo.y.min(y), lo.z.min(z));
                            hi = VoxelPos::new(hi.x.max(x), hi.y.max(y), hi.z.max(z));
                        }
                    }
                }
            }
            if count == 0 {
                continue;
            }
            changed_voxels += count;
            work.normalize();
            let new = if work.is_empty() { None } else { Some(work) };
            let changed = Region::new(lo, VoxelPos::new(hi.x + 1, hi.y + 1, hi.z + 1));
            let expected = match &existing {
                Some(b) => ExpectedBrick::Revision(b.revision()),
                None => ExpectedBrick::Absent,
            };
            writes.push(BrickWrite {
                coord,
                expected,
                new,
                changed,
            });
            changes.push(BrickChange {
                coord,
                before: existing,
                after: None,
                changed,
            });
        }

        if writes.is_empty() {
            return Ok(EditOutcome {
                operation: None,
                asset_token: self.asset.asset_token(),
                layer_token: self.asset.layer_token(layer)?,
                dirty: Vec::new(),
                changed_voxels: 0,
            });
        }

        let operation = OperationId(self.next_op);
        self.next_op += 1;
        let outcome = self.asset.commit(WriteSet {
            layer,
            operation: Some(operation),
            writes,
        })?;
        for ch in &mut changes {
            ch.after = self.asset.layer(layer)?.storage().get(ch.coord).cloned();
        }
        self.redo.clear();
        self.undo.push(Delta {
            operation,
            layer,
            changes,
            expected_now: outcome.post.clone(),
        });
        if self.undo.len() > self.history_limit {
            self.undo.remove(0);
        }
        Ok(EditOutcome {
            operation: Some(operation),
            asset_token: outcome.asset_token,
            layer_token: outcome.layer_token,
            dirty: outcome.dirty,
            changed_voxels,
        })
    }

    pub fn undo(&mut self) -> Result<EditOutcome, EditError> {
        let delta = self.undo.pop().ok_or(EditError::NothingToUndo)?;
        match self.apply(delta, true) {
            Ok((d, out)) => {
                self.redo.push(d);
                Ok(out)
            }
            Err((d, e)) => {
                self.undo.push(d);
                Err(e)
            }
        }
    }

    pub fn redo(&mut self) -> Result<EditOutcome, EditError> {
        let delta = self.redo.pop().ok_or(EditError::NothingToRedo)?;
        match self.apply(delta, false) {
            Ok((d, out)) => {
                self.undo.push(d);
                Ok(out)
            }
            Err((d, e)) => {
                self.redo.push(d);
                Err(e)
            }
        }
    }

    /// Undo/redo is a new forward commit that expects the asset to be exactly in the state the
    /// delta last produced; any other state is a conflict and nothing changes.
    fn apply(
        &mut self,
        mut delta: Delta,
        to_before: bool,
    ) -> Result<(Delta, EditOutcome), (Delta, EditError)> {
        let writes: Vec<BrickWrite> = delta
            .changes
            .iter()
            .map(|c| {
                let target = if to_before { &c.before } else { &c.after };
                let expected = delta
                    .expected_now
                    .iter()
                    .find(|(k, _)| *k == c.coord)
                    .map(|(_, e)| *e)
                    .expect("expected state recorded for every change");
                BrickWrite {
                    coord: c.coord,
                    expected,
                    new: target.as_deref().cloned(),
                    changed: c.changed,
                }
            })
            .collect();
        match self.asset.commit(WriteSet {
            layer: delta.layer,
            operation: Some(delta.operation),
            writes,
        }) {
            Ok(out) => {
                delta.expected_now = out.post.clone();
                // Undo/redo is a forward commit, so bricks get new revisions even though their
                // content returns to an earlier state. Older/newer deltas touching these bricks
                // must expect the new revisions; they are only applicable once they reach the
                // top of their stack, where the expectation is compared again.
                for other in self.undo.iter_mut().chain(self.redo.iter_mut()) {
                    if other.layer != delta.layer {
                        continue;
                    }
                    for (coord, exp) in other.expected_now.iter_mut() {
                        if let Some((_, now)) = out.post.iter().find(|(c, _)| c == coord) {
                            *exp = *now;
                        }
                    }
                }
                let outcome = EditOutcome {
                    operation: Some(delta.operation),
                    asset_token: out.asset_token,
                    layer_token: out.layer_token,
                    dirty: out.dirty,
                    changed_voxels: 0,
                };
                Ok((delta, outcome))
            }
            Err(e) => Err((delta, e.into())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cube_core::{BrickLookup, RevisionOrder};

    fn editor() -> (Editor, LayerId) {
        let mut e = Editor::new(GeometryAsset::new(AssetId(1)));
        let l = e.add_voxel_layer(0.1);
        (e, l)
    }

    fn region(a: [i32; 3], b: [i32; 3]) -> Region {
        Region::new(
            VoxelPos::new(a[0], a[1], a[2]),
            VoxelPos::new(b[0], b[1], b[2]),
        )
    }

    fn material_at(e: &Editor, l: LayerId, p: VoxelPos) -> u16 {
        let (c, loc) = p.split();
        match e.asset().layer(l).unwrap().storage().lookup(c) {
            BrickLookup::Present(b) => b.get(loc),
            _ => 0,
        }
    }

    #[test]
    fn fill_sets_voxels_and_reports_exact_dirty_region() {
        let (mut e, l) = editor();
        let r = region([2, 3, 4], [5, 6, 7]);
        let out = e
            .execute(EditOp::FillRegion {
                layer: l,
                region: r,
                material: 7,
            })
            .unwrap();
        assert_eq!(out.changed_voxels, 27);
        assert_eq!(out.dirty.len(), 1);
        assert_eq!(out.dirty[0].region, r);
        assert_eq!(material_at(&e, l, VoxelPos::new(2, 3, 4)), 7);
        assert_eq!(material_at(&e, l, VoxelPos::new(5, 6, 7)), 0);
        assert_eq!(out.dirty[0].source, out.layer_token);
    }

    #[test]
    fn fill_across_negative_brick_boundary() {
        let (mut e, l) = editor();
        let r = region([-2, -2, -2], [2, 2, 2]);
        e.execute(EditOp::FillRegion {
            layer: l,
            region: r,
            material: 3,
        })
        .unwrap();
        assert_eq!(e.asset().layer(l).unwrap().storage().len(), 8);
        assert_eq!(material_at(&e, l, VoxelPos::new(-1, -1, -1)), 3);
        assert_eq!(material_at(&e, l, VoxelPos::new(1, 1, 1)), 3);
        assert_eq!(material_at(&e, l, VoxelPos::new(2, 2, 2)), 0);
    }

    #[test]
    fn noop_fill_changes_no_revision_and_no_history() {
        let (mut e, l) = editor();
        let t = e.asset().asset_token();
        let out = e
            .execute(EditOp::FillRegion {
                layer: l,
                region: region([0, 0, 0], [4, 4, 4]),
                material: 0,
            })
            .unwrap();
        assert!(out.dirty.is_empty());
        assert_eq!(e.asset().asset_token(), t);
        assert!(!e.can_undo());
    }

    #[test]
    fn clearing_everything_removes_the_brick() {
        let (mut e, l) = editor();
        let r = region([0, 0, 0], [16, 16, 16]);
        e.execute(EditOp::FillRegion {
            layer: l,
            region: r,
            material: 2,
        })
        .unwrap();
        e.execute(EditOp::FillRegion {
            layer: l,
            region: r,
            material: 0,
        })
        .unwrap();
        assert!(e.asset().layer(l).unwrap().storage().is_empty());
        assert!(matches!(
            e.asset()
                .layer(l)
                .unwrap()
                .storage()
                .lookup(BrickCoord::new(0, 0, 0)),
            BrickLookup::KnownEmpty
        ));
    }

    #[test]
    fn invalid_operations_leave_asset_unchanged() {
        let (mut e, l) = editor();
        let t = e.asset().asset_token();
        let empty = EditOp::FillRegion {
            layer: l,
            region: region([1, 1, 1], [1, 5, 5]),
            material: 1,
        };
        assert!(matches!(
            e.execute(empty),
            Err(EditError::InvalidOperation(_))
        ));
        let huge = EditOp::FillRegion {
            layer: l,
            region: region([0, 0, 0], [4096, 4096, 4096]),
            material: 1,
        };
        assert!(matches!(
            e.execute(huge),
            Err(EditError::InvalidOperation(_))
        ));
        let bad = EditOp::FillRegion {
            layer: LayerId(42),
            region: region([0, 0, 0], [1, 1, 1]),
            material: 1,
        };
        assert!(matches!(
            e.execute(bad),
            Err(EditError::Core(CoreError::InvalidLayer(_)))
        ));
        assert_eq!(e.asset().asset_token(), t);
    }

    #[test]
    fn undo_redo_restore_data_and_move_revisions_forward() {
        let (mut e, l) = editor();
        let r = region([0, 0, 0], [20, 4, 4]); // two bricks
        let first = e
            .execute(EditOp::FillRegion {
                layer: l,
                region: r,
                material: 5,
            })
            .unwrap();
        let over = e
            .execute(EditOp::FillRegion {
                layer: l,
                region: region([0, 0, 0], [3, 3, 3]),
                material: 9,
            })
            .unwrap();

        let undone = e.undo().unwrap();
        assert_eq!(material_at(&e, l, VoxelPos::new(1, 1, 1)), 5);
        assert_eq!(
            undone.layer_token.compare(&over.layer_token),
            RevisionOrder::Newer
        );

        e.undo().unwrap();
        assert_eq!(material_at(&e, l, VoxelPos::new(1, 1, 1)), 0);
        assert!(e.asset().layer(l).unwrap().storage().is_empty());
        assert!(e.asset().asset_token().value > first.asset_token.value);

        e.redo().unwrap();
        assert_eq!(material_at(&e, l, VoxelPos::new(19, 3, 3)), 5);
        e.redo().unwrap();
        assert_eq!(material_at(&e, l, VoxelPos::new(1, 1, 1)), 9);
        assert!(matches!(e.redo(), Err(EditError::NothingToRedo)));
    }

    #[test]
    fn new_edit_clears_redo_and_nothing_to_undo_is_typed() {
        let (mut e, l) = editor();
        assert_eq!(e.undo().unwrap_err(), EditError::NothingToUndo);
        e.execute(EditOp::FillRegion {
            layer: l,
            region: region([0, 0, 0], [2, 2, 2]),
            material: 1,
        })
        .unwrap();
        e.undo().unwrap();
        assert!(e.can_redo());
        e.execute(EditOp::FillRegion {
            layer: l,
            region: region([0, 0, 0], [1, 1, 1]),
            material: 2,
        })
        .unwrap();
        assert!(!e.can_redo());
    }

    #[test]
    fn history_is_bounded() {
        let (e, l) = editor();
        let mut e = e.with_history_limit(2);
        for m in 1..=5u16 {
            e.execute(EditOp::FillRegion {
                layer: l,
                region: region([0, 0, 0], [1, 1, 1]),
                material: m,
            })
            .unwrap();
        }
        assert!(e.undo().is_ok());
        assert!(e.undo().is_ok());
        assert_eq!(e.undo().unwrap_err(), EditError::NothingToUndo);
        assert_eq!(material_at(&e, l, VoxelPos::new(0, 0, 0)), 3);
    }

    #[test]
    fn undo_conflicts_when_state_diverged_and_changes_nothing() {
        // Force divergence by replacing a brick behind the history's back via a raw core commit
        // on a clone-free path: edit, then commit directly through a second Editor on the same asset state.
        let (mut e, l) = editor();
        e.execute(EditOp::FillRegion {
            layer: l,
            region: region([0, 0, 0], [2, 2, 2]),
            material: 1,
        })
        .unwrap();
        let mut other = Editor::new(e.asset().clone());
        other
            .execute(EditOp::FillRegion {
                layer: l,
                region: region([0, 0, 0], [1, 1, 1]),
                material: 8,
            })
            .unwrap();
        // Move the diverged asset into `e` while keeping `e`'s history.
        e.asset = other.asset.clone();
        let t = e.asset().asset_token();
        let err = e.undo().unwrap_err();
        assert!(matches!(
            err,
            EditError::Core(CoreError::RevisionConflict { .. })
        ));
        assert_eq!(e.asset().asset_token(), t);
        assert!(e.can_undo());
    }
}
