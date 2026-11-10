//! PERS-08 (EXPERIMENTAL format): the world record, tested with the real generator, the real
//! Orchestration and voxel backend, the file journal and a real directory; creation crash safety
//! on the `FaultFs` model. Nothing here is a device durability claim (DG-2).
#![cfg(not(loom))]

use std::sync::{Arc, Mutex};

use cube_backend_voxel::{VoxelBackend, remove};
use cube_core::{
    AssetId, BrickCoord, GeneratorVersion, LayerId, OperationId, Region, VoxelPos, WorldSeed,
};
use cube_editing_api::{EditMode, EditingOperation, JournalSeq, OperationJournal, kinds};
use cube_integration_tests::fakes::*;
use cube_persistence::{
    DirStorage, Limits, RECORD_FILE, RecordError, Storage, StoreError, WorldRecord,
    codec::{decode_world_record, encode_world_record},
    create_world_with_record, load_world_record, open_world_with_record, replay_confirmed,
};
use cube_testkit::faultfs::{FaultFs, FsError, explore};
use cube_worldgen::{current_version, generate_brick, v1};

fn limits() -> Limits {
    Limits::default()
}

fn record() -> WorldRecord {
    WorldRecord {
        world_id: 0xC0FFEE,
        seed: WorldSeed::new(7),
        generator_version: current_version(),
        structure_version: cube_persistence::STRUCTURE_VERSION,
    }
}

fn runnable(v: GeneratorVersion) -> bool {
    cube_worldgen::registry().iter().any(|i| i.version == v)
}

fn surface(seed: WorldSeed) -> (BrickCoord, i64) {
    let h = v1::height(seed, 0, 0);
    (BrickCoord::new(0, h.div_euclid(16), 0), h)
}

/// The world of a record: its base is generated from the record's seed and version, never from
/// anything else.
fn world_of(rec: &WorldRecord, journal: Arc<dyn OperationJournal>) -> Fixture {
    let f = fixture_with(Options {
        backends: vec![VoxelBackend::arc()],
        journal_port: Some(journal),
        with_payload: true,
        ..Options::default()
    });
    let (centre, _) = surface(rec.seed);
    let mut n = 0;
    for z in -1..=1 {
        for y in -1..=1 {
            for x in -1..=1 {
                n += 1;
                let b = BrickCoord::new(centre.x + x, centre.y + y, centre.z + z);
                let g = generate_brick(rec.seed, rec.generator_version, b).unwrap();
                f.system
                    .materialise(
                        OperationId::new(5, n),
                        LayerId::new(L_VOXEL),
                        0,
                        b,
                        g.into_change(),
                    )
                    .unwrap();
            }
        }
    }
    f
}

fn remove_op(n: u64, p: [i64; 3]) -> EditingOperation {
    EditingOperation {
        id: OperationId::new(41, n),
        kind: kinds::REMOVE_GEOMETRY,
        asset: AssetId::new(ASSET),
        layers: vec![LayerId::new(L_VOXEL)],
        parameters: remove(None),
        read_region: None,
        write_region: Region::new(
            0,
            VoxelPos::new(p[0], p[1], p[2]),
            VoxelPos::new(p[0] + 1, p[1] + 1, p[2] + 1),
        )
        .unwrap(),
        conversion: None,
        mode: EditMode::Destructive,
    }
}

fn hash(f: &Fixture) -> u128 {
    f.reader.snapshot().export_view().content_hash().value()
}

