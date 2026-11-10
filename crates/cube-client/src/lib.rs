//! Cube client runtime foundation (CLI-00..02, parallel preparation; not production, no
//! authority). Engine-neutral: no renderer, no Godot, no networking yet.
//!
//! What it does: keeps an **active region** of bricks around a centre, asks a [`DerivedSource`]
//! for the derived geometry of those bricks (nearest first), caches the answers read-only, learns
//! about changes through [`ClientWorld::on_dirty`] and hands an engine adapter a [`RenderBatch`]
//! of what to show or drop. Bricks that leave the region are dropped from the cache; the
//! lifecycle is `Created -> Running -> Stopped`.
//!
//! What it does not do: own world state (it holds no commit handle and cannot write), compute
//! geometry itself (a second voxel engine inside the client is exactly what is not wanted) or
//! decide what a "better" mesh is (the provider and its version decide).
//!
//! Stale data: a brick whose derived value became stale keeps its last shown payload until the
//! new one is current (the batch carries the replacement as soon as Processing publishes it);
//! the report says how many bricks are in that state. A failed brick is reported and not retried
//! until it changes or re-enters the region.

pub mod buffers;
pub mod intent;
pub mod profile;
pub mod session;

pub use buffers::MeshBuffers;
pub use intent::{
    Camera, EditIntent, EditPort, FaceIndex, Hit, Intent, edit_from_camera, pick, replay,
};
pub use profile::{DeviceClass, ViewProfile};
pub use session::{CellKeyExt, Controls, FrameOut, Pose, SessionPort};

/// The derived kind of the baseline renderer (faces, GOV-13).
pub const fn faces_kind() -> DerivedKind {
    cube_backend_voxel::FACES_KIND
}

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex, PoisonError};

use cube_core::{BrickCoord, CellKey, DirtyRegion, LayerId};
use cube_processing::{
    DerivedKind, DerivedTarget, Executor, Payload, Priority, PriorityClass, TargetStatus,
};

/// Why a client call failed.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ClientError {
    /// The call is not allowed in the current lifecycle state.
    WrongState(Lifecycle),
    /// The requested active region is larger than the configured maximum.
    InterestTooLarge {
        /// Bricks requested.
        requested: u64,
        /// Bricks allowed.
        max: u64,
    },
    /// The coordinate range of the region is not representable.
    InvalidRegion,
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WrongState(s) => write!(f, "not allowed while {s:?}"),
            Self::InterestTooLarge { requested, max } => {
                write!(f, "active region of {requested} bricks exceeds {max}")
            }
            Self::InvalidRegion => f.write_str("active region is not representable"),
        }
    }
}

impl std::error::Error for ClientError {}

/// Lifecycle of the client world.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lifecycle {
    /// Constructed, not started.
    Created,
    /// Serving updates.
    Running,
    /// Stopped; the cache is gone. A stopped client is not restarted: create a new one.
    Stopped,
}

/// What a derived source says about one brick.
#[derive(Clone)]
pub enum Poll {
    /// A current value.
    Ready(Payload),
    /// Requested and being computed, or stale and waiting for a recomputation.
    Pending,
    /// Never requested.
    Missing,
}

/// Where derived geometry comes from. Implemented for Processing's [`Executor`]; a mock
/// implements it in tests, and a network source can implement it later.
pub trait DerivedSource: Send + Sync {
    /// Asks for the derived value of `target` with the given urgency.
    fn request(&self, target: DerivedTarget, class: PriorityClass) -> Result<(), String>;
    /// The current state of `target` (a stale value is never `Ready`).
    fn poll(&self, target: DerivedTarget) -> Result<Poll, String>;
    /// Tells the source that cells changed.
    fn invalidate(&self, dirty: &[DirtyRegion]);
    /// Lets the source do at most `budget` units of work on the calling thread; returns how many
    /// it did. Sources that work elsewhere do nothing.
    fn drive(&self, budget: usize) -> usize;
}

