//! M1-A3b: cost of meshing a brick with neighbour data read on demand (the current provider)
//! against a temporary halo (copy the 18x18x18 voxels around the brick into one array, then
//! mesh from the array). Decides DEC-095 (brick boundary handling) with numbers.
//!
//! ```text
//! cargo run -p cube-app --example mesh_cost --release
//! ```
//!
//! Every present brick of a generated region is meshed by both ways; the face counts must be
//! equal (the halo variant is only a different way to read the same neighbours). Times are
//! per brick, median of five passes, microseconds. Nothing is asserted except equality.

use std::path::PathBuf;
use std::time::Instant;

use cube_backend_voxel::{FACES_KIND, FacesMesh, FacesProvider};
use cube_core::{
    BRICK_EDGE, BrickCoord, CellKey, CellPayload, CellState, ChannelId, GeneratorVersion, LayerId,
    LocalPos, VoxelPos, WorldSeed,
};
use cube_persistence::WorldRecord;
use cube_processing::{
    CancellationToken, ComputeContext, DerivedTarget, Provider, Snapshot, SnapshotRequest,
};
use cube_server::{LocalHost, StaticArea, VOXEL_LAYER};

const EDGE: usize = BRICK_EDGE as usize;
const H: usize = EDGE + 2; // 18

fn median(v: &mut [f64]) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

/// Material of the voxel at `o + (x, y, z)` offset (x, y, z may be -1 or 16), read through the
/// snapshot cell by cell (the on-demand way, one lookup per voxel).
fn sample(snap: &Snapshot, p: VoxelPos) -> u16 {
    let (b, l) = p.split();
    match snap.cell(b) {
        Ok(CellState::Present(pl)) => match &**pl {
            CellPayload::Channels(set) => {
                set.channel(ChannelId::Material).map_or(0, |c| c.get(l)) as u16
            }
            CellPayload::MeshPage(_) => 0,
        },
        _ => 0,
    }
}

/// The halo: all 18^3 voxels around the brick, filled by walking the 27 bricks once.
fn halo(snap: &Snapshot, centre: BrickCoord) -> Vec<u16> {
    let mut a = vec![0u16; H * H * H];
    for dz in -1i64..=1 {
        for dy in -1i64..=1 {
            for dx in -1i64..=1 {
                let b = BrickCoord::new(centre.x + dx, centre.y + dy, centre.z + dz);
                let Ok(CellState::Present(pl)) = snap.cell(b) else {
                    continue;
                };
                let CellPayload::Channels(set) = &**pl else {
                    continue;
                };
                let Some(ch) = set.channel(ChannelId::Material) else {
                    continue;
                };
                // Local voxel range of this neighbour that falls inside -1..=16 of the centre.
                let lo = |d: i64| if d < 0 { EDGE as i64 - 1 } else { 0 };
                let hi = |d: i64| if d > 0 { 0 } else { EDGE as i64 - 1 };
                let (x0, x1) = (
                    if dx < 0 { lo(dx) } else { 0 },
                    if dx > 0 { hi(dx) } else { EDGE as i64 - 1 },
                );
                let (y0, y1) = (
                    if dy < 0 { lo(dy) } else { 0 },
                    if dy > 0 { hi(dy) } else { EDGE as i64 - 1 },
                );
                let (z0, z1) = (
                    if dz < 0 { lo(dz) } else { 0 },
                    if dz > 0 { hi(dz) } else { EDGE as i64 - 1 },
                );
                for z in z0..=z1 {
                    for y in y0..=y1 {
                        for x in x0..=x1 {
                            let l = LocalPos::new(x as u8, y as u8, z as u8).unwrap();
                            let gx = (dx * EDGE as i64 + x + 1) as usize;
                            let gy = (dy * EDGE as i64 + y + 1) as usize;
                            let gz = (dz * EDGE as i64 + z + 1) as usize;
                            if gx < H && gy < H && gz < H {
                                a[gx + H * (gy + H * gz)] = ch.get(l) as u16;
                            }
                        }
                    }
                }
            }
        }
    }
    a
}

/// Faces of the centre brick from the halo array (same rule as the provider: a face where a
/// solid voxel meets air).
fn faces_from_halo(a: &[u16]) -> usize {
    let at = |x: usize, y: usize, z: usize| a[x + H * (y + H * z)];
    let mut n = 0;
    for z in 1..=EDGE {
        for y in 1..=EDGE {
            for x in 1..=EDGE {
                if at(x, y, z) == 0 {
                    continue;
                }
                for (dx, dy, dz) in [
                    (1i32, 0i32, 0i32),
                    (-1, 0, 0),
                    (0, 1, 0),
                    (0, -1, 0),
                    (0, 0, 1),
                    (0, 0, -1),
                ] {
                    let q = at(
                        (x as i32 + dx) as usize,
                        (y as i32 + dy) as usize,
                        (z as i32 + dz) as usize,
                    );
                    if q == 0 {
                        n += 1;
                    }
                }
            }
        }
    }
    n
}

