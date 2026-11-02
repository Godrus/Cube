//! GeometryAsset, voxel layer and the commit primitive (draft C-01, C-02, C-08, C-16..C-18).
//!
//! The commit primitive is the only writer of revision values and of authoritative bricks.
//! Commit protocol (proposal, semantics section 4.1): validate every expectation, then apply
//! all writes and increment brick/layer/asset revisions by exactly +1 each; all-or-nothing.

use std::collections::HashSet;
use std::sync::Arc;

use crate::brick::{Brick, BrickStorage};
use crate::coord::BrickCoord;
use crate::dirty::{ChangeType, DirtyRegion};
use crate::error::CoreError;
use crate::ids::{AssetId, LayerId, OperationId};
use crate::region::Region;
use crate::revision::{BrickKey, ResourceId, RevisionToken};

/// What a writer observed for a brick before computing its write.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExpectedBrick {
    Absent,
    Revision(u64),
}

#[derive(Clone, Debug)]
pub struct BrickWrite {
    pub coord: BrickCoord,
    pub expected: ExpectedBrick,
    /// `None` removes the brick.
    pub new: Option<Brick>,
    /// Bounding region of the voxels that actually changed in this brick.
    pub changed: Region,
}

#[derive(Clone, Debug)]
pub struct WriteSet {
    pub layer: LayerId,
    pub operation: Option<OperationId>,
    pub writes: Vec<BrickWrite>,
}

#[derive(Clone, Debug)]
pub struct CommitOutcome {
    pub asset_token: RevisionToken,
    pub layer_token: RevisionToken,
    /// State of every written brick after the commit (used by History for undo validation).
    pub post: Vec<(BrickCoord, ExpectedBrick)>,
    pub dirty: Vec<DirtyRegion>,
}

#[derive(Clone, Debug)]
pub struct VoxelLayer {
    id: LayerId,
    voxel_size_m: f32,
    revision: u64,
    storage: BrickStorage,
}

impl VoxelLayer {
    pub fn id(&self) -> LayerId {
        self.id
    }
    pub fn voxel_size_m(&self) -> f32 {
        self.voxel_size_m
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn storage(&self) -> &BrickStorage {
        &self.storage
    }
}

#[derive(Clone, Debug)]
pub struct GeometryAsset {
    id: AssetId,
    epoch: u32,
    revision: u64,
    layers: Vec<VoxelLayer>,
}

/// Brick references pinned at one consistent cut. Immutable: bricks are shared `Arc`s that
/// writers replace (copy-on-write), never mutate.
#[derive(Clone, Debug)]
pub struct BrickPins {
    pub asset_token: RevisionToken,
    pub layer_token: RevisionToken,
    /// One entry per brick coordinate of the requested region.
    pub bricks: Vec<(BrickCoord, Option<Arc<Brick>>)>,
}

/// Export/import views (no file format here; format belongs to persistence).
#[derive(Clone, Debug)]
pub struct LayerExport {
    pub id: LayerId,
    pub voxel_size_m: f32,
    pub revision: u64,
    pub bricks: Vec<(BrickCoord, Arc<Brick>)>,
    pub tombstones: Vec<(BrickCoord, u64)>,
}

#[derive(Clone, Debug)]
pub struct AssetExport {
    pub id: AssetId,
    pub epoch: u32,
    pub revision: u64,
    pub layers: Vec<LayerExport>,
}

impl GeometryAsset {
    pub fn new(id: AssetId) -> Self {
        Self {
            id,
            epoch: 0,
            revision: 0,
            layers: Vec::new(),
        }
    }

    pub fn id(&self) -> AssetId {
        self.id
    }

    pub fn epoch(&self) -> u32 {
        self.epoch
    }

    pub fn layers(&self) -> &[VoxelLayer] {
        &self.layers
    }

    pub fn layer(&self, id: LayerId) -> Result<&VoxelLayer, CoreError> {
        self.layers
            .iter()
            .find(|l| l.id == id)
            .ok_or(CoreError::InvalidLayer(id))
    }

    /// Structural change: increments the asset revision.
    pub fn add_voxel_layer(&mut self, voxel_size_m: f32) -> LayerId {
        let id = LayerId(
            self.layers
                .iter()
                .map(|l| l.id.0)
                .max()
                .map_or(1, |m| m + 1),
        );
        self.layers.push(VoxelLayer {
            id,
            voxel_size_m,
            revision: 0,
            storage: BrickStorage::new(),
        });
        self.revision += 1;
        id
    }

