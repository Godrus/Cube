//! Versioned, checksummed record codec (PERS-01, P-02 slice).
//!
//! **EXPERIMENTAL.** The schema number carries the experimental bit (`0x8000`): this is an
//! internal working format, not the approved persistence format (GOV-10 is open), and nothing
//! may rely on it as a stable public format. Every record is
//!
//! ```text
//! magic "CUBP" | schema u16 | kind u8 | body_len u32 | body | crc32(magic..body) u32
//! ```
//!
//! (all little endian). Decoding validates the header, the declared length against the actual
//! input and against [`Limits`] *before* anything is allocated, the checksum, and then every
//! field of the body; any failure is a typed [`DecodeError`], never a panic. Encoding is
//! deterministic: equal values give equal bytes. Core knows nothing of this module.

use cube_core::{
    AssetId, AssetView, BrickBounds, BrickCoord, CellChange, CellPayload, CellView, CellWrite,
    Channel, ChannelId, ChannelSet, CommitBatch, Epoch, ExpectedRevision, GeometryTransform,
    LayerId, LayerKind, LayerView, LocalPos, MeshPage, OperationId, Revision, RevisionToken,
    Rotation, WriteSet,
};
use cube_editing_api::{ActorId, BatchEncoder, DeltaId, JournalEntry, JournalPhase, JournalSeq};

/// Magic bytes of every record.
pub const MAGIC: [u8; 4] = *b"CUBP";
/// Current schema: version 1 with the experimental bit set.
pub const SCHEMA: u16 = 0x8001;

const KIND_ASSET: u8 = 1;
const KIND_JOURNAL: u8 = 2;
const KIND_BATCH: u8 = 3;
const KIND_MANIFEST: u8 = 4;
const KIND_WORLD_RECORD: u8 = 5;
const KIND_BRICK: u8 = 6;
const HEADER: usize = 4 + 2 + 1 + 4;

/// Bounds applied to untrusted input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    /// Largest body accepted.
    pub max_body: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_body: 256 * 1024 * 1024,
        }
    }
}

/// Why a record could not be decoded.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DecodeError {
    /// The input is shorter than a header or than the length it declares.
    Truncated,
    /// The magic bytes are wrong: this is not a record.
    BadMagic,
    /// The schema is not one this build understands.
    UnsupportedSchema(u16),
    /// The record is of another kind than the one asked for.
    WrongKind {
        /// Kind found.
        found: u8,
    },
    /// The body is larger than the limit.
    TooLarge,
    /// The checksum does not match: the record is corrupt.
    ChecksumMismatch,
    /// Bytes follow the record.
    TrailingBytes,
    /// A field has a value the format does not allow.
    Invalid(&'static str),
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Truncated => f.write_str("record is truncated"),
            Self::BadMagic => f.write_str("not a Cube record"),
            Self::UnsupportedSchema(s) => write!(f, "unsupported schema {s:#06x}"),
            Self::WrongKind { found } => write!(f, "record has kind {found}"),
            Self::TooLarge => f.write_str("record exceeds the size limit"),
            Self::ChecksumMismatch => f.write_str("checksum mismatch"),
            Self::TrailingBytes => f.write_str("bytes after the record"),
            Self::Invalid(w) => write!(f, "invalid field: {w}"),
        }
    }
}

impl std::error::Error for DecodeError {}

/// CRC-32 (IEEE 802.3), bitwise.
pub fn crc32(bytes: &[u8]) -> u32 {
    let mut c = 0xFFFF_FFFFu32;
    for b in bytes {
        c ^= u32::from(*b);
        for _ in 0..8 {
            c = if c & 1 == 1 {
                (c >> 1) ^ 0xEDB8_8320
            } else {
                c >> 1
            };
        }
    }
    !c
}

struct W(Vec<u8>);

impl W {
    fn u8(&mut self, v: u8) {
        self.0.push(v);
    }
    fn u16(&mut self, v: u16) {
        self.0.extend(v.to_le_bytes());
    }
    fn u32(&mut self, v: u32) {
        self.0.extend(v.to_le_bytes());
    }
    fn u64(&mut self, v: u64) {
        self.0.extend(v.to_le_bytes());
    }
    fn i64(&mut self, v: i64) {
        self.0.extend(v.to_le_bytes());
    }
}

struct R<'a>(&'a [u8]);

impl R<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8], DecodeError> {
        if self.0.len() < n {
            return Err(DecodeError::Truncated);
        }
        let (a, b) = self.0.split_at(n);
        self.0 = b;
        Ok(a)
    }
    fn u8(&mut self) -> Result<u8, DecodeError> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, DecodeError> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().expect("2")))
    }
    fn u32(&mut self) -> Result<u32, DecodeError> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().expect("4")))
    }
    fn u64(&mut self) -> Result<u64, DecodeError> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().expect("8")))
    }
    fn i64(&mut self) -> Result<i64, DecodeError> {
        Ok(i64::from_le_bytes(self.take(8)?.try_into().expect("8")))
    }
    fn done(&self) -> Result<(), DecodeError> {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err(DecodeError::TrailingBytes)
        }
    }
}