impl DerivedSource for Executor {
    fn request(&self, target: DerivedTarget, class: PriorityClass) -> Result<(), String> {
        self.submit(target, Priority::new(class))
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
    fn poll(&self, target: DerivedTarget) -> Result<Poll, String> {
        match self.query(target).map_err(|e| e.to_string())? {
            TargetStatus::Valid => match self.get(target).map_err(|e| e.to_string())? {
                Some(p) => Ok(Poll::Ready(p)),
                None => Ok(Poll::Pending),
            },
            TargetStatus::Computing | TargetStatus::Stale => Ok(Poll::Pending),
            TargetStatus::Missing => Ok(Poll::Missing),
        }
    }
    fn invalidate(&self, dirty: &[DirtyRegion]) {
        self.notify_commit(dirty);
    }
    fn drive(&self, budget: usize) -> usize {
        let mut n = 0;
        while n < budget && self.run_one() {
            n += 1;
        }
        n
    }
}

/// Static configuration of a client world.
#[derive(Debug, Clone, Copy)]
pub struct ClientConfig {
    /// The layer whose bricks are shown.
    pub layer: LayerId,
    /// Resolution level of the shown bricks.
    pub level: i8,
    /// The derived kind to show (for example the faces mesh).
    pub kind: DerivedKind,
    /// Largest active region, in bricks.
    pub max_active_bricks: u64,
    /// Work units the source may do per [`ClientWorld::update`] (a frame budget).
    pub work_per_update: usize,
}

/// What an adapter should do: show or replace these bricks, drop those.
#[derive(Clone, Default)]
pub struct RenderBatch {
    /// New or replaced derived values, nearest bricks first.
    pub upsert: Vec<(CellKey, Payload)>,
    /// Bricks to remove from the scene.
    pub remove: Vec<CellKey>,
}

impl RenderBatch {
    /// `true` if there is nothing to do.
    pub fn is_empty(&self) -> bool {
        self.upsert.is_empty() && self.remove.is_empty()
    }
}

/// The change of the active region.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InterestChange {
    /// Bricks that entered.
    pub entered: Vec<BrickCoord>,
    /// Bricks that left.
    pub left: Vec<BrickCoord>,
}

/// Outcome of one [`ClientWorld::update`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UpdateReport {
    /// Bricks of the active region with a current value in the cache.
    pub ready: usize,
    /// Bricks waiting for their first value.
    pub pending: usize,
    /// Bricks showing an older value while a newer one is computed.
    pub stale: usize,
    /// Bricks whose derived value failed, with the reason.
    pub failed: Vec<(BrickCoord, String)>,
    /// Work units the source did during this update.
    pub work_done: usize,
}

struct Entry {
    payload: Payload,
    stale: bool,
}

struct Inner {
    lifecycle: Lifecycle,
    centre: Option<BrickCoord>,
    active: BTreeSet<BrickCoord>,
    refresh: BTreeSet<BrickCoord>,
    cache: BTreeMap<BrickCoord, Entry>,
    failed: BTreeMap<BrickCoord, String>,
    upserts: BTreeSet<BrickCoord>,
    removes: BTreeSet<BrickCoord>,
}

/// The client's read-only view of the world: an active region and a cache of derived geometry.
pub struct ClientWorld {
    cfg: ClientConfig,
    source: Arc<dyn DerivedSource>,
    inner: Mutex<Inner>,
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

fn dist2(a: BrickCoord, b: BrickCoord) -> i128 {
    let d = |p: i64, q: i64| i128::from(p) - i128::from(q);
    d(a.x, b.x).pow(2) + d(a.y, b.y).pow(2) + d(a.z, b.z).pow(2)
}

impl ClientWorld {
    /// A client world in the `Created` state.
    pub fn new(cfg: ClientConfig, source: Arc<dyn DerivedSource>) -> Self {
        Self {
            cfg,
            source,
            inner: Mutex::new(Inner {
                lifecycle: Lifecycle::Created,
                centre: None,
                active: BTreeSet::new(),
                refresh: BTreeSet::new(),
                cache: BTreeMap::new(),
                failed: BTreeMap::new(),
                upserts: BTreeSet::new(),
                removes: BTreeSet::new(),
            }),
        }
    }

    /// The current lifecycle state.
    pub fn lifecycle(&self) -> Lifecycle {
        lock(&self.inner).lifecycle
    }

    /// `Created -> Running`.
    pub fn start(&self) -> Result<(), ClientError> {
        let mut g = lock(&self.inner);
        if g.lifecycle != Lifecycle::Created {
            return Err(ClientError::WrongState(g.lifecycle));
        }
        g.lifecycle = Lifecycle::Running;
        Ok(())
    }

    /// `Running -> Stopped`: the cache and the active region are dropped; the batch the
    /// adapter still has to apply removes everything that was shown.
    pub fn stop(&self) -> Result<RenderBatch, ClientError> {
        let mut g = lock(&self.inner);
        if g.lifecycle != Lifecycle::Running {
            return Err(ClientError::WrongState(g.lifecycle));
        }
        g.lifecycle = Lifecycle::Stopped;
        let shown: Vec<BrickCoord> = g.cache.keys().copied().collect();
        g.cache.clear();
        g.active.clear();
        g.refresh.clear();
        g.failed.clear();
        g.upserts.clear();
        g.removes.clear();
        Ok(RenderBatch {
            upsert: vec![],
            remove: shown.into_iter().filter_map(|b| self.key(b)).collect(),
        })
    }

