//! Tests that the greedy mesh has no T-junctions (a vertex lying in the interior of another triangle
//! edge), triangulates every quad exactly, and keeps the section/chunk seams conforming, also after
//! edits, undo and redo. See `WorldBridge::section_quad_meshes` for the vertex rule.

use std::collections::{BTreeMap, HashMap, HashSet};

use cube_godot_adapter::world_bridge::{
    edge_on_section_boundary, ChunkMesh, Quad, SeamRule, WorldBridge, GREEDY_SEAM_RULE,
};

const RULES: [SeamRule; 2] = [SeamRule::Minimal, SeamRule::VisibleEdges];

type P = [i32; 3];
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

fn gcd(a: i32, b: i32) -> i32 {
    if b == 0 {
        a.abs()
    } else {
        gcd(b, a % b)
    }
}

fn verts(m: &ChunkMesh) -> Vec<P> {
    m.positions
        .chunks_exact(3)
        .map(|c| [c[0] as i32, c[1] as i32, c[2] as i32])
        .collect()
}

/// Panics if any triangle edge of any mesh has a vertex of any mesh strictly inside it.
fn assert_conforming(meshes: &[ChunkMesh], what: &str) {
    let mut all: HashSet<P> = HashSet::new();
    for m in meshes {
        all.extend(verts(m));
    }
    for m in meshes {
        let v = verts(m);
        for t in m.indices.chunks_exact(3) {
            for e in 0..3 {
                let (a, b) = (v[t[e] as usize], v[t[(e + 1) % 3] as usize]);
                let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
                let g = gcd(gcd(d[0], d[1]), d[2]);
                for k in 1..g {
                    let p = [
                        a[0] + k * d[0] / g,
                        a[1] + k * d[1] / g,
                        a[2] + k * d[2] / g,
                    ];
                    assert!(
                        !all.contains(&p),
                        "{what}: T-junction, vertex {p:?} inside edge {a:?} -> {b:?}"
                    );
                }
            }
        }
    }
}

fn greedy_meshes(b: &WorldBridge) -> Vec<ChunkMesh> {
    sections(b)
        .into_iter()
        .map(|s| b.section_mesh_greedy(s[0], s[1], s[2]).unwrap())
        .collect()
}

/// Meshes of every section built with an explicit seam rule.
fn meshes_with(b: &WorldBridge, rule: SeamRule) -> Vec<ChunkMesh> {
    sections(b)
        .into_iter()
        .map(|s| {
            ChunkMesh::from_quad_meshes(
                &b.section_quad_meshes_with(s[0], s[1], s[2], rule).unwrap(),
            )
        })
        .collect()
}

#[test]
fn the_default_rule_is_visible_edges() {
    assert_eq!(GREEDY_SEAM_RULE, SeamRule::VisibleEdges);
}

#[test]
fn the_whole_greedy_surface_has_no_t_junctions() {
    for (seed, radius, centre) in [
        (777u64, 2, (8.0, 8.0)),
        (1, 1, (-40.0, -70.0)),
        (0xC0FFEE, 2, (300.0, -20.0)),
    ] {
        let mut b = WorldBridge::new(seed, radius);
        b.set_player(centre.0, 20.0, centre.1);
        b.pump_all();
        for rule in RULES {
            assert_conforming(&meshes_with(&b, rule), &format!("seed {seed} {rule:?}"));
        }
    }
}

#[test]
fn every_quad_is_triangulated_exactly_without_degenerate_triangles() {
    let b = world(2);
    let mut quads = 0;
    for (s, rule) in sections(&b).into_iter().flat_map(|s| RULES.map(|r| (s, r))) {
        for qm in b.section_quad_meshes_with(s[0], s[1], s[2], rule).unwrap() {
            quads += 1;
            let q = qm.quad;
            assert_eq!(qm.triangles.len(), qm.vertices.len() - 2, "{q:?}");
            let (u, v) = q.axes();
            let c = q.corners();
            let mut twice_area = 0i64;
            for t in &qm.triangles {
                let (a, bb, cc) = (qm.vertices[t[0]], qm.vertices[t[1]], qm.vertices[t[2]]);
                let e1 = [bb[0] - a[0], bb[1] - a[1], bb[2] - a[2]];
                let e2 = [cc[0] - a[0], cc[1] - a[1], cc[2] - a[2]];
                let cr = [
                    e1[1] as i64 * e2[2] as i64 - e1[2] as i64 * e2[1] as i64,
                    e1[2] as i64 * e2[0] as i64 - e1[0] as i64 * e2[2] as i64,
                    e1[0] as i64 * e2[1] as i64 - e1[1] as i64 * e2[0] as i64,
                ];
                let dot = cr[0] * q.normal[0] as i64
                    + cr[1] * q.normal[1] as i64
                    + cr[2] * q.normal[2] as i64;
                assert!(
                    dot < 0,
                    "clockwise from outside and not degenerate: {q:?} {t:?}"
                );
                twice_area += -dot;
            }
            assert_eq!(twice_area, 2 * (q.size[0] * q.size[1]) as i64, "{q:?}");
            // Every vertex lies on the quad's rectangle (same plane, inside the bounds).
            for p in &qm.vertices {
                for k in 0..3 {
                    let lo = c.iter().map(|x| x[k]).min().unwrap();
                    let hi = c.iter().map(|x| x[k]).max().unwrap();
                    assert!((lo..=hi).contains(&p[k]), "{q:?} vertex {p:?}");
                }
            }
            let _ = (u, v);
            // The boundary vertices are distinct.
            let set: HashSet<P> = qm.vertices.iter().copied().collect();
            assert_eq!(set.len(), qm.vertices.len());
        }
    }
    assert!(quads > 1000);
}

