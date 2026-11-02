//! Cube Persistence (experimental spike; draft P-01, P-02, P-05, P-06 at minimal level).
//!
//! Depends on `cube-core` only. Single-file asset format with per-brick CRC32 and a whole-file
//! CRC32 trailer, written atomically (temp file, flush, rename). The format is a spike format
//! (version 1) and is NOT the approved serialization format (an Open Decision).
//! Not implemented: incremental deltas, durable snapshots, deduplication, migrations,
//! unknown-feature preservation, async saving, multi-asset commits.

use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use cube_core::{
    AssetExport, AssetId, Brick, BrickCoord, BrickData, CoreError, GeometryAsset, LayerExport,
    LayerId, BRICK_VOLUME,
};

const MAGIC: &[u8; 8] = b"CUBEAST1";
const FORMAT_VERSION: u32 = 1;

#[derive(Debug)]
pub enum PersistenceError {
    Io(std::io::Error),
    /// Bad magic, truncated data or structural inconsistency.
    Corrupt(&'static str),
    /// A CRC did not match.
    IntegrityFailure(&'static str),
    UnsupportedVersion(u32),
    Core(CoreError),
}

impl PersistenceError {
    pub fn code(&self) -> &'static str {
        match self {
            PersistenceError::Io(_) => "PERS-0001",
            PersistenceError::Corrupt(_) => "PERS-0002",
            PersistenceError::IntegrityFailure(_) => "PERS-0003",
            PersistenceError::UnsupportedVersion(_) => "PERS-0004",
            PersistenceError::Core(_) => "PERS-0005",
        }
    }
}

impl std::fmt::Display for PersistenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PersistenceError::Io(e) => write!(f, "{}: io: {e}", self.code()),
            PersistenceError::Corrupt(m) => write!(f, "{}: corrupt file: {m}", self.code()),
            PersistenceError::IntegrityFailure(m) => {
                write!(f, "{}: integrity failure: {m}", self.code())
            }
            PersistenceError::UnsupportedVersion(v) => {
                write!(f, "{}: unsupported format version {v}", self.code())
            }
            PersistenceError::Core(e) => write!(f, "{}: {e}", self.code()),
        }
    }
}

impl std::error::Error for PersistenceError {}

impl From<std::io::Error> for PersistenceError {
    fn from(e: std::io::Error) -> Self {
        PersistenceError::Io(e)
    }
}

