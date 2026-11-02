//! Micro-benchmark of the world-generation spike: `cargo run --release -p cube-worldgen --example bench`.
//! Wall-clock medians over repeated runs on the machine that runs it. Not a mobile result.

use std::time::Instant;

use cube_worldgen::generator::{ChunkGenerator, TerrainGenerator};
use cube_worldgen::{ChunkCoord, StreamConfig, WorldSeed, WorldStreamer, CHUNK_SIZE};

const SEED: WorldSeed = WorldSeed(12345);

fn median(v: &mut [f64]) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

fn ms(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1000.0
}

fn streamer(radius: i32) -> WorldStreamer {
    WorldStreamer::new(
        SEED,
        Box::new(TerrainGenerator),
        StreamConfig {
            load_radius: radius,
            ..StreamConfig::default()
        },
    )
}

#[cfg(windows)]
fn peak_memory_kb() -> Option<u64> {
    #[repr(C)]
    struct Counters {
        cb: u32,
        page_faults: u32,
        peak_working_set: usize,
        working_set: usize,
        q1: usize,
        q2: usize,
        q3: usize,
        q4: usize,
        pagefile: usize,
        peak_pagefile: usize,
    }
    extern "system" {
        fn GetCurrentProcess() -> isize;
        fn K32GetProcessMemoryInfo(h: isize, c: *mut Counters, cb: u32) -> i32;
    }
    let mut c = Counters {
        cb: std::mem::size_of::<Counters>() as u32,
        page_faults: 0,
        peak_working_set: 0,
        working_set: 0,
        q1: 0,
        q2: 0,
        q3: 0,
        q4: 0,
        pagefile: 0,
        peak_pagefile: 0,
    };
    // SAFETY: plain Win32 call with a correctly sized, initialised out-structure.
    let ok = unsafe { K32GetProcessMemoryInfo(GetCurrentProcess(), &mut c, c.cb) };
    (ok != 0).then_some((c.peak_working_set / 1024) as u64)
}

#[cfg(not(windows))]
fn peak_memory_kb() -> Option<u64> {
    let s = std::fs::read_to_string("/proc/self/status").ok()?;
    s.lines()
        .find(|l| l.starts_with("VmHWM:"))?
        .split_whitespace()
        .nth(1)?
        .parse()
        .ok()
}

fn main() {
    let build = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    println!(
        "build={build}  (peak memory baseline at start: {:?} KB)",
        peak_memory_kb()
    );

    // 1. Generator alone: one chunk, different coordinates each time (including negatives).
    let mut t = Vec::new();
    for i in 0..300 {
        let c = ChunkCoord::new(i % 17 - 8, i / 17 - 8);
        let s = Instant::now();
        let ch = TerrainGenerator.generate(SEED, c).unwrap();
        t.push(ms(s));
        std::hint::black_box(ch);
    }
    println!(
        "generate one chunk (generator only):        median {:.3} ms",
        median(&mut t)
    );

    // 2. Streaming load: generate + commit into Geometry Core, per chunk, and unload.
    let mut load = Vec::new();
    let mut unload = Vec::new();
    let mut w = streamer(0);
    for i in 0..200 {
        w.set_center_world(i * CHUNK_SIZE * 3, 0);
        let s = Instant::now();
        w.pump_all();
        load.push(ms(s));
        let c = ChunkCoord::new(i * 3, 0);
        let s = Instant::now();
        assert!(w.unload(c));
        unload.push(ms(s));
    }
    println!(
        "load one chunk (generate + commit):         median {:.3} ms",
        median(&mut load)
    );
    println!(
        "unload one chunk (commit removal):          median {:.3} ms",
        median(&mut unload)
    );

    // 3. Area loads on a fresh streamer each time.
    for (name, r) in [("3x3", 1), ("5x5", 2), ("7x7", 3)] {
        let mut t = Vec::new();
        let mut voxels = 0;
        let mut bytes = 0;
        for _ in 0..15 {
            let mut w = streamer(r);
            let s = Instant::now();
            w.set_center_world(8, 8);
            w.pump_all();
            t.push(ms(s));
            voxels = w.active_voxels();
            bytes = w.approx_brick_bytes();
        }
        println!("generate {name} chunks (queue + generate + commit): median {:.2} ms   active voxels {voxels}   brick bytes {} KB", median(&mut t), bytes / 1024);
    }

    // 4. Walking: move one chunk at a time for 200 steps, 5x5 window (loads + unloads continuously).
    let mut w = streamer(2);
    w.set_center_world(0, 0);
    w.pump_all();
    let s = Instant::now();
    for i in 1..=200 {
        w.set_center_world(i * CHUNK_SIZE, 0);
        w.pump_all();
        w.take_delta();
    }
    println!("walk 200 chunks with a 5x5 window:          {:.1} ms total, {:.3} ms per step (window stays {} chunks)", ms(s), ms(s) / 200.0, w.active_chunks().len());

    println!(
        "process peak working set after all runs:    {:?} KB",
        peak_memory_kb()
    );
}
