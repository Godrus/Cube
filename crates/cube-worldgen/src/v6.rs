//! Algorithm "terrain-v6": terrain-v5 plus a small dungeon under every ruin (frozen once
//! released: any change of output needs a new version). Everything outside a ruin's footprint,
//! and everything of a ruin above its floor, is exactly v5. Integer arithmetic only.
//!
//! Coordinates below are relative to the ruin: `dx`, `dz` from its centre column, `dy` from its
//! floor (the floor's top voxel is `dy = 0`). The block `dy = -8..=-1` inside the footprint
//! (`|dx|, |dz| <= 5`) becomes solid stone, with these cut into it:
//!
//! * **The stairway.** Two columns wide (`dx` in `-4..=-3`), six steps going south to north
//!   (`dz` from `-4` to `1`). Step `k = dz + 4` has its walking top at `dy = -(k + 1)`, with
//!   three voxels of air above it. The first three steps open a hole in the floor (`dz` from
//!   `-4` to `-2`), so a player on the floor walks down one voxel at a time.
//! * **The chamber.** `dx` in `-4..=4`, `dz` in `2..=4`: a red-brick floor at `dy = -7` (one
//!   voxel below the last step), four voxels of air above it (`dy = -6..=-3`), stone ceiling.
//!   A one-voxel red-brick pedestal stands at `dx = 0`, `dz = 4`, `dy = -6`: the place of the
//!   deep cache (the host decides what that is, see `cube-server`).

use crate::materials;

/// Columns of the stairway (`dx`).
const STAIR_DX: [i64; 2] = [-4, -3];
/// First stair column (`dz`) and the number of steps.
const STAIR_DZ0: i64 = -4;
const STEPS: i64 = 6;

/// Position of the pedestal of the deep cache, relative to the ruin's centre column.
pub const PEDESTAL: (i64, i64) = (0, 4);
/// Height of the walking surface of the chamber floor relative to the ruin's floor.
pub const CHAMBER_FLOOR: i64 = -7;

/// What the dungeon says about the voxel at `(dx, dy, dz)` relative to the ruin: `Some(m)` if the
/// dungeon decides it (0 = air), `None` if the voxel is the ruin's own (version 5 rules).
pub fn dungeon(dx: i64, dz: i64, dy: i64) -> Option<u16> {
    if dx.abs() > 5 || dz.abs() > 5 || !(-8..=0).contains(&dy) {
        return None;
    }
    let stair = STAIR_DX.contains(&dx) && (STAIR_DZ0..STAIR_DZ0 + STEPS).contains(&dz);
    if dy == 0 {
        // Only the floor above the first steps is opened.
        return (stair && dz <= -2).then_some(0);
    }
    if stair {
        let k = dz - STAIR_DZ0;
        let top = -(k + 1);
        return Some(if dy <= top {
            materials::STONE
        } else if dy <= top + 3 {
            0
        } else {
            materials::STONE
        });
    }
    let chamber = (-4..=4).contains(&dx) && (2..=4).contains(&dz);
    if chamber {
        if (dx, dz) == PEDESTAL && dy == CHAMBER_FLOOR + 1 {
            return Some(materials::BRICK);
        }
        return Some(if dy == CHAMBER_FLOOR {
            materials::BRICK
        } else if dy > CHAMBER_FLOOR && dy <= -3 {
            0
        } else {
            materials::STONE
        });
    }
    Some(materials::STONE)
}
