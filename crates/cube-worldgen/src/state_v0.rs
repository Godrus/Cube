//! `world-state-v0`: EXPERIMENTAL, TEMPORARY save format for the world-generation spike.
//!
//! Not the approved persistence format (`cube-persistence`), not versioned for compatibility, not for
//! production. It stores only what is needed to rebuild the streamed world: seed, streaming config,
//! centre, the list of Ready chunks, player edits per chunk, and the asset epoch/revision (informational).
//! Terrain is not stored: it is regenerated from the seed, which is why determinism is a hard rule.
//!
//! Layout (little endian): magic `CUBEWST0`, u32 version (0), u64 seed, i32 load_radius, i32 unload_margin,
//! u32 max_queue, u32 asset_epoch, u64 asset_revision, u8 has_center, i32 cx, i32 cz,
//! u32 n_active, n_active x (i32 x, i32 z), u32 n_edit_chunks, each: i32 x, i32 z, u32 n, n x (i32 x, i32 y, i32 z, u16 m),
//! trailing u32 CRC-32 of everything before it.

use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use cube_core::VoxelPos;

use crate::generator::{ChunkCoord, ChunkGenerator, WorldSeed, CHUNK_SIZE};
use crate::streamer::{StreamConfig, VoxelEdit, WorldStreamer};

const MAGIC: &[u8; 8] = b"CUBEWST0";
pub const FORMAT_VERSION: u32 = 0;

#[derive(Debug)]
pub enum StateError {
    Io(std::io::Error),
    Corrupt(&'static str),
    Checksum,
    UnsupportedVersion(u32),
}

impl std::fmt::Display for StateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StateError::Io(e) => write!(f, "WSTATE-0001: io: {e}"),
            StateError::Corrupt(m) => write!(f, "WSTATE-0002: corrupt: {m}"),
            StateError::Checksum => write!(f, "WSTATE-0003: checksum mismatch"),
            StateError::UnsupportedVersion(v) => {
                write!(f, "WSTATE-0004: unsupported world-state version {v}")
            }
        }
    }
}

impl std::error::Error for StateError {}

impl From<std::io::Error> for StateError {
    fn from(e: std::io::Error) -> Self {
        StateError::Io(e)
    }
}

/// Result of restoring a saved world.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadReport {
    pub saved_active: usize,
    pub restored_active: usize,
    /// Saved Ready chunks that did not come back as Ready (generation failed or out of range).
    pub missing: Vec<ChunkCoord>,
    pub edits_restored: usize,
}

pub fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            let mask = (!(crc & 1)).wrapping_add(1);
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

pub fn encode(w: &WorldStreamer) -> Vec<u8> {
    let mut b: Vec<u8> = Vec::new();
    b.extend_from_slice(MAGIC);
    b.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    b.extend_from_slice(&w.seed().0.to_le_bytes());
    let cfg = w.config();
    b.extend_from_slice(&cfg.load_radius.to_le_bytes());
    b.extend_from_slice(&cfg.unload_margin.to_le_bytes());
    b.extend_from_slice(&(cfg.max_queue as u32).to_le_bytes());
    b.extend_from_slice(&w.asset().epoch().to_le_bytes());
    b.extend_from_slice(&w.asset().asset_token().value.to_le_bytes());
    match w.center() {
        Some(c) => {
            b.push(1);
            b.extend_from_slice(&c.x.to_le_bytes());
            b.extend_from_slice(&c.z.to_le_bytes());
        }
        None => {
            b.push(0);
            b.extend_from_slice(&[0u8; 8]);
        }
    }
    let active = w.active_chunks();
    b.extend_from_slice(&(active.len() as u32).to_le_bytes());
    for c in &active {
        b.extend_from_slice(&c.x.to_le_bytes());
        b.extend_from_slice(&c.z.to_le_bytes());
    }
    let mut chunks: Vec<_> = w.edits.iter().filter(|(_, v)| !v.is_empty()).collect();
    chunks.sort_by_key(|(c, _)| **c);
    b.extend_from_slice(&(chunks.len() as u32).to_le_bytes());
    for (c, list) in chunks {
        b.extend_from_slice(&c.x.to_le_bytes());
        b.extend_from_slice(&c.z.to_le_bytes());
        b.extend_from_slice(&(list.len() as u32).to_le_bytes());
        for e in list {
            b.extend_from_slice(&e.pos.x.to_le_bytes());
            b.extend_from_slice(&e.pos.y.to_le_bytes());
            b.extend_from_slice(&e.pos.z.to_le_bytes());
            b.extend_from_slice(&e.material.to_le_bytes());
        }
    }
    let crc = crc32(&b);
    b.extend_from_slice(&crc.to_le_bytes());
    b
}

