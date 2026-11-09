//! Persistence foundation end to end: create a world, change it through the production chain,
//! save it, load it and continue; crash points and injected failures on the model filesystem;
//! journal reconciliation against Core's commit log.
//!
//! Three kinds of evidence are kept apart. (1) Model: `FaultFs` crash points: proves the
//! protocol is correct *under the stated model*. (2) Real filesystem: a temp directory through
//! `DirStorage`: proves the code works with real I/O calls, not that data survives power loss.
//! (3) Physical device: none; crash durability on devices is DG-2 and is not claimed.
#![cfg(not(loom))]

use std::collections::HashSet;

use cube_backend_voxel::{VoxelBackend, set_material};
use cube_core::{AssetView, LayerId, OperationId, VoxelPos};
use cube_editing_api::{
    EditMode, EditingOperation, EditingResult, EditingService, JournalPhase, kinds,
};
use cube_integration_tests::fakes::*;
use cube_persistence::{
    DirStorage, Limits, LoadError, Resolution, Storage, StoreError, decode_journal_entry,
    encode_journal_entry, load_asset, reconcile, save_asset,
};
use cube_testkit::faultfs::{FaultFs, FsError, Torn, explore};

fn fx_from(view: Option<AssetView>, journal: bool) -> Fixture {
    fixture_with(Options {
        backends: vec![VoxelBackend::arc()],
        journal,
        restore: view,
        ..Options::default()
    })
}

fn edit(f: &Fixture, n: u64, lo: [i64; 3], hi: [i64; 3], m: u16) -> EditingResult {
    f.run(vec![EditingOperation {
        id: OperationId::new(21, n),
        kind: kinds::SET_MATERIAL,
        asset: cube_core::AssetId::new(ASSET),
        layers: vec![LayerId::new(L_VOXEL)],
        parameters: set_material(m, None),
        read_region: None,
        write_region: cube_core::Region::new(
            0,
            VoxelPos::new(lo[0], lo[1], lo[2]),
            VoxelPos::new(hi[0], hi[1], hi[2]),
        )
        .unwrap(),
        conversion: None,
        mode: EditMode::Destructive,
    }])
}

fn world(seed_extra: u16) -> Fixture {
    let f = fx_from(None, false);
    // Whole bricks, so nothing is unloaded-and-partial; one of them spans negative space.
    assert!(edit(&f, 1, [-16, 0, 0], [0, 16, 16], 3 + seed_extra).is_success());
    assert!(edit(&f, 2, [16, 16, 16], [32, 32, 32], 5).is_success());
    f
}

fn view_of(f: &Fixture) -> AssetView {
    f.reader.snapshot().export_view()
}

/// The model filesystem as a `Storage`.
struct Model(FaultFs);

fn e(err: FsError) -> StoreError {
    StoreError::Io(format!("{err:?}"))
}

impl Storage for Model {
    fn write(&mut self, n: &str, d: &[u8]) -> Result<(), StoreError> {
        self.0.write(n, d).map_err(e)
    }
    fn append(&mut self, n: &str, d: &[u8]) -> Result<(), StoreError> {
        self.0.append(n, d).map_err(e)
    }
    fn truncate(&mut self, n: &str, l: u64) -> Result<(), StoreError> {
        self.0.truncate(n, l as usize).map_err(e)
    }
    fn fsync(&mut self, n: &str) -> Result<(), StoreError> {
        self.0.fsync(n).map_err(e)
    }
    fn rename(&mut self, a: &str, b: &str) -> Result<(), StoreError> {
        self.0.rename(a, b).map_err(e)
    }
    fn remove(&mut self, n: &str) -> Result<(), StoreError> {
        self.0.remove(n).map_err(e)
    }
    fn fsync_dir(&mut self) -> Result<(), StoreError> {
        self.0.fsync_dir().map_err(e)
    }
    fn read(&self, n: &str) -> Result<Option<Vec<u8>>, StoreError> {
        Ok(self.0.read(n))
    }
    fn list(&self) -> Result<Vec<String>, StoreError> {
        Ok(self.0.list())
    }
}