impl From<CoreError> for PersistenceError {
    fn from(e: CoreError) -> Self {
        PersistenceError::Core(e)
    }
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

struct W(Vec<u8>);
impl W {
    fn u8(&mut self, v: u8) {
        self.0.push(v);
    }
    fn u16(&mut self, v: u16) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn u32(&mut self, v: u32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn i32(&mut self, v: i32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn u64(&mut self, v: u64) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn f32(&mut self, v: f32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
}

struct R<'a> {
    b: &'a [u8],
    pos: usize,
}
impl<'a> R<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], PersistenceError> {
        if self.b.len() - self.pos < n {
            return Err(PersistenceError::Corrupt("unexpected end of data"));
        }
        let s = &self.b[self.pos..self.pos + n];
        self.pos += n;
        Ok(s)
    }
    fn u8(&mut self) -> Result<u8, PersistenceError> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, PersistenceError> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }
    fn u32(&mut self) -> Result<u32, PersistenceError> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn i32(&mut self) -> Result<i32, PersistenceError> {
        Ok(i32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn u64(&mut self) -> Result<u64, PersistenceError> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn f32(&mut self) -> Result<f32, PersistenceError> {
        Ok(f32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
}

fn encode_brick_payload(b: &Brick) -> Vec<u8> {
    let mut w = W(Vec::new());
    w.u64(b.revision());
    match b.data() {
        BrickData::Constant(m) => {
            w.u8(0);
            w.u16(*m);
        }
        BrickData::Dense(d) => {
            w.u8(1);
            for v in d.iter() {
                w.u16(*v);
            }
        }
    }
    w.0
}

fn decode_brick_payload(p: &[u8]) -> Result<Brick, PersistenceError> {
    let mut r = R { b: p, pos: 0 };
    let rev = r.u64()?;
    let data = match r.u8()? {
        0 => BrickData::Constant(r.u16()?),
        1 => {
            let mut d = Box::new([0u16; BRICK_VOLUME]);
            for v in d.iter_mut() {
                *v = r.u16()?;
            }
            BrickData::Dense(d)
        }
        _ => return Err(PersistenceError::Corrupt("unknown brick kind")),
    };
    if r.pos != p.len() {
        return Err(PersistenceError::Corrupt("trailing bytes in brick"));
    }
    Ok(Brick::from_data(data, rev))
}

pub fn encode_asset(asset: &GeometryAsset) -> Vec<u8> {
    let e = asset.export_view();
    let mut w = W(Vec::new());
    w.0.extend_from_slice(MAGIC);
    w.u32(FORMAT_VERSION);
    w.u64(e.id.0);
    w.u32(e.epoch);
    w.u64(e.revision);
    w.u32(e.layers.len() as u32);
    for l in &e.layers {
        w.u64(l.id.0);
        w.f32(l.voxel_size_m);
        w.u64(l.revision);
        w.u32(l.bricks.len() as u32);
        for (c, b) in &l.bricks {
            w.i32(c.x);
            w.i32(c.y);
            w.i32(c.z);
            let payload = encode_brick_payload(b);
            w.u32(payload.len() as u32);
            w.u32(crc32(&payload));
            w.0.extend_from_slice(&payload);
        }
        w.u32(l.tombstones.len() as u32);
        for (c, r) in &l.tombstones {
            w.i32(c.x);
            w.i32(c.y);
            w.i32(c.z);
            w.u64(*r);
        }
    }
    let crc = crc32(&w.0);
    w.u32(crc);
    w.0
}

pub fn decode_asset(bytes: &[u8]) -> Result<GeometryAsset, PersistenceError> {
    if bytes.len() < MAGIC.len() + 4 + 4 {
        return Err(PersistenceError::Corrupt("file too short"));
    }
    let (body, trailer) = bytes.split_at(bytes.len() - 4);
    if &body[..MAGIC.len()] != MAGIC {
        return Err(PersistenceError::Corrupt("bad magic"));
    }
    let stored = u32::from_le_bytes(trailer.try_into().unwrap());
    if crc32(body) != stored {
        return Err(PersistenceError::IntegrityFailure("file checksum mismatch"));
    }
    let mut r = R {
        b: body,
        pos: MAGIC.len(),
    };
    let version = r.u32()?;
    if version != FORMAT_VERSION {
        return Err(PersistenceError::UnsupportedVersion(version));
    }
    let id = AssetId(r.u64()?);
    let epoch = r.u32()?;
    let revision = r.u64()?;
    let layer_count = r.u32()? as usize;
    let mut layers = Vec::new();
    for _ in 0..layer_count {
        let lid = LayerId(r.u64()?);
        let voxel_size_m = r.f32()?;
        let lrev = r.u64()?;
        let brick_count = r.u32()? as usize;
        let mut bricks = Vec::new();
        for _ in 0..brick_count {
            let c = BrickCoord::new(r.i32()?, r.i32()?, r.i32()?);
            let len = r.u32()? as usize;
            let crc = r.u32()?;
            let payload = r.take(len)?;
            if crc32(payload) != crc {
                return Err(PersistenceError::IntegrityFailure(
                    "brick checksum mismatch",
                ));
            }
            bricks.push((c, Arc::new(decode_brick_payload(payload)?)));
        }
        let tomb_count = r.u32()? as usize;
        let mut tombstones = Vec::new();
        for _ in 0..tomb_count {
            tombstones.push((BrickCoord::new(r.i32()?, r.i32()?, r.i32()?), r.u64()?));
        }
        layers.push(LayerExport {
            id: lid,
            voxel_size_m,
            revision: lrev,
            bricks,
            tombstones,
        });
    }
    if r.pos != body.len() {
        return Err(PersistenceError::Corrupt("trailing bytes"));
    }
    Ok(GeometryAsset::import_view(AssetExport {
        id,
        epoch,
        revision,
        layers,
    })?)
}

fn temp_path(path: &Path) -> PathBuf {
    let mut name = path
        .file_name()
        .map(|n| n.to_os_string())
        .unwrap_or_default();
    name.push(".tmp");
    path.with_file_name(name)
}

/// Atomic save: the destination is either the old file or the complete new file.
pub fn save_asset(path: &Path, asset: &GeometryAsset) -> Result<(), PersistenceError> {
    let bytes = encode_asset(asset);
    let tmp = temp_path(path);
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

/// Loads and validates. The returned asset starts a new epoch: tokens issued before the
/// save/reload are `Incomparable` with tokens of the loaded asset.
pub fn load_asset(path: &Path) -> Result<GeometryAsset, PersistenceError> {
    let mut bytes = Vec::new();
    fs::File::open(path)?.read_to_end(&mut bytes)?;
    let mut asset = decode_asset(&bytes)?;
    asset.begin_new_epoch();
    Ok(asset)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cube_core::{BrickWrite, ExpectedBrick, LocalPos, Region, WriteSet};

    fn sample() -> GeometryAsset {
        let mut a = GeometryAsset::new(AssetId(7));
        let l = a.add_voxel_layer(0.25);
        for (i, c) in [BrickCoord::new(-1, 0, 0), BrickCoord::new(2, -3, 4)]
            .into_iter()
            .enumerate()
        {
            let mut b = Brick::constant(0);
            b.set(LocalPos { x: 1, y: 2, z: 3 }, 10 + i as u16);
            a.commit(WriteSet {
                layer: l,
                operation: None,
                writes: vec![BrickWrite {
                    coord: c,
                    expected: ExpectedBrick::Absent,
                    new: Some(b),
                    changed: Region::of_brick(c),
                }],
            })
            .unwrap();
        }
        // Constant non-zero brick and a removed brick (tombstone).
        let c = BrickCoord::new(5, 5, 5);
        a.commit(WriteSet {
            layer: l,
            operation: None,
            writes: vec![BrickWrite {
                coord: c,
                expected: ExpectedBrick::Absent,
                new: Some(Brick::constant(3)),
                changed: Region::of_brick(c),
            }],
        })
        .unwrap();
        a.commit(WriteSet {
            layer: l,
            operation: None,
            writes: vec![BrickWrite {
                coord: c,
                expected: ExpectedBrick::Revision(1),
                new: None,
                changed: Region::of_brick(c),
            }],
        })
        .unwrap();
        a
    }

    fn tmp_file(name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("cube_pers_{}_{}", std::process::id(), name));
        p
    }

    #[test]
    fn crc32_known_value() {
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
    }

    #[test]
    fn roundtrip_preserves_data_and_revisions() {
        let a = sample();
        let b = decode_asset(&encode_asset(&a)).unwrap();
        assert_eq!(b.asset_token(), a.asset_token());
        let (ea, eb) = (a.export_view(), b.export_view());
        assert_eq!(ea.layers[0].tombstones, eb.layers[0].tombstones);
        assert_eq!(ea.layers[0].bricks.len(), eb.layers[0].bricks.len());
        for ((ca, ba), (cb, bb)) in ea.layers[0].bricks.iter().zip(eb.layers[0].bricks.iter()) {
            assert_eq!(ca, cb);
            assert!(ba.same_content(bb));
            assert_eq!(ba.revision(), bb.revision());
        }
    }

    #[test]
    fn corruption_is_detected_everywhere() {
        let bytes = encode_asset(&sample());
        for i in [0usize, 9, 20, bytes.len() / 2, bytes.len() - 1] {
            let mut bad = bytes.clone();
            bad[i] ^= 0x55;
            assert!(decode_asset(&bad).is_err(), "flip at {i} not detected");
        }
        assert!(decode_asset(&bytes[..bytes.len() - 7]).is_err());
        assert!(decode_asset(&[]).is_err());
    }

    #[test]
    fn save_load_starts_new_epoch_and_leaves_no_temp_file() {
        let a = sample();
        let p = tmp_file("a.cube");
        save_asset(&p, &a).unwrap();
        assert!(!temp_path(&p).exists());
        let b = load_asset(&p).unwrap();
        assert_eq!(b.epoch(), a.epoch() + 1);
        assert_eq!(
            b.asset_token().compare(&a.asset_token()),
            cube_core::RevisionOrder::Incomparable
        );
        // Overwrite keeps working.
        save_asset(&p, &b).unwrap();
        assert!(load_asset(&p).is_ok());
        let _ = fs::remove_file(&p);
    }

    #[test]
    fn failed_save_keeps_old_file() {
        let a = sample();
        let p = tmp_file("keep.cube");
        save_asset(&p, &a).unwrap();
        let before = fs::read(&p).unwrap();
        // A directory in place of the temp file makes the write fail before any rename.
        fs::create_dir(temp_path(&p)).unwrap();
        assert!(save_asset(&p, &a).is_err());
        assert_eq!(fs::read(&p).unwrap(), before);
        let _ = fs::remove_dir(temp_path(&p));
        let _ = fs::remove_file(&p);
    }

    #[test]
    fn missing_file_is_io_error() {
        let p = tmp_file("missing.cube");
        assert!(matches!(load_asset(&p), Err(PersistenceError::Io(_))));
    }
}
