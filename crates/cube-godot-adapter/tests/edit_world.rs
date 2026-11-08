//! P-WORLD stage C: voxel picking, block removal and placement, geometry and collision after edits, chunk borders,
//! and persistence between two separate OS processes. No Godot.

use cube_godot_adapter::world_bridge::WorldBridge;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;

const SEEDS: [u64; 4] = [1, 7, 12345, 424_242];

fn world(seed: u64, radius: i32) -> WorldBridge {
    let mut b = WorldBridge::new(seed, radius);
    b.set_player(0.5, 10.0, 0.5);
    b.pump_all();
    b
}

/// xorshift, so the tests need no extra crate and are reproducible.
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn unit(&mut self) -> f32 {
        (self.next() >> 40) as f32 / (1u64 << 24) as f32
    }
    fn range(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.unit()
    }
}

/// Reference: first solid voxel along the ray by sampling every 0.002 voxels.
fn brute_force(b: &WorldBridge, o: [f32; 3], d: [f32; 3], max: f32) -> Option<[i32; 3]> {
    let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    let mut t = 0.0f32;
    while t <= max {
        let p = [o[0] + d[0] / len * t, o[1] + d[1] / len * t, o[2] + d[2] / len * t];
        let v = [p[0].floor() as i32, p[1].floor() as i32, p[2].floor() as i32];
        match b.voxel_at(v[0], v[1], v[2]) {
            m if m > 0 => return Some(v),
            -1 if (0..32).contains(&v[1]) => return None, // unloaded chunk
            _ => {}
        }
        t += 0.002;
    }
    None
}

#[test]
fn raycast_matches_a_brute_force_march_and_its_face_invariants_across_chunk_borders() {
    let (mut hits, mut agree, mut total) = (0, 0, 0);
    for seed in SEEDS {
        let b = world(seed, 2);
        let mut r = Rng(seed ^ 0x9E37_79B9_7F4A_7C15 | 1);
        for _ in 0..400 {
            // origins above the terrain around the corner of chunks (-1..0, -1..0): negative and positive coordinates
            let o = [r.range(-24.0, 24.0), r.range(22.0, 30.0), r.range(-24.0, 24.0)];
            let d = [r.range(-1.0, 1.0), r.range(-1.0, -0.05), r.range(-1.0, 1.0)];
            let hit = b.raycast(o, d, 60.0);
            total += 1;
            if let Some(h) = hit {
                hits += 1;
                assert!(b.voxel_at(h.voxel[0], h.voxel[1], h.voxel[2]) > 0, "seed {seed}: hit voxel is not solid");
                let n = h.normal;
                assert_eq!(n.iter().map(|v| v.abs()).sum::<i32>(), 1, "seed {seed}: the face must be one axis ({n:?})");
                let before = [h.voxel[0] + n[0], h.voxel[1] + n[1], h.voxel[2] + n[2]];
                assert_eq!(b.voxel_at(before[0], before[1], before[2]), 0, "seed {seed}: the voxel before the face must be air");
            }
            if brute_force(&b, o, d, 60.0) == hit.map(|h| h.voxel) {
                agree += 1;
            }
        }
    }
    eprintln!("raycast: {total} rays, {hits} hits, {agree} agree with the brute-force march");
    assert!(hits > total / 3, "too few hits ({hits} of {total}) for a meaningful test");
    // the brute force can skip a voxel corner grazed by less than its 0.002 step; exact DDA must agree otherwise
    assert!(agree * 100 >= total * 99, "only {agree} of {total} rays agree with the brute-force march");
}

