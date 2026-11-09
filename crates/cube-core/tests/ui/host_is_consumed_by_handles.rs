//! The host hands out handles once; it cannot be used to mint a second set.
use cube_core::{AssetHost, AssetId, GeometryAsset};

fn main() {
    let host = AssetHost::new(GeometryAsset::new(AssetId::new(1), vec![]).unwrap());
    let _first = host.into_handles();
    let _second = host.into_handles();
}
