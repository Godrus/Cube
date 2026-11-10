//! The first-world session (DEC-092): one object that an engine adapter drives.
//!
//! `Session` joins the authoritative [`LocalHost`] with the engine-neutral [`ClientWorld`]:
//! the camera follows intents, the active region follows the camera, derived faces come from
//! the host's Processing executor, edit requests derived from what the camera sees go back to
//! the host (the only commit path), and what the engine must show is returned as plain
//! [`MeshBuffers`]. It computes no geometry, owns no world state and holds no handle.
//!
//! The active region follows the camera. Before it moves, the host is asked to generate the
//! bricks around the new centre (one brick more than the region, so every face of the region is
//! decided with a real neighbour), a few per frame; until they exist the region stays where it is.
//! Bricks are never unloaded and the world has a finite edge (`WORLD_LIMIT_BRICKS`).

pub mod combat;
pub mod creature;
pub mod game;
pub mod port;
pub mod spawn;

use std::sync::{Arc, Mutex, PoisonError};

use cube_client::{
    Camera, ClientWorld, EditIntent, EditPort, FaceIndex, Intent, MeshBuffers, UpdateReport,
    ViewProfile, WalkInput, Walker,
};
use cube_core::{BrickCoord, CellKey, DirtyRegion, LayerId};
use cube_orchestration::DirtySink;
use cube_server::{EditRequest, HostError, LocalHost, VOXEL_LAYER, WORLD_LIMIT_BRICKS};

pub use cube_client::{DeviceClass, Intent as PlayerIntent};

/// What one frame produced for the engine.
pub struct Frame {
    /// Bricks to create or replace, nearest first.
    pub upsert: Vec<(CellKey, MeshBuffers)>,
    /// Bricks to remove.
    pub remove: Vec<CellKey>,
    /// Client statistics.
    pub report: UpdateReport,
}

struct ToClient(Arc<ClientWorld>);

impl DirtySink for ToClient {
    fn dirty(&self, regions: &[DirtyRegion]) {
        self.0.on_dirty(regions);
    }
}

/// Distance of the third-person camera behind the eye, and the gap it keeps from a wall.
const CHASE_DISTANCE: f64 = 4.0;
const CHASE_MARGIN: f64 = 0.3;
/// Below this height (voxels) the player has left the world and is put back at the spawn point.
const VOID_Y: f64 = -64.0;
const EYE: f64 = cube_client::EYE_HEIGHT;
/// Streaming limits (all in bricks). Defaults are engineering values, not measured optima.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamBudget {
    /// Bricks the host may generate in one frame (generation is cheap but not free).
    pub load_per_frame: usize,
    /// Bricks beyond the view region that are kept generated (so faces see real neighbours).
    pub load_margin: i64,
    /// Extra distance beyond the loaded surroundings before a brick is unloaded (hysteresis:
    /// walking back and forth over a border must not load and unload the same bricks).
    pub evict_hysteresis: i64,
    /// Upper bound on resident bricks; the farthest unedited bricks beyond the loaded
    /// surroundings are unloaded first. Edited bricks are never unloaded, so edits can exceed it.
    pub max_resident: usize,
}

impl Default for StreamBudget {
    fn default() -> Self {
        Self {
            load_per_frame: 48,
            load_margin: 1,
            evict_hysteresis: 2,
            max_resident: 1500,
        }
    }
}

/// The player as it is saved between runs (client-side data, not world state).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerState {
    /// Feet position.
    pub feet: [f64; 3],
    /// View yaw.
    pub yaw: f32,
    /// View pitch.
    pub pitch: f32,
    /// Walking (`true`) or free flight.
    pub walking: bool,
    /// Third-person camera.
    pub third_person: bool,
}

/// How far the player can reach to mine or build (voxels from the eye).
pub const REACH: f64 = 6.0;

/// What the crosshair points at.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Target {
    /// The solid voxel.
    pub voxel: [i64; 3],
    /// The empty voxel in front of the hit face.
    pub front: [i64; 3],
    /// Its material.
    pub material: u16,
    /// Distance from the eye to the hit face.
    pub distance: f64,
}

/// Collision data around the player: the derived faces of the bricks nearby.
struct Near {
    brick: BrickCoord,
    /// The faces of the player's own brick are known (otherwise there is no ground yet and the
    /// body must not fall).
    ready: bool,
    index: FaceIndex,
}

