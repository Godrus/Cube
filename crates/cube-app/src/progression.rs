//! Experience and levels (M4 slice 1). **Prototype values (PROPOSED), not a balance.**
//!
//! Pure rules, no engine types and no I/O: experience is a total `u32`, the level is a function
//! of it (no stored level that could disagree), and the maximum health follows the level. A level
//! up refills health. Classes, stats and abilities are owner decisions (ODM-05) and are not here.

use crate::combat::PLAYER_MAX_HEALTH;

/// Highest level; experience above its threshold is not counted.
pub const MAX_LEVEL: u32 = 20;

/// Experience for taking a ruin's altar cache.
pub const RUIN_XP: u32 = 10;

/// Experience for taking the deep cache under a ruin.
pub const DEEP_XP: u32 = 25;

/// Extra maximum health per level above the first.
pub const HEALTH_PER_LEVEL: u32 = 4;

/// Total experience needed to reach `level` (1 needs none): `5 * level * (level - 1)`.
pub fn xp_for_level(level: u32) -> u32 {
    let l = level.clamp(1, MAX_LEVEL);
    5 * l * (l - 1)
}

/// The level of a total experience, 1..=[`MAX_LEVEL`].
pub fn level_of(xp: u32) -> u32 {
    let mut level = 1;
    while level < MAX_LEVEL && xp >= xp_for_level(level + 1) {
        level += 1;
    }
    level
}

/// The player's maximum health at `level`.
pub fn max_health_for(level: u32) -> u32 {
    PLAYER_MAX_HEALTH + HEALTH_PER_LEVEL * (level.clamp(1, MAX_LEVEL) - 1)
}

/// A character's total experience.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Progress {
    xp: u32,
}

/// What an experience gain did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gain {
    /// Experience actually added (0 at the cap).
    pub added: u32,
    /// The level before.
    pub from: u32,
    /// The level after.
    pub to: u32,
}

impl Progress {
    /// A new character: no experience, level 1.
    pub fn new() -> Self {
        Self { xp: 0 }
    }

    /// From a saved total; values above the cap are refused.
    pub fn from_xp(xp: u32) -> Option<Self> {
        (xp <= xp_for_level(MAX_LEVEL)).then_some(Self { xp })
    }

    /// Total experience.
    pub fn xp(&self) -> u32 {
        self.xp
    }

    /// Current level.
    pub fn level(&self) -> u32 {
        level_of(self.xp)
    }

    /// Total experience of the next level, `None` at the cap.
    pub fn next_level_xp(&self) -> Option<u32> {
        let l = self.level();
        (l < MAX_LEVEL).then(|| xp_for_level(l + 1))
    }

    /// Adds experience (saturating at the cap, never overflows).
    pub fn gain(&mut self, amount: u32) -> Gain {
        let from = self.level();
        let cap = xp_for_level(MAX_LEVEL);
        let new = self.xp.saturating_add(amount).min(cap);
        let added = new - self.xp;
        self.xp = new;
        Gain {
            added,
            from,
            to: self.level(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thresholds_and_levels_agree() {
        assert_eq!(level_of(0), 1);
        assert_eq!(xp_for_level(2), 10);
        assert_eq!(level_of(9), 1);
        assert_eq!(level_of(10), 2);
        assert_eq!(level_of(29), 2);
        assert_eq!(level_of(30), 3);
        for l in 1..=MAX_LEVEL {
            assert_eq!(level_of(xp_for_level(l)), l);
            if l > 1 {
                assert_eq!(level_of(xp_for_level(l) - 1), l - 1);
            }
        }
        assert_eq!(level_of(u32::MAX), MAX_LEVEL);
    }

    #[test]
    fn gain_reports_level_ups_and_caps_without_overflow() {
        let mut p = Progress::new();
        let g = p.gain(10);
        assert_eq!((g.added, g.from, g.to), (10, 1, 2));
        let g = p.gain(60);
        assert_eq!((g.from, g.to), (2, 4), "several levels in one gain");
        assert_eq!(p.next_level_xp(), Some(xp_for_level(5)));
        let g = p.gain(u32::MAX);
        assert_eq!(g.to, MAX_LEVEL);
        assert_eq!(p.xp(), xp_for_level(MAX_LEVEL));
        assert_eq!(p.gain(5).added, 0);
        assert_eq!(p.next_level_xp(), None);
    }

    #[test]
    fn health_grows_with_the_level_and_saved_totals_are_checked() {
        assert_eq!(max_health_for(1), PLAYER_MAX_HEALTH);
        assert_eq!(max_health_for(3), PLAYER_MAX_HEALTH + 2 * HEALTH_PER_LEVEL);
        assert_eq!(max_health_for(99), max_health_for(MAX_LEVEL));
        assert!(Progress::from_xp(xp_for_level(MAX_LEVEL)).is_some());
        assert!(Progress::from_xp(xp_for_level(MAX_LEVEL) + 1).is_none());
    }
}
