// Outside the orchestration crate nobody can call the commit primitive of the user path.
use cube_core::{CommitBatch, OperationId};
use cube_orchestration::CommitPath;

fn main() {
    let path: CommitPath = todo!();
    let batch = CommitBatch::default();
    let _ = path.commit(OperationId::new(1, 1), &batch);
}
