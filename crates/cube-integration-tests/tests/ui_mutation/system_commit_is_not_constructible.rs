// A request handler cannot make itself a system writer.
use cube_core::SystemCommit;

fn main() {
    let shared: cube_core::SharedAsset = todo!();
    let _forged = SystemCommit { shared };
}