struct Reader<'a> {
    b: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], StateError> {
        if self.b.len() - self.pos < n {
            return Err(StateError::Corrupt("unexpected end of data"));
        }
        let s = &self.b[self.pos..self.pos + n];
        self.pos += n;
        Ok(s)
    }
    fn u8(&mut self) -> Result<u8, StateError> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, StateError> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }
    fn u32(&mut self) -> Result<u32, StateError> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn i32(&mut self) -> Result<i32, StateError> {
        Ok(i32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn u64(&mut self) -> Result<u64, StateError> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
}

pub fn save(w: &WorldStreamer, path: &Path) -> Result<(), StateError> {
    let bytes = encode(w);
    let mut name = path
        .file_name()
        .map(|n| n.to_os_string())
        .unwrap_or_default();
    name.push(".tmp");
    let tmp: PathBuf = path.with_file_name(name);
    let result = (|| {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(&bytes)?;
        f.sync_all()?;
        drop(f);
        fs::rename(&tmp, path)
    })();
    if let Err(e) = result {
        let _ = fs::remove_file(&tmp);
        return Err(e.into());
    }
    Ok(())
}

/// Rebuilds a streamer from a saved state: same seed and config, edits restored, asset in a new epoch,
/// chunks regenerated around the saved centre. Terrain comes from the generator, not from the file.
pub fn load(
    path: &Path,
    generator: Box<dyn ChunkGenerator>,
) -> Result<(WorldStreamer, LoadReport), StateError> {
    let mut bytes = Vec::new();
    fs::File::open(path)?.read_to_end(&mut bytes)?;
    decode(&bytes, generator)
}

pub fn decode(
    bytes: &[u8],
    generator: Box<dyn ChunkGenerator>,
) -> Result<(WorldStreamer, LoadReport), StateError> {
    if bytes.len() < MAGIC.len() + 8 || &bytes[..MAGIC.len()] != MAGIC {
        return Err(StateError::Corrupt("bad magic or too short"));
    }
    let (body, tail) = bytes.split_at(bytes.len() - 4);
    if crc32(body) != u32::from_le_bytes(tail.try_into().unwrap()) {
        return Err(StateError::Checksum);
    }
    let mut r = Reader {
        b: body,
        pos: MAGIC.len(),
    };
    let version = r.u32()?;
    if version != FORMAT_VERSION {
        return Err(StateError::UnsupportedVersion(version));
    }
    let seed = WorldSeed(r.u64()?);
    let cfg = StreamConfig {
        load_radius: r.i32()?,
        unload_margin: r.i32()?,
        max_queue: r.u32()? as usize,
    };
    if !(0..=64).contains(&cfg.load_radius) || !(0..=64).contains(&cfg.unload_margin) {
        return Err(StateError::Corrupt("streaming config out of range"));
    }
    let epoch = r.u32()?;
    let _revision = r.u64()?;
    let has_center = r.u8()? == 1;
    let (cx, cz) = (r.i32()?, r.i32()?);
    let n_active = r.u32()? as usize;
    let mut saved_active = Vec::new();
    for _ in 0..n_active {
        saved_active.push(ChunkCoord::new(r.i32()?, r.i32()?));
    }
    let n_edit_chunks = r.u32()? as usize;
    let mut edits = Vec::new();
    for _ in 0..n_edit_chunks {
        let c = ChunkCoord::new(r.i32()?, r.i32()?);
        let n = r.u32()? as usize;
        let mut list = Vec::new();
        for _ in 0..n {
            list.push(VoxelEdit {
                pos: VoxelPos::new(r.i32()?, r.i32()?, r.i32()?),
                material: r.u16()?,
            });
        }
        edits.push((c, list));
    }
    if r.pos != body.len() {
        return Err(StateError::Corrupt("trailing bytes"));
    }

    let mut w = WorldStreamer::new(seed, generator, cfg);
    for _ in 0..=epoch {
        w.asset_mut().begin_new_epoch();
    }
    let mut edits_restored = 0;
    for (c, list) in edits {
        edits_restored += list.len();
        w.edits.insert(c, list);
    }
    if has_center {
        w.set_center_world(cx * CHUNK_SIZE, cz * CHUNK_SIZE);
        // The saved active set may include chunks kept by the unload margin; bring those back too.
        for c in &saved_active {
            w.queue_explicit(*c);
        }
        w.pump_all();
        w.take_delta();
    }
    let ready = w.active_chunks();
    let missing: Vec<ChunkCoord> = saved_active
        .iter()
        .copied()
        .filter(|c| !ready.contains(c))
        .collect();
    let report = LoadReport {
        saved_active: saved_active.len(),
        restored_active: ready.len(),
        missing,
        edits_restored,
    };
    Ok((w, report))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc_known_value() {
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
    }
}
