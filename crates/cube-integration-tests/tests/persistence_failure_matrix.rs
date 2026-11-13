//! M1-A5: the storage failure matrix, cells that the crash-point model and the existing tests do
//! not reach. The full matrix and which test covers which cell: docs/roadmap/WORK-PACKAGES.md,
//! section "M1-A5 failure matrix". This file adds: an unreadable file is not a missing file,
//! a write that fails half way (disk full) leaves a repairable torn tail, repeated failures never
//! touch the last good snapshot, and a read-only file is a controlled error on a real directory.
#![cfg(not(loom))]

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use cube_core::{AssetId, OperationId};
use cube_editing_api::{
    ActorId, EditingError, JournalEntry, JournalPhase, JournalSeq, OperationJournal,
};
use cube_persistence::{
    DirStorage, FileJournal, JournalOpenError, Limits, LoadError, Storage, StoreError, load_asset,
    save_asset,
};

/// An in-memory directory with switchable faults.
#[derive(Default)]
struct Inner {
    files: BTreeMap<String, Vec<u8>>,
    /// Reading this file fails (a permission or device error, not "no such file").
    unreadable: Option<String>,
    /// The n-th append (1-based) to the journal file itself writes half of its bytes and then
    /// fails: a full disk.
    partial_append_at: Option<usize>,
    appends: usize,
    /// Every call that changes anything.
    mutations: usize,
    /// The n-th mutating call (1-based) fails before it changes anything.
    fail_mutation_at: Option<usize>,
}

#[derive(Clone, Default)]
struct Mem(Arc<Mutex<Inner>>);

fn disk_full() -> StoreError {
    StoreError::Io("no space left on device".to_owned())
}

impl Mem {
    fn tick(&self) -> Result<(), StoreError> {
        let mut g = self.0.lock().unwrap();
        g.mutations += 1;
        if g.fail_mutation_at == Some(g.mutations) {
            return Err(StoreError::Io("injected failure".to_owned()));
        }
        Ok(())
    }
    fn mutations(&self) -> usize {
        self.0.lock().unwrap().mutations
    }
    fn raw(&self, name: &str) -> Option<Vec<u8>> {
        self.0.lock().unwrap().files.get(name).cloned()
    }
}

impl Storage for Mem {
    fn write(&mut self, name: &str, data: &[u8]) -> Result<(), StoreError> {
        self.tick()?;
        self.0
            .lock()
            .unwrap()
            .files
            .insert(name.to_owned(), data.to_vec());
        Ok(())
    }
    fn append(&mut self, name: &str, data: &[u8]) -> Result<(), StoreError> {
        self.tick()?;
        let mut g = self.0.lock().unwrap();
        if name == "journal" {
            g.appends += 1;
        }
        let partial = name == "journal" && g.partial_append_at == Some(g.appends);
        let take = if partial { data.len() / 2 } else { data.len() };
        g.files
            .entry(name.to_owned())
            .or_default()
            .extend_from_slice(&data[..take]);
        if partial { Err(disk_full()) } else { Ok(()) }
    }
    fn truncate(&mut self, name: &str, len: u64) -> Result<(), StoreError> {
        self.tick()?;
        if let Some(f) = self.0.lock().unwrap().files.get_mut(name) {
            f.truncate(len as usize);
        }
        Ok(())
    }
    fn fsync(&mut self, _name: &str) -> Result<(), StoreError> {
        self.tick()
    }
    fn rename(&mut self, from: &str, to: &str) -> Result<(), StoreError> {
        self.tick()?;
        let mut g = self.0.lock().unwrap();
        if let Some(f) = g.files.remove(from) {
            g.files.insert(to.to_owned(), f);
        }
        Ok(())
    }
    fn remove(&mut self, name: &str) -> Result<(), StoreError> {
        self.tick()?;
        self.0.lock().unwrap().files.remove(name);
        Ok(())
    }
    fn fsync_dir(&mut self) -> Result<(), StoreError> {
        self.tick()
    }
    fn read(&self, name: &str) -> Result<Option<Vec<u8>>, StoreError> {
        let g = self.0.lock().unwrap();
        if g.unreadable.as_deref() == Some(name) {
            return Err(StoreError::Io("permission denied".to_owned()));
        }
        Ok(g.files.get(name).cloned())
    }
    fn list(&self) -> Result<Vec<String>, StoreError> {
        Ok(self.0.lock().unwrap().files.keys().cloned().collect())
    }
}

