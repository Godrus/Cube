//! P-WORLD stage B: the character against the real streamed world (no Godot). Frame loop = the client's:
//! one physics step of 1/60 s, then `set_player` + `pump(2)` (the client's `PUMP_PER_FRAME`).

use cube_godot_adapter::player::{HEIGHT, WIDTH};
use cube_godot_adapter::world_bridge::WorldBridge;
use std::time::Instant;

const SEEDS: [u64; 5] = [1, 7, 12345, 424_242, 999_999];
const DT: f32 = 1.0 / 60.0;

/// Jittery frame times (seconds), as a real frame loop on a 75-144 Hz display with hitches produces them.
const JITTER: [f32; 8] = [0.0036, 0.0133, 0.0167, 0.0069, 0.03, 0.0133, 0.0052, 0.0133];

fn frame(b: &mut WorldBridge, mx: f32, mz: f32, jump: bool) {
    frame_dt(b, DT, mx, mz, jump);
}

fn frame_dt(b: &mut WorldBridge, dt: f32, mx: f32, mz: f32, jump: bool) {
    b.player_step(dt, mx, mz, jump);
    let p = b.player_pos();
    b.set_player(p[0], p[1], p[2]);
    b.pump(2);
}

fn start(seed: u64, radius: i32) -> WorldBridge {
    let mut b = WorldBridge::new(seed, radius);
    b.set_player(0.5, 10.0, 0.5);
    b.pump_all();
    assert!(b.player_spawn(0, 0));
    b
}

/// Highest ground (top solid voxel + 1) under any column the body box covers.
fn ground_under(b: &WorldBridge) -> i32 {
    let p = b.player_pos();
    let h = WIDTH / 2.0 - 1e-3;
    let mut top = 0;
    for dz in [-h, h] {
        for dx in [-h, h] {
            top = top.max(b.column_top((p[0] + dx).floor() as i32, (p[2] + dz).floor() as i32) + 1);
        }
    }
    top
}

#[test]
fn walks_across_many_chunk_borders_in_all_directions_without_falling_or_sticking() {
    let dirs: [(f32, f32, &str); 4] = [(1.0, 0.0, "+x"), (-1.0, 0.0, "-x"), (0.0, 1.0, "+z"), (0.0, -1.0, "-z")];
    let (mut total_borders, mut max_step_us, mut total_held, mut max_held_run) = (0, 0u128, 0usize, 0usize);
    for seed in SEEDS {
        for (mx, mz, name) in dirs {
            let mut b = start(seed, 2);
            let p0 = b.player_pos();
            let mut chunk = ((p0[0].floor() as i32).div_euclid(16), (p0[2].floor() as i32).div_euclid(16));
            let (mut borders, mut held_run, mut stuck) = (0, 0usize, 0usize);
            let mut last_dist = 0.0f32;
            for i in 0..2400 {
                let t = Instant::now();
                // jump when the way is blocked for a moment (the terrain's own steps are one voxel: step-up handles them)
                frame_dt(&mut b, JITTER[i % JITTER.len()], mx, mz, stuck > 30);
                max_step_us = max_step_us.max(t.elapsed().as_micros());
                let p = b.player_pos();
                assert!(!b.player_inside_solid(), "seed {seed} {name} step {i}: inside solid at {p:?}");
                assert!(
                    b.player_held() || p[1] + 0.01 >= ground_under(&b) as f32,
                    "seed {seed} {name} step {i}: below the surface at {p:?} (ground {})",
                    ground_under(&b)
                );
                assert!(p[1] >= 0.0 && p[1] < 60.0, "seed {seed} {name} step {i}: y out of range {p:?}");
                if b.player_held() {
                    held_run += 1;
                    total_held += 1;
                    max_held_run = max_held_run.max(held_run);
                    assert!(held_run < 120, "seed {seed} {name} step {i}: held for 2 s at {p:?}");
                } else {
                    held_run = 0;
                }
                let c = ((p[0].floor() as i32).div_euclid(16), (p[2].floor() as i32).div_euclid(16));
                if c != chunk {
                    borders += 1;
                    chunk = c;
                }
                let dist = (p[0] - p0[0]).abs() + (p[2] - p0[2]).abs();
                if dist > last_dist + 0.001 {
                    stuck = 0;
                    last_dist = dist;
                } else {
                    stuck += 1;
                    assert!(stuck < 240, "seed {seed} {name} step {i}: no progress for 4 s at {p:?}");
                }
            }
            assert!(last_dist >= 100.0, "seed {seed} {name}: walked only {last_dist} voxels");
            assert!(borders >= 5, "seed {seed} {name}: crossed only {borders} chunk borders");
            total_borders += borders;
        }
    }
    eprintln!(
        "walks: 20 runs, {total_borders} chunk borders crossed, max physics+stream frame {max_step_us} us, held steps {total_held}, longest held run {max_held_run}"
    );
}

