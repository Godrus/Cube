//! PERS-04: the file journal, and the recovery guarantees built on it, against the real Core,
//! the real Orchestration and (a) the crash-point model, (b) real files, (c) a real child
//! process killed mid-write. Model results are not claims about any physical drive or about
//! Android/iOS (DG-2).
#![cfg(not(loom))]

use std::cell::RefCell;
use std::io::BufRead;
use std::sync::{Arc, Mutex};

use cube_backend_voxel::{VoxelBackend, set_material};
use cube_core::{AssetView, LayerId, OperationId, Region, VoxelPos};
use cube_editing_api::{
    ActorId, EditMode, EditingError, EditingOperation, EditingResult, JournalEntry, JournalPhase,
    JournalSeq, OperationJournal, kinds,
};
use cube_integration_tests::fakes::*;
use cube_persistence::{
    DirStorage, FileJournal, JournalOpenError, Limits, Storage, StoreError, load_asset,
    replay_confirmed, save_asset,
};
use cube_testkit::faultfs::{FaultFs, FsError, explore};

/// The model filesystem shared between a journal and the test that crashes it.
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

fn entry(op: u64, phase: JournalPhase, payload: Vec<u8>) -> JournalEntry {
    JournalEntry {
        seq: JournalSeq(0),
        phase,
        operation: OperationId::new(61, op),
        actor: ActorId(1),
        asset: cube_core::AssetId::new(ASSET),
        post_tokens: vec![],
        delta: None,
        members: vec![(OperationId::new(61, op), op)],
        payload,
    }
}

fn open(fs: &SharedFs) -> (FileJournal<SharedFs>, cube_persistence::OpenReport) {
    FileJournal::open(fs.clone(), "journal", Limits::default()).expect("journal opens")
}

fn fresh() -> SharedFs {
    SharedFs(Arc::new(Mutex::new(FaultFs::new())))
}

#[test]
fn records_round_trip_with_a_gapless_sequence_and_survive_reopening() {
    let fs = fresh();
    let (j, rep) = open(&fs);
    assert_eq!((rep.records, rep.truncated_bytes), (0, 0));
    for n in 1..=5 {
        assert_eq!(
            j.append(entry(n, JournalPhase::Intent, vec![n as u8; 40]))
                .unwrap(),
            JournalSeq(n)
        );
    }
    drop(j);
    let (j2, rep) = open(&fs);
    assert_eq!(rep.records, 5);
    let all = j2.read_from(JournalSeq(1), 100).unwrap();
    assert_eq!(
        all.iter().map(|x| x.seq.0).collect::<Vec<_>>(),
        vec![1, 2, 3, 4, 5]
    );
    assert_eq!(j2.read_from(JournalSeq(4), 1).unwrap().len(), 1);
    assert_eq!(all[2].payload, vec![3u8; 40]);
    // Appending continues the sequence.
    assert_eq!(
        j2.append(entry(6, JournalPhase::Confirmed, vec![]))
            .unwrap(),
        JournalSeq(6)
    );
}

