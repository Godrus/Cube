//! The first visible world, headless: `WorldSeed -> WorldGen -> Core -> Processing -> voxel faces
//! -> client runtime -> render batch` (the data a Godot adapter will receive), then an edit
//! through Editing API and Orchestration that updates the geometry, and a save and restore that
//! brings the edited world back by regenerating the untouched base from the seed and replaying
//! the journalled edit.
//!
//! This is **not** a player-visible scene: there is no engine adapter, no window and no input
//! yet (ADP-01 and CLI-02..04 are later). Every claim here is about data, checked against an
//! independent oracle built straight from the generator's voxel function.
#![cfg(not(loom))]

use std::collections::BTreeSet;
use std::sync::Arc;

use cube_backend_voxel::{FACES_KIND, FacesMesh, FacesProvider, VoxelBackend, remove};
use cube_client::{ClientConfig, ClientWorld};
use cube_core::{BrickCoord, DirtyRegion, LayerId, OperationId, Region, VoxelPos, WorldSeed};
use cube_editing_api::{EditMode, EditingOperation, JournalSeq, OperationJournal, kinds};
use cube_integration_tests::fakes::*;
use cube_orchestration::DirtySink;
use cube_persistence::{DirStorage, Limits, create_world, open_world, replay_confirmed};
use cube_processing::{Executor, ExecutorConfig, ManualClock, ProviderRegistry};
use cube_worldgen::{current_version, generate_brick, v1, voxel_at};

const SEED: WorldSeed = WorldSeed::new(7);

/// The brick that holds the surface of column (0, 0), and the column's surface height.
fn surface() -> (BrickCoord, i64) {
    let h = v1::height(SEED, 0, 0);
    (BrickCoord::new(0, h.div_euclid(16), 0), h)
}

fn cube(c: BrickCoord, r: i64) -> Vec<BrickCoord> {
    let mut v = Vec::new();
    for z in -r..=r {
        for y in -r..=r {
            for x in -r..=r {
                v.push(BrickCoord::new(c.x + x, c.y + y, c.z + z));
            }
        }
    }
    v
}

/// A world fixture whose bricks within `radius` of `centre` were generated and loaded the way a
/// materialiser would do it (WGEN-03 is not built: the tests call the system-commit port
/// directly, in a fixed order).
fn generated_world(
    journal: Option<Arc<dyn OperationJournal>>,
    centre: BrickCoord,
    radius: i64,
) -> Fixture {
    let f = fixture_with(Options {
        backends: vec![VoxelBackend::arc()],
        journal_port: journal,
        with_payload: true,
        ..Options::default()
    });
    for (n, b) in cube(centre, radius).into_iter().enumerate() {
        let g = generate_brick(SEED, current_version(), b).unwrap();
        f.system
            .materialise(
                OperationId::new(5, n as u64 + 1),
                LayerId::new(L_VOXEL),
                0,
                b,
                g.into_change(),
            )
            .unwrap();
    }
    f
}

struct ClientSink(Arc<ClientWorld>);

impl DirtySink for ClientSink {
    fn dirty(&self, regions: &[DirtyRegion]) {
        self.0.on_dirty(regions);
    }
}

/// Executor, client and the wiring from commits to the client.
fn client_for(f: &Fixture) -> (Arc<Executor>, Arc<ClientWorld>) {
    let mut r = ProviderRegistry::new();
    r.register(Arc::new(FacesProvider)).unwrap();
    let e = Arc::new(Executor::new(
        f.reader.clone(),
        r.freeze().unwrap(),
        ExecutorConfig::default(),
        Arc::new(ManualClock::new(0)),
    ));
    let c = Arc::new(ClientWorld::new(
        ClientConfig {
            layer: LayerId::new(L_VOXEL),
            level: 0,
            kind: FACES_KIND,
            max_active_bricks: 125,
            work_per_update: 1000,
        },
        e.clone(),
    ));
    f.sink.forward_to(Arc::new(ClientSink(c.clone())));
    (e, c)
}

/// Visible faces of the bricks in `active`, by brute force over the generator's voxel function,
/// with the voxels in `removed` turned to air.
fn oracle_faces(active: &[BrickCoord], removed: &BTreeSet<[i64; 3]>) -> usize {
    let mat = |x: i64, y: i64, z: i64| -> u16 {
        if removed.contains(&[x, y, z]) {
            0
        } else {
            voxel_at(SEED, current_version(), x, y, z).unwrap()
        }
    };
    let mut n = 0;
    for b in active {
        let o = b.origin().unwrap();
        for z in o.z..o.z + 16 {
            for y in o.y..o.y + 16 {
                for x in o.x..o.x + 16 {
                    if mat(x, y, z) == 0 {
                        continue;
                    }
                    for (dx, dy, dz) in [
                        (1, 0, 0),
                        (-1, 0, 0),
                        (0, 1, 0),
                        (0, -1, 0),
                        (0, 0, 1),
                        (0, 0, -1),
                    ] {
                        if mat(x + dx, y + dy, z + dz) == 0 {
                            n += 1;
                        }
                    }
                }
            }
        }
    }
    n
}

fn shown_faces(c: &ClientWorld, e: &Executor) -> usize {
    c.active()
        .iter()
        .map(|b| {
            let t = cube_processing::DerivedTarget {
                kind: FACES_KIND,
                key: cube_core::CellKey::new(LayerId::new(L_VOXEL), 0, *b).unwrap(),
            };
            e.get(t)
                .unwrap()
                .expect("current")
                .downcast_ref::<FacesMesh>()
                .unwrap()
                .faces
                .len()
        })
        .sum()
}

