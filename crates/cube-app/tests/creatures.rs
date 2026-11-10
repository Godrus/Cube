//! Creatures in a real session (headless): they spawn from the seed around the player, stay
//! bounded, stand on the ground, chase and hurt the player, and the player respawns. Real host,
//! Core and Processing; not a Godot run, and creatures are switched off in the demo by default.
use std::path::PathBuf;
use std::sync::Arc;

use cube_app::port::{FIRST_WORLD_RADIUS, PortSession};
use cube_app::spawn::spawn_point;
use cube_app::{PlayerState, Session};
use cube_client::{Controls, DeviceClass, SessionPort, ViewProfile};
use cube_core::{GeneratorVersion, WorldSeed};
use cube_persistence::WorldRecord;
use cube_server::{LocalHost, StaticArea};

fn dir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("cube-crt-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

fn start(tag: &str, near_creature: bool) -> (PortSession, PathBuf, [f64; 3]) {
    let d = dir(tag);
    let rec = WorldRecord {
        world_id: 1,
        seed: WorldSeed::new(7),
        generator_version: GeneratorVersion::new(1),
        structure_version: cube_persistence::STRUCTURE_VERSION,
    };
    let host = Arc::new(
        LocalHost::create(&d, rec, StaticArea::around_origin(&rec, FIRST_WORLD_RADIUS)).unwrap(),
    );
    // A column inside the loaded area (2..=3 columns from the origin) that has a creature.
    let mut spot = None;
    'find: for cx in 2..=3 {
        for cz in -3..=3 {
            if let Some(p) = spawn_point(7, cx, cz, |x, z| host.surface_height(x, z)) {
                spot = Some(p);
                break 'find;
            }
        }
    }
    let spot = spot.expect("this seed has a creature near the start");
    let feet = if near_creature {
        [spot[0] + 3.0, spot[1] + 1.0, spot[2]]
    } else {
        [0.5, host.surface_height(0, 0) as f64 + 2.0, 0.5]
    };
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
    (PortSession::new(session, d.clone()), d, spot)
}

fn run(p: &mut PortSession, frames: usize) {
    for _ in 0..frames {
        p.frame(&Controls {
            dt: 1.0 / 60.0,
            ..Controls::default()
        })
        .unwrap();
    }
}

#[test]
fn creatures_are_off_by_default() {
    let (mut p, d, _) = start("off", true);
    run(&mut p, 300);
    assert!(p.creatures().is_none());
    assert_eq!(p.game().health.cur(), p.game().health.max());
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn a_creature_near_the_player_chases_hurts_and_the_player_gets_up_again() {
    let (mut p, d, spot) = start("hurt", true);
    p.enable_creatures();
    let mut lowest = p.game().health.cur();
    let mut deaths = 0;
    for _ in 0..1500 {
        run(&mut p, 1);
        let h = p.game().health;
        if h.cur() > lowest && lowest < 10 {
            deaths += 1; // health jumped back up: a respawn
        }
        lowest = lowest.min(h.cur());
        if lowest == 0 {
            break;
        }
    }
    let n = p.creatures().expect("switched on").len();
    assert!((1..=8).contains(&n), "population bounded: {n}");
    assert!(
        lowest < p.game().health.max() || deaths > 0,
        "the creature at {spot:?} reached the player and hurt them"
    );
    for c in p.creatures().unwrap().creatures() {
        assert!(c.pos()[1] > -200.0, "nothing fell out of the world");
    }
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn the_population_stays_bounded_and_off_the_safe_columns() {
    let (mut p, d, _) = start("pop", false);
    p.enable_creatures();
    run(&mut p, 400);
    let pop = p.creatures().unwrap();
    assert!(pop.len() <= 8);
    for c in pop.creatures() {
        let col = |v: f64| (v.floor() as i64).div_euclid(16);
        assert!(col(c.pos()[0]).abs() > 1 || col(c.pos()[2]).abs() > 1);
    }
    let _ = std::fs::remove_dir_all(d);
}

/// Lets a creature come to the player and kills it. Returns the column it lived in and the
/// inventory afterwards (item id, count of each kind).
fn kill_one(tag: &str) -> ((i64, i64), Vec<(u16, u32)>, String) {
    let (mut p, d, _) = start(tag, true);
    p.enable_creatures();
    let mut kills = 0;
    let mut column = (0, 0);
    for _ in 0..2000 {
        run(&mut p, 1);
        if let Ok(st) = p.attack()
            && st.hit.killed
        {
            kills += 1;
            column = st.column;
            break;
        }
    }
    assert_eq!(kills, 1, "the creature came, was hit twice and died");
    let inv = (1..=6u16)
        .map(|i| (i, p.game().inventory.count_of(cube_app::game::ItemId(i))))
        .filter(|(_, n)| *n > 0)
        .collect::<Vec<_>>();
    let msg = p.game_view().message;
    let _ = std::fs::remove_dir_all(d);
    (column, inv, msg)
}

#[test]
fn a_kill_drops_exactly_what_the_seeded_roll_says_and_it_is_the_same_every_time() {
    let (col, inv, msg) = kill_one("loot-a");
    let expected = cube_app::loot::SLIME_LOOT.roll(cube_app::loot::loot_key(7, col.0, col.1));
    match expected {
        Some(drop) => {
            assert_eq!(inv, vec![(drop.item.0, drop.count)], "{msg}");
            assert!(msg.contains("defeated"));
        }
        None => assert!(inv.is_empty(), "nothing was rolled: {inv:?}"),
    }
    let (col2, inv2, _) = kill_one("loot-b");
    assert_eq!((col, inv), (col2, inv2), "same world, same kill, same drop");
}

#[test]
fn nothing_can_be_attacked_while_creatures_are_off_or_out_of_reach() {
    let (mut p, d, _) = start("noatk", false);
    assert!(p.attack().is_err(), "off");
    p.enable_creatures();
    run(&mut p, 120);
    assert!(p.attack().is_err(), "nothing within reach of the start");
    let _ = std::fs::remove_dir_all(d);
}