#[test]
fn a_torn_tail_is_repaired_and_damage_in_the_middle_is_refused() {
    let fs = fresh();
    let (j, _) = open(&fs);
    for n in 1..=4 {
        j.append(entry(n, JournalPhase::Intent, vec![9; 30]))
            .unwrap();
    }
    drop(j);
    let good = fs.0.lock().unwrap().read("journal").unwrap();
    // (a) a crash during the fifth write: a prefix of a record is on disk.
    let half = {
        let probe = fresh();
        let (p, _) = open(&probe);
        p.append(entry(5, JournalPhase::Intent, vec![9; 30]))
            .unwrap();
        let b = probe.0.lock().unwrap().read("journal").unwrap();
        b[..b.len() / 2].to_vec()
    };
    fs.0.lock().unwrap().append("journal", &half).unwrap();
    let (j, rep) = open(&fs);
    assert_eq!((rep.records, rep.truncated_bytes), (4, half.len()));
    assert_eq!(
        fs.0.lock().unwrap().read("journal").unwrap(),
        good,
        "the tail was cut off"
    );
    assert_eq!(
        j.append(entry(5, JournalPhase::Intent, vec![])).unwrap(),
        JournalSeq(5)
    );
    drop(j);

    // (b) a flipped byte inside record 2 while records 3..5 are intact: not a torn tail.
    let mut bad = fs.0.lock().unwrap().read("journal").unwrap();
    bad[good.len() / 4 + 20] ^= 0x01;
    let broken = fresh();
    broken.0.lock().unwrap().write("journal", &bad).unwrap();
    let r = FileJournal::open(broken.clone(), "journal", Limits::default());
    assert!(
        matches!(r, Err(JournalOpenError::CorruptInMiddle { .. })),
        "{:?}",
        r.err()
    );
    assert_eq!(
        broken.0.lock().unwrap().read("journal").unwrap(),
        bad,
        "nothing was cut"
    );

    // (c) a gap in the sequence is refused, an unknown schema at the start is refused.
    let skipped = fresh();
    {
        let (j, _) = open(&skipped);
        for n in 1..=3 {
            j.append(entry(n, JournalPhase::Intent, vec![])).unwrap();
        }
    }
    let all = skipped.0.lock().unwrap().read("journal").unwrap();
    let third = all.len() * 2 / 3;
    let gap = fresh();
    gap.0
        .lock()
        .unwrap()
        .write("journal", &[&all[..all.len() / 3], &all[third..]].concat())
        .unwrap();
    let r = FileJournal::open(gap, "journal", Limits::default());
    assert!(
        matches!(
            r,
            Err(JournalOpenError::Sequence {
                expected: 2,
                found: 3
            })
        ),
        "{:?}",
        r.err()
    );
    let mut foreign = all.clone();
    foreign[4] = 0x02;
    let f = fresh();
    f.0.lock().unwrap().write("journal", &foreign).unwrap();
    assert!(matches!(
        FileJournal::open(f, "journal", Limits::default()),
        Err(JournalOpenError::UnsupportedSchema(_))
    ));
}

/// Replaces a file's durable content (a damaged disk), for negative tests.
fn put(fs: &SharedFs, name: &str, bytes: &[u8]) {
    let mut g = fs.0.lock().unwrap();
    g.write(name, bytes).unwrap();
    g.fsync(name).unwrap();
    g.fsync_dir().unwrap();
}

fn read(fs: &SharedFs, name: &str) -> Vec<u8> {
    fs.0.lock().unwrap().read(name).unwrap_or_default()
}

fn three() -> (SharedFs, Vec<usize>) {
    let fs = fresh();
    let (j, _) = open(&fs);
    let mut ends = vec![];
    for n in 1..=3 {
        j.append(entry(n, JournalPhase::Intent, vec![7; 25]))
            .unwrap();
        ends.push(read(&fs, "journal").len());
    }
    (fs, ends)
}

#[test]
fn a_damaged_last_acknowledged_record_is_never_mistaken_for_a_torn_tail() {
    let (fs, ends) = three();
    let good = read(&fs, "journal");
    // Corrupt one byte inside record 3 (acknowledged: it has a mark).
    let mut bad = good.clone();
    bad[ends[1] + 12] ^= 0xFF;
    put(&fs, "journal", &bad);
    let r = FileJournal::open(fs.clone(), "journal", Limits::default());
    assert!(
        matches!(
            r.err(),
            Some(JournalOpenError::AcknowledgedLoss {
                acknowledged: 3,
                readable: 2
            })
        ),
        "fail closed"
    );
    // The damaged bytes are untouched (nothing was truncated).
    assert_eq!(read(&fs, "journal"), bad);
    // Cut short in the middle of the acknowledged record, or missing altogether: also refused.
    for cut in [ends[2] - 1, ends[1] + 5, ends[1]] {
        put(&fs, "journal", &good[..cut]);
        assert!(matches!(
            FileJournal::open(fs.clone(), "journal", Limits::default()).err(),
            Some(JournalOpenError::AcknowledgedLoss {
                acknowledged: 3,
                ..
            })
        ));
        assert_eq!(read(&fs, "journal").len(), cut);
    }
}

