//! Core types and the single terrain generator (one biome, height-map terrain).
//!
//! Coordinates: world voxel `(x, y, z)` with `y` up, 1 voxel = 1 world unit. A chunk is a column of
//! `16 x 32 x 16` voxels = two stacked 16^3 bricks (exactly the Geometry Core brick size).
//! `ChunkCoord{x, z}` = `floor(world / 16)` on each horizontal axis (negative coordinates floor);
//! chunk `(cx, cz)` stores its bricks at brick coordinates `(cx, 0, cz)` and `(cx, 1, cz)`.

use cube_core::{Brick, LocalPos};

use crate::noise::value_noise;

pub const CHUNK_SIZE: i32 = 16;
pub const CHUNK_HEIGHT: i32 = 32;
pub const BRICKS_PER_CHUNK: usize = 2;
/// Chunk coordinates beyond this limit are refused (keeps world coordinates inside `i32`).
pub const MAX_CHUNK_ABS: i32 = 1 << 20;

pub const MAT_AIR: u16 = 0;
pub const MAT_GRASS: u16 = 1;
pub const MAT_DIRT: u16 = 2;
pub const MAT_STONE: u16 = 3;
/// Material written by player edits in the demo client.
pub const MAT_PLACED: u16 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WorldSeed(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ChunkCoord {
    pub x: i32,
    pub z: i32,
}

impl ChunkCoord {
    pub const fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    /// Chunk containing a world column (floor division, correct for negatives).
    pub fn from_world(wx: i32, wz: i32) -> Self {
        Self {
            x: wx.div_euclid(CHUNK_SIZE),
            z: wz.div_euclid(CHUNK_SIZE),
        }
    }

    /// Chebyshev distance in chunks.
    pub fn distance(&self, o: ChunkCoord) -> i32 {
        (self.x - o.x).abs().max((self.z - o.z).abs())
    }

    pub fn origin_x(&self) -> i32 {
        self.x * CHUNK_SIZE
    }

    pub fn origin_z(&self) -> i32 {
        self.z * CHUNK_SIZE
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BiomeId(pub u16);

impl BiomeId {
    pub const PLAINS: BiomeId = BiomeId(1);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChunkState {
    Unloaded,
    Queued,
    Generating,
    Ready,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GenError {
    /// Chunk coordinate beyond `MAX_CHUNK_ABS`.
    OutOfWorld(ChunkCoord),
    /// Raised by test generators.
    Injected(&'static str),
    /// Writing the generated bricks into Geometry Core failed.
    Commit(String),
}

impl GenError {
    pub fn code(&self) -> &'static str {
        match self {
            GenError::OutOfWorld(_) => "WGEN-0001",
            GenError::Injected(_) => "WGEN-0002",
            GenError::Commit(_) => "WGEN-0003",
        }
    }
}

impl std::fmt::Display for GenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GenError::OutOfWorld(c) => write!(
                f,
                "{}: chunk ({}, {}) is outside the world limit",
                self.code(),
                c.x,
                c.z
            ),
            GenError::Injected(m) => write!(f, "{}: {m}", self.code()),
            GenError::Commit(m) => write!(f, "{}: commit failed: {m}", self.code()),
        }
    }
}

impl std::error::Error for GenError {}

/// Generated chunk data (pure value; independent of any streamer state).
#[derive(Clone, Debug)]
pub struct Chunk {
    pub coord: ChunkCoord,
    pub biome: BiomeId,
    /// Bottom brick, top brick. `None` = entirely air (not stored in Geometry Core).
    pub bricks: [Option<Brick>; BRICKS_PER_CHUNK],
}

impl Chunk {
    pub fn voxel_count(&self) -> usize {
        self.bricks
            .iter()
            .flatten()
            .map(|b| b.occupied().len())
            .sum()
    }

    /// Material at chunk-local `(x, y, z)`, `x,z in 0..16`, `y in 0..32`.
    pub fn material_at(&self, x: u8, y: u8, z: u8) -> u16 {
        let (bi, ly) = ((y / 16) as usize, y % 16);
        match &self.bricks[bi] {
            Some(b) => b.get(LocalPos { x, y: ly, z }),
            None => MAT_AIR,
        }
    }

    /// FNV-1a over all occupied voxels in a fixed order; equal content gives an equal hash.
    pub fn content_hash(&self) -> u64 {
        let mut h = 0xCBF2_9CE4_8422_2325u64;
        let mut feed = |v: u64| {
            h ^= v;
            h = h.wrapping_mul(0x0000_0100_0000_01B3);
        };
        for (bi, b) in self.bricks.iter().enumerate() {
            if let Some(b) = b {
                let mut cells = b.occupied();
                cells.sort_by_key(|(l, _)| (l.z, l.y, l.x));
                for (l, m) in cells {
                    feed(
                        ((bi as u64) << 40)
                            | ((l.x as u64) << 32)
                            | ((l.y as u64) << 24)
                            | ((l.z as u64) << 16)
                            | m as u64,
                    );
                }
            }
        }
        h
    }
}

impl PartialEq for Chunk {
    fn eq(&self, o: &Chunk) -> bool {
        self.coord == o.coord
            && self.biome == o.biome
            && self
                .bricks
                .iter()
                .zip(o.bricks.iter())
                .all(|(a, b)| match (a, b) {
                    (Some(a), Some(b)) => a.same_content(b),
                    (None, None) => true,
                    (Some(a), None) | (None, Some(a)) => a.is_empty(),
                })
    }
}

pub trait ChunkGenerator {
    fn generate(&self, seed: WorldSeed, coord: ChunkCoord) -> Result<Chunk, GenError>;
}

/// One biome (plains): rolling hills from two octaves of integer value noise.
#[derive(Clone, Copy, Debug, Default)]
pub struct TerrainGenerator;

const BASE_HEIGHT: i32 = 6;

impl TerrainGenerator {
    /// Highest solid voxel `y` of a world column. Pure function of `(seed, x, z)`; range `6..=19`.
    pub fn height_at(seed: WorldSeed, wx: i32, wz: i32) -> i32 {
        let s = seed.0;
        let n1 = value_noise(s, 1, wx, wz, 32) as i64;
        let n2 = value_noise(s, 2, wx, wz, 16) as i64;
        BASE_HEIGHT + ((n1 * 10 + n2 * 4) / 255) as i32
    }

    /// Material of a voxel given the column height `h` (grass on top, 3 dirt layers, stone below).
    pub fn material_for(y: i32, h: i32) -> u16 {
        if y > h {
            MAT_AIR
        } else if y == h {
            MAT_GRASS
        } else if y >= h - 3 {
            MAT_DIRT
        } else {
            MAT_STONE
        }
    }
}

impl ChunkGenerator for TerrainGenerator {
    fn generate(&self, seed: WorldSeed, coord: ChunkCoord) -> Result<Chunk, GenError> {
        if coord.x.abs() > MAX_CHUNK_ABS || coord.z.abs() > MAX_CHUNK_ABS {
            return Err(GenError::OutOfWorld(coord));
        }
        let mut heights = [0i32; 256];
        for lz in 0..CHUNK_SIZE {
            for lx in 0..CHUNK_SIZE {
                heights[(lz * CHUNK_SIZE + lx) as usize] =
                    Self::height_at(seed, coord.origin_x() + lx, coord.origin_z() + lz);
            }
        }
        let mut bricks: [Option<Brick>; BRICKS_PER_CHUNK] = [None, None];
        for (bi, slot) in bricks.iter_mut().enumerate() {
            let mut brick = Brick::constant(0);
            for lz in 0..CHUNK_SIZE as u8 {
                for lx in 0..CHUNK_SIZE as u8 {
                    let h = heights[lz as usize * 16 + lx as usize];
                    for ly in 0..CHUNK_SIZE as u8 {
                        let y = bi as i32 * CHUNK_SIZE + ly as i32;
                        let m = Self::material_for(y, h);
                        if m != MAT_AIR {
                            brick.set(
                                LocalPos {
                                    x: lx,
                                    y: ly,
                                    z: lz,
                                },
                                m,
                            );
                        }
                    }
                }
            }
            brick.normalize();
            if !brick.is_empty() {
                *slot = Some(brick);
            }
        }
        Ok(Chunk {
            coord,
            biome: BiomeId::PLAINS,
            bricks,
        })
    }
}
