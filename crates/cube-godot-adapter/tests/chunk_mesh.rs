//! Tests of the experimental face-culled renderer data (`WorldBridge::chunk_mesh`), pure Rust.

use cube_godot_adapter::world_bridge::{ChunkMesh, WorldBridge};

const DIRS: [[i32; 3]; 6] = [
    [1, 0, 0],
    [-1, 0, 0],
    [0, 1, 0],
    [0, -1, 0],
    [0, 0, 1],
    [0, 0, -1],
];

fn world(radius: i32) -> WorldBridge {
    let mut b = WorldBridge::new(777, radius);
    b.set_player(8.0, 20.0, 8.0);
    b.pump_all();
    b
}

/// Independent definition: a face is visible iff its voxel is solid and the neighbour is air, not
/// loaded, or above the world; below the world counts as solid.
fn expected_faces(b: &WorldBridge, cx: i32, cz: i32) -> usize {
    let solid: std::collections::HashSet<(i32, i32, i32)> = b
        .active_chunks()
        .iter()
        .flat_map(|c| {
            let v = b.chunk_voxels(c[0], c[1]).unwrap();
            v.chunks(4).map(|q| (q[0], q[1], q[2])).collect::<Vec<_>>()
        })
        .collect();
    let v = b.chunk_voxels(cx, cz).unwrap();
    let mut n = 0;
    for q in v.chunks(4) {
        for d in DIRS {
            let nb = (q[0] + d[0], q[1] + d[1], q[2] + d[2]);
            if nb.1 < 0 || solid.contains(&nb) {
                continue;
            }
            n += 1;
        }
    }
    n
}

#[test]
fn face_count_matches_the_brute_force_definition_in_every_chunk() {
    let b = world(2);
    for c in b.active_chunks() {
        let mesh = b.chunk_mesh(c[0], c[1]).unwrap();
        assert_eq!(mesh.faces, expected_faces(&b, c[0], c[1]), "chunk {c:?}");
        assert_eq!(mesh.vertex_count(), mesh.faces * 4);
        assert_eq!(mesh.indices.len(), mesh.faces * 6);
        assert_eq!(mesh.normals.len(), mesh.positions.len());
        assert_eq!(mesh.colors.len(), mesh.vertex_count() * 4);
    }
}

#[test]
fn far_fewer_faces_than_cube_per_voxel() {
    let b = world(1);
    let voxels = b.chunk_voxel_count(0, 0).unwrap();
    let mesh = b.chunk_mesh(0, 0).unwrap();
    assert!(mesh.faces > 256, "top surface at least");
    assert!(
        mesh.faces * 10 < voxels * 6,
        "{} faces vs {} drawn cube faces",
        mesh.faces,
        voxels * 6
    );
}

#[test]
fn faces_point_outwards_and_wind_clockwise_seen_from_outside() {
    let b = world(1);
    let m = b.chunk_mesh(0, 0).unwrap();
    for t in m.indices.chunks(3) {
        let p = |i: u32| {
            let i = i as usize * 3;
            [
                m.positions[i] as f64,
                m.positions[i + 1] as f64,
                m.positions[i + 2] as f64,
            ]
        };
        let (a, bb, c) = (p(t[0]), p(t[1]), p(t[2]));
        let e1 = [bb[0] - a[0], bb[1] - a[1], bb[2] - a[2]];
        let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let cross = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        let i = t[0] as usize * 3;
        let n = [
            m.normals[i] as f64,
            m.normals[i + 1] as f64,
            m.normals[i + 2] as f64,
        ];
        let dot = cross[0] * n[0] + cross[1] * n[1] + cross[2] * n[2];
        assert!(dot < 0.0, "clockwise from outside means cross . normal < 0");
    }
}

#[test]
fn border_faces_hide_when_the_neighbour_loads_and_return_when_it_unloads() {
    let mut b = WorldBridge::new(777, 0);
    b.set_player(8.0, 20.0, 8.0);
    b.pump_all();
    let alone: ChunkMesh = b.chunk_mesh(0, 0).unwrap();
    // Load chunk (1,0) next to it: the shared border becomes internal.
    let mut b2 = WorldBridge::new(777, 1);
    b2.set_player(8.0, 20.0, 8.0);
    b2.pump_all();
    let with_neighbours = b2.chunk_mesh(0, 0).unwrap();
    assert!(with_neighbours.faces < alone.faces);
    assert_eq!(with_neighbours.faces, expected_faces(&b2, 0, 0));
    // Moving away unloads the neighbours' influence again.
    b2.set_player(8.0 + 160.0, 20.0, 8.0);
    b2.pump_all();
    assert!(b2.chunk_mesh(0, 0).is_none(), "unloaded chunk has no mesh");
}

#[test]
fn edits_change_the_mesh_and_stay_consistent() {
    let mut b = world(1);
    let before = b.chunk_mesh(0, 0).unwrap().faces;
    let h = b.ground_height(8, 8);
    assert_eq!(b.edit_around(8, h + 3, 8), 27);
    let after = b.chunk_mesh(0, 0).unwrap();
    assert_eq!(after.faces, expected_faces(&b, 0, 0));
    assert!(after.faces > before, "a floating 3x3x3 block adds faces");
    // An edit across a chunk border changes both chunks consistently.
    assert!(b.edit_around(16, h + 3, 8) > 0);
    for c in [[0, 0], [1, 0]] {
        assert_eq!(
            b.chunk_mesh(c[0], c[1]).unwrap().faces,
            expected_faces(&b, c[0], c[1])
        );
    }
}

#[test]
fn mesh_is_deterministic() {
    let a = world(1).chunk_mesh(0, 0).unwrap();
    let c = world(1).chunk_mesh(0, 0).unwrap();
    assert_eq!(a, c);
}
