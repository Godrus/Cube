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

// ---------------------------------------------------------------- partial rebuild after edits

use std::collections::{BTreeMap, BTreeSet};

fn all_meshes(b: &WorldBridge) -> BTreeMap<[i32; 2], ChunkMesh> {
    b.active_chunks()
        .into_iter()
        .map(|c| (c, b.chunk_mesh(c[0], c[1]).unwrap()))
        .collect()
}

/// What a face renderer must rebuild after edits: edited chunks plus neighbours across edited borders.
fn rebuild_set(b: &mut WorldBridge) -> BTreeSet<[i32; 2]> {
    let mut s: BTreeSet<[i32; 2]> = b.take_modified().into_iter().collect();
    s.extend(b.take_border_touched());
    s
}

fn set_of(cs: &[[i32; 2]]) -> BTreeSet<[i32; 2]> {
    cs.iter().copied().collect()
}

fn unchanged_outside(
    before: &BTreeMap<[i32; 2], ChunkMesh>,
    after: &BTreeMap<[i32; 2], ChunkMesh>,
    rebuilt: &BTreeSet<[i32; 2]>,
) {
    for (c, m) in before {
        if !rebuilt.contains(c) {
            assert_eq!(
                Some(m),
                after.get(c),
                "chunk {c:?} changed but was not in the rebuild set"
            );
        }
    }
}

/// Highest y that is solid at all given columns.
fn shared_solid_y(b: &WorldBridge, cols: &[(i32, i32)]) -> i32 {
    cols.iter()
        .map(|&(x, z)| b.ground_height(x, z))
        .min()
        .unwrap()
}

fn world5() -> WorldBridge {
    let mut b = WorldBridge::new(777, 2);
    b.set_player(8.0, 20.0, 8.0);
    b.pump_all();
    b.take_added();
    b
}

#[test]
fn edit_inside_a_chunk_rebuilds_only_that_chunk() {
    let mut b = world5();
    let before = all_meshes(&b);
    let y = b.ground_height(8, 8) + 3;
    assert!(b.edit_voxel(8, y, 8, 4));
    let set = rebuild_set(&mut b);
    assert_eq!(set, set_of(&[[0, 0]]));
    let after = all_meshes(&b);
    assert_ne!(before[&[0, 0]], after[&[0, 0]]);
    unchanged_outside(&before, &after, &set);
}

#[test]
fn edit_on_a_border_rebuilds_the_neighbour_and_the_neighbour_really_changes() {
    let mut b = world5();
    let before = all_meshes(&b);
    // Remove a solid voxel at the +x border of chunk (0,0): chunk (1,0) gets a new exposed face.
    let y = shared_solid_y(&b, &[(15, 8), (16, 8)]);
    assert!(b.edit_voxel(15, y, 8, 0));
    let set = rebuild_set(&mut b);
    assert_eq!(set, set_of(&[[0, 0], [1, 0]]));
    let after = all_meshes(&b);
    assert_ne!(
        before[&[1, 0]],
        after[&[1, 0]],
        "rebuilding the neighbour is necessary"
    );
    assert_eq!(after[&[1, 0]].faces, before[&[1, 0]].faces + 1);
    unchanged_outside(&before, &after, &set);
}

#[test]
fn edits_on_every_side_including_negative_coordinates() {
    // (x, z, expected neighbour) for voxels on the four borders of chunk (0,0) and across zero.
    let cases: [(i32, i32, [i32; 2]); 4] = [
        (0, 8, [-1, 0]),
        (15, 8, [1, 0]),
        (8, 0, [0, -1]),
        (8, 15, [0, 1]),
    ];
    for (x, z, n) in cases {
        let mut b = world5();
        let before = all_meshes(&b);
        let y = b.ground_height(x, z);
        assert!(b.edit_voxel(x, y, z, 0));
        let set = rebuild_set(&mut b);
        assert_eq!(set, set_of(&[[0, 0], n]), "voxel ({x},{z})");
        unchanged_outside(&before, &all_meshes(&b), &set);
    }
    // A voxel in chunk -1 at local 15 touches chunk 0.
    let mut b = world5();
    let y = b.ground_height(-1, 8);
    assert!(b.edit_voxel(-1, y, 8, 0));
    assert_eq!(rebuild_set(&mut b), set_of(&[[-1, 0], [0, 0]]));
}

