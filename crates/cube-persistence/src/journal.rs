//! File-backed operation journal (PERS-04, P-03 slice; EXPERIMENTAL record format).
//!
//! Two append-only files: the journal itself (framed, checksummed records) and a small
//! **acknowledgement mark file** (`<name>.mark`, 16-byte checksummed records, one per
//! acknowledged journal record).
//!
//! * **Acknowledged means durable and provable.** `append` writes the record and `fsync`s it,
//!   then appends a mark for its sequence number and `fsync`s that, and only then returns `Ok`.
//!   So every acknowledged record has a mark, and at most one record (the one in flight at a
//!   crash) can be durable without a mark.
//! * **Torn tail versus damaged acknowledged record.** On open the valid prefix of the journal
//!   is read and compared with the highest valid mark `W`. Bytes after the prefix are discarded
//!   only if the prefix still reaches `W`: they are then provably *unacknowledged* (a torn or
//!   rotted in-flight record). If the prefix is shorter than `W` (a record that was
//!   acknowledged is damaged, cut short or missing, including the last one), opening fails
//!   with [`JournalOpenError::AcknowledgedLoss`]: the journal is **fail-closed** and never
//!   claims a successful recovery.
//! * More than one unmarked record, or marks that go backwards, are refused too
//!   ([`JournalOpenError::UnmarkedRecords`], [`JournalOpenError::BadMarks`]); a journal written
//!   before marks existed can only be taken over explicitly with
//!   [`FileJournal::open_adopting_unmarked`].
//! * Gapless sequence, damage in the middle refused, unknown schema refused (never a silent
//!   downgrade), size limits, and a failure poisons the journal (state of the record unknown).
//!
//! Limits (stated, not hidden): the marks and the journal are two files; if both lose the same
//! acknowledged record (for example one disk failure destroying the tail of both) it cannot be
//! detected. Nothing here claims platform durability beyond what `fsync` gives (DG-2).

use std::sync::{Mutex, PoisonError};

use cube_editing_api::{EditingError, JournalEntry, JournalSeq, OperationJournal};

use crate::codec::{DecodeError, Limits, MAGIC, crc32, decode_journal_entry, encode_journal_entry};
use crate::store::{Storage, StoreError};

const HEADER: usize = 4 + 2 + 1 + 4;
const MARK_MAGIC: [u8; 4] = *b"CMRK";
const MARK_LEN: usize = 16;

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
    /// Records that were acknowledged are not readable: the journal is fail-closed.
    AcknowledgedLoss {
        /// Highest acknowledged sequence number (from the marks).
        acknowledged: u64,
        /// Highest sequence number that could be read.
        readable: u64,
    },
    /// More records than one lack a mark: marks are missing or damaged (or the journal is from
    /// before marks existed); use [`FileJournal::open_adopting_unmarked`] only on purpose.
    UnmarkedRecords {
        /// How many records have no mark.
        unmarked: u64,
    },
    /// The mark file is inconsistent (marks go backwards or repeat).
    BadMarks,
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
            Self::AcknowledgedLoss {
                acknowledged,
                readable,
            } => write!(
                f,
                "records up to {acknowledged} were acknowledged but only {readable} are readable"
            ),
            Self::UnmarkedRecords { unmarked } => {
                write!(f, "{unmarked} records have no acknowledgement mark")
            }
            Self::BadMarks => f.write_str("acknowledgement marks are inconsistent"),
        }
    }
}

impl std::error::Error for JournalOpenError {}

/// What opening found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenReport {
    /// Valid records read.
    pub records: usize,
    /// Bytes of an unacknowledged torn or damaged tail that were truncated.
    pub truncated_bytes: usize,
    /// Highest acknowledged sequence number (0 for none).
    pub acknowledged: u64,
}

struct Inner<S> {
    storage: S,
    name: String,
    mark_name: String,
    entries: Vec<JournalEntry>,
    next_seq: u64,
    marks: u64,
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

fn mark_bytes(seq: u64) -> [u8; MARK_LEN] {
    let mut b = [0u8; MARK_LEN];
    b[..4].copy_from_slice(&MARK_MAGIC);
    b[4..12].copy_from_slice(&seq.to_le_bytes());
    let c = crc32(&b[..12]);
    b[12..].copy_from_slice(&c.to_le_bytes());
    b
}

/// The valid marks in order; parsing stops at the first invalid one (a torn last mark is simply
/// an acknowledgement that never completed).
fn parse_marks(bytes: &[u8]) -> Vec<u64> {
    let mut out = Vec::new();
    for c in bytes.chunks(MARK_LEN) {
        if c.len() != MARK_LEN || c[..4] != MARK_MAGIC || crc32(&c[..12]).to_le_bytes() != c[12..] {
            break;
        }
        out.push(u64::from_le_bytes(c[4..12].try_into().expect("8 bytes")));
    }
    out
}

impl<S: Storage + Send> FileJournal<S> {
    /// Opens (or creates) the journal `name` whose first record has sequence number 1.
    pub fn open(
        storage: S,
        name: &str,
        limits: Limits,
    ) -> Result<(Self, OpenReport), JournalOpenError> {
        Self::open_inner(storage, name, 1, limits, false)
    }

