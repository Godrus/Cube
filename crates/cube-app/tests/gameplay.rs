//! Mining, building, inventory and the first objective through the real host and session
//! (headless). Data, not a window: the Godot run of the same rules is `walk_smoke.py --play`.
use cube_app::game::{GameState, ItemId, MAX_STACK, Mode};
use cube_app::port::{self, GAME_FILE};
use cube_client::{Controls, SessionPort};
use std::path::PathBuf;

fn dir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("cube-play-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

fn tick(s: &mut Box<dyn SessionPort>, n: usize) {
    for _ in 0..n {
        s.frame(&Controls {
            dt: 1.0 / 60.0,
            ..Controls::default()
        })
        .unwrap();
    }
}

/// A world with the player standing on the ground, looking at the ground ahead.
fn start(tag: &str) -> (Box<dyn SessionPort>, PathBuf) {
    let d = dir(tag);
    let mut s = port::open_first_world(&d, 7, true).unwrap();
    s.set_walk(true);
    for _ in 0..900 {
        tick(&mut s, 1);
        if s.grounded() {
            break;
        }
    }
    assert!(s.grounded());
    // Look about 45 degrees down: the ray hits the ground a few voxels ahead, not under the feet.
    s.frame(&Controls {
        look_pitch: -0.2,
        dt: 1.0 / 60.0,
        ..Controls::default()
    })
    .unwrap();
    tick(&mut s, 30);
    (s, d)
}

fn total(s: &dyn SessionPort) -> u32 {
    s.game_view().slots.iter().map(|(_, c)| *c).sum()
}

#[test]
fn mining_fills_the_inventory_and_building_uses_it_up() {
    let (mut s, d) = start("a");
    assert_eq!(total(s.as_ref()), 0);
    let t = s.target().expect("the crosshair points at the ground");
    assert!(t.can_remove && t.distance <= 6.0);
    // An empty hand cannot place, and a refused placement costs nothing.
    assert!(s.place().is_err());
    assert_eq!(total(s.as_ref()), 0);
    assert_eq!(s.mine(), Ok(true));
    assert_eq!(total(s.as_ref()), 1, "one block mined, one item");
    tick(&mut s, 20);
    // Select the slot that holds it and build it back.
    s.select_slot(0);
    assert_eq!(s.place(), Ok(true));
    assert_eq!(total(s.as_ref()), 0, "the placed block was used up");
    tick(&mut s, 20);
    assert!(s.place().is_err(), "nothing left to place");
    assert_eq!(total(s.as_ref()), 0);
    s.shutdown();
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn a_full_inventory_refuses_mining_and_the_block_stays() {
    let (mut s, d) = start("b");
    // Fill everything with a material the ground does not give.
    s.shutdown();
    drop(s);
    let mut g = GameState::default();
    g.inventory.add(ItemId(5), 27 * MAX_STACK);
    port::save_game(&d, &g).unwrap();
    let mut s = port::open_first_world(&d, 7, false).unwrap();
    for _ in 0..300 {
        tick(&mut s, 1);
        if s.grounded() {
            break;
        }
    }
    tick(&mut s, 30);
    let t = s.target().expect("a target");
    assert!(
        !t.can_remove,
        "the target says it cannot be taken: {}",
        t.reason
    );
    assert_eq!(s.mine(), Err("Inventory full".to_owned()));
    assert_eq!(
        total(s.as_ref()),
        27 * MAX_STACK,
        "nothing was created or lost"
    );
    // The block is still there: the same target.
    tick(&mut s, 10);
    assert_eq!(s.target().unwrap().voxel, t.voxel);
    s.shutdown();
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn the_shelter_objective_completes_and_everything_survives_a_restart() {
    let (mut s, d) = start("c");
    let mut message_seen = false;
    for i in 0..12 {
        assert_eq!(s.mine(), Ok(true), "mine {i}");
        tick(&mut s, 25);
        s.select_slot(0);
        assert_eq!(s.place(), Ok(true), "place {i}");
        message_seen |= s.game_view().message.contains("Objective complete");
        tick(&mut s, 25);
    }
    let v = s.game_view();
    assert!(v.objective_done, "{}", v.objective_text);
    assert!(message_seen, "the player was told");
    assert_eq!(total(s.as_ref()), 0);
    // Keep three items, switch to creative and back, then quit and continue.
    assert_eq!(s.mine(), Ok(true));
    s.shutdown();
    drop(s);
    assert!(d.join(GAME_FILE).exists());
    let mut again = port::open_first_world(&d, 7, false).unwrap();
    let v2 = again.game_view();
    assert!(v2.objective_done && v2.objective_text.contains("12/12"));
    assert_eq!(
        v2.slots.iter().map(|(_, c)| *c).sum::<u32>(),
        1,
        "the mined block is still in the inventory"
    );
    // Creative: unlimited materials, nothing consumed.
    again.set_creative(true);
    assert!(again.game_view().creative);
    again.select_slot(4);
    for _ in 0..300 {
        tick(&mut again, 1);
        if again.grounded() {
            break;
        }
    }
    tick(&mut again, 30);
    assert_eq!(again.place(), Ok(true));
    assert_eq!(
        again.game_view().slots.iter().map(|(_, c)| *c).sum::<u32>(),
        0,
        "creative shows no counts"
    );
    again.shutdown();
    assert_eq!(port::load_game(&d).unwrap().mode, Mode::Creative);
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn a_damaged_game_file_means_a_fresh_game() {
    let (mut s, d) = start("d");
    s.shutdown();
    drop(s);
    std::fs::write(d.join(GAME_FILE), "cube-game 1\nslot 0 1 9999\n").unwrap();
    let again = port::open_first_world(&d, 7, false).unwrap();
    assert_eq!(
        again.game_view().slots.iter().map(|(_, c)| *c).sum::<u32>(),
        0
    );
    assert!(!again.game_view().creative);
    let _ = std::fs::remove_dir_all(&d);
}