/// An edge stretch borders "another surface" unless the coplanar cell across it holds a visible face of
/// the same material (independent re-implementation from the reference unit faces).
fn stretch_is_visible(q: &Quad, e: usize, k: i32, faces: &HashMap<(P, P), u16>) -> bool {
    let (u, v) = q.axes();
    let (w, h) = (q.size[0], q.size[1]);
    let at = |base: P, steps: &[(P, i32)]| {
        let mut p = base;
        for (d, t) in steps {
            for a in 0..3 {
                p[a] += d[a] * t;
            }
        }
        p
    };
    let outside = match e {
        0 => at(q.voxel, &[(v, k), (u, -1)]),
        1 => at(q.voxel, &[(v, h - 1), (u, k), (v, 1)]),
        2 => at(q.voxel, &[(u, w - 1), (v, h - 1 - k), (u, 1)]),
        _ => at(q.voxel, &[(u, w - 1 - k), (v, -1)]),
    };
    faces.get(&(outside, q.normal)) != Some(&q.material)
}

#[test]
fn boundary_vertices_follow_the_rule() {
    let b = world(2);
    for rule in RULES {
        let mut visible_points = 0;
        for s in sections(&b) {
            let origin = [s[0] * 16, s[1] * 16, s[2] * 16];
            let qms = b.section_quad_meshes_with(s[0], s[1], s[2], rule).unwrap();
            let corners: HashSet<P> = qms.iter().flat_map(|qm| qm.quad.corners()).collect();
            let faces: HashMap<(P, P), u16> = b
                .section_unit_faces(s[0], s[1], s[2])
                .unwrap()
                .into_iter()
                .map(|(p, n, m)| ((p, n), m))
                .collect();
            for qm in &qms {
                let c = qm.quad.corners();
                let have: HashSet<P> = qm.vertices.iter().copied().collect();
                for e in 0..4 {
                    let (a, bb) = (c[e], c[(e + 1) % 4]);
                    let len: i32 = (0..3).map(|k| (bb[k] - a[k]).abs()).sum();
                    let dir = [
                        (bb[0] - a[0]).signum(),
                        (bb[1] - a[1]).signum(),
                        (bb[2] - a[2]).signum(),
                    ];
                    let on_boundary = edge_on_section_boundary(a, bb, origin);
                    for k in 1..len {
                        let p = [a[0] + k * dir[0], a[1] + k * dir[1], a[2] + k * dir[2]];
                        let visible = rule == SeamRule::VisibleEdges
                            && (stretch_is_visible(&qm.quad, e, k - 1, &faces)
                                || stretch_is_visible(&qm.quad, e, k, &faces));
                        // A point is a vertex iff the edge lies on a boundary plane, the point is a corner
                        // of another quad of the section, or (VisibleEdges) it touches a stretch that
                        // borders another surface.
                        assert_eq!(
                            have.contains(&p),
                            on_boundary || visible || corners.contains(&p),
                            "{rule:?} section {s:?} edge {a:?}->{bb:?} point {p:?}"
                        );
                        if visible && !on_boundary && !corners.contains(&p) {
                            visible_points += 1;
                        }
                    }
                }
            }
        }
        if rule == SeamRule::VisibleEdges {
            assert!(
                visible_points > 100,
                "the rule adds vertices beyond the minimum: {visible_points}"
            );
        }
    }
}

#[test]
fn two_greedy_sections_conform_on_a_brick_joint_and_on_a_chunk_border() {
    let mut b = world(2);
    // Make both sides of the joints busy: a block straddling y = 16 and a fill across x = 16.
    assert_eq!(b.fill_box([2, 14, 2], [6, 18, 5], 4), 5 * 5 * 4);
    assert!(b.fill_box([13, 8, 3], [19, 12, 9], 2) > 100); // some voxels are already dirt
    let m = |s: Sec| b.section_mesh_greedy(s[0], s[1], s[2]).unwrap();
    // brick joint inside chunk (0,0)
    assert_conforming(&[m([0, 0, 0]), m([0, 1, 0])], "brick joint");
    // chunk border x = 16 between chunks (0,0) and (1,0), same brick
    assert_conforming(&[m([0, 0, 0]), m([1, 0, 0])], "chunk border");
    // z border and the diagonal pair
    assert_conforming(
        &[m([0, 0, 0]), m([0, 0, 1]), m([1, 0, 0]), m([1, 0, 1])],
        "four chunks",
    );
    // each section alone is also fine, and the joined set
    assert_conforming(&greedy_meshes(&b), "world after fills");
    // The shared plane: every lattice point on the shared edges is a vertex on both sides.
    let (a, c) = (m([0, 0, 0]), m([1, 0, 0]));
    let on_plane = |mesh: &ChunkMesh| -> HashSet<P> {
        verts(mesh).into_iter().filter(|p| p[0] == 16).collect()
    };
    let (pa, pc) = (on_plane(&a), on_plane(&c));
    // Vertices on x = 16 that belong to quad edges lying in the plane exist on both sides at the
    // same lattice points wherever both surfaces reach the plane at that point.
    let common: Vec<_> = pa.intersection(&pc).collect();
    assert!(!common.is_empty());
}