    pub fn asset_token(&self) -> RevisionToken {
        RevisionToken::new(ResourceId::Asset(self.id), self.epoch, self.revision)
    }

    pub fn layer_token(&self, id: LayerId) -> Result<RevisionToken, CoreError> {
        let l = self.layer(id)?;
        Ok(RevisionToken::new(
            ResourceId::Layer(id),
            self.epoch,
            l.revision,
        ))
    }

    pub fn brick_token(
        &self,
        layer: LayerId,
        c: BrickCoord,
    ) -> Result<Option<RevisionToken>, CoreError> {
        let l = self.layer(layer)?;
        Ok(l.storage.get(c).map(|b| {
            RevisionToken::new(
                ResourceId::Brick(BrickKey { layer, coord: c }),
                self.epoch,
                b.revision(),
            )
        }))
    }

    pub fn expected_state(
        &self,
        layer: LayerId,
        c: BrickCoord,
    ) -> Result<ExpectedBrick, CoreError> {
        let l = self.layer(layer)?;
        Ok(match l.storage.get(c) {
            Some(b) => ExpectedBrick::Revision(b.revision()),
            None => ExpectedBrick::Absent,
        })
    }

    /// Consistent cut of the bricks intersecting `region`.
    pub fn pin_region(&self, layer: LayerId, region: &Region) -> Result<BrickPins, CoreError> {
        let l = self.layer(layer)?;
        let bricks = region
            .brick_coords()
            .into_iter()
            .map(|c| (c, l.storage.get(c).map(Arc::clone)))
            .collect();
        Ok(BrickPins {
            asset_token: self.asset_token(),
            layer_token: self.layer_token(layer)?,
            bricks,
        })
    }

    /// The commit primitive. Fails without modifying anything.
    pub fn commit(&mut self, ws: WriteSet) -> Result<CommitOutcome, CoreError> {
        let layer_idx = self
            .layers
            .iter()
            .position(|l| l.id == ws.layer)
            .ok_or(CoreError::InvalidLayer(ws.layer))?;

        // Validation phase: nothing is mutated.
        let mut seen = HashSet::new();
        let mut conflicts = Vec::new();
        for w in &ws.writes {
            if !seen.insert(w.coord) {
                return Err(CoreError::InvalidWriteSet("duplicate brick in write set"));
            }
            if w.new.is_none() && w.expected == ExpectedBrick::Absent {
                return Err(CoreError::InvalidWriteSet("removal of an absent brick"));
            }
            if w.changed.is_empty() {
                return Err(CoreError::InvalidWriteSet("empty changed region"));
            }
            let current = match self.layers[layer_idx].storage.get(w.coord) {
                Some(b) => ExpectedBrick::Revision(b.revision()),
                None => ExpectedBrick::Absent,
            };
            if current != w.expected {
                conflicts.push(w.coord);
            }
        }
        if !conflicts.is_empty() {
            return Err(CoreError::RevisionConflict { bricks: conflicts });
        }

        // No-op write sets change nothing and increment nothing.
        if ws.writes.is_empty() {
            return Ok(CommitOutcome {
                asset_token: self.asset_token(),
                layer_token: self.layer_token(ws.layer)?,
                post: Vec::new(),
                dirty: Vec::new(),
            });
        }

        // Apply phase (cannot fail).
        let mut post = Vec::with_capacity(ws.writes.len());
        let mut dirty_region: Option<Region> = None;
        let layer = &mut self.layers[layer_idx];
        for w in ws.writes {
            dirty_region = Some(match dirty_region {
                Some(r) => r.union_bounds(&w.changed),
                None => w.changed,
            });
            match w.new {
                Some(mut b) => {
                    let rev = match w.expected {
                        ExpectedBrick::Revision(r) => r + 1,
                        ExpectedBrick::Absent => layer.storage.next_revision_for_new(w.coord),
                    };
                    b.set_revision(rev);
                    layer.storage.insert(w.coord, Arc::new(b));
                    post.push((w.coord, ExpectedBrick::Revision(rev)));
                }
                None => {
                    let old = match w.expected {
                        ExpectedBrick::Revision(r) => r,
                        ExpectedBrick::Absent => 0,
                    };
                    layer.storage.remove(w.coord, old + 1);
                    post.push((w.coord, ExpectedBrick::Absent));
                }
            }
        }
        layer.revision += 1;
        self.revision += 1;

        let layer_token = self.layer_token(ws.layer)?;
        let dirty = vec![DirtyRegion {
            asset: self.id,
            layer: ws.layer,
            region: dirty_region.expect("non-empty writes"),
            change: ChangeType::Geometry,
            source: layer_token,
            operation: ws.operation,
        }];
        Ok(CommitOutcome {
            asset_token: self.asset_token(),
            layer_token,
            post,
            dirty,
        })
    }

