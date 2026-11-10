//! The session as an engine-neutral port, and the Godot 4 library entry point.
//!
//! This is the composition root of the first world (DEC-092): it builds a [`Session`] from a
//! directory and a seed, wraps it as a [`SessionPort`] and installs the factory in the thin Godot
//! adapter. It contains no algorithm of its own.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use cube_client::{Controls, DeviceClass, FrameOut, Intent, Pose, SessionPort, ViewProfile};
use cube_core::{CellKey, GeneratorVersion, WorldSeed};
use cube_persistence::{STRUCTURE_VERSION, WorldRecord};
use cube_server::{LocalHost, StaticArea};

use crate::combat::{AttackRefused, AttackTimer, WOODEN_SWORD};
use crate::creature::TICK;
use crate::game::{BLOCK_ITEMS, GameState, ITEMS, Mode, Reconciled, SLOTS};
use crate::loot::loot_key;
use crate::spawn::{Population, SpawnArea};
use crate::{PlayerState, REACH, Session};

/// Radius of the static bounded area of the first world, in bricks.
pub const FIRST_WORLD_RADIUS: i64 = 5;

/// File next to the world that remembers where the player stood (client data, not world state).
pub const PLAYER_FILE: &str = "player.state";
/// File next to the world with the inventory, rules and objective progress.
pub const GAME_FILE: &str = "game.state";
/// Seconds between automatic saves of the player.
const AUTOSAVE_SECONDS: f32 = 5.0;

/// Text form of a [`PlayerState`]: one `key value...` line each, version first.
pub fn encode_player(s: &PlayerState) -> String {
    format!(
        "cube-player 1\nfeet {} {} {}\nview {} {}\nwalking {}\nthird_person {}\n",
        s.feet[0],
        s.feet[1],
        s.feet[2],
        s.yaw,
        s.pitch,
        u8::from(s.walking),
        u8::from(s.third_person)
    )
}

/// Reads [`encode_player`] output; `None` for anything that is not a complete, finite state.
pub fn decode_player(text: &str) -> Option<PlayerState> {
    let mut lines = text.lines();
    if lines.next()? != "cube-player 1" {
        return None;
    }
    let (mut feet, mut view, mut walking, mut third) = (None, None, None, None);
    for l in lines {
        let mut it = l.split_whitespace();
        match it.next()? {
            "feet" => {
                let v: Vec<f64> = it.map(str::parse).collect::<Result<_, _>>().ok()?;
                feet = <[f64; 3]>::try_from(v).ok();
            }
            "view" => {
                let v: Vec<f32> = it.map(str::parse).collect::<Result<_, _>>().ok()?;
                view = <[f32; 2]>::try_from(v).ok();
            }
            "walking" => walking = Some(it.next()? == "1"),
            "third_person" => third = Some(it.next()? == "1"),
            _ => return None,
        }
    }
    let (feet, view) = (feet?, view?);
    if !feet.iter().all(|c| c.is_finite()) || !view.iter().all(|c| c.is_finite()) {
        return None;
    }
    Some(PlayerState {
        feet,
        yaw: view[0],
        pitch: view[1],
        walking: walking?,
        third_person: third?,
    })
}

/// Writes the player next to the world (temporary file, then rename: a crash never leaves a
/// half-written file).
pub fn save_player(dir: &Path, s: &PlayerState) -> std::io::Result<()> {
    let tmp = dir.join(format!("{PLAYER_FILE}.tmp"));
    std::fs::write(&tmp, encode_player(s))?;
    std::fs::rename(&tmp, dir.join(PLAYER_FILE))
}

/// The saved player of the world in `dir`, if there is a valid one.
pub fn load_player(dir: &Path) -> Option<PlayerState> {
    decode_player(&std::fs::read_to_string(dir.join(PLAYER_FILE)).ok()?)
}

/// Writes the game state next to the world: temporary file, flushed to the device, then renamed
/// over the old one, so a crash leaves the old or the new file, never a mixture. (No directory
/// sync: no claim about power loss on a device, DG-2.)
pub fn save_game(dir: &Path, g: &GameState) -> std::io::Result<()> {
    use std::io::Write;
    let tmp = dir.join(format!("{GAME_FILE}.tmp"));
    let mut f = std::fs::File::create(&tmp)?;
    f.write_all(g.encode().as_bytes())?;
    f.sync_all()?;
    drop(f);
    std::fs::rename(&tmp, dir.join(GAME_FILE))
}

/// The saved game state of the world in `dir`, if there is a valid one.
pub fn load_game(dir: &Path) -> Option<GameState> {
    GameState::decode(&std::fs::read_to_string(dir.join(GAME_FILE)).ok()?)
}

