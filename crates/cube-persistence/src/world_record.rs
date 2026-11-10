//! The world record (PERS-08, EXPERIMENTAL format until GOV-10): identity of a world, created once.
//!
//! A world is `record` (who it is and which untouched base it comes from) plus `manifest`
//! (which snapshot and journal are current). The record holds the seed and the generator version
//! and nothing else about the world; the base is regenerated, user changes come from the
//! snapshot and the journal. It never duplicates Core data.
//!
//! Safety rules:
//!
//! * **No silent generator change.** The manifest repeats the generator version; opening a world
//!   whose record and manifest disagree is an error, and so is a version the running build
//!   cannot reproduce (`GeneratorUnavailable`): the caller supplies the predicate, persistence
//!   does not know the generators.
//! * **Unknown structure versions are refused**, never read as a known one.
//! * **Creation order:** the record is written first (atomically), the manifest second. A crash
//!   in between leaves a record without a manifest, which is an incomplete creation that
//!   `create_world_with_record` replaces; a manifest without a record is a damaged world and is
//!   refused.

use cube_core::GeneratorVersion;

use crate::codec::{DecodeError, Limits, WorldRecord, decode_world_record, encode_world_record};
use crate::compact::{OpenedWorld, WorldError, create_world, open_world};
use crate::store::{Storage, StoreError, save_atomic};

/// File name of the record.
pub const RECORD_FILE: &str = "world.record";

/// Why a world with a record could not be created or opened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordError {
    /// The storage failed.
    Store(StoreError),
    /// The record is damaged or has an unknown structure version.
    Corrupt(DecodeError),
    /// The manifest exists but the record does not.
    RecordMissing,
    /// The record and the manifest name different generator versions.
    GeneratorMismatch {
        /// Version in the record.
        record: GeneratorVersion,
        /// Version in the manifest.
        manifest: u32,
    },
    /// This build cannot reproduce the generator version of the world.
    GeneratorUnavailable(GeneratorVersion),
    /// The world files themselves failed.
    World(WorldError),
}

impl std::fmt::Display for RecordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Store(e) => e.fmt(f),
            Self::Corrupt(e) => write!(f, "world record is damaged: {e}"),
            Self::RecordMissing => f.write_str("the world has a manifest but no world record"),
            Self::GeneratorMismatch { record, manifest } => write!(
                f,
                "record says generator {}, manifest says {manifest}",
                record.raw()
            ),
            Self::GeneratorUnavailable(v) => {
                write!(
                    f,
                    "generator version {} is not available in this build",
                    v.raw()
                )
            }
            Self::World(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for RecordError {}

impl From<WorldError> for RecordError {
    fn from(e: WorldError) -> Self {
        Self::World(e)
    }
}

/// Creates a world: record first, then the first manifest. Fails with `AlreadyExists` if a
/// manifest exists (the existing world is never touched).
pub fn create_world_with_record(
    s: &mut dyn Storage,
    record: &WorldRecord,
    limits: &Limits,
) -> Result<(), RecordError> {
    if crate::manifest::load_manifest(s, limits)
        .map_err(|e| RecordError::World(WorldError::Manifest(e)))?
        .is_some()
    {
        return Err(RecordError::World(WorldError::AlreadyExists));
    }
    save_atomic(s, RECORD_FILE, &encode_world_record(record)).map_err(RecordError::Store)?;
    create_world(s, record.generator_version.raw(), limits)?;
    Ok(())
}

/// Reads the record.
pub fn load_world_record(
    s: &dyn Storage,
    limits: &Limits,
) -> Result<Option<WorldRecord>, RecordError> {
    match s.read(RECORD_FILE).map_err(RecordError::Store)? {
        None => Ok(None),
        Some(b) => decode_world_record(&b, limits)
            .map(Some)
            .map_err(RecordError::Corrupt),
    }
}

/// Opens a world with its record. `runnable` says whether this build can reproduce a generator
/// version; the generator is never substituted.
pub fn open_world_with_record<S: Storage + Send>(
    storage: S,
    limits: &Limits,
    runnable: &dyn Fn(GeneratorVersion) -> bool,
) -> Result<(WorldRecord, OpenedWorld<S>), RecordError> {
    let record = load_world_record(&storage, limits)?.ok_or(RecordError::RecordMissing)?;
    let world = open_world(storage, limits)?;
    if world.manifest.generator_version != record.generator_version.raw() {
        return Err(RecordError::GeneratorMismatch {
            record: record.generator_version,
            manifest: world.manifest.generator_version,
        });
    }
    if !runnable(record.generator_version) {
        return Err(RecordError::GeneratorUnavailable(record.generator_version));
    }
    Ok((record, world))
}