#[test]
fn a_damaged_or_torn_unacknowledged_record_is_dropped_and_the_acknowledged_ones_stay() {
    let (fs, ends) = three();
    let good = read(&fs, "journal");
    let marks = read(&fs, "journal.mark");
    // The fourth record reached the disk damaged, its mark never did.
    let probe = fresh();
    let (p, _) = open(&probe);
    for n in 1..=4 {
        p.append(entry(n, JournalPhase::Intent, vec![7; 25]))
            .unwrap();
    }
    let with4 = read(&probe, "journal");
    for bad4 in [
        with4[..with4.len() - 3].to_vec(),
        {
            let mut v = with4.clone();
            let n = v.len();
            v[n - 9] ^= 0x55;
            v
        },
        {
            let mut v = good.clone();
            v.extend_from_slice(&[0; 20]);
            v
        },
    ] {
        put(&fs, "journal", &bad4);
        put(&fs, "journal.mark", &marks);
        let (j, rep) = open(&fs);
        assert_eq!(rep.records, 3);
        assert_eq!(rep.acknowledged, 3);
        assert_eq!(read(&fs, "journal").len(), ends[2]);
        assert_eq!(
            j.append(entry(9, JournalPhase::Intent, vec![])).unwrap().0,
            4
        );
    }
    // A whole fourth record without a mark is kept (it was written, just not acknowledged).
    put(&fs, "journal", &with4);
    put(&fs, "journal.mark", &marks);
    let (_, rep) = open(&fs);
    assert_eq!(rep.records, 4);
}

#[test]
fn damage_in_the_middle_lost_marks_and_a_foreign_schema_are_refused_not_hidden() {
    let (fs, ends) = three();
    let good = read(&fs, "journal");
    let marks = read(&fs, "journal.mark");
    // Mid-log checksum error with valid records after it.
    let mut bad = good.clone();
    bad[ends[0] + 12] ^= 0xFF;
    put(&fs, "journal", &bad);
    assert!(matches!(
        FileJournal::open(fs.clone(), "journal", Limits::default()).err(),
        Some(JournalOpenError::CorruptInMiddle { .. })
    ));
    put(&fs, "journal", &good);
    // Marks lost: three unmarked records are refused; explicit adoption marks them.
    put(&fs, "journal.mark", &[]);
    assert!(matches!(
        FileJournal::open(fs.clone(), "journal", Limits::default()).err(),
        Some(JournalOpenError::UnmarkedRecords { unmarked: 3 })
    ));
    let (_, rep) =
        FileJournal::open_adopting_unmarked(fs.clone(), "journal", 1, Limits::default()).unwrap();
    assert_eq!((rep.records, rep.acknowledged), (3, 3));
    assert_eq!(read(&fs, "journal.mark").len(), 3 * 16);
    // Marks that go backwards.
    let mut back = marks.clone();
    let (a, b) = (back[..16].to_vec(), back[16..32].to_vec());
    back[..16].copy_from_slice(&b);
    back[16..32].copy_from_slice(&a);
    put(&fs, "journal.mark", &back);
    assert!(matches!(
        FileJournal::open(fs.clone(), "journal", Limits::default()).err(),
        Some(JournalOpenError::BadMarks)
    ));
    // A mark pointing beyond an empty journal: everything acknowledged is gone.
    put(&fs, "journal.mark", &marks);
    put(&fs, "journal", &[]);
    assert!(matches!(
        FileJournal::open(fs.clone(), "journal", Limits::default()).err(),
        Some(JournalOpenError::AcknowledgedLoss {
            acknowledged: 3,
            readable: 0
        })
    ));
    // A foreign schema in the first record is refused, never read as a torn tail.
    let mut foreign = good.clone();
    foreign[4] = 0x7F;
    foreign[5] = 0x7F;
    put(&fs, "journal", &foreign);
    put(&fs, "journal.mark", &marks);
    assert!(FileJournal::open(fs.clone(), "journal", Limits::default()).is_err());
}

#[test]
fn a_failed_write_or_fsync_poisons_the_journal_and_nothing_is_acknowledged() {
    // Fail the n-th storage call of the first append.
    for fail_at in 1..=6 {
        let fs = fresh();
        let counted = Counting {
            inner: fs.clone(),
            calls: 0,
            fail_at,
        };
        let (j, _) = FileJournal::open(counted, "journal", Limits::default()).unwrap();
        let r = j.append(entry(1, JournalPhase::Intent, vec![1; 20]));
        assert_eq!(r, Err(EditingError::PersistenceFailure), "call {fail_at}");
        assert!(j.is_poisoned());
        // No later append may claim success, whatever the disk now holds.
        assert!(j.append(entry(2, JournalPhase::Intent, vec![])).is_err());
        assert!(j.read_from(JournalSeq(1), 10).unwrap().is_empty());
        // After a restart the file is repaired to a consistent prefix: record 1 is either whole
        // (the failure came after the write) or absent, never half.
        let (j2, rep) = FileJournal::open(fs.clone(), "journal", Limits::default()).unwrap();
        assert!(rep.records <= 1);
        let seq = j2.append(entry(3, JournalPhase::Intent, vec![])).unwrap();
        assert_eq!(seq.0, rep.records as u64 + 1);
    }
    // Oversize records are refused outright.
    let (j, _) = FileJournal::open(fresh(), "journal", Limits { max_body: 64 }).unwrap();
    assert!(
        j.append(entry(1, JournalPhase::Intent, vec![0; 1000]))
            .is_err()
    );
}

