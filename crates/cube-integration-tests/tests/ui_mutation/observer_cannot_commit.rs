// Observers receive immutable summaries; there is no commit method to reach.
use cube_editing_api::{GeometryDelta, TransactionObserver, TransactionSummary};

struct Bad;

impl TransactionObserver for Bad {
    fn on_committed(&self, summary: &TransactionSummary, _delta: &GeometryDelta) {
        summary.commit();
    }
    fn on_rolled_back(&self, _summary: &TransactionSummary) {}
}

fn main() {}