#[test]
fn raycast_straight_down_hits_the_top_voxel_with_an_upward_face_and_misses_unloaded_chunks() {
    let b = world(12345, 1);
    for (x, z) in [(3, 3), (-5, 9), (15, 15), (16, -1), (-1, -1)] {
        let top = b.column_top(x, z);
        let h = b.raycast([x as f32 + 0.5, 40.0, z as f32 + 0.5], [0.0, -1.0, 0.0], 100.0).expect("hit");
        assert_eq!((h.voxel, h.normal), ([x, top, z], [0, 1, 0]));
    }
    // a column in an unloaded chunk: not a hit
    assert!(b.raycast([500.5, 40.0, 500.5], [0.0, -1.0, 0.0], 100.0).is_none());
    // a ray flying above the terrain towards the unloaded area ends without a hit (it never touches the ground)
    assert!(b.raycast([0.5, 31.5, 0.5], [1.0, 0.0, 0.0], 400.0).is_none());
    // infinite or NaN direction
    assert!(b.raycast([0.5, 40.0, 0.5], [f32::INFINITY, -1.0, 0.0], 10.0).is_none());
    // degenerate input
    assert!(b.raycast([0.5, 40.0, 0.5], [0.0, 0.0, 0.0], 10.0).is_none());
    assert!(b.raycast([f32::NAN, 40.0, 0.5], [0.0, -1.0, 0.0], 10.0).is_none());
}

#[test]
fn remove_and_place_follow_the_rules() {
    let mut b = world(12345, 1);
    assert!(b.player_spawn(8, 8));
    let top = b.column_top(0, 0);
    // remove: the top voxel goes, an air voxel cannot be removed again
    assert!(b.remove_block(0, top, 0));
    assert_eq!(b.voxel_at(0, top, 0), 0);
    assert!(!b.remove_block(0, top, 0));
    // place back into the hole, not into solid, not outside the world height, not in an unloaded chunk
    assert!(b.place_block(0, top, 0, 4));
    assert_eq!(b.voxel_at(0, top, 0), 4);
    assert!(!b.place_block(0, top, 0, 4), "placing into a solid voxel");
    assert!(!b.place_block(0, 32, 0, 4), "above the world");
    assert!(!b.place_block(0, -1, 0, 4), "below the world");
    assert!(!b.remove_block(0, 32, 0));
    assert!(!b.place_block(500, 10, 500, 4), "unloaded chunk");
    assert!(!b.remove_block(500, 10, 500), "unloaded chunk");
    // a block may not be placed inside the character
    let p = b.player_pos();
    let (px, py, pz) = (p[0].floor() as i32, p[1].floor() as i32, p[2].floor() as i32);
    assert!(!b.place_block(px, py, pz, 4), "inside the feet");
    assert!(!b.place_block(px, py + 1, pz, 4), "inside the head");
    assert!(b.place_block(px, py + 2, pz, 4), "above the head is allowed");
    assert!(!b.player_inside_solid());
}

#[test]
fn edits_update_collision_and_stored_voxels_at_once() {
    let mut b = world(12345, 2);
    assert!(b.player_spawn(8, 8));
    for _ in 0..30 {
        b.player_step(1.0 / 60.0, 0.0, 0.0, false);
    }
    let p = b.player_pos();
    let (x, z) = (p[0].floor() as i32, p[2].floor() as i32);
    let rest = p[1];
    let top = b.column_top(x, z);
    // pick the voxel under the feet with a ray from above, remove it: the body falls one voxel
    let hit = b.raycast([p[0], p[1] + 5.0, p[2]], [0.0, -1.0, 0.0], 20.0).expect("hit");
    assert_eq!(hit.voxel, [x, top, z]);
    assert!(b.remove_block(hit.voxel[0], hit.voxel[1], hit.voxel[2]));
    assert_eq!(b.column_top(x, z), top - 1);
    for _ in 0..60 {
        b.player_step(1.0 / 60.0, 0.0, 0.0, false);
    }
    assert!(rest - b.player_pos()[1] > 0.99 || b.player_pos()[1] >= b.column_top(x, z) as f32, "did not fall: {:?}", b.player_pos());
    // place on the picked face of the ground one voxel ahead: a block that blocks walking
    let ahead = b.raycast([p[0] + 3.0, p[1] + 5.0, p[2]], [0.0, -1.0, 0.0], 20.0).expect("hit ahead");
    let target = [ahead.voxel[0] + ahead.normal[0], ahead.voxel[1] + ahead.normal[1], ahead.voxel[2] + ahead.normal[2]];
    assert!(b.place_block(target[0], target[1], target[2], 4));
    assert_eq!(b.voxel_at(target[0], target[1], target[2]), 4);
}

