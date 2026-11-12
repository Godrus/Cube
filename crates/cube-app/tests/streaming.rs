//! Streaming with unloading, through the real local host, Core, Processing and journal (headless):
//! far bricks are unloaded and come back identical, edited bricks stay, the resident set stops
//! growing, revisions stay consistent across unload, reload and restart, and concurrent requests
//! never generate a brick twice.
use std::path::PathBuf;
use std::sync::Arc;

use cube_app::{Session, StreamBudget};
use cube_client::{DeviceClass, Intent, ViewProfile};
use cube_core::{BrickCoord, GeneratorVersion, LayerId, WorldSeed};
use cube_persistence::WorldRecord;
use cube_server::{EditRequest, LocalHost, StaticArea, VOXEL_LAYER};

fn dir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("cube-stream-{}-{tag}", std::process::id()));
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

fn revision(host: &LocalHost, b: BrickCoord) -> Option<u64> {
    host.reader()
        .snapshot()
        .cell_token(LayerId::new(VOXEL_LAYER), 0, b)
        .map(|t| t.value().raw())
}

/// Brick 10 east of the origin column, at the terrain height.
fn far(host: &LocalHost) -> BrickCoord {
    let h = host.surface_height(160, 0);
    BrickCoord::new(10, h.div_euclid(16), 0)
}

fn origin(host: &LocalHost) -> BrickCoord {
    host.area().centre
}

