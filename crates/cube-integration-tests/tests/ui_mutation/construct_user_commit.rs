// A client cannot build a user commit handle: its fields are private.
use cube_core::UserCommit;

fn main() {
    let shared: cube_core::SharedAsset = todo!();
    let _forged = UserCommit { shared };
}