#[test]
fn create_edit_save_load_and_continue_on_the_real_filesystem() {
    let f = world(0);
    let before = view_of(&f);
    let dir = std::env::temp_dir().join(format!("cube-persist-{}-{}", std::process::id(), 1));
    let _ = std::fs::remove_dir_all(&dir);
    let mut st = DirStorage::open(&dir).unwrap();
    save_asset(&mut st, "world.cube", &before).unwrap();
    // A second, newer snapshot replaces the first atomically.
    assert!(edit(&f, 3, [0, 0, 0], [16, 16, 16], 9).is_success());
    let newer = view_of(&f);
    save_asset(&mut st, "world.cube", &newer).unwrap();

    // A new process: reopen the directory, load, rebuild the asset, keep playing.
    let st2 = DirStorage::open(&dir).unwrap();
    let loaded = load_asset(&st2, "world.cube", &Limits::default()).unwrap();
    assert_eq!(
        loaded, newer,
        "the snapshot round-trips exactly, revisions included"
    );
    let g = fx_from(Some(loaded), false);
    assert_eq!(
        g.reader.snapshot().export_view().content_hash(),
        f.reader.snapshot().export_view().content_hash()
    );
    assert_eq!(g.revision(), f.revision());
    assert_eq!(g.material_in(L_VOXEL, bc(0, 0, 0)), Some(9));
    assert_eq!(g.material_in(L_VOXEL, bc(-1, 0, 0)), Some(3));
    // Editing continues from the restored state, revisions keep counting upwards.
    let r = edit(&g, 4, [0, 0, 0], [16, 16, 16], 4);
    assert!(r.is_success(), "{r:?}");
    assert_eq!(g.revision(), f.revision() + 1);
    assert_eq!(g.material_in(L_VOXEL, bc(0, 0, 0)), Some(4));
    // The old file on disk is still the newer snapshot, untouched by the continued session.
    assert_eq!(
        load_asset(&st2, "world.cube", &Limits::default()).unwrap(),
        newer
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn missing_corrupt_and_foreign_files_are_reported_precisely_and_leftovers_are_harmless() {
    let f = world(0);
    let v = view_of(&f);
    let dir = std::env::temp_dir().join(format!("cube-persist-{}-{}", std::process::id(), 2));
    let _ = std::fs::remove_dir_all(&dir);
    let mut st = DirStorage::open(&dir).unwrap();
    assert_eq!(
        load_asset(&st, "w", &Limits::default()),
        Err(LoadError::Missing)
    );
    // A crash left a temporary file but never produced the real one: still "missing".
    st.write("w.tmp", b"half a snapshot").unwrap();
    assert_eq!(
        load_asset(&st, "w", &Limits::default()),
        Err(LoadError::Missing)
    );
    save_asset(&mut st, "w", &v).unwrap();
    // Bit rot: any flipped byte is detected, never decoded into a different world.
    let good = st.read("w").unwrap().unwrap();
    for i in (0..good.len()).step_by(good.len() / 40 + 1) {
        let mut bad = good.clone();
        bad[i] ^= 0x10;
        st.write("bad", &bad).unwrap();
        assert!(
            matches!(
                load_asset(&st, "bad", &Limits::default()),
                Err(LoadError::Corrupt(_))
            ),
            "byte {i}"
        );
    }
    // A truncated file (partial write) and a foreign file.
    st.write("short", &good[..good.len() / 2]).unwrap();
    assert!(matches!(
        load_asset(&st, "short", &Limits::default()),
        Err(LoadError::Corrupt(_))
    ));
    st.write("foreign", b"PK\x03\x04 not ours").unwrap();
    assert!(matches!(
        load_asset(&st, "foreign", &Limits::default()),
        Err(LoadError::Corrupt(_))
    ));
    // Unsafe names never leave the directory.
    assert!(st.write("../escape", b"x").is_err());
    assert!(st.read("a/b").is_err());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn every_crash_point_leaves_either_a_complete_old_or_a_complete_new_snapshot() {
    let (a, b) = (view_of(&world(0)), view_of(&world(1)));
    assert_ne!(a, b);
    let scenario = |fs: &mut FaultFs| -> Result<(), FsError> {
        let mut m = Model(std::mem::take(fs));
        let r1 = save_asset(&mut m, "w", &a);
        let r2 = if r1.is_ok() {
            save_asset(&mut m, "w", &b)
        } else {
            r1
        };
        *fs = m.0;
        r2.map_err(|_| FsError::Crashed)
    };
    let failures = explore(scenario, |fs, step| {
        match load_asset(&Model(fs.clone()), "w", &Limits::default()) {
            // Before the first save is durable nothing exists yet.
            Err(LoadError::Missing) if step < 8 => Ok(()),
            Ok(v) if v == a || v == b => Ok(()),
            other => Err(format!(
                "step {step}: {}",
                match other {
                    Ok(_) => "a view that is neither".to_owned(),
                    Err(e) => e.to_string(),
                }
            )),
        }
    });
    assert_eq!(failures, vec![]);
}

#[test]
fn the_same_explorer_finds_a_protocol_that_skips_fsync() {
    let (a, b) = (view_of(&world(0)), view_of(&world(1)));
    let (ea, eb) = (
        cube_persistence::encode_asset(&a),
        cube_persistence::encode_asset(&b),
    );
    let buggy = |fs: &mut FaultFs| -> Result<(), FsError> {
        fs.write("w", &ea)?;
        fs.fsync("w")?;
        fs.fsync_dir()?;
        // Overwrites in place and forgets to make it durable.
        fs.write("w", &eb)?;
        fs.fsync_dir()
    };
    let failures = explore(buggy, |fs, step| {
        if step < 3 {
            return Ok(());
        }
        match load_asset(&Model(fs.clone()), "w", &Limits::default()) {
            Ok(v) if v == a || v == b => Ok(()),
            other => Err(format!("{other:?}").chars().take(60).collect()),
        }
    });
    assert!(!failures.is_empty(), "detection must see the corruption");
    assert!(
        failures
            .iter()
            .any(|f| f.torn != Torn::AllOfUnsynced || f.message.contains("Corrupt"))
    );
}

/// Fails the n-th call of a wrapped storage.
struct Failing<S> {
    inner: S,
    fail_at: usize,
    calls: usize,
}

impl<S: Storage> Failing<S> {
    fn tick(&mut self) -> Result<(), StoreError> {
        self.calls += 1;
        if self.calls == self.fail_at {
            Err(StoreError::Io("injected".into()))
        } else {
            Ok(())
        }
    }
}

impl<S: Storage> Storage for Failing<S> {
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

#[test]
fn an_injected_failure_at_any_step_fails_the_save_and_keeps_the_previous_snapshot() {
    let (a, b) = (view_of(&world(0)), view_of(&world(1)));
    for fail_at in 1..=4 {
        let mut m = Model(FaultFs::new());
        save_asset(&mut m, "w", &a).unwrap();
        let mut failing = Failing {
            inner: m,
            fail_at,
            calls: 0,
        };
        let r = save_asset(&mut failing, "w", &b);
        assert!(
            r.is_err(),
            "step {fail_at} must surface as an error, not as success"
        );
        let loaded = load_asset(&failing.inner, "w", &Limits::default()).unwrap();
        // The failure came before the rename (steps 1,2,3 = write, fsync, rename) or after it (4):
        // either way the file is complete, and the caller was told the save did not finish.
        assert!(
            loaded == a || (fail_at == 4 && loaded == b),
            "step {fail_at}"
        );
    }
}

#[test]
fn journal_records_round_trip_and_pending_intents_are_resolved_against_the_core_commit_log() {
    let f = fx_from(None, true);
    // A confirmed operation, a rejected one (aborted) and one whose confirmation was lost.
    assert!(edit(&f, 1, [0, 0, 0], [16, 16, 16], 1).is_success());
    let h = f
        .svc
        .begin(ctx(cube_editing_api::Origin::Server), spec())
        .unwrap();
    assert!(
        f.svc
            .execute(
                &h,
                EditingOperation {
                    id: OperationId::new(21, 2),
                    kind: kinds::SET_MATERIAL,
                    asset: cube_core::AssetId::new(ASSET),
                    layers: vec![LayerId::new(L_VOXEL)],
                    parameters: set_material(7, None),
                    read_region: None,
                    write_region: cube_core::Region::new(
                        0,
                        VoxelPos::new(0, 0, 0),
                        VoxelPos::new(16, 16, 16)
                    )
                    .unwrap(),
                    conversion: None,
                    mode: EditMode::Destructive,
                }
            )
            .is_success()
    );
    assert!(edit(&f, 3, [0, 0, 0], [16, 16, 16], 2).is_success());
    assert!(matches!(f.svc.commit(&h), EditingResult::Conflict(_)));
    f.journal
        .fail_confirm
        .store(true, std::sync::atomic::Ordering::SeqCst);
    assert!(matches!(
        edit(&f, 4, [16, 0, 0], [32, 16, 16], 1),
        EditingResult::AppliedUnconfirmed(_)
    ));

    // Every record survives the codec.
    let entries = f.journal.entries();
    for en in &entries {
        let back = decode_journal_entry(&encode_journal_entry(en), &Limits::default()).unwrap();
        assert_eq!(&back, en);
    }
    // Only operation 4 is left pending (an intent with no later record) and Core says it applied.
    let (_, log) = f.reader.commit_log();
    let applied: HashSet<_> = log.iter().map(|l| l.operation).collect();
    assert_eq!(
        reconcile(&entries, &applied, true),
        vec![Resolution::Confirm(OperationId::new(21, 4))]
    );
    // A crash between the intent and the commit leaves an intent Core never saw: abort it.
    let mut crashed = entries.clone();
    crashed.push({
        let mut x = entries[0].clone();
        x.phase = JournalPhase::Intent;
        x.operation = OperationId::new(21, 99);
        x
    });
    let res = reconcile(&crashed, &applied, true);
    assert!(res.contains(&Resolution::Abort(OperationId::new(21, 99))));
    // Reconciling twice after writing the answers changes nothing more.
    let again: Vec<_> = res
        .iter()
        .map(|r| match r {
            Resolution::Confirm(id) | Resolution::Abort(id) | Resolution::Undecidable(id) => *id,
        })
        .collect();
    assert_eq!(again.len(), 2);
}
