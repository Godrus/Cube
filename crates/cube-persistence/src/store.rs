//! Atomic snapshot files over an abstract storage, and a real-directory implementation
//! (PERS foundation; the full snapshot store port, manifest, compaction come with PS-10).
//!
//! The protocol for replacing a snapshot is the textbook one: write a temporary file, make it
//! durable (`fsync`), rename it over the target, make the directory entry durable
//! (`fsync_dir`). A reader therefore sees either the complete old file or the complete new one
//! (checksummed, so a torn or corrupt file is *detected*, never silently accepted). The same
//! protocol is exercised against the crash-point model of `cube-testkit::faultfs`; that is a
//! model, not a durability claim about any real platform (DG-2). `DirStorage` performs the real
//! calls; on platforms that cannot `fsync` a directory (Windows) that last step is a no-op and
//! the platform guarantee is whatever the OS gives: it is not claimed.

use std::io;
use std::path::PathBuf;

use cube_core::AssetView;

use crate::codec::{DecodeError, Limits, decode_asset, encode_asset};

/// What the protocol needs from a filesystem.
pub trait Storage {
    /// Creates or truncates `name` and writes `data` (not yet durable).
    fn write(&mut self, name: &str, data: &[u8]) -> Result<(), StoreError>;
    /// Appends `data` to `name`, creating it if needed (not yet durable).
    fn append(&mut self, name: &str, data: &[u8]) -> Result<(), StoreError>;
    /// Shortens `name` to `len` bytes (not yet durable until `fsync`).
    fn truncate(&mut self, name: &str, len: u64) -> Result<(), StoreError>;
    /// Makes the content of `name` durable.
    fn fsync(&mut self, name: &str) -> Result<(), StoreError>;
    /// Atomically renames `from` over `to`.
    fn rename(&mut self, from: &str, to: &str) -> Result<(), StoreError>;
    /// Makes the directory entries durable.
    fn fsync_dir(&mut self) -> Result<(), StoreError>;
    /// Reads `name`, `None` if it does not exist.
    fn read(&self, name: &str) -> Result<Option<Vec<u8>>, StoreError>;
}

/// A storage failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreError {
    /// The operation failed (message of the underlying error).
    Io(String),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(m) => write!(f, "storage error: {m}"),
        }
    }
}

impl std::error::Error for StoreError {}

/// Why a snapshot could not be loaded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadError {
    /// No snapshot exists under that name.
    Missing,
    /// The storage failed.
    Store(StoreError),
    /// The file exists but is not a valid record: corrupt, truncated or foreign.
    Corrupt(DecodeError),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing => f.write_str("no snapshot"),
            Self::Store(e) => e.fmt(f),
            Self::Corrupt(e) => write!(f, "corrupt snapshot: {e}"),
        }
    }
}

impl std::error::Error for LoadError {}

/// Replaces `name` with `bytes` atomically (see the module documentation). If any step fails the
/// old file is untouched; a leftover temporary file is harmless and overwritten next time.
pub fn save_atomic(s: &mut dyn Storage, name: &str, bytes: &[u8]) -> Result<(), StoreError> {
    let tmp = format!("{name}.tmp");
    s.write(&tmp, bytes)?;
    s.fsync(&tmp)?;
    s.rename(&tmp, name)?;
    s.fsync_dir()
}

/// Saves an asset snapshot.
pub fn save_asset(s: &mut dyn Storage, name: &str, view: &AssetView) -> Result<(), StoreError> {
    save_atomic(s, name, &encode_asset(view))
}

/// Loads and verifies an asset snapshot.
pub fn load_asset(s: &dyn Storage, name: &str, limits: &Limits) -> Result<AssetView, LoadError> {
    let bytes = s
        .read(name)
        .map_err(LoadError::Store)?
        .ok_or(LoadError::Missing)?;
    decode_asset(&bytes, limits).map_err(LoadError::Corrupt)
}

/// A real directory.
#[derive(Debug, Clone)]
pub struct DirStorage {
    root: PathBuf,
}

impl DirStorage {
    /// Uses `root`, creating it if needed.
    pub fn open(root: impl Into<PathBuf>) -> Result<Self, StoreError> {
        let root = root.into();
        std::fs::create_dir_all(&root).map_err(io_err)?;
        Ok(Self { root })
    }

    fn path(&self, name: &str) -> Result<PathBuf, StoreError> {
        // File names only: no separators, no parent references.
        if name.is_empty() || name.contains(['/', '\\']) || name == ".." || name == "." {
            return Err(StoreError::Io(format!("invalid file name {name:?}")));
        }
        Ok(self.root.join(name))
    }
}

fn io_err(e: io::Error) -> StoreError {
    StoreError::Io(e.to_string())
}

impl Storage for DirStorage {
    fn write(&mut self, name: &str, data: &[u8]) -> Result<(), StoreError> {
        std::fs::write(self.path(name)?, data).map_err(io_err)
    }
    fn append(&mut self, name: &str, data: &[u8]) -> Result<(), StoreError> {
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.path(name)?)
            .map_err(io_err)?;
        f.write_all(data).map_err(io_err)
    }
    fn truncate(&mut self, name: &str, len: u64) -> Result<(), StoreError> {
        let f = std::fs::OpenOptions::new()
            .write(true)
            .open(self.path(name)?)
            .map_err(io_err)?;
        f.set_len(len).map_err(io_err)
    }
    fn fsync(&mut self, name: &str) -> Result<(), StoreError> {
        // Write access is needed to flush on Windows; the file is not truncated.
        let f = std::fs::OpenOptions::new()
            .write(true)
            .open(self.path(name)?)
            .map_err(io_err)?;
        f.sync_all().map_err(io_err)
    }
    fn rename(&mut self, from: &str, to: &str) -> Result<(), StoreError> {
        std::fs::rename(self.path(from)?, self.path(to)?).map_err(io_err)
    }
    fn fsync_dir(&mut self) -> Result<(), StoreError> {
        #[cfg(unix)]
        {
            let d = std::fs::File::open(&self.root).map_err(io_err)?;
            d.sync_all().map_err(io_err)?;
        }
        Ok(())
    }
    fn read(&self, name: &str) -> Result<Option<Vec<u8>>, StoreError> {
        match std::fs::read(self.path(name)?) {
            Ok(b) => Ok(Some(b)),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(io_err(e)),
        }
    }
}