struct Counting {
    inner: SharedFs,
    calls: usize,
    fail_at: usize,
}

impl Counting {
    fn tick(&mut self) -> Result<(), StoreError> {
        self.calls += 1;
        if self.calls == self.fail_at {
            Err(StoreError::Io("injected".into()))
        } else {
            Ok(())
        }
    }
}

impl Storage for Counting {
    fn write(&mut self, n: &str, d: &[u8]) -> Result<(), StoreError> {
        self.tick()?;
        self.inner.write(n, d)
    }
    fn append(&mut self, n: &str, d: &[u8]) -> Result<(), StoreError> {
        self.tick()?;
        self.inner.append(n, d)
    }
    fn truncate(&mut self, n: &str, l: u64) -> Result<(), StoreError> {
        self.tick()?;
        self.inner.truncate(n, l)
    }
    fn fsync(&mut self, n: &str) -> Result<(), StoreError> {
        self.tick()?;
        self.inner.fsync(n)
    }
    fn rename(&mut self, a: &str, b: &str) -> Result<(), StoreError> {
        self.tick()?;
        self.inner.rename(a, b)
    }
    fn remove(&mut self, n: &str) -> Result<(), StoreError> {
        self.tick()?;
        self.inner.remove(n)
    }
    fn fsync_dir(&mut self) -> Result<(), StoreError> {
        self.tick()?;
        self.inner.fsync_dir()
    }
    fn read(&self, n: &str) -> Result<Option<Vec<u8>>, StoreError> {
        self.inner.read(n)
    }
    fn list(&self) -> Result<Vec<String>, StoreError> {
        self.inner.list()
    }
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

#[test]
fn every_crash_point_keeps_every_acknowledged_change_and_replay_restores_it() {
    let acked: RefCell<Vec<u64>> = RefCell::new(Vec::new());
    let scenario = |fs: &mut FaultFs| -> Result<(), FsError> {
        acked.borrow_mut().clear();
        let shared = SharedFs(Arc::new(Mutex::new(std::mem::take(fs))));
        {
            let Ok((j, _)) = FileJournal::open(shared.clone(), "journal", Limits::default()) else {
                *fs = std::mem::take(&mut *shared.0.lock().unwrap());
                return Err(FsError::Crashed);
            };
            let f = boot(None, Some(Arc::new(j)));
            for i in 1..=3 {
                if let EditingResult::Success(_) = f.run(vec![brick_op(i)]) {
                    acked.borrow_mut().push(i);
                }
            }
        }
        *fs = std::mem::take(&mut *shared.0.lock().unwrap());
        Ok(())
    };
    let failures = explore(scenario, |fs, step| {
        let shared = SharedFs(Arc::new(Mutex::new(fs.clone())));
        let (j, _) = match FileJournal::open(shared, "journal", Limits::default()) {
            Ok(x) => x,
            Err(err) => return Err(format!("step {step}: journal does not open: {err}")),
        };
        // Recovery: start from the empty base, replay what the journal confirmed.
        let g = boot(None, None);
        let report = replay_confirmed(&j.entries(), &g.recovery, &Limits::default());
        if !report.failed.is_empty() {
            return Err(format!("step {step}: replay failures {:?}", report.failed));
        }
        for i in acked.borrow().iter() {
            if g.material_in(L_VOXEL, bc(*i as i64, 0, 0)) != Some(*i as u32) {
                return Err(format!("step {step}: acknowledged operation {i} was lost"));
            }
        }
        // No change that was not at least journalled as confirmed may appear.
        for i in 1..=3u64 {
            let present = g.material_in(L_VOXEL, bc(i as i64, 0, 0)).is_some();
            let confirmed = report.replayed.contains(&OperationId::new(71, i));
            if present != confirmed {
                return Err(format!(
                    "step {step}: brick {i} present={present} but replayed={confirmed}"
                ));
            }
        }
        Ok(())
    });
    assert_eq!(
        failures,
        vec![],
        "acknowledged must mean durable at every crash point"
    );
}

#[test]
fn restart_twice_with_real_files_restores_confirmed_changes_and_never_applies_an_id_again() {
    let dir = std::env::temp_dir().join(format!("cube-journal-{}-1", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let journal_of = |dir: &std::path::Path| {
        let (j, rep) = FileJournal::open(
            DirStorage::open(dir).unwrap(),
            "journal.cubelog",
            Limits::default(),
        )
        .unwrap();
        (Arc::new(j), rep)
    };

    // Life 1: operations 1 and 2, snapshot, operation 3 after the snapshot; then "kill".
    let (j, _) = journal_of(&dir);
    let f = boot(None, Some(j.clone()));
    assert!(f.run(vec![brick_op(1)]).is_success());
    assert!(f.run(vec![brick_op(2)]).is_success());
    let mut st = DirStorage::open(&dir).unwrap();
    save_asset(&mut st, "world.cube", &f.reader.snapshot().export_view()).unwrap();
    assert!(f.run(vec![brick_op(3)]).is_success());
    let truth = f.reader.snapshot().export_view().content_hash();
    drop((f, j));

    // Life 2: restore the snapshot, replay the journal, rebuild idempotency.
    let (j, rep) = journal_of(&dir);
    assert_eq!(rep.truncated_bytes, 0);
    let snap = load_asset(
        &DirStorage::open(&dir).unwrap(),
        "world.cube",
        &Limits::default(),
    )
    .unwrap();
    let g = boot(Some(snap), Some(j.clone()));
    assert_eq!(
        g.material_in(L_VOXEL, bc(3, 0, 0)),
        None,
        "operation 3 is not in the snapshot"
    );
    let replay = replay_confirmed(&j.entries(), &g.recovery, &Limits::default());
    assert_eq!(replay.replayed, vec![OperationId::new(71, 3)]);
    assert_eq!(
        g.reader.snapshot().export_view().content_hash(),
        truth,
        "the exact pre-crash world"
    );
    let rec = g.svc.recover_idempotency().unwrap();
    assert_eq!(
        (
            rec.recorded,
            rec.lost_since_snapshot.len(),
            rec.blocked.len()
        ),
        (3, 0, 0)
    );
    // The same id again: answered, nothing applied. The same id with another payload: refused.
    let (rev, commits) = (g.revision(), g.commits());
    assert!(g.run(vec![brick_op(2)]).is_success());
    assert!(g.run(vec![brick_op(3)]).is_success());
    assert_eq!((g.revision(), g.commits()), (rev, commits));
    let mut other = brick_op(2);
    other.parameters = set_material(99, None);
    let r = g.run(vec![other]);
    assert!(
        matches!(r, EditingResult::ValidationFailure(EditingError::InvalidOperation(m)) if m.contains("different content")),
        "{r:?}"
    );
    // New work continues in the same journal.
    assert!(g.run(vec![brick_op(4)]).is_success());
    let truth2 = g.reader.snapshot().export_view().content_hash();
    drop((g, j));

    // Life 3: a second restart from the same snapshot replays three operations (3, 4 and the
    // continuing journal) without applying anything twice, and lands on the same world.
    let (j, _) = journal_of(&dir);
    let snap = load_asset(
        &DirStorage::open(&dir).unwrap(),
        "world.cube",
        &Limits::default(),
    )
    .unwrap();
    let h = boot(Some(snap), Some(j.clone()));
    let replay = replay_confirmed(&j.entries(), &h.recovery, &Limits::default());
    assert_eq!(
        replay.replayed,
        vec![OperationId::new(71, 3), OperationId::new(71, 4)]
    );
    assert_eq!(h.reader.snapshot().export_view().content_hash(), truth2);
    let again = replay_confirmed(&j.entries(), &h.recovery, &Limits::default());
    assert!(again.replayed.is_empty());
    let rec = h.svc.recover_idempotency().unwrap();
    assert_eq!(
        (
            rec.recorded,
            rec.lost_since_snapshot.len(),
            rec.blocked.len()
        ),
        (4, 0, 0)
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_crash_between_the_core_commit_and_the_confirmation_blocks_the_retry_until_resolved() {
    // The journal fails on the Confirmed record of operation 2: the change is in Core, the
    // client got AppliedUnconfirmed, then the process dies.
    let fs = fresh();
    let (j, _) = open(&fs);
    let gate = Arc::new(FailConfirm {
        inner: j,
        fail: std::sync::atomic::AtomicBool::new(false),
    });
    let f = boot(None, Some(gate.clone()));
    assert!(f.run(vec![brick_op(1)]).is_success());
    gate.fail.store(true, std::sync::atomic::Ordering::SeqCst);
    assert!(matches!(
        f.run(vec![brick_op(2)]),
        EditingResult::AppliedUnconfirmed(_)
    ));
    let saved = f.reader.snapshot().export_view();
    drop((f, gate));

    let (j2, _) = open(&fs);
    let g = boot(Some(saved), Some(Arc::new(j2)));
    let rec = g.svc.recover_idempotency().unwrap();
    assert_eq!(rec.blocked, vec![OperationId::new(71, 2)]);
    // The retry is refused, not applied a second time and not answered with a guess.
    let r = g.run(vec![brick_op(2)]);
    assert!(
        matches!(r, EditingResult::ValidationFailure(EditingError::InvalidOperation(m)) if m.contains("undecided")),
        "{r:?}"
    );
    // The restored world holds the change, so the explicit reconciliation says "applied".
    assert_eq!(g.material_in(L_VOXEL, bc(2, 0, 0)), Some(2));
    g.svc
        .resolve_pending(OperationId::new(71, 2), true)
        .unwrap();
    let rev = g.revision();
    assert!(g.run(vec![brick_op(2)]).is_success());
    assert_eq!(g.revision(), rev);
}

struct FailConfirm {
    inner: FileJournal<SharedFs>,
    fail: std::sync::atomic::AtomicBool,
}

impl OperationJournal for FailConfirm {
    fn append(&self, e: JournalEntry) -> Result<JournalSeq, EditingError> {
        if e.phase == JournalPhase::Confirmed && self.fail.load(std::sync::atomic::Ordering::SeqCst)
        {
            return Err(EditingError::PersistenceFailure);
        }
        self.inner.append(e)
    }
    fn read_from(&self, from: JournalSeq, limit: usize) -> Result<Vec<JournalEntry>, EditingError> {
        self.inner.read_from(from, limit)
    }
}

/// Child side of the process-crash test: appends forever and reports each acknowledged record.
#[test]
fn child_writer() {
    let Ok(dir) = std::env::var("CUBE_JOURNAL_CHILD_DIR") else {
        return;
    };
    let (j, _) = FileJournal::open(
        DirStorage::open(dir).unwrap(),
        "journal.cubelog",
        Limits::default(),
    )
    .unwrap();
    for n in 1.. {
        let seq = j
            .append(entry(n, JournalPhase::Intent, vec![n as u8; 2000]))
            .unwrap();
        println!("ACK {}", seq.0);
    }
}

#[test]
fn a_killed_process_loses_no_acknowledged_record_and_leaves_a_repairable_file() {
    let dir = std::env::temp_dir().join(format!("cube-journal-{}-child", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["child_writer", "--exact", "--nocapture", "--test-threads=1"])
        .env("CUBE_JOURNAL_CHILD_DIR", &dir)
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let mut last_ack = 0u64;
    let out = child.stdout.take().unwrap();
    for line in std::io::BufReader::new(out).lines() {
        let line = line.unwrap();
        if let Some(n) = line.strip_prefix("ACK ") {
            last_ack = n.parse().unwrap();
            if last_ack >= 40 {
                break;
            }
        }
    }
    child.kill().unwrap();
    let _ = child.wait();
    assert!(last_ack >= 40, "the child must have made progress");
    // Reopen: everything acknowledged is there, the sequence is gapless, a torn tail (if the
    // kill landed inside a write) is repaired.
    let (j, rep) = FileJournal::open(
        DirStorage::open(&dir).unwrap(),
        "journal.cubelog",
        Limits::default(),
    )
    .unwrap();
    assert!(
        rep.records as u64 >= last_ack,
        "{} records, {} acknowledged",
        rep.records,
        last_ack
    );
    let all = j.entries();
    assert!(
        all.iter()
            .enumerate()
            .all(|(i, x)| x.seq.0 == i as u64 + 1 && x.payload == vec![(i + 1) as u8; 2000])
    );
    assert_eq!(
        j.append(entry(9999, JournalPhase::Aborted, vec![]))
            .unwrap()
            .0,
        rep.records as u64 + 1
    );
    let _ = std::fs::remove_dir_all(&dir);
}
