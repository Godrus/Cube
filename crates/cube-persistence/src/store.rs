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
    /// Deletes `name` (not durable until `fsync_dir`); a missing file is not an error.
    fn remove(&mut self, name: &str) -> Result<(), StoreError>;
    /// Makes the directory entries durable.
    fn fsync_dir(&mut self) -> Result<(), StoreError>;
    /// Reads `name`, `None` if it does not exist.
    fn read(&self, name: &str) -> Result<Option<Vec<u8>>, StoreError>;
    /// Names of all files, ascending.
    fn list(&self) -> Result<Vec<String>, StoreError>;
}

impl<T: Storage + ?Sized> Storage for &mut T {
    fn write(&mut self, n: &str, d: &[u8]) -> Result<(), StoreError> {
        (**self).write(n, d)
    }
    fn append(&mut self, n: &str, d: &[u8]) -> Result<(), StoreError> {
        (**self).append(n, d)
    }
    fn truncate(&mut self, n: &str, l: u64) -> Result<(), StoreError> {
        (**self).truncate(n, l)
    }
    fn fsync(&mut self, n: &str) -> Result<(), StoreError> {
        (**self).fsync(n)
    }
    fn rename(&mut self, a: &str, b: &str) -> Result<(), StoreError> {
        (**self).rename(a, b)
    }
    fn remove(&mut self, n: &str) -> Result<(), StoreError> {
        (**self).remove(n)
    }
    fn fsync_dir(&mut self) -> Result<(), StoreError> {
        (**self).fsync_dir()
    }
    fn read(&self, n: &str) -> Result<Option<Vec<u8>>, StoreError> {
        (**self).read(n)
    }
    fn list(&self) -> Result<Vec<String>, StoreError> {
        (**self).list()
    }
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

/// Name of the writer lock file inside a world directory.
pub const LOCK_FILE: &str = ".world.lock";

/// The exclusive right to write a world directory (M1-A5, concurrent writers).
///
/// Two processes (or two hosts in one process) that append to the same journal would interleave
/// records and break the gapless sequence, so a world is opened by one writer at a time. The lock
/// is an advisory OS file lock on [`LOCK_FILE`]: it is released when the value is dropped and by
/// the operating system when the process dies, so a crash never leaves a stale lock behind (the
/// leftover file itself is harmless and is not removed). Readers that do not write do not need
/// it.
#[derive(Debug)]
pub struct WorldLock {
    _file: std::fs::File,
}

impl WorldLock {
    /// Takes the lock of the world directory `root` (created if needed). Fails at once, without
    /// waiting, when another writer holds it.
    pub fn acquire(root: impl AsRef<std::path::Path>) -> Result<Self, StoreError> {
        let root = root.as_ref();
        std::fs::create_dir_all(root).map_err(io_err)?;
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(root.join(LOCK_FILE))
            .map_err(io_err)?;
        match file.try_lock() {
            Ok(()) => Ok(Self { _file: file }),
            Err(std::fs::TryLockError::WouldBlock) => Err(StoreError::Io(
                "the world is already open for writing (another host holds the writer lock)"
                    .to_owned(),
            )),
            Err(std::fs::TryLockError::Error(e)) => Err(io_err(e)),
        }
    }
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
    fn remove(&mut self, name: &str) -> Result<(), StoreError> {
        match std::fs::remove_file(self.path(name)?) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(io_err(e)),
        }
    }
    fn fsync_dir(&mut self) -> Result<(), StoreError> {
        #[cfg(unix)]
        {
            let d = std::fs::File::open(&self.root).map_err(io_err)?;
            d.sync_all().map_err(io_err)?;
        }
        Ok(())
    }
    fn list(&self) -> Result<Vec<String>, StoreError> {
        let mut out = Vec::new();
        for e in std::fs::read_dir(&self.root).map_err(io_err)? {
            let e = e.map_err(io_err)?;
            if e.file_type().map_err(io_err)?.is_file() {
                out.push(e.file_name().to_string_lossy().into_owned());
            }
        }
        out.sort();
        Ok(out)
    }
    fn read(&self, name: &str) -> Result<Option<Vec<u8>>, StoreError> {
        match std::fs::read(self.path(name)?) {
            Ok(b) => Ok(Some(b)),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(io_err(e)),
        }
    }
}

#[cfg(test)]
mod lock_tests {
    use super::*;

    fn dir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("cube-lock-{}-{tag}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    #[test]
    fn a_second_writer_is_refused_until_the_first_lets_go() {
        let d = dir("two");
        let first = WorldLock::acquire(&d).unwrap();
        let second = WorldLock::acquire(&d);
        assert!(
            matches!(&second, Err(StoreError::Io(m)) if m.contains("already open")),
            "{second:?}"
        );
        drop(first);
        let again = WorldLock::acquire(&d);
        assert!(
            again.is_ok(),
            "the lock is free after the writer is gone: {again:?}"
        );
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn a_leftover_lock_file_is_not_a_stale_lock() {
        // A crash leaves the file behind but the OS drops the lock with the process.
        let d = dir("stale");
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join(LOCK_FILE), b"left by a dead writer").unwrap();
        assert!(WorldLock::acquire(&d).is_ok());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn different_worlds_do_not_block_each_other() {
        let (a, b) = (dir("a"), dir("b"));
        let _la = WorldLock::acquire(&a).unwrap();
        assert!(WorldLock::acquire(&b).is_ok());
        let _ = std::fs::remove_dir_all(&a);
        let _ = std::fs::remove_dir_all(&b);
    }
}
