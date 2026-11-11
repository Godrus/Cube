//! The altar cache of a ruin (headless): taken once, the same every time for the world, saved
//! with the kill-style single file write, and nothing happens away from an altar.
use std::path::PathBuf;
use std::sync::Arc;

use cube_app::port::{FIRST_WORLD_RADIUS, PortSession};
use cube_app::{PlayerState, Session};
use cube_client::{DeviceClass, ViewProfile};
use cube_core::{GeneratorVersion, WorldSeed};
use cube_persistence::WorldRecord;
use cube_server::{LocalHost, StaticArea};

fn start(tag: &str, offset: f64) -> (PortSession, PathBuf, (i64, i64)) {
    let d = std::env::temp_dir().join(format!("cube-ruin-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    let rec = WorldRecord {
        world_id: 1,
        seed: WorldSeed::new(7),
        generator_version: GeneratorVersion::new(5),
        structure_version: cube_persistence::STRUCTURE_VERSION,
    };
    // The pinned ruin of seed 7 (see cube-integration-tests worldgen_v5): centre (-95, -29), floor -10.
    let ruin = (-95_i64, -29_i64, -10_i64);
    let host = Arc::new(
        LocalHost::create(&d, rec, StaticArea::around_origin(&rec, FIRST_WORLD_RADIUS)).unwrap(),
    );
    let feet = [
        ruin.0 as f64 + 0.5 + offset,
        (ruin.2 + 1) as f64,
        ruin.1 as f64 + 0.5,
    ];
    let session = Session::new(
        host,
        ViewProfile::of(DeviceClass::Desktop),
        [feet[0], feet[1] + 1.6, feet[2]],
    )
    .unwrap();
    session.restore_player(&PlayerState {
        feet,
        yaw: 0.0,
        pitch: 0.0,
        walking: true,
        third_person: false,
    });
    (PortSession::new(session, d.clone()), d, (ruin.0, ruin.1))
}

#[test]
fn the_cache_is_taken_once_saved_and_gives_experience() {
    let (mut p, d, site) = start("once", 1.0);
    let before = p.game().progress.xp();
    let t = p.interact();
    assert!(t.starts_with("Altar cache:") && t.contains("XP"), "{t}");
    assert_eq!(
        p.game().progress.xp(),
        before + cube_app::progression::RUIN_XP
    );
    assert!(p.game().looted.contains(&site));
    let items: u32 = (1..=12u16)
        .map(|i| p.game().inventory.count_of(cube_app::game::ItemId(i)))
        .sum();
    assert!(items >= 4, "two rolls of at least two");
    // Again: nothing more.
    assert_eq!(p.interact(), "The altar is empty");
    assert_eq!(
        p.game().progress.xp(),
        before + cube_app::progression::RUIN_XP
    );
    // A crash right after: the cache, the items and the experience are on disk together.
    drop(p);
    let saved = cube_app::port::load_game(&d).expect("written when it happened");
    assert!(saved.looted.contains(&site));
    assert_eq!(saved.progress.xp(), cube_app::progression::RUIN_XP);
    let saved_items: u32 = (1..=12u16)
        .map(|i| saved.inventory.count_of(cube_app::game::ItemId(i)))
        .sum();
    assert_eq!(saved_items, items);
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn nothing_happens_away_from_an_altar() {
    let (mut p, d, _) = start("far", 8.0);
    assert_eq!(p.interact(), "");
    assert!(p.game().looted.is_empty());
    assert_eq!(p.game().progress.xp(), 0);
    let _ = std::fs::remove_dir_all(d);
}
