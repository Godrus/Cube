//! The walking player through the real local host, generator, Processing and client (headless):
//! gravity onto the generated ground, walking without ever standing inside a block, digging under
//! the feet, refusing to build into the player's body, and continuing after quitting.
//! It is data, not a rendered scene; the Godot run of the same behaviour is separate.
use cube_app::port::{self, PLAYER_FILE};
use cube_client::{Controls, Intent, SessionPort};
use std::path::PathBuf;

fn dir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("cube-walk-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

fn idle() -> Controls {
    Controls {
        dt: 1.0 / 60.0,
        ..Controls::default()
    }
}

/// Runs frames until the walking player stands on something (bounded).
fn land(s: &mut Box<dyn SessionPort>) {
    for _ in 0..900 {
        s.frame(&idle()).unwrap();
        if s.grounded() {
            return;
        }
    }
    panic!("the player never landed: feet {:?}", s.player_feet());
}

#[test]
fn falls_onto_the_generated_ground_and_walks_without_sinking_in() {
    let d = dir("a");
    let mut s = port::open_first_world(&d, 7, true).unwrap();
    s.set_walk(true);
    assert!(s.walking());
    let spawn = s.player_feet().unwrap();
    land(&mut s);
    let feet = s.player_feet().unwrap();
    assert!(
        feet[1] < spawn[1] - 3.0,
        "gravity pulled the player down: {spawn:?} -> {feet:?}"
    );
    // Walk forward for 6 seconds, looking along the ground.
    let mut c = Controls {
        forward: 1,
        ..idle()
    };
    c.look_pitch = 0.6; // level the view
    s.frame(&c).unwrap();
    c.look_pitch = 0.0;
    let start = s.player_feet().unwrap();
    for _ in 0..360 {
        s.frame(&c).unwrap();
    }
    let end = s.player_feet().unwrap();
    let moved = ((end[0] - start[0]).powi(2) + (end[2] - start[2]).powi(2)).sqrt();
    assert!(moved > 5.0, "the player walked: moved {moved:.1}");
    assert!(end[1] > -64.0, "the player did not fall out of the world");
    s.shutdown();
}

#[test]
fn digging_under_the_feet_drops_the_player_and_building_into_the_body_is_refused() {
    let d = dir("b");
    let mut s = port::open_first_world(&d, 7, true).unwrap();
    s.set_walk(true);
    land(&mut s);
    // Look straight down.
    s.frame(&Controls {
        look_pitch: -3.0,
        ..idle()
    })
    .unwrap();
    for _ in 0..30 {
        s.frame(&idle()).unwrap();
    }
    let before = s.player_feet().unwrap();
    // The block in front of the hit face is where the player stands: refused.
    assert!(
        !s.edit(Some(5)).unwrap(),
        "a block must not be built into the player's body"
    );
    // Remove the block under the feet; the player must drop by about one voxel.
    assert!(s.edit(None).unwrap());
    for _ in 0..120 {
        s.frame(&idle()).unwrap();
    }
    let after = s.player_feet().unwrap();
    assert!(s.grounded());
    assert!(
        (before[1] - after[1] - 1.0).abs() < 0.2,
        "dropped from {:.2} to {:.2}",
        before[1],
        after[1]
    );
    s.shutdown();
}

#[test]
fn the_player_continues_where_they_left_after_quitting() {
    let d = dir("c");
    let mut s = port::open_first_world(&d, 7, true).unwrap();
    s.set_walk(true);
    land(&mut s);
    for _ in 0..90 {
        s.frame(&Controls {
            forward: 1,
            look_yaw: 0.01,
            ..idle()
        })
        .unwrap();
    }
    let feet = s.player_feet().unwrap();
    let pose = s.pose();
    s.shutdown();
    drop(s);
    assert!(
        d.join(PLAYER_FILE).exists(),
        "the player was saved next to the world"
    );

    let mut again = port::open_first_world(&d, 7, false).unwrap();
    assert!(again.walking(), "walking mode is restored");
    let restored = again.player_feet().unwrap();
    for i in 0..3 {
        assert!(
            (restored[i] - feet[i]).abs() < 1e-6,
            "feet {restored:?} vs {feet:?}"
        );
    }
    assert!((again.pose().yaw - pose.yaw).abs() < 1e-6);
    land(&mut again);
    again.shutdown();
}

#[test]
fn a_damaged_player_file_means_a_fresh_start_not_a_crash() {
    let d = dir("d");
    let mut s = port::open_first_world(&d, 7, true).unwrap();
    s.shutdown();
    drop(s);
    std::fs::write(d.join(PLAYER_FILE), "garbage\n").unwrap();
    let again = port::open_first_world(&d, 7, false).unwrap();
    assert!(!again.walking());
    assert!(port::decode_player("cube-player 1\nfeet 1 2\n").is_none());
    assert!(
        port::decode_player("cube-player 1\nfeet 1 2 NaN\nview 0 0\nwalking 1\nthird_person 0\n")
            .is_none()
    );
    let _ = Intent::Speed(1.0);
}

#[test]
fn an_edit_far_from_the_start_survives_a_restart() {
    use cube_core::{BrickCoord, GeneratorVersion, WorldSeed};
    use cube_persistence::WorldRecord;
    use cube_server::{EditRequest, LocalHost, StaticArea};
    let d = dir("e");
    let rec = WorldRecord {
        world_id: 1,
        seed: WorldSeed::new(7),
        generator_version: GeneratorVersion::new(1),
        structure_version: cube_persistence::STRUCTURE_VERSION,
    };
    let area = StaticArea::around_origin(&rec, 1);
    let host = LocalHost::create(&d, rec, area).unwrap();
    // Walk "far away": 100 voxels east is brick 6, outside the 3x3x3 initial area.
    let h = host.surface_height(100, 0);
    let far = BrickCoord::new(6, h.div_euclid(16), 0);
    assert_eq!(host.ensure_loaded(far, 1, usize::MAX).unwrap(), 0);
    let loaded = host.loaded_bricks();
    let base = host.content_hash();
    host.submit_edit(EditRequest::Remove {
        voxel: [100, h - 1, 0],
    })
    .unwrap();
    let edited = host.content_hash();
    assert_ne!(edited, base, "the edit changed the world");
    drop(host);

    let host = LocalHost::open(&d, area).unwrap();
    // Before streaming, the brick holding the edit is already generated and edited (its base was
    // materialised before the journal was replayed), so the edit is not lost or silently dropped.
    assert!(
        host.loaded_bricks() > 27,
        "the edited brick was generated at open"
    );
    host.ensure_loaded(far, 1, usize::MAX).unwrap();
    assert_eq!(host.loaded_bricks(), loaded);
    assert_eq!(
        host.content_hash(),
        edited,
        "the far edit is still there after the restart"
    );
    let _ = std::fs::remove_dir_all(&d);
}