#[test]
fn a_chunk_that_is_not_loaded_holds_the_body_then_it_lands_on_the_surface() {
    for seed in SEEDS {
        let mut b = start(seed, 1);
        // teleport far away: nothing is loaded there yet
        b.player_teleport(-1000.5, 2000.5);
        let before = b.player_pos();
        for _ in 0..120 {
            b.player_step(DT, 1.0, 1.0, true); // walking and jumping while held changes nothing
        }
        assert!(b.player_held(), "seed {seed}: not held over an unloaded chunk");
        assert_eq!(b.player_pos(), before, "seed {seed}: the held body moved");
        // load around it, as the client does frame by frame
        let mut frames = 0;
        while b.player_held() || !b.player_on_ground() {
            frame(&mut b, 0.0, 0.0, false);
            frames += 1;
            assert!(frames < 600, "seed {seed}: never landed ({:?})", b.player_pos());
            assert!(!b.player_inside_solid());
        }
        let p = b.player_pos();
        assert!((p[1] - ground_under(&b) as f32).abs() < 0.01, "seed {seed}: rests at {p:?}, ground {}", ground_under(&b));
    }
}

#[test]
fn edits_change_the_collision() {
    let mut b = start(12345, 2);
    let (x, z) = (0, 0);
    let top = b.column_top(x, z);
    assert!(top >= 0);
    // dig the voxel under the feet: the body falls one voxel
    for _ in 0..30 {
        frame(&mut b, 0.0, 0.0, false);
    }
    let rest = b.player_pos()[1];
    assert!(b.edit_voxel(x, top, z, 0), "digging failed");
    assert_eq!(b.column_top(x, z), top - 1, "the stored column must reflect the edit");
    for _ in 0..60 {
        frame(&mut b, 0.0, 0.0, false);
    }
    // the body is 0.6 wide and may overlap neighbouring columns, so it falls at most one voxel and lands
    let fell = rest - b.player_pos()[1];
    assert!(fell > 0.99 || b.player_pos()[1] >= ground_under(&b) as f32 - 0.01, "fell {fell}");
    assert!(!b.player_inside_solid());
    // place a 2-high wall of material 4 in front: it blocks
    let p = b.player_pos();
    let (wx, wz) = (p[0].floor() as i32 + 2, p[2].floor() as i32);
    let base = b.column_top(wx, wz) + 1;
    for dy in 0..2 {
        assert!(b.edit_voxel(wx, base + dy, wz, 4));
    }
    for _ in 0..600 {
        frame(&mut b, 1.0, 0.0, true);
    }
    assert!(b.player_pos()[0] < wx as f32 - WIDTH / 2.0 + 0.01 || b.player_pos()[1] > (base + 2) as f32 - 0.01,
        "walked through the wall: {:?}", b.player_pos());
    assert!(!b.player_inside_solid());
    let _ = HEIGHT;
}

#[test]
fn spawn_waits_for_a_loaded_chunk() {
    let mut b = WorldBridge::new(12345, 1);
    assert!(!b.player_spawn(500, 500), "spawn must refuse an unloaded column");
    assert_eq!(b.voxel_at(500, 10, 500), -1);
    assert_eq!(b.column_top(500, 500), -2);
}
