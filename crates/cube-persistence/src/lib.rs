//! Cube persistence foundation (start of PS-10): an experimental versioned codec, the atomic
//! snapshot file protocol and journal reconciliation.
//!
//! **Nothing here is an approved format (GOV-10 open) or a durability claim for any real
//! platform.** Core does not depend on this crate; it only exports plain views.

pub mod codec;
pub mod recovery;
pub mod store;

pub use codec::{
    BatchCodec, DecodeError, Limits, SCHEMA, decode_asset, decode_batch, decode_journal_entry,
    encode_asset, encode_batch, encode_journal_entry,
};
pub use recovery::{ReplayReport, Resolution, reconcile, replay_confirmed};
pub use store::{DirStorage, LoadError, Storage, StoreError, load_asset, save_asset, save_atomic};
