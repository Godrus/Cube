//! Brick-level bounds slot (PROC-06, PR-08 subset; needs CORE-15 / C-05, Provisional).
//!
//! [`BoundsProvider`] derives the [`GeometryBounds`] of one brick from a snapshot. It is
//! conservative (a loaded, non-empty brick is bounded by its whole box; C-05 leaves tight versus
//! conservative open) and it never misreads absent data:
//!
//! * `Present` gives the box of the brick;
//! * `KnownEmpty` gives `Empty`;
//! * `OutsideBounds` (beyond the layer's bounds, where no data can exist) gives `Empty`;
//! * `NotLoaded` and `LoadFailure` are **failures**: the bounds are unknown, and "unknown" is
//!   never reported as `Empty`.
//!
//! BVH and spatial-hierarchy slots (PR-08) are not part of TC-01 and are not implemented.

use std::sync::Arc;

use cube_core::{CellState, CoreError, GeometryBounds};

use crate::provider::{
    ChangeMask, ComputeContext, DerivedKind, Payload, Provider, ProviderFailure,
};

/// The derived kind of brick bounds. Processing-owned; other modules choose ids outside
/// `0xB000_0000..0xB001_0000`.
pub const BOUNDS_KIND: DerivedKind = DerivedKind(0xB000_0001);

/// Provider of per-brick [`GeometryBounds`]; the payload is an `Arc<GeometryBounds>`.
#[derive(Debug, Default, Clone, Copy)]
pub struct BoundsProvider;

impl Provider for BoundsProvider {
    fn kind(&self) -> DerivedKind {
        BOUNDS_KIND
    }
    fn name(&self) -> &str {
        "brick-bounds"
    }
    fn version(&self) -> u32 {
        1
    }
    fn consumes(&self) -> ChangeMask {
        // Occupancy can move the bounds; material and attribute changes cannot.
        ChangeMask::GEOMETRY
    }
    fn compute(&self, ctx: &ComputeContext<'_>) -> Result<Payload, ProviderFailure> {
        ctx.token.check()?;
        let key = ctx.target.key;
        let state = ctx
            .snapshot
            .cell(key.brick())
            .map_err(|e| ProviderFailure::Failed(e.to_string()))?;
        let bounds = match state {
            CellState::Present(_) => GeometryBounds::from_brick(key.level(), key.brick())
                .map_err(|e| ProviderFailure::Failed(e.to_string()))?,
            CellState::KnownEmpty | CellState::OutsideBounds => GeometryBounds::Empty,
            CellState::NotLoaded => {
                return Err(ProviderFailure::Failed(
                    "brick is not loaded: bounds unknown".to_owned(),
                ));
            }
            CellState::LoadFailure(r) => {
                return Err(ProviderFailure::Failed(format!(
                    "brick failed to load ({r:?}): bounds unknown"
                )));
            }
        };
        Ok(Arc::new(bounds))
    }
}

/// Union of brick bounds of one level (for a region or a layer); `Empty` for no input.
pub fn union_all(
    bounds: impl IntoIterator<Item = GeometryBounds>,
) -> Result<GeometryBounds, CoreError> {
    bounds
        .into_iter()
        .try_fold(GeometryBounds::Empty, |acc, b| acc.union(&b))
}

#[cfg(all(test, not(loom)))]
mod tests {
    use super::*;
    use cube_core::BrickCoord;

    fn from(b: i64) -> GeometryBounds {
        GeometryBounds::from_brick(0, BrickCoord::new(b, 0, 0)).unwrap()
    }

    #[test]
    fn union_all_of_nothing_is_empty_and_of_bricks_is_their_hull() {
        assert_eq!(union_all([]), Ok(GeometryBounds::Empty));
        let hull = union_all([from(-1), from(3), GeometryBounds::Empty]).unwrap();
        assert!(hull.contains(&from(-1)) && hull.contains(&from(3)) && hull.contains(&from(0)));
    }

    #[test]
    fn mixed_levels_are_an_error_not_a_silent_mix() {
        let fine = GeometryBounds::from_brick(1, BrickCoord::new(0, 0, 0)).unwrap();
        assert_eq!(union_all([from(0), fine]), Err(CoreError::InvalidLevel(1)));
    }

    #[test]
    fn kind_is_stable_and_provider_metadata_is_fixed() {
        let p = BoundsProvider;
        assert_eq!(p.kind(), BOUNDS_KIND);
        assert_eq!(p.apron(), 0);
        assert!(p.consumes().contains(cube_core::ChangeType::Geometry));
        assert!(!p.consumes().contains(cube_core::ChangeType::Material));
    }
}
