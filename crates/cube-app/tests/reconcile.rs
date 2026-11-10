//! The world journal and `game.state` are two stores. These tests stop the game at every point
//! between "intent written" and "effect written" (by doing the steps by hand on the real host and
//! never calling shutdown, which is what a crash looks like to the files) and check that the
//! next start settles the inventory against the journal: no item is created, none is lost, none
//! is counted twice. Real host, real files; not a device crash (DG-2).
use std::path::{Path, PathBuf};

use cube_app::game::{GameState, ItemId};
use cube_app::port::{self, FIRST_WORLD_RADIUS};
use cube_client::{Controls, SessionPort};
use cube_core::{GeneratorVersion, WorldSeed};
use cube_persistence::WorldRecord;
use cube_server::{EditRequest, LocalHost, StaticArea};

fn dir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("cube-reconcile-{}-{tag}", std::process::id()));
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

/// Creates a world, lets the player land and look at the ground, remembers what the crosshair
/// points at, and quits cleanly. Returns the directory and the target (voxel, material).
fn world_with_target(tag: &str) -> (PathBuf, [i64; 3], u16) {
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
    s.frame(&Controls {
        look_pitch: -0.2,
        dt: 1.0 / 60.0,
        ..Controls::default()
    })
    .unwrap();
    tick(&mut s, 30);
    let t = s.target().expect("the crosshair points at the ground");
    s.shutdown();
    (d, t.voxel, t.material)
}

fn host(d: &Path) -> LocalHost {
    let rec = WorldRecord {
        world_id: 1,
        seed: WorldSeed::new(7),
        generator_version: GeneratorVersion::new(1),
        structure_version: cube_persistence::STRUCTURE_VERSION,
    };
    LocalHost::open(d, StaticArea::around_origin(&rec, FIRST_WORLD_RADIUS)).unwrap()
}

fn items(d: &Path) -> u32 {
    let g = port::load_game(d).expect("a valid game file");
    (1..=5).map(|i| g.inventory.count_of(ItemId(i))).sum()
}

fn total(s: &dyn SessionPort) -> u32 {
    s.game_view().slots.iter().map(|(_, c)| *c).sum()
}

#[test]
fn a_crash_after_the_world_change_and_before_the_inventory_still_gives_exactly_one_item() {
    let (d, voxel, material) = world_with_target("after");
    // The game does steps 1 and 2 of a mine and dies before step 3.
    let seq;
    {
        let h = host(&d);
        seq = h.reserve_edit_seq();
        let mut g = port::load_game(&d).unwrap();
        g.begin_mined(seq, material);
        port::save_game(&d, &g).unwrap();
        h.submit_edit_reserved(EditRequest::Remove { voxel }, seq)
            .unwrap();
        // A retry of the same operation id never changes the world twice.
        let hash = h.content_hash();
        let _ = h.submit_edit_reserved(EditRequest::Remove { voxel }, seq);
        assert_eq!(h.content_hash(), hash, "the retry changed nothing");
    } // no shutdown: a crash
    assert_eq!(items(&d), 0, "the file still has the intent, not the item");
    let s = port::open_first_world(&d, 7, false).unwrap();
    assert_eq!(
        total(s.as_ref()),
        1,
        "the confirmed change gave its item once"
    );
    drop(s);
    let g = port::load_game(&d).unwrap();
    assert!(g.pending.is_none(), "the settled state was saved at once");
    assert_eq!(g.objective.mined, 1);
    // Another restart, and another: nothing is counted again.
    for _ in 0..2 {
        let mut s = port::open_first_world(&d, 7, false).unwrap();
        assert_eq!(total(s.as_ref()), 1);
        s.shutdown();
    }
    assert!(host(&d).edit_confirmed_at_open(seq));
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn a_crash_after_the_intent_and_before_the_world_change_creates_nothing() {
    let (d, voxel, material) = world_with_target("before");
    let hash_before = host(&d).content_hash();
    {
        let h = host(&d);
        let seq = h.reserve_edit_seq();
        let mut g = port::load_game(&d).unwrap();
        g.begin_mined(seq, material);
        port::save_game(&d, &g).unwrap();
        // dies here: the world is never asked
    }
    let mut s = port::open_first_world(&d, 7, false).unwrap();
    assert_eq!(total(s.as_ref()), 0, "no item for a change that never came");
    s.shutdown();
    drop(s);
    let g = port::load_game(&d).unwrap();
    assert!(g.pending.is_none());
    assert_eq!(g.objective.mined, 0);
    assert_eq!(
        host(&d).content_hash(),
        hash_before,
        "the world is untouched"
    );
    // The block is still there: the same operation can be done now.
    let h = host(&d);
    let seq = h.reserve_edit_seq();
    h.submit_edit_reserved(EditRequest::Remove { voxel }, seq)
        .unwrap();
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn a_placed_block_costs_exactly_one_item_whichever_store_was_written_last() {
    let (d, voxel, material) = world_with_target("place");
    // Mine one block properly, then crash after the world got a placement and before the file.
    let mut g = GameState::default();
    g.inventory.add(ItemId(material), 2);
    port::save_game(&d, &g).unwrap();
    {
        let h = host(&d);
        let seq = h.reserve_edit_seq();
        let mut g = port::load_game(&d).unwrap();
        g.begin_placed(seq);
        port::save_game(&d, &g).unwrap();
        h.submit_edit_reserved(
            EditRequest::Place {
                voxel: [voxel[0], voxel[1] + 40, voxel[2]],
                material,
            },
            seq,
        )
        .unwrap();
    }
    let mut s = port::open_first_world(&d, 7, false).unwrap();
    assert_eq!(
        total(s.as_ref()),
        1,
        "two items, one was used by the placement"
    );
    s.shutdown();
    drop(s);
    assert_eq!(port::load_game(&d).unwrap().objective.placed, 1);

    // The other order: the intent is written, the world never gets it.
    {
        let h = host(&d);
        let seq = h.reserve_edit_seq();
        let mut g = port::load_game(&d).unwrap();
        g.begin_placed(seq);
        port::save_game(&d, &g).unwrap();
    }
    let s = port::open_first_world(&d, 7, false).unwrap();
    assert_eq!(
        total(s.as_ref()),
        1,
        "nothing was used for a placement that never happened"
    );
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn normal_play_leaves_no_intent_behind() {
    let (d, _, _) = world_with_target("normal");
    let mut s = port::open_first_world(&d, 7, false).unwrap();
    s.set_walk(true);
    for _ in 0..300 {
        tick(&mut s, 1);
        if s.grounded() {
            break;
        }
    }
    tick(&mut s, 30);
    assert_eq!(s.mine(), Ok(true));
    assert!(
        port::load_game(&d).unwrap().pending.is_none(),
        "a finished mine clears its intent in the same write"
    );
    assert_eq!(items(&d), 1);
    s.shutdown();
    let _ = std::fs::remove_dir_all(&d);
}
