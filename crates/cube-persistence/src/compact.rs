//! World files, opening and compaction (PERS-05, EXPERIMENTAL; AOD-21 protocol).
//!
//! Layout: `manifest` names one verified snapshot (optional) and one journal. Compaction folds
//! the journal prefix into a new snapshot and drops it from the journal:
//!
//! 1. check the boundary (every operation begun at or before it is closed at or before it);
//! 2. write the new snapshot atomically and **verify** it (read back, decode, hash);
//! 3. write the journal tail (records after the boundary, same sequence numbers) to a new
//!    file with its marks, and verify it by reopening;
//!    The manifest also carries the **retry floors** (L-COMP-1): per scope the highest folded
//!    confirmed sequence, so that a retry of a folded id is refused instead of re-applied;
//! 4. publish the new manifest atomically (the single commit point);
//! 5. only then delete the old snapshot and journal (garbage collection, repeatable).
//!
//! A crash at any step leaves the old manifest or the new one valid, and every file the valid
//! manifest names is whole. Leftovers are unnamed garbage that [`open_world`] removes.
//! Compaction needs exclusive access to the storage (no live journal handle on the old file);
//! that is a stated limit, not enforced here.

use std::collections::HashMap;

use cube_core::{AssetView, OperationId};
use cube_editing_api::{JournalEntry, JournalPhase, JournalSeq, OperationJournal};

use crate::codec::{Limits, Manifest, encode_journal_entry};
use crate::journal::{FileJournal, JournalOpenError, OpenReport};
use crate::manifest::{
    ManifestError, collect_garbage, has_world_files, load_manifest, publish_manifest,
};
use crate::snapshot::{SnapshotError, load_snapshot, write_snapshot};
use crate::store::{Storage, StoreError};

/// File name of the journal whose first record has sequence number `first_seq`.
pub fn journal_file(first_seq: u64) -> String {
    format!("journal-{first_seq:020}")
}

/// Why a world could not be created, opened or compacted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldError {
    /// The storage failed.
    Store(StoreError),
    /// The manifest is damaged or does not continue.
    Manifest(ManifestError),
    /// The snapshot the manifest names is not usable.
    Snapshot(SnapshotError),
    /// The journal could not be opened (fail closed; includes a lost acknowledged record).
    Journal(JournalOpenError),
    /// There is no world in this storage.
    NoWorld,
    /// A world already exists.
    AlreadyExists,
    /// World files exist but the manifest does not: refused rather than guessed.
    ManifestLost,
    /// The manifest contradicts itself (snapshot position versus journal start).
    Inconsistent,
    /// More scopes than the bounded retry-floor table may hold.
    TooManyScopes,
    /// The compaction boundary is outside the journal or not beyond the current snapshot.
    BadBoundary,
    /// An operation begun at or before the boundary is not closed at or before it.
    PendingAtBoundary(OperationId),
    /// The freshly written journal does not read back as written.
    VerifyFailed,
}

impl std::fmt::Display for WorldError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Store(e) => e.fmt(f),
            Self::Manifest(e) => e.fmt(f),
            Self::Snapshot(e) => e.fmt(f),
            Self::Journal(e) => e.fmt(f),
            Self::NoWorld => f.write_str("no world in this storage"),
            Self::AlreadyExists => f.write_str("a world already exists"),
            Self::ManifestLost => f.write_str("world files exist but the manifest is missing"),
            Self::Inconsistent => f.write_str("manifest is inconsistent"),
            Self::TooManyScopes => f.write_str("too many operation scopes for the retry floors"),
            Self::BadBoundary => f.write_str("compaction boundary is not inside the journal"),
            Self::PendingAtBoundary(id) => write!(f, "operation {id:?} is open at the boundary"),
            Self::VerifyFailed => f.write_str("new journal does not read back as written"),
        }
    }
}

impl std::error::Error for WorldError {}

impl From<StoreError> for WorldError {
    fn from(e: StoreError) -> Self {
        Self::Store(e)
    }
}

/// Creates an empty world (no snapshot, empty journal) by publishing the first manifest.
pub fn create_world(
    s: &mut dyn Storage,
    generator_version: u32,
    limits: &Limits,
) -> Result<(), WorldError> {
    if load_manifest(s, limits)
        .map_err(WorldError::Manifest)?
        .is_some()
    {
        return Err(WorldError::AlreadyExists);
    }
    if has_world_files(s)? {
        return Err(WorldError::ManifestLost);
    }
    let m = Manifest {
        generation: 1,
        generator_version,
        snapshot: None,
        journal: journal_file(1),
        journal_first_seq: 1,
        retry_floors: Vec::new(),
    };
    publish_manifest(s, &m, limits).map_err(WorldError::Manifest)
}

/// A world opened for recovery.
pub struct OpenedWorld<S: Storage + Send> {
    /// The manifest in force.
    pub manifest: Manifest,
    /// The verified snapshot, if any. Replay the journal records after
    /// `manifest.snapshot.journal_seq` onto it.
    pub snapshot: Option<AssetView>,
    /// The journal (every record in it is after the snapshot).
    pub journal: FileJournal<S>,
    /// What opening the journal found.
    pub journal_report: OpenReport,
    /// Unnamed files that were removed.
    pub garbage_removed: Vec<String>,
}

