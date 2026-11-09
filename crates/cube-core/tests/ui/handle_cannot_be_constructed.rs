//! A handle cannot be built by struct literal: its field is private.
use cube_core::{AssetHost, AssetId, GeometryAsset, UserCommit};

fn main() {
    let host = AssetHost::new(GeometryAsset::new(AssetId::new(1), vec![]).unwrap());
    let reader = host.reader();
    let _forged = UserCommit { shared: reader };
}
