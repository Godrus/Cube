//! Property-style tests of channels and brick storage against simple reference models.
//! Storage mutators are crate-private, so storage is exercised through `LayerData::empty` only for
//! state queries here; populated-store behaviour is covered by the unit tests inside the crate.

use std::collections::HashMap;

use cube_core::{BrickBounds, BrickCoord, CellState, CellStore, Channel, CoreError, LocalPos};
use cube_testkit::{DetRng, for_each_seed};

fn lp(r: &mut DetRng) -> LocalPos {
    LocalPos::new(r.below(16) as u8, r.below(16) as u8, r.below(16) as u8).unwrap()
}

#[test]
fn channel_behaves_like_a_hash_map_with_a_default_across_all_widths() {
    for_each_seed(21, 300, |seed| {
        let mut r = DetRng::new(seed);
        let default = r.below(70_000) as u32;
        let mut ch = Channel::constant(default);
        let mut model: HashMap<(u8, u8, u8), u32> = HashMap::new();
        for _ in 0..r.below(200) {
            let p = lp(&mut r);
            // Often write the default again: such writes must keep the channel observably equal.
            let v = if r.below(3) == 0 {
                default
            } else {
                r.below(70_000) as u32
            };
            ch.set(p, v);
            model.insert((p.x(), p.y(), p.z()), v);
        }
        for p in LocalPos::all() {
            let expected = model
                .get(&(p.x(), p.y(), p.z()))
                .copied()
                .unwrap_or(default);
            assert_eq!(ch.get(p), expected);
        }
        // compact() never changes the observable values, and uniform() agrees with the values.
        let before: Vec<u32> = LocalPos::all().map(|p| ch.get(p)).collect();
        let mut compacted = ch.clone();
        compacted.compact();
        let after: Vec<u32> = LocalPos::all().map(|p| compacted.get(p)).collect();
        assert_eq!(before, after);
        assert_eq!(compacted, ch);
        let all_same = before.iter().all(|v| *v == before[0]);
        assert_eq!(ch.uniform(), all_same.then_some(before[0]));
        assert_eq!(compacted.is_dense(), !all_same);
    });
}

#[test]
fn unbounded_empty_store_reports_not_loaded_everywhere() {
    for_each_seed(22, 500, |seed| {
        let mut r = DetRng::new(seed);
        let s = CellStore::new();
        let b = BrickCoord::new(
            r.next_u64() as i64,
            r.next_u64() as i64,
            r.next_u64() as i64,
        );
        assert_eq!(s.get(r.below(5) as i8, b), Ok(CellState::NotLoaded));
        assert!(s.is_empty());
    });
}

#[test]
fn bounded_store_classifies_every_brick_as_inside_or_outside() {
    for_each_seed(23, 500, |seed| {
        let mut r = DetRng::new(seed);
        let lo = BrickCoord::new(r.range_i64(-5, 0), r.range_i64(-5, 0), r.range_i64(-5, 0));
        let hi = BrickCoord::new(
            lo.x + r.range_i64(0, 6),
            lo.y + r.range_i64(0, 6),
            lo.z + r.range_i64(0, 6),
        );
        let s = CellStore::with_bounds(BrickBounds::new(lo, hi).unwrap());
        let level = r.below(4) as i8;
        let b = BrickCoord::new(
            r.range_i64(-40, 40),
            r.range_i64(-40, 40),
            r.range_i64(-40, 40),
        );
        let base = |v: i64| v >> level;
        let inside = (lo.x..hi.x).contains(&base(b.x))
            && (lo.y..hi.y).contains(&base(b.y))
            && (lo.z..hi.z).contains(&base(b.z));
        let expected = if inside {
            CellState::NotLoaded
        } else {
            CellState::OutsideBounds
        };
        assert_eq!(s.get(level, b), Ok(expected));
    });
}

#[test]
fn reserved_levels_are_always_errors() {
    for_each_seed(24, 200, |seed| {
        let mut r = DetRng::new(seed);
        let lvl = -(1 + r.below(128) as i16) as i8;
        assert_eq!(
            CellStore::new().get(lvl, BrickCoord::new(0, 0, 0)),
            Err(CoreError::InvalidLevel(lvl))
        );
    });
}