fn frame(kind: u8, body: Vec<u8>) -> Vec<u8> {
    let mut out = Vec::with_capacity(HEADER + body.len() + 4);
    out.extend(MAGIC);
    out.extend(SCHEMA.to_le_bytes());
    out.push(kind);
    out.extend((body.len() as u32).to_le_bytes());
    out.extend(body);
    let c = crc32(&out);
    out.extend(c.to_le_bytes());
    out
}

/// Validates the frame and returns the body.
fn unframe(bytes: &[u8], kind: u8, limits: &Limits) -> Result<Vec<u8>, DecodeError> {
    if bytes.len() < HEADER {
        return Err(DecodeError::Truncated);
    }
    if bytes[..4] != MAGIC {
        return Err(DecodeError::BadMagic);
    }
    let schema = u16::from_le_bytes([bytes[4], bytes[5]]);
    if schema != SCHEMA {
        return Err(DecodeError::UnsupportedSchema(schema));
    }
    if bytes[6] != kind {
        return Err(DecodeError::WrongKind { found: bytes[6] });
    }
    let len = u32::from_le_bytes([bytes[7], bytes[8], bytes[9], bytes[10]]) as usize;
    if len > limits.max_body {
        return Err(DecodeError::TooLarge);
    }
    let total = HEADER + len + 4;
    if bytes.len() < total {
        return Err(DecodeError::Truncated);
    }
    if bytes.len() > total {
        return Err(DecodeError::TrailingBytes);
    }
    let want = u32::from_le_bytes(bytes[total - 4..].try_into().expect("4"));
    if crc32(&bytes[..total - 4]) != want {
        return Err(DecodeError::ChecksumMismatch);
    }
    Ok(bytes[HEADER..HEADER + len].to_vec())
}

fn kind_code(k: LayerKind) -> u8 {
    match k {
        LayerKind::Voxel => 1,
        LayerKind::Sdf => 2,
        LayerKind::Mesh => 3,
    }
}

fn channel_tag(id: ChannelId) -> (u8, u8) {
    match id {
        ChannelId::Material => (1, 0),
        ChannelId::Density => (2, 0),
        ChannelId::Attribute(n) => (3, n),
    }
}

fn write_token(w: &mut W, t: &RevisionToken) {
    use cube_core::ResourceId;
    match t.resource() {
        ResourceId::Asset(a) => {
            w.u8(1);
            w.u64(a.raw());
        }
        ResourceId::Layer(l) => {
            w.u8(2);
            w.u64(l.raw());
        }
        ResourceId::Cell(k) => {
            w.u8(3);
            w.u64(k.layer().raw());
            w.u8(k.level() as u8);
            w.i64(k.brick().x);
            w.i64(k.brick().y);
            w.i64(k.brick().z);
        }
        // Region and derived tokens are not persisted: they are recomputable (P-04 note).
        ResourceId::Region(_) | ResourceId::Derived(_) => w.u8(0),
    }
    w.u32(t.epoch().raw());
    w.u64(t.value().raw());
}

fn read_token(r: &mut R<'_>) -> Result<RevisionToken, DecodeError> {
    use cube_core::{CellKey, ResourceId};
    let res = match r.u8()? {
        1 => ResourceId::Asset(AssetId::new(r.u64()?)),
        2 => ResourceId::Layer(LayerId::new(r.u64()?)),
        3 => {
            let layer = LayerId::new(r.u64()?);
            let level = r.u8()? as i8;
            let b = BrickCoord::new(r.i64()?, r.i64()?, r.i64()?);
            ResourceId::Cell(
                CellKey::new(layer, level, b).map_err(|_| DecodeError::Invalid("cell level"))?,
            )
        }
        _ => return Err(DecodeError::Invalid("token resource")),
    };
    Ok(RevisionToken::new(
        res,
        Epoch::new(r.u32()?),
        Revision::new(r.u64()?),
    ))
}

/// Encodes an asset view (deterministic).
pub fn encode_asset(v: &AssetView) -> Vec<u8> {
    let mut w = W(Vec::new());
    w.u64(v.id.raw());
    w.u32(v.epoch.raw());
    w.u64(v.revision.raw());
    w.u32(v.layers.len() as u32);
    for l in &v.layers {
        w.u64(l.id.raw());
        w.u8(kind_code(l.kind));
        for t in l.transform.translation() {
            w.i64(t);
        }
        w.u8(l.transform.rotation().index());
        w.u8(l.transform.scale_shift() as u8);
        match &l.bounds {
            None => w.u8(0),
            Some(b) => {
                w.u8(1);
                for p in [b.min(), b.max()] {
                    w.i64(p.x);
                    w.i64(p.y);
                    w.i64(p.z);
                }
            }
        }
        w.u64(l.revision.raw());
        w.u64(l.cells.len() as u64);
        for c in &l.cells {
            w.u8(c.level as u8);
            w.i64(c.brick.x);
            w.i64(c.brick.y);
            w.i64(c.brick.z);
            w.u64(c.revision.raw());
            write_payload(&mut w, &c.payload);
        }
    }
    frame(KIND_ASSET, w.0)
}