#[test]
fn far_bricks_are_unloaded_and_come_back_identical() {
    let d = dir("a");
    let rec = record();
    let host = LocalHost::create(&d, rec, StaticArea::around_origin(&rec, 1)).unwrap();
    let f = far(&host);
    host.ensure_loaded(f, 1, usize::MAX).unwrap();
    host.ensure_loaded(origin(&host), 1, usize::MAX).unwrap();
    let resident = host.loaded_bricks();
    let hash = host.content_hash();
    assert_eq!(revision(&host, f), Some(1));

    let gone = host.evict_far(origin(&host), 3, 1, usize::MAX).unwrap();
    assert!(gone >= 27, "the far cube was unloaded: {gone}");
    assert_eq!(host.loaded_bricks(), resident - gone);
    assert_ne!(
        host.content_hash(),
        hash,
        "the world now holds fewer bricks"
    );

    host.ensure_loaded(f, 1, usize::MAX).unwrap();
    assert_eq!(host.loaded_bricks(), resident);
    assert_eq!(
        host.content_hash(),
        hash,
        "the regenerated base is identical"
    );
    assert_eq!(
        revision(&host, f),
        Some(2),
        "a reload is one more revision, never a reset"
    );
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn an_edit_after_unload_and_reload_survives_a_restart_with_the_right_revision() {
    let d = dir("b");
    let rec = record();
    let area = StaticArea::around_origin(&rec, 1);
    let host = LocalHost::create(&d, rec, area).unwrap();
    let f = far(&host);
    // Two full unload and load cycles of the far brick, then an edit on it.
    for _ in 0..2 {
        host.ensure_loaded(f, 1, usize::MAX).unwrap();
        host.evict_far(origin(&host), 3, 1, usize::MAX).unwrap();
    }
    host.ensure_loaded(f, 1, usize::MAX).unwrap();
    let rev_before_edit = revision(&host, f).unwrap();
    assert_eq!(rev_before_edit, 3);
    let h = host.surface_height(160, 0);
    host.submit_edit(EditRequest::Remove {
        voxel: [160, h - 1, 0],
    })
    .unwrap();
    let edited_rev = revision(&host, f).unwrap();
    assert_eq!(edited_rev, 4);
    let hash = host.content_hash();
    let resident = host.loaded_bricks();
    drop(host); // no clean shutdown: the journal alone must be enough

    let host = LocalHost::open(&d, area).unwrap();
    assert_eq!(
        revision(&host, f),
        Some(edited_rev),
        "same revision after replay"
    );
    host.ensure_loaded(f, 1, usize::MAX).unwrap();
    assert_eq!(host.loaded_bricks(), resident);
    assert_eq!(host.content_hash(), hash, "the edit is still there");
    // And it keeps working: edit again, restart again.
    host.submit_edit(EditRequest::Remove {
        voxel: [160, h - 2, 0],
    })
    .unwrap();
    let hash2 = host.content_hash();
    drop(host);
    let host = LocalHost::open(&d, area).unwrap();
    host.ensure_loaded(f, 1, usize::MAX).unwrap();
    assert_eq!(host.content_hash(), hash2);
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn edited_bricks_are_never_unloaded() {
    let d = dir("c");
    let rec = record();
    let host = LocalHost::create(&d, rec, StaticArea::around_origin(&rec, 1)).unwrap();
    let f = far(&host);
    host.ensure_loaded(f, 1, usize::MAX).unwrap();
    let h = host.surface_height(160, 0);
    host.submit_edit(EditRequest::Remove {
        voxel: [160, h - 1, 0],
    })
    .unwrap();
    let hash_with_edit = host.content_hash();
    host.evict_far(origin(&host), 0, 0, 0).unwrap(); // the most aggressive setting
    assert!(host.stream_stats().edited >= 1);
    assert!(
        host.reader()
            .snapshot()
            .cell_token(LayerId::new(VOXEL_LAYER), 0, f)
            .is_some(),
        "the edited brick is still there"
    );
    // The edit is in the resident world: nothing about it was lost by the unloading around it.
    assert_ne!(
        host.content_hash(),
        hash_with_edit,
        "other bricks went, the edited one stayed"
    );
    host.ensure_loaded(f, 1, usize::MAX).unwrap();
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn resident_bricks_stop_growing_during_a_long_walk_there_and_back() {
    let d = dir("d");
    let rec = record();
    let host = LocalHost::create(&d, rec, StaticArea::around_origin(&rec, 1)).unwrap();
    let (load, evict, protect, cap) = (4, 6, 4, 1500);
    let y = origin(&host).y;
    let path: Vec<i64> = (0..=40).chain((0..40).rev()).collect();
    let mut samples = Vec::new();
    for (i, x) in path.iter().enumerate() {
        let c = BrickCoord::new(*x, y, 0);
        while host.ensure_loaded(c, load, 48).unwrap() > 0 {}
        host.evict_far(c, evict, protect, cap).unwrap();
        samples.push(host.loaded_bricks());
        assert!(
            host.loaded_bricks() <= cap + 729,
            "step {i}: {}",
            host.loaded_bricks()
        );
    }
    let max = *samples.iter().max().unwrap();
    let first_half_max = *samples[..40].iter().max().unwrap();
    let second_half_max = *samples[40..].iter().max().unwrap();
    assert!(
        second_half_max <= first_half_max,
        "no growth on the way back: {first_half_max} -> {second_half_max}"
    );
    let stats = host.stream_stats();
    assert!(
        stats.loads > 2 * max as u64,
        "many more bricks were generated ({}) than ever resident ({max})",
        stats.loads
    );
    assert!(stats.evictions > 0);
    // Back at the start the original area is whole again and nothing leaked: the world around the
    // origin equals a fresh host's.
    let fresh_dir = dir("d2");
    let fresh = LocalHost::create(&fresh_dir, rec, StaticArea::around_origin(&rec, 1)).unwrap();
    let o = BrickCoord::new(0, y, 0);
    while fresh.ensure_loaded(o, load, 48).unwrap() > 0 {}
    while host.ensure_loaded(o, load, 48).unwrap() > 0 {}
    // Same resident set on both (the host kept some hysteresis bricks the fresh one never had).
    host.evict_far(o, load, load, cap).unwrap();
    fresh.evict_far(o, load, load, cap).unwrap();
    assert_eq!(
        host.content_hash(),
        fresh.content_hash(),
        "walking away and back changed nothing"
    );
    let _ = std::fs::remove_dir_all(&d);
    let _ = std::fs::remove_dir_all(&fresh_dir);
}

#[test]
fn concurrent_requests_never_generate_a_brick_twice() {
    let d = dir("e");
    let rec = record();
    let host = Arc::new(LocalHost::create(&d, rec, StaticArea::around_origin(&rec, 1)).unwrap());
    let c = BrickCoord::new(5, origin(&host).y, 5);
    let before = host.stream_stats();
    let handles: Vec<_> = (0..4)
        .map(|_| {
            let h = host.clone();
            std::thread::spawn(move || while h.ensure_loaded(c, 2, 16).unwrap() > 0 {})
        })
        .collect();
    for t in handles {
        t.join().unwrap();
    }
    let after = host.stream_stats();
    let new = (after.resident - before.resident) as u64;
    assert_eq!(
        after.loads - before.loads,
        new,
        "each new brick was generated exactly once"
    );
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn a_session_walking_far_keeps_residency_and_the_derived_cache_bounded() {
    let d = dir("f");
    let rec = record();
    let host = Arc::new(LocalHost::create(&d, rec, StaticArea::around_origin(&rec, 2)).unwrap());
    let top = (host.area().centre.y * 16 + 8) as f64;
    let s = Session::new(
        host.clone(),
        ViewProfile::of(DeviceClass::Desktop),
        [0.5, top + 12.0, 0.5],
    )
    .unwrap();
    // The frames here simulate 0.5 s each but run in milliseconds of real time, far faster than
    // any player: time slicing is off so the test measures residency, not generation speed.
    s.set_budget(StreamBudget {
        max_resident: 900,
        load_time_us: 0,
        ..StreamBudget::default()
    });
    let mut peak = 0u64;
    let mut peak_cache = 0u64;
    let mut shown = 0usize;
    // Fly east 40 bricks and back (speed 12 v/s: 0.5 s frames are 6 voxels each).
    for leg in [1i8, -1i8] {
        for _ in 0..110 {
            let f = s
                .frame(&[(
                    Intent::Move {
                        forward: 0,
                        right: leg,
                        up: 0,
                    },
                    0.5,
                )])
                .unwrap();
            shown += f.upsert.len();
            peak = peak.max(s.stats().resident);
            peak_cache = peak_cache.max(s.stats().derived_cached);
        }
    }
    let st = s.stats();
    assert!(shown > 300, "geometry kept arriving: {shown}");
    assert!(st.evictions > 0 && st.loads > st.resident, "{st:?}");
    assert!(peak <= 900 + 729, "resident peaked at {peak}");
    assert!(
        peak_cache <= 1500,
        "the derived cache stays bounded: {peak_cache}"
    );
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn time_sliced_loading_makes_one_chunk_per_frame_and_still_loads_everything() {
    // M1-A3c: with a time limit of one microsecond every frame ends its generation after the
    // first chunk (two bricks), however many bricks the frame budget would still allow.
    let d = dir("slice");
    let rec = record();
    let host = Arc::new(LocalHost::create(&d, rec, StaticArea::around_origin(&rec, 2)).unwrap());
    let top = (host.area().centre.y * 16 + 8) as f64;
    let s = Session::new(
        host.clone(),
        ViewProfile::of(DeviceClass::Desktop),
        [0.5, top + 12.0, 0.5],
    )
    .unwrap();
    s.set_budget(StreamBudget {
        load_time_us: 1,
        load_chunk: 2,
        ..StreamBudget::default()
    });
    // The first region is always generated in full, so start from a settled world.
    s.frame(&[]).unwrap();
    let mut max_per_frame = 0u64;
    let frame = |intents: &[(Intent, f32)]| {
        let before = host.stream_stats().loads;
        s.frame(intents).unwrap();
        host.stream_stats().loads - before
    };
    // Fly east: far more bricks are needed than two per frame.
    let east = [(
        Intent::Move {
            forward: 0,
            right: 1,
            up: 0,
        },
        0.5,
    )];
    for _ in 0..60 {
        max_per_frame = max_per_frame.max(frame(&east));
    }
    assert!(
        max_per_frame <= 2,
        "one chunk per frame, got {max_per_frame} bricks in a frame"
    );
    // Standing still, the backlog drains one chunk per frame until nothing is missing.
    let (mut quiet, mut frames) = (0, 0);
    while quiet < 3 && frames < 3000 {
        let made = frame(&[]);
        max_per_frame = max_per_frame.max(made);
        quiet = if made == 0 { quiet + 1 } else { 0 };
        frames += 1;
    }
    assert!(quiet >= 3, "the backlog drained within {frames} frames");
    assert!(
        max_per_frame <= 2,
        "never more than a chunk: {max_per_frame}"
    );
}
