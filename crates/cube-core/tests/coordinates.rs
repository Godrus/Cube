//! Property-style tests for coordinates, keys and regions. Seeds are deterministic (cube-testkit);
//! a failing seed is printed by `for_each_seed`.

use cube_core::{BrickCoord, CellKey, CoreError, LayerId, Region, VoxelPos};
use cube_testkit::{DetRng, assert_deterministic, for_each_seed};

/// A coordinate that is often near the extremes and often small.
fn coord(r: &mut DetRng) -> i64 {
    match r.below(4) {
        0 => r.range_i64(-40, 40),
        1 => i64::MIN + r.below(40) as i64,
        2 => i64::MAX - r.below(40) as i64,
        _ => r.next_u64() as i64,
    }
}

#[test]
fn split_matches_an_independent_i128_reference() {
    for_each_seed(1, 2000, |seed| {
        let mut r = DetRng::new(seed);
        let v = coord(&mut r);
        let (b, l) = VoxelPos::new(v, v, v).split();
        let rebuilt = i128::from(b.x) * 16 + i128::from(l.x());
        assert_eq!(rebuilt, i128::from(v), "v = {v}");
        assert!(l.x() < 16 && l.y() < 16 && l.z() < 16);
        // Floor semantics: the local part is never negative and the brick is the floor.
        assert_eq!(i128::from(b.x), i128::from(v).div_euclid(16));
    });
}

#[test]
fn join_split_round_trip_for_every_voxel() {
    for_each_seed(2, 2000, |seed| {
        let mut r = DetRng::new(seed);
        let p = VoxelPos::new(coord(&mut r), coord(&mut r), coord(&mut r));
        let (b, l) = p.split();
        assert_eq!(b.join(l), Ok(p));
        assert_eq!(p.brick(), b);
    });
}

#[test]
fn addressable_bricks_are_exactly_those_whose_voxels_fit() {
    for_each_seed(3, 1000, |seed| {
        let mut r = DetRng::new(seed);
        let b = BrickCoord::new(coord(&mut r) / 8, 0, 0);
        let lo = i128::from(b.x) * 16;
        let fits = lo >= i128::from(i64::MIN) && lo + 15 <= i128::from(i64::MAX);
        assert_eq!(b.is_addressable(), fits, "b.x = {}", b.x);
    });
}

#[test]
fn coarse_to_fine_to_coarse_is_identity_and_coarsening_floors() {
    for_each_seed(4, 2000, |seed| {
        let mut r = DetRng::new(seed);
        let p = VoxelPos::new(
            r.range_i64(-1 << 40, 1 << 40),
            r.range_i64(-1 << 40, 1 << 40),
            r.range_i64(-1 << 40, 1 << 40),
        );
        let a = r.below(8) as i8;
        let b = a + r.below(8) as i8;
        let fine = p.convert_level(a, b).unwrap();
        assert_eq!(fine.convert_level(b, a), Ok(p));
        // Coarsening an arbitrary fine position floors: c * 2^d <= p < (c + 1) * 2^d.
        let q = VoxelPos::new(coord(&mut r), coord(&mut r), coord(&mut r));
        let c = q.convert_level(b, a).unwrap();
        let d = u32::from((b - a) as u8);
        let (lo, hi) = (i128::from(c.x) << d, (i128::from(c.x) + 1) << d);
        assert!(
            lo <= i128::from(q.x) && i128::from(q.x) < hi,
            "q.x = {}",
            q.x
        );
    });
}

#[test]
fn overflowing_refinement_is_reported_never_wrapped() {
    for_each_seed(5, 500, |seed| {
        let mut r = DetRng::new(seed);
        let p = VoxelPos::new(coord(&mut r), 0, 0);
        let d = 1 + r.below(70) as i8;
        let exact = i128::from(p.x).checked_shl(u32::try_from(d).unwrap());
        match p.convert_level(0, d) {
            Ok(q) => assert_eq!(Some(i128::from(q.x)), exact),
            Err(e) => {
                assert_eq!(e, CoreError::Overflow);
                let fits = exact.is_some_and(|v| i64::try_from(v).is_ok()) && d <= 62;
                assert!(!fits, "p.x = {}, d = {d}", p.x);
            }
        }
    });
}

fn small_region(r: &mut DetRng) -> Region {
    let lo = |r: &mut DetRng| r.range_i64(-20, 20);
    let (x, y, z) = (lo(r), lo(r), lo(r));
    let len = |r: &mut DetRng| r.range_i64(0, 18);
    Region::new(
        0,
        VoxelPos::new(x, y, z),
        VoxelPos::new(x + len(r), y + len(r), z + len(r)),
    )
    .unwrap()
}

#[test]
fn intersection_agrees_with_point_membership() {
    for_each_seed(6, 300, |seed| {
        let mut r = DetRng::new(seed);
        let (a, b) = (small_region(&mut r), small_region(&mut r));
        let i = a.intersection(&b).unwrap();
        for _ in 0..200 {
            let p = VoxelPos::new(
                r.range_i64(-25, 45),
                r.range_i64(-25, 45),
                r.range_i64(-25, 45),
            );
            let both = a.contains(p) && b.contains(p);
            assert_eq!(i.is_some_and(|i| i.contains(p)), both);
        }
        assert_eq!(a.intersection(&b), b.intersection(&a));
    });
}

#[test]
fn brick_iteration_covers_the_region_exactly_and_in_order() {
    for_each_seed(7, 200, |seed| {
        let mut r = DetRng::new(seed);
        let a = small_region(&mut r);
        let bricks: Vec<BrickCoord> = a.bricks().collect();
        assert_eq!(Some(bricks.len() as u128), a.brick_count());
        assert!(
            bricks.windows(2).all(|w| w[0] < w[1]),
            "strictly increasing lexicographic order"
        );
        // Every voxel of the region lies in a yielded brick.
        for _ in 0..200 {
            let p = VoxelPos::new(
                r.range_i64(-25, 45),
                r.range_i64(-25, 45),
                r.range_i64(-25, 45),
            );
            if a.contains(p) {
                assert!(bricks.contains(&p.brick()));
            }
        }
    });
}

#[test]
fn cell_key_order_equals_tuple_order() {
    for_each_seed(8, 500, |seed| {
        let mut r = DetRng::new(seed);
        let mk = |r: &mut DetRng| {
            let t = (
                r.below(3),
                r.below(3) as i8,
                r.range_i64(-3, 3),
                r.range_i64(-3, 3),
                r.range_i64(-3, 3),
            );
            (
                t,
                CellKey::new(LayerId::new(t.0), t.1, BrickCoord::new(t.2, t.3, t.4)).unwrap(),
            )
        };
        let (ta, ka) = mk(&mut r);
        let (tb, kb) = mk(&mut r);
        assert_eq!(ta.cmp(&tb), ka.cmp(&kb));
    });
}

#[test]
fn results_are_deterministic() {
    assert_deterministic(|| {
        let mut r = DetRng::new(99);
        (0..64)
            .map(|_| VoxelPos::new(coord(&mut r), coord(&mut r), coord(&mut r)).split())
            .collect::<Vec<_>>()
    });
}
