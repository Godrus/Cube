//! The first-world session (DEC-092): one object that an engine adapter drives.
//!
//! `Session` joins the authoritative [`LocalHost`] with the engine-neutral [`ClientWorld`]:
//! the camera follows intents, the active region follows the camera, derived faces come from
//! the host's Processing executor, edit requests derived from what the camera sees go back to
//! the host (the only commit path), and what the engine must show is returned as plain
//! [`MeshBuffers`]. It computes no geometry, owns no world state and holds no handle.
//!
//! The active region is limited to the host's static bounded area (one brick of apron stays
//! inside it); this is not streaming.

pub mod port;

use std::sync::{Arc, Mutex, PoisonError};

use cube_client::{
    Camera, ClientWorld, EditIntent, EditPort, Intent, MeshBuffers, UpdateReport, ViewProfile,
};
use cube_core::{BrickCoord, CellKey, DirtyRegion, LayerId};
use cube_orchestration::DirtySink;
use cube_server::{EditRequest, HostError, LocalHost, VOXEL_LAYER};

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

/// A running first world.
pub struct Session {
    host: Arc<LocalHost>,
    client: Arc<ClientWorld>,
    profile: ViewProfile,
    camera: Mutex<Camera>,
    centre: Mutex<Option<BrickCoord>>,
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
        })
    }

    /// The host (read-only use: hash, record).
    pub fn host(&self) -> &Arc<LocalHost> {
        &self.host
    }

    /// Turns the camera without running a frame (so no batch of geometry is consumed).
    pub fn look(&self, yaw: f32, pitch: f32) {
        lock(&self.camera).apply(&Intent::Look { yaw, pitch }, 0.0);
    }

    /// The current camera pose.
    pub fn camera(&self) -> Camera {
        *lock(&self.camera)
    }

    /// The active region radius actually used: the profile's, kept one brick inside the
    /// host's loaded area so that every face is decided with a real neighbour.
    fn radius(&self) -> u32 {
        let inner = (self.host.area().radius - 1).max(0) as u32;
        self.profile.radius.min(inner)
    }

    fn clamp(&self, b: BrickCoord) -> BrickCoord {
        let a = self.host.area();
        let r = i64::from(self.radius());
        let c = |v: i64, centre: i64| v.clamp(centre - a.radius + r + 1, centre + a.radius - r - 1);
        BrickCoord::new(c(b.x, a.centre.x), c(b.y, a.centre.y), c(b.z, a.centre.z))
    }

    /// One frame: applies the timed intents to the camera, moves the active region with it,
    /// lets the client compute derived geometry within its budget and returns what to show.
    pub fn frame(&self, intents: &[(Intent, f32)]) -> Result<Frame, HostError> {
        let want = {
            let mut cam = lock(&self.camera);
            for (i, dt) in intents {
                cam.apply(i, *dt);
            }
            self.clamp(cam.brick())
        };
        {
            let mut c = lock(&self.centre);
            if *c != Some(want) {
                self.client
                    .set_interest(want, self.radius())
                    .map_err(|e| HostError::Setup(format!("{e:?}")))?;
                *c = Some(want);
            }
        }
        let report = self
            .client
            .update()
            .map_err(|e| HostError::Setup(format!("{e:?}")))?;
        let batch = self.client.take_batch();
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
        let cam = self.camera();
        let index = self.face_index();
        let Some(intent) = cube_client::edit_from_camera(&index, &cam, reach, place) else {
            return Ok(false);
        };
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
