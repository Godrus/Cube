//! File-backed operation journal (PERS-04, P-03 slice; EXPERIMENTAL record format).
//!
//! One append-only file of framed, checksummed records (`codec::encode_journal_entry`). The
//! contract it gives:
//!
//! * **Acknowledged means durable.** `append` returns `Ok` only after the record was written and
//!   `fsync`ed (and, for the first record, the directory entry was made durable). Any failure
//!   *poisons* the journal: no further append succeeds until the file is reopened and repaired,
//!   because after a failed `fsync` the state of the record is unknown.
//! * **Gapless, monotonic sequence** assigned by the journal, verified on open.
//! * **Torn tails are repaired, damage in the middle is refused.** On open the valid prefix is
//!   read. Bytes after it are either a torn tail (a crash during a write: nothing valid follows)
//!   and are truncated, or they contain a later valid record, which means damage in the middle
//!   of the log: opening fails with [`JournalOpenError::CorruptInMiddle`] instead of silently
//!   dropping records that were acknowledged.
//! * Every record is bounded by the codec limits; oversize records are refused.
//!
//! Nothing here claims platform durability beyond what `fsync` gives (DG-2).

use std::sync::{Mutex, PoisonError};

use cube_editing_api::{EditingError, JournalEntry, JournalSeq, OperationJournal};

use crate::codec::{DecodeError, Limits, MAGIC, decode_journal_entry, encode_journal_entry};
use crate::store::{Storage, StoreError};

const HEADER: usize = 4 + 2 + 1 + 4;

/// Why a journal could not be opened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JournalOpenError {
    /// The storage failed.
    Store(StoreError),
    /// A valid record follows damaged bytes at `offset`: the log is damaged in the middle and
    /// is not repaired automatically.
    CorruptInMiddle {
        /// Byte offset where the valid prefix ends.
        offset: usize,
    },
    /// A record has the wrong sequence number (a gap or a repeat).
    Sequence {
        /// Expected sequence number.
        expected: u64,
        /// Found sequence number.
        found: u64,
    },
    /// The first record uses a schema this build does not know (not a torn tail).
    UnsupportedSchema(u16),
}

impl std::fmt::Display for JournalOpenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Store(e) => e.fmt(f),
            Self::CorruptInMiddle { offset } => {
                write!(f, "journal damaged in the middle at byte {offset}")
            }
            Self::Sequence { expected, found } => {
                write!(f, "journal sequence {found}, expected {expected}")
            }
            Self::UnsupportedSchema(s) => write!(f, "unsupported journal schema {s:#06x}"),
        }
    }
}

impl std::error::Error for JournalOpenError {}

/// What opening found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenReport {
    /// Valid records read.
    pub records: usize,
    /// Bytes of a torn tail that were truncated (0 for a clean file).
    pub truncated_bytes: usize,
}

struct Inner<S> {
    storage: S,
    name: String,
    entries: Vec<JournalEntry>,
    next_seq: u64,
    poisoned: bool,
    limits: Limits,
}

/// The journal over any [`Storage`].
pub struct FileJournal<S: Storage + Send> {
    inner: Mutex<Inner<S>>,
}

impl<S: Storage + Send> std::fmt::Debug for FileJournal<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileJournal").finish_non_exhaustive()
    }
}

/// Length of the frame starting at the beginning of `b`, if its header is complete and sane.
fn frame_len(b: &[u8], limits: &Limits) -> Option<usize> {
    if b.len() < HEADER || b[..4] != MAGIC {
        return None;
    }
    let len = u32::from_le_bytes([b[7], b[8], b[9], b[10]]) as usize;
    (len <= limits.max_body).then_some(HEADER + len + 4)
}

fn valid_frame_at(b: &[u8], limits: &Limits) -> bool {
    frame_len(b, limits)
        .is_some_and(|n| n <= b.len() && decode_journal_entry(&b[..n], limits).is_ok())
}