fn write_payload(w: &mut W, p: &Option<CellPayload>) {
    match p {
        None => w.u8(0),
        Some(CellPayload::Channels(set)) => {
            w.u8(1);
            w.u8(set.len() as u8);
            for (id, ch) in set.iter() {
                let (t, n) = channel_tag(id);
                w.u8(t);
                w.u8(n);
                // MAT-01: values that fit 16 bits keep the original encodings (1 constant, 0 dense),
                // so existing files stay byte for byte valid. Wider values use the new
                // encodings 3 (constant u32) and 2 (dense u32); an older reader rejects them
                // as an unknown channel encoding, never misreads them.
                let wide = ch.value_bits() > 16;
                match ch.uniform() {
                    Some(u) if wide => {
                        w.u8(3);
                        w.u32(u);
                    }
                    Some(u) => {
                        w.u8(1);
                        w.u16(u as u16);
                    }
                    None if wide => {
                        w.u8(2);
                        for p in LocalPos::all() {
                            w.u32(ch.get(p));
                        }
                    }
                    None => {
                        w.u8(0);
                        for p in LocalPos::all() {
                            w.u16(ch.get(p) as u16);
                        }
                    }
                }
            }
        }
        Some(CellPayload::MeshPage(m)) => {
            w.u8(2);
            w.u32(m.bytes().len() as u32);
            w.0.extend_from_slice(m.bytes());
        }
    }
}

fn read_payload(r: &mut R<'_>) -> Result<Option<CellPayload>, DecodeError> {
    Ok(match r.u8()? {
        0 => None,
        1 => {
            let n = r.u8()?;
            let mut set = ChannelSet::new();
            let mut last = None;
            for _ in 0..n {
                let (cid, ch) = read_channel(r)?;
                if last.is_some_and(|l| l >= cid) {
                    return Err(DecodeError::Invalid("channels out of order or repeated"));
                }
                last = Some(cid);
                set = set.with(cid, ch);
            }
            Some(CellPayload::Channels(set))
        }
        2 => {
            let len = r.u32()? as usize;
            Some(CellPayload::MeshPage(MeshPage::new(r.take(len)?.to_vec())))
        }
        _ => return Err(DecodeError::Invalid("payload tag")),
    })
}

/// Encodes a write batch (deterministic): the unit Orchestration journals ahead of a commit.
pub fn encode_batch(b: &CommitBatch) -> Vec<u8> {
    let mut w = W(Vec::new());
    w.u32(b.sets.len() as u32);
    for set in &b.sets {
        w.u64(set.layer.raw());
        w.u32(set.writes.len() as u32);
        for x in &set.writes {
            w.u8(x.level as u8);
            w.i64(x.brick.x);
            w.i64(x.brick.y);
            w.i64(x.brick.z);
            match x.expected {
                ExpectedRevision::Absent => w.u8(0),
                ExpectedRevision::At(r) => {
                    w.u8(1);
                    w.u64(r.raw());
                }
            }
            match &x.change {
                CellChange::KnownEmpty => w.u8(0),
                CellChange::Put(p) => {
                    w.u8(1);
                    write_payload(&mut w, &Some(p.clone()));
                }
            }
        }
    }
    frame(KIND_BATCH, w.0)
}

/// Decodes a write batch.
pub fn decode_batch(bytes: &[u8], limits: &Limits) -> Result<CommitBatch, DecodeError> {
    let body = unframe(bytes, KIND_BATCH, limits)?;
    let mut r = R(&body);
    let n = r.u32()? as usize;
    if n > body.len() / 12 + 1 {
        return Err(DecodeError::Invalid("set count"));
    }
    let mut sets = Vec::with_capacity(n.min(64));
    for _ in 0..n {
        let layer = LayerId::new(r.u64()?);
        let m = r.u32()? as usize;
        if m > body.len() / 30 + 1 {
            return Err(DecodeError::Invalid("write count"));
        }
        let mut writes = Vec::with_capacity(m.min(1024));
        for _ in 0..m {
            let level = r.u8()? as i8;
            let brick = BrickCoord::new(r.i64()?, r.i64()?, r.i64()?);
            let expected = match r.u8()? {
                0 => ExpectedRevision::Absent,
                1 => ExpectedRevision::At(Revision::new(r.u64()?)),
                _ => return Err(DecodeError::Invalid("expected flag")),
            };
            let change = match r.u8()? {
                0 => CellChange::KnownEmpty,
                1 => CellChange::Put(
                    read_payload(&mut r)?.ok_or(DecodeError::Invalid("empty put payload"))?,
                ),
                _ => return Err(DecodeError::Invalid("change tag")),
            };
            writes.push(CellWrite {
                level,
                brick,
                expected,
                change,
            });
        }
        sets.push(WriteSet { layer, writes });
    }
    r.done()?;
    Ok(CommitBatch { sets })
}

