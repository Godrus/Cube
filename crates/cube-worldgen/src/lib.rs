//! Cube world generator, 11A (WGEN-01): a pure, deterministic, integer-only function from
//! `(WorldSeed, GeneratorVersion, brick coordinate)` to the untouched base content of a brick.
//!
//! * Pure: no I/O, no clock, no global state, no floats, no dependence on any other brick.
//!   The result of a brick is the same whether it is generated alone, in an area, in any order,
//!   on any thread. A voxel is a pure function of its world coordinates ([`voxel_at`]).
//! * Versioned: every released algorithm has an entry in [`registry`]; output that changes
//!   without a new version is a defect (pinned hashes guard it, see the determinism suite).
//! * No world-size limit below the documented coordinate range (+-2^60 voxels); negative
//!   coordinates use Euclidean division throughout.
//! * It writes nothing: [`GeneratedBrick::into_change`] only *describes* what a materialiser
//!   (WGEN-03, not implemented) would commit through the system-commit port, and that port never
//!   overwrites a loaded (possibly user-edited) cell.
//!
//! Version 1, "terrain-v1": a height field from three octaves of integer value noise; grass on
//! the surface, three layers of dirt, stone below; no caves, biomes, water or structures.
//! Version 2, "terrain-v2" (see [`v2`]): four biomes from two climate fields. Version 1 stays
//! in the registry unchanged so that worlds created with it keep working.
//!
//! Version 3, "terrain-v3" (see [`v3`]): version 2 plus caves and ores.
//!
//! Version 4, "terrain-v4" (see [`v4`]): version 3 plus trees.
//!
//! Version 5, "terrain-v5" (see [`v5`]): version 4 plus ruins.
//!
//! Determinism is argued from integer arithmetic only; cross-device identity is DG-2 and is
//! **not** claimed.

pub mod v1;
pub mod v2;
pub mod v3;
pub mod v4;
pub mod v5;

use cube_core::{
    BRICK_VOLUME, BrickCoord, CellChange, CellPayload, Channel, ChannelId, ChannelSet,
    GeneratorVersion, LocalPos, WorldSeed,
};

/// Largest coordinate magnitude (voxels) the generator accepts.
pub const COORD_LIMIT: i64 = 1 << 60;

/// Material indices of version 1 (raw palette indices; 0 is air).
pub mod materials {
    /// Stone.
    pub const STONE: u16 = 1;
    /// Dirt.
    pub const DIRT: u16 = 2;
    /// Grass.
    pub const GRASS: u16 = 3;
    /// Sand (version 2).
    pub const SAND: u16 = 4;
    /// Red brick (ruins, version 5; also the item of the same name).
    pub const BRICK: u16 = 5;
    /// Snow (version 2).
    pub const SNOW: u16 = 7;
    /// Tree trunk (reserved for a later version).
    pub const WOOD: u16 = 8;
    /// Tree leaves (reserved for a later version).
    pub const LEAVES: u16 = 9;
    /// Forest floor (version 2).
    pub const MOSS: u16 = 10;
    /// Coal ore (reserved for a later version).
    pub const COAL: u16 = 11;
    /// Iron ore (reserved for a later version).
    pub const IRON: u16 = 12;
}

/// Why a brick could not be generated (stable codes, WG-07).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum GenError {
    /// The generator version is not in the registry.
    GeneratorVersionUnknown(GeneratorVersion),
    /// A coordinate beyond [`COORD_LIMIT`] or a brick whose origin is not representable.
    InvalidCoord,
}

impl GenError {
    /// Stable numeric code (range 5000..5100 is WorldGen).
    pub const fn code(self) -> u16 {
        match self {
            Self::GeneratorVersionUnknown(_) => 5001,
            Self::InvalidCoord => 5002,
        }
    }
}

impl std::fmt::Display for GenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GeneratorVersionUnknown(v) => write!(f, "unknown generator version {}", v.raw()),
            Self::InvalidCoord => f.write_str("coordinate outside the generator range"),
        }
    }
}

impl std::error::Error for GenError {}

/// Status of a registered version (WG-02).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Runnable and the default for new worlds.
    Active,
    /// Runnable (worlds created with it keep working), no longer the default.
    Legacy,
}

/// A registry entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VersionInfo {
    /// The version.
    pub version: GeneratorVersion,
    /// Algorithm identifier.
    pub algorithm: &'static str,
    /// Status.
    pub status: Status,
}

/// All released generator versions, ascending.
pub fn registry() -> &'static [VersionInfo] {
    const R: [VersionInfo; 5] = [
        VersionInfo {
            version: GeneratorVersion::new(1),
            algorithm: "terrain-v1",
            status: Status::Legacy,
        },
        VersionInfo {
            version: GeneratorVersion::new(2),
            algorithm: "terrain-v2",
            status: Status::Legacy,
        },
        VersionInfo {
            version: GeneratorVersion::new(3),
            algorithm: "terrain-v3",
            status: Status::Legacy,
        },
        VersionInfo {
            version: GeneratorVersion::new(4),
            algorithm: "terrain-v4",
            status: Status::Legacy,
        },
        VersionInfo {
            version: GeneratorVersion::new(5),
            algorithm: "terrain-v5",
            status: Status::Active,
        },
    ];
    &R
}

/// The version new worlds use.
pub const fn current_version() -> GeneratorVersion {
    GeneratorVersion::new(5)
}

/// The ruin of the 40 x 40 column region `(rx, rz)` in the world of `version` (only version 5
/// has places of interest).
pub fn ruin_of_region(
    seed: WorldSeed,
    version: GeneratorVersion,
    rx: i64,
    rz: i64,
) -> Result<Option<v5::Ruin>, GenError> {
    check(version)?;
    Ok(if version.raw() >= 5 {
        v5::ruin_of_region(seed.raw(), rx, rz)
    } else {
        None
    })
}

