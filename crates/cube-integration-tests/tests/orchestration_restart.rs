//! TD-4: idempotency across a restart, rebuilt from the durable journal. A "restart" here means:
//! the asset is restored from a saved snapshot, the journal survives, the in-memory retry table
//! is gone. The journal is a contract fake (no file durability claimed); real-file and
//! crash-point evidence for the files themselves is in `persistence_world.rs`.
//!
//! The observable for "applied again" is a sentinel: operation 1 sets material 5, operation 2
//! later sets material 9 on the same brick; re-applying operation 1 would bring 5 back.
#![cfg(not(loom))]

use std::sync::Arc;
use std::sync::atomic::Ordering;

use cube_backend_voxel::{VoxelBackend, set_material};
use cube_core::{AssetView, LayerId, OperationId, Region, VoxelPos};
use cube_editing_api::{
    ActorId, EditMode, EditingError, EditingOperation, EditingResult, JournalEntry, JournalPhase,
    JournalSeq, OperationJournal, kinds,
};
use cube_integration_tests::fakes::*;

fn brick_op(n: u64, m: u16) -> EditingOperation {
    EditingOperation {
        id: OperationId::new(41, n),
        kind: kinds::SET_MATERIAL,
        asset: cube_core::AssetId::new(ASSET),
        layers: vec![LayerId::new(L_VOXEL)],
        parameters: set_material(m, None),
        read_region: None,
        write_region: Region::new(0, VoxelPos::new(0, 0, 0), VoxelPos::new(16, 16, 16)).unwrap(),
        conversion: None,
        mode: EditMode::Destructive,
    }
}

fn boot(view: Option<AssetView>, journal: &Arc<FaultyJournal>, cap: usize) -> Fixture {
    fixture_with(Options {
        backends: vec![VoxelBackend::arc()],
        restore: view,
        shared_journal: Some(Arc::clone(journal)),
        retry_capacity: cap,
        ..Options::default()
    })
}

fn snapshot(f: &Fixture) -> AssetView {
    f.reader.snapshot().export_view()
}

/// First run: ops 1 and 2 on one brick; returns the journal and the snapshot after op 1 / op 2.
fn first_life() -> (Arc<FaultyJournal>, AssetView, AssetView) {
    let journal = Arc::new(FaultyJournal::default());
    let f = boot(None, &journal, 1024);
    assert!(f.run(vec![brick_op(1, 5)]).is_success());
    let after1 = snapshot(&f);
    assert!(f.run(vec![brick_op(2, 9)]).is_success());
    let after2 = snapshot(&f);
    (journal, after1, after2)
}

#[test]
fn a_confirmed_operation_is_not_applied_again_after_a_restart() {
    let (journal, _, after2) = first_life();
    let f = boot(Some(after2), &journal, 1024);
    let report = f.svc.recover_idempotency().unwrap();
    assert_eq!(report.recorded, 2);
    assert!(report.lost_since_snapshot.is_empty() && report.blocked.is_empty());
    assert_eq!(f.material(bc(0, 0, 0)), Some(9));
    let (rev, commits) = (f.revision(), f.commits());

    // The client lost the answer to operation 1 before the crash and retries it now.
    let r = f.run(vec![brick_op(1, 5)]);
    let EditingResult::Success(s) = r else {
        panic!("{r:?}")
    };
    assert_eq!(s.operation, OperationId::new(41, 1));
    assert_eq!(
        (f.material(bc(0, 0, 0)), f.revision(), f.commits()),
        (Some(9), rev, commits),
        "nothing was applied again"
    );
    // The same id with other content is refused.
    let r = f.run(vec![brick_op(1, 6)]);
    assert!(
        matches!(r, EditingResult::ValidationFailure(EditingError::InvalidOperation(m)) if m.contains("different content")),
        "{r:?}"
    );
    // New work continues normally and is journalled after the recovered records.
    assert!(f.run(vec![brick_op(3, 7)]).is_success());
    assert_eq!(f.material(bc(0, 0, 0)), Some(7));
    let seqs: Vec<u64> = journal.entries().iter().map(|e| e.seq.0).collect();
    assert_eq!(seqs, (1..=seqs.len() as u64).collect::<Vec<_>>());
}

#[test]
fn changes_lost_since_the_snapshot_are_reported_and_can_be_applied_again_not_falsely_confirmed() {
    let (journal, after1, _) = first_life();
    // Crash after op 2 was confirmed but before the next snapshot: the world only has op 1.
    let f = boot(Some(after1), &journal, 1024);
    let report = f.svc.recover_idempotency().unwrap();
    assert_eq!(report.recorded, 1);
    assert_eq!(report.lost_since_snapshot, vec![OperationId::new(41, 2)]);
    assert_eq!(
        f.material(bc(0, 0, 0)),
        Some(5),
        "the world does not contain op 2"
    );
    // Retrying op 1 is answered from the journal; retrying op 2 applies it, because claiming
    // success for a change the world does not contain would be a lie.
    let rev = f.revision();
    assert!(f.run(vec![brick_op(1, 5)]).is_success());
    assert_eq!(f.revision(), rev);
    assert!(f.run(vec![brick_op(2, 9)]).is_success());
    assert_eq!(f.material(bc(0, 0, 0)), Some(9));
}

fn intent_only(journal: &FaultyJournal, op: &EditingOperation) {
    journal
        .append(JournalEntry {
            seq: JournalSeq(0),
            phase: JournalPhase::Intent,
            operation: op.id,
            actor: ActorId(42),
            asset: cube_core::AssetId::new(ASSET),
            post_tokens: vec![],
            delta: None,
            members: vec![(op.id, op.fingerprint())],
            payload: vec![],
        })
        .unwrap();
}