    /// Opens a journal whose first record has sequence number `first_seq` (a journal that was
    /// rewritten by compaction keeps the original numbers).
    pub fn open_at(
        storage: S,
        name: &str,
        first_seq: u64,
        limits: Limits,
    ) -> Result<(Self, OpenReport), JournalOpenError> {
        Self::open_inner(storage, name, first_seq, limits, false)
    }

    /// Explicit recovery mode: takes over records that have no marks (a journal from before
    /// marks existed, or lost marks) by marking them. Everything else is still refused.
    pub fn open_adopting_unmarked(
        storage: S,
        name: &str,
        first_seq: u64,
        limits: Limits,
    ) -> Result<(Self, OpenReport), JournalOpenError> {
        Self::open_inner(storage, name, first_seq, limits, true)
    }

    fn open_inner(
        mut storage: S,
        name: &str,
        first_seq: u64,
        limits: Limits,
        adopt: bool,
    ) -> Result<(Self, OpenReport), JournalOpenError> {
        let mark_name = format!("{name}.mark");
        let store = JournalOpenError::Store;
        let bytes = storage.read(name).map_err(store)?.unwrap_or_default();
        let mark_bytes_all = storage.read(&mark_name).map_err(store)?.unwrap_or_default();

        let mut entries: Vec<JournalEntry> = Vec::new();
        let mut at = 0usize;
        let mut next = first_seq;
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
        // Damage in the middle: a valid record somewhere after the damaged bytes.
        if (1..tail.len())
            .any(|i| tail[i..].starts_with(&MAGIC) && valid_frame_at(&tail[i..], &limits))
        {
            return Err(JournalOpenError::CorruptInMiddle { offset: at });
        }

        let marks = parse_marks(&mark_bytes_all);
        if marks.windows(2).any(|w| w[1] <= w[0]) || marks.first().is_some_and(|m| *m < first_seq) {
            return Err(JournalOpenError::BadMarks);
        }
        let acknowledged = marks.last().copied().unwrap_or(first_seq - 1);
        let readable = next - 1;
        if readable < acknowledged {
            return Err(JournalOpenError::AcknowledgedLoss {
                acknowledged,
                readable,
            });
        }
        let unmarked = readable - acknowledged;
        let mut mark_count = marks.len() as u64;
        if unmarked > 1 {
            if !adopt {
                return Err(JournalOpenError::UnmarkedRecords { unmarked });
            }
            // Explicit takeover: mark the unmarked records one by one.
            for seq in acknowledged + 1..=readable {
                storage
                    .append(&mark_name, &mark_bytes(seq))
                    .map_err(store)?;
                mark_count += 1;
            }
            storage.fsync(&mark_name).map_err(store)?;
            storage.fsync_dir().map_err(store)?;
        }
        // Only now is the tail provably unacknowledged: cut it.
        if !tail.is_empty() {
            storage.truncate(name, at as u64).map_err(store)?;
            storage.fsync(name).map_err(store)?;
        }
        let report = OpenReport {
            records: entries.len(),
            truncated_bytes: tail.len(),
            acknowledged: if unmarked > 1 { readable } else { acknowledged },
        };
        Ok((
            Self {
                inner: Mutex::new(Inner {
                    storage,
                    name: name.to_owned(),
                    mark_name,
                    entries,
                    next_seq: next,
                    marks: mark_count,
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
        let first_mark = g.marks == 0;
        let (name, mark_name) = (g.name.clone(), g.mark_name.clone());
        let mark = mark_bytes(entry.seq.0);
        let written = g
            .storage
            .append(&name, &bytes)
            .and_then(|()| g.storage.fsync(&name))
            .and_then(|()| if first { g.storage.fsync_dir() } else { Ok(()) })
            .and_then(|()| g.storage.append(&mark_name, &mark))
            .and_then(|()| g.storage.fsync(&mark_name))
            .and_then(|()| {
                if first_mark {
                    g.storage.fsync_dir()
                } else {
                    Ok(())
                }
            });
        if written.is_err() {
            // The record may or may not have reached the disk: never claim either.
            g.poisoned = true;
            return Err(EditingError::PersistenceFailure);
        }
        g.next_seq += 1;
        g.marks += 1;
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