/// Ground height of the column `(x, z)` in the world of `version`.
pub fn surface_height(
    seed: WorldSeed,
    version: GeneratorVersion,
    x: i64,
    z: i64,
) -> Result<i64, GenError> {
    check(version)?;
    Ok(match version.raw() {
        1 => v1::height(seed, x, z),
        _ => v2::height(seed, x, z), // versions 2 and 3 share the surface
    })
}

/// The biome of the column `(x, z)` in the world of `version` (version 1 has one: plains).
pub fn biome_at(
    seed: WorldSeed,
    version: GeneratorVersion,
    x: i64,
    z: i64,
) -> Result<v2::Biome, GenError> {
    check(version)?;
    Ok(match version.raw() {
        1 => v2::Biome::Plains,
        _ => v2::biome(seed, x, z),
    })
}

/// The content of one generated brick. The three shapes exist so that a mostly empty or mostly
/// solid sky/underground costs nothing; they are an encoding of one value (see
/// [`GeneratedBrick::materials`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrickContent {
    /// No solid voxel at all.
    Empty,
    /// Every voxel has this material.
    Uniform(u16),
    /// One material per voxel in `LocalPos::all()` order (x fastest).
    Dense(Box<[u16; BRICK_VOLUME]>),
}

/// One generated brick.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedBrick {
    /// Which brick.
    pub coord: BrickCoord,
    /// Its content.
    pub content: BrickContent,
}

impl GeneratedBrick {
    /// All 4096 materials, whatever the encoding (the canonical value of the brick).
    pub fn materials(&self) -> Vec<u16> {
        match &self.content {
            BrickContent::Empty => vec![0; BRICK_VOLUME],
            BrickContent::Uniform(m) => vec![*m; BRICK_VOLUME],
            BrickContent::Dense(d) => d.to_vec(),
        }
    }

    /// What a materialiser would commit for this brick: `KnownEmpty` for an empty brick,
    /// otherwise a material channel. Describes, never commits.
    pub fn into_change(self) -> CellChange {
        match self.content {
            BrickContent::Empty => CellChange::KnownEmpty,
            BrickContent::Uniform(m) => CellChange::Put(CellPayload::Channels(
                ChannelSet::new().with(ChannelId::Material, Channel::constant(m)),
            )),
            BrickContent::Dense(d) => {
                let mut ch = Channel::constant(d[0]);
                for (l, v) in LocalPos::all().zip(d.iter()) {
                    if *v != d[0] {
                        ch.set(l, *v);
                    }
                }
                ch.compact();
                CellChange::Put(CellPayload::Channels(
                    ChannelSet::new().with(ChannelId::Material, ch),
                ))
            }
        }
    }

    /// The content hash of this brick (see [`chunk_hash`]).
    pub fn hash(&self, seed: WorldSeed, version: GeneratorVersion) -> u64 {
        chunk_hash(seed, version, self.coord, &self.materials())
    }
}

/// FNV-1a over `(seed, version, coord, expanded materials)`, fixed little-endian byte order.
/// 64 bits, not collision resistant (WG-06 leaves the final digest open).
pub fn chunk_hash(
    seed: WorldSeed,
    version: GeneratorVersion,
    coord: BrickCoord,
    materials: &[u16],
) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    let mut put = |bytes: &[u8]| {
        for b in bytes {
            h ^= u64::from(*b);
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
    };
    put(&seed.to_le_bytes());
    put(&version.to_le_bytes());
    put(&coord.x.to_le_bytes());
    put(&coord.y.to_le_bytes());
    put(&coord.z.to_le_bytes());
    for m in materials {
        put(&m.to_le_bytes());
    }
    h
}

fn check(version: GeneratorVersion) -> Result<(), GenError> {
    registry()
        .iter()
        .any(|i| i.version == version)
        .then_some(())
        .ok_or(GenError::GeneratorVersionUnknown(version))
}

/// The material of one voxel at world coordinates (0 = air): the definition of the world.
pub fn voxel_at(
    seed: WorldSeed,
    version: GeneratorVersion,
    x: i64,
    y: i64,
    z: i64,
) -> Result<u16, GenError> {
    check(version)?;
    if [x, y, z].iter().any(|c| c.abs() > COORD_LIMIT) {
        return Err(GenError::InvalidCoord);
    }
    Ok(match version.raw() {
        1 => v1::material(seed, x, y, z),
        2 => v2::material(seed, x, y, z),
        3 => v3::material(seed, x, y, z),
        4 => v4::material(seed, x, y, z),
        _ => v5::material(seed, x, y, z),
    })
}

/// Generates one brick of the untouched base world.
pub fn generate_brick(
    seed: WorldSeed,
    version: GeneratorVersion,
    brick: BrickCoord,
) -> Result<GeneratedBrick, GenError> {
    check(version)?;
    let origin = brick.origin().map_err(|_| GenError::InvalidCoord)?;
    if [origin.x, origin.y, origin.z]
        .iter()
        .any(|c| c.abs() > COORD_LIMIT)
    {
        return Err(GenError::InvalidCoord);
    }
    let content = match version.raw() {
        1 => v1::brick(seed, origin.x, origin.y, origin.z),
        2 => v2::brick(seed, origin.x, origin.y, origin.z),
        3 => v3::brick(seed, origin.x, origin.y, origin.z),
        4 => v4::brick(seed, origin.x, origin.y, origin.z),
        _ => v5::brick(seed, origin.x, origin.y, origin.z),
    };
    Ok(GeneratedBrick {
        coord: brick,
        content,
    })
}
