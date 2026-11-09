//! Dirty rule for derived voxel meshes (VOX-04): which brick meshes must be rebuilt after a
//! commit.
//!
//! A brick's faces depend on its own voxels and on the voxels directly across its six sides, so
//! a change in a brick can change the meshes of that brick and of its six face neighbours, and
//! of nothing else (edge and corner neighbours do not matter). That is tighter than the
//! conservative 3x3x3 apron invalidation of Processing, which stays correct (it also marks
//! results stale); this rule says what actually has to be recomputed. "Incremental equals full
//! rebuild" is verified against real edits through Orchestration in the integration suite.
//!
//! The "section" of the contract is the brick: derived meshes are per brick.

use std::collections::BTreeSet;

use cube_core::{BrickCoord, CellKey, DirtyRegion};

const OFFSETS: [[i64; 3]; 7] = [
    [0, 0, 0],
    [1, 0, 0],
    [-1, 0, 0],
    [0, 1, 0],
    [0, -1, 0],
    [0, 0, 1],
    [0, 0, -1],
];

/// The brick meshes that must be rebuilt after the given commit results, in ascending order. A
/// neighbour beyond the end of the `i64` grid does not exist and is skipped.
pub fn rebuild_set(dirty: &[DirtyRegion]) -> BTreeSet<CellKey> {
    let mut out = BTreeSet::new();
    for d in dirty {
        let level = d.region.level();
        for b in d.region.bricks() {
            for o in OFFSETS {
                let (Some(x), Some(y), Some(z)) = (
                    b.x.checked_add(o[0]),
                    b.y.checked_add(o[1]),
                    b.z.checked_add(o[2]),
                ) else {
                    continue;
                };
                if let Ok(k) = CellKey::new(d.layer, level, BrickCoord::new(x, y, z)) {
                    out.insert(k);
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use cube_core::{
        ActorKind, AssetId, ChangeType, Epoch, LayerId, OperationId, Region, ResourceId, Revision,
        RevisionToken, VoxelPos,
    };

    fn dirty(b: BrickCoord) -> DirtyRegion {
        let o = b.origin().unwrap();
        DirtyRegion {
            asset: AssetId::new(1),
            layer: LayerId::new(1),
            region: Region::new(0, o, VoxelPos::new(o.x + 16, o.y + 16, o.z + 16)).unwrap(),
            change: ChangeType::Geometry,
            source: RevisionToken::new(
                ResourceId::Asset(AssetId::new(1)),
                Epoch::INITIAL,
                Revision::new(1),
            ),
            operation: OperationId::new(1, 1),
            actor: ActorKind::User,
        }
    }

    #[test]
    fn a_change_dirties_the_brick_and_its_six_face_neighbours_only() {
        let s = rebuild_set(&[dirty(BrickCoord::new(-1, 0, 5))]);
        assert_eq!(s.len(), 7);
        assert!(!s.iter().any(|k| {
            let b = k.brick();
            (b.x - -1).abs() + b.y.abs() + (b.z - 5).abs() > 1
        }));
    }

    #[test]
    fn overlapping_changes_are_merged_and_the_grid_end_is_safe() {
        let s = rebuild_set(&[
            dirty(BrickCoord::new(0, 0, 0)),
            dirty(BrickCoord::new(1, 0, 0)),
        ]);
        assert_eq!(s.len(), 7 + 7 - 2);
        let edge = rebuild_set(&[dirty(BrickCoord::new(i64::MAX / 16 - 1, 0, 0))]);
        assert!(edge.len() <= 7 && !edge.is_empty());
    }
}
