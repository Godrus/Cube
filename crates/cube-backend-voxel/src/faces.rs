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

use cube_core::{
    BRICK_EDGE, BrickCoord, CellKey, CellPayload, CellState, ChannelId, LocalPos, VoxelPos,
};
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

/// State of one of the 27 bricks around the centre brick in a [`Halo`].
enum Nb {
    /// Loaded (or known empty): its voxels are in the array.
    Data,
    /// Not loaded or outside the bounds: counts as air and sets `unloaded_neighbours`.
    Unloaded,
    /// Beyond the end of the i64 grid: air, no flag (the same as a step past the grid).
    Absent,
    /// Failed to load or unreadable: a failure, raised only when a face decision reads it.
    Failed(String),
}

/// The temporary halo of one brick: the material of the 18^3 voxels around and inside it, copied
/// once from the 27 bricks (DEC-095). Wide materials are kept as `u32` and checked when read, so
/// the failure for a material above 65535 is raised exactly when the per-voxel read raised it.
struct Halo {
    m: Vec<u32>,
    nb: Vec<Nb>,
}

const H: usize = BRICK_EDGE as usize + 2;

impl Halo {
    fn build(snapshot: &Snapshot, centre: BrickCoord) -> Result<Self, String> {
        let mut m = vec![0u32; H * H * H];
        let mut nb = Vec::with_capacity(27);
        for dz in -1i64..=1 {
            for dy in -1i64..=1 {
                for dx in -1i64..=1 {
                    let (Some(bx), Some(by), Some(bz)) = (
                        centre.x.checked_add(dx),
                        centre.y.checked_add(dy),
                        centre.z.checked_add(dz),
                    ) else {
                        nb.push(Nb::Absent);
                        continue;
                    };
                    let b = BrickCoord::new(bx, by, bz);
                    let state = match snapshot.cell(b) {
                        Ok(s) => s,
                        Err(e) => {
                            nb.push(Nb::Failed(format!("snapshot read: {e}")));
                            continue;
                        }
                    };
                    match state {
                        CellState::Present(pl) => {
                            nb.push(Nb::Data);
                            let CellPayload::Channels(set) = &**pl else {
                                continue;
                            };
                            let Some(ch) = set.channel(ChannelId::Material) else {
                                continue;
                            };
                            // The local voxels of this brick that fall in -1..=16 of the centre.
                            let range = |d: i64| match d {
                                -1 => (BRICK_EDGE - 1, BRICK_EDGE - 1),
                                0 => (0, BRICK_EDGE - 1),
                                _ => (0, 0),
                            };
                            let ((x0, x1), (y0, y1), (z0, z1)) = (range(dx), range(dy), range(dz));
                            for z in z0..=z1 {
                                for y in y0..=y1 {
                                    for x in x0..=x1 {
                                        let Ok(l) = LocalPos::new(x as u8, y as u8, z as u8) else {
                                            continue;
                                        };
                                        let gx = (dx * BRICK_EDGE + x + 1) as usize;
                                        let gy = (dy * BRICK_EDGE + y + 1) as usize;
                                        let gz = (dz * BRICK_EDGE + z + 1) as usize;
                                        m[gx + H * (gy + H * gz)] = ch.get(l);
                                    }
                                }
                            }
                        }
                        CellState::KnownEmpty => nb.push(Nb::Data),
                        CellState::NotLoaded | CellState::OutsideBounds => nb.push(Nb::Unloaded),
                        CellState::LoadFailure(r) => {
                            nb.push(Nb::Failed(format!("brick {b:?} failed to load ({r:?})")));
                        }
                    }
                }
            }
        }
        Ok(Self { m, nb })
    }

    /// Material at halo coordinates (0..=17 per axis, the brick itself is 1..=16).
    fn material(&self, h: [usize; 3], unloaded: &mut bool) -> Result<u16, String> {
        let side = |v: usize| {
            if v == 0 {
                0
            } else if v == H - 1 {
                2
            } else {
                1
            }
        };
        let k = side(h[0]) + 3 * side(h[1]) + 9 * side(h[2]);
        match &self.nb[k] {
            Nb::Data => {}
            Nb::Unloaded => {
                *unloaded = true;
                return Ok(0);
            }
            Nb::Absent => return Ok(0),
            Nb::Failed(e) => return Err(e.clone()),
        }
        u16::try_from(self.m[h[0] + H * (h[1] + H * h[2])])
            .map_err(|_| "material index above 65535: mesh format is 16-bit (MAT-02)".to_owned())
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
        // DEC-095: the neighbour voxels come from one temporary halo array (18^3), copied once
        // from the 27 bricks, not from a snapshot read per voxel (16.5 times faster on the M1-A3b
        // measurement, same faces). The array is dropped when the mesh is built; nothing about the
        // brick storage changes.
        let halo = Halo::build(ctx.snapshot, centre).map_err(ProviderFailure::Failed)?;
        let mut unloaded = false;
        let mut faces = Vec::new();
        for z in 0..BRICK_EDGE {
            ctx.token.check()?;
            for y in 0..BRICK_EDGE {
                for x in 0..BRICK_EDGE {
                    let p = VoxelPos::new(origin.x + x, origin.y + y, origin.z + z);
                    let h = [(x + 1) as usize, (y + 1) as usize, (z + 1) as usize];
                    let m = halo
                        .material(h, &mut unloaded)
                        .map_err(ProviderFailure::Failed)?;
                    if m == 0 {
                        continue;
                    }
                    for d in Dir::ALL {
                        // A neighbour beyond the end of the i64 grid does not exist: air.
                        let n = match step(p, d) {
                            Some(_) => {
                                let st = d.step();
                                let q = [
                                    (h[0] as i64 + st[0]) as usize,
                                    (h[1] as i64 + st[1]) as usize,
                                    (h[2] as i64 + st[2]) as usize,
                                ];
                                halo.material(q, &mut unloaded)
                                    .map_err(ProviderFailure::Failed)?
                            }
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
            unloaded_neighbours: unloaded,
        }))
    }
}