/// All chunk meshes of the loaded area hashed, keyed by chunk.
fn mesh_hashes(b: &WorldBridge) -> BTreeMap<(i32, i32), u64> {
    b.active_chunks()
        .into_iter()
        .map(|[cx, cz]| ((cx, cz), b.chunk_mesh(cx, cz).expect("ready").content_hash()))
        .collect()
}

#[test]
fn border_edits_mark_the_neighbour_and_meshes_equal_a_world_rebuilt_from_the_save() {
    for seed in SEEDS {
        let mut b = world(seed, 2);
        b.take_dirty_sections();
        b.take_modified();
        // edit the voxels on both sides of every chunk border around the origin (x = -1|0, x = 15|16, z = -1|0)
        let cols = [(-1, 3), (0, 3), (15, -5), (16, -5), (3, -1), (3, 0), (-1, -1), (0, 0), (15, 15), (16, 16)];
        for (x, z) in cols {
            let top = b.column_top(x, z);
            assert!(b.remove_block(x, top, z), "seed {seed}: remove at {x},{z}");
        }
        for (x, z) in [(-1, 3), (16, -5)] {
            let top = b.column_top(x, z);
            assert!(b.place_block(x, top + 1, z, 4));
        }
        let dirty = b.take_dirty_sections();
        // removing (0, top, 3) is on the border with chunk (-1, 0): both chunks' sections must be dirty
        let chunks: Vec<(i32, i32)> = dirty.iter().map(|s| (s[0], s[2])).collect();
        for want in [(0, 0), (-1, 0), (1, -1), (0, -1)] {
            assert!(chunks.contains(&want), "seed {seed}: chunk {want:?} not dirty; dirty = {chunks:?}");
        }
        // save, load into a fresh world: the same content and the same meshes (borders included)
        let path = std::env::temp_dir().join(format!("pworld-border-{seed}-{}.wst0", std::process::id()));
        b.save(&path).expect("save");
        let mut c = WorldBridge::new(0, 1);
        c.load(&path).expect("load");
        c.pump_all();
        assert_eq!(b.world_hash(), c.world_hash(), "seed {seed}: content differs after load");
        assert_eq!(mesh_hashes(&b), mesh_hashes(&c), "seed {seed}: meshes differ after load");
        let _ = std::fs::remove_file(&path);
    }
}

#[test]
fn edits_survive_unloading_the_chunk_and_walking_back() {
    let mut b = world(12345, 1);
    let top = b.column_top(3, 3);
    assert!(b.remove_block(3, top, 3));
    assert!(b.place_block(5, b.column_top(5, 5) + 1, 5, 4));
    let before = (b.voxel_at(3, top, 3), b.voxel_at(5, b.column_top(5, 5), 5));
    b.set_player(400.5, 10.0, 400.5);
    b.pump_all();
    assert_eq!(b.voxel_at(3, top, 3), -1, "the chunk must be unloaded");
    b.set_player(0.5, 10.0, 0.5);
    b.pump_all();
    assert_eq!((b.voxel_at(3, top, 3), b.voxel_at(5, b.column_top(5, 5), 5)), before);
    assert_eq!(b.voxel_at(3, top, 3), 0);
}

// ---------------------------------------------------------------------------------------------
// Persistence between two separate processes. The parent test starts the same test executable twice (phase
// "write", then phase "verify") so that nothing but the file can carry the world from one to the other.

fn phase_edits() -> Vec<(i32, i32, i32, i32)> {
    // (x, y, z, expected material after the edits): removals on both sides of chunk borders incl. negative
    // coordinates, and placements. y is derived from the generated terrain in the write phase.
    vec![(-1, 0, 3, 0), (0, 0, 3, 0), (15, 0, -5, 0), (16, 0, -5, 0), (3, 1, -1, 4), (7, 2, 7, 4), (-17, 0, -17, 0), (31, 1, 33, 4)]
}

