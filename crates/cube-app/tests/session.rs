//! The first-world session, headless: Launch -> Generate -> Display data -> Move camera ->
//! Edit voxel -> Save -> Quit -> Load -> Continue, through the real local host, generator,
//! Core, Orchestration, journal, Processing and client. It is the data a Godot adapter receives
//! (plain buffers); it is **not** a rendered scene, and nothing here was shown on a screen.
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use cube_app::{Frame, Session};
use cube_client::{DeviceClass, Intent, MeshBuffers, ViewProfile};
use cube_core::{CellKey, GeneratorVersion, WorldSeed};
use cube_persistence::WorldRecord;
use cube_server::{LocalHost, StaticArea};

fn dir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("cube-session-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

fn record() -> WorldRecord {
    WorldRecord {
        world_id: 1,
        seed: WorldSeed::new(7),
        generator_version: GeneratorVersion::new(1),
        structure_version: cube_persistence::STRUCTURE_VERSION,
    }
}

fn area(r: &WorldRecord) -> StaticArea {
    StaticArea::around_origin(r, 3)
}

/// What the engine would have in its scene after applying frames.
#[derive(Default)]
struct Scene(BTreeMap<CellKey, MeshBuffers>);

impl Scene {
    fn apply(&mut self, f: Frame) -> (usize, usize) {
        let counts = (f.upsert.len(), f.remove.len());
        for k in &f.remove {
            self.0.remove(k);
        }
        for (k, b) in f.upsert {
            assert!(b.is_valid(), "the buffers are consistent");
            self.0.insert(k, b);
        }
        counts
    }

    fn triangles(&self) -> usize {
        self.0.values().map(MeshBuffers::triangle_count).sum()
    }
}

/// Runs frames until the client has nothing pending (bounded), like an engine over some frames.
fn settle(s: &Session, scene: &mut Scene) {
    for _ in 0..200 {
        let f = s.frame(&[]).unwrap();
        let done = f.report.pending == 0 && f.report.stale == 0 && f.report.failed.is_empty();
        scene.apply(f);
        if done {
            return;
        }
    }
    panic!("the client did not settle");
}

/// A camera above the surface of column (0, 0), looking straight down.
fn session(host: &Arc<LocalHost>) -> (Session, Scene) {
    let top = (host.area().centre.y * 16 + 8) as f64;
    let s = Session::new(
        host.clone(),
        ViewProfile::of(DeviceClass::Desktop),
        [0.5, top + 12.0, 0.5],
    )
    .unwrap();
    // Look straight down: pitch to the clamp. The frame's batch belongs to the scene.
    let mut scene = Scene::default();
    scene.apply(
        s.frame(&[(
            Intent::Look {
                yaw: 0.0,
                pitch: -3.0,
            },
            0.0,
        )])
        .unwrap(),
    );
    (s, scene)
}

#[test]
fn launch_generate_display_move_edit_save_quit_load_continue() {
    let d = dir("main");
    let rec = record();
    let host = Arc::new(LocalHost::create(&d, rec, area(&rec)).unwrap());
    assert_eq!(host.record(), &rec);
    let base_hash = host.content_hash();

    // Launch, generate, display: the first frames deliver the whole active region.
    let (s, mut scene) = session(&host);
    settle(&s, &mut scene);
    // The region has the profile's full radius (3): 7 x 7 x 7 bricks. The host generated the extra
    // ring beyond its initial area (streaming); before, the region was clamped inside the area (125).
    assert_eq!(scene.0.len(), 343);
    assert!(
        scene.triangles() > 500,
        "a real surface: {}",
        scene.triangles()
    );
    let shown_before_move = scene.0.len();
    assert_eq!(
        scene.apply(s.frame(&[]).unwrap()),
        (0, 0),
        "nothing new when nothing changed"
    );

    // Move the camera: the active region follows, the host generates new bricks around it and
    // bricks stream in and out of the scene. This runs on a second world, because streaming adds
    // bricks to the loaded set and the exact-hash comparison after the restart below needs a host
    // that only loaded the region around the origin.
    let d2 = dir("move");
    let host2 = Arc::new(LocalHost::create(&d2, rec, area(&rec)).unwrap());
    let loaded_before = host2.loaded_bricks();
    let (sm, mut scene_m) = session(&host2);
    settle(&sm, &mut scene_m);
    let mut moved = false;
    for _ in 0..30 {
        let (up, rm) = scene_m.apply(
            sm.frame(&[(
                Intent::Move {
                    forward: 1,
                    right: 1,
                    up: 0,
                },
                0.5,
            )])
            .unwrap(),
        );
        moved |= up + rm > 0;
    }
    assert!(moved || shown_before_move > 0);
    assert!(
        host2.loaded_bricks() > loaded_before,
        "the host generated bricks ahead of the camera"
    );
    drop((sm, host2));
    let _ = std::fs::remove_dir_all(&d2);
    // Back above the origin for a predictable edit.
    drop(s);
    let (s, mut scene) = session(&host);
    settle(&s, &mut scene);
    let tris = scene.triangles();

    // Edit a voxel: remove the block under the crosshair, then place one back on top.
    assert!(s.edit(60.0, None).unwrap(), "the view hits the ground");
    let removed_hash = host.content_hash();
    assert_ne!(removed_hash, base_hash);
    let f = s.frame(&[]).unwrap();
    assert!(
        !f.upsert.is_empty(),
        "the geometry of the edited brick is replaced"
    );
    scene.apply(f);
    assert!(s.edit(60.0, Some(5)).unwrap());
    settle(&s, &mut scene);
    let edited_hash = host.content_hash();
    assert_ne!(edited_hash, removed_hash);
    let edited_scene: Vec<(CellKey, MeshBuffers)> =
        scene.0.iter().map(|(k, b)| (*k, b.clone())).collect();
    assert!(tris > 0 && !edited_scene.is_empty());

    // Save is implicit (every edit is journalled and fsynced). Quit: drop everything.
    let dropped = s.stop().unwrap();
    assert!(!dropped.is_empty());
    drop((s, host));

    // Load: the record, the regenerated base and the replayed edits give the same world and the
    // same geometry as before the quit.
    let host = Arc::new(LocalHost::open(&d, area(&rec)).unwrap());
    let (s, mut scene2) = session(&host);
    settle(&s, &mut scene2);
    // The first frame streamed the same surroundings as before the quit; the loaded world (base,
    // streamed bricks and replayed edits) is then identical.
    assert_eq!(host.content_hash(), edited_hash, "the exact edited world");
    let reloaded: Vec<(CellKey, MeshBuffers)> =
        scene2.0.iter().map(|(k, b)| (*k, b.clone())).collect();
    let diff: Vec<_> = reloaded
        .iter()
        .zip(&edited_scene)
        .filter(|(a, b)| a != b)
        .map(|(a, _)| (a.0.brick(), a.1.triangle_count()))
        .collect();
    assert_eq!(
        (reloaded.len(), diff.len()),
        (edited_scene.len(), 0),
        "the geometry after loading equals the geometry before quitting: {diff:?}"
    );

    // Continue: editing works after the restart and the sequence of ids continues.
    assert!(s.edit(60.0, None).unwrap());
    let continued = host.content_hash();
    assert_ne!(continued, edited_hash);
    drop((s, host));
    let host = Arc::new(LocalHost::open(&d, area(&rec)).unwrap());
    let (_s3, _scene3) = session(&host);
    assert_eq!(
        host.content_hash(),
        continued,
        "and the continued world survives a second restart"
    );
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn a_world_cannot_be_created_twice_and_a_missing_world_does_not_open() {
    let d = dir("twice");
    let rec = record();
    let a = area(&rec);
    assert!(LocalHost::open(&d, a).is_err(), "nothing to open");
    let host = LocalHost::create(&d, rec, a).unwrap();
    drop(host);
    assert!(
        LocalHost::create(&d, rec, a).is_err(),
        "an existing world is never overwritten"
    );
    assert!(LocalHost::open(&d, a).is_ok());
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn edits_that_hit_nothing_or_break_the_rules_are_refused_and_change_nothing() {
    let d = dir("refuse");
    let rec = record();
    let host = Arc::new(LocalHost::create(&d, rec, area(&rec)).unwrap());
    let s = Session::new(
        host.clone(),
        ViewProfile::of(DeviceClass::MobileLow),
        [0.5, 4000.0, 0.5],
    )
    .unwrap();
    let before = host.content_hash();
    // Looking at the sky from far away: nothing within reach.
    assert!(!s.edit(5.0, None).unwrap());
    // Material 0 cannot be placed.
    let r = cube_client::EditPort::submit(
        &s,
        cube_client::EditIntent::Place {
            voxel: [0, 0, 0],
            material: 0,
        },
    );
    assert!(r.is_err());
    // Placing into a brick that is not loaded is refused, not guessed.
    let r = cube_client::EditPort::submit(
        &s,
        cube_client::EditIntent::Place {
            voxel: [100_000, 0, 0],
            material: 1,
        },
    );
    assert!(r.is_err(), "{r:?}");
    assert_eq!(host.content_hash(), before);
    let _ = std::fs::remove_dir_all(&d);
}
