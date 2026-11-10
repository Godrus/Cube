//! Where creatures live (CMB-05): a deterministic spawn rule and a bounded population around
//! the player. **Prototype values (PROPOSED).** Engine-neutral, no clock, no I/O.
//!
//! * **Rule:** for every 16 x 16 column of the world, a pure function of the world seed and the
//!   column decides whether a creature lives there and where (about one column in six, never in
//!   the safe columns around the origin). Nothing about untouched creatures is saved: the same
//!   seed gives the same spawns, like the terrain.
//! * **Population:** creatures exist only near the player (`spawn_radius` columns), at most
//!   `max` at a time (nearest columns first), and are removed again when the player is farther
//!   than `despawn_radius` columns (so the resident count stays bounded on a long walk). A
//!   creature that was removed by despawn comes back fresh (full health) later; one the player
//!   killed does not come back in this session. **Kills are not saved yet** (a restart
//!   brings the creature back): CMB-07 settles that together with loot.

use std::collections::{BTreeMap, BTreeSet};

use cube_client::FaceIndex;

use crate::combat::{AttackRefused, AttackTimer, Health, Hit, WeaponDef};
use crate::creature::{Creature, SLIME, distance_milli};

/// One in this many columns holds a creature.
const SPAWN_ONE_IN: u64 = 6;
/// Columns (16 voxels) around the origin where nothing spawns.
const SAFE_COLUMNS: i64 = 1;

fn mix(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Where the creature of the column `(cx, cz)` stands (feet), if the column has one.
/// `height(x, z)` is the ground height: the highest solid voxel of that voxel column.
pub fn spawn_point(
    seed: u64,
    cx: i64,
    cz: i64,
    height: impl Fn(i64, i64) -> i64,
) -> Option<[f64; 3]> {
    if cx.abs() <= SAFE_COLUMNS && cz.abs() <= SAFE_COLUMNS {
        return None;
    }
    let h =
        mix(mix(seed ^ 0xC4EA_7B01) ^ (cx as u64).wrapping_mul(0x1000_0000_01B3) ^ mix(cz as u64));
    if !h.is_multiple_of(SPAWN_ONE_IN) {
        return None;
    }
    let (ox, oz) = (((h >> 8) % 14 + 1) as i64, ((h >> 16) % 14 + 1) as i64);
    let (x, z) = (cx * 16 + ox, cz * 16 + oz);
    Some([x as f64 + 0.5, (height(x, z) + 1) as f64, z as f64 + 0.5])
}

/// The creatures near the player.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Population {
    alive: BTreeMap<(i64, i64), Creature>,
    killed: BTreeSet<(i64, i64)>,
}

/// Where and how many creatures to keep (in columns of 16 voxels).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnArea {
    /// Column of the player.
    pub centre: (i64, i64),
    /// Columns around the player that get creatures.
    pub spawn_radius: i64,
    /// Creatures farther than this many columns are removed.
    pub despawn_radius: i64,
    /// Most creatures at a time.
    pub max: usize,
}

/// What one tick of the population did.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TickReport {
    /// Hits the creatures landed on the player this tick.
    pub hits_on_player: Vec<Hit>,
}

impl Population {
    /// No creatures.
    pub fn new() -> Self {
        Self::default()
    }

    /// The living creatures, nearest the origin of the column order first.
    pub fn creatures(&self) -> impl Iterator<Item = &Creature> {
        self.alive.values()
    }

    /// How many creatures exist now.
    pub fn len(&self) -> usize {
        self.alive.len()
    }

    /// `true` if there are none.
    pub fn is_empty(&self) -> bool {
        self.alive.is_empty()
    }