/// Encodes the file of one unloaded edited brick (M1-A1b): the brick, the revision it has after
/// the reload that follows the unload, and its content.
pub fn encode_stored_brick(
    brick: BrickCoord,
    revision_after_reload: u64,
    change: &CellChange,
) -> Vec<u8> {
    let mut w = W(Vec::new());
    w.i64(brick.x);
    w.i64(brick.y);
    w.i64(brick.z);
    w.u64(revision_after_reload);
    match change {
        CellChange::KnownEmpty => w.u8(0),
        CellChange::Put(p) => {
            w.u8(1);
            write_payload(&mut w, &Some(p.clone()));
        }
    }
    frame(KIND_BRICK, w.0)
}

/// Decodes the output of `encode_stored_brick`.
pub fn decode_stored_brick(
    bytes: &[u8],
    limits: &Limits,
) -> Result<(BrickCoord, u64, CellChange), DecodeError> {
    let body = unframe(bytes, KIND_BRICK, limits)?;
    let mut r = R(&body);
    let brick = BrickCoord::new(r.i64()?, r.i64()?, r.i64()?);
    let revision = r.u64()?;
    let change = match r.u8()? {
        0 => CellChange::KnownEmpty,
        1 => {
            CellChange::Put(read_payload(&mut r)?.ok_or(DecodeError::Invalid("empty put payload"))?)
        }
        _ => return Err(DecodeError::Invalid("change tag")),
    };
    r.done()?;
    Ok((brick, revision, change))
}

/// Encoder handed to Orchestration so that its `Intent` records carry the batch.
#[derive(Debug, Default, Clone, Copy)]
pub struct BatchCodec;

impl BatchEncoder for BatchCodec {
    fn encode(&self, batch: &CommitBatch) -> Vec<u8> {
        encode_batch(batch)
    }
}

fn read_channel(r: &mut R<'_>) -> Result<(ChannelId, Channel), DecodeError> {
    let (t, n) = (r.u8()?, r.u8()?);
    let id = match t {
        1 => ChannelId::Material,
        2 => ChannelId::Density,
        3 => ChannelId::Attribute(n),
        _ => return Err(DecodeError::Invalid("channel id")),
    };
    let ch = match r.u8()? {
        1 => Channel::constant(u32::from(r.u16()?)),
        0 => {
            let raw = r.take(2 * cube_core::BRICK_VOLUME)?;
            let val = |i: usize| u32::from(u16::from_le_bytes([raw[2 * i], raw[2 * i + 1]]));
            let mut ch = Channel::constant(val(0));
            for (i, p) in LocalPos::all().enumerate() {
                if val(i) != val(0) {
                    ch.set(p, val(i));
                }
            }
            ch.compact();
            ch
        }
        // MAT-01: wide encodings (values above 0xFFFF).
        3 => Channel::constant(r.u32()?),
        2 => {
            let raw = r.take(4 * cube_core::BRICK_VOLUME)?;
            let val = |i: usize| {
                u32::from_le_bytes([raw[4 * i], raw[4 * i + 1], raw[4 * i + 2], raw[4 * i + 3]])
            };
            let mut ch = Channel::constant(val(0));
            for (i, p) in LocalPos::all().enumerate() {
                if val(i) != val(0) {
                    ch.set(p, val(i));
                }
            }
            ch.compact();
            ch
        }
        _ => return Err(DecodeError::Invalid("channel encoding")),
    };
    Ok((id, ch))
}

