//! A read-only SharedAsset has no commit method (the primitive is crate-private).
use cube_core::{ActorKind, CommitBatch, OperationId, SharedAsset};

fn attempt(reader: &SharedAsset) {
    let _ = reader.commit(ActorKind::User, OperationId::new(1, 1), &CommitBatch::default());
}

fn main() {}