#[test]
fn different_materials_keep_conforming_seams() {
    let mut b = world(2);
    b.fill_box([6, 25, 6], [8, 25, 8], 4);
    b.edit_voxel(7, 25, 7, 2);
    b.edit_voxel(15, 10, 8, 4); // material change on a chunk border
    b.edit_voxel(16, 10, 8, 2);
    assert_conforming(&greedy_meshes(&b), "materials");
}

#[test]
fn edit_undo_redo_keep_the_mesh_conforming_and_restore_it() {
    let mut b = world(2);
    let original: BTreeMap<Sec, ChunkMesh> = sections(&b)
        .into_iter()
        .map(|s| (s, b.section_mesh_greedy(s[0], s[1], s[2]).unwrap()))
        .collect();
    let y = b.ground_height(15, 8);
    assert!(b.edit_voxel(15, y, 8, 0));
    assert_eq!(b.edit_around(8, 20, 8), 27);
    assert_eq!(b.fill_box([-3, 22, -3], [3, 24, 3], 4), 147);
    b.take_dirty_sections();
    let edited: BTreeMap<Sec, ChunkMesh> = sections(&b)
        .into_iter()
        .map(|s| (s, b.section_mesh_greedy(s[0], s[1], s[2]).unwrap()))
        .collect();
    assert_ne!(original, edited);
    assert_conforming(&edited.values().cloned().collect::<Vec<_>>(), "after edits");
    for _ in 0..3 {
        assert!(b.undo() > 0);
    }
    b.take_dirty_sections();
    let undone: BTreeMap<Sec, ChunkMesh> = sections(&b)
        .into_iter()
        .map(|s| (s, b.section_mesh_greedy(s[0], s[1], s[2]).unwrap()))
        .collect();
    assert_eq!(undone, original, "undo restores every greedy mesh");
    assert_conforming(&undone.values().cloned().collect::<Vec<_>>(), "after undo");
    for _ in 0..3 {
        assert!(b.redo() > 0);
    }
    b.take_dirty_sections();
    let redone: BTreeMap<Sec, ChunkMesh> = sections(&b)
        .into_iter()
        .map(|s| (s, b.section_mesh_greedy(s[0], s[1], s[2]).unwrap()))
        .collect();
    assert_eq!(redone, edited, "redo restores the edited meshes");
    assert_conforming(&redone.values().cloned().collect::<Vec<_>>(), "after redo");
}

/// After every random edit the cache refreshed only from dirty sections equals a full rebuild (the
/// conforming rule needs no neighbour data), and the whole surface stays conforming.
#[test]
fn partial_rebuild_equals_full_rebuild_and_stays_conforming() {
    let mut b = world(2);
    let mut cache: BTreeMap<Sec, ChunkMesh> = sections(&b)
        .into_iter()
        .map(|s| (s, b.section_mesh_greedy(s[0], s[1], s[2]).unwrap()))
        .collect();
    let mut state = 0x7a11_ba5e_0d15_ea5eu64;
    let mut next = |n: i32| {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((state >> 33) as i32).rem_euclid(n)
    };
    for i in 0..40 {
        let (cx, cz) = (next(5) - 2, next(5) - 2);
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
        let full: BTreeMap<Sec, ChunkMesh> = sections(&b)
            .into_iter()
            .map(|s| (s, b.section_mesh_greedy(s[0], s[1], s[2]).unwrap()))
            .collect();
        assert_eq!(cache, full, "edit {i}");
        if i % 8 == 0 {
            assert_conforming(
                &full.values().cloned().collect::<Vec<_>>(),
                &format!("edit {i}"),
            );
        }
    }
}

#[test]
fn conforming_greedy_keeps_most_of_the_triangle_saving() {
    let b = world(2);
    let (mut faces_tris, mut greedy_tris, mut quads) = (0, 0, 0);
    for s in sections(&b) {
        faces_tris += 2 * b.section_mesh(s[0], s[1], s[2]).unwrap().faces;
        let g = b.section_mesh_greedy(s[0], s[1], s[2]).unwrap();
        greedy_tris += g.triangle_count();
        quads += g.faces;
        assert_eq!(g.triangle_count(), g.vertex_count() - 2 * g.faces);
    }
    println!("faces {faces_tris} triangles, greedy {greedy_tris} triangles in {quads} quads");
    assert!(
        greedy_tris * 2 < faces_tris,
        "{greedy_tris} vs {faces_tris}"
    );
}
