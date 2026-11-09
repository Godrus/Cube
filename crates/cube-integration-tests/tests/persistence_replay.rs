//! A journal with write payload: confirmed changes made after the last snapshot are replayed on
//! restart instead of being lost (closes the loss part of TD-4). Journal and files are contract
//! fakes and the crash-point model; no device durability is claimed (DG-2).
#![cfg(not(loom))]

use std::sync::Arc;

use cube_backend_voxel::{VoxelBackend, set_material};
use cube_core::{AssetView, LayerId, OperationId, Region, VoxelPos};
use cube_editing_api::{EditMode, EditingOperation, JournalPhase, kinds};
use cube_integration_tests::fakes::*;
use cube_persistence::{Limits, decode_journal_entry, encode_journal_entry, replay_confirmed};

fn brick_op(n: u64, brick_x: i64, m: u16) -> EditingOperation {
    EditingOperation {
        id: OperationId::new(51, n),
        kind: kinds::SET_MATERIAL,
        asset: cube_core::AssetId::new(ASSET),
        layers: vec![LayerId::new(L_VOXEL)],
        parameters: set_material(m, None),
        read_region: None,
        write_region: Region::new(
            0,
            VoxelPos::new(brick_x * 16, 0, 0),
            VoxelPos::new(brick_x * 16 + 16, 16, 16),
        )
        .unwrap(),
        conversion: None,
        mode: EditMode::Destructive,
    }
}

fn boot(view: Option<AssetView>, journal: &Arc<FaultyJournal>) -> Fixture {
    fixture_with(Options {
        backends: vec![VoxelBackend::arc()],
        restore: view,
        shared_journal: Some(Arc::clone(journal)),
        with_payload: true,
        ..Options::default()
    })
}

fn first_life() -> (Arc<FaultyJournal>, AssetView, AssetView) {
    let journal = Arc::new(FaultyJournal::default());
    let f = boot(None, &journal);
    assert!(f.run(vec![brick_op(1, 0, 5)]).is_success());
    let after1 = f.reader.snapshot().export_view();
    // Two more confirmed changes that the next snapshot never saw: a new brick and an overwrite.
    assert!(f.run(vec![brick_op(2, 1, 7)]).is_success());
    assert!(f.run(vec![brick_op(3, 0, 9)]).is_success());
    (journal, after1, f.reader.snapshot().export_view())
}

#[test]
fn confirmed_changes_after_the_last_snapshot_are_replayed_exactly_and_replay_is_idempotent() {
    let (journal, after1, full) = first_life();
    assert!(
        journal
            .entries()
            .iter()
            .any(|e| e.phase == JournalPhase::Intent && !e.payload.is_empty())
    );

    let g = boot(Some(after1), &journal);
    assert_eq!(
        g.material(bc(0, 0, 0)),
        Some(5),
        "the snapshot lacks operations 2 and 3"
    );
    let entries = journal.entries();
    let report = replay_confirmed(&entries, &g.recovery, &Limits::default());
    assert_eq!(
        report.replayed,
        vec![OperationId::new(51, 2), OperationId::new(51, 3)]
    );
    assert_eq!(
        report.already_applied_or_superseded,
        vec![OperationId::new(51, 1)]
    );
    assert!(
        report.failed.is_empty() && report.undecided.is_empty(),
        "{report:?}"
    );
    // The recovered world equals the world before the crash, content and revisions.
    assert_eq!(
        g.reader.snapshot().export_view().content_hash(),
        full.content_hash()
    );
    assert_eq!(
        (g.material(bc(0, 0, 0)), g.material(bc(1, 0, 0))),
        (Some(9), Some(7))
    );
    assert_eq!(g.revision(), 3);

    // Replaying again changes nothing.
    let commits = g.commits();
    let again = replay_confirmed(&entries, &g.recovery, &Limits::default());
    assert!(again.replayed.is_empty(), "{again:?}");
    assert_eq!((g.commits(), g.revision()), (commits, 3));

    // The idempotency table now finds every operation contained in the world: nothing is lost,
    // and a retry is answered without applying anything.
    let rec = g.svc.recover_idempotency().unwrap();
    assert_eq!(
        (
            rec.recorded,
            rec.lost_since_snapshot.len(),
            rec.blocked.len()
        ),
        (3, 0, 0)
    );
    assert!(g.run(vec![brick_op(2, 1, 7)]).is_success());
    assert!(g.run(vec![brick_op(3, 0, 9)]).is_success());
    assert_eq!((g.revision(), g.material(bc(0, 0, 0))), (3, Some(9)));
}

#[test]
fn an_unresolved_intent_is_not_replayed_and_a_damaged_or_missing_payload_is_reported_not_hidden() {
    let (journal, after1, _) = first_life();
    // A crash after the Intent of operation 4 reached the journal, before its Core commit.
    let mut entries = journal.entries();
    let intent4 = {
        let mut e = entries
            .iter()
            .find(|e| e.operation == OperationId::new(51, 3) && e.phase == JournalPhase::Intent)
            .unwrap()
            .clone();
        e.operation = OperationId::new(51, 4);
        e.members = vec![(OperationId::new(51, 4), 1)];
        e
    };
    entries.push(intent4);
    // Damage the payload of operation 2 (a flipped byte: the checksum catches it) and drop the
    // payload of operation 3.
    for e in entries.iter_mut() {
        if e.operation == OperationId::new(51, 2) && e.phase == JournalPhase::Intent {
            let n = e.payload.len() / 2;
            e.payload[n] ^= 0x40;
        }
        if e.operation == OperationId::new(51, 3) && e.phase == JournalPhase::Intent {
            e.payload.clear();
        }
    }
    let g = boot(Some(after1), &journal);
    let report = replay_confirmed(&entries, &g.recovery, &Limits::default());
    assert!(report.replayed.is_empty(), "{report:?}");
    assert_eq!(report.undecided, vec![OperationId::new(51, 4)]);
    let failed: Vec<_> = report.failed.iter().map(|(id, _)| *id).collect();
    assert_eq!(
        failed,
        vec![OperationId::new(51, 2), OperationId::new(51, 3)]
    );
    assert!(
        report.failed[0].1.contains("checksum") && report.failed[1].1.contains("no write payload")
    );
    // Nothing was applied from the bad records.
    assert_eq!(
        (g.material(bc(0, 0, 0)), g.material(bc(1, 0, 0))),
        (Some(5), None)
    );
}

#[test]
fn journal_records_with_payload_survive_the_codec_and_the_replay_of_decoded_records_works() {
    let (journal, after1, full) = first_life();
    let decoded: Vec<_> = journal
        .entries()
        .iter()
        .map(|e| decode_journal_entry(&encode_journal_entry(e), &Limits::default()).unwrap())
        .collect();
    assert_eq!(decoded, journal.entries());
    let g = boot(Some(after1), &journal);
    let report = replay_confirmed(&decoded, &g.recovery, &Limits::default());
    assert_eq!(report.replayed.len(), 2);
    assert_eq!(
        g.reader.snapshot().export_view().content_hash(),
        full.content_hash()
    );
}