#[test]
fn child_process_phase() {
    let Ok(phase) = std::env::var("PW_PHASE") else { return };
    let dir = PathBuf::from(std::env::var("PW_DIR").expect("PW_DIR"));
    let save = dir.join("world.wst0");
    let expect = dir.join("expected.txt");
    match phase.as_str() {
        "write" => {
            let mut b = world(424_242, 2);
            let mut lines = Vec::new();
            for (x, mode, z, mat) in phase_edits() {
                let top = b.column_top(x, z);
                match mode {
                    0 => assert!(b.remove_block(x, top, z)),
                    1 => assert!(b.place_block(x, top + 1, z, 4)),
                    _ => assert!(b.place_block(x, top + 2, z, 4)),
                }
                let y = if mode == 0 { top } else if mode == 1 { top + 1 } else { top + 2 };
                lines.push(format!("{x} {y} {z} {mat}"));
            }
            b.save(&save).expect("save");
            let meshes: Vec<String> = mesh_hashes(&b).iter().map(|((cx, cz), h)| format!("mesh {cx} {cz} {h:016x}")).collect();
            std::fs::write(&expect, format!("hash {:016x}\n{}\n{}\n", b.world_hash(), lines.join("\n"), meshes.join("\n"))).unwrap();
        }
        "verify" => {
            let mut b = WorldBridge::new(1, 0); // deliberately a different world: the load must replace it all
            b.load(&save).expect("load must succeed after a restart");
            b.pump_all();
            let text = std::fs::read_to_string(&expect).unwrap();
            let mut meshes = BTreeMap::new();
            for l in text.lines() {
                let w: Vec<&str> = l.split(' ').collect();
                match w[0] {
                    "hash" => assert_eq!(format!("{:016x}", b.world_hash()), w[1], "world hash differs after restart"),
                    "mesh" => {
                        meshes.insert((w[1].parse::<i32>().unwrap(), w[2].parse::<i32>().unwrap()), w[3].to_string());
                    }
                    _ if w.len() == 4 => {
                        let v: Vec<i32> = w.iter().map(|s| s.parse().unwrap()).collect();
                        assert_eq!(b.voxel_at(v[0], v[1], v[2]), v[3], "voxel {v:?} after restart");
                    }
                    _ => {}
                }
            }
            let now: BTreeMap<(i32, i32), String> = mesh_hashes(&b).iter().map(|(k, h)| (*k, format!("{h:016x}"))).collect();
            assert_eq!(now, meshes, "meshes differ after restart");
            assert_eq!(b.seed(), 424_242);
        }
        "corrupt" => {
            let mut b = WorldBridge::new(1, 0);
            assert!(b.load(&save).is_err(), "a damaged save must be refused");
            assert_eq!(b.seed(), 1, "a refused load must leave the world unchanged");
        }
        other => panic!("unknown phase {other}"),
    }
}

fn run_phase(phase: &str, dir: &std::path::Path) -> std::process::ExitStatus {
    Command::new(std::env::current_exe().unwrap())
        .args(["child_process_phase", "--exact", "--nocapture", "--test-threads=1"])
        .env("PW_PHASE", phase)
        .env("PW_DIR", dir)
        .status()
        .expect("spawn child test process")
}

#[test]
fn saved_edits_come_back_after_a_full_restart_in_a_separate_process() {
    let dir = std::env::temp_dir().join(format!("pworld-two-process-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    assert!(run_phase("write", &dir).success(), "write process failed");
    assert!(dir.join("world.wst0").exists());
    assert!(run_phase("verify", &dir).success(), "verify process failed: the saved edits did not come back");
    // negative control: flip one byte in the middle of the file; the third process must refuse it
    let mut bytes = std::fs::read(dir.join("world.wst0")).unwrap();
    let mid = bytes.len() / 2;
    bytes[mid] ^= 0xFF;
    std::fs::write(dir.join("world.wst0"), bytes).unwrap();
    assert!(run_phase("corrupt", &dir).success(), "damaged save was not refused");
    let _ = std::fs::remove_dir_all(&dir);
}
