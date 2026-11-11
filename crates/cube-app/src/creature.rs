//! A hostile creature as engine-neutral data and a state machine (CMB-04). **Prototype values
//! (PROPOSED), not a design.**
//!
//! The creature moves with the same body and collision as the player ([`cube_client::Walker`] on
//! the derived faces), so it cannot walk through solid voxels. States: `Idle` (nothing in
//! sight), `Chase` (target in sight, out of reach), `Attack` (in reach), `Dead`. One call of
//! [`Creature::update`] is one fixed tick of 1/60 s; no randomness, no wall clock, so equal
//! inputs give equal results. Damage is resolved by [`crate::combat::attack`]; this module
//! never decides anything the combat rules do not. Known prototype limits: the creature walks
//! at the player's walking speed (the body has one speed) and does not path-find around walls.

use cube_client::{FaceIndex, WalkInput, Walker};

use crate::combat::{AttackRefused, AttackTimer, Health, Hit, WeaponDef, attack};
use crate::loot::{LootTable, SLIME_LOOT};

/// Length of one tick in seconds.
pub const TICK: f32 = 1.0 / 60.0;

/// What a kind of creature is (data).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreatureDef {
    /// Display name.
    pub name: &'static str,
    /// Health.
    pub max_health: u32,
    /// How far it notices a target, thousandths of a voxel.
    pub sight_milli: u32,
    /// How it hits (damage, reach, cooldown).
    pub weapon: WeaponDef,
    /// What it drops when killed.
    pub loot: LootTable,
    /// Experience the player gets for the kill.
    pub xp: u32,
}

/// The first hostile creature of the prototype.
pub const SLIME: CreatureDef = CreatureDef {
    name: "Slime",
    max_health: 8,
    sight_milli: 12_000,
    weapon: WeaponDef {
        name: "Slam",
        damage: 2,
        reach_milli: 1_600,
        cooldown_ticks: 60,
    },
    loot: SLIME_LOOT,
    xp: 4,
};

/// What the creature is doing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreatureState {
    /// Nothing in sight.
    Idle,
    /// Target in sight, walking towards it.
    Chase,
    /// Target in reach.
    Attack,
    /// No health left; does nothing any more.
    Dead,
}

/// One creature in the world.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Creature {
    def: &'static CreatureDef,
    body: Walker,
    health: Health,
    timer: AttackTimer,
    state: CreatureState,
}

/// Distance between two feet positions in thousandths of a voxel (rounded).
pub fn distance_milli(a: [f64; 3], b: [f64; 3]) -> u64 {
    let d = ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
    (d * 1000.0).round() as u64
}

impl Creature {
    /// A creature of `def` standing (or about to fall) at `feet`.
    pub fn new(def: &'static CreatureDef, feet: [f64; 3]) -> Self {
        Self {
            def,
            body: Walker::new(feet),
            health: Health::full(def.max_health),
            timer: AttackTimer::new(),
            state: CreatureState::Idle,
        }
    }

