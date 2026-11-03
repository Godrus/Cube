//! Tests of incremental mesh sections (experimental faces renderer): which sections an edit dirties,
//! that rebuilding only those is always enough, and that incremental and full rebuild agree.
//! Section = one 16^3 brick of a chunk: `[chunk x, brick, chunk z]`, brick 0 = y 0..15, 1 = y 16..31.

use std::collections::{BTreeMap, BTreeSet};

use cube_godot_adapter::world_bridge::{ChunkMesh, WorldBridge};

type Sec = [i32; 3];

fn world(radius: i32) -> WorldBridge {
    let mut b = WorldBridge::new(777, radius);
    b.set_player(8.0, 20.0, 8.0);
    b.pump_all();
    b.take_added();
    b
}

fn dirty(b: &mut WorldBridge) -> BTreeSet<Sec> {
    b.take_dirty_sections().into_iter().collect()
}

fn set_of(v: &[Sec]) -> BTreeSet<Sec> {
    v.iter().copied().collect()
}

fn all_sections(b: &WorldBridge) -> BTreeMap<Sec, ChunkMesh> {
    let mut m = BTreeMap::new();
    for c in b.active_chunks() {
        for bi in 0..2 {
            m.insert([c[0], bi, c[1]], b.section_mesh(c[0], bi, c[1]).unwrap());
        }
    }
    m
}

fn unchanged_outside(
    before: &BTreeMap<Sec, ChunkMesh>,
    after: &BTreeMap<Sec, ChunkMesh>,
    d: &BTreeSet<Sec>,
) {
    for (k, m) in before {
        if !d.contains(k) {
            assert_eq!(
                Some(m),
                after.get(k),
                "section {k:?} changed but was not dirty"
            );
        }
    }
}

/// Toggles a voxel to a placed block and checks the dirty sections and that nothing else changed.
fn edit_expect(b: &mut WorldBridge, p: (i32, i32, i32), expect: &[Sec]) {
    let before = all_sections(b);
    assert!(
        b.edit_voxel(p.0, p.1, p.2, 4),
        "edit {p:?} must change the voxel"
    );
    let d = dirty(b);
    assert_eq!(d, set_of(expect), "edit at {p:?}");
    unchanged_outside(&before, &all_sections(b), &d);
}

#[test]
fn edit_inside_a_section_dirties_one_section() {
    let mut b = world(2);
    edit_expect(&mut b, (8, 10, 8), &[[0, 0, 0]]);
    edit_expect(&mut b, (8, 22, 8), &[[0, 1, 0]]);
}

#[test]
fn edit_on_a_brick_boundary_dirties_two_sections() {
    let mut b = world(2);
    edit_expect(&mut b, (8, 15, 8), &[[0, 0, 0], [0, 1, 0]]);
    let mut b = world(2);
    edit_expect(&mut b, (8, 16, 8), &[[0, 0, 0], [0, 1, 0]]);
}

#[test]
fn edit_on_a_chunk_border_dirties_the_section_of_the_neighbour_chunk() {
    for (p, n) in [
        ((15, 10, 8), [1, 0, 0]),
        ((0, 10, 8), [-1, 0, 0]),
        ((8, 10, 15), [0, 0, 1]),
        ((8, 10, 0), [0, 0, -1]),
    ] {
        let mut b = world(2);
        edit_expect(&mut b, p, &[[0, 0, 0], n]);
    }
    // Across zero: voxel x = -1 belongs to chunk -1 and touches chunk 0.
    let mut b = world(2);
    edit_expect(&mut b, (-1, 10, 8), &[[-1, 0, 0], [0, 0, 0]]);
}

#[test]
fn corner_edits_dirty_only_the_necessary_sections() {
    // Horizontal corner: two neighbours, never the diagonal chunk.
    let mut b = world(2);
    edit_expect(&mut b, (15, 10, 15), &[[0, 0, 0], [1, 0, 0], [0, 0, 1]]);
    // Horizontal corner on a brick boundary: own two bricks + the two neighbours' section of this brick.
    let mut b = world(2);
    edit_expect(
        &mut b,
        (15, 15, 15),
        &[[0, 0, 0], [0, 1, 0], [1, 0, 0], [0, 0, 1]],
    );
    // Chunk border and brick boundary, upper side.
    let mut b = world(2);
    edit_expect(
        &mut b,
        (0, 16, 0),
        &[[0, 1, 0], [0, 0, 0], [-1, 1, 0], [0, 1, -1]],
    );
}

#[test]
fn world_limits_do_not_create_sections() {
    let mut b = world(2);
    edit_expect(&mut b, (8, 0, 8), &[[0, 0, 0]]); // nothing below y = 0
    let mut b = world(2);
    edit_expect(&mut b, (8, 31, 8), &[[0, 1, 0]]); // nothing above y = 31
}

#[test]
fn unloaded_neighbour_sections_are_skipped() {
    let mut b = world(1);
    let outer = b.active_chunks().into_iter().map(|c| c[0]).max().unwrap();
    let x = outer * 16 + 15;
    assert!(b.edit_voxel(x, 10, 8, 4));
    assert_eq!(dirty(&mut b), set_of(&[[outer, 0, 0]]));
}