/// Decodes an asset view. The result is plain data; Core validates it again when a live asset is
/// made from it (`AssetHost::from_view`).
pub fn decode_asset(bytes: &[u8], limits: &Limits) -> Result<AssetView, DecodeError> {
    let body = unframe(bytes, KIND_ASSET, limits)?;
    let mut r = R(&body);
    let id = AssetId::new(r.u64()?);
    let epoch = Epoch::new(r.u32()?);
    let revision = Revision::new(r.u64()?);
    let n_layers = r.u32()? as usize;
    // Every layer needs at least 50 bytes: reject counts the input cannot hold before allocating.
    if n_layers > body.len() / 50 + 1 {
        return Err(DecodeError::Invalid("layer count"));
    }
    let mut layers = Vec::with_capacity(n_layers.min(1024));
    for _ in 0..n_layers {
        let lid = LayerId::new(r.u64()?);
        let kind = match r.u8()? {
            1 => LayerKind::Voxel,
            2 => LayerKind::Sdf,
            3 => LayerKind::Mesh,
            _ => return Err(DecodeError::Invalid("layer kind")),
        };
        let t = [r.i64()?, r.i64()?, r.i64()?];
        let rot = Rotation::from_index(r.u8()?).map_err(|_| DecodeError::Invalid("rotation"))?;
        let shift = r.u8()? as i8;
        let transform =
            GeometryTransform::new(t, rot, shift).map_err(|_| DecodeError::Invalid("transform"))?;
        let bounds = match r.u8()? {
            0 => None,
            1 => {
                let a = BrickCoord::new(r.i64()?, r.i64()?, r.i64()?);
                let b = BrickCoord::new(r.i64()?, r.i64()?, r.i64()?);
                Some(BrickBounds::new(a, b).map_err(|_| DecodeError::Invalid("bounds"))?)
            }
            _ => return Err(DecodeError::Invalid("bounds flag")),
        };
        let lrev = Revision::new(r.u64()?);
        let n_cells = r.u64()? as usize;
        if n_cells > body.len() / 28 + 1 {
            return Err(DecodeError::Invalid("cell count"));
        }
        let mut cells = Vec::with_capacity(n_cells.min(4096));
        for _ in 0..n_cells {
            let level = r.u8()? as i8;
            let brick = BrickCoord::new(r.i64()?, r.i64()?, r.i64()?);
            let crev = Revision::new(r.u64()?);
            let payload = read_payload(&mut r)?;
            cells.push(CellView {
                level,
                brick,
                revision: crev,
                payload,
            });
        }
        layers.push(LayerView {
            id: lid,
            kind,
            transform,
            bounds,
            revision: lrev,
            cells,
        });
    }
    r.done()?;
    Ok(AssetView {
        id,
        epoch,
        revision,
        layers,
    })
}

/// Encodes one journal record: versioned operation id and revision fields (P-02/P-03 slice).
pub fn encode_journal_entry(e: &JournalEntry) -> Vec<u8> {
    let mut w = W(Vec::new());
    w.u64(e.seq.0);
    w.u8(match e.phase {
        JournalPhase::Intent => 1,
        JournalPhase::Confirmed => 2,
        JournalPhase::Aborted => 3,
    });
    w.u64(e.operation.scope());
    w.u64(e.operation.sequence());
    w.u64(e.actor.0);
    w.u64(e.asset.raw());
    w.u32(e.post_tokens.len() as u32);
    for t in &e.post_tokens {
        write_token(&mut w, t);
    }
    match e.delta {
        None => w.u8(0),
        Some(d) => {
            w.u8(1);
            w.u64(d.0);
        }
    }
    w.u32(e.members.len() as u32);
    for (op, fp) in &e.members {
        w.u64(op.scope());
        w.u64(op.sequence());
        w.u64(*fp);
    }
    w.u32(e.payload.len() as u32);
    w.0.extend_from_slice(&e.payload);
    frame(KIND_JOURNAL, w.0)
}

/// Decodes one journal record.
pub fn decode_journal_entry(bytes: &[u8], limits: &Limits) -> Result<JournalEntry, DecodeError> {
    let body = unframe(bytes, KIND_JOURNAL, limits)?;
    let mut r = R(&body);
    let seq = JournalSeq(r.u64()?);
    let phase = match r.u8()? {
        1 => JournalPhase::Intent,
        2 => JournalPhase::Confirmed,
        3 => JournalPhase::Aborted,
        _ => return Err(DecodeError::Invalid("journal phase")),
    };
    let operation = OperationId::new(r.u64()?, r.u64()?);
    let actor = ActorId(r.u64()?);
    let asset = AssetId::new(r.u64()?);
    let n = r.u32()? as usize;
    if n > body.len() / 13 + 1 {
        return Err(DecodeError::Invalid("token count"));
    }
    let mut post_tokens = Vec::with_capacity(n.min(256));
    for _ in 0..n {
        post_tokens.push(read_token(&mut r)?);
    }
    let delta = match r.u8()? {
        0 => None,
        1 => Some(DeltaId(r.u64()?)),
        _ => return Err(DecodeError::Invalid("delta flag")),
    };
    let m = r.u32()? as usize;
    if m > body.len() / 24 + 1 {
        return Err(DecodeError::Invalid("member count"));
    }
    let mut members = Vec::with_capacity(m.min(256));
    for _ in 0..m {
        members.push((OperationId::new(r.u64()?, r.u64()?), r.u64()?));
    }
    let plen = r.u32()? as usize;
    let payload = r.take(plen)?.to_vec();
    r.done()?;
    Ok(JournalEntry {
        seq,
        phase,
        operation,
        actor,
        asset,
        post_tokens,
        delta,
        members,
        payload,
    })
}