    /// Its kind.
    pub fn def(&self) -> &'static CreatureDef {
        self.def
    }

    /// Feet position.
    pub fn pos(&self) -> [f64; 3] {
        self.body.pos
    }

    /// Facing (radians around the vertical axis, as [`Walker::yaw`]).
    pub fn yaw(&self) -> f32 {
        self.body.yaw
    }

    /// Health.
    pub fn health(&self) -> Health {
        self.health
    }

    /// Current state.
    pub fn state(&self) -> CreatureState {
        self.state
    }

    /// A weapon hit on this creature, resolved by the combat rules (cooldown and reach belong
    /// to the attacker's `timer`). A kill turns the creature `Dead`.
    pub fn receive_attack(
        &mut self,
        weapon: &WeaponDef,
        timer: &mut AttackTimer,
        distance_milli: u64,
    ) -> Result<Hit, AttackRefused> {
        let hit = attack(weapon, timer, &mut self.health, distance_milli)?;
        if hit.killed {
            self.state = CreatureState::Dead;
        }
        Ok(hit)
    }

    /// One tick. `target_feet` and `target` are the player's position and health. Returns the
    /// hit the creature dealt this tick, if any.
    pub fn update(
        &mut self,
        world: &FaceIndex,
        target_feet: [f64; 3],
        target: &mut Health,
    ) -> Option<Hit> {
        if self.state == CreatureState::Dead || self.health.is_dead() {
            self.state = CreatureState::Dead;
            return None;
        }
        self.timer.tick();
        let dist = distance_milli(self.body.pos, target_feet);
        let sees = !target.is_dead() && dist <= u64::from(self.def.sight_milli);
        let in_reach = sees && dist <= u64::from(self.def.weapon.reach_milli);
        let mut walk = WalkInput::default();
        let mut hit = None;
        if in_reach {
            self.state = CreatureState::Attack;
            self.face(target_feet);
            hit = attack(&self.def.weapon, &mut self.timer, target, dist).ok();
        } else if sees {
            self.state = CreatureState::Chase;
            self.face(target_feet);
            walk.forward = 1;
        } else {
            self.state = CreatureState::Idle;
        }
        // Gravity applies in every state.
        self.body.step(world, &walk, TICK);
        hit
    }

    fn face(&mut self, to: [f64; 3]) {
        let dx = to[0] - self.body.pos[0];
        let dz = to[2] - self.body.pos[2];
        if dx != 0.0 || dz != 0.0 {
            self.body.yaw = (-dx).atan2(-dz) as f32;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::{PLAYER_MAX_HEALTH, WOODEN_SWORD};
    use cube_backend_voxel::{Dir, Face};
    use cube_core::VoxelPos;

    const DIRS: [Dir; 6] = [
        Dir::PosX,
        Dir::NegX,
        Dir::PosY,
        Dir::NegY,
        Dir::PosZ,
        Dir::NegZ,
    ];

    /// A slab `x, z in -30..30` with solid voxels `y < 10`, plus extra solid voxels.
    fn world(extra: &[[i64; 3]]) -> FaceIndex {
        let mut faces = Vec::new();
        let mut add = |x: i64, y: i64, z: i64| {
            for dir in DIRS {
                faces.push(Face {
                    voxel: VoxelPos::new(x, y, z),
                    dir,
                    material: 1,
                });
            }
        };
        for x in -30..30 {
            for z in -30..30 {
                add(x, 8, z);
                add(x, 9, z);
            }
        }
        for e in extra {
            add(e[0], e[1], e[2]);
        }
        FaceIndex::new(faces.iter())
    }

    fn settle(c: &mut Creature, w: &FaceIndex) {
        let mut nobody = Health::full(1);
        nobody.damage(1);
        for _ in 0..120 {
            c.update(w, [0.5, 10.0, 0.5], &mut nobody);
        }
    }

    #[test]
    fn it_stays_idle_when_nothing_is_in_sight_and_lands_on_the_ground() {
        let w = world(&[]);
        let mut c = Creature::new(&SLIME, [20.5, 12.0, 0.5]);
        let mut p = Health::full(PLAYER_MAX_HEALTH);
        for _ in 0..120 {
            assert!(c.update(&w, [0.5, 10.0, 0.5], &mut p).is_none());
        }
        assert_eq!(c.state(), CreatureState::Idle);
        assert!((c.pos()[1] - 10.0).abs() < 0.01, "stands on the ground");
        assert_eq!(p.cur(), PLAYER_MAX_HEALTH);
    }

    #[test]
    fn it_chases_what_it_sees_and_gets_closer_every_second() {
        let w = world(&[]);
        let mut c = Creature::new(&SLIME, [8.5, 10.0, 0.5]);
        settle(&mut c, &w);
        let target = [0.5, 10.0, 0.5];
        let mut p = Health::full(PLAYER_MAX_HEALTH);
        c.update(&w, target, &mut p);
        assert_eq!(c.state(), CreatureState::Chase);
        let mut last = distance_milli(c.pos(), target);
        for _ in 0..3 {
            for _ in 0..20 {
                c.update(&w, target, &mut p);
            }
            let now = distance_milli(c.pos(), target);
            assert!(now < last, "closer: {now} < {last}");
            last = now;
        }
    }

    #[test]
    fn it_attacks_in_reach_once_per_cooldown_and_the_player_can_die() {
        let w = world(&[]);
        let mut c = Creature::new(&SLIME, [1.0, 10.0, 0.5]);
        settle(&mut c, &w);
        let mut p = Health::full(PLAYER_MAX_HEALTH);
        let mut hits = Vec::new();
        for tick in 0..700u32 {
            if let Some(h) = c.update(&w, [0.5, 10.0, 0.5], &mut p) {
                hits.push((tick, h.dealt, h.remaining));
            }
        }
        assert_eq!(
            c.state(),
            CreatureState::Idle,
            "the dead player is not a target"
        );
        assert!(hits.len() >= 2 && hits.iter().all(|h| h.1 == 2));
        assert_eq!(hits[1].0 - hits[0].0, 60, "one hit per cooldown");
        assert!(p.is_dead(), "ten hits of two take twenty health");
        assert_eq!(hits.len(), 10);
    }

    #[test]
    fn a_wall_stops_it_and_it_never_ends_up_inside_a_solid_voxel() {
        // A wall two voxels high and several wide between the creature and the target.
        let mut wall = Vec::new();
        for z in -6..=6 {
            for y in 10..13 {
                wall.push([4, y, z]);
            }
        }
        let w = world(&wall);
        let mut c = Creature::new(&SLIME, [8.5, 10.0, 0.5]);
        settle(&mut c, &w);
        let mut p = Health::full(PLAYER_MAX_HEALTH);
        for _ in 0..600 {
            c.update(&w, [0.5, 10.0, 0.5], &mut p);
            assert!(!Walker::collides_at(&w, c.pos()), "never inside geometry");
        }
        assert!(c.pos()[0] > 5.0, "the wall held it: x = {}", c.pos()[0]);
        assert_eq!(p.cur(), PLAYER_MAX_HEALTH, "it never reached the player");
    }

    #[test]
    fn the_player_can_kill_it_and_a_dead_creature_does_nothing() {
        let w = world(&[]);
        let mut c = Creature::new(&SLIME, [1.0, 10.0, 0.5]);
        settle(&mut c, &w);
        let mut mine = AttackTimer::new();
        let mut kills = 0;
        let mut p = Health::full(1000);
        for tick in 0..400u32 {
            c.update(&w, [0.5, 10.0, 0.5], &mut p);
            let d = distance_milli(c.pos(), [0.5, 10.0, 0.5]);
            if let Ok(h) = c.receive_attack(&WOODEN_SWORD, &mut mine, d) {
                kills += u32::from(h.killed);
            }
            mine.tick();
            let _ = tick;
        }
        assert_eq!(kills, 1, "reported once");
        assert_eq!(c.state(), CreatureState::Dead);
        let before = (c.pos(), p);
        for _ in 0..100 {
            assert!(c.update(&w, [0.5, 10.0, 0.5], &mut p).is_none());
        }
        assert_eq!((c.pos(), p), before, "no movement, no damage once dead");
    }

    #[test]
    fn equal_inputs_give_equal_creatures() {
        let run = || {
            let w = world(&[]);
            let mut c = Creature::new(&SLIME, [9.5, 12.0, 3.5]);
            let mut p = Health::full(PLAYER_MAX_HEALTH);
            for _ in 0..500 {
                c.update(&w, [0.5, 10.0, 0.5], &mut p);
            }
            (c, p)
        };
        assert_eq!(run(), run());
    }
}