fn entry(op: u64) -> JournalEntry {
    JournalEntry {
        seq: JournalSeq(0),
        phase: JournalPhase::Intent,
        operation: OperationId::new(61, op),
        actor: ActorId(1),
        asset: AssetId::new(1),
        post_tokens: vec![],
        delta: None,
        members: vec![(OperationId::new(61, op), op)],
        payload: vec![7; 30],
    }
}

fn open(m: &Mem) -> Result<(FileJournal<Mem>, cube_persistence::OpenReport), JournalOpenError> {
    FileJournal::open(m.clone(), "journal", Limits::default())
}

#[test]
fn an_unreadable_file_is_an_error_never_an_empty_or_missing_one() {
    // A journal of three records exists. Reading it fails (permission, device): opening must say
    // so and must not treat the journal as empty, which would start a new one over it.
    let m = Mem::default();
    {
        let (j, _) = open(&m).unwrap();
        for n in 1..=3 {
            j.append(entry(n)).unwrap();
        }
    }
    let before = (m.raw("journal"), m.raw("journal.mark"));
    let writes_before = m.mutations();
    m.0.lock().unwrap().unreadable = Some("journal".to_owned());
    let r = open(&m);
    assert!(
        matches!(r, Err(JournalOpenError::Store(_))),
        "{:?}",
        r.err()
    );
    // The mark file too.
    m.0.lock().unwrap().unreadable = Some("journal.mark".to_owned());
    let r = open(&m);
    assert!(
        matches!(r, Err(JournalOpenError::Store(_))),
        "{:?}",
        r.err()
    );
    assert_eq!(
        m.mutations(),
        writes_before,
        "a failed open changes nothing"
    );
    assert_eq!((m.raw("journal"), m.raw("journal.mark")), before);
    // And a snapshot: unreadable is `Store`, not `Missing`.
    m.0.lock().unwrap().unreadable = Some("snapshot".to_owned());
    let mut s = m.clone();
    s.write("snapshot", b"x").unwrap();
    let r = load_asset(&m, "snapshot", &Limits::default());
    assert!(matches!(r, Err(LoadError::Store(_))), "{r:?}");
    // Once readable again, the journal is whole.
    m.0.lock().unwrap().unreadable = None;
    let (j, rep) = open(&m).unwrap();
    assert_eq!(rep.records, 3);
    assert_eq!(j.append(entry(4)).unwrap(), JournalSeq(4));
}

#[test]
fn a_write_that_fails_half_way_leaves_a_torn_tail_and_the_acknowledged_records_stay() {
    // Disk full: the fourth append writes half a record and fails.
    let m = Mem::default();
    m.0.lock().unwrap().partial_append_at = Some(4);
    let (j, _) = open(&m).unwrap();
    for n in 1..=3 {
        j.append(entry(n)).unwrap();
    }
    let good_len = m.raw("journal").unwrap().len();
    let r = j.append(entry(4));
    assert_eq!(r, Err(EditingError::PersistenceFailure));
    assert!(j.is_poisoned(), "a failed write poisons the journal");
    assert!(
        j.append(entry(5)).is_err(),
        "nothing is acknowledged afterwards"
    );
    let torn = m.raw("journal").unwrap().len();
    assert!(torn > good_len, "half a record reached the disk");
    drop(j);
    // After space is freed and the process restarts: the torn tail is cut, nothing acknowledged
    // is lost, and the sequence continues without a gap.
    m.0.lock().unwrap().partial_append_at = None;
    let (j, rep) = open(&m).unwrap();
    assert_eq!((rep.records, rep.truncated_bytes), (3, torn - good_len));
    assert_eq!(m.raw("journal").unwrap().len(), good_len);
    assert_eq!(j.append(entry(4)).unwrap(), JournalSeq(4));
    assert_eq!(j.read_from(JournalSeq(1), 10).unwrap().len(), 4);
}