/// The snapshot a manifest points to (EXPERIMENTAL).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotRef {
    /// File name of the snapshot.
    pub file: String,
    /// Last journal sequence number whose effect the snapshot contains.
    pub journal_seq: u64,
    /// `CH1` content hash of the snapshot's view.
    pub content_hash: u128,
    /// Size of the snapshot file in bytes.
    pub bytes: u64,
}

/// The identity of a world (EXPERIMENTAL, PERS-08): who it is, which untouched base it was
/// generated from and where its files are named. It holds no world content (that is the
/// generator's output plus the journal and snapshots) and is written once at creation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldRecord {
    /// Opaque identifier of the world.
    pub world_id: u64,
    /// Seed of the procedural base.
    pub seed: cube_core::WorldSeed,
    /// Generator version that produced (and reproduces) the untouched base.
    pub generator_version: cube_core::GeneratorVersion,
    /// Version of the structure of the world files (this record, manifest, journal naming).
    pub structure_version: u32,
}

/// Structure version written by this build.
pub const STRUCTURE_VERSION: u32 = 1;

/// Encodes a world record.
pub fn encode_world_record(r: &WorldRecord) -> Vec<u8> {
    let mut w = W(Vec::new());
    w.u32(r.structure_version);
    w.u64(r.world_id);
    w.u64(r.seed.raw());
    w.u32(r.generator_version.raw());
    frame(KIND_WORLD_RECORD, w.0)
}

/// Decodes a world record. An unknown structure version is refused, never read as a known one.
pub fn decode_world_record(bytes: &[u8], limits: &Limits) -> Result<WorldRecord, DecodeError> {
    let body = unframe(bytes, KIND_WORLD_RECORD, limits)?;
    let mut r = R(&body);
    let structure_version = r.u32()?;
    if structure_version != STRUCTURE_VERSION {
        return Err(DecodeError::Invalid("unsupported world structure version"));
    }
    let world_id = r.u64()?;
    let seed = cube_core::WorldSeed::new(r.u64()?);
    let generator_version = cube_core::GeneratorVersion::new(r.u32()?);
    r.done()?;
    Ok(WorldRecord {
        world_id,
        seed,
        generator_version,
        structure_version,
    })
}

/// The one record that says which files make up the world (EXPERIMENTAL).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    /// Increases by exactly one with every publication.
    pub generation: u64,
    /// Version of the generator the untouched world comes from.
    pub generator_version: u32,
    /// The verified snapshot, if there is one.
    pub snapshot: Option<SnapshotRef>,
    /// File name of the journal that continues after the snapshot.
    pub journal: String,
    /// Sequence number of the first record of that journal.
    pub journal_first_seq: u64,
    /// Retry floors (L-COMP-1): for each scope, the highest sequence number of a confirmed
    /// operation that compaction has folded into the snapshot. Sorted by scope, one entry per
    /// scope. An id at or below its scope's floor that the journal no longer shows is refused
    /// by Orchestration, never applied again. Size is bounded by the number of scopes, not by
    /// the number of operations. Valid only under the retry-safety requirement RS-1 (ids of a
    /// scope are allocated by the authoritative side with strictly increasing sequences).
    pub retry_floors: Vec<(u64, u64)>,
}

fn put_name(w: &mut W, s: &str) {
    w.u16(s.len() as u16);
    w.0.extend_from_slice(s.as_bytes());
}

fn get_name(r: &mut R) -> Result<String, DecodeError> {
    let n = r.u16()? as usize;
    if n == 0 || n > 255 {
        return Err(DecodeError::Invalid("file name length"));
    }
    String::from_utf8(r.take(n)?.to_vec()).map_err(|_| DecodeError::Invalid("file name"))
}

/// Encodes a manifest.
pub fn encode_manifest(m: &Manifest) -> Vec<u8> {
    let mut w = W(Vec::new());
    w.u64(m.generation);
    w.u32(m.generator_version);
    match &m.snapshot {
        None => w.u8(0),
        Some(s) => {
            w.u8(1);
            put_name(&mut w, &s.file);
            w.u64(s.journal_seq);
            w.u64((s.content_hash >> 64) as u64);
            w.u64(s.content_hash as u64);
            w.u64(s.bytes);
        }
    }
    put_name(&mut w, &m.journal);
    w.u64(m.journal_first_seq);
    w.u32(m.retry_floors.len() as u32);
    for (scope, seq) in &m.retry_floors {
        w.u64(*scope);
        w.u64(*seq);
    }
    frame(KIND_MANIFEST, w.0)
}

