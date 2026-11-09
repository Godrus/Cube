//! A commit handle cannot be duplicated behind the composition root's back.
use cube_core::{AssetHost, AssetId, GeometryAsset};

fn main() {
    let host = AssetHost::new(GeometryAsset::new(AssetId::new(1), vec![]).unwrap());
    let handles = host.into_handles();
    let _copy = handles.user.clone();
}