/// A session behind the port an adapter drives.
pub struct PortSession {
    session: Session,
    dir: PathBuf,
    since_save: f32,
    game: GameState,
    message: String,
    message_left: f32,
    /// Creatures near the player; `None` while creatures are switched off (the default: the
    /// player cannot fight back before CMB-09).
    creatures: Option<Population>,
    creature_time: f32,
    attack_timer: AttackTimer,
    attack_time: f32,
}

impl PortSession {
    /// Wraps a session whose world lives in `dir` (the player and the game state are saved there).
    pub fn new(session: Session, dir: PathBuf) -> Self {
        Self::with_game(session, dir, GameState::default())
    }

    /// Like [`PortSession::new`], continuing a saved game state.
    pub fn with_game(session: Session, dir: PathBuf, game: GameState) -> Self {
        Self {
            session,
            dir,
            since_save: 0.0,
            game,
            message: String::new(),
            message_left: 0.0,
            creatures: None,
            creature_time: 0.0,
            attack_timer: AttackTimer::new(),
            attack_time: 0.0,
        }
    }

    /// Switches creatures on (internal: the environment variable `CUBE_CREATURES=1` does it for
    /// the Godot demo; they are not a player feature before CMB-09).
    pub fn enable_creatures(&mut self) {
        self.creatures.get_or_insert_with(Population::new);
    }

    /// The player swings the wooden sword at the nearest creature. A hit is resolved by the
    /// combat rules (cooldown, reach). A kill rolls the creature's loot (a pure function of the
    /// world seed and the creature's column) and puts it into the inventory; loot that does not
    /// fit is lost and the player is told. Internal until CMB-09 gives it an input.
    pub fn attack(&mut self) -> Result<crate::spawn::Strike, AttackRefused> {
        let Some(feet) = self.session.player_feet() else {
            return Err(AttackRefused::OutOfReach);
        };
        let Some(pop) = self.creatures.as_mut() else {
            return Err(AttackRefused::OutOfReach);
        };
        let strike = pop.player_attack(&WOODEN_SWORD, &mut self.attack_timer, feet)?;
        if strike.hit.killed {
            let key = loot_key(self.session.world_seed(), strike.column.0, strike.column.1);
            let name = strike.def.name;
            match strike.def.loot.roll(key) {
                Some(drop) => {
                    let left = self.game.grant_loot(drop.item, drop.count);
                    let item = crate::game::definition(drop.item).map_or("item", |d| d.name);
                    if left > 0 {
                        self.say(format!(
                            "{name} defeated. Inventory full: {left} {item} lost"
                        ));
                    } else {
                        self.say(format!("{name} defeated: +{} {item}", drop.count));
                    }
                    self.save_game();
                }
                None => self.say(format!("{name} defeated")),
            }
        }
        Ok(strike)
    }

    /// The creatures, if switched on (tests, and later the view).
    pub fn creatures(&self) -> Option<&Population> {
        self.creatures.as_ref()
    }

    /// Spawns, despawns and advances the creatures for the time `dt` (fixed ticks, a bounded
    /// number per frame). The player's health follows the creatures' hits; at zero the player
    /// is told and respawns with full health where they stand (prototype rule).
    fn run_creatures(&mut self, dt: f32) {
        let Some(pop) = self.creatures.as_mut() else {
            return;
        };
        let Some(feet) = self.session.player_feet() else {
            return;
        };
        let (cx, cz) = (
            (feet[0].floor() as i64).div_euclid(16),
            (feet[2].floor() as i64).div_euclid(16),
        );
        let limit = cube_server::WORLD_LIMIT_BRICKS;
        pop.update(
            self.session.world_seed(),
            SpawnArea {
                centre: (cx, cz),
                spawn_radius: 4,
                despawn_radius: 6,
                max: 8,
            },
            |x, z| x.abs() <= limit && z.abs() <= limit,
            |x, z| self.session.ground_height(x, z),
        );
        self.creature_time = (self.creature_time + dt).min(5.0 * TICK);
        let mut died = false;
        while self.creature_time >= TICK {
            self.creature_time -= TICK;
            let health = &mut self.game.health;
            let ran = self.session.with_faces_near(feet, |faces, brick| {
                let covers = |p: [f64; 3]| {
                    let b = |v: f64| (v.floor() as i64).div_euclid(16);
                    (b(p[0]) - brick.x).abs() <= 1
                        && (b(p[1]) - brick.y).abs() <= 1
                        && (b(p[2]) - brick.z).abs() <= 1
                };
                pop.tick(faces, covers, feet, health)
            });
            if ran.is_some() && self.game.health.is_dead() {
                died = true;
                self.game.health.revive();
            }
        }
        if died {
            self.say("You died and got up again");
        }
    }