/// A running first world.
pub struct Session {
    host: Arc<LocalHost>,
    client: Arc<ClientWorld>,
    profile: ViewProfile,
    camera: Mutex<Camera>,
    centre: Mutex<Option<BrickCoord>>,
    walker: Mutex<Option<Walker>>,
    third_person: Mutex<bool>,
    spawn: Mutex<[f64; 3]>,
    near: Mutex<Option<Near>>,
    budget: Mutex<StreamBudget>,
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

impl Session {
    /// Starts a session on `host` with the camera at `camera_pos` (voxel units).
    pub fn new(
        host: Arc<LocalHost>,
        profile: ViewProfile,
        camera_pos: [f64; 3],
    ) -> Result<Self, HostError> {
        let client = Arc::new(ClientWorld::new(
            profile.config(LayerId::new(VOXEL_LAYER), 0, cube_client::faces_kind()),
            host.executor(),
        ));
        client
            .start()
            .map_err(|e| HostError::Setup(format!("{e:?}")))?;
        host.add_dirty_listener(Arc::new(ToClient(client.clone())));
        Ok(Self {
            host,
            client,
            profile,
            camera: Mutex::new(Camera::new(camera_pos)),
            centre: Mutex::new(None),
            walker: Mutex::new(None),
            third_person: Mutex::new(false),
            spawn: Mutex::new([camera_pos[0], camera_pos[1] - EYE, camera_pos[2]]),
            near: Mutex::new(None),
            budget: Mutex::new(StreamBudget::default()),
        })
    }

    /// Replaces the streaming limits.
    pub fn set_budget(&self, b: StreamBudget) {
        *lock(&self.budget) = b;
    }

    /// Streaming and memory counters.
    pub fn stats(&self) -> cube_client::SessionStats {
        let s = self.host.stream_stats();
        cube_client::SessionStats {
            resident: s.resident as u64,
            edited: s.edited as u64,
            loads: s.loads,
            evictions: s.evictions,
            derived_cached: self.host.executor().cached_entries() as u64,
            active: self.client.cached() as u64,
        }
    }

    /// The host (read-only use: hash, record).
    pub fn host(&self) -> &Arc<LocalHost> {
        &self.host
    }

    /// Turns the camera without running a frame (so no batch of geometry is consumed).
    pub fn look(&self, yaw: f32, pitch: f32) {
        if let Some(w) = lock(&self.walker).as_mut() {
            w.look(yaw, pitch);
        }
        lock(&self.camera).apply(&Intent::Look { yaw, pitch }, 0.0);
    }

    /// The current view camera (free-fly camera, or the walking player's eye or chase camera).
    pub fn camera(&self) -> Camera {
        *lock(&self.camera)
    }

    /// `true` while the player walks (gravity, collision) instead of flying.
    pub fn walking(&self) -> bool {
        lock(&self.walker).is_some()
    }

    /// Switches between walking and free flight. Walking starts where the camera is (the feet
    /// below the eye); flying continues from the eye.
    pub fn set_walk(&self, on: bool) {
        let mut w = lock(&self.walker);
        match (on, w.is_some()) {
            (true, false) => {
                let cam = self.camera();
                let mut walker = Walker::new([cam.pos[0], cam.pos[1] - EYE, cam.pos[2]]);
                walker.yaw = cam.yaw;
                walker.pitch = cam.pitch;
                *w = Some(walker);
            }
            (false, true) => {
                if let Some(walker) = w.take() {
                    let mut cam = lock(&self.camera);
                    cam.pos = walker.eye();
                    cam.yaw = walker.yaw;
                    cam.pitch = walker.pitch;
                }
                *lock(&self.third_person) = false;
            }
            _ => {}
        }
    }

    /// Chooses the third-person camera (only while walking).
    pub fn set_third_person(&self, on: bool) {
        let walking = self.walking();
        *lock(&self.third_person) = on && walking;
    }

    /// The player's feet position while walking, `None` when flying.
    pub fn player_feet(&self) -> Option<[f64; 3]> {
        lock(&self.walker).as_ref().map(|w| w.pos)
    }

    /// Standing on something (walking only).
    pub fn grounded(&self) -> bool {
        lock(&self.walker).as_ref().is_some_and(|w| w.grounded)
    }

