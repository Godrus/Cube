//! Renderer data comparison (cube-per-voxel control vs visible-face mesh):
//! `cargo run --release -p cube-godot-adapter --example render_bench`. Pure Rust, no Godot: it measures the
//! CPU side and the size of the arrays handed to the renderer. Desktop numbers, not mobile.

use std::time::Instant;

use cube_godot_adapter::world_bridge::{WorldBridge, FLOATS_PER_INSTANCE};

fn median(v: &mut [f64]) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

fn main() {
    println!(
        "build={}",
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        }
    );
    for (name, radius) in [("3x3", 1), ("5x5", 2), ("7x7", 3)] {
        let mut b = WorldBridge::new(12345, radius);
        b.set_player(8.5, 20.0, 8.5);
        b.pump_all();
        let chunks = b.active_chunks();
        let voxels = b.active_voxels();

        let (mut t_cube, mut t_face) = (Vec::new(), Vec::new());
        let (mut cube_bytes, mut face_bytes, mut faces) = (0usize, 0usize, 0usize);
        for _ in 0..15 {
            cube_bytes = 0;
            let s = Instant::now();
            for c in &chunks {
                cube_bytes += b.chunk_instance_buffer(c[0], c[1]).unwrap().len() * 4;
            }
            t_cube.push(s.elapsed().as_secs_f64() * 1000.0 / chunks.len() as f64);

            face_bytes = 0;
            faces = 0;
            let s = Instant::now();
            for c in &chunks {
                let m = b.chunk_mesh(c[0], c[1]).unwrap();
                faces += m.faces;
                face_bytes += m.byte_size();
            }
            t_face.push(s.elapsed().as_secs_f64() * 1000.0 / chunks.len() as f64);
        }
        println!("{name}: chunks {}  voxels {voxels}", chunks.len());
        println!(
            "  cubes: drawable elements {voxels}  (faces submitted {})  triangles {}  buffer {} KB  build median {:.3} ms/chunk",
            voxels * 6,
            voxels * 12,
            cube_bytes / 1024,
            median(&mut t_cube)
        );
        println!(
            "  faces: drawable elements {faces}  triangles {}  arrays {} KB  build median {:.3} ms/chunk",
            faces * 2,
            face_bytes / 1024,
            median(&mut t_face)
        );
        println!(
            "  ratio: elements {:.1}x fewer, triangles {:.1}x fewer, bytes {:.1}x {}",
            voxels as f64 / faces as f64,
            (voxels * 12) as f64 / (faces * 2) as f64,
            if cube_bytes > face_bytes {
                cube_bytes as f64 / face_bytes as f64
            } else {
                face_bytes as f64 / cube_bytes as f64
            },
            if cube_bytes > face_bytes {
                "smaller"
            } else {
                "larger"
            },
        );
        let _ = FLOATS_PER_INSTANCE;
    }
}
