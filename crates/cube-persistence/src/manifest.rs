//! The manifest (PERS-02, EXPERIMENTAL): one checksummed record that names the files that make
//! up a world, published atomically (temporary file, `fsync`, rename, directory `fsync`).
//!
//! A reader sees the complete old manifest or the complete new one, never a mix: the rename is
//! the single commit point. A manifest that is present but damaged is an error (fail closed),
//! never "no world". Files that the manifest does not name are garbage left by an interrupted
//! step; they are never read as data and are removed by [`collect_garbage`].

use crate::codec::{DecodeError, Limits, Manifest, decode_manifest, encode_manifest};
use crate::store::{Storage, StoreError, save_atomic};

/// File name of the manifest.
pub const MANIFEST_FILE: &str = "manifest";

/// Why the manifest could not be used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestError {
    /// The storage failed.
    Store(StoreError),
    /// The manifest exists but is damaged or of an unknown schema.
    Corrupt(DecodeError),
    /// The new manifest does not continue the published one (generation must grow by one).
    NotNext {
        /// Generation that was expected.
        expected: u64,
        /// Generation that was offered.
        found: u64,
    },
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Store(e) => e.fmt(f),
            Self::Corrupt(e) => write!(f, "manifest is damaged: {e}"),
            Self::NotNext { expected, found } => {
                write!(f, "manifest generation {found}, expected {expected}")
            }
        }
    }
}

impl std::error::Error for ManifestError {}

/// Reads the manifest. `Ok(None)` means there is none; a damaged one is an error.
pub fn load_manifest(s: &dyn Storage, limits: &Limits) -> Result<Option<Manifest>, ManifestError> {
    match s.read(MANIFEST_FILE).map_err(ManifestError::Store)? {
        None => Ok(None),
        Some(b) => decode_manifest(&b, limits)
            .map(Some)
            .map_err(ManifestError::Corrupt),
    }
}

/// Publishes `next` atomically. Its generation must be the published one plus one (1 for the
/// first manifest). After a failure the old manifest is still the valid one (or the new one is
/// whole): the caller reloads to find out which.
pub fn publish_manifest(
    s: &mut dyn Storage,
    next: &Manifest,
    limits: &Limits,
) -> Result<(), ManifestError> {
    let expected = load_manifest(s, limits)?.map_or(1, |m| m.generation + 1);
    if next.generation != expected {
        return Err(ManifestError::NotNext {
            expected,
            found: next.generation,
        });
    }
    save_atomic(s, MANIFEST_FILE, &encode_manifest(next)).map_err(ManifestError::Store)
}

/// `true` for the names this layer owns (snapshots, journals, their marks and temporary files).
fn owned(name: &str) -> bool {
    name.starts_with("snap-") || name.starts_with("journal-") || name == "manifest.tmp"
}

/// Removes owned files that the manifest does not name. Returns the removed names.
pub fn collect_garbage(s: &mut dyn Storage, m: &Manifest) -> Result<Vec<String>, StoreError> {
    let keep = |n: &str| {
        n == MANIFEST_FILE
            || n == m.journal
            || n == format!("{}.mark", m.journal)
            || m.snapshot.as_ref().is_some_and(|r| r.file == n)
    };
    let mut gone = Vec::new();
    for n in s.list()? {
        if owned(&n) && !keep(&n) {
            s.remove(&n)?;
            gone.push(n);
        }
    }
    if !gone.is_empty() {
        s.fsync_dir()?;
    }
    Ok(gone)
}

/// `true` if any file of the world exists without a manifest (a lost manifest).
pub fn has_world_files(s: &dyn Storage) -> Result<bool, StoreError> {
    Ok(s.list()?
        .iter()
        .any(|n| n.starts_with("snap-") || n.starts_with("journal-")))
}