    /// Removes creatures farther than `despawn_radius` columns from `centre`, then spawns the
    /// creatures of the columns within `spawn_radius` (nearest first) that have none, are not
    /// killed and fit under `max`. `in_world(cx, cz)` says whether the column exists (the world
    /// is finite). Returns how many were spawned.
    pub fn update(
        &mut self,
        seed: u64,
        area: SpawnArea,
        in_world: impl Fn(i64, i64) -> bool,
        height: impl Fn(i64, i64) -> i64,
    ) -> usize {
        let SpawnArea {
            centre,
            spawn_radius,
            despawn_radius,
            max,
        } = area;
        let cheb = |c: &(i64, i64)| (c.0 - centre.0).abs().max((c.1 - centre.1).abs());
        self.alive.retain(|c, _| cheb(c) <= despawn_radius);
        let mut cols: Vec<(i64, i64)> = Vec::new();
        for dz in -spawn_radius..=spawn_radius {
            for dx in -spawn_radius..=spawn_radius {
                cols.push((centre.0 + dx, centre.1 + dz));
            }
        }
        cols.sort_by_key(|c| ((c.0 - centre.0).pow(2) + (c.1 - centre.1).pow(2), c.0, c.1));
        let mut spawned = 0;
        for c in cols {
            if self.alive.len() >= max {
                break;
            }
            if self.alive.contains_key(&c) || self.killed.contains(&c) || !in_world(c.0, c.1) {
                continue;
            }
            if let Some(p) = spawn_point(seed, c.0, c.1, &height) {
                self.alive.insert(c, Creature::new(&SLIME, p));
                spawned += 1;
            }
        }
        spawned
    }

    /// One tick. Only creatures for which `covers(feet)` holds are advanced (the derived faces
    /// under the others are not at hand, and a creature must not fall through a world that is
    /// not there); `world` is the collision data. Creature hits reduce `player`.
    pub fn tick(
        &mut self,
        world: &FaceIndex,
        covers: impl Fn([f64; 3]) -> bool,
        player_feet: [f64; 3],
        player: &mut Health,
    ) -> TickReport {
        let mut report = TickReport::default();
        for c in self.alive.values_mut() {
            if covers(c.pos()) {
                report
                    .hits_on_player
                    .extend(c.update(world, player_feet, player));
            }
        }
        report
    }