#[test]
fn corner_voxel_rebuilds_two_neighbours_but_not_the_diagonal() {
    let mut b = world5();
    let before = all_meshes(&b);
    let y = shared_solid_y(&b, &[(15, 15), (16, 15), (15, 16)]);
    assert!(b.edit_voxel(15, y, 15, 0));
    let set = rebuild_set(&mut b);
    assert_eq!(set, set_of(&[[0, 0], [1, 0], [0, 1]]));
    assert!(!set.contains(&[1, 1]));
    let after = all_meshes(&b);
    assert_ne!(before[&[1, 0]], after[&[1, 0]]);
    assert_ne!(before[&[0, 1]], after[&[0, 1]]);
    unchanged_outside(&before, &after, &set);
}

#[test]
fn far_chunks_are_never_rebuilt_and_unloaded_neighbours_are_skipped() {
    let mut b = world5();
    let y = b.ground_height(8, 8) + 3;
    b.edit_voxel(8, y, 8, 4);
    let set = rebuild_set(&mut b);
    assert!(set.iter().all(|c| c[0].abs() <= 1 && c[1].abs() <= 1));
    assert_eq!(set.len(), 1, "25 loaded chunks, one rebuilt");

    // Edit at the outer border of the loaded area: the unloaded neighbour is not reported.
    let mut r1 = WorldBridge::new(777, 1);
    r1.set_player(8.0, 20.0, 8.0);
    r1.pump_all();
    r1.take_added();
    let outer = r1.active_chunks().into_iter().map(|c| c[0]).max().unwrap();
    let x = outer * 16 + 15;
    let y = r1.ground_height(x, 8);
    assert!(r1.edit_voxel(x, y, 8, 0));
    let set = rebuild_set(&mut r1);
    assert_eq!(set, set_of(&[[outer, 0]]));
}

#[test]
fn an_edit_that_changes_nothing_reports_nothing() {
    let mut b = world5();
    let y = b.ground_height(15, 8);
    let m = 1; // grass is already there
    assert!(!b.edit_voxel(15, y, 8, m));
    assert!(rebuild_set(&mut b).is_empty());
}

/// 120 pseudo-random edits (placing and removing, border and interior): chunks outside the rebuild
/// set never change, so rebuilding only the set is always enough.
#[test]
fn partial_rebuild_is_always_sufficient() {
    let mut b = world5();
    let mut state = 0x1234_5678_9abc_def0u64;
    let mut next = |n: i32| {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((state >> 33) as i32).rem_euclid(n)
    };
    for i in 0..120 {
        let before = all_meshes(&b);
        // Bias towards borders: half of the edits sit on a chunk border column.
        let cx = next(5) - 2;
        let cz = next(5) - 2;
        let lx = if next(2) == 0 {
            [0, 15][next(2) as usize]
        } else {
            next(16)
        };
        let lz = if next(2) == 0 {
            [0, 15][next(2) as usize]
        } else {
            next(16)
        };
        let (x, z) = (cx * 16 + lx, cz * 16 + lz);
        let h = b.ground_height(x, z);
        let y = (h + next(7) - 3).clamp(0, 31);
        let mat = [0u16, 4][next(2) as usize];
        b.edit_voxel(x, y, z, mat);
        let set = rebuild_set(&mut b);
        unchanged_outside(&before, &all_meshes(&b), &set);
        assert!(
            set.len() <= 3,
            "edit {i}: a single voxel rebuilds at most 3 chunks"
        );
    }
}
