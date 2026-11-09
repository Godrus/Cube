//! L-COMP-1: a retry of an operation that compaction folded into a snapshot must never be
//! applied again, across restarts and repeated compactions, with several actors, a bounded
//! retry table and a real directory. Requirement RS-1 (per-scope strictly increasing sequences
//! from the authoritative side) is what makes the persisted floors sound; ids that break it are
//! refused, and a test shows that.
#![cfg(not(loom))]

use std::sync::Arc;

use cube_backend_voxel::{VoxelBackend, set_material};
use cube_core::{AssetView, LayerId, OperationId, Region, VoxelPos};
use cube_editing_api::{
    EditMode, EditingError, EditingOperation, EditingResult, OperationJournal, kinds,
};
use cube_integration_tests::fakes::*;
use cube_persistence::{DirStorage, Limits, compact, create_world, open_world};

fn limits() -> Limits {
    Limits::default()
}

/// Operation `seq` of `scope`: writes material `seq` into its own brick (scopes never overlap).
fn op(scope: u64, seq: u64, material: u16) -> EditingOperation {
    let x = (scope as i64 - 70) * 1000 + seq as i64;
    EditingOperation {
        id: OperationId::new(scope, seq),
        kind: kinds::SET_MATERIAL,
        asset: cube_core::AssetId::new(ASSET),
        layers: vec![LayerId::new(L_VOXEL)],
        parameters: set_material(material, None),
        read_region: None,
        write_region: Region::new(
            0,
            VoxelPos::new(16 * x, 0, 0),
            VoxelPos::new(16 * x + 16, 16, 16),
        )
        .unwrap(),
        conversion: None,
        mode: EditMode::Destructive,
    }
}

fn boot(
    view: Option<AssetView>,
    journal: Option<Arc<dyn OperationJournal>>,
    capacity: usize,
) -> Fixture {
    fixture_with(Options {
        backends: vec![VoxelBackend::arc()],
        restore: view,
        journal_port: journal,
        with_payload: true,
        retry_capacity: capacity,
        ..Options::default()
    })
}

fn hash(f: &Fixture) -> u128 {
    f.reader.snapshot().export_view().content_hash().value()
}

fn refused(r: &EditingResult) -> bool {
    matches!(
        r,
        EditingResult::ValidationFailure(EditingError::InvalidOperation(_))
    )
}

/// Opens the world, restores the retry floors, rebuilds the retry table.
fn start(dir: &std::path::Path, capacity: usize) -> (Fixture, Arc<dyn OperationJournal>, u64) {
    let w = open_world(DirStorage::open(dir).unwrap(), &limits()).unwrap();
    let floors = w.manifest.retry_floors.clone();
    let gen_ = w.manifest.generation;
    let j: Arc<dyn OperationJournal> = Arc::new(w.journal);
    let f = boot(w.snapshot.clone(), Some(j.clone()), capacity);
    let rep = cube_persistence::replay_confirmed(
        &j.read_from(cube_editing_api::JournalSeq(0), usize::MAX)
            .unwrap(),
        &f.recovery,
        &limits(),
    );
    assert!(rep.failed.is_empty());
    f.svc.restore_retry_floors(&floors);
    f.svc.recover_idempotency().unwrap();
    (f, j, gen_)
}

/// Checkpoint: stop the live system, fold the journal into a snapshot.
fn checkpoint(dir: &std::path::Path, f: Fixture, j: Arc<dyn OperationJournal>) -> u64 {
    let view = f.reader.snapshot().export_view();
    let upto = j
        .read_from(cube_editing_api::JournalSeq(0), usize::MAX)
        .unwrap()
        .last()
        .unwrap()
        .seq
        .0;
    drop((f, j));
    let mut st = DirStorage::open(dir).unwrap();
    compact(&mut st, &view, upto, &limits()).unwrap().generation
}

