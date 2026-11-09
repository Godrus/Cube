//! The snapshot store (PERS-03, EXPERIMENTAL): snapshots that record the journal position they
//! contain, and that are **invalid until verified**.
//!
//! `write_snapshot` writes the file atomically, then reads it back, decodes it and compares the
//! content hash and the size with what it meant to write; only then does it hand out a
//! [`SnapshotRef`]. A snapshot is part of the world only once a manifest names it, so a file
//! that was written but never verified or never published is garbage. `load_snapshot` checks
//! the file against the reference again (size, checksum through the codec, content hash).

use cube_core::AssetView;

use crate::codec::{DecodeError, Limits, SnapshotRef, decode_asset, encode_asset};
use crate::store::{Storage, StoreError, save_atomic};

/// File name for the snapshot that contains the journal up to `journal_seq`.
pub fn snapshot_file(journal_seq: u64) -> String {
    format!("snap-{journal_seq:020}")
}

/// Why a snapshot is not usable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotError {
    /// The storage failed.
    Store(StoreError),
    /// The file named by the reference does not exist.
    Missing(String),
    /// The file does not decode (damaged, truncated, unknown schema).
    Corrupt(DecodeError),
    /// The file decodes but is not the one the manifest promised (size or content hash).
    Mismatch,
}

impl std::fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Store(e) => e.fmt(f),
            Self::Missing(n) => write!(f, "snapshot {n} is missing"),
            Self::Corrupt(e) => write!(f, "snapshot is damaged: {e}"),
            Self::Mismatch => f.write_str("snapshot does not match its reference"),
        }
    }
}

impl std::error::Error for SnapshotError {}

/// Writes and verifies a snapshot of `view` that contains the journal up to `journal_seq`.
pub fn write_snapshot(
    s: &mut dyn Storage,
    view: &AssetView,
    journal_seq: u64,
    limits: &Limits,
) -> Result<SnapshotRef, SnapshotError> {
    let file = snapshot_file(journal_seq);
    let bytes = encode_asset(view);
    save_atomic(s, &file, &bytes).map_err(SnapshotError::Store)?;
    let r = SnapshotRef {
        file,
        journal_seq,
        content_hash: view.content_hash().value(),
        bytes: bytes.len() as u64,
    };
    load_snapshot(s, &r, limits)?;
    Ok(r)
}

/// Loads the snapshot `r` and checks it against the reference.
pub fn load_snapshot(
    s: &dyn Storage,
    r: &SnapshotRef,
    limits: &Limits,
) -> Result<AssetView, SnapshotError> {
    let bytes = s
        .read(&r.file)
        .map_err(SnapshotError::Store)?
        .ok_or_else(|| SnapshotError::Missing(r.file.clone()))?;
    if bytes.len() as u64 != r.bytes {
        return Err(SnapshotError::Mismatch);
    }
    let view = decode_asset(&bytes, limits).map_err(SnapshotError::Corrupt)?;
    if view.content_hash().value() != r.content_hash {
        return Err(SnapshotError::Mismatch);
    }
    Ok(view)
}
