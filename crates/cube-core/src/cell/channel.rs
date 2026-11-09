//! Per-cell value channels (contract C-09, Provisional PC-1, DEC-066).
//!
//! A brick holds up to a few channels, each with one raw `u16` value per cell. Encodings are
//! `Constant` (one value for the whole brick) and `Dense` (4096 values). Encodings are an
//! implementation detail: two channels are equal when their values are equal.

use std::collections::BTreeMap;

use crate::coord::{BRICK_VOLUME, LocalPos};
use crate::material::MaterialIndex;

/// Identifier of a built-in channel. A closed set at this stage; the attribute registry is later.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ChannelId {
    /// Categorical material index (never interpolated).
    Material,
    /// Quantised signed density (raw bits of an `i16`; the scale is per-layer data).
    Density,
    /// A numbered auxiliary attribute channel.
    Attribute(u8),
}

#[derive(Debug, Clone)]
enum Encoding {
    Constant(u16),
    Dense(Box<[u16; BRICK_VOLUME]>),
}

/// One channel of a brick: a raw `u16` per cell.
#[derive(Debug, Clone)]
pub struct Channel {
    enc: Encoding,
}

impl Channel {
    /// A channel whose every cell holds `value`.
    pub const fn constant(value: u16) -> Self {
        Self {
            enc: Encoding::Constant(value),
        }
    }

    /// Value of cell `p`.
    pub fn get(&self, p: LocalPos) -> u16 {
        match &self.enc {
            Encoding::Constant(v) => *v,
            Encoding::Dense(d) => d[p.index()],
        }
    }

    /// Sets cell `p`; a constant channel becomes dense only if the value actually differs.
    pub fn set(&mut self, p: LocalPos, value: u16) {
        match &mut self.enc {
            Encoding::Constant(c) => {
                if *c != value {
                    let mut d = Box::new([*c; BRICK_VOLUME]);
                    d[p.index()] = value;
                    self.enc = Encoding::Dense(d);
                }
            }
            Encoding::Dense(d) => d[p.index()] = value,
        }
    }

    /// Collapses a dense channel whose cells are all equal into the constant encoding.
    pub fn compact(&mut self) {
        if let Encoding::Dense(d) = &self.enc {
            let first = d[0];
            if d.iter().all(|v| *v == first) {
                self.enc = Encoding::Constant(first);
            }
        }
    }

    /// `Some(v)` if every cell holds the same value `v` (regardless of encoding).
    pub fn uniform(&self) -> Option<u16> {
        match &self.enc {
            Encoding::Constant(v) => Some(*v),
            Encoding::Dense(d) => {
                let first = d[0];
                d.iter().all(|v| *v == first).then_some(first)
            }
        }
    }

    /// `true` if stored densely (an encoding detail, exposed for tests and statistics).
    pub fn is_dense(&self) -> bool {
        matches!(self.enc, Encoding::Dense(_))
    }
}

impl PartialEq for Channel {
    fn eq(&self, other: &Self) -> bool {
        match (&self.enc, &other.enc) {
            (Encoding::Constant(a), Encoding::Constant(b)) => a == b,
            (Encoding::Dense(a), Encoding::Dense(b)) => a == b,
            (Encoding::Constant(c), Encoding::Dense(d))
            | (Encoding::Dense(d), Encoding::Constant(c)) => d.iter().all(|v| v == c),
        }
    }
}
impl Eq for Channel {}

/// The channels of one brick, ordered by [`ChannelId`] (deterministic iteration).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ChannelSet {
    channels: BTreeMap<ChannelId, Channel>,
}

impl ChannelSet {
    /// An empty set (no channels).
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds or replaces a channel (builder style).
    pub fn with(mut self, id: ChannelId, channel: Channel) -> Self {
        self.channels.insert(id, channel);
        self
    }

    /// A set with a single material channel filled with `m`.
    pub fn uniform_material(m: MaterialIndex) -> Self {
        Self::new().with(ChannelId::Material, Channel::constant(m.raw()))
    }

    /// The channel with the given id.
    pub fn channel(&self, id: ChannelId) -> Option<&Channel> {
        self.channels.get(&id)
    }

