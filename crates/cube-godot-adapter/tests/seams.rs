//! P-WORLD stage A: seam check across chunk borders for many seeds, including negative coordinates.
//!
//! For every border between two loaded chunks, the number of side faces the face-culling mesher emits on the
//! border plane must equal the height difference of the two columns, summed over the border: a missing face
//! would be a hole (seam), an extra face an internal wall. Heights come from the generator function, the faces
//! from the stored voxels, so the check also ties the storage to the function.

use cube_godot_adapter::world_bridge::WorldBridge;

const SEEDS: [u64; 12] = [0, 1, 2, 7, 42, 99, 1234, 12345, 424_242, 999_999, 0xFFFF_FFFF, u64::MAX];

/// Counts mesh faces of chunk `(cx, cz)` whose four vertices lie on the plane `axis = plane` with normal `+dir`
/// (`axis` 0 = x, 2 = z), restricted to column range `lo..hi` along the other axis.
fn faces_on_plane(b: &WorldBridge, cx: i32, cz: i32, axis: usize, plane: f32, dir: f32) -> usize {
    let m = b.chunk_mesh(cx, cz).expect("chunk ready");
    let nv = m.positions.len() / 3;
    let mut faces = 0;
    for f in 0..nv / 4 {
        let on_plane = (0..4).all(|k| {
            let i = f * 4 + k;
            m.positions[i * 3 + axis] == plane && m.normals[i * 3 + axis] == dir
        });
        if on_plane {
            faces += 1;
        }
    }
    faces
}

#[test]
fn side_faces_on_every_chunk_border_match_the_height_steps() {
    let mut borders_checked = 0;
    let mut faces_total = 0;
    for seed in SEEDS {
        let mut b = WorldBridge::new(seed, 2);
        // centre on a chunk corner so that negative and positive chunk coordinates are both loaded
        b.set_player(-0.5, 10.0, -0.5);
        b.pump_all();
        let chunks = b.active_chunks();
        assert!(chunks.len() >= 16, "seed {seed}: {} chunks", chunks.len());
        for [cx, cz] in chunks.iter().copied() {
            // east neighbour (x border) and south neighbour (z border), when loaded
            for (nx, nz, axis) in [(cx + 1, cz, 0usize), (cx, cz + 1, 2usize)] {
                if !chunks.contains(&[nx, nz]) {
                    continue;
                }
                let (mut a_over_b, mut b_over_a) = (0i32, 0i32);
                for t in 0..16 {
                    let (xa, za, xb, zb) = if axis == 0 {
                        (cx * 16 + 15, cz * 16 + t, nx * 16, nz * 16 + t)
                    } else {
                        (cx * 16 + t, cz * 16 + 15, nx * 16 + t, nz * 16)
                    };
                    let ha = b.ground_height(xa, za);
                    let hb = b.ground_height(xb, zb);
                    a_over_b += (ha - hb).max(0);
                    b_over_a += (hb - ha).max(0);
                }
                let plane = if axis == 0 { (cx * 16 + 16) as f32 } else { (cz * 16 + 16) as f32 };
                let fa = faces_on_plane(&b, cx, cz, axis, plane, 1.0) as i32;
                let fb = faces_on_plane(&b, nx, nz, axis, plane, -1.0) as i32;
                assert_eq!(fa, a_over_b, "seed {seed} border {:?}->{:?}: faces of the first chunk", [cx, cz], [nx, nz]);
                assert_eq!(fb, b_over_a, "seed {seed} border {:?}->{:?}: faces of the second chunk", [cx, cz], [nx, nz]);
                borders_checked += 1;
                faces_total += (fa + fb) as usize;
            }
        }
    }
    assert!(borders_checked >= SEEDS.len() * 20, "only {borders_checked} borders checked");
    // the check must have compared real step faces, not only zeros
    assert!(faces_total > 500, "only {faces_total} border faces compared");
    eprintln!("borders {borders_checked}, border faces compared {faces_total}");
}
