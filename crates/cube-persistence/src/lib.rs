//! Cube persistence foundation (start of PS-10): an experimental versioned codec, the atomic
//! snapshot file protocol and journal reconciliation.
//!
//! **Nothing here is an approved format (GOV-10 open) or a durability claim for any real
//! platform.** Core does not depend on this crate; it only exports plain views.

pub mod codec;
pub mod compact;
pub mod journal;
pub mod manifest;
pub mod recovery;
pub mod snapshot;
pub mod store;
pub mod world_record;

pub use codec::{
    BatchCodec, DecodeError, Limits, SCHEMA, decode_asset, decode_batch, decode_journal_entry,
    encode_asset, encode_batch, encode_journal_entry,
};
pub use codec::{Manifest, STRUCTURE_VERSION, SnapshotRef, WorldRecord};
pub use compact::{
    CompactReport, OpenedWorld, WorldError, compact, create_world, journal_file, open_world,
};
pub use journal::{FileJournal, JournalOpenError, OpenReport};
pub use manifest::{ManifestError, collect_garbage, load_manifest, publish_manifest};
pub use recovery::{ReplayReport, Resolution, reconcile, replay_confirmed};
pub use snapshot::{SnapshotError, load_snapshot, snapshot_file, write_snapshot};
pub use store::{
    DirStorage, LoadError, Storage, StoreError, WorldLock, load_asset, save_asset, save_atomic,
};
pub use world_record::{
    RECORD_FILE, RecordError, create_world_with_record, load_world_record, open_world_with_record,
};