/// Opens the world: manifest, verified snapshot, journal. Garbage left by an interrupted
/// compaction is removed. Anything damaged is an error; nothing is repaired silently.
pub fn open_world<S: Storage + Send>(
    mut storage: S,
    limits: &Limits,
) -> Result<OpenedWorld<S>, WorldError> {
    let manifest = match load_manifest(&storage, limits).map_err(WorldError::Manifest)? {
        Some(m) => m,
        None if has_world_files(&storage)? => return Err(WorldError::ManifestLost),
        None => return Err(WorldError::NoWorld),
    };
    let expected_first = manifest.snapshot.as_ref().map_or(1, |r| r.journal_seq + 1);
    if manifest.journal_first_seq != expected_first {
        return Err(WorldError::Inconsistent);
    }
    let snapshot = match &manifest.snapshot {
        Some(r) => Some(load_snapshot(&storage, r, limits).map_err(WorldError::Snapshot)?),
        None => None,
    };
    let garbage_removed = collect_garbage(&mut storage, &manifest)?;
    let (journal, journal_report) = FileJournal::open_at(
        storage,
        &manifest.journal,
        manifest.journal_first_seq,
        *limits,
    )
    .map_err(WorldError::Journal)?;
    Ok(OpenedWorld {
        manifest,
        snapshot,
        journal,
        journal_report,
        garbage_removed,
    })
}

/// What a compaction did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompactReport {
    /// Generation of the manifest that was published.
    pub generation: u64,
    /// Last journal sequence number contained in the new snapshot.
    pub snapshot_seq: u64,
    /// Journal records kept after the boundary.
    pub kept_records: usize,
    /// Old files that were removed.
    pub removed: Vec<String>,
}

/// Most scopes the retry floors may track (bounded disk and memory cost).
pub const MAX_RETRY_SCOPES: usize = 65_536;

/// Merges the floors of confirmed operations in `folded` into `floors` (L-COMP-1).
fn merge_floors(
    floors: &[(u64, u64)],
    folded: &[JournalEntry],
) -> Result<Vec<(u64, u64)>, WorldError> {
    let mut map: std::collections::BTreeMap<u64, u64> = floors.iter().copied().collect();
    for e in folded.iter().filter(|e| e.phase == JournalPhase::Confirmed) {
        let ids = e.members.iter().map(|(id, _)| *id).chain([e.operation]);
        for id in ids {
            let f = map.entry(id.scope()).or_insert(0);
            *f = (*f).max(id.sequence());
        }
    }
    if map.len() > MAX_RETRY_SCOPES {
        return Err(WorldError::TooManyScopes);
    }
    Ok(map.into_iter().collect())
}

fn pending_at(entries: &[JournalEntry]) -> Option<OperationId> {
    let mut last: HashMap<OperationId, JournalPhase> = HashMap::new();
    let mut order = Vec::new();
    for e in entries {
        if last.insert(e.operation, e.phase).is_none() {
            order.push(e.operation);
        }
    }
    order
        .into_iter()
        .find(|id| last[id] == JournalPhase::Intent)
}

/// Folds the journal up to `upto` (inclusive) into a new snapshot. `view` must be the state
/// after applying exactly the confirmed changes up to `upto` to the current snapshot (the
/// caller's duty; the hash is recorded, not recomputed from the journal). Steps and crash
/// behaviour are described in the module documentation.
pub fn compact<S: Storage + Send>(
    storage: &mut S,
    view: &AssetView,
    upto: u64,
    limits: &Limits,
) -> Result<CompactReport, WorldError> {
    let m = load_manifest(&*storage, limits)
        .map_err(WorldError::Manifest)?
        .ok_or(WorldError::NoWorld)?;
    let (old, _) = FileJournal::open_at(&mut *storage, &m.journal, m.journal_first_seq, *limits)
        .map_err(WorldError::Journal)?;
    let entries = old.entries();
    drop(old);
    let last = m.journal_first_seq - 1 + entries.len() as u64;
    if upto < m.journal_first_seq || upto > last {
        return Err(WorldError::BadBoundary);
    }
    let split = (upto - m.journal_first_seq + 1) as usize;
    if let Some(id) = pending_at(&entries[..split]) {
        return Err(WorldError::PendingAtBoundary(id));
    }
    let retry_floors = merge_floors(&m.retry_floors, &entries[..split])?;
    let snap = write_snapshot(storage, view, upto, limits).map_err(WorldError::Snapshot)?;

    let name = journal_file(upto + 1);
    for n in [name.clone(), format!("{name}.mark")] {
        if storage.read(&n)?.is_some() {
            storage.remove(&n)?; // leftover of an interrupted earlier attempt: unnamed garbage
        }
    }
    {
        let (new, _) = FileJournal::open_at(&mut *storage, &name, upto + 1, *limits)
            .map_err(WorldError::Journal)?;
        for e in &entries[split..] {
            new.append(e.clone())
                .map_err(|_| WorldError::Store(StoreError::Io("journal append failed".into())))?;
        }
    }
    let (check, _) = FileJournal::open_at(&mut *storage, &name, upto + 1, *limits)
        .map_err(WorldError::Journal)?;
    let back = check
        .read_from(JournalSeq(0), usize::MAX)
        .unwrap_or_default();
    drop(check);
    let same = back.len() == entries.len() - split
        && back
            .iter()
            .zip(&entries[split..])
            .all(|(a, b)| encode_journal_entry(a) == encode_journal_entry(b));
    if !same {
        return Err(WorldError::VerifyFailed);
    }

    let next = Manifest {
        generation: m.generation + 1,
        generator_version: m.generator_version,
        snapshot: Some(snap),
        journal: name,
        journal_first_seq: upto + 1,
        retry_floors,
    };
    publish_manifest(storage, &next, limits).map_err(WorldError::Manifest)?;
    let removed = collect_garbage(storage, &next)?;
    Ok(CompactReport {
        generation: next.generation,
        snapshot_seq: upto,
        kept_records: entries.len() - split,
        removed,
    })
}