    fn key(&self, b: BrickCoord) -> Option<CellKey> {
        CellKey::new(self.cfg.layer, self.cfg.level, b).ok()
    }

    fn target(&self, b: BrickCoord) -> Option<DerivedTarget> {
        Some(DerivedTarget {
            kind: self.cfg.kind,
            key: self.key(b)?,
        })
    }

    /// Sets the active region to the cube of bricks within `radius` of `centre` (a radius of 0 is
    /// the single brick). Returns which bricks entered and left.
    pub fn set_interest(
        &self,
        centre: BrickCoord,
        radius: u32,
    ) -> Result<InterestChange, ClientError> {
        let mut g = lock(&self.inner);
        if g.lifecycle != Lifecycle::Running {
            return Err(ClientError::WrongState(g.lifecycle));
        }
        let side = 2 * u64::from(radius) + 1;
        let requested = side.saturating_mul(side).saturating_mul(side);
        if requested > self.cfg.max_active_bricks {
            return Err(ClientError::InterestTooLarge {
                requested,
                max: self.cfg.max_active_bricks,
            });
        }
        let r = i64::from(radius);
        let mut wanted = BTreeSet::new();
        for dx in -r..=r {
            for dy in -r..=r {
                for dz in -r..=r {
                    let (Some(x), Some(y), Some(z)) = (
                        centre.x.checked_add(dx),
                        centre.y.checked_add(dy),
                        centre.z.checked_add(dz),
                    ) else {
                        continue; // beyond the end of the grid: nothing there
                    };
                    let b = BrickCoord::new(x, y, z);
                    if self.key(b).is_some() {
                        wanted.insert(b);
                    }
                }
            }
        }
        if wanted.is_empty() {
            return Err(ClientError::InvalidRegion);
        }
        let entered: Vec<BrickCoord> = wanted.difference(&g.active).copied().collect();
        let left: Vec<BrickCoord> = g.active.difference(&wanted).copied().collect();
        for b in &left {
            g.refresh.remove(b);
            g.failed.remove(b);
            g.upserts.remove(b);
            if g.cache.remove(b).is_some() {
                g.removes.insert(*b);
            }
        }
        for b in &entered {
            g.removes.remove(b);
        }
        g.active = wanted;
        g.centre = Some(centre);
        Ok(InterestChange { entered, left })
    }

