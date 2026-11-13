//! M1-A3: cost of generating and materialising bricks on the calling thread.
//!
//! Drives `LocalHost::ensure_loaded` the way a walking player does in `Session::frame`: the
//! centre moves one brick per step (the worst case, a new border every frame), each call may
//! generate at most `load_per_frame` bricks, and the call time is the frame cost of streaming.
//! Run in both profiles and compare with the frame budget:
//!
//! ```text
//! cargo run -p cube-app --example gen_cost
//! cargo run -p cube-app --example gen_cost --release
//! ```
//!
//! Pass `quick` to limit the table to generator versions 1 and 6 and view radii 1, 2 and 3 (the
//! debug profile is slow). Radii 1, 2 and 3 are the default view radii of the device classes.
//!
//! Output is a Markdown table (one row per generator version and view radius). Nothing is
//! asserted: this is a measurement, the decision about a worker thread is made from the table
//! (docs/roadmap/WORK-PACKAGES.md, M1-A3).

use std::path::PathBuf;
use std::time::Instant;

use cube_core::{BrickCoord, GeneratorVersion, WorldSeed};
use cube_persistence::WorldRecord;
use cube_server::{LocalHost, StaticArea};

const SEED: u64 = 7;
const STEPS: i64 = 40;
const BUDGET_PER_FRAME: usize = 48;

fn dir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("cube-gencost-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let i = ((sorted.len() - 1) as f64 * p).round() as usize;
    sorted[i]
}

/// (frame index, ms, loads delta, evictions delta, derived cache delta, active delta)
type SlowFrame = (usize, f64, u64, u64, i64, i64);

/// `Session::frame` cost while flying east at 12 voxels per second, 60 simulated frames per
/// second, desktop view radius, with the default time-sliced budget and with slicing off.
fn session_frames() {
    use cube_app::{Session, StreamBudget};
    use cube_client::{DeviceClass, Intent, ViewProfile};
    use std::sync::Arc;

    println!();
    println!(
        "Session::frame while flying east (desktop radius, generator v6, 1800 frames of 1/60 s)"
    );
    println!();
    println!(
        "| budget | frame ms mean | frame ms p95 | frame ms max | frames over 16.7 ms | bricks loaded |"
    );
    println!("|---|---|---|---|---|---|");
    for (label, time_us) in [
        ("time-sliced 4 ms (default)", 4_000u64),
        ("unsliced, 48 bricks", 0),
    ] {
        let rec = WorldRecord {
            world_id: 1,
            seed: WorldSeed::new(SEED),
            generator_version: GeneratorVersion::new(6),
            structure_version: cube_persistence::STRUCTURE_VERSION,
        };
        let d = dir(&format!("session-{time_us}"));
        let host =
            Arc::new(LocalHost::create(&d, rec, StaticArea::around_origin(&rec, 1)).unwrap());
        let top = (host.area().centre.y * 16 + 8) as f64;
        let s = Session::new(
            host.clone(),
            ViewProfile::of(DeviceClass::Desktop),
            [0.5, top + 12.0, 0.5],
        )
        .unwrap();
        s.set_budget(StreamBudget {
            load_time_us: time_us,
            ..StreamBudget::default()
        });
        s.frame(&[]).unwrap(); // the first region is generated in full
        let east = [(
            Intent::Move {
                forward: 0,
                right: 1,
                up: 0,
            },
            1.0 / 60.0,
        )];
        let mut times: Vec<f64> = Vec::new();
        // (frame index, ms, loads delta, evictions delta, derived cache delta, active delta)
        let mut slow: Vec<SlowFrame> = Vec::new();
        for i in 0..1800 {
            let before = s.stats();
            let t = Instant::now();
            s.frame(&east).unwrap();
            let ms = t.elapsed().as_secs_f64() * 1000.0;
            times.push(ms);
            let after = s.stats();
            if ms > 16.7 {
                slow.push((
                    i,
                    ms,
                    after.loads - before.loads,
                    after.evictions - before.evictions,
                    after.derived_cached as i64 - before.derived_cached as i64,
                    after.active as i64 - before.active as i64,
                ));
            }
        }
        if !slow.is_empty() {
            println!();
            println!(
                "slow frames of `{label}`: frame, ms, bricks loaded, bricks evicted, derived cache delta, active delta"
            );
            for (i, ms, l, e, d, a) in &slow {
                println!("  {i:5} {ms:6.1} {l:4} {e:4} {d:5} {a:5}");
            }
            let with = |f: &dyn Fn(&SlowFrame) -> bool| slow.iter().filter(|x| f(x)).count();
            println!(
                "  of {} slow frames: {} loaded bricks, {} evicted bricks, {} changed the derived cache by 20 or more, {} changed the active set",
                slow.len(),
                with(&|x| x.2 > 0),
                with(&|x| x.3 > 0),
                with(&|x| x.4.abs() >= 20),
                with(&|x| x.5 != 0)
            );
            println!();
        }
        let mut sorted = times.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mean = times.iter().sum::<f64>() / times.len() as f64;
        let over = times.iter().filter(|t| **t > 16.7).count();
        println!(
            "| {label} | {mean:.2} | {:.2} | {:.2} | {over} | {} |",
            percentile(&sorted, 0.95),
            sorted.last().copied().unwrap_or(0.0),
            host.stream_stats().loads
        );
        drop(s);
        drop(host);
        let _ = std::fs::remove_dir_all(&d);
    }
}

