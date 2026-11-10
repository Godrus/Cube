//! Loot tables and the seeded roll (CMB-06). **Prototype values (PROPOSED), not a balance.**
//!
//! A table is data: weighted entries, each an item with a count range, or nothing. The roll is a
//! pure function of the table and a key; the key comes from the world seed and the creature's
//! column ([`loot_key`]), so the same kill in the same world always drops the same thing, with
//! no hidden state and no wall clock.

use crate::game::ItemId;

/// One possible result of a roll.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LootEntry {
    /// The item, or `None` for "nothing".
    pub item: Option<ItemId>,
    /// Fewest items.
    pub min: u32,
    /// Most items (at least `min`).
    pub max: u32,
    /// Relative weight (0 never drops).
    pub weight: u32,
}

/// A weighted drop table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LootTable {
    /// The entries.
    pub entries: &'static [LootEntry],
}

/// What a roll produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Drop {
    /// The item.
    pub item: ItemId,
    /// How many.
    pub count: u32,
}

fn mix(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// The roll key of the creature that lived in column `(cx, cz)` of the world with `seed`.
pub fn loot_key(seed: u64, cx: i64, cz: i64) -> u64 {
    mix(mix(seed ^ 0x100F) ^ mix(cx as u64) ^ mix((cz as u64).rotate_left(17)))
}

impl LootTable {
    /// Rolls the table with `key`. `None` for "nothing" (also for an empty or all-zero table).
    pub fn roll(&self, key: u64) -> Option<Drop> {
        let total: u64 = self.entries.iter().map(|e| u64::from(e.weight)).sum();
        if total == 0 {
            return None;
        }
        let mut r = mix(key) % total;
        for e in self.entries {
            let w = u64::from(e.weight);
            if r < w {
                let item = e.item?;
                let span = u64::from(e.max.max(e.min) - e.min) + 1;
                let count = e.min + (mix(key ^ 0xC0DE) % span) as u32;
                return (count > 0).then_some(Drop { item, count });
            }
            r -= w;
        }
        None
    }
}

/// Slime gel.
pub const SLIME_GEL: ItemId = ItemId(6);

/// What a slime drops (prototype).
pub const SLIME_LOOT: LootTable = LootTable {
    entries: &[
        LootEntry {
            item: Some(SLIME_GEL),
            min: 1,
            max: 2,
            weight: 6,
        },
        LootEntry {
            item: Some(ItemId(5)),
            min: 1,
            max: 1,
            weight: 1,
        },
        LootEntry {
            item: None,
            min: 0,
            max: 0,
            weight: 3,
        },
    ],
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_roll_is_a_pure_function_of_table_and_key() {
        for k in 0..200u64 {
            assert_eq!(SLIME_LOOT.roll(k), SLIME_LOOT.roll(k));
        }
        assert_eq!(loot_key(7, 3, -2), loot_key(7, 3, -2));
        assert_ne!(loot_key(7, 3, -2), loot_key(8, 3, -2));
        assert_ne!(loot_key(7, 3, -2), loot_key(7, -2, 3));
    }

    #[test]
    fn drops_follow_the_table_and_the_weights() {
        let (mut gel, mut brick, mut none) = (0u32, 0u32, 0u32);
        for k in 0..10_000u64 {
            match SLIME_LOOT.roll(loot_key(1, k as i64, 0)) {
                Some(d) if d.item == SLIME_GEL => {
                    assert!((1..=2).contains(&d.count));
                    gel += 1;
                }
                Some(d) => {
                    assert_eq!((d.item, d.count), (ItemId(5), 1));
                    brick += 1;
                }
                None => none += 1,
            }
        }
        // Weights 6 : 1 : 3 of 10, with room for chance.
        assert!((5_500..=6_500).contains(&gel), "gel {gel}");
        assert!((600..=1_400).contains(&brick), "brick {brick}");
        assert!((2_500..=3_500).contains(&none), "nothing {none}");
    }

    #[test]
    fn empty_and_zero_weight_tables_drop_nothing() {
        let empty = LootTable { entries: &[] };
        assert_eq!(empty.roll(5), None);
        let zero = LootTable {
            entries: &[LootEntry {
                item: Some(ItemId(1)),
                min: 1,
                max: 1,
                weight: 0,
            }],
        };
        assert_eq!(zero.roll(5), None);
    }
}