    /// The player attacks the nearest creature (resolved by the combat rules). A kill removes
    /// the creature for this session and is returned with its position (loot is CMB-06).
    /// `Err(OutOfReach)` if none is in reach or there are no creatures.
    pub fn player_attack(
        &mut self,
        weapon: &WeaponDef,
        timer: &mut AttackTimer,
        player_feet: [f64; 3],
    ) -> Result<(Hit, [f64; 3]), AttackRefused> {
        let nearest = self
            .alive
            .iter()
            .map(|(k, c)| (distance_milli(c.pos(), player_feet), *k))
            .min();
        let Some((dist, key)) = nearest else {
            return Err(AttackRefused::OutOfReach);
        };
        let Some(c) = self.alive.get_mut(&key) else {
            return Err(AttackRefused::OutOfReach);
        };
        let hit = c.receive_attack(weapon, timer, dist)?;
        let at = c.pos();
        if hit.killed {
            self.alive.remove(&key);
            self.killed.insert(key);
        }
        Ok((hit, at))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::{PLAYER_MAX_HEALTH, WOODEN_SWORD};

    fn flat(_x: i64, _z: i64) -> i64 {
        9
    }

    fn everywhere(_x: i64, _z: i64) -> bool {
        true
    }

    #[test]
    fn the_spawn_rule_is_a_pure_function_of_seed_and_column() {
        let a: Vec<_> = (-20..20)
            .flat_map(|x| (-20..20).map(move |z| (x, z)))
            .map(|(x, z)| spawn_point(7, x, z, flat))
            .collect();
        let b: Vec<_> = (-20..20)
            .flat_map(|x| (-20..20).map(move |z| (x, z)))
            .map(|(x, z)| spawn_point(7, x, z, flat))
            .collect();
        assert_eq!(a, b);
        let n = a.iter().flatten().count();
        assert!(
            (100..=400).contains(&n),
            "about one column in six: {n} of 1600"
        );
        let other: Vec<_> = (-20..20)
            .flat_map(|x| (-20..20).map(move |z| (x, z)))
            .map(|(x, z)| spawn_point(8, x, z, flat))
            .collect();
        assert_ne!(a, other, "another seed, another spawns");
        for p in a.iter().flatten() {
            assert_eq!(p[1], 10.0, "feet stand on the highest solid voxel");
        }
    }

    #[test]
    fn spawns_stay_inside_their_column_and_away_from_the_origin() {
        for x in -30..30 {
            for z in -30..30 {
                if let Some(p) = spawn_point(3, x, z, flat) {
                    {
                        assert!(p[0] > (x * 16) as f64 && p[0] < (x * 16 + 16) as f64);
                        assert!(p[2] > (z * 16) as f64 && p[2] < (z * 16 + 16) as f64);
                        assert!(x.abs() > 1 || z.abs() > 1, "safe columns are empty");
                    }
                }
            }
        }
    }

    #[test]
    fn the_population_is_bounded_deterministic_and_follows_the_player() {
        let mut p = Population::new();
        let n = p.update(
            7,
            SpawnArea {
                centre: (0, 0),
                spawn_radius: 6,
                despawn_radius: 8,
                max: 5,
            },
            everywhere,
            flat,
        );
        assert_eq!((n, p.len()), (5, 5), "bounded by max");
        let snapshot = p.clone();
        let mut q = Population::new();
        q.update(
            7,
            SpawnArea {
                centre: (0, 0),
                spawn_radius: 6,
                despawn_radius: 8,
                max: 5,
            },
            everywhere,
            flat,
        );
        assert_eq!(snapshot, q, "same seed and place, same creatures");
        // Walk far away: the old ones are gone, new ones are near the new centre, count bounded.
        for step in 1..=40 {
            p.update(
                7,
                SpawnArea {
                    centre: (step * 2, 0),
                    spawn_radius: 6,
                    despawn_radius: 8,
                    max: 5,
                },
                everywhere,
                flat,
            );
            assert!(p.len() <= 5);
        }
        assert!(
            p.creatures()
                .all(|c| (c.pos()[0] / 16.0 - 80.0).abs() <= 9.0)
        );
        // Coming back recreates the very same creatures for the same columns.
        let mut back = Population::new();
        back.update(
            7,
            SpawnArea {
                centre: (80, 0),
                spawn_radius: 6,
                despawn_radius: 8,
                max: 5,
            },
            everywhere,
            flat,
        );
        assert_eq!(p.len(), back.len());
    }

    #[test]
    fn columns_outside_the_world_never_spawn() {
        let mut p = Population::new();
        p.update(
            7,
            SpawnArea {
                centre: (0, 0),
                spawn_radius: 6,
                despawn_radius: 8,
                max: 100,
            },
            |x, _| x < 0,
            flat,
        );
        assert!(p.creatures().all(|c| c.pos()[0] < 0.0));
        assert!(!p.is_empty());
    }

    #[test]
    fn a_killed_creature_is_gone_and_does_not_come_back_this_session() {
        let mut p = Population::new();
        p.update(
            7,
            SpawnArea {
                centre: (0, 0),
                spawn_radius: 6,
                despawn_radius: 8,
                max: 100,
            },
            everywhere,
            flat,
        );
        let before = p.len();
        let target = p.creatures().next().unwrap().pos();
        let mut timer = AttackTimer::new();
        let mut kills = 0;
        for _ in 0..40 {
            if let Ok((h, _)) = p.player_attack(&WOODEN_SWORD, &mut timer, target) {
                kills += u32::from(h.killed);
            }
            for _ in 0..WOODEN_SWORD.cooldown_ticks {
                timer.tick();
            }
        }
        assert_eq!(kills, 1, "it needed two hits and died once");
        assert_eq!(p.len(), before - 1);
        p.update(
            7,
            SpawnArea {
                centre: (0, 0),
                spawn_radius: 6,
                despawn_radius: 8,
                max: 100,
            },
            everywhere,
            flat,
        );
        assert_eq!(p.len(), before - 1, "not respawned");
        let _ = PLAYER_MAX_HEALTH;
    }

    #[test]
    fn attacks_with_nothing_in_reach_are_refused() {
        let mut p = Population::new();
        let mut t = AttackTimer::new();
        assert!(p.player_attack(&WOODEN_SWORD, &mut t, [0.0; 3]).is_err());
        p.update(
            7,
            SpawnArea {
                centre: (0, 0),
                spawn_radius: 6,
                despawn_radius: 8,
                max: 100,
            },
            everywhere,
            flat,
        );
        let far = [10_000.0, 10.0, 10_000.0];
        assert_eq!(
            p.player_attack(&WOODEN_SWORD, &mut t, far),
            Err(AttackRefused::OutOfReach)
        );
        assert_eq!(t.left(), 0, "a refused attack costs no cooldown");
    }
}
