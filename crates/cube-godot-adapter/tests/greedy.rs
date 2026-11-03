//! Tests of the experimental greedy mesher (material-aware, per mesh section). The reference is the
//! `faces` representation: the list of visible unit voxel faces. Greedy quads are compared with it as
//! normalised geometry (position, normal, size, material), not as pixels.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use cube_godot_adapter::world_bridge::{ChunkMesh, Quad, UnitFace, WorldBridge};

type Sec = [i32; 3];

fn world(radius: i32) -> WorldBridge {
    let mut b = WorldBridge::new(777, radius);
    b.set_player(8.0, 20.0, 8.0);
    b.pump_all();
    b.take_added();
    b
}

fn sections(b: &WorldBridge) -> Vec<Sec> {
    let mut v = Vec::new();
    for c in b.active_chunks() {
        for bi in 0..2 {
            v.push([c[0], bi, c[1]]);
        }
    }
    v
}

fn quads(b: &WorldBridge, s: Sec) -> Vec<Quad> {
    b.section_quads(s[0], s[1], s[2]).unwrap()
}

fn expand(qs: &[Quad]) -> Vec<UnitFace> {
    let mut v: Vec<UnitFace> = qs.iter().flat_map(|q| q.unit_faces()).collect();
    v.sort();
    v
}

fn reference(b: &WorldBridge, s: Sec) -> Vec<UnitFace> {
    let mut v = b.section_unit_faces(s[0], s[1], s[2]).unwrap();
    v.sort();
    v
}

/// Normalised quad list of the whole world: sorted by normal, position, size, material.
fn normalised(b: &WorldBridge) -> Vec<Quad> {
    let mut v: Vec<Quad> = sections(b).into_iter().flat_map(|s| quads(b, s)).collect();
    v.sort();
    v
}

fn assert_same_surface(b: &WorldBridge, what: &str) {
    for s in sections(b) {
        let g = expand(&quads(b, s));
        let r = reference(b, s);
        // Equality of sorted lists also rules out gaps, overlaps (duplicates) and wrong materials.
        assert_eq!(g, r, "{what}: section {s:?}");
        // The faces mesh itself agrees with the reference count.
        let m = b.section_mesh(s[0], s[1], s[2]).unwrap();
        assert_eq!(m.faces, r.len(), "{what}: faces mesh of {s:?}");
    }
}

#[test]
fn greedy_and_faces_represent_the_same_visible_surface_in_every_section() {
    for (seed, radius) in [(777u64, 2), (1, 1), (0xC0FFEE, 2)] {
        let mut b = WorldBridge::new(seed, radius);
        b.set_player(-20.0, 20.0, 40.0);
        b.pump_all();
        assert_same_surface(&b, &format!("seed {seed}"));
    }
}

#[test]
fn every_quad_has_one_material_and_covers_only_voxels_of_that_material() {
    let b = world(2);
    let mut mat: HashMap<[i32; 3], u16> = HashMap::new();
    for c in b.active_chunks() {
        for v in b.chunk_voxels(c[0], c[1]).unwrap().chunks(4) {
            mat.insert([v[0], v[1], v[2]], v[3] as u16);
        }
    }
    let mut merged = 0;
    for s in sections(&b) {
        for q in quads(&b, s) {
            if q.size[0] * q.size[1] > 1 {
                merged += 1;
            }
            for (p, n, m) in q.unit_faces() {
                assert_eq!(mat[&p], m, "voxel {p:?} in quad {q:?}");
                assert_eq!(n, q.normal);
            }
        }
    }
    assert!(merged > 100, "terrain has large flat regions to merge");
}

#[test]
fn different_materials_are_not_merged() {
    let mut b = world(2);
    // A 3x3 floating slab of placed blocks (material 4): one top quad of 3x3.
    let y = 25;
    assert_eq!(b.fill_box([6, y, 6], [8, y, 8], 4), 9);
    let top = |b: &WorldBridge| -> Vec<Quad> {
        quads(b, [0, 1, 0])
            .into_iter()
            .filter(|q| q.normal == [0, 1, 0] && q.voxel[1] == y)
            .collect()
    };
    let t = top(&b);
    assert_eq!(t.len(), 1);
    assert_eq!((t[0].size, t[0].material), ([3, 3], 4));
    // Replace the centre by dirt (material 2): the top face can no longer be one quad.
    assert!(b.edit_voxel(7, y, 7, 2));
    let t = top(&b);
    assert!(
        t.len() >= 5,
        "ring of material 4 (4 rectangles) + the dirt cell, got {t:?}"
    );
    assert!(t.iter().all(|q| q.size[0] * q.size[1] <= 3));
    let dirt: Vec<_> = t.iter().filter(|q| q.material == 2).collect();
    assert_eq!(dirt.len(), 1);
    assert_eq!((dirt[0].voxel, dirt[0].size), ([7, y, 7], [1, 1]));
    let area: i32 = t.iter().map(|q| q.size[0] * q.size[1]).sum();
    assert_eq!(area, 9);
    assert_same_surface(&b, "after material change");
}