    /// Mutable access to a channel of a value being built (not authoritative state).
    pub fn channel_mut(&mut self, id: ChannelId) -> Option<&mut Channel> {
        self.channels.get_mut(&id)
    }

    /// Raw value of channel `id` at `p`, `None` if the channel does not exist.
    pub fn get(&self, id: ChannelId, p: LocalPos) -> Option<u16> {
        self.channels.get(&id).map(|c| c.get(p))
    }

    /// Material at `p`, `None` if there is no material channel.
    pub fn material(&self, p: LocalPos) -> Option<MaterialIndex> {
        self.get(ChannelId::Material, p)
            .map(MaterialIndex::from_raw)
    }

    /// Density at `p` as a signed value, `None` if there is no density channel.
    pub fn density(&self, p: LocalPos) -> Option<i16> {
        self.get(ChannelId::Density, p)
            .map(|v| i16::from_ne_bytes(v.to_ne_bytes()))
    }

    /// Channels in ascending [`ChannelId`] order.
    pub fn iter(&self) -> impl Iterator<Item = (ChannelId, &Channel)> {
        self.channels.iter().map(|(k, v)| (*k, v))
    }

    /// Number of channels.
    pub fn len(&self) -> usize {
        self.channels.len()
    }

    /// `true` if there are no channels.
    pub fn is_empty(&self) -> bool {
        self.channels.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lp(x: u8, y: u8, z: u8) -> LocalPos {
        LocalPos::new(x, y, z).unwrap()
    }

    #[test]
    fn constant_stays_constant_for_equal_writes_and_becomes_dense_otherwise() {
        let mut c = Channel::constant(7);
        c.set(lp(1, 2, 3), 7);
        assert!(!c.is_dense());
        c.set(lp(1, 2, 3), 9);
        assert!(c.is_dense());
        assert_eq!(c.get(lp(1, 2, 3)), 9);
        assert_eq!(c.get(lp(3, 2, 1)), 7);
    }

    #[test]
    fn every_local_position_has_its_own_slot() {
        let mut c = Channel::constant(0);
        for (i, p) in LocalPos::all().enumerate() {
            c.set(p, (i % 65535 + 1) as u16);
        }
        for (i, p) in LocalPos::all().enumerate() {
            assert_eq!(c.get(p), (i % 65535 + 1) as u16);
        }
        assert_eq!(LocalPos::all().count(), BRICK_VOLUME);
    }

    #[test]
    fn equality_ignores_encoding_and_compact_collapses() {
        let mut dense = Channel::constant(5);
        dense.set(lp(0, 0, 0), 6);
        dense.set(lp(0, 0, 0), 5);
        assert!(dense.is_dense());
        assert_eq!(dense, Channel::constant(5));
        assert_eq!(dense.uniform(), Some(5));
        dense.compact();
        assert!(!dense.is_dense());
        dense.set(lp(15, 15, 15), 1);
        assert_ne!(dense, Channel::constant(5));
        assert_eq!(dense.uniform(), None);
    }

    #[test]
    fn channel_set_orders_channels_and_reports_missing_ones() {
        let s = ChannelSet::new()
            .with(ChannelId::Attribute(2), Channel::constant(1))
            .with(ChannelId::Density, Channel::constant(0xFFFF))
            .with(ChannelId::Material, Channel::constant(3));
        let ids: Vec<ChannelId> = s.iter().map(|(i, _)| i).collect();
        assert_eq!(
            ids,
            [
                ChannelId::Material,
                ChannelId::Density,
                ChannelId::Attribute(2)
            ]
        );
        assert_eq!(s.material(lp(0, 0, 0)), Some(MaterialIndex::from_raw(3)));
        assert_eq!(s.density(lp(0, 0, 0)), Some(-1));
        assert_eq!(s.get(ChannelId::Attribute(9), lp(0, 0, 0)), None);
        assert_eq!(s.len(), 3);
        assert!(!s.is_empty() && ChannelSet::new().is_empty());
    }

    #[test]
    fn material_channel_is_categorical_raw_value() {
        let s = ChannelSet::uniform_material(MaterialIndex::from_raw(u16::MAX));
        assert_eq!(
            s.material(lp(5, 5, 5)),
            Some(MaterialIndex::from_raw(u16::MAX))
        );
    }
}
