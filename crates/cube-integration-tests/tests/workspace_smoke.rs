//! First integration test: the shared testkit is usable from a second crate and is reproducible.

use cube_testkit::{DetRng, assert_deterministic, for_each_seed};

#[test]
fn testkit_sequences_are_reproducible_across_crates() {
    assert_deterministic(|| {
        let mut r = DetRng::new(2026);
        (0..16).map(|_| r.next_u64()).collect::<Vec<_>>()
    });
}

#[test]
fn seeded_cases_cover_wide_negative_ranges() {
    for_each_seed(9, 50, |seed| {
        let mut r = DetRng::new(seed);
        let v = r.range_i64(i64::MIN / 2, i64::MAX / 2);
        assert!((i64::MIN / 2..i64::MAX / 2).contains(&v));
    });
}
