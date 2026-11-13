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
    start_at(tag, 5, [offset, 0.0, 0.0], 1)
}

/// `at` is an offset from the ruin centre column and `dy` the feet above the floor (1 = on it).
fn start_at(tag: &str, version: u32, at: [f64; 3], dy: i64) -> (PortSession, PathBuf, (i64, i64)) {
    start_with(tag, version, at, dy, None)
}

/// Like `start_at`, with a given game state (loaded as if from a save) and a kept directory when
/// `keep` names one: used to stop and start again in the same world.
fn start_with(
    tag: &str,
    version: u32,
    at: [f64; 3],
    dy: i64,
    game: Option<cube_app::game::GameState>,
) -> (PortSession, PathBuf, (i64, i64)) {
    let d = std::env::temp_dir().join(format!("cube-ruin-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    let rec = WorldRecord {
        world_id: 1,
        seed: WorldSeed::new(7),
        generator_version: GeneratorVersion::new(version),
        structure_version: cube_persistence::STRUCTURE_VERSION,
    };
    // The pinned ruin of seed 7 (see cube-integration-tests worldgen_v5): centre (-95, -29), floor -10.
    let ruin = (-95_i64, -29_i64, -10_i64);
    let host = Arc::new(
        LocalHost::create(&d, rec, StaticArea::around_origin(&rec, FIRST_WORLD_RADIUS)).unwrap(),
    );
    let feet = [
        ruin.0 as f64 + 0.5 + at[0],
        (ruin.2 + dy) as f64,
        ruin.1 as f64 + 0.5 + at[2],
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
    let port = match game {
        Some(g) => PortSession::with_game(session, d.clone(), g),
        None => PortSession::new(session, d.clone()),
    };
    (port, d, (ruin.0, ruin.1))
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

#[test]
fn the_deep_cache_under_the_ruin_is_bigger_taken_once_and_only_in_version_six() {
    // Feet on the chamber floor (7 below the ruin floor, standing one above it), beside the pedestal.
    let (mut p, d, site) = start_at("deep", 6, [0.0, 0.0, 3.0], -6);
    let t = p.interact();
    assert!(t.starts_with("Deep cache:"), "{t}");
    assert_eq!(p.game().progress.xp(), cube_app::progression::DEEP_XP);
    assert!(p.game().looted.contains(&(site.0, site.1 + 4)));
    assert_eq!(p.interact(), "The altar is empty");
    assert_eq!(p.game().progress.xp(), cube_app::progression::DEEP_XP);
    // The surface altar of the same ruin is a separate cache (version 6 keeps it).
    drop(p);
    let (mut p, d2, _) = start_at("surface6", 6, [1.0, 0.0, 0.0], 1);
    assert!(p.interact().starts_with("Altar cache:"));
    let _ = std::fs::remove_dir_all(d);
    let _ = std::fs::remove_dir_all(d2);
    // In a version 5 world there is no chamber, so nothing answers down there.
    let (mut p5, d5, _) = start_at("deep5", 5, [0.0, 0.0, 3.0], -6);
    assert_eq!(p5.interact(), "");
    let _ = std::fs::remove_dir_all(d5);
}

#[test]
fn what_does_not_fit_lies_on_the_ground_is_saved_and_is_picked_up_when_there_is_room() {
    use cube_app::game::{GameState, ItemId, MAX_STACK};
    use cube_client::{Controls, SessionPort};
    // A full inventory at the altar: the cache goes to the ground, not into the void.
    let mut full = GameState::default();
    full.inventory.add(ItemId(5), 27 * MAX_STACK);
    let (mut p, d, _) = start_with("ground", 5, [1.0, 0.0, 0.0], 1, Some(full));
    let t = p.interact();
    assert!(t.contains("lie on the ground"), "{t}");
    assert!(!t.contains("are lost"), "{t}");
    let on_ground: u32 = p.game().ground.iter().map(|g| g.count).sum();
    assert!(on_ground >= 4, "two rolls of at least two: {on_ground}");
    assert_eq!(
        p.ground_items().iter().map(|g| g.count).sum::<u32>(),
        on_ground,
        "the view shows the same stacks"
    );
    // The stacks are written with the cache: a crash right now keeps them.
    drop(p);
    let saved = cube_app::port::load_game(&d).expect("written when it happened");
    assert_eq!(saved.ground.iter().map(|g| g.count).sum::<u32>(), on_ground);
    // Start again with a free inventory at the same place: one frame and the stacks are taken.
    let mut roomy = saved;
    roomy.inventory = cube_app::game::Inventory::new();
    let (mut p, d2, _) = start_with("ground2", 5, [1.0, 0.0, 0.0], 1, Some(roomy));
    p.frame(&Controls {
        dt: 1.0 / 60.0,
        ..Controls::default()
    })
    .unwrap();
    assert!(p.game().ground.is_empty(), "picked up");
    let items: u32 = (1..=12u16)
        .map(|i| p.game().inventory.count_of(ItemId(i)))
        .sum();
    assert_eq!(items, on_ground);
    let _ = std::fs::remove_dir_all(d);
    let _ = std::fs::remove_dir_all(d2);
}