#[test]
fn an_intent_without_a_later_record_is_undecided_and_never_silently_aborted_or_applied() {
    let (journal, _, after2) = first_life();
    // A crash between the Intent of operation 3 and its Core commit.
    let op3 = brick_op(3, 7);
    intent_only(&journal, &op3);
    let f = boot(Some(after2), &journal, 1024);
    let report = f.svc.recover_idempotency().unwrap();
    assert_eq!(report.blocked, vec![op3.id]);
    // Retrying it is refused with an explicit reason; the world is untouched.
    let before = (f.revision(), f.commits());
    let r = f.run(vec![op3.clone()]);
    assert!(
        matches!(r, EditingResult::ValidationFailure(EditingError::InvalidOperation(m)) if m.contains("undecided")),
        "{r:?}"
    );
    assert_eq!((f.revision(), f.commits()), before);
    // Other operations are not held up.
    assert!(f.run(vec![brick_op(4, 8)]).is_success());
    // Decided as "never applied": the id is free again and the retry applies it.
    f.svc.resolve_pending(op3.id, false).unwrap();
    assert!(f.run(vec![op3.clone()]).is_success());
    assert_eq!(f.material(bc(0, 0, 0)), Some(7));
    assert_eq!(journal.phases().last(), Some(&JournalPhase::Confirmed));
    assert_eq!(
        f.svc.resolve_pending(op3.id, false),
        Err(EditingError::NotFound)
    );
}

#[test]
fn a_crash_between_the_core_commit_and_the_confirmation_is_resolved_as_applied_without_reapplying()
{
    let journal = Arc::new(FaultyJournal::default());
    let f = boot(None, &journal, 1024);
    assert!(f.run(vec![brick_op(1, 5)]).is_success());
    journal.fail_confirm.store(true, Ordering::SeqCst);
    let r = f.run(vec![brick_op(2, 9)]);
    assert!(matches!(r, EditingResult::AppliedUnconfirmed(_)));
    // The snapshot is taken afterwards, so it contains operation 2; the Confirmed record never
    // reached the journal. Then the process dies.
    let saved = snapshot(&f);
    journal.fail_confirm.store(false, Ordering::SeqCst);
    assert_eq!(
        journal.phases(),
        vec![
            JournalPhase::Intent,
            JournalPhase::Confirmed,
            JournalPhase::Intent
        ]
    );

    let g = boot(Some(saved), &journal, 1024);
    let report = g.svc.recover_idempotency().unwrap();
    assert_eq!(report.blocked, vec![OperationId::new(41, 2)]);
    // The procedure that decides: compare with the restored world (it holds material 9), then
    // resolve. Persistence owns that comparison; here it is the test.
    assert_eq!(g.material(bc(0, 0, 0)), Some(9));
    g.svc
        .resolve_pending(OperationId::new(41, 2), true)
        .unwrap();
    assert!(g.run(vec![brick_op(3, 7)]).is_success());
    let rev = g.revision();
    // Retrying operation 2 now returns the recorded success and changes nothing.
    assert!(g.run(vec![brick_op(2, 9)]).is_success());
    assert_eq!((g.revision(), g.material(bc(0, 0, 0))), (rev, Some(7)));
}

#[test]
fn the_retry_table_is_bounded_and_forgotten_ids_are_refused_not_applied_again() {
    let journal = Arc::new(FaultyJournal::default());
    let f = boot(None, &journal, 3);
    for n in 1..=6 {
        assert!(f.run(vec![brick_op(n, n as u16)]).is_success());
    }
    assert!(f.svc.remembered() <= 3);
    let (rev, mat) = (f.revision(), f.material(bc(0, 0, 0)));
    // Ids that fell out of the window are refused: the table cannot know they were applied.
    for n in 1..=3 {
        let r = f.run(vec![brick_op(n, n as u16)]);
        assert!(
            matches!(r, EditingResult::ValidationFailure(EditingError::InvalidOperation(m)) if m.contains("retry window")),
            "op {n}: {r:?}"
        );
    }
    assert_eq!((f.revision(), f.material(bc(0, 0, 0))), (rev, mat));
    // Ids still in the window are answered, new ids work.
    assert!(f.run(vec![brick_op(6, 6)]).is_success());
    assert_eq!(f.revision(), rev);
    assert!(f.run(vec![brick_op(7, 7)]).is_success());
    // A restart with a small window keeps the same safety.
    let saved = snapshot(&f);
    let g = boot(Some(saved), &journal, 2);
    let report = g.svc.recover_idempotency().unwrap();
    assert_eq!(report.recorded, 7);
    assert!(g.svc.remembered() <= 2);
    let r = g.run(vec![brick_op(1, 1)]);
    assert!(matches!(r, EditingResult::ValidationFailure(_)), "{r:?}");
}

#[test]
fn recovery_without_a_journal_is_an_error_and_the_fingerprint_is_stable() {
    let f = fixture();
    assert!(f.svc.recover_idempotency().is_err());
    let op = brick_op(1, 5);
    assert_eq!(op.fingerprint(), op.clone().fingerprint());
    let mut other = op.clone();
    other.parameters.bytes[0] ^= 1;
    assert_ne!(op.fingerprint(), other.fingerprint());
    // Pinned: a change here means journals written by older builds stop matching their retries.
    assert_eq!(op.fingerprint(), PINNED_FINGERPRINT);
}

const PINNED_FINGERPRINT: u64 = 12_685_795_464_807_257_649;