    pub fn export_view(&self) -> AssetExport {
        AssetExport {
            id: self.id,
            epoch: self.epoch,
            revision: self.revision,
            layers: self
                .layers
                .iter()
                .map(|l| LayerExport {
                    id: l.id,
                    voxel_size_m: l.voxel_size_m,
                    revision: l.revision,
                    bricks: l.storage.bricks_sorted(),
                    tombstones: l.storage.tombstones_sorted(),
                })
                .collect(),
        }
    }

    pub fn import_view(e: AssetExport) -> Result<Self, CoreError> {
        let mut ids = HashSet::new();
        let mut layers = Vec::new();
        for l in e.layers {
            if !ids.insert(l.id) {
                return Err(CoreError::InvalidImport("duplicate layer id"));
            }
            let mut coords = HashSet::new();
            if !l.bricks.iter().all(|(c, _)| coords.insert(*c)) {
                return Err(CoreError::InvalidImport("duplicate brick"));
            }
            layers.push(VoxelLayer {
                id: l.id,
                voxel_size_m: l.voxel_size_m,
                revision: l.revision,
                storage: BrickStorage::restore(l.bricks, l.tombstones),
            });
        }
        Ok(Self {
            id: e.id,
            epoch: e.epoch,
            revision: e.revision,
            layers,
        })
    }