#[test]
fn folded_ids_are_never_applied_again_across_restarts_two_compactions_and_two_actors() {
    let dir = std::env::temp_dir().join(format!("cube-retry-{}-1", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let mut st = DirStorage::open(&dir).unwrap();
    create_world(&mut st, 1, &limits()).unwrap();

    // Life 1: two actors edit concurrently, then compaction 1.
    let (f, j, _) = start(&dir, 64);
    std::thread::scope(|s| {
        for scope in [71u64, 72] {
            let f = &f;
            s.spawn(move || {
                for seq in 1..=3 {
                    assert!(
                        f.run(vec![op(scope, seq, (scope * 10 + seq) as u16)])
                            .is_success()
                    );
                }
            });
        }
    });
    assert_eq!(checkpoint(&dir, f, j), 2);
    let w = open_world(DirStorage::open(&dir).unwrap(), &limits()).unwrap();
    assert_eq!(w.manifest.retry_floors, vec![(71, 3), (72, 3)]);
    drop(w);

    // Life 2: after the restart every old retry is refused and changes nothing.
    let (f, j, generation) = start(&dir, 64);
    assert_eq!(generation, 2);
    let (rev, commits, h0) = (f.revision(), f.commits(), hash(&f));
    for scope in [71u64, 72] {
        for seq in 1..=3 {
            let same = f.run(vec![op(scope, seq, (scope * 10 + seq) as u16)]);
            assert!(refused(&same), "scope {scope} seq {seq}: {same:?}");
            // The same id with another payload is refused as well.
            let other = f.run(vec![op(scope, seq, 999)]);
            assert!(
                refused(&other),
                "changed payload must be refused: {other:?}"
            );
        }
    }
    assert_eq!((f.revision(), f.commits(), hash(&f)), (rev, commits, h0));
    // An id below the floor that was never used is refused too (RS-1: fail closed).
    assert!(refused(&f.run(vec![op(71, 0, 5)])));
    // New work with higher sequences goes through, also for a scope seen for the first time.
    assert!(f.run(vec![op(71, 4, 41)]).is_success());
    assert!(f.run(vec![op(72, 4, 42)]).is_success());
    assert!(f.run(vec![op(73, 1, 43)]).is_success());
    // And the journal window still answers a retry of a recent id without applying it.
    let (rev, commits) = (f.revision(), f.commits());
    assert!(f.run(vec![op(71, 4, 41)]).is_success());
    assert_eq!((f.revision(), f.commits()), (rev, commits));

    // Compaction 2, restart again.
    assert_eq!(checkpoint(&dir, f, j), 3);
    let (f, _j, generation) = start(&dir, 64);
    assert_eq!(generation, 3);
    let (rev, commits, h1) = (f.revision(), f.commits(), hash(&f));
    for (scope, seq, m) in [(71u64, 1u64, 71u16), (71, 4, 41), (72, 4, 42), (73, 1, 43)] {
        assert!(
            refused(&f.run(vec![op(scope, seq, m)])),
            "scope {scope} seq {seq} after the second compaction"
        );
    }
    assert_eq!((f.revision(), f.commits(), hash(&f)), (rev, commits, h1));

    // The independent model: the same successful operations on a plain fixture.
    let model = boot(None, None, 64);
    for scope in [71u64, 72] {
        for seq in 1..=3 {
            assert!(
                model
                    .run(vec![op(scope, seq, (scope * 10 + seq) as u16)])
                    .is_success()
            );
        }
    }
    for (scope, seq, m) in [(71u64, 4u64, 41u16), (72, 4, 42), (73, 1, 43)] {
        assert!(model.run(vec![op(scope, seq, m)]).is_success());
    }
    assert_eq!(h1, hash(&model));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_small_retry_table_evicts_safely_and_the_floors_survive_a_restart_that_is_smaller_still() {
    let dir = std::env::temp_dir().join(format!("cube-retry-{}-2", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let mut st = DirStorage::open(&dir).unwrap();
    create_world(&mut st, 1, &limits()).unwrap();
    let (f, j, _) = start(&dir, 2);
    for seq in 1..=5 {
        assert!(f.run(vec![op(71, seq, seq as u16)]).is_success());
    }
    // Capacity 2: ids 1..=3 were evicted; they are refused, not re-applied.
    let (rev, commits) = (f.revision(), f.commits());
    for seq in 1..=3 {
        assert!(refused(&f.run(vec![op(71, seq, seq as u16)])), "seq {seq}");
    }
    // 4 and 5 are still answered.
    assert!(f.run(vec![op(71, 5, 5)]).is_success());
    assert_eq!((f.revision(), f.commits()), (rev, commits));
    checkpoint(&dir, f, j);
    // Restart with an even smaller table: the journal window is empty (all folded), the floor
    // covers every id; nothing can be applied twice.
    let (f, _j, _) = start(&dir, 1);
    let (rev, commits) = (f.revision(), f.commits());
    for seq in 1..=5 {
        assert!(refused(&f.run(vec![op(71, seq, seq as u16)])), "seq {seq}");
    }
    assert_eq!((f.revision(), f.commits()), (rev, commits));
    assert!(f.run(vec![op(71, 6, 6)]).is_success());
    let _ = std::fs::remove_dir_all(&dir);
}
