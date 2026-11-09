//! Deterministic helpers for Cube tests (TEST-01).
//!
//! No external dependencies: everything here is reproducible from a seed, on every platform.

/// Small deterministic pseudo-random generator (SplitMix64). Not cryptographic.
#[derive(Debug, Clone)]
pub struct DetRng {
    state: u64,
}

impl DetRng {
    /// Creates a generator from a seed. The same seed always yields the same sequence.
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Next 64-bit value.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Value in `0..bound`. Returns 0 when `bound` is 0.
    pub fn below(&mut self, bound: u64) -> u64 {
        if bound == 0 {
            0
        } else {
            self.next_u64() % bound
        }
    }

    /// Signed value in `lo..hi` (half-open). Returns `lo` when the range is empty.
    pub fn range_i64(&mut self, lo: i64, hi: i64) -> i64 {
        if hi <= lo {
            return lo;
        }
        let span = (hi as i128 - lo as i128) as u64;
        lo.wrapping_add(self.below(span) as i64)
    }
}

/// Runs `case` for `count` seeds derived from `base_seed`; the failing seed is in the panic message.
pub fn for_each_seed(base_seed: u64, count: usize, mut case: impl FnMut(u64)) {
    let mut seeder = DetRng::new(base_seed);
    for _ in 0..count {
        let seed = seeder.next_u64();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| case(seed)));
        if let Err(payload) = result {
            eprintln!("case failed for seed {seed:#x}");
            std::panic::resume_unwind(payload);
        }
    }
}

/// Asserts that `f` returns equal values on two runs (a cheap determinism check).
pub fn assert_deterministic<T: PartialEq + std::fmt::Debug>(mut f: impl FnMut() -> T) {
    let first = f();
    let second = f();
    assert_eq!(first, second, "function is not deterministic");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_sequence() {
        let mut a = DetRng::new(42);
        let mut b = DetRng::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn known_splitmix64_vector() {
        // Reference values of SplitMix64 for seed 0.
        let mut r = DetRng::new(0);
        assert_eq!(r.next_u64(), 0xE220_A839_7B1D_CDAF);
        assert_eq!(r.next_u64(), 0x6E78_9E6A_A1B9_65F4);
    }

    #[test]
    fn range_stays_in_bounds_including_negative() {
        let mut r = DetRng::new(7);
        for _ in 0..1000 {
            let v = r.range_i64(-5, 5);
            assert!((-5..5).contains(&v));
        }
        assert_eq!(r.range_i64(3, 3), 3);
        assert_eq!(r.below(0), 0);
    }

    #[test]
    fn for_each_seed_runs_all_cases() {
        let mut n = 0;
        for_each_seed(1, 25, |_| n += 1);
        assert_eq!(n, 25);
    }

    #[test]
    #[should_panic(expected = "not deterministic")]
    fn nondeterminism_is_detected() {
        let mut c = 0;
        assert_deterministic(|| {
            c += 1;
            c
        });
    }
}
