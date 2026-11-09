//! The actor kind is fixed by the handle: there is no parameter to claim another origin.
use cube_core::{ActorKind, AssetHost, AssetId, CommitBatch, GeometryAsset, OperationId};

fn main() {
    let host = AssetHost::new(GeometryAsset::new(AssetId::new(1), vec![]).unwrap());
    let handles = host.into_handles();
    let _ = handles.system.commit(ActorKind::User, OperationId::new(1, 1), &CommitBatch::default());
}