    /// The game state (tests and diagnostics).
    pub fn game(&self) -> &GameState {
        &self.game
    }

    fn save_game(&self) {
        let _ = save_game(&self.dir, &self.game);
    }

    /// Saves the game file and tells whether that worked (needed before a world edit).
    fn try_save_game(&self) -> std::io::Result<()> {
        save_game(&self.dir, &self.game)
    }

    fn say(&mut self, text: impl Into<String>) {
        self.message = text.into();
        self.message_left = 4.0;
    }

    fn save(&self) {
        // A failed save must not stop the game; the world itself is saved by the host.
        let _ = save_player(&self.dir, &self.session.player_state());
    }
}

impl SessionPort for PortSession {
    fn frame(&mut self, c: &Controls) -> Result<FrameOut, String> {
        self.since_save += c.dt;
        self.message_left = (self.message_left - c.dt).max(0.0);
        if self.since_save >= AUTOSAVE_SECONDS {
            self.since_save = 0.0;
            self.save();
        }
        let mut intents = Vec::new();
        if c.look_yaw != 0.0 || c.look_pitch != 0.0 {
            intents.push((
                Intent::Look {
                    yaw: c.look_yaw,
                    pitch: c.look_pitch,
                },
                0.0,
            ));
        }
        // A walking player needs time to pass even when no key is down (gravity).
        if c.forward != 0 || c.right != 0 || c.up != 0 || self.session.walking() {
            intents.push((
                Intent::Move {
                    forward: c.forward,
                    right: c.right,
                    up: c.up,
                },
                c.dt,
            ));
        }
        let f = self.session.frame(&intents).map_err(|e| e.to_string())?;
        self.attack_time = (self.attack_time + c.dt).min(5.0 * TICK);
        while self.attack_time >= TICK {
            self.attack_time -= TICK;
            self.attack_timer.tick();
        }
        self.run_creatures(c.dt);
        Ok(FrameOut {
            upsert: f.upsert,
            remove: f.remove,
            pending: f.report.pending,
        })
    }

    fn edit(&mut self, place: Option<u16>) -> Result<bool, String> {
        self.session.edit(60.0, place).map_err(|e| e.to_string())
    }

    fn pose(&self) -> Pose {
        let c = self.session.camera();
        Pose {
            pos: c.pos,
            yaw: c.yaw,
            pitch: c.pitch,
        }
    }

    fn shutdown(&mut self) -> Vec<CellKey> {
        self.save();
        self.save_game();
        self.session.stop().unwrap_or_default()
    }

    fn set_walk(&mut self, on: bool) {
        self.session.set_walk(on);
    }

    fn set_third_person(&mut self, on: bool) {
        self.session.set_third_person(on);
    }

    fn walking(&self) -> bool {
        self.session.walking()
    }

    fn player_feet(&self) -> Option<[f64; 3]> {
        self.session.player_feet()
    }

    fn grounded(&self) -> bool {
        self.session.grounded()
    }

    fn stats(&self) -> cube_client::SessionStats {
        self.session.stats()
    }

    fn target(&self) -> Option<cube_client::TargetInfo> {
        let t = self.session.target(REACH)?;
        let can_remove = self.game.can_keep(t.material);
        let selected = self.game.selected_material();
        let blocked = self.session.body_overlaps(t.front);
        let reason = if !can_remove {
            "Inventory full"
        } else if selected.is_none() {
            "Nothing to place in the selected slot"
        } else if blocked {
            "You are standing there"
        } else {
            ""
        };
        Some(cube_client::TargetInfo {
            voxel: t.voxel,
            front: t.front,
            material: t.material,
            distance: t.distance,
            can_remove,
            can_place: selected.is_some() && !blocked,
            reason: reason.to_owned(),
        })
    }

    fn mine(&mut self) -> Result<bool, String> {
        let Some(t) = self.session.target(REACH) else {
            return Ok(false);
        };
        if !self.game.can_keep(t.material) {
            self.say("Inventory full");
            return Err("Inventory full".into());
        }
        // 1. the intent reaches the game file; 2. the world changes; 3. the effect and the cleared
        // intent reach the file in one write. A crash anywhere is settled at the next start.
        let seq = self.session.reserve_edit_seq();
        self.game.begin_mined(seq, t.material);
        if let Err(e) = self.try_save_game() {
            self.game.abandon_pending();
            self.say("Could not save the game: nothing changed");
            return Err(format!("game state not saved: {e}"));
        }
        match self.session.remove_voxel_reserved(t.voxel, seq) {
            Ok(()) => {
                if let Some(m) = self.game.finish_pending() {
                    self.say(m);
                }
                self.save_game();
                Ok(true)
            }
            Err(e) => {
                self.game.abandon_pending();
                self.save_game();
                Err(e.to_string())
            }
        }
    }

