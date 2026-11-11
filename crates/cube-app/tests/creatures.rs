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
    start_with(tag, near_creature, cube_app::game::GameState::default())
}

fn start_with(
    tag: &str,
    near_creature: bool,
    game: cube_app::game::GameState,
) -> (PortSession, PathBuf, [f64; 3]) {
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
    (PortSession::with_game(session, d.clone(), game), d, spot)
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

#[test]
fn a_kill_and_its_loot_survive_a_crash_and_restart_exactly_once() {
    let (mut p, d, spot) = start("persist", true);
    p.enable_creatures();
    let mut column = None;
    for _ in 0..2000 {
        run(&mut p, 1);
        if let Ok(st) = p.attack()
            && st.hit.killed
        {
            column = Some(st.column);
            break;
        }
    }
    let column = column.expect("killed");
    let items = |g: &cube_app::game::GameState| {
        (1..=6u16)
            .map(|i| (i, g.inventory.count_of(cube_app::game::ItemId(i))))
            .filter(|(_, n)| *n > 0)
            .collect::<Vec<_>>()
    };
    let before = items(p.game());
    drop(p); // a crash: no shutdown, no final save
    let saved = cube_app::port::load_game(&d).expect("the kill was written when it happened");
    assert!(saved.killed.contains(&column));
    assert_eq!(items(&saved), before, "kill and loot saved together");

    // Restart on the same files.
    let rec = WorldRecord {
        world_id: 1,
        seed: WorldSeed::new(7),
        generator_version: GeneratorVersion::new(1),
        structure_version: cube_persistence::STRUCTURE_VERSION,
    };
    let host =
        Arc::new(LocalHost::open(&d, StaticArea::around_origin(&rec, FIRST_WORLD_RADIUS)).unwrap());
    let feet = [spot[0] + 3.0, spot[1] + 1.0, spot[2]];
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
    let mut p2 = PortSession::with_game(session, d.clone(), saved);
    p2.enable_creatures();
    for _ in 0..600 {
        run(&mut p2, 1);
        assert!(
            p2.attack().map_or(true, |s| s.column != column),
            "it did not come back"
        );
    }
    assert!(p2.creatures().unwrap().creatures().all(|c| {
        let col = |v: f64| (v.floor() as i64).div_euclid(16);
        (col(c.pos()[0]), col(c.pos()[2])) != column
    }));
    assert_eq!(items(p2.game()), before, "no second drop");
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn the_port_view_equals_the_model() {
    let (mut p, d, _) = start("view", true);
    assert!(SessionPort::creatures(&p).is_empty(), "off: none shown");
    p.enable_creatures();
    let mut attacked = false;
    for _ in 0..2000 {
        run(&mut p, 1);
        let v = p.game_view();
        assert_eq!(
            (v.health, v.max_health),
            (p.game().health.cur(), p.game().health.max())
        );
        let shown = SessionPort::creatures(&p);
        assert_eq!(shown.len(), p.creatures().unwrap().len());
        for (a, b) in shown.iter().zip(p.creatures().unwrap().creatures()) {
            assert_eq!(
                (a.pos, a.health, a.max_health),
                (b.pos(), b.health().cur(), b.health().max())
            );
        }
        if let Ok(true) = SessionPort::attack(&mut p) {
            attacked = true;
            break;
        }
    }
    assert!(
        attacked,
        "a creature came into reach and the port attack hit it"
    );
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn swinging_again_and_again_never_duplicates_a_kill_or_its_loot() {
    let (mut p, d, _) = start("dup", true);
    p.enable_creatures();
    let mut killed: Vec<(i64, i64)> = Vec::new();
    for _ in 0..2500 {
        run(&mut p, 1);
        // A swing every frame: most are refused (cooldown, nothing in reach) and change nothing.
        if let Ok(st) = p.attack()
            && st.hit.killed
        {
            killed.push(st.column);
        }
    }
    assert!(!killed.is_empty(), "at least one kill happened");
    let mut unique = killed.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), killed.len(), "no column was killed twice");
    let mut expected = std::collections::BTreeMap::<u16, u32>::new();
    for c in &killed {
        if let Some(drop) = cube_app::loot::SLIME_LOOT.roll(cube_app::loot::loot_key(7, c.0, c.1)) {
            *expected.entry(drop.item.0).or_default() += drop.count;
        }
    }
    let got: std::collections::BTreeMap<u16, u32> = (1..=6u16)
        .map(|i| (i, p.game().inventory.count_of(cube_app::game::ItemId(i))))
        .filter(|(_, n)| *n > 0)
        .collect();
    assert_eq!(
        got, expected,
        "the inventory is exactly the sum of the rolls of the kills"
    );
    assert_eq!(p.game().killed.len(), killed.len());
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn a_kill_gives_experience_that_is_saved_with_it_and_never_given_twice() {
    let (mut p, d, _) = start("xp", true);
    p.enable_creatures();
    assert_eq!(p.game().progress.xp(), 0);
    let mut kills = 0;
    for _ in 0..2000 {
        run(&mut p, 1);
        if let Ok(st) = p.attack()
            && st.hit.killed
        {
            kills += 1;
            break;
        }
    }
    assert_eq!(kills, 1);
    let xp = cube_app::creature::SLIME.xp;
    assert_eq!(p.game().progress.xp(), xp);
    assert!(p.game_view().message.contains("XP"));
    let v = p.game_view();
    assert_eq!((v.level, v.xp), (1, xp));
    assert_eq!(v.xp_next, cube_app::progression::xp_for_level(2));
    // Swinging at the corpse or at nothing gives nothing more.
    for _ in 0..200 {
        run(&mut p, 1);
        let _ = p.attack();
    }
    let got = p.game().progress.xp();
    assert_eq!(got % xp, 0);
    drop(p); // a crash: no shutdown save
    let saved = cube_app::port::load_game(&d).expect("written at the kill");
    assert_eq!(saved.progress.xp(), got, "kill, loot and XP in one write");
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn the_selected_iron_sword_kills_a_slime_in_one_hit_and_the_wooden_one_needs_two() {
    use cube_app::game::{GameState, IRON_SWORD_ITEM, ItemId};
    let hits_to_kill = |iron: bool, tag: &str| {
        let mut g = GameState::default();
        if iron {
            g.inventory.add(IRON_SWORD_ITEM, 1);
            g.inventory.select(0);
            assert_eq!(g.inventory.selected_item(), Some(IRON_SWORD_ITEM));
        } else {
            g.inventory.add(ItemId(1), 1); // a block selected: the wooden sword is used
        }
        let (mut p, d, _) = start_with(tag, true, g);
        p.enable_creatures();
        let mut hits = 0;
        for _ in 0..2000 {
            run(&mut p, 1);
            if let Ok(st) = p.attack() {
                hits += 1;
                if st.hit.killed {
                    break;
                }
            }
        }
        let _ = std::fs::remove_dir_all(d);
        hits
    };
    assert_eq!(hits_to_kill(true, "iron"), 1);
    assert_eq!(hits_to_kill(false, "wood"), 2);
}

#[test]
fn crafting_makes_the_sword_once_per_materials_and_saves_it() {
    use cube_app::game::{GameState, IRON_SWORD_ITEM, ItemId};
    let mut g = GameState::default();
    g.inventory.add(ItemId(12), 7); // iron ore
    g.inventory.add(ItemId(8), 2); // wood
    let (mut p, d, _) = start_with("craft", false, g);
    assert!(p.craft().starts_with("Crafted Iron sword"));
    assert_eq!(p.game().inventory.count_of(IRON_SWORD_ITEM), 1);
    assert_eq!(p.game().inventory.count_of(ItemId(12)), 4);
    assert_eq!(p.game().inventory.count_of(ItemId(8)), 0);
    assert!(p.craft().starts_with("Not enough"), "no wood left");
    drop(p); // a crash: no shutdown save
    let saved = cube_app::port::load_game(&d).expect("written at the craft");
    assert_eq!(saved.inventory.count_of(IRON_SWORD_ITEM), 1);
    assert_eq!(saved.inventory.count_of(ItemId(12)), 4);
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn crafted_armour_is_worn_from_the_inventory() {
    use cube_app::game::{GameState, IRON_ARMOUR_ITEM, ItemId, armour_of};
    let mut g = GameState::default();
    g.inventory.add(ItemId(12), 5); // iron ore
    g.inventory.add(ItemId(11), 2); // coal ore
    assert_eq!(armour_of(&g.inventory), 0);
    let (mut p, d, _) = start_with("armour", false, g);
    assert!(p.craft().starts_with("Crafted Iron armour"));
    assert_eq!(p.game().inventory.count_of(IRON_ARMOUR_ITEM), 1);
    assert_eq!(armour_of(&p.game().inventory), 1);
    let _ = std::fs::remove_dir_all(d);
}
