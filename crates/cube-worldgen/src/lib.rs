//! Cube world generation (experimental spike). A user of Geometry Core, not a change to any contract.
//!
//! Pure Rust, no engine dependency, depends on `cube-core` only. Synchronous generation; no scheduler.
//! See `docs/spike/rust-world-generation/README.md`.

pub mod generator;
pub mod noise;
pub mod state_v0;
pub mod streamer;

pub use generator::{
    BiomeId, Chunk, ChunkCoord, ChunkGenerator, ChunkState, GenError, TerrainGenerator, WorldSeed,
    BRICKS_PER_CHUNK, CHUNK_HEIGHT, CHUNK_SIZE, MAT_AIR, MAT_DIRT, MAT_GRASS, MAT_PLACED,
    MAT_STONE, MAX_CHUNK_ABS,
};
pub use state_v0::{LoadReport, StateError};
pub use streamer::{
    PumpSummary, StateCounts, StreamConfig, StreamDelta, UpdateSummary, VoxelEdit, WorldError,
    WorldStreamer,
};