    fn place(&mut self) -> Result<bool, String> {
        let Some(t) = self.session.target(REACH) else {
            return Ok(false);
        };
        let Some(material) = self.game.selected_material() else {
            self.say("Nothing to place in the selected slot");
            return Err("Nothing to place".into());
        };
        if self.session.body_overlaps(t.front) {
            self.say("You are standing there");
            return Err("You are standing there".into());
        }
        let seq = self.session.reserve_edit_seq();
        self.game.begin_placed(seq);
        if let Err(e) = self.try_save_game() {
            self.game.abandon_pending();
            self.say("Could not save the game: nothing changed");
            return Err(format!("game state not saved: {e}"));
        }
        match self.session.place_voxel_reserved(t.front, material, seq) {
            Ok(true) => {
                if let Some(m) = self.game.finish_pending() {
                    self.say(m);
                }
                self.save_game();
                Ok(true)
            }
            Ok(false) => {
                self.game.abandon_pending();
                self.save_game();
                self.say("You are standing there");
                Err("You are standing there".into())
            }
            Err(e) => {
                self.game.abandon_pending();
                self.save_game();
                Err(e.to_string())
            }
        }
    }

    fn game_view(&self) -> cube_client::GameView {
        let creative = self.game.mode == Mode::Creative;
        let slots = if creative {
            (0..SLOTS)
                .map(|i| {
                    if i < BLOCK_ITEMS {
                        (ITEMS[i].id.0, 0)
                    } else {
                        (0, 0)
                    }
                })
                .collect()
        } else {
            self.game
                .inventory
                .slots()
                .iter()
                .map(|s| s.map_or((0, 0), |st| (st.item.0, st.count)))
                .collect()
        };
        let (title, text) = self.game.objective.text();
        cube_client::GameView {
            slots,
            selected: self.game.inventory.selected(),
            creative,
            objective_title: title.to_owned(),
            objective_text: text,
            objective_done: self.game.objective.done(),
            message: if self.message_left > 0.0 {
                self.message.clone()
            } else {
                String::new()
            },
        }
    }

    fn select_slot(&mut self, slot: usize) {
        self.game.inventory.select(slot);
    }

    fn scroll_slot(&mut self, delta: i32) {
        self.game.inventory.scroll(delta);
    }

    fn swap_slots(&mut self, a: usize, b: usize) {
        if self.game.mode == Mode::Survival {
            self.game.inventory.swap(a, b);
            self.save_game();
        }
    }

    fn set_creative(&mut self, on: bool) {
        self.game.mode = if on { Mode::Creative } else { Mode::Survival };
        self.save_game();
    }
}

/// Opens (or creates) the first world in `dir` with `seed` and starts a session above its
/// surface, looking down at a gentle angle.
pub fn open_first_world(
    dir: &Path,
    seed: u64,
    create: bool,
) -> Result<Box<dyn SessionPort>, String> {
    let record = WorldRecord {
        world_id: 1,
        seed: WorldSeed::new(seed),
        generator_version: GeneratorVersion::new(1),
        structure_version: STRUCTURE_VERSION,
    };
    let area = StaticArea::around_origin(&record, FIRST_WORLD_RADIUS);
    let host = if create {
        LocalHost::create(dir, record, area)
    } else {
        LocalHost::open(dir, area)
    }
    .map_err(|e| e.to_string())?;
    let host = Arc::new(host);
    let h = host.surface_height(0, 0) as f64;
    let session = Session::new(
        host,
        ViewProfile::of(DeviceClass::Desktop),
        [0.5, h + 14.0, 0.5],
    )
    .map_err(|e| e.to_string())?;
    // Tilt down so the first picture shows the ground (no frame is run here: the first frame
    // belongs to the engine, which needs every batch of geometry).
    session.look(0.0, -0.6);
    if !create {
        // Continue where the player left off (a missing or damaged file means a fresh start).
        if let Some(saved) = load_player(dir) {
            session.restore_player(&saved);
        }
    }
    let mut game = if create { None } else { load_game(dir) }.unwrap_or_default();
    // An edit that was under way when the game stopped is settled against the journal now,
    // before anything new happens, and the result is saved at once.
    if game.reconcile(|seq| session.host().edit_confirmed_at_open(seq)) != Reconciled::Nothing {
        let _ = save_game(dir, &game);
    }
    let mut port = PortSession::with_game(session, dir.to_path_buf(), game);
    if std::env::var_os("CUBE_CREATURES").is_some_and(|v| v == "1") {
        port.enable_creatures();
    }
    Ok(Box::new(port))
}