#[test]
fn an_empty_operation_dirties_nothing() {
    let mut b = world(2);
    let y = b.ground_height(15, 8);
    // Grass is already there; air above the world and a chunk that is not loaded change nothing either.
    assert!(!b.edit_voxel(15, y, 8, 1));
    assert!(!b.edit_voxel(15, 40, 8, 4));
    assert!(!b.edit_voxel(10_000, 10, 8, 4));
    assert!(dirty(&mut b).is_empty());
    assert_eq!(b.undo_depth(), 0, "no change, no undo step");
}

#[test]
fn far_sections_never_change_and_never_become_dirty() {
    let mut b = world(2);
    let before = all_sections(&b);
    assert!(b.edit_voxel(8, 10, 8, 4));
    let d = dirty(&mut b);
    assert_eq!(d.len(), 1);
    let after = all_sections(&b);
    unchanged_outside(&before, &after, &d);
    assert_eq!(before.len(), 50, "25 chunks x 2 sections");
}

fn faces_of(m: &ChunkMesh) -> Vec<(Vec<i32>, Vec<i32>, Vec<u32>)> {
    (0..m.faces)
        .map(|f| {
            let v = &m.positions[f * 12..f * 12 + 12];
            let n = &m.normals[f * 12..f * 12 + 3];
            let c = &m.colors[f * 16..f * 16 + 4];
            (
                v.iter().map(|x| *x as i32).collect(),
                n.iter().map(|x| *x as i32).collect(),
                c.iter().map(|x| (x * 255.0).round() as u32).collect(),
            )
        })
        .collect()
}

#[test]
fn chunk_mesh_equals_its_two_sections_appended() {
    let b = world(2);
    for c in b.active_chunks() {
        let full = b.chunk_mesh(c[0], c[1]).unwrap();
        let mut joined = b.section_mesh(c[0], 0, c[1]).unwrap();
        joined.append(&b.section_mesh(c[0], 1, c[1]).unwrap());
        assert_eq!(faces_of(&full), faces_of(&joined), "chunk {c:?}");
        assert_eq!(full, joined);
        assert_eq!(full.content_hash(), joined.content_hash());
    }
}

/// The client keeps one mesh per section and refreshes only dirty sections. After every random edit
/// (placing, removing, borders, brick boundary, corners) the cache must equal a full rebuild of the
/// whole world: same faces in the same order, same hash.
#[test]
fn incremental_rebuild_equals_full_rebuild_after_random_edits() {
    let mut b = world(2);
    let mut cache = all_sections(&b);
    let mut state = 0xfeed_beef_1234_5678u64;
    let mut next = |n: i32| {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((state >> 33) as i32).rem_euclid(n)
    };
    let mut rebuilt_total = 0;
    for i in 0..80 {
        let cx = next(5) - 2;
        let cz = next(5) - 2;
        let edge = |n: &mut dyn FnMut(i32) -> i32| {
            if n(2) == 0 {
                [0, 15][n(2) as usize]
            } else {
                n(16)
            }
        };
        let (lx, lz) = (edge(&mut next), edge(&mut next));
        let y = [0, 1, 14, 15, 16, 17, 30, 31, next(32)][next(9) as usize];
        let (x, z) = (cx * 16 + lx, cz * 16 + lz);
        let mat = [0u16, 4, 2][next(3) as usize];
        b.edit_voxel(x, y, z, mat);
        let d = dirty(&mut b);
        assert!(d.len() <= 4, "edit {i}: {d:?}");
        rebuilt_total += d.len();
        for s in &d {
            cache.insert(*s, b.section_mesh(s[0], s[1], s[2]).unwrap());
        }
        let full = all_sections(&b);
        assert_eq!(cache, full, "edit {i} at ({x},{y},{z}) material {mat}");
        for c in b.active_chunks() {
            let whole = b.chunk_mesh(c[0], c[1]).unwrap();
            let mut joined = cache[&[c[0], 0, c[1]]].clone();
            joined.append(&cache[&[c[0], 1, c[1]]]);
            assert_eq!(whole.content_hash(), joined.content_hash());
            assert_eq!(faces_of(&whole), faces_of(&joined));
        }
    }
    assert!(rebuilt_total < 80 * 4);
}

#[test]
fn undo_and_redo_restore_meshes_exactly_and_report_the_same_sections() {
    let mut b = world(2);
    let original = all_sections(&b);
    let y = b.ground_height(15, 8);
    assert!(b.edit_voxel(15, y, 8, 0));
    assert_eq!(b.edit_around(8, 20, 8), 27);
    let d_edit = dirty(&mut b);
    let edited = all_sections(&b);
    assert_ne!(original, edited);
    assert_eq!(b.undo_depth(), 2);

    assert_eq!(b.undo(), 27);
    let d1 = dirty(&mut b);
    assert!(!d1.is_empty());
    assert_eq!(b.undo(), 1);
    let d2 = dirty(&mut b);
    assert_eq!(all_sections(&b), original, "undo restores every mesh");
    assert_eq!(d1.union(&d2).copied().collect::<BTreeSet<_>>(), d_edit);
    assert_eq!(b.undo(), 0, "nothing left to undo");

    assert_eq!(b.redo(), 1);
    assert_eq!(b.redo(), 27);
    dirty(&mut b);
    assert_eq!(all_sections(&b), edited, "redo restores the edited meshes");
    assert_eq!(b.redo(), 0);
    // A new edit clears the redo stack.
    assert_eq!(b.undo(), 27);
    assert!(b.edit_voxel(8, 10, 8, 4));
    assert_eq!(b.redo_depth(), 0);
}
