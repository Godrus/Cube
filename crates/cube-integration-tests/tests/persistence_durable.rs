//! PERS-02/03/05 (EXPERIMENTAL formats): manifest, snapshot store and crash-safe compaction,
//! tested together with the real Orchestration, the real voxel backend and the file journal.
//!
//! Evidence levels: the crash-point runs use the `FaultFs` model (weakest POSIX semantics);
//! the full cycle also runs on a real directory. Neither is a claim about any physical device
//! (DG-2), and the formats are not approved (GOV-10).
#![cfg(not(loom))]

use std::cell::RefCell;
use std::sync::{Arc, Mutex};

use cube_backend_voxel::{VoxelBackend, set_material};
use cube_core::{AssetView, LayerId, OperationId, Region, VoxelPos};
use cube_editing_api::{EditMode, EditingOperation, JournalEntry, OperationJournal, kinds};
use cube_integration_tests::fakes::*;
use cube_persistence::{
    DirStorage, FileJournal, JournalOpenError, Limits, Manifest, Storage, StoreError, WorldError,
    compact, create_world, load_manifest, open_world, publish_manifest, replay_confirmed,
};
use cube_testkit::faultfs::{FaultFs, FsError, explore};

#[derive(Clone)]
struct SharedFs(Arc<Mutex<FaultFs>>);

fn e(err: FsError) -> StoreError {
    StoreError::Io(format!("{err:?}"))
}

impl Storage for SharedFs {
    fn write(&mut self, n: &str, d: &[u8]) -> Result<(), StoreError> {
        self.0.lock().unwrap().write(n, d).map_err(e)
    }
    fn append(&mut self, n: &str, d: &[u8]) -> Result<(), StoreError> {
        self.0.lock().unwrap().append(n, d).map_err(e)
    }
    fn truncate(&mut self, n: &str, l: u64) -> Result<(), StoreError> {
        self.0.lock().unwrap().truncate(n, l as usize).map_err(e)
    }
    fn fsync(&mut self, n: &str) -> Result<(), StoreError> {
        self.0.lock().unwrap().fsync(n).map_err(e)
    }
    fn rename(&mut self, a: &str, b: &str) -> Result<(), StoreError> {
        self.0.lock().unwrap().rename(a, b).map_err(e)
    }
    fn remove(&mut self, n: &str) -> Result<(), StoreError> {
        self.0.lock().unwrap().remove(n).map_err(e)
    }
    fn fsync_dir(&mut self) -> Result<(), StoreError> {
        self.0.lock().unwrap().fsync_dir().map_err(e)
    }
    fn read(&self, n: &str) -> Result<Option<Vec<u8>>, StoreError> {
        Ok(self.0.lock().unwrap().read(n))
    }
    fn list(&self) -> Result<Vec<String>, StoreError> {
        Ok(self.0.lock().unwrap().list())
    }
}

fn fresh() -> SharedFs {
    SharedFs(Arc::new(Mutex::new(FaultFs::new())))
}

fn put(fs: &SharedFs, name: &str, bytes: &[u8]) {
    let mut g = fs.0.lock().unwrap();
    g.write(name, bytes).unwrap();
    g.fsync(name).unwrap();
    g.fsync_dir().unwrap();
}

fn brick_op(i: u64) -> EditingOperation {
    EditingOperation {
        id: OperationId::new(71, i),
        kind: kinds::SET_MATERIAL,
        asset: cube_core::AssetId::new(ASSET),
        layers: vec![LayerId::new(L_VOXEL)],
        parameters: set_material(i as u16, None),
        read_region: None,
        write_region: Region::new(
            0,
            VoxelPos::new(16 * i as i64, 0, 0),
            VoxelPos::new(16 * i as i64 + 16, 16, 16),
        )
        .unwrap(),
        conversion: None,
        mode: EditMode::Destructive,
    }
}

fn boot(view: Option<AssetView>, journal: Option<Arc<dyn OperationJournal>>) -> Fixture {
    fixture_with(Options {
        backends: vec![VoxelBackend::arc()],
        restore: view,
        journal_port: journal,
        with_payload: true,
        ..Options::default()
    })
}