    /// Learns that cells changed (call it with what the commit reported). Active bricks in
    /// the changed regions are refreshed; the source is told so that old results stop being
    /// current. Bricks outside the active region are ignored here (the source still hears it).
    pub fn on_dirty(&self, dirty: &[DirtyRegion]) {
        self.source.invalidate(dirty);
        let mut g = lock(&self.inner);
        if g.lifecycle != Lifecycle::Running {
            return;
        }
        for d in dirty
            .iter()
            .filter(|d| d.layer == self.cfg.layer && d.region.level() == self.cfg.level)
        {
            // A change of a brick can change the derived value of its neighbours too; which
            // ones is the provider's business (the source's invalidation says). Here every
            // active brick near the region is simply asked again, which is always correct.
            for b in d.region.bricks() {
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        for dz in -1..=1 {
                            let (Some(x), Some(y), Some(z)) = (
                                b.x.checked_add(dx),
                                b.y.checked_add(dy),
                                b.z.checked_add(dz),
                            ) else {
                                continue;
                            };
                            let n = BrickCoord::new(x, y, z);
                            if g.active.contains(&n) {
                                g.refresh.insert(n);
                                g.failed.remove(&n);
                            }
                        }
                    }
                }
            }
        }
    }

    /// One client frame: requests what is missing (nearest first), lets the source work within
    /// the budget, and collects what became current.
    pub fn update(&self) -> Result<UpdateReport, ClientError> {
        let (active, centre): (Vec<BrickCoord>, BrickCoord) = {
            let g = lock(&self.inner);
            if g.lifecycle != Lifecycle::Running {
                return Err(ClientError::WrongState(g.lifecycle));
            }
            let Some(c) = g.centre else {
                return Ok(UpdateReport::default());
            };
            (g.active.iter().copied().collect(), c)
        };
        let mut order = active;
        order.sort_by_key(|b| (dist2(*b, centre), *b));

        let mut report = UpdateReport::default();
        // 1. request what is not current in the cache (or was flagged) and not failed.
        for b in &order {
            let (need, failed) = {
                let g = lock(&self.inner);
                (
                    !g.cache.contains_key(b) || g.refresh.contains(b),
                    g.failed.contains_key(b),
                )
            };
            let target = self.target(*b).filter(|_| need && !failed);
            if let Some(Err(e)) = target.map(|t| self.source.request(t, PriorityClass::Visible)) {
                lock(&self.inner).failed.insert(*b, e);
            }
        }
        // 2. let the source work.
        report.work_done = self.source.drive(self.cfg.work_per_update);
        // 3. collect.
        let mut g = lock(&self.inner);
        for b in &order {
            let Some(t) = self.target(*b) else { continue };
            if let Some(why) = g.failed.get(b) {
                report.failed.push((*b, why.clone()));
                continue;
            }
            match self.source.poll(t) {
                Ok(Poll::Ready(p)) => {
                    let changed = g.cache.get(b).is_none_or(|e| !Arc::ptr_eq(&e.payload, &p));
                    if changed {
                        g.cache.insert(
                            *b,
                            Entry {
                                payload: p,
                                stale: false,
                            },
                        );
                        g.upserts.insert(*b);
                        g.removes.remove(b);
                    } else if let Some(e) = g.cache.get_mut(b) {
                        e.stale = false;
                    }
                    g.refresh.remove(b);
                    report.ready += 1;
                }
                Ok(Poll::Pending | Poll::Missing) => {
                    if let Some(e) = g.cache.get_mut(b) {
                        e.stale = true;
                        report.stale += 1;
                    } else {
                        report.pending += 1;
                    }
                }
                Err(e) => {
                    g.failed.insert(*b, e.clone());
                    report.failed.push((*b, e));
                }
            }
        }
        Ok(report)
    }

    /// What the adapter has to apply since the last call (and forgets it).
    pub fn take_batch(&self) -> RenderBatch {
        let mut g = lock(&self.inner);
        let centre = g.centre;
        let mut up: Vec<BrickCoord> = std::mem::take(&mut g.upserts).into_iter().collect();
        if let Some(c) = centre {
            up.sort_by_key(|b| (dist2(*b, c), *b));
        }
        let upsert = up
            .into_iter()
            .filter_map(|b| {
                let p = g.cache.get(&b)?.payload.clone();
                Some((self.key(b)?, p))
            })
            .collect();
        let remove = std::mem::take(&mut g.removes)
            .into_iter()
            .filter_map(|b| self.key(b))
            .collect();
        RenderBatch { upsert, remove }
    }

    /// Bricks of the active region.
    pub fn active(&self) -> Vec<BrickCoord> {
        lock(&self.inner).active.iter().copied().collect()
    }

    /// Bricks held in the cache.
    pub fn cached(&self) -> usize {
        lock(&self.inner).cache.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// A source that answers instantly, can be told to fail, and counts requests.
    #[derive(Default)]
    struct Mock {
        requests: AtomicUsize,
        failing: Mutex<BTreeSet<BrickCoord>>,
        values: Mutex<BTreeMap<BrickCoord, Payload>>,
        stale: Mutex<BTreeSet<BrickCoord>>,
    }

    impl DerivedSource for Mock {
        fn request(&self, t: DerivedTarget, _: PriorityClass) -> Result<(), String> {
            self.requests.fetch_add(1, Ordering::SeqCst);
            if lock(&self.failing).contains(&t.key.brick()) {
                return Err("provider failed".into());
            }
            let mut v = lock(&self.values);
            let b = t.key.brick();
            let n = v.len();
            v.entry(b).or_insert_with(|| Arc::new(n));
            Ok(())
        }
        fn poll(&self, t: DerivedTarget) -> Result<Poll, String> {
            let b = t.key.brick();
            if lock(&self.stale).contains(&b) {
                return Ok(Poll::Pending);
            }
            Ok(lock(&self.values)
                .get(&b)
                .map_or(Poll::Missing, |p| Poll::Ready(p.clone())))
        }
        fn invalidate(&self, dirty: &[DirtyRegion]) {
            for d in dirty {
                for b in d.region.bricks() {
                    lock(&self.values).remove(&b);
                }
            }
        }
        fn drive(&self, _: usize) -> usize {
            0
        }
    }

    fn cfg() -> ClientConfig {
        ClientConfig {
            layer: LayerId::new(1),
            level: 0,
            kind: DerivedKind(1),
            max_active_bricks: 125,
            work_per_update: 8,
        }
    }

    fn world() -> (ClientWorld, Arc<Mock>) {
        let m = Arc::new(Mock::default());
        (ClientWorld::new(cfg(), m.clone()), m)
    }

    #[test]
    fn lifecycle_is_enforced_and_a_stopped_client_hands_back_what_to_remove() {
        let (w, _) = world();
        assert_eq!(
            w.set_interest(BrickCoord::new(0, 0, 0), 1),
            Err(ClientError::WrongState(Lifecycle::Created))
        );
        w.start().unwrap();
        assert!(w.start().is_err());
        w.set_interest(BrickCoord::new(0, 0, 0), 1).unwrap();
        assert_eq!(w.update().unwrap().ready, 27);
        let shown = w.take_batch().upsert.len();
        assert_eq!(shown, 27);
        let stop = w.stop().unwrap();
        assert_eq!(stop.remove.len(), 27);
        assert_eq!(w.cached(), 0);
        assert!(w.update().is_err());
        assert!(w.stop().is_err());
    }

    #[test]
    fn the_active_region_streams_in_and_out_with_negative_coordinates() {
        let (w, m) = world();
        w.start().unwrap();
        let c = w.set_interest(BrickCoord::new(-1, 0, 0), 1).unwrap();
        assert_eq!((c.entered.len(), c.left.len()), (27, 0));
        w.update().unwrap();
        w.take_batch();
        // Move by one brick: 9 enter, 9 leave, 18 stay cached without new requests.
        let before = m.requests.load(Ordering::SeqCst);
        let c = w.set_interest(BrickCoord::new(0, 0, 0), 1).unwrap();
        assert_eq!((c.entered.len(), c.left.len()), (9, 9));
        let rep = w.update().unwrap();
        assert_eq!((rep.ready, rep.pending), (27, 0));
        assert_eq!(m.requests.load(Ordering::SeqCst) - before, 9);
        let b = w.take_batch();
        assert_eq!((b.upsert.len(), b.remove.len()), (9, 9));
        assert_eq!(w.cached(), 27);
        // Nearest first among the new bricks: the one straight ahead of the centre.
        assert_eq!(b.upsert[0].0.brick(), BrickCoord::new(1, 0, 0));
    }

    #[test]
    fn too_large_regions_are_refused_and_failures_are_reported_not_retried() {
        let (w, m) = world();
        w.start().unwrap();
        assert!(matches!(
            w.set_interest(BrickCoord::new(0, 0, 0), 3),
            Err(ClientError::InterestTooLarge {
                requested: 343,
                max: 125
            })
        ));
        lock(&m.failing).insert(BrickCoord::new(0, 0, 0));
        w.set_interest(BrickCoord::new(0, 0, 0), 1).unwrap();
        let rep = w.update().unwrap();
        assert_eq!((rep.ready, rep.failed.len()), (26, 1));
        let n = m.requests.load(Ordering::SeqCst);
        w.update().unwrap();
        assert_eq!(
            m.requests.load(Ordering::SeqCst),
            n,
            "a failed brick is not retried"
        );
    }

    #[test]
    fn a_dirty_brick_is_refreshed_and_shows_the_old_value_until_the_new_one_is_current() {
        let (w, m) = world();
        w.start().unwrap();
        let centre = BrickCoord::new(0, 0, 0);
        w.set_interest(centre, 0).unwrap();
        w.update().unwrap();
        let first = w.take_batch().upsert.remove(0).1;
        // The source says stale; the cache keeps the old value and reports it.
        lock(&m.stale).insert(centre);
        let region = cube_core::Region::new(
            0,
            cube_core::VoxelPos::new(0, 0, 0),
            cube_core::VoxelPos::new(16, 16, 16),
        )
        .unwrap();
        w.on_dirty(&[DirtyRegion {
            asset: cube_core::AssetId::new(1),
            layer: LayerId::new(1),
            region,
            change: cube_core::ChangeType::Geometry,
            source: cube_core::RevisionToken::new(
                cube_core::ResourceId::Asset(cube_core::AssetId::new(1)),
                cube_core::Epoch::INITIAL,
                cube_core::Revision::new(2),
            ),
            operation: cube_core::OperationId::new(1, 1),
            actor: cube_core::ActorKind::User,
        }]);
        let rep = w.update().unwrap();
        assert_eq!((rep.ready, rep.stale), (0, 1));
        assert!(w.take_batch().is_empty(), "nothing replaced yet");
        lock(&m.stale).clear();
        let rep = w.update().unwrap();
        assert_eq!(rep.ready, 1);
        let b = w.take_batch();
        assert_eq!(b.upsert.len(), 1);
        assert!(
            !Arc::ptr_eq(&b.upsert[0].1, &first),
            "the replacement is a new value"
        );
    }
}
