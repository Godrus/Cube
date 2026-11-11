//! Engine-neutral combat rules (CMB-01, CMB-02). **Prototype values (PROPOSED), not a balance.**
//!
//! Pure data and integer arithmetic: no engine types, no clock, no randomness, no I/O. Time is
//! counted in fixed ticks by the caller. Every function either returns the changed value with
//! an event or refuses and changes nothing, so a server can run the same code as authority
//! (`intent in, validated result out`); a client never decides damage.

/// Health of a character or creature. `cur <= max`, `max >= 1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Health {
    cur: u32,
    max: u32,
    /// Flat damage taken off every hit (never below 1 dealt). Derived from worn items by the
    /// caller, not saved; creatures have none.
    armour: u32,
}

/// Prototype maximum health of the player.
pub const PLAYER_MAX_HEALTH: u32 = 20;

/// What one damage application did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Damaged {
    /// Health actually removed (never more than the target had).
    pub dealt: u32,
    /// Health left.
    pub remaining: u32,
    /// `true` only for the hit that took the last health (never repeated).
    pub killed: bool,
}

impl Health {
    /// Sets the maximum to `max` (at least 1) and fills health to it (a level-up reward).
    pub fn refill_to(&mut self, max: u32) {
        self.max = max.max(1);
        self.cur = self.max;
    }

    /// Full health of `max` (a `max` of 0 becomes 1: nothing lives with no health).
    pub fn full(max: u32) -> Self {
        let max = max.max(1);
        Self {
            cur: max,
            max,
            armour: 0,
        }
    }

    /// Sets the armour value (flat reduction of each hit).
    pub fn set_armour(&mut self, armour: u32) {
        self.armour = armour;
    }

    /// The armour value.
    pub fn armour(&self) -> u32 {
        self.armour
    }

    /// From saved values; `None` if they are not valid (`max` 0 or `cur` above `max`).
    pub fn from_parts(cur: u32, max: u32) -> Option<Self> {
        (max >= 1 && cur <= max).then_some(Self {
            cur,
            max,
            armour: 0,
        })
    }

    /// Current health.
    pub fn cur(&self) -> u32 {
        self.cur
    }

    /// Maximum health.
    pub fn max(&self) -> u32 {
        self.max
    }

    /// No health left.
    pub fn is_dead(&self) -> bool {
        self.cur == 0
    }

    /// Removes `amount`, clamped at zero. A dead target takes nothing and is not killed again.
    pub fn damage(&mut self, amount: u32) -> Damaged {
        let dealt = amount.min(self.cur);
        let was_alive = self.cur > 0;
        self.cur -= dealt;
        Damaged {
            dealt,
            remaining: self.cur,
            killed: was_alive && self.cur == 0,
        }
    }

    /// Adds up to `amount` (never above `max`, never raises the dead: use [`Health::revive`]).
    /// Returns how much was added.
    pub fn heal(&mut self, amount: u32) -> u32 {
        if self.is_dead() {
            return 0;
        }
        let add = amount.min(self.max - self.cur);
        self.cur += add;
        add
    }

    /// Respawn rule of the prototype: a dead character returns with full health. A living one is
    /// unchanged. Returns `true` if it revived.
    pub fn revive(&mut self) -> bool {
        if self.is_dead() {
            self.cur = self.max;
            true
        } else {
            false
        }
    }
}

/// A melee weapon (data).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeaponDef {
    /// Display name.
    pub name: &'static str,
    /// Health removed by a hit.
    pub damage: u32,
    /// Reach in thousandths of a voxel, from the attacker to the target (integer: exact and
    /// deterministic).
    pub reach_milli: u32,
    /// Ticks before the next attack.
    pub cooldown_ticks: u32,
}

/// The first weapon of the prototype.
pub const WOODEN_SWORD: WeaponDef = WeaponDef {
    name: "Wooden sword",
    damage: 4,
    reach_milli: 2500,
    cooldown_ticks: 24,
};

/// The iron sword: one hit kills a slime (8 health), a little longer reach, a quicker swing.
pub const IRON_SWORD: WeaponDef = WeaponDef {
    name: "Iron sword",
    damage: 8,
    reach_milli: 2600,
    cooldown_ticks: 20,
};

/// The attacker's attack timer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AttackTimer {
    left: u32,
}

impl AttackTimer {
    /// A timer ready to attack.
    pub fn new() -> Self {
        Self::default()
    }

    /// One tick passes.
    pub fn tick(&mut self) {
        self.left = self.left.saturating_sub(1);
    }

    /// Ticks until the next attack is possible.
    pub fn left(&self) -> u32 {
        self.left
    }
}

/// Why an attack did nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttackRefused {
    /// The weapon is still recovering.
    OnCooldown {
        /// Ticks left.
        left: u32,
    },
    /// The target is farther than the weapon reaches.
    OutOfReach,
    /// The target is already dead.
    TargetDead,
}

/// What a resolved attack did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hit {
    /// Health removed.
    pub dealt: u32,
    /// Target health left.
    pub remaining: u32,
    /// The target died from this hit.
    pub killed: bool,
}