    /// Recovery/reload: tokens issued before become `Incomparable` (semantics section 6).
    pub fn begin_new_epoch(&mut self) {
        self.epoch += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coord::VoxelPos;
    use crate::revision::RevisionOrder;

    fn write(coord: BrickCoord, expected: ExpectedBrick, m: u16) -> BrickWrite {
        let mut b = Brick::constant(0);
        b.set(crate::coord::LocalPos { x: 0, y: 0, z: 0 }, m);
        BrickWrite {
            coord,
            expected,
            new: Some(b),
            changed: Region::of_brick(coord),
        }
    }

    fn setup() -> (GeometryAsset, LayerId) {
        let mut a = GeometryAsset::new(AssetId(1));
        let l = a.add_voxel_layer(0.1);
        (a, l)
    }

    #[test]
    fn hierarchy_increments_exactly_once_per_commit() {
        let (mut a, l) = setup();
        let a0 = a.asset_token();
        let l0 = a.layer_token(l).unwrap();
        let ws = WriteSet {
            layer: l,
            operation: None,
            writes: vec![
                write(BrickCoord::new(0, 0, 0), ExpectedBrick::Absent, 1),
                write(BrickCoord::new(1, 0, 0), ExpectedBrick::Absent, 1),
            ],
        };
        let out = a.commit(ws).unwrap();
        assert_eq!(out.asset_token.value, a0.value + 1);
        assert_eq!(out.layer_token.value, l0.value + 1);
        assert_eq!(
            a.brick_token(l, BrickCoord::new(0, 0, 0))
                .unwrap()
                .unwrap()
                .value,
            1
        );
        assert_eq!(out.asset_token.compare(&a0), RevisionOrder::Newer);
        assert_eq!(out.dirty.len(), 1);
    }

    #[test]
    fn conflict_is_atomic() {
        let (mut a, l) = setup();
        let c = BrickCoord::new(0, 0, 0);
        a.commit(WriteSet {
            layer: l,
            operation: None,
            writes: vec![write(c, ExpectedBrick::Absent, 1)],
        })
        .unwrap();
        let before = a.asset_token();
        let ws = WriteSet {
            layer: l,
            operation: None,
            writes: vec![
                write(BrickCoord::new(5, 5, 5), ExpectedBrick::Absent, 2), // would be valid
                write(c, ExpectedBrick::Revision(99), 2),                  // conflicts
            ],
        };
        let err = a.commit(ws).unwrap_err();
        assert_eq!(err, CoreError::RevisionConflict { bricks: vec![c] });
        assert_eq!(a.asset_token(), before);
        assert!(a
            .layer(l)
            .unwrap()
            .storage()
            .get(BrickCoord::new(5, 5, 5))
            .is_none());
    }

    #[test]
    fn disjoint_writers_both_commit() {
        let (mut a, l) = setup();
        let c1 = BrickCoord::new(0, 0, 0);
        let c2 = BrickCoord::new(9, 0, 0);
        a.commit(WriteSet {
            layer: l,
            operation: None,
            writes: vec![write(c1, ExpectedBrick::Absent, 1)],
        })
        .unwrap();
        // Second writer observed c2 as absent before the first commit; layer token moved, no conflict.
        a.commit(WriteSet {
            layer: l,
            operation: None,
            writes: vec![write(c2, ExpectedBrick::Absent, 1)],
        })
        .unwrap();
        assert_eq!(a.layer_token(l).unwrap().value, 2);
    }

    #[test]
    fn removal_leaves_tombstone_and_revisions_never_reused() {
        let (mut a, l) = setup();
        let c = BrickCoord::new(0, 0, 0);
        a.commit(WriteSet {
            layer: l,
            operation: None,
            writes: vec![write(c, ExpectedBrick::Absent, 1)],
        })
        .unwrap();
        let rm = BrickWrite {
            coord: c,
            expected: ExpectedBrick::Revision(1),
            new: None,
            changed: Region::of_brick(c),
        };
        a.commit(WriteSet {
            layer: l,
            operation: None,
            writes: vec![rm],
        })
        .unwrap();
        assert!(a.layer(l).unwrap().storage().get(c).is_none());
        a.commit(WriteSet {
            layer: l,
            operation: None,
            writes: vec![write(c, ExpectedBrick::Absent, 3)],
        })
        .unwrap();
        assert_eq!(a.brick_token(l, c).unwrap().unwrap().value, 3);
    }

    #[test]
    fn empty_write_set_changes_nothing() {
        let (mut a, l) = setup();
        let t = a.asset_token();
        let out = a
            .commit(WriteSet {
                layer: l,
                operation: None,
                writes: vec![],
            })
            .unwrap();
        assert!(out.dirty.is_empty());
        assert_eq!(a.asset_token(), t);
    }

    #[test]
    fn duplicate_and_invalid_write_sets_rejected() {
        let (mut a, l) = setup();
        let c = BrickCoord::new(0, 0, 0);
        let dup = WriteSet {
            layer: l,
            operation: None,
            writes: vec![
                write(c, ExpectedBrick::Absent, 1),
                write(c, ExpectedBrick::Absent, 1),
            ],
        };
        assert!(matches!(a.commit(dup), Err(CoreError::InvalidWriteSet(_))));
        let bad_layer = WriteSet {
            layer: LayerId(99),
            operation: None,
            writes: vec![],
        };
        assert_eq!(
            a.commit(bad_layer).unwrap_err(),
            CoreError::InvalidLayer(LayerId(99))
        );
    }

    #[test]
    fn pins_are_immutable_after_later_commit() {
        let (mut a, l) = setup();
        let c = BrickCoord::new(0, 0, 0);
        a.commit(WriteSet {
            layer: l,
            operation: None,
            writes: vec![write(c, ExpectedBrick::Absent, 1)],
        })
        .unwrap();
        let region = Region::of_brick(c);
        let pins = a.pin_region(l, &region).unwrap();
        a.commit(WriteSet {
            layer: l,
            operation: None,
            writes: vec![write(c, ExpectedBrick::Revision(1), 9)],
        })
        .unwrap();
        let pinned = pins.bricks[0].1.as_ref().unwrap();
        assert_eq!(pinned.get(crate::coord::LocalPos { x: 0, y: 0, z: 0 }), 1);
        assert_eq!(pinned.revision(), 1);
    }

    #[test]
    fn new_epoch_makes_old_tokens_incomparable() {
        let (mut a, _) = setup();
        let old = a.asset_token();
        a.begin_new_epoch();
        assert_eq!(a.asset_token().compare(&old), RevisionOrder::Incomparable);
    }

    #[test]
    fn export_import_roundtrip() {
        let (mut a, l) = setup();
        a.commit(WriteSet {
            layer: l,
            operation: None,
            writes: vec![write(BrickCoord::new(-1, 0, 0), ExpectedBrick::Absent, 4)],
        })
        .unwrap();
        let b = GeometryAsset::import_view(a.export_view()).unwrap();
        assert_eq!(b.asset_token(), a.asset_token());
        let p = VoxelPos::new(-16, 0, 0).split();
        assert_eq!(b.layer(l).unwrap().storage().get(p.0).unwrap().get(p.1), 4);
    }
}