#[test]
fn quads_do_not_cross_section_boundaries_and_chunks_keep_their_border_faces() {
    let b = world(2);
    for s in sections(&b) {
        let (ox, oy, oz) = (s[0] * 16, s[1] * 16, s[2] * 16);
        for q in quads(&b, s) {
            let (u, v) = q.axes();
            for k in 0..3 {
                let lo = q.voxel[k];
                let hi = lo + (q.size[0] - 1) * u[k] + (q.size[1] - 1) * v[k];
                let origin = [ox, oy, oz][k];
                assert!(
                    lo >= origin && hi < origin + 16,
                    "quad {q:?} leaves section {s:?}"
                );
            }
        }
    }
    // Chunk borders lose nothing: outer faces of the loaded area are present (neighbour unloaded = air).
    assert_same_surface(&b, "borders");
    let outer_west: usize = (0..2)
        .map(|bi| {
            quads(&b, [-2, bi, 0])
                .iter()
                .filter(|q| q.normal == [-1, 0, 0] && q.voxel[0] == -32)
                .map(|q| (q.size[0] * q.size[1]) as usize)
                .sum::<usize>()
        })
        .sum();
    assert!(
        outer_west > 0,
        "the border faces towards the unloaded west are drawn"
    );
}

#[test]
fn removing_a_voxel_removes_its_face_and_exposes_the_neighbours() {
    let mut b = world(2);
    let y = b.ground_height(8, 8);
    let before = expand(&quads(&b, [0, y / 16, 0]));
    let top = ([8, y, 8], [0, 1, 0], 1u16);
    assert!(before.contains(&top));
    assert!(b.edit_voxel(8, y, 8, 0));
    let after = expand(&quads(&b, [0, y / 16, 0]));
    assert!(
        !after.contains(&top),
        "the removed voxel's top face is gone"
    );
    // The hole exposes the walls of its neighbours and the voxel below (checked against the reference).
    assert_same_surface(&b, "after removal");
    assert!(after
        .iter()
        .any(|f| f.0 == [8, y - 1, 8] && f.1 == [0, 1, 0]));
}

#[test]
fn a_change_in_the_neighbouring_chunk_updates_the_shared_border() {
    let mut b = world(2);
    let y = b.ground_height(15, 8).min(b.ground_height(16, 8));
    let sec = [1, y / 16, 0];
    let before = quads(&b, sec);
    // Remove the voxel at x = 15 (chunk 0); the wall at x = 16 (chunk 1) becomes visible.
    assert!(b.edit_voxel(15, y, 8, 0));
    let dirty: BTreeSet<Sec> = b.take_dirty_sections().into_iter().collect();
    assert!(
        dirty.contains(&sec),
        "the dirty rule reports the neighbour section"
    );
    let after = quads(&b, sec);
    assert_ne!(before, after);
    let wall = (
        [16, y, 8],
        [-1, 0, 0],
        b.chunk_voxels(1, 0)
            .unwrap()
            .chunks(4)
            .find(|v| v[0] == 16 && v[1] == y && v[2] == 8)
            .unwrap()[3] as u16,
    );
    assert!(expand(&after).contains(&wall));
    assert!(!expand(&before).contains(&wall));
    assert_same_surface(&b, "after border edit");
}

#[test]
fn empty_sections_produce_no_quads() {
    let b = world(2);
    // Find a chunk whose upper brick is empty (all heights < 16).
    let empty = sections(&b)
        .into_iter()
        .find(|s| s[1] == 1 && b.section_mesh(s[0], 1, s[2]).unwrap().faces == 0);
    let s = empty.expect("terrain heights 6..19 leave some upper bricks empty");
    assert!(quads(&b, s).is_empty());
    let m = b.section_mesh_greedy(s[0], s[1], s[2]).unwrap();
    assert_eq!((m.faces, m.area, m.positions.len()), (0, 0, 0));
    assert!(b.section_quads(99, 0, 99).is_none(), "unloaded chunk");
    assert!(b.section_quads(0, 2, 0).is_none(), "no third section");
}