fn remove_op(n: u64, p: [i64; 3]) -> EditingOperation {
    EditingOperation {
        id: OperationId::new(31, n),
        kind: kinds::REMOVE_GEOMETRY,
        asset: cube_core::AssetId::new(ASSET),
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
fn a_generated_world_reaches_the_client_as_geometry_that_matches_the_oracle_and_follows_an_edit() {
    let (centre, h) = surface();
    // Loaded one brick beyond the region the client will move to, so every face is decided
    // with a real neighbour.
    let f = generated_world(None, centre, 3);
    let (e, c) = client_for(&f);
    c.start().unwrap();
    let change = c.set_interest(centre, 1).unwrap();
    assert_eq!(change.entered.len(), 27);

    // 1. The first update turns generated bricks into faces for the whole active region.
    let rep = c.update().unwrap();
    assert_eq!(
        (rep.ready, rep.pending, rep.failed.len()),
        (27, 0, 0),
        "{rep:?}"
    );
    let batch = c.take_batch();
    assert_eq!((batch.upsert.len(), batch.remove.len()), (27, 0));
    let faces: usize = batch
        .upsert
        .iter()
        .map(|(_, p)| p.downcast_ref::<FacesMesh>().unwrap().faces.len())
        .sum();
    assert!(faces > 200, "a real surface: {faces} faces");
    let active = c.active();
    assert_eq!(
        faces,
        oracle_faces(&active, &BTreeSet::new()),
        "equals the oracle"
    );
    assert!(
        c.take_batch().is_empty(),
        "nothing new until something changes"
    );

    // 2. The same seed gives the same world and the same geometry (determinism end to end).
    let f2 = generated_world(None, centre, 3);
    assert_eq!(hash(&f), hash(&f2));

    // 3. An edit through the real chain updates exactly the geometry that depends on it.
    let before = f.commits();
    assert!(f.run(vec![remove_op(1, [0, h, 0])]).is_success());
    assert_eq!(f.commits(), before + 1);
    let rep = c.update().unwrap();
    assert_eq!((rep.pending, rep.failed.len()), (0, 0), "{rep:?}");
    let b = c.take_batch();
    assert!(!b.upsert.is_empty(), "the dirty brick is replaced");
    assert!(b.upsert.iter().any(|(k, _)| k.brick() == centre));
    let removed: BTreeSet<[i64; 3]> = [[0, h, 0]].into();
    assert_eq!(
        shown_faces(&c, &e),
        oracle_faces(&active, &removed),
        "after the edit the geometry equals the oracle with the block removed"
    );

    // 4. Moving the active region streams bricks in and out and keeps the geometry right.
    let moved = BrickCoord::new(centre.x + 1, centre.y, centre.z);
    let ch = c.set_interest(moved, 1).unwrap();
    assert_eq!((ch.entered.len(), ch.left.len()), (9, 9));
    c.update().unwrap();
    let b = c.take_batch();
    assert_eq!((b.upsert.len(), b.remove.len()), (9, 9));
    assert_eq!(shown_faces(&c, &e), oracle_faces(&c.active(), &removed));
    c.stop().unwrap();
}

#[test]
fn an_edited_world_is_restored_from_the_seed_and_the_journal_and_the_client_shows_the_same() {
    let dir = std::env::temp_dir().join(format!("cube-first-world-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let limits = Limits::default();
    let (centre, h) = surface();
    let mut st = DirStorage::open(&dir).unwrap();
    // The world record (PERS-08) does not exist yet: the seed lives in this test; the generator
    // version is the one the manifest carries.
    create_world(&mut st, current_version().raw(), &limits).unwrap();

    // Life 1: generate, edit, "kill".
    let w = open_world(DirStorage::open(&dir).unwrap(), &limits).unwrap();
    assert_eq!(w.manifest.generator_version, current_version().raw());
    let j: Arc<dyn OperationJournal> = Arc::new(w.journal);
    let f = generated_world(Some(j.clone()), centre, 2);
    assert!(f.run(vec![remove_op(1, [0, h, 0])]).is_success());
    assert!(
        f.run(vec![remove_op(2, [1, v1::height(SEED, 1, 0), 0])])
            .is_success()
    );
    let truth = hash(&f);
    drop((f, j));

    // Life 2: regenerate the untouched base from the seed, replay what the user changed.
    let w = open_world(DirStorage::open(&dir).unwrap(), &limits).unwrap();
    let entries = w.journal.read_from(JournalSeq(0), usize::MAX).unwrap();
    let f = generated_world(Some(Arc::new(w.journal)), centre, 2);
    assert_ne!(hash(&f), truth, "the base alone is not the edited world");
    let rep = replay_confirmed(&entries, &f.recovery, &limits);
    assert!(rep.failed.is_empty(), "{rep:?}");
    assert_eq!(rep.replayed.len(), 2);
    assert_eq!(
        hash(&f),
        truth,
        "seed plus journal gives the exact edited world"
    );

    // The client shows the edited world.
    let (e, c) = client_for(&f);
    c.start().unwrap();
    c.set_interest(centre, 1).unwrap();
    let r = c.update().unwrap();
    assert_eq!((r.ready, r.failed.len()), (27, 0));
    let removed: BTreeSet<[i64; 3]> = [[0, h, 0], [1, v1::height(SEED, 1, 0), 0]].into();
    assert_eq!(shown_faces(&c, &e), oracle_faces(&c.active(), &removed));
    let _ = std::fs::remove_dir_all(&dir);
}