    /// What to save of the player.
    pub fn player_state(&self) -> PlayerState {
        let third_person = *lock(&self.third_person);
        let walker = *lock(&self.walker);
        match walker {
            Some(w) => PlayerState {
                feet: w.pos,
                yaw: w.yaw,
                pitch: w.pitch,
                walking: true,
                third_person,
            },
            None => {
                let c = self.camera();
                PlayerState {
                    feet: [c.pos[0], c.pos[1] - EYE, c.pos[2]],
                    yaw: c.yaw,
                    pitch: c.pitch,
                    walking: false,
                    third_person: false,
                }
            }
        }
    }

    /// Puts the player back where a save left them (before the first frame).
    pub fn restore_player(&self, s: &PlayerState) {
        {
            let mut cam = lock(&self.camera);
            cam.pos = [s.feet[0], s.feet[1] + EYE, s.feet[2]];
            cam.yaw = s.yaw;
            cam.pitch = s.pitch;
        }
        *lock(&self.walker) = None;
        if s.walking {
            self.set_walk(true);
            if let Some(w) = lock(&self.walker).as_mut() {
                w.pos = s.feet;
            }
            self.set_third_person(s.third_person);
        }
    }

    /// Runs `f` on the collision data around `feet`; it is rebuilt only when the player changes
    /// brick or a batch of new geometry arrived.
    fn with_near<R>(&self, feet: [f64; 3], f: impl FnOnce(&Near) -> R) -> R {
        let v = |c: f64| (c.floor() as i64).div_euclid(16);
        let brick = BrickCoord::new(v(feet[0]), v(feet[1]), v(feet[2]));
        let mut slot = lock(&self.near);
        if slot.as_ref().is_none_or(|n| n.brick != brick) {
            let executor = self.host.executor();
            let mut faces = Vec::new();
            let mut ready = false;
            for dx in -1..=1 {
                for dy in -1..=1 {
                    for dz in -1..=1 {
                        let b = BrickCoord::new(brick.x + dx, brick.y + dy, brick.z + dz);
                        let Ok(key) = CellKey::new(LayerId::new(VOXEL_LAYER), 0, b) else {
                            continue;
                        };
                        let t = cube_processing::DerivedTarget {
                            kind: cube_client::faces_kind(),
                            key,
                        };
                        let payload = executor.get(t).ok().flatten();
                        if let Some(m) = payload
                            .as_ref()
                            .and_then(|p| p.downcast_ref::<cube_backend_voxel::FacesMesh>())
                        {
                            faces.extend(m.faces.iter().copied());
                            if (dx, dy, dz) == (0, 0, 0) {
                                ready = true;
                            }
                        }
                    }
                }
            }
            *slot = Some(Near {
                brick,
                ready,
                index: FaceIndex::new(faces.iter()),
            });
        }
        match slot.as_ref() {
            Some(n) => f(n),
            None => unreachable!("the slot was filled above"),
        }
    }

    /// The block under the crosshair within `reach` voxels of the eye, read from the faces the
    /// client holds around the player (no frame is run, nothing changes).
    pub fn target(&self, reach: f64) -> Option<Target> {
        let walker = *lock(&self.walker);
        let (eye, dir) = match walker {
            Some(w) => (w.eye(), w.forward()),
            None => {
                let c = self.camera();
                (c.pos, c.forward())
            }
        };
        self.with_near(eye, |near| {
            if !near.ready {
                return None;
            }
            cube_client::pick_distance(&near.index, eye, dir, reach).map(|(h, d)| Target {
                voxel: h.voxel,
                front: h.front,
                material: h.material,
                distance: d,
            })
        })
    }

    /// Runs `f` on the collision data around `feet` and the brick it was built for; `None` while
    /// the player's own brick has no faces yet. Creatures in the bricks next to that brick may be
    /// advanced against this data (it holds the 3 x 3 x 3 bricks around it).
    pub fn with_faces_near<R>(
        &self,
        feet: [f64; 3],
        f: impl FnOnce(&FaceIndex, BrickCoord) -> R,
    ) -> Option<R> {
        self.with_near(feet, |n| n.ready.then(|| f(&n.index, n.brick)))
    }

    /// The seed of the world (the spawn rule depends on it).
    pub fn world_seed(&self) -> u64 {
        self.host.record().seed.raw()
    }

    /// Ground height of the voxel column `(x, z)`.
    pub fn ground_height(&self, x: i64, z: i64) -> i64 {
        self.host.surface_height(x, z)
    }