/// Resolves one attack: checks the cooldown, the target and the reach (`distance_milli` between
/// the bodies, thousandths of a voxel), then applies the weapon's damage and starts the
/// cooldown. On refusal nothing changes (neither timer nor health).
pub fn attack(
    weapon: &WeaponDef,
    timer: &mut AttackTimer,
    target: &mut Health,
    distance_milli: u64,
) -> Result<Hit, AttackRefused> {
    if timer.left > 0 {
        return Err(AttackRefused::OnCooldown { left: timer.left });
    }
    if target.is_dead() {
        return Err(AttackRefused::TargetDead);
    }
    if distance_milli > u64::from(weapon.reach_milli) {
        return Err(AttackRefused::OutOfReach);
    }
    // Armour takes a flat amount off the hit, but a hit always does at least 1.
    let amount = if weapon.damage == 0 {
        0
    } else {
        weapon.damage.saturating_sub(target.armour).max(1)
    };
    let d = target.damage(amount);
    timer.left = weapon.cooldown_ticks;
    Ok(Hit {
        dealt: d.dealt,
        remaining: d.remaining,
        killed: d.killed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn damage_clamps_at_zero_and_death_is_reported_once() {
        let mut h = Health::full(10);
        let a = h.damage(4);
        assert_eq!((a.dealt, a.remaining, a.killed), (4, 6, false));
        let b = h.damage(u32::MAX);
        assert_eq!((b.dealt, b.remaining, b.killed), (6, 0, true));
        assert!(h.is_dead());
        let c = h.damage(5);
        assert_eq!((c.dealt, c.killed), (0, false), "no second death");
        assert!(!h.damage(0).killed);
    }

    #[test]
    fn armour_reduces_each_hit_but_never_below_one() {
        let mut h = Health::full(20);
        h.set_armour(1);
        let mut t = AttackTimer::new();
        let hit = attack(&WOODEN_SWORD, &mut t, &mut h, 0).unwrap();
        assert_eq!(hit.dealt, WOODEN_SWORD.damage - 1);
        h.set_armour(100);
        let mut t = AttackTimer::new();
        assert_eq!(attack(&WOODEN_SWORD, &mut t, &mut h, 0).unwrap().dealt, 1);
        assert_eq!(h.armour(), 100);
        assert_eq!(Health::from_parts(5, 9).unwrap().armour(), 0, "not saved");
    }

    #[test]
    fn zero_damage_to_the_living_changes_nothing() {
        let mut h = Health::full(3);
        let d = h.damage(0);
        assert_eq!((d.dealt, d.remaining, d.killed), (0, 3, false));
    }

    #[test]
    fn heal_stops_at_max_and_cannot_raise_the_dead() {
        let mut h = Health::full(10);
        h.damage(3);
        assert_eq!(h.heal(u32::MAX), 3);
        assert_eq!(h.cur(), 10);
        h.damage(10);
        assert_eq!(h.heal(5), 0, "the dead are revived only by respawn");
        assert!(h.is_dead());
    }

    #[test]
    fn respawn_restores_full_health_once() {
        let mut h = Health::full(20);
        assert!(!h.revive(), "a living character is unchanged");
        h.damage(20);
        assert!(h.revive());
        assert_eq!(h.cur(), 20);
        assert!(!h.revive());
    }

    #[test]
    fn invalid_saved_values_are_refused_and_limits_hold() {
        assert!(Health::from_parts(5, 0).is_none());
        assert!(Health::from_parts(11, 10).is_none());
        assert!(Health::from_parts(0, 10).is_some());
        let mut h = Health::full(u32::MAX);
        assert_eq!(h.damage(u32::MAX).remaining, 0);
        assert_eq!(Health::full(0).max(), 1);
    }

    #[test]
    fn an_attack_hits_in_reach_and_starts_the_cooldown() {
        let mut t = AttackTimer::new();
        let mut h = Health::full(10);
        let hit = attack(&WOODEN_SWORD, &mut t, &mut h, 2500).unwrap();
        assert_eq!((hit.dealt, hit.remaining, hit.killed), (4, 6, false));
        assert_eq!(t.left(), WOODEN_SWORD.cooldown_ticks);
    }

    #[test]
    fn refused_attacks_change_nothing() {
        let mut t = AttackTimer::new();
        let mut h = Health::full(10);
        assert_eq!(
            attack(&WOODEN_SWORD, &mut t, &mut h, 2501),
            Err(AttackRefused::OutOfReach)
        );
        assert_eq!((t.left(), h.cur()), (0, 10));
        attack(&WOODEN_SWORD, &mut t, &mut h, 0).unwrap();
        let (t0, h0) = (t, h);
        assert!(matches!(
            attack(&WOODEN_SWORD, &mut t, &mut h, 0),
            Err(AttackRefused::OnCooldown { .. })
        ));
        assert_eq!((t, h), (t0, h0));
        let mut dead = Health::full(1);
        dead.damage(1);
        let mut ready = AttackTimer::new();
        assert_eq!(
            attack(&WOODEN_SWORD, &mut ready, &mut dead, 0),
            Err(AttackRefused::TargetDead)
        );
        assert_eq!(ready.left(), 0);
    }

    #[test]
    fn the_cooldown_runs_out_after_its_ticks_and_the_fight_is_deterministic() {
        let run = || {
            let mut t = AttackTimer::new();
            let mut h = Health::full(10);
            let mut log = Vec::new();
            for tick in 0..100u32 {
                if let Ok(hit) = attack(&WOODEN_SWORD, &mut t, &mut h, 1000) {
                    log.push((tick, hit.remaining, hit.killed));
                }
                t.tick();
            }
            log
        };
        let log = run();
        assert_eq!(log, run(), "equal inputs, equal results");
        assert_eq!(log, vec![(0, 6, false), (24, 2, false), (48, 0, true)]);
    }
}
