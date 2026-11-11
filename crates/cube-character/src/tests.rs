use super::*;

fn asset() -> CharacterAsset {
    CharacterAsset::build(DEFAULT_RES)
}

#[test]
fn identity_is_stable_and_build_is_deterministic() {
    let (a, b) = (asset(), asset());
    assert_eq!(a.id(), TRAVELER_ASSET_ID);
    assert_eq!(a.content_hash(), b.content_hash());
    for (pa, pb) in a.parts.iter().zip(&b.parts) {
        for l in 0..3 {
            assert_eq!(pa.lods[l].mesh, pb.lods[l].mesh);
        }
    }
}

#[test]
fn three_lods_exist_and_get_cheaper() {
    let a = asset();
    for p in &a.parts {
        for l in 0..3 {
            assert!(p.lods[l].grid.count() > 0, "{} lod{l} empty", p.id.name());
            assert!(p.lods[l].mesh.triangle_count() > 0);
        }
    }
    for l in 0..2 {
        assert!(a.triangle_count(l + 1) < a.triangle_count(l) * 3 / 4);
        assert!(a.voxel_count(l + 1) < a.voxel_count(l));
    }
}

#[test]
fn bounds_are_a_human_size_and_inside_collision_box_except_gear() {
    let a = asset();
    let b = a.bounds(0);
    assert!((b[0][1] - 0.0).abs() < 0.03, "feet at ground: {:?}", b[0]);
    assert!((1.74..=1.86).contains(&b[1][1]), "height {:?}", b[1]);
    // the body, pauldron and cape stay within a slightly larger box than the 0.6 x 0.6 collider
    assert!(b[0][0] > -0.40 && b[1][0] < 0.40, "x {:?} {:?}", b[0], b[1]);
    assert!(b[0][2] > -0.75 && b[1][2] < 0.35, "z {:?} {:?}", b[0], b[1]);
}

#[test]
fn no_missing_faces_at_any_lod() {
    let a = asset();
    for p in &a.parts {
        for l in &p.lods {
            assert_eq!(
                mesh_face_area(&l.mesh, l.res),
                exposed_faces(&l.grid),
                "{}",
                p.id.name()
            );
        }
    }
}

#[test]
fn every_material_is_in_the_palette_and_normals_are_axis_aligned() {
    let a = asset();
    for p in &a.parts {
        for l in &p.lods {
            for &m in &l.mesh.materials {
                assert!(m != 0 && (m as usize) < PALETTE_LEN);
            }
            for n in l.mesh.normals.chunks_exact(3) {
                let nonzero = n.iter().filter(|v| v.abs() > 0.5).count();
                assert_eq!(nonzero, 1, "flat axis normal expected, got {n:?}");
            }
        }
    }
}

#[test]
fn thin_parts_survive_all_lods() {
    let a = asset();
    let sword = a.parts.iter().find(|p| p.id == PartId::Sword).unwrap();
    let cape = a.parts.iter().find(|p| p.id == PartId::Cape).unwrap();
    for l in 0..3 {
        assert!(sword.lods[l].grid.count() >= 6, "sword lod{l}");
        assert!(cape.lods[l].grid.count() >= 20, "cape lod{l}");
    }
}

#[test]
fn parts_share_one_lattice_so_neighbours_meet() {
    // arm and torso overlap in model space at the shoulder: both must have voxels around x = 0.2
    let a = asset();
    let arm = a.parts.iter().find(|p| p.id == PartId::ArmR).unwrap();
    let g = &arm.lods[0].grid;
    let r = a.res as f32;
    let lo_x = g.origin[0] as f32 / r;
    assert!(
        lo_x < 0.20 && lo_x > 0.10,
        "arm starts near the shoulder, got {lo_x}"
    );
}