    /// `true` if a block in `voxel` would intersect the walking player's body.
    pub fn body_overlaps(&self, voxel: [i64; 3]) -> bool {
        lock(&self.walker).is_some_and(|w| w.overlaps_voxel(voxel))
    }

    /// Removes the block in `voxel` (the request goes through the host's Editing API).
    pub fn remove_voxel(&self, voxel: [i64; 3]) -> Result<(), HostError> {
        self.submit(EditIntent::Remove { voxel })
            .map_err(HostError::Edit)
    }

    /// Takes the operation sequence of the next edit, so that its id can be recorded (by the
    /// game state) before the world changes.
    pub fn reserve_edit_seq(&self) -> u64 {
        self.host.reserve_edit_seq()
    }

    /// Removes the block in `voxel` under a sequence from [`Session::reserve_edit_seq`].
    pub fn remove_voxel_reserved(&self, voxel: [i64; 3], seq: u64) -> Result<(), HostError> {
        self.host
            .submit_edit_reserved(EditRequest::Remove { voxel }, seq)
    }

    /// Places `material` in `voxel` under a sequence from [`Session::reserve_edit_seq`]; `Ok(false)`
    /// (nothing submitted) if it would fill the player's body.
    pub fn place_voxel_reserved(
        &self,
        voxel: [i64; 3],
        material: u16,
        seq: u64,
    ) -> Result<bool, HostError> {
        if self.body_overlaps(voxel) {
            return Ok(false);
        }
        self.host
            .submit_edit_reserved(EditRequest::Place { voxel, material }, seq)
            .map(|()| true)
    }

    /// Places `material` in `voxel`; refused (`Ok(false)`) if it would fill the player's body.
    pub fn place_voxel(&self, voxel: [i64; 3], material: u16) -> Result<bool, HostError> {
        if self.body_overlaps(voxel) {
            return Ok(false);
        }
        self.submit(EditIntent::Place { voxel, material })
            .map(|()| true)
            .map_err(HostError::Edit)
    }

    /// Keeps the walking body inside the loaded area (an invisible wall at its edge) and rescues
    /// it from the void.
    fn confine(&self, w: &mut Walker) {
        let lo = (-WORLD_LIMIT_BRICKS * 16) as f64 + 1.0;
        let hi = ((WORLD_LIMIT_BRICKS + 1) * 16) as f64 - 1.0;
        w.pos[0] = w.pos[0].clamp(lo, hi);
        w.pos[2] = w.pos[2].clamp(lo, hi);
        if w.pos[1] < VOID_Y {
            w.pos = *lock(&self.spawn);
            w.vel_y = 0.0;
        }
    }

    /// Runs the walking player for one frame and updates the view camera.
    fn walk_frame(&self, intents: &[(Intent, f32)]) {
        let mut body = match *lock(&self.walker) {
            Some(w) => w,
            None => return,
        };
        let (mut input, mut dt) = (WalkInput::default(), 0.0f32);
        for (i, t) in intents {
            match *i {
                Intent::Look { yaw, pitch } => body.look(yaw, pitch),
                Intent::Move { forward, right, up } => {
                    input = WalkInput {
                        forward,
                        right,
                        jump: up > 0,
                    };
                    dt += *t;
                }
                Intent::Speed(_) => {}
            }
        }
        // No ground yet (its derived faces have not arrived): the body must not fall.
        let moved = self.with_near(body.pos, |near| {
            let mut b = body;
            if near.ready && dt > 0.0 {
                b.step(&near.index, &input, dt);
            }
            b
        });
        body = moved;
        self.confine(&mut body);
        let third = *lock(&self.third_person);
        let pos = if third {
            self.with_near(body.pos, |near| {
                body.chase_position(&near.index, CHASE_DISTANCE, CHASE_MARGIN)
            })
        } else {
            body.eye()
        };
        *lock(&self.walker) = Some(body);
        let mut cam = lock(&self.camera);
        cam.pos = pos;
        cam.yaw = body.yaw;
        cam.pitch = body.pitch;
    }

    /// The brick `b` moved into the finite world (horizontally).
    fn clamp(&self, b: BrickCoord) -> BrickCoord {
        let l = WORLD_LIMIT_BRICKS;
        BrickCoord::new(b.x.clamp(-l, l), b.y, b.z.clamp(-l, l))
    }