#[test]
fn repeated_failed_saves_never_touch_the_last_good_snapshot() {
    use cube_core::{AssetHost, AssetView, GeometryAsset, Layer, LayerData, LayerId, LayerKind};
    let view = |id: u64| -> AssetView {
        let layer = Layer::new(
            LayerId::new(1),
            cube_core::GeometryTransform::IDENTITY,
            LayerData::empty(LayerKind::Voxel),
        );
        AssetHost::new(GeometryAsset::new(AssetId::new(id), vec![layer]).unwrap())
            .reader()
            .snapshot()
            .export_view()
    };
    let good = view(7);
    let m = Mem::default();
    let mut s = m.clone();
    save_asset(&mut s, "snapshot", &good).unwrap();
    let steps = {
        let before = m.mutations();
        save_asset(&mut s, "snapshot", &good).unwrap();
        m.mutations() - before
    };
    // Three rounds: every step of a save fails once, in turn. The file on disk is always a whole
    // snapshot: the one before the save, or (when the failure came after the rename, at the
    // directory sync) the offered one; never a half-written one, never missing.
    let mut current = good.clone();
    for round in 0..3 {
        for step in 1..=steps {
            let offered = view(100 + round * 10 + step as u64);
            let base = m.mutations();
            m.0.lock().unwrap().fail_mutation_at = Some(base + step);
            let r = save_asset(&mut s, "snapshot", &offered);
            assert!(r.is_err(), "round {round} step {step}");
            m.0.lock().unwrap().fail_mutation_at = None;
            let loaded = load_asset(&m, "snapshot", &Limits::default())
                .unwrap_or_else(|e| panic!("round {round} step {step}: {e}"));
            assert!(
                loaded == current || loaded == offered,
                "round {round} step {step}: neither the old nor the offered snapshot"
            );
            current = loaded;
        }
    }
    assert!(
        m.list()
            .unwrap()
            .iter()
            .all(|f| f == "snapshot" || f.ends_with(".tmp")),
        "leftovers are only temporary files: {:?}",
        m.list()
    );
    // After the failures a save works again and replaces the snapshot.
    let newer = view(8);
    save_asset(&mut s, "snapshot", &newer).unwrap();
    assert_eq!(
        load_asset(&m, "snapshot", &Limits::default()).unwrap(),
        newer
    );
}

#[test]
fn a_read_only_journal_file_on_a_real_directory_is_a_controlled_error() {
    let dir = std::env::temp_dir().join(format!("cube-readonly-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let storage = DirStorage::open(&dir).unwrap();
    {
        let (j, _) = FileJournal::open(storage.clone(), "journal", Limits::default()).unwrap();
        j.append(entry(1)).unwrap();
    }
    let path = dir.join("journal");
    let mut perm = std::fs::metadata(&path).unwrap().permissions();
    perm.set_readonly(true);
    std::fs::set_permissions(&path, perm.clone()).unwrap();
    // A privileged user (or an OS that ignores the flag) can still write: nothing to check.
    let ignored = std::fs::OpenOptions::new().append(true).open(&path).is_ok();
    if !ignored {
        let (j, rep) = FileJournal::open(storage.clone(), "journal", Limits::default()).unwrap();
        assert_eq!(rep.records, 1, "reading a read-only journal works");
        let r = j.append(entry(2));
        assert_eq!(r, Err(EditingError::PersistenceFailure));
        assert!(j.is_poisoned());
    }
    #[allow(clippy::permissions_set_readonly_false)]
    perm.set_readonly(false);
    std::fs::set_permissions(&path, perm).unwrap();
    let (j, rep) = FileJournal::open(storage, "journal", Limits::default()).unwrap();
    assert_eq!(rep.records, 1, "the record written before is intact");
    assert_eq!(j.append(entry(2)).unwrap(), JournalSeq(2));
    drop(j);
    let _ = std::fs::remove_dir_all(&dir);
}