#[test]
fn a_world_survives_a_kill_regenerates_its_base_and_continues_editing() {
    let dir = std::env::temp_dir().join(format!("cube-world-record-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let rec = record();
    let (_, h) = surface(rec.seed);
    let h1 = v1::height(rec.seed, 1, 0);

    // 1. Create the world record (and the first manifest).
    let mut st = DirStorage::open(&dir).unwrap();
    create_world_with_record(&mut st, &rec, &limits()).unwrap();
    assert_eq!(load_world_record(&st, &limits()).unwrap(), Some(rec));

    // 2-5. Life 1: generate from the record, one user edit, save (the journal is durable), kill.
    let (loaded, w) =
        open_world_with_record(DirStorage::open(&dir).unwrap(), &limits(), &runnable).unwrap();
    assert_eq!(loaded, rec);
    let j: Arc<dyn OperationJournal> = Arc::new(w.journal);
    let f = world_of(&loaded, j.clone());
    let base = hash(&f);
    assert!(f.run(vec![remove_op(1, [0, h, 0])]).is_success());
    let edited = hash(&f);
    assert_ne!(base, edited);
    drop((f, j));

    // 6-9. Life 2: load the record, regenerate the base from its seed and version, replay.
    let (loaded, w) =
        open_world_with_record(DirStorage::open(&dir).unwrap(), &limits(), &runnable).unwrap();
    assert_eq!((loaded.seed, loaded.generator_version), (rec.seed, rec.generator_version));
    let entries = w.journal.read_from(JournalSeq(0), usize::MAX).unwrap();
    let j: Arc<dyn OperationJournal> = Arc::new(w.journal);
    let f = world_of(&loaded, j.clone());
    assert_eq!(hash(&f), base, "the regenerated base equals the original base");
    let rep = replay_confirmed(&entries, &f.recovery, &limits());
    assert!(rep.failed.is_empty() && rep.replayed.len() == 1, "{rep:?}");
    assert_eq!(hash(&f), edited, "the Core hash equals the pre-kill world");

    // The retry table is rebuilt: the same id is answered, nothing is applied twice.
    assert_eq!(f.svc.recover_idempotency().unwrap().recorded, 1);
    let (rev, commits) = (f.revision(), f.commits());
    assert!(f.run(vec![remove_op(1, [0, h, 0])]).is_success());
    assert_eq!((f.revision(), f.commits()), (rev, commits));

    // 10. Editing continues, in the same journal.
    assert!(f.run(vec![remove_op(2, [1, h1, 0])]).is_success());
    let continued = hash(&f);
    assert_ne!(continued, edited);
    drop((f, j));

    // And once more: the continued world is restored too.
    let (loaded, w) =
        open_world_with_record(DirStorage::open(&dir).unwrap(), &limits(), &runnable).unwrap();
    let entries = w.journal.read_from(JournalSeq(0), usize::MAX).unwrap();
    let f = world_of(&loaded, Arc::new(w.journal));
    let rep = replay_confirmed(&entries, &f.recovery, &limits());
    assert_eq!(rep.replayed.len(), 2, "{rep:?}");
    assert_eq!(hash(&f), continued);
    let _ = std::fs::remove_dir_all(&dir);
}

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

#[test]
fn creation_is_crash_safe_at_every_step_and_never_half_visible() {
    let rec = record();
    let scenario = |fs: &mut FaultFs| -> Result<(), FsError> {
        let shared = SharedFs(Arc::new(Mutex::new(std::mem::take(fs))));
        let mut st = shared.clone();
        let _ = create_world_with_record(&mut st, &rec, &limits());
        *fs = std::mem::take(&mut *shared.0.lock().unwrap());
        Ok(())
    };
    let failures = explore(scenario, |fs, step| {
        let shared = SharedFs(Arc::new(Mutex::new(fs.clone())));
        match open_world_with_record(shared.clone(), &limits(), &runnable) {
            Ok((r, w)) => {
                if r != rec || w.manifest.generator_version != rec.generator_version.raw() {
                    return Err(format!("step {step}: wrong world"));
                }
            }
            // Not created yet: nothing, or only the record (an incomplete creation).
            Err(RecordError::RecordMissing | RecordError::World(_)) => {}
            Err(other) => return Err(format!("step {step}: unexpected {other}")),
        }
        // Whatever happened, creating again works and gives the complete world.
        let mut again = shared.clone();
        match create_world_with_record(&mut again, &rec, &limits()) {
            Ok(()) | Err(RecordError::World(cube_persistence::WorldError::AlreadyExists)) => {}
            Err(err) => return Err(format!("step {step}: retry failed: {err}")),
        }
        open_world_with_record(shared, &limits(), &runnable)
            .map(|_| ())
            .map_err(|err| format!("step {step}: after retry: {err}"))
    });
    assert_eq!(failures, vec![]);
}

#[test]
fn records_with_the_wrong_generator_structure_or_content_are_refused_never_adapted() {
    let rec = record();
    let fs = fresh();
    let mut st = fs.clone();
    create_world_with_record(&mut st, &rec, &limits()).unwrap();
    assert!(
        create_world_with_record(&mut st, &rec, &limits()).is_err(),
        "an existing world is never overwritten"
    );
    // A generator this build cannot reproduce is an error, not a substitute.
    let r = open_world_with_record(fs.clone(), &limits(), &|_| false);
    assert!(matches!(r.err(), Some(RecordError::GeneratorUnavailable(_))));
    // Record and manifest disagree about the generator: refused.
    let other = WorldRecord {
        generator_version: GeneratorVersion::new(2),
        ..rec
    };
    put(&fs, RECORD_FILE, &encode_world_record(&other));
    let r = open_world_with_record(fs.clone(), &limits(), &|_| true);
    assert!(matches!(
        r.err(),
        Some(RecordError::GeneratorMismatch { manifest: 1, .. })
    ));
    // An unknown structure version is refused.
    let future = WorldRecord {
        structure_version: 9,
        ..rec
    };
    put(&fs, RECORD_FILE, &encode_world_record(&future));
    assert!(matches!(
        open_world_with_record(fs.clone(), &limits(), &runnable).err(),
        Some(RecordError::Corrupt(_))
    ));
    // Damage anywhere is refused.
    let good = encode_world_record(&rec);
    for i in 0..good.len() {
        let mut b = good.clone();
        b[i] ^= 0x01;
        assert!(decode_world_record(&b, &limits()).is_err(), "byte {i}");
    }
    for cut in 0..good.len() {
        assert!(decode_world_record(&good[..cut], &limits()).is_err());
    }
    // A manifest without a record is a damaged world.
    fs.0.lock().unwrap().remove(RECORD_FILE).unwrap();
    fs.0.lock().unwrap().fsync_dir().unwrap();
    assert!(matches!(
        open_world_with_record(fs.clone(), &limits(), &runnable).err(),
        Some(RecordError::RecordMissing)
    ));
}