impl<S: Storage + Send> FileJournal<S> {
    /// Opens (or creates) the journal `name`, repairing a torn tail.
    pub fn open(
        mut storage: S,
        name: &str,
        limits: Limits,
    ) -> Result<(Self, OpenReport), JournalOpenError> {
        let bytes = storage
            .read(name)
            .map_err(JournalOpenError::Store)?
            .unwrap_or_default();
        let mut entries: Vec<JournalEntry> = Vec::new();
        let mut at = 0usize;
        let mut next = 1u64;
        while at < bytes.len() {
            let rest = &bytes[at..];
            let Some(n) = frame_len(rest, &limits).filter(|n| *n <= rest.len()) else {
                break;
            };
            match decode_journal_entry(&rest[..n], &limits) {
                Ok(e) => {
                    if e.seq.0 != next {
                        return Err(JournalOpenError::Sequence {
                            expected: next,
                            found: e.seq.0,
                        });
                    }
                    next += 1;
                    entries.push(e);
                    at += n;
                }
                Err(DecodeError::UnsupportedSchema(s)) if entries.is_empty() && at == 0 => {
                    return Err(JournalOpenError::UnsupportedSchema(s));
                }
                Err(_) => break,
            }
        }
        let tail = &bytes[at..];
        // A torn tail has nothing valid after its start; a valid record later means damage.
        if (1..tail.len())
            .any(|i| tail[i..].starts_with(&MAGIC) && valid_frame_at(&tail[i..], &limits))
        {
            return Err(JournalOpenError::CorruptInMiddle { offset: at });
        }
        if !tail.is_empty() {
            storage
                .truncate(name, at as u64)
                .map_err(JournalOpenError::Store)?;
            storage.fsync(name).map_err(JournalOpenError::Store)?;
        }
        let report = OpenReport {
            records: entries.len(),
            truncated_bytes: tail.len(),
        };
        Ok((
            Self {
                inner: Mutex::new(Inner {
                    storage,
                    name: name.to_owned(),
                    entries,
                    next_seq: next,
                    poisoned: false,
                    limits,
                }),
            },
            report,
        ))
    }

    /// `true` after a storage failure: appends are refused until the journal is reopened.
    pub fn is_poisoned(&self) -> bool {
        self.inner
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .poisoned
    }

    /// All records (a copy), in order.
    pub fn entries(&self) -> Vec<JournalEntry> {
        self.inner
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .entries
            .clone()
    }

    /// Hands the storage back (for tests that reopen the same files).
    pub fn into_storage(self) -> S {
        self.inner
            .into_inner()
            .unwrap_or_else(PoisonError::into_inner)
            .storage
    }
}

impl<S: Storage + Send> OperationJournal for FileJournal<S> {
    fn append(&self, mut entry: JournalEntry) -> Result<JournalSeq, EditingError> {
        let mut g = self.inner.lock().unwrap_or_else(PoisonError::into_inner);
        if g.poisoned {
            return Err(EditingError::PersistenceFailure);
        }
        entry.seq = JournalSeq(g.next_seq);
        let bytes = encode_journal_entry(&entry);
        if bytes.len() > HEADER + g.limits.max_body + 4 {
            return Err(EditingError::PersistenceFailure);
        }
        let first = g.entries.is_empty();
        let name = g.name.clone();
        let written = g
            .storage
            .append(&name, &bytes)
            .and_then(|()| g.storage.fsync(&name))
            .and_then(|()| if first { g.storage.fsync_dir() } else { Ok(()) });
        if written.is_err() {
            // The record may or may not have reached the disk: never claim either.
            g.poisoned = true;
            return Err(EditingError::PersistenceFailure);
        }
        g.next_seq += 1;
        let seq = entry.seq;
        g.entries.push(entry);
        Ok(seq)
    }

    fn read_from(&self, from: JournalSeq, limit: usize) -> Result<Vec<JournalEntry>, EditingError> {
        let g = self.inner.lock().unwrap_or_else(PoisonError::into_inner);
        Ok(g.entries
            .iter()
            .filter(|e| e.seq >= from)
            .take(limit)
            .cloned()
            .collect())
    }
}
