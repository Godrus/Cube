//! Density comparison: `cargo run --release -p cube-character --example measure`.

use std::time::Instant;

use cube_character::{CharacterAsset, PartId};

fn main() {
    println!(
        "res/unit | tall(vox) | lod | voxels | triangles | vertices | mesh KiB | grid KiB | build ms"
    );
    for res in [32u32, 48, 64, 80] {
        let t = Instant::now();
        let a = CharacterAsset::build(res);
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        for lod in 0..3 {
            println!(
                "{res:>8} | {:>9} | {lod:>3} | {:>6} | {:>9} | {:>8} | {:>8.1} | {:>8.1} | {}",
                (1.8 * a.parts[0].lods[lod].res) as u32,
                a.voxel_count(lod),
                a.triangle_count(lod),
                a.vertex_count(lod),
                a.mesh_bytes(lod) as f64 / 1024.0,
                a.grid_bytes(lod) as f64 / 1024.0,
                if lod == 0 {
                    format!("{ms:.1}")
                } else {
                    "-".into()
                },
            );
        }
        if res == 80 {
            for p in &a.parts {
                println!(
                    "   part {:<6} lod0 {:>6} vox {:>6} tris | lod1 {:>5} tris | lod2 {:>5} tris",
                    p.id.name(),
                    p.lods[0].grid.count(),
                    p.lods[0].mesh.triangle_count(),
                    p.lods[1].mesh.triangle_count(),
                    p.lods[2].mesh.triangle_count()
                );
            }
            let _ = PartId::ALL;
        }
    }
}