fn main() {
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    let rec = WorldRecord {
        world_id: 1,
        seed: WorldSeed::new(7),
        generator_version: GeneratorVersion::new(6),
        structure_version: cube_persistence::STRUCTURE_VERSION,
    };
    let d: PathBuf = std::env::temp_dir().join(format!("cube-meshcost-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    let host = LocalHost::create(&d, rec, StaticArea::around_origin(&rec, 1)).unwrap();
    let centre = host.area().centre;
    let radius = 3;
    host.ensure_loaded(centre, radius + 1, usize::MAX).unwrap();
    let reader = host.reader();
    let layer = LayerId::new(VOXEL_LAYER);

    // The present bricks of the region.
    let mut bricks = Vec::new();
    for dz in -radius..=radius {
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let b = BrickCoord::new(centre.x + dx, centre.y + dy, centre.z + dz);
                let key = CellKey::new(layer, 0, b).unwrap();
                let snap =
                    Snapshot::capture(&reader, &SnapshotRequest::for_brick(key, 1), None).unwrap();
                if matches!(snap.cell(b), Ok(CellState::Present(_))) {
                    bricks.push(b);
                }
            }
        }
    }
    println!(
        "profile: {profile}, generator v6, seed 7, {} present bricks of {} in the region",
        bricks.len(),
        (2 * radius + 1).pow(3)
    );
    println!();

    let provider = FacesProvider;
    let mut capture_us = Vec::new();
    let mut demand_us = Vec::new();
    let mut halo_copy_us = Vec::new();
    let mut halo_mesh_us = Vec::new();
    let mut faces_total = 0usize;
    for _pass in 0..5 {
        let (mut c, mut dm, mut hc, mut hm) = (0.0, 0.0, 0.0, 0.0);
        faces_total = 0;
        for b in &bricks {
            let key = CellKey::new(layer, 0, *b).unwrap();
            let t = Instant::now();
            let snap =
                Snapshot::capture(&reader, &SnapshotRequest::for_brick(key, 1), None).unwrap();
            c += t.elapsed().as_secs_f64() * 1e6;

            // On demand: the provider as it is.
            let token = CancellationToken::new();
            let ctx = ComputeContext {
                target: DerivedTarget {
                    kind: FACES_KIND,
                    key,
                },
                snapshot: &snap,
                token: &token,
                inputs: &[],
            };
            let t = Instant::now();
            let payload = provider.compute(&ctx).unwrap();
            dm += t.elapsed().as_secs_f64() * 1e6;
            let on_demand = payload.downcast_ref::<FacesMesh>().unwrap().faces.len();

            // Halo: copy once, then mesh from the array.
            let t = Instant::now();
            let a = halo(&snap, *b);
            hc += t.elapsed().as_secs_f64() * 1e6;
            let t = Instant::now();
            let n = faces_from_halo(&a);
            hm += t.elapsed().as_secs_f64() * 1e6;
            assert_eq!(
                n, on_demand,
                "halo and on-demand give the same faces for {b:?}"
            );
            faces_total += n;
        }
        let k = bricks.len() as f64;
        capture_us.push(c / k);
        demand_us.push(dm / k);
        halo_copy_us.push(hc / k);
        halo_mesh_us.push(hm / k);
    }
    // A reference for the sampling cost alone: every voxel of the 18^3 cube through the snapshot.
    let b0 = bricks[bricks.len() / 2];
    let key0 = CellKey::new(layer, 0, b0).unwrap();
    let snap0 = Snapshot::capture(&reader, &SnapshotRequest::for_brick(key0, 1), None).unwrap();
    let o = b0.origin().unwrap();
    let t = Instant::now();
    let mut s = 0u64;
    for z in -1..=EDGE as i64 {
        for y in -1..=EDGE as i64 {
            for x in -1..=EDGE as i64 {
                s += u64::from(sample(&snap0, VoxelPos::new(o.x + x, o.y + y, o.z + z)));
            }
        }
    }
    let sample_all_us = t.elapsed().as_secs_f64() * 1e6;

    let (cap, dem, hcp, hms) = (
        median(&mut capture_us),
        median(&mut demand_us),
        median(&mut halo_copy_us),
        median(&mut halo_mesh_us),
    );
    println!("| per brick, microseconds (median of 5 passes) | value |");
    println!("|---|---|");
    println!("| snapshot capture (same for both) | {cap:.1} |");
    println!("| on demand: provider compute (neighbours read per voxel) | {dem:.1} |");
    println!("| halo: copy 18^3 voxels from the 27 bricks | {hcp:.1} |");
    println!("| halo: faces from the array | {hms:.1} |");
    println!("| halo total (copy + faces) | {:.1} |", hcp + hms);
    println!("| on demand / halo | {:.1}x |", dem / (hcp + hms));
    println!(
        "| reference: 5832 voxel samples through the snapshot | {sample_all_us:.1} (checksum {s}) |"
    );
    println!(
        "| faces per brick (mean) | {:.0} |",
        faces_total as f64 / bricks.len() as f64
    );
    drop(host);
    let _ = std::fs::remove_dir_all(&d);
}
