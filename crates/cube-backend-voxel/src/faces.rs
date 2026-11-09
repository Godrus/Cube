//! Faces provider, the reference oracle of voxel meshing (VOX-02; provider ownership pending O-1,
//! so this is provisional like every provider of PS-5).
//!
//! A face is the side of a solid voxel (material != 0) whose neighbour across that side is air.
//! Faces between two solid voxels are hidden whatever their materials. The provider computes the
//! faces of the voxels of ONE brick from a Processing snapshot with an apron of one brick, so
//! faces on a brick border are decided with the real neighbour voxel: there are no gaps and no
//! duplicated faces at brick borders.
//!
//! Absent data is explicit: a neighbouring brick that is `NotLoaded` or `OutsideBounds` counts
//! as air (the face is emitted) and the result says so (`unloaded_neighbours`); the face set
//! changes when that brick loads, and the commit that loads it invalidates this brick. A brick
//! that failed to load, or the centre brick not being loaded, is a failure, never an empty mesh.
//!
//! Output order is fixed (z, y, x, direction), so results are deterministic.

use std::sync::Arc;

use cube_core::{BRICK_EDGE, BrickCoord, CellKey, CellPayload, CellState, ChannelId, VoxelPos};
use cube_processing::{
    ChangeMask, ComputeContext, DerivedKind, Payload, Provider, ProviderFailure, Snapshot,
};

/// Derived kind of the faces mesh. The voxel backend owns ids `0xB100_0000..0xB101_0000`.
pub const FACES_KIND: DerivedKind = DerivedKind(0xB100_0001);

/// Side of a voxel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Dir {
    /// +x
    PosX,
    /// -x
    NegX,
    /// +y
    PosY,
    /// -y
    NegY,
    /// +z
    PosZ,
    /// -z
    NegZ,
}

impl Dir {
    /// All six, in the fixed output order.
    pub const ALL: [Dir; 6] = [
        Dir::PosX,
        Dir::NegX,
        Dir::PosY,
        Dir::NegY,
        Dir::PosZ,
        Dir::NegZ,
    ];

    /// Unit step of the direction.
    pub const fn step(self) -> [i64; 3] {
        match self {
            Dir::PosX => [1, 0, 0],
            Dir::NegX => [-1, 0, 0],
            Dir::PosY => [0, 1, 0],
            Dir::NegY => [0, -1, 0],
            Dir::PosZ => [0, 0, 1],
            Dir::NegZ => [0, 0, -1],
        }
    }

    /// Axis index 0..3 of the direction's normal.
    pub const fn axis(self) -> usize {
        match self {
            Dir::PosX | Dir::NegX => 0,
            Dir::PosY | Dir::NegY => 1,
            Dir::PosZ | Dir::NegZ => 2,
        }
    }
}

/// One visible unit face.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Face {
    /// The voxel the face belongs to.
    pub voxel: VoxelPos,
    /// Which side.
    pub dir: Dir,
    /// Material of the voxel (never 0).
    pub material: u16,
}

/// The faces of one brick.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FacesMesh {
    /// The brick the mesh belongs to.
    pub key: CellKey,
    /// Visible faces in the fixed order.
    pub faces: Vec<Face>,
    /// `true` if some neighbouring brick was not loaded (or outside the bounds) and was treated
    /// as air.
    pub unloaded_neighbours: bool,
}

/// Samples voxels of the centre brick and its neighbours from a snapshot.
pub(crate) struct Sampler<'a> {
    snapshot: &'a Snapshot,
    pub(crate) unloaded: bool,
}

impl<'a> Sampler<'a> {
    pub(crate) fn new(snapshot: &'a Snapshot) -> Self {
        Self {
            snapshot,
            unloaded: false,
        }
    }

    /// Material at voxel `p`; `Err` only for a brick that failed to load.
    pub(crate) fn material(&mut self, p: VoxelPos, centre: BrickCoord) -> Result<u16, String> {
        let (b, l) = p.split();
        let state = self
            .snapshot
            .cell(b)
            .map_err(|e| format!("snapshot read: {e}"))?;
        match state {
            CellState::Present(pl) => Ok(match &**pl {
                CellPayload::Channels(set) => {
                    set.channel(ChannelId::Material).map_or(0, |c| c.get(l))
                }
                CellPayload::MeshPage(_) => 0,
            }),
            CellState::KnownEmpty => Ok(0),
            CellState::NotLoaded | CellState::OutsideBounds => {
                if b != centre {
                    self.unloaded = true;
                }
                Ok(0)
            }
            CellState::LoadFailure(r) => Err(format!("brick {b:?} failed to load ({r:?})")),
        }
    }
}

/// The faces provider.
#[derive(Debug, Default, Clone, Copy)]
pub struct FacesProvider;

fn step(p: VoxelPos, d: Dir) -> Option<VoxelPos> {
    let s = d.step();
    Some(VoxelPos::new(
        p.x.checked_add(s[0])?,
        p.y.checked_add(s[1])?,
        p.z.checked_add(s[2])?,
    ))
}

impl Provider for FacesProvider {
    fn kind(&self) -> DerivedKind {
        FACES_KIND
    }
    fn name(&self) -> &str {
        "voxel-faces"
    }
    fn version(&self) -> u32 {
        1
    }
    fn consumes(&self) -> ChangeMask {
        ChangeMask::GEOMETRY.or(ChangeMask::MATERIAL)
    }
    fn apron(&self) -> u32 {
        1
    }
    fn compute(&self, ctx: &ComputeContext<'_>) -> Result<Payload, ProviderFailure> {
        let key = ctx.target.key;
        let centre = key.brick();
        let state = ctx
            .snapshot
            .cell(centre)
            .map_err(|e| ProviderFailure::Failed(e.to_string()))?;
        match state {
            CellState::NotLoaded => {
                return Err(ProviderFailure::Failed("brick is not loaded".to_owned()));
            }
            CellState::LoadFailure(_) => {
                return Err(ProviderFailure::Failed("brick failed to load".to_owned()));
            }
            CellState::OutsideBounds | CellState::KnownEmpty => {
                return Ok(Arc::new(FacesMesh {
                    key,
                    faces: vec![],
                    unloaded_neighbours: false,
                }));
            }
            CellState::Present(_) => {}
        }
        let origin = centre
            .origin()
            .map_err(|e| ProviderFailure::Failed(e.to_string()))?;
        let mut s = Sampler::new(ctx.snapshot);
        let mut faces = Vec::new();
        for z in 0..BRICK_EDGE {
            ctx.token.check()?;
            for y in 0..BRICK_EDGE {
                for x in 0..BRICK_EDGE {
                    let p = VoxelPos::new(origin.x + x, origin.y + y, origin.z + z);
                    let m = s.material(p, centre).map_err(ProviderFailure::Failed)?;
                    if m == 0 {
                        continue;
                    }
                    for d in Dir::ALL {
                        // A neighbour beyond the end of the i64 grid does not exist: air.
                        let n = match step(p, d) {
                            Some(q) => s.material(q, centre).map_err(ProviderFailure::Failed)?,
                            None => 0,
                        };
                        if n == 0 {
                            faces.push(Face {
                                voxel: p,
                                dir: d,
                                material: m,
                            });
                        }
                    }
                }
            }
        }
        Ok(Arc::new(FacesMesh {
            key,
            faces,
            unloaded_neighbours: s.unloaded,
        }))
    }
}
