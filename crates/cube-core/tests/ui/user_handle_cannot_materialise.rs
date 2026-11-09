//! System generation is not reachable from the user handle.
use cube_core::{AssetHost, AssetId, BrickCoord, CellChange, GeometryAsset, LayerId, OperationId};

fn main() {
    let host = AssetHost::new(GeometryAsset::new(AssetId::new(1), vec![]).unwrap());
    let handles = host.into_handles();
    let _ = handles.user.materialise(OperationId::new(1, 1), LayerId::new(1), 0, BrickCoord::new(0, 0, 0), CellChange::KnownEmpty);
    let _ = handles.user.evict(LayerId::new(1), 0, BrickCoord::new(0, 0, 0));
}
