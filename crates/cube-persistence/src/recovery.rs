//! Journal reconciliation: the recovery procedure for the write-ahead phases (CB-1, DEC-086).
//!
//! Orchestration journals `Intent` before the Core commit and `Confirmed` (or `Aborted`) after.
//! After a crash an `Intent` may stand alone. Persistence resolves it by comparing with the
//! operation ids that Core's commit log shows as applied:
//!
//! * applied: the change happened, so a `Confirmed` record is what is missing (the operation is
//!   **not** re-applied, and its id stays taken);
//! * not applied: the change did not happen, so the intent is closed with `Aborted`; its id is
//!   free again for a retry by the client.
//!
//! The function is pure; writing the resulting records and reading the live commit log are the
//! caller's job. Limit (stated, not hidden): Core keeps a bounded commit log; an intent older
//! than that window cannot be decided from the log alone and is reported as `Undecidable`, to be
//! resolved by comparing revisions with the recovered snapshot (PS-10).

use std::collections::{HashMap, HashSet};

use cube_core::{CommitError, OperationId, RecoveryCommit};
use cube_editing_api::{JournalEntry, JournalPhase};

use crate::codec::{Limits, decode_batch};

/// What to do about one pending intent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolution {
    /// Core applied it: append `Confirmed`.
    Confirm(OperationId),
    /// Core did not apply it: append `Aborted`.
    Abort(OperationId),
    /// The log window does not reach back far enough to decide.
    Undecidable(OperationId),
}

/// Resolves the pending intents of `entries` (in journal order). `applied` are the operation ids
/// in Core's commit log; `log_window_complete` says whether the log reaches back to the first
/// intent (if not, an id missing from `applied` is undecidable rather than "not applied").
pub fn reconcile(
    entries: &[JournalEntry],
    applied: &HashSet<OperationId>,
    log_window_complete: bool,
) -> Vec<Resolution> {
    let mut state: HashMap<OperationId, JournalPhase> = HashMap::new();
    let mut order: Vec<OperationId> = Vec::new();
    for e in entries {
        match state.insert(e.operation, e.phase) {
            None => order.push(e.operation),
            Some(JournalPhase::Intent) => {}
            // A closed operation id that gets a new intent starts a new cycle (a retry after
            // an abort): it is pending again until closed.
            Some(_) => {}
        }
    }
    order
        .into_iter()
        .filter(|id| state[id] == JournalPhase::Intent)
        .map(|id| {
            if applied.contains(&id) {
                Resolution::Confirm(id)
            } else if log_window_complete {
                Resolution::Abort(id)
            } else {
                Resolution::Undecidable(id)
            }
        })
        .collect()
}

/// What [`replay_confirmed`] did.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReplayReport {
    /// Confirmed changes that the restored world lacked and that were applied again.
    pub replayed: Vec<OperationId>,
    /// Confirmed changes whose batch no longer applies because the restored world already has
    /// them or has moved on (a revision expectation failed). Not an error.
    pub already_applied_or_superseded: Vec<OperationId>,
    /// Operations with an `Intent` and no later record: not replayed, left for the decision
    /// of `resolve_pending`.
    pub undecided: Vec<OperationId>,
    /// Confirmed operations that could not be replayed (no payload, or a payload that does not
    /// decode or apply), with the reason. Nothing is hidden: these are lost changes.
    pub failed: Vec<(OperationId, String)>,
}

/// Replays the *confirmed* changes of a journal onto a restored world, in journal order, through
/// the recovery writer. The batches carry the revision expectations they were committed with,
/// so replaying is idempotent: a batch the world already contains fails its expectation and is
/// counted, never applied twice. Only operations whose last record is `Confirmed` are replayed;
/// an unresolved `Intent` is reported, not applied (it may never have reached Core).
pub fn replay_confirmed(
    entries: &[JournalEntry],
    recovery: &RecoveryCommit,
    limits: &Limits,
) -> ReplayReport {
    let mut payload: HashMap<OperationId, &[u8]> = HashMap::new();
    let mut last: HashMap<OperationId, JournalPhase> = HashMap::new();
    let mut order: Vec<OperationId> = Vec::new();
    for e in entries {
        if e.phase == JournalPhase::Intent && !e.payload.is_empty() {
            payload.insert(e.operation, &e.payload);
        }
        if last.insert(e.operation, e.phase).is_none() {
            order.push(e.operation);
        }
    }
    let mut report = ReplayReport::default();
    for id in order {
        match last[&id] {
            JournalPhase::Aborted => {}
            JournalPhase::Intent => report.undecided.push(id),
            JournalPhase::Confirmed => {
                let Some(bytes) = payload.get(&id) else {
                    report
                        .failed
                        .push((id, "no write payload in the journal".to_owned()));
                    continue;
                };
                let batch = match decode_batch(bytes, limits) {
                    Ok(b) => b,
                    Err(e) => {
                        report.failed.push((id, format!("payload: {e}")));
                        continue;
                    }
                };
                match recovery.restore(id, &batch) {
                    Ok(o) if o.changed => report.replayed.push(id),
                    Ok(_) => report.already_applied_or_superseded.push(id),
                    Err(CommitError::Conflict(_)) => report.already_applied_or_superseded.push(id),
                    Err(e) => report.failed.push((id, format!("commit: {e:?}"))),
                }
            }
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use cube_core::AssetId;
    use cube_editing_api::{ActorId, JournalSeq};

    fn e(seq: u64, phase: JournalPhase, op: u64) -> JournalEntry {
        JournalEntry {
            seq: JournalSeq(seq),
            phase,
            operation: OperationId::new(1, op),
            actor: ActorId(1),
            asset: AssetId::new(1),
            post_tokens: vec![],
            delta: None,
            members: vec![],
            payload: vec![],
        }
    }

    #[test]
    fn closed_operations_need_nothing_and_pending_ones_are_decided_by_core() {
        let j = [
            e(1, JournalPhase::Intent, 1),
            e(2, JournalPhase::Confirmed, 1),
            e(3, JournalPhase::Intent, 2),
            e(4, JournalPhase::Aborted, 2),
            e(5, JournalPhase::Intent, 3),
            e(6, JournalPhase::Intent, 4),
        ];
        let applied: HashSet<_> = [OperationId::new(1, 1), OperationId::new(1, 3)]
            .into_iter()
            .collect();
        assert_eq!(
            reconcile(&j, &applied, true),
            vec![
                Resolution::Confirm(OperationId::new(1, 3)),
                Resolution::Abort(OperationId::new(1, 4)),
            ]
        );
        assert_eq!(
            reconcile(&j, &applied, false),
            vec![
                Resolution::Confirm(OperationId::new(1, 3)),
                Resolution::Undecidable(OperationId::new(1, 4)),
            ]
        );
    }

    #[test]
    fn an_id_retried_after_an_abort_is_pending_again_and_reconciling_is_idempotent() {
        let j = [
            e(1, JournalPhase::Intent, 9),
            e(2, JournalPhase::Aborted, 9),
            e(3, JournalPhase::Intent, 9),
        ];
        let none = HashSet::new();
        assert_eq!(
            reconcile(&j, &none, true),
            vec![Resolution::Abort(OperationId::new(1, 9))]
        );
        // After the resolution is written the intent is closed and nothing is left to do.
        let mut closed = j.to_vec();
        closed.push(e(4, JournalPhase::Aborted, 9));
        assert!(reconcile(&closed, &none, true).is_empty());
        assert!(reconcile(&[], &none, true).is_empty());
    }
}