fn hash(f: &Fixture) -> u128 {
    f.reader.snapshot().export_view().content_hash().value()
}

fn limits() -> Limits {
    Limits::default()
}

/// Runs operations 1..=n against a plain fixture (no persistence at all): the independent model.
fn model_hash(n: u64) -> u128 {
    let f = boot(None, None);
    for i in 1..=n {
        assert!(f.run(vec![brick_op(i)]).is_success());
    }
    hash(&f)
}

/// Journal records (with payload) of operations 1..=3 produced by the real chain, plus the
/// view after operations 1 and 2.
fn real_records() -> (Vec<JournalEntry>, AssetView) {
    let fs = fresh();
    let (j, _) = FileJournal::open(fs, "journal", limits()).unwrap();
    let j = Arc::new(j);
    let f = boot(None, Some(j.clone()));
    for i in 1..=2 {
        assert!(f.run(vec![brick_op(i)]).is_success());
    }
    let view12 = f.reader.snapshot().export_view();
    assert!(f.run(vec![brick_op(3)]).is_success());
    let entries = j.entries();
    assert_eq!(entries.len(), 6, "intent + confirmed per operation");
    (entries, view12)
}

#[test]
fn the_full_cycle_survives_a_kill_on_a_real_directory_and_matches_the_independent_model() {
    let dir = std::env::temp_dir().join(format!("cube-durable-{}-1", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let mut st = DirStorage::open(&dir).unwrap();
    create_world(&mut st, 7, &limits()).unwrap();

    // Life 1: operations 1 and 2 through the real chain into the file journal, then a
    // checkpoint: fold the journal into a verified snapshot (compaction).
    let w = open_world(DirStorage::open(&dir).unwrap(), &limits()).unwrap();
    assert!(w.snapshot.is_none());
    let j = Arc::new(w.journal);
    let f = boot(None, Some(j.clone()));
    assert!(f.run(vec![brick_op(1)]).is_success());
    assert!(f.run(vec![brick_op(2)]).is_success());
    let view = f.reader.snapshot().export_view();
    let upto = j.entries().last().unwrap().seq.0;
    drop((f, j));
    let rep = compact(&mut st, &view, upto, &limits()).unwrap();
    assert_eq!(
        (rep.generation, rep.snapshot_seq, rep.kept_records),
        (2, 4, 0)
    );
    assert_eq!(
        rep.removed.len(),
        2,
        "the old journal and its marks are gone"
    );

    // Life 2: restore from the snapshot, two more operations, then "kill".
    let w = open_world(DirStorage::open(&dir).unwrap(), &limits()).unwrap();
    let snap = w.snapshot.clone().expect("snapshot");
    assert_eq!(snap.content_hash(), view.content_hash());
    let j = Arc::new(w.journal);
    let f = boot(Some(snap), Some(j.clone()));
    assert!(f.run(vec![brick_op(3)]).is_success());
    assert!(f.run(vec![brick_op(4)]).is_success());
    assert_eq!(
        j.entries()[0].seq.0,
        5,
        "numbers continue after the snapshot"
    );
    let truth = hash(&f);
    drop((f, j));

    // Life 3: load the snapshot, replay the journal, compare with the independent model.
    let w = open_world(DirStorage::open(&dir).unwrap(), &limits()).unwrap();
    assert!(w.garbage_removed.is_empty());
    let g = boot(w.snapshot.clone(), Some(Arc::new(w.journal)));
    let j = open_world(DirStorage::open(&dir).unwrap(), &limits())
        .unwrap()
        .journal;
    let rep = replay_confirmed(&j.entries(), &g.recovery, &limits());
    assert_eq!(
        rep.replayed,
        vec![OperationId::new(71, 3), OperationId::new(71, 4)]
    );
    assert!(rep.failed.is_empty());
    assert_eq!(hash(&g), truth, "the exact pre-kill world");
    assert_eq!(hash(&g), model_hash(4), "and the independent model");
    // Replaying again changes nothing.
    assert!(
        replay_confirmed(&j.entries(), &g.recovery, &limits())
            .replayed
            .is_empty()
    );

    // Idempotent ids within the journal window; editing continues.
    let rec = g.svc.recover_idempotency().unwrap();
    assert_eq!(rec.recorded, 2);
    let (rev, commits) = (g.revision(), g.commits());
    assert!(g.run(vec![brick_op(4)]).is_success());
    assert_eq!((g.revision(), g.commits()), (rev, commits));
    assert!(g.run(vec![brick_op(5)]).is_success());
    assert_eq!(hash(&g), model_hash(5));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn every_crash_point_of_a_compaction_leaves_one_whole_generation_and_loses_no_acknowledged_record()
{
    let (entries, view12) = real_records();
    let expected: Vec<u128> = (0..=6u64).map(|n| model_hash(n / 2)).collect();
    let acked: RefCell<usize> = RefCell::new(0);
    let created: RefCell<bool> = RefCell::new(false);
    let scenario = |fs: &mut FaultFs| -> Result<(), FsError> {
        *acked.borrow_mut() = 0;
        *created.borrow_mut() = false;
        let shared = SharedFs(Arc::new(Mutex::new(std::mem::take(fs))));
        let run = || -> Result<(), ()> {
            let mut st = shared.clone();
            create_world(&mut st, 1, &limits()).map_err(|_| ())?;
            *created.borrow_mut() = true;
            {
                let w = open_world(shared.clone(), &limits()).map_err(|_| ())?;
                for en in &entries {
                    w.journal.append(en.clone()).map_err(|_| ())?;
                    *acked.borrow_mut() += 1;
                }
            }
            compact(&mut st, &view12, 4, &limits()).map_err(|_| ())?;
            Ok(())
        };
        let _ = run();
        *fs = std::mem::take(&mut *shared.0.lock().unwrap());
        Ok(())
    };
    let failures = explore(scenario, |fs, step| {
        let shared = SharedFs(Arc::new(Mutex::new(fs.clone())));
        let w = match open_world(shared.clone(), &limits()) {
            Ok(w) => w,
            Err(WorldError::NoWorld) if *acked.borrow() == 0 => return Ok(()),
            Err(err) => return Err(format!("step {step}: world does not open: {err}")),
        };
        let m = w.manifest.clone();
        if !(1..=2).contains(&m.generation) {
            return Err(format!("step {step}: generation {}", m.generation));
        }
        let snap_seq = m.snapshot.as_ref().map_or(0, |r| r.journal_seq);
        if (m.generation == 1) != m.snapshot.is_none() || (m.generation == 2 && snap_seq != 4) {
            return Err(format!("step {step}: mixed generation {m:?}"));
        }
        let total = snap_seq as usize + w.journal.entries().len();
        if total < *acked.borrow() {
            return Err(format!(
                "step {step}: {} acknowledged records, only {total} recovered",
                acked.borrow()
            ));
        }
        let g = boot(w.snapshot.clone(), None);
        let report = replay_confirmed(&w.journal.entries(), &g.recovery, &limits());
        if !report.failed.is_empty() {
            return Err(format!("step {step}: replay failures {:?}", report.failed));
        }
        if hash(&g) != expected[total] {
            return Err(format!(
                "step {step}: world differs from the model at {total}"
            ));
        }
        // Opening again changes nothing, and no unnamed file is left behind.
        drop(w);
        let again = open_world(shared.clone(), &limits())
            .map_err(|err| format!("step {step}: second open: {err}"))?;
        if again.manifest != m || !again.garbage_removed.is_empty() {
            return Err(format!("step {step}: second open differs"));
        }
        let names = shared.list().unwrap();
        let stray: Vec<_> = names
            .iter()
            .filter(|n| {
                (n.starts_with("snap-") || n.starts_with("journal-"))
                    && **n != m.journal
                    && **n != format!("{}.mark", m.journal)
                    && m.snapshot.as_ref().is_none_or(|r| r.file != **n)
            })
            .collect();
        if !stray.is_empty() {
            return Err(format!("step {step}: stray files {stray:?}"));
        }
        Ok(())
    });
    assert_eq!(
        failures,
        vec![],
        "compaction must be crash safe at every step"
    );
}

#[test]
fn damaged_missing_or_inconsistent_world_files_are_refused_never_guessed() {
    let (entries, view12) = real_records();
    let fs = fresh();
    let mut st = fs.clone();
    assert!(matches!(
        open_world(fs.clone(), &limits()).err(),
        Some(WorldError::NoWorld)
    ));
    create_world(&mut st, 1, &limits()).unwrap();
    assert!(matches!(
        create_world(&mut st, 1, &limits()),
        Err(WorldError::AlreadyExists)
    ));
    {
        let w = open_world(fs.clone(), &limits()).unwrap();
        for en in &entries {
            w.journal.append(en.clone()).unwrap();
        }
    }
    // Boundary checks: outside the journal, and in the middle of an operation.
    for bad in [0u64, 7] {
        assert!(matches!(
            compact(&mut st, &view12, bad, &limits()),
            Err(WorldError::BadBoundary)
        ));
    }
    assert!(matches!(
        compact(&mut st, &view12, 3, &limits()),
        Err(WorldError::PendingAtBoundary(_))
    ));
    compact(&mut st, &view12, 4, &limits()).unwrap();
    // A boundary that is not beyond the snapshot is refused.
    assert!(matches!(
        compact(&mut st, &view12, 4, &limits()),
        Err(WorldError::BadBoundary)
    ));
    let m = load_manifest(&st, &limits()).unwrap().unwrap();
    let snap_name = m.snapshot.clone().unwrap().file;
    let good_snap = st.read(&snap_name).unwrap().unwrap();
    let good_manifest = st.read("manifest").unwrap().unwrap();

    // Generation must grow by exactly one.
    let mut skip = m.clone();
    skip.generation += 5;
    assert!(publish_manifest(&mut st, &skip, &limits()).is_err());

    // A flipped byte in the snapshot, and a valid but different snapshot under the same name.
    let mut flipped = good_snap.clone();
    let mid = flipped.len() / 2;
    flipped[mid] ^= 0x01;
    put(&fs, &snap_name, &flipped);
    assert!(matches!(
        open_world(fs.clone(), &limits()).err(),
        Some(WorldError::Snapshot(_))
    ));
    put(
        &fs,
        &snap_name,
        &cube_persistence::encode_asset(&boot(None, None).reader.snapshot().export_view()),
    );
    assert!(matches!(
        open_world(fs.clone(), &limits()).err(),
        Some(WorldError::Snapshot(_))
    ));
    put(&fs, &snap_name, &good_snap);
    assert!(open_world(fs.clone(), &limits()).is_ok());

    // A damaged manifest is an error, not "no world"; a lost one is refused too.
    let mut bad_manifest = good_manifest.clone();
    bad_manifest[14] ^= 0xFF;
    put(&fs, "manifest", &bad_manifest);
    assert!(matches!(
        open_world(fs.clone(), &limits()).err(),
        Some(WorldError::Manifest(_))
    ));
    fs.0.lock().unwrap().remove("manifest").unwrap();
    fs.0.lock().unwrap().fsync_dir().unwrap();
    assert!(matches!(
        open_world(fs.clone(), &limits()).err(),
        Some(WorldError::ManifestLost)
    ));
    assert!(matches!(
        create_world(&mut st, 1, &limits()),
        Err(WorldError::ManifestLost)
    ));
    put(&fs, "manifest", &good_manifest);

    // A manifest whose journal start contradicts its snapshot.
    let inconsistent = Manifest {
        journal_first_seq: m.journal_first_seq + 3,
        ..m.clone()
    };
    put(
        &fs,
        "manifest",
        &cube_persistence::codec::encode_manifest(&inconsistent),
    );
    assert!(matches!(
        open_world(fs.clone(), &limits()).err(),
        Some(WorldError::Inconsistent)
    ));
    put(&fs, "manifest", &good_manifest);

    // A journal that lost an acknowledged record (journal rewritten empty, marks intact).
    let w = open_world(fs.clone(), &limits()).unwrap();
    assert_eq!(w.journal.entries().len(), 2);
    drop(w);
    let jname = m.journal.clone();
    put(&fs, &jname, &[]);
    assert!(matches!(
        open_world(fs.clone(), &limits()).err(),
        Some(WorldError::Journal(
            JournalOpenError::AcknowledgedLoss { .. }
        ))
    ));
}