    /// One frame: applies the timed intents to the camera, moves the active region with it,
    /// lets the client compute derived geometry within its budget and returns what to show.
    pub fn frame(&self, intents: &[(Intent, f32)]) -> Result<Frame, HostError> {
        let want = if self.walking() {
            self.walk_frame(intents);
            self.clamp(self.camera().brick())
        } else {
            let mut cam = lock(&self.camera);
            for (i, dt) in intents {
                cam.apply(i, *dt);
            }
            self.clamp(cam.brick())
        };
        {
            let r = i64::from(self.profile.radius);
            let bud = *lock(&self.budget);
            let mut c = lock(&self.centre);
            if *c != Some(want) {
                // Generate the new surroundings (region + one brick) first; move the region only
                // when they all exist, so no face is ever decided against a missing neighbour.
                // The very first region is generated in full (nothing is shown before it exists).
                let budget = if c.is_none() {
                    usize::MAX
                } else {
                    bud.load_per_frame
                };
                let missing = self.host.ensure_loaded(want, r + bud.load_margin, budget)?;
                if missing == 0 {
                    self.client
                        .set_interest(want, self.profile.radius)
                        .map_err(|e| HostError::Setup(format!("{e:?}")))?;
                    *c = Some(want);
                    // Unload what the player left far behind (never edited bricks).
                    self.host.evict_far(
                        want,
                        r + bud.load_margin + bud.evict_hysteresis,
                        r + bud.load_margin,
                        bud.max_resident,
                    )?;
                }
            }
        }
        let report = self
            .client
            .update()
            .map_err(|e| HostError::Setup(format!("{e:?}")))?;
        let batch = self.client.take_batch();
        if !batch.upsert.is_empty() || !batch.remove.is_empty() {
            *lock(&self.near) = None; // new geometry: collision data is rebuilt on demand
        }
        let upsert = batch
            .upsert
            .iter()
            .filter_map(|(k, p)| {
                p.downcast_ref::<cube_backend_voxel::FacesMesh>()
                    .map(|m| (*k, MeshBuffers::from_faces(m)))
            })
            .collect();
        Ok(Frame {
            upsert,
            remove: batch.remove,
            report,
        })
    }

    /// Adds (`Some(material)`) or removes (`None`) the block the camera looks at, within
    /// `reach` voxels. The change goes through the host; the geometry follows on the next frame.
    pub fn edit(&self, reach: f64, place: Option<u16>) -> Result<bool, HostError> {
        // The pointer ray starts at the eye (not at the third-person camera).
        let walker = *lock(&self.walker);
        let cam = match walker {
            Some(w) => Camera {
                pos: w.eye(),
                yaw: w.yaw,
                pitch: w.pitch,
                speed: 0.0,
            },
            None => self.camera(),
        };
        let index = self.face_index();
        let Some(intent) = cube_client::edit_from_camera(&index, &cam, reach, place) else {
            return Ok(false);
        };
        if let (Some(w), EditIntent::Place { voxel, .. }) = (walker, intent)
            && w.overlaps_voxel(voxel)
        {
            return Ok(false); // never build a block into the player's own body
        }
        self.submit(intent).map_err(HostError::Edit)?;
        Ok(true)
    }

    fn face_index(&self) -> cube_client::FaceIndex {
        let executor = self.host.executor();
        let mut faces = Vec::new();
        for b in self.client.active() {
            let Ok(key) = CellKey::new(LayerId::new(VOXEL_LAYER), 0, b) else {
                continue;
            };
            let t = cube_processing::DerivedTarget {
                kind: cube_client::faces_kind(),
                key,
            };
            let payload = executor.get(t).ok().flatten();
            if let Some(m) = payload
                .as_ref()
                .and_then(|p| p.downcast_ref::<cube_backend_voxel::FacesMesh>())
            {
                faces.extend(m.faces.iter().copied());
            }
        }
        cube_client::FaceIndex::new(faces.iter())
    }

    /// Stops the client and returns the bricks the engine must drop.
    pub fn stop(&self) -> Result<Vec<CellKey>, HostError> {
        self.client
            .stop()
            .map(|b| b.remove)
            .map_err(|e| HostError::Setup(format!("{e:?}")))
    }
}

impl EditPort for Session {
    fn submit(&self, edit: EditIntent) -> Result<(), String> {
        let req = match edit {
            EditIntent::Place { voxel, material } => EditRequest::Place { voxel, material },
            EditIntent::Remove { voxel } => EditRequest::Remove { voxel },
        };
        self.host.submit_edit(req).map_err(|e| e.to_string())
    }
}
