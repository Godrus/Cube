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
        // Mesh sections: one section (16^3 brick) vs the whole chunk (two sections).
        let (mut t_chunk, mut t_s0, mut t_s1) = (Vec::new(), Vec::new(), Vec::new());
        for _ in 0..15 {
            let (mut c, mut s0, mut s1) = (0.0, 0.0, 0.0);
            for ch in &chunks {
                let t = Instant::now();
                std::hint::black_box(b.chunk_mesh(ch[0], ch[1]));
                c += t.elapsed().as_secs_f64();
                let t = Instant::now();
                std::hint::black_box(b.section_mesh(ch[0], 0, ch[1]));
                s0 += t.elapsed().as_secs_f64();
                let t = Instant::now();
                std::hint::black_box(b.section_mesh(ch[0], 1, ch[1]));
                s1 += t.elapsed().as_secs_f64();
            }
            let n = chunks.len() as f64 / 1000.0;
            t_chunk.push(c / n);
            t_s0.push(s0 / n);
            t_s1.push(s1 / n);
        }
        println!(
            "  sections: whole chunk {:.3} ms, section 0 (y 0-15) {:.3} ms, section 1 (y 16-31) {:.3} ms",
            median(&mut t_chunk),
            median(&mut t_s0),
            median(&mut t_s1)
        );
        // Greedy (per section, same-material rectangles) vs faces (unit faces), both built per section.
        let (mut t_faces, mut t_greedy) = (Vec::new(), Vec::new());
        let (mut gq, mut garea, mut gbytes) = (0usize, 0usize, 0usize);
        for _ in 0..15 {
            let (mut f, mut g) = (0.0, 0.0);
            gq = 0;
            garea = 0;
            gbytes = 0;
            for ch in &chunks {
                for bi in 0..2 {
                    let t = Instant::now();
                    std::hint::black_box(b.section_mesh(ch[0], bi, ch[1]));
                    f += t.elapsed().as_secs_f64();
                    let t = Instant::now();
                    let m = b.section_mesh_greedy(ch[0], bi, ch[1]).unwrap();
                    g += t.elapsed().as_secs_f64();
                    gq += m.faces;
                    garea += m.area;
                    gbytes += m.byte_size();
                }
            }
            let n = chunks.len() as f64 / 1000.0;
            t_faces.push(f / n);
            t_greedy.push(g / n);
        }
        println!(
            "  greedy: quads {gq} (covering {garea} unit faces)  triangles {}  arrays {} KB  build per chunk (2 sections): faces {:.3} ms, greedy {:.3} ms",
            gq * 2,
            gbytes / 1024,
            median(&mut t_faces),
            median(&mut t_greedy)
        );
        // Estimate of a T-junction-free greedy mesh: every lattice vertex on a quad's boundary is emitted and the
        // quad is fan-triangulated (2 * (w + h) boundary vertices, 2 * (w + h) - 2 triangles). Not implemented.
        let (mut tj_tris, mut tj_verts) = (0usize, 0usize);
        for ch in &chunks {
            for bi in 0..2 {
                for q in b.section_quads(ch[0], bi, ch[1]).unwrap() {
                    let nb = (2 * (q.size[0] + q.size[1])) as usize;
                    tj_verts += nb;
                    tj_tris += nb - 2;
                }
            }
        }
        println!(
            "  greedy without T-junctions (estimate): {tj_verts} vertices, {tj_tris} triangles (current greedy {} triangles, faces {})",
            gq * 2,
            faces * 2
        );
        let _ = FLOATS_PER_INSTANCE;
    }
}