/// Decodes a manifest.
pub fn decode_manifest(bytes: &[u8], limits: &Limits) -> Result<Manifest, DecodeError> {
    let body = unframe(bytes, KIND_MANIFEST, limits)?;
    let mut r = R(&body);
    let generation = r.u64()?;
    let generator_version = r.u32()?;
    let snapshot = match r.u8()? {
        0 => None,
        1 => Some(SnapshotRef {
            file: get_name(&mut r)?,
            journal_seq: r.u64()?,
            content_hash: (u128::from(r.u64()?) << 64) | u128::from(r.u64()?),
            bytes: r.u64()?,
        }),
        _ => return Err(DecodeError::Invalid("snapshot flag")),
    };
    let journal = get_name(&mut r)?;
    let journal_first_seq = r.u64()?;
    let n = r.u32()? as usize;
    if n > body.len() / 16 + 1 {
        return Err(DecodeError::Invalid("floor count"));
    }
    let mut retry_floors: Vec<(u64, u64)> = Vec::with_capacity(n.min(1024));
    for _ in 0..n {
        let f = (r.u64()?, r.u64()?);
        if retry_floors.last().is_some_and(|l| l.0 >= f.0) {
            return Err(DecodeError::Invalid("floors not sorted by scope"));
        }
        retry_floors.push(f);
    }
    r.done()?;
    Ok(Manifest {
        generation,
        generator_version,
        snapshot,
        journal,
        journal_first_seq,
        retry_floors,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view() -> AssetView {
        let mut mat = Channel::constant(3);
        mat.set(LocalPos::new(1, 2, 3).unwrap(), 9);
        let set = ChannelSet::new()
            .with(ChannelId::Material, mat)
            .with(ChannelId::Density, Channel::constant(65000))
            .with(ChannelId::Attribute(4), Channel::constant(1));
        AssetView {
            id: AssetId::new(7),
            epoch: Epoch::new(2),
            revision: Revision::new(40),
            layers: vec![
                LayerView {
                    id: LayerId::new(1),
                    kind: LayerKind::Voxel,
                    transform: GeometryTransform::new(
                        [-5, 0, i64::MAX],
                        Rotation::from_index(9).unwrap(),
                        2,
                    )
                    .unwrap(),
                    bounds: Some(
                        BrickBounds::new(BrickCoord::new(-3, -3, -3), BrickCoord::new(4, 4, 4))
                            .unwrap(),
                    ),
                    revision: Revision::new(12),
                    cells: vec![
                        CellView {
                            level: 0,
                            brick: BrickCoord::new(-1, 0, 2),
                            revision: Revision::new(5),
                            payload: Some(CellPayload::Channels(set)),
                        },
                        CellView {
                            level: 3,
                            brick: BrickCoord::new(i64::MIN, 1, 1),
                            revision: Revision::new(6),
                            payload: None,
                        },
                    ],
                },
                LayerView {
                    id: LayerId::new(2),
                    kind: LayerKind::Mesh,
                    transform: GeometryTransform::IDENTITY,
                    bounds: None,
                    revision: Revision::new(0),
                    cells: vec![],
                },
            ],
        }
    }

    fn entry() -> JournalEntry {
        use cube_core::{CellKey, ResourceId};
        JournalEntry {
            seq: JournalSeq(9),
            phase: JournalPhase::Confirmed,
            operation: OperationId::new(u64::MAX, 3),
            actor: ActorId(42),
            asset: AssetId::new(1),
            post_tokens: vec![
                RevisionToken::new(
                    ResourceId::Asset(AssetId::new(1)),
                    Epoch::INITIAL,
                    Revision::new(8),
                ),
                RevisionToken::new(
                    ResourceId::Cell(
                        CellKey::new(LayerId::new(1), 2, BrickCoord::new(-9, 0, 9)).unwrap(),
                    ),
                    Epoch::new(1),
                    Revision::new(2),
                ),
            ],
            delta: Some(DeltaId(5)),
            members: vec![
                (OperationId::new(u64::MAX, 3), 77),
                (OperationId::new(2, 4), u64::MAX),
            ],
            payload: vec![1, 2, 3, 250],
        }
    }

    #[test]
    fn assets_and_journal_entries_round_trip_and_encoding_is_deterministic() {
        let v = view();
        let bytes = encode_asset(&v);
        assert_eq!(bytes, encode_asset(&v));
        assert_eq!(decode_asset(&bytes, &Limits::default()), Ok(v));
        let e = entry();
        let b = encode_journal_entry(&e);
        assert_eq!(decode_journal_entry(&b, &Limits::default()), Ok(e));
    }

    /// `view()` with the first cell's material channel replaced.
    fn view_with_material(mat: Channel) -> AssetView {
        let mut v = view();
        let set = ChannelSet::new()
            .with(ChannelId::Material, mat)
            .with(ChannelId::Density, Channel::constant(65000));
        v.layers[0].cells[0].payload = Some(CellPayload::Channels(set));
        v
    }

    #[test]
    fn wide_material_values_round_trip_and_narrow_values_keep_the_old_encoding() {
        // MAT-01: values above 0xFFFF use the new encodings 3 (constant) and 2 (dense).
        let wide_const = view_with_material(Channel::constant(70_000));
        let mut dense = Channel::constant(3);
        dense.set(LocalPos::new(1, 2, 3).unwrap(), 4_000_000_000);
        dense.set(LocalPos::new(0, 0, 1).unwrap(), 300);
        let wide_dense = view_with_material(dense);
        for v in [wide_const, wide_dense] {
            let bytes = encode_asset(&v);
            assert_eq!(decode_asset(&bytes, &Limits::default()), Ok(v));
        }
        // A channel that fits 16 bits is stored exactly as before MAT-01: the encoding does not
        // depend on the in-memory width, so a dense channel of small values has the same bytes
        // whether it was built narrow or widened and compacted.
        let mut narrow = Channel::constant(3);
        narrow.set(LocalPos::new(1, 2, 3).unwrap(), 9);
        let mut widened = Channel::constant(3);
        widened.set(LocalPos::new(1, 2, 3).unwrap(), 70_000);
        widened.set(LocalPos::new(1, 2, 3).unwrap(), 9);
        assert_ne!(narrow.storage_bits(), widened.storage_bits());
        assert_eq!(
            encode_asset(&view_with_material(narrow)),
            encode_asset(&view_with_material(widened)),
            "equal values encode equally whatever the storage width"
        );
        // 65_535 still uses the old 16-bit constant encoding (tag 1, two bytes), 65_536 the new one.
        let small = encode_asset(&view_with_material(Channel::constant(65_535)));
        let large = encode_asset(&view_with_material(Channel::constant(65_536)));
        assert_eq!(large.len(), small.len() + 2);
    }

    #[test]
    fn an_unknown_channel_encoding_is_a_controlled_error() {
        // What an older reader sees of a future encoding: an error, never a misread. Tags 0..=3
        // are known; 7 is not.
        let mut r = R(&[1, 0, 7]);
        assert_eq!(
            read_channel(&mut r),
            Err(DecodeError::Invalid("channel encoding"))
        );
    }

    #[test]
    fn crc32_matches_the_standard_check_value() {
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
    }

    #[test]
    fn every_single_byte_corruption_is_detected() {
        for bytes in [encode_asset(&view()), encode_journal_entry(&entry())] {
            for i in 0..bytes.len() {
                let mut bad = bytes.clone();
                bad[i] ^= 0x01;
                let r1 = decode_asset(&bad, &Limits::default());
                let r2 = decode_journal_entry(&bad, &Limits::default());
                assert!(
                    r1.is_err() && r2.is_err(),
                    "flip at byte {i} went unnoticed"
                );
            }
        }
    }

    #[test]
    fn every_truncation_and_extension_is_detected() {
        let bytes = encode_asset(&view());
        for cut in 0..bytes.len() {
            assert!(
                decode_asset(&bytes[..cut], &Limits::default()).is_err(),
                "cut {cut}"
            );
        }
        let mut long = bytes;
        long.push(0);
        assert_eq!(
            decode_asset(&long, &Limits::default()),
            Err(DecodeError::TrailingBytes)
        );
    }

    #[test]
    fn headers_schema_kind_and_limits_are_checked_before_the_body() {
        let good = encode_asset(&view());
        let mut magic = good.clone();
        magic[0] = b'X';
        assert_eq!(
            decode_asset(&magic, &Limits::default()),
            Err(DecodeError::BadMagic)
        );
        let mut schema = good.clone();
        schema[4] = 0x02;
        assert!(matches!(
            decode_asset(&schema, &Limits::default()),
            Err(DecodeError::UnsupportedSchema(_))
        ));
        assert!(matches!(
            decode_journal_entry(&good, &Limits::default()),
            Err(DecodeError::WrongKind { found: 1 })
        ));
        assert_eq!(
            decode_asset(&good, &Limits { max_body: 10 }),
            Err(DecodeError::TooLarge)
        );
        // A header that promises a huge body is refused without allocating it.
        let mut huge = good[..HEADER].to_vec();
        huge[7..11].copy_from_slice(&u32::MAX.to_le_bytes());
        assert_eq!(
            decode_asset(&huge, &Limits::default()),
            Err(DecodeError::TooLarge)
        );
        assert_eq!(
            decode_asset(&[], &Limits::default()),
            Err(DecodeError::Truncated)
        );
    }

    #[test]
    fn hostile_counts_with_a_valid_checksum_do_not_allocate_or_panic() {
        // Hand-build a framed body that claims four billion layers.
        let mut w = W(Vec::new());
        w.u64(1);
        w.u32(0);
        w.u64(0);
        w.u32(u32::MAX);
        let framed = frame(KIND_ASSET, w.0);
        assert_eq!(
            decode_asset(&framed, &Limits::default()),
            Err(DecodeError::Invalid("layer count"))
        );
        let mut w = W(Vec::new());
        w.u64(1);
        w.u32(0);
        w.u64(0);
        w.u32(1);
        w.u64(1);
        w.u8(99);
        assert!(decode_asset(&frame(KIND_ASSET, w.0), &Limits::default()).is_err());
    }
}
