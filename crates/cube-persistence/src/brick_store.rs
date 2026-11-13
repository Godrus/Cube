//! Per-brick store of unloaded edited bricks (M1-A1b, DEC-106; experimental until GOV-10).
//!
//! A brick the player edited cannot be regenerated, so before it is unloaded its content goes to
//! one checksummed file, written atomically (temporary file, fsync, rename, directory fsync, the
//! same protocol as the snapshot). The file also keeps the revision the brick has after the
//! reload that follows the unload: a restart restores the content, brings the revision to that
//! number with the same unload and reload steps the host already uses, and then replays the
//! journal unchanged (the M1-A1 spike, `docs/roadmap/WORK-PACKAGES.md`).
//!
//! The file is a normal framed record (magic, schema, kind, length, CRC-32), so a damaged,
//! truncated or foreign file is refused, never half read. Nothing here is an approved format.

use cube_core::{BrickCoord, CellChange};

use crate::codec::{DecodeError, Limits, decode_stored_brick, encode_stored_brick};
use crate::store::{Storage, StoreError, save_atomic};

/// The content of an unloaded edited brick and its revision after the next reload.
#[derive(Debug, Clone, PartialEq)]
pub struct StoredBrick {
    /// The revision the brick has once it is loaded again from this content.
    pub revision_after_reload: u64,
    /// What the brick holds.
    pub change: CellChange,
}

/// Why a stored brick could not be read.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum BrickStoreError {
    /// The storage failed.
    Store(StoreError),
    /// The file exists but is damaged or belongs to another brick.
    Corrupt(DecodeError),
}

impl std::fmt::Display for BrickStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Store(e) => e.fmt(f),
            Self::Corrupt(e) => write!(f, "damaged brick file: {e}"),
        }
    }
}

impl std::error::Error for BrickStoreError {}

/// File name of a brick (no separators; negative coordinates keep their minus sign).
pub fn brick_file(b: BrickCoord) -> String {
    format!("brick-{}_{}_{}.cbk", b.x, b.y, b.z)
}

/// Writes the brick's file atomically. On `Err` nothing is replaced: the caller must keep the
/// brick resident.
pub fn save_brick(
    s: &mut dyn Storage,
    b: BrickCoord,
    stored: &StoredBrick,
) -> Result<(), StoreError> {
    let bytes = encode_stored_brick(b, stored.revision_after_reload, &stored.change);
    save_atomic(s, &brick_file(b), &bytes)
}

/// Reads the brick's file: `Ok(None)` if there is none, an error if it is damaged.
pub fn load_brick(
    s: &dyn Storage,
    b: BrickCoord,
    limits: &Limits,
) -> Result<Option<StoredBrick>, BrickStoreError> {
    let Some(bytes) = s.read(&brick_file(b)).map_err(BrickStoreError::Store)? else {
        return Ok(None);
    };
    let (found, revision_after_reload, change) =
        decode_stored_brick(&bytes, limits).map_err(BrickStoreError::Corrupt)?;
    if found != b {
        return Err(BrickStoreError::Corrupt(DecodeError::Invalid(
            "file holds another brick",
        )));
    }
    Ok(Some(StoredBrick {
        revision_after_reload,
        change,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::DirStorage;
    use cube_core::{CellPayload, ChannelSet, MaterialIndex};

    fn stored(material: u32, revision: u64) -> StoredBrick {
        StoredBrick {
            revision_after_reload: revision,
            change: CellChange::Put(CellPayload::Channels(ChannelSet::uniform_material(
                MaterialIndex::from_raw(material),
            ))),
        }
    }

    fn dir(tag: &str) -> std::path::PathBuf {
        let t = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        std::env::temp_dir().join(format!("cube-brickstore-{tag}-{t}"))
    }

    #[test]
    fn a_brick_round_trips_including_wide_materials_and_known_empty() {
        let d = dir("rt");
        let mut s = DirStorage::open(&d).unwrap();
        let b = BrickCoord::new(-3, 0, 12);
        for st in [
            stored(9, 4),
            stored(70_000, 7),
            StoredBrick {
                revision_after_reload: 2,
                change: CellChange::KnownEmpty,
            },
        ] {
            save_brick(&mut s, b, &st).unwrap();
            assert_eq!(load_brick(&s, b, &Limits::default()).unwrap(), Some(st));
        }
        assert_eq!(
            load_brick(&s, BrickCoord::new(1, 1, 1), &Limits::default()).unwrap(),
            None
        );
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn a_damaged_truncated_or_foreign_file_is_refused() {
        let d = dir("bad");
        let mut s = DirStorage::open(&d).unwrap();
        let b = BrickCoord::new(1, 2, 3);
        save_brick(&mut s, b, &stored(5, 3)).unwrap();
        let name = brick_file(b);
        let good = s.read(&name).unwrap().unwrap();
        let limits = Limits::default();
        // One flipped byte, a truncated file, trailing bytes.
        let mut flipped = good.clone();
        let mid = flipped.len() / 2;
        flipped[mid] ^= 1;
        let mut extra = good.clone();
        extra.push(0);
        for bad in [flipped, good[..good.len() - 1].to_vec(), extra] {
            s.write(&name, &bad).unwrap();
            assert!(matches!(
                load_brick(&s, b, &limits),
                Err(BrickStoreError::Corrupt(_))
            ));
        }
        // A valid file of another brick under this brick's name.
        let other = encode_stored_brick(BrickCoord::new(9, 9, 9), 3, &stored(5, 3).change);
        s.write(&name, &other).unwrap();
        assert!(matches!(
            load_brick(&s, b, &limits),
            Err(BrickStoreError::Corrupt(_))
        ));
        let _ = std::fs::remove_dir_all(&d);
    }
}
