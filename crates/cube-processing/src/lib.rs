//! Cube Processing (experimental spike, draft PR-01, PR-02, PR-04).
//!
//! Depends on `cube-core` only. Contains no editing logic. Provides immutable
//! snapshots, currency validation (stale rejection) and two derived-data
//! computations (voxel cells, solid bounds). No scheduler or threads yet:
//! the task state machine, priorities and cancellation (PR-03, PR-05, PR-06)
//! are not implemented in this spike.

use std::sync::Arc;

use cube_core::{
    AssetId, Brick, BrickCoord, CoreError, ExpectedBrick, GeometryAsset, LayerId, Region,
    RevisionOrder, RevisionToken, VoxelPos,
};

/// Immutable consistent input for derived computations (copy-on-write pins).
#[derive(Clone, Debug)]
pub struct ProcessingSnapshot {
    asset: AssetId,
    layer: LayerId,
    region: Region,
    asset_token: RevisionToken,
    layer_token: RevisionToken,
    bricks: Vec<(BrickCoord, Option<Arc<Brick>>)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Freshness {
    Current,
    Stale,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PublishOutcome {
    Published,
    /// Inputs changed since the snapshot; the result was discarded.
    Stale,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VoxelCell {
    pub pos: VoxelPos,
    pub material: u16,
}

impl ProcessingSnapshot {
    pub fn capture(
        asset: &GeometryAsset,
        layer: LayerId,
        region: Region,
    ) -> Result<Self, CoreError> {
        let pins = asset.pin_region(layer, &region)?;
        Ok(Self {
            asset: asset.id(),
            layer,
            region,
            asset_token: pins.asset_token,
            layer_token: pins.layer_token,
            bricks: pins.bricks,
        })
    }

    pub fn asset_id(&self) -> AssetId {
        self.asset
    }
    pub fn layer_id(&self) -> LayerId {
        self.layer
    }
    pub fn region(&self) -> Region {
        self.region
    }
    pub fn layer_token(&self) -> RevisionToken {
        self.layer_token
    }
    pub fn asset_token(&self) -> RevisionToken {
        self.asset_token
    }

    /// Current iff every brick in the snapshot region is in the same state as when captured
    /// and the asset epoch is unchanged. Edits outside the region do not make it stale.
    pub fn freshness(&self, asset: &GeometryAsset) -> Freshness {
        if asset.id() != self.asset
            || asset.asset_token().compare(&self.asset_token) == RevisionOrder::Incomparable
        {
            return Freshness::Stale;
        }
        for (c, pinned) in &self.bricks {
            let now = match asset.expected_state(self.layer, *c) {
                Ok(s) => s,
                Err(_) => return Freshness::Stale,
            };
            let then = match pinned {
                Some(b) => ExpectedBrick::Revision(b.revision()),
                None => ExpectedBrick::Absent,
            };
            if now != then {
                return Freshness::Stale;
            }
        }
        Freshness::Current
    }

    /// Non-empty voxels inside the snapshot region, deterministic order by brick then index.
    pub fn voxel_cells(&self) -> Vec<VoxelCell> {
        let mut out = Vec::new();
        for (c, b) in &self.bricks {
            let Some(b) = b else { continue };
            for (l, m) in b.occupied() {
                let pos = c.join(l);
                if self.region.contains(pos) {
                    out.push(VoxelCell { pos, material: m });
                }
            }
        }
        out
    }

    /// Tight bounds of non-empty voxels inside the region.
    pub fn solid_bounds(&self) -> Option<Region> {
        let mut acc: Option<Region> = None;
        for cell in self.voxel_cells() {
            let r = Region::new(
                cell.pos,
                VoxelPos::new(cell.pos.x + 1, cell.pos.y + 1, cell.pos.z + 1),
            );
            acc = Some(match acc {
                Some(a) => a.union_bounds(&r),
                None => r,
            });
        }
        acc
    }
}

/// A published derived value together with the snapshot stamp it was computed from.
#[derive(Debug)]
pub struct DerivedSlot<T> {
    value: Option<(T, RevisionToken)>,
}

impl<T> Default for DerivedSlot<T> {
    fn default() -> Self {
        Self { value: None }
    }
}

impl<T> DerivedSlot<T> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Publishes only if the snapshot is still current. Stale results are dropped.
    pub fn publish(
        &mut self,
        snapshot: &ProcessingSnapshot,
        asset: &GeometryAsset,
        value: T,
    ) -> PublishOutcome {
        match snapshot.freshness(asset) {
            Freshness::Current => {
                self.value = Some((value, snapshot.layer_token()));
                PublishOutcome::Published
            }
            Freshness::Stale => PublishOutcome::Stale,
        }
    }

    pub fn get(&self) -> Option<&T> {
        self.value.as_ref().map(|(v, _)| v)
    }

    pub fn stamp(&self) -> Option<RevisionToken> {
        self.value.as_ref().map(|(_, t)| *t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cube_core::{BrickWrite, LocalPos, WriteSet};

    fn put(a: &mut GeometryAsset, l: LayerId, c: BrickCoord, expected: ExpectedBrick, m: u16) {
        let mut b = Brick::constant(0);
        b.set(LocalPos { x: 1, y: 2, z: 3 }, m);
        a.commit(WriteSet {
            layer: l,
            operation: None,
            writes: vec![BrickWrite {
                coord: c,
                expected,
                new: Some(b),
                changed: Region::of_brick(c),
            }],
        })
        .unwrap();
    }

    fn setup() -> (GeometryAsset, LayerId) {
        let mut a = GeometryAsset::new(AssetId(1));
        let l = a.add_voxel_layer(0.1);
        (a, l)
    }

    #[test]
    fn snapshot_is_stable_and_detects_inside_edit() {
        let (mut a, l) = setup();
        let c = BrickCoord::new(0, 0, 0);
        put(&mut a, l, c, ExpectedBrick::Absent, 5);
        let snap = ProcessingSnapshot::capture(&a, l, Region::of_brick(c)).unwrap();
        assert_eq!(snap.freshness(&a), Freshness::Current);
        put(&mut a, l, c, ExpectedBrick::Revision(1), 6);
        assert_eq!(snap.freshness(&a), Freshness::Stale);
        // The snapshot still reads the old data.
        assert_eq!(
            snap.voxel_cells(),
            vec![VoxelCell {
                pos: VoxelPos::new(1, 2, 3),
                material: 5
            }]
        );
    }

    #[test]
    fn edit_outside_region_does_not_make_snapshot_stale() {
        let (mut a, l) = setup();
        let c = BrickCoord::new(0, 0, 0);
        put(&mut a, l, c, ExpectedBrick::Absent, 5);
        let snap = ProcessingSnapshot::capture(&a, l, Region::of_brick(c)).unwrap();
        put(
            &mut a,
            l,
            BrickCoord::new(4, 0, 0),
            ExpectedBrick::Absent,
            1,
        );
        assert_eq!(snap.freshness(&a), Freshness::Current);
    }

    #[test]
    fn creation_of_absent_brick_inside_region_is_detected() {
        let (mut a, l) = setup();
        let c = BrickCoord::new(0, 0, 0);
        let snap = ProcessingSnapshot::capture(&a, l, Region::of_brick(c)).unwrap();
        put(&mut a, l, c, ExpectedBrick::Absent, 1);
        assert_eq!(snap.freshness(&a), Freshness::Stale);
    }

    #[test]
    fn stale_result_is_never_published() {
        let (mut a, l) = setup();
        let c = BrickCoord::new(0, 0, 0);
        put(&mut a, l, c, ExpectedBrick::Absent, 5);
        let snap = ProcessingSnapshot::capture(&a, l, Region::of_brick(c)).unwrap();
        let bounds = snap.solid_bounds();
        put(&mut a, l, c, ExpectedBrick::Revision(1), 6);
        let mut slot = DerivedSlot::new();
        assert_eq!(slot.publish(&snap, &a, bounds), PublishOutcome::Stale);
        assert!(slot.get().is_none());
        let fresh = ProcessingSnapshot::capture(&a, l, Region::of_brick(c)).unwrap();
        assert_eq!(
            slot.publish(&fresh, &a, fresh.solid_bounds()),
            PublishOutcome::Published
        );
        assert!(slot.get().is_some());
    }

    #[test]
    fn new_epoch_invalidates_snapshot() {
        let (mut a, l) = setup();
        let snap =
            ProcessingSnapshot::capture(&a, l, Region::of_brick(BrickCoord::new(0, 0, 0))).unwrap();
        a.begin_new_epoch();
        assert_eq!(snap.freshness(&a), Freshness::Stale);
    }

    #[test]
    fn solid_bounds_are_tight() {
        let (mut a, l) = setup();
        let c = BrickCoord::new(0, 0, 0);
        put(&mut a, l, c, ExpectedBrick::Absent, 5);
        let snap = ProcessingSnapshot::capture(&a, l, Region::of_brick(c)).unwrap();
        assert_eq!(
            snap.solid_bounds().unwrap(),
            Region::new(VoxelPos::new(1, 2, 3), VoxelPos::new(2, 3, 4))
        );
    }
}