fn main() {
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    let quick = std::env::args().any(|a| a == "quick");
    let versions: Vec<u32> = if quick { vec![1, 6] } else { (1..=6).collect() };
    let radii: Vec<i64> = if quick {
        vec![1, 2, 3]
    } else {
        vec![1, 2, 3, 5, 8]
    };
    println!(
        "profile: {profile}, seed {SEED}, {STEPS} steps along +x, budget {BUDGET_PER_FRAME} bricks per frame"
    );
    println!();
    println!(
        "| gen | view radius | region bricks | first region ms | calls | bricks/call mean | call ms mean | call ms p95 | call ms max | us per brick |"
    );
    println!("|---|---|---|---|---|---|---|---|---|---|");
    for &version in &versions {
        for &radius in &radii {
            let rec = WorldRecord {
                world_id: 1,
                seed: WorldSeed::new(SEED),
                generator_version: GeneratorVersion::new(version),
                structure_version: cube_persistence::STRUCTURE_VERSION,
            };
            let d = dir(&format!("{version}-{radius}"));
            let host = match LocalHost::create(&d, rec, StaticArea::around_origin(&rec, 1)) {
                Ok(h) => h,
                Err(_) => continue, // version not available in this build
            };
            let load_radius = radius + 1; // view radius plus the margin of StreamBudget::default
            let surface =
                |x: i64| BrickCoord::new(x, host.surface_height(x * 16, 0).div_euclid(16), 0);

            // The first region is generated in full before anything is shown.
            let t0 = Instant::now();
            let first = surface(0);
            host.ensure_loaded(first, load_radius, usize::MAX).unwrap();
            let first_ms = t0.elapsed().as_secs_f64() * 1000.0;
            let region = (2 * load_radius + 1).pow(3);

            // Walking: one brick per step, repeated calls until the surroundings exist, the way
            // the frame loop catches up under a per-frame budget.
            let mut times: Vec<f64> = Vec::new();
            let mut bricks: usize = 0;
            for step in 1..=STEPS {
                let c = surface(step);
                loop {
                    let before = host.loaded_bricks();
                    let t = Instant::now();
                    let missing = host
                        .ensure_loaded(c, load_radius, BUDGET_PER_FRAME)
                        .unwrap();
                    let ms = t.elapsed().as_secs_f64() * 1000.0;
                    let made = host.loaded_bricks() - before;
                    if made > 0 {
                        times.push(ms);
                        bricks += made;
                    }
                    if missing == 0 {
                        break;
                    }
                }
            }
            let mut sorted = times.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let mean = times.iter().sum::<f64>() / times.len().max(1) as f64;
            let total: f64 = times.iter().sum();
            println!(
                "| v{version} | {radius} | {region} | {first_ms:.1} | {} | {:.1} | {mean:.2} | {:.2} | {:.2} | {:.0} |",
                times.len(),
                bricks as f64 / times.len().max(1) as f64,
                percentile(&sorted, 0.95),
                sorted.last().copied().unwrap_or(0.0),
                total * 1000.0 / bricks.max(1) as f64,
            );
            drop(host);
            let _ = std::fs::remove_dir_all(&d);
        }
    }
    session_frames();
}