#[test]
fn greedy_output_is_deterministic_and_independent_of_generation_order() {
    let a = world(2);
    let a2 = world(2);
    assert_eq!(normalised(&a), normalised(&a2));
    for s in sections(&a) {
        assert_eq!(
            a.section_mesh_greedy(s[0], s[1], s[2]),
            a2.section_mesh_greedy(s[0], s[1], s[2])
        );
    }
    // Same area reached by a different path and a different generation order.
    let mut c = WorldBridge::new(777, 2);
    for (x, z) in [(500.0, 500.0), (-300.0, 80.0), (8.0, 900.0), (8.0, 8.0)] {
        c.set_player(x, 20.0, z);
        c.pump(7);
        c.pump_all();
    }
    c.take_added();
    assert_eq!(normalised(&a), normalised(&c));
    // The mesh buffers are identical too.
    for s in sections(&a) {
        assert_eq!(
            a.section_mesh_greedy(s[0], s[1], s[2])
                .map(|m| m.content_hash()),
            c.section_mesh_greedy(s[0], s[1], s[2])
                .map(|m| m.content_hash())
        );
    }
}

fn greedy_cache(b: &WorldBridge) -> BTreeMap<Sec, ChunkMesh> {
    sections(b)
        .into_iter()
        .map(|s| (s, b.section_mesh_greedy(s[0], s[1], s[2]).unwrap()))
        .collect()
}

/// Client-style cache refreshed only from dirty sections vs a full rebuild after every random edit
/// (placing, removing, other materials, borders, brick boundary, corners, negative chunks).
#[test]
fn partial_rebuild_equals_full_rebuild_of_the_greedy_mesh() {
    let mut b = world(2);
    let mut cache = greedy_cache(&b);
    let mut state = 0x0bad_cafe_5eed_1234u64;
    let mut next = |n: i32| {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((state >> 33) as i32).rem_euclid(n)
    };
    for i in 0..60 {
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
        let y = [0, 1, 14, 15, 16, 17, 30, 31, next(32)][next(9) as usize];
        let mat = [0u16, 4, 2, 3][next(4) as usize];
        b.edit_voxel(cx * 16 + lx, y, cz * 16 + lz, mat);
        for s in b.take_dirty_sections() {
            cache.insert(s, b.section_mesh_greedy(s[0], s[1], s[2]).unwrap());
        }
        assert_eq!(cache, greedy_cache(&b), "edit {i}");
        if i % 10 == 0 {
            assert_same_surface(&b, &format!("edit {i}"));
        }
    }
    assert_same_surface(&b, "end");
}

#[test]
fn undo_and_redo_restore_the_same_greedy_mesh() {
    let mut b = world(2);
    let original = greedy_cache(&b);
    let original_quads = normalised(&b);
    let y = b.ground_height(15, 8);
    assert!(b.edit_voxel(15, y, 8, 0));
    assert_eq!(b.edit_around(8, 20, 8), 27);
    assert_eq!(b.fill_box([-3, 22, -3], [3, 24, 3], 4), 7 * 3 * 7);
    b.take_dirty_sections();
    let edited = greedy_cache(&b);
    assert_ne!(original, edited);
    for _ in 0..3 {
        assert!(b.undo() > 0);
    }
    b.take_dirty_sections();
    assert_eq!(
        greedy_cache(&b),
        original,
        "undo restores every greedy mesh"
    );
    assert_eq!(normalised(&b), original_quads);
    for _ in 0..3 {
        assert!(b.redo() > 0);
    }
    b.take_dirty_sections();
    assert_eq!(
        greedy_cache(&b),
        edited,
        "redo restores the edited greedy mesh"
    );
    assert_same_surface(&b, "after redo");
}

#[test]
fn greedy_reduces_quads_and_keeps_the_covered_area() {
    let b = world(2);
    let (mut faces, mut greedy, mut area) = (0, 0, 0);
    for s in sections(&b) {
        let f = b.section_mesh(s[0], s[1], s[2]).unwrap();
        let g = b.section_mesh_greedy(s[0], s[1], s[2]).unwrap();
        faces += f.faces;
        greedy += g.faces;
        area += g.area;
        assert_eq!(g.area, f.faces, "section {s:?}");
        assert_eq!(g.vertex_count(), g.faces * 4);
        assert_eq!(g.indices.len(), g.faces * 6);
    }
    assert_eq!(area, faces);
    assert!(greedy * 4 < faces, "{greedy} quads vs {faces} faces");
}
