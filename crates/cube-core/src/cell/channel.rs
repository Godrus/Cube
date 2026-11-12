//! Per-cell value channels (contract C-09, Provisional PC-1, DEC-066, MAT-01).
//!
//! A brick holds up to a few channels, each with one raw `u32` value per cell. Encodings are
//! `Constant` (one value for the whole brick) and three dense widths chosen by the largest value
//! the channel holds: `Dense8` (`u8`), `Dense16` (`u16`) and `Dense32` (`u32`). A channel widens
//! when a larger value is written and narrows again in [`Channel::compact`]. Encodings are an
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
    Constant(u32),
    Dense8(Box<[u8; BRICK_VOLUME]>),
    Dense16(Box<[u16; BRICK_VOLUME]>),
    Dense32(Box<[u32; BRICK_VOLUME]>),
}

/// Storage width in bits needed for `value`: 8, 16 or 32.
const fn bits_for(value: u32) -> u32 {
    if value <= 0xFF {
        8
    } else if value <= 0xFFFF {
        16
    } else {
        32
    }
}

/// One channel of a brick: a raw `u32` per cell, stored in the narrowest dense width that fits.
#[derive(Debug, Clone)]
pub struct Channel {
    enc: Encoding,
}

impl Channel {
    /// A channel whose every cell holds `value`.
    pub const fn constant(value: u32) -> Self {
        Self {
            enc: Encoding::Constant(value),
        }
    }

    /// Value of cell `p`.
    pub fn get(&self, p: LocalPos) -> u32 {
        match &self.enc {
            Encoding::Constant(v) => *v,
            Encoding::Dense8(d) => u32::from(d[p.index()]),
            Encoding::Dense16(d) => u32::from(d[p.index()]),
            Encoding::Dense32(d) => d[p.index()],
        }
    }

    /// Sets cell `p`; a constant channel becomes dense only if the value actually differs, and a
    /// dense channel widens only if the value does not fit its current width.
    pub fn set(&mut self, p: LocalPos, value: u32) {
        let i = p.index();
        match &mut self.enc {
            Encoding::Constant(c) => {
                if *c == value {
                    return;
                }
                let base = *c;
                self.enc = Self::filled(base, bits_for(base.max(value)));
                self.set(p, value);
            }
            Encoding::Dense8(d) => {
                if let Ok(v) = u8::try_from(value) {
                    d[i] = v;
                } else {
                    self.widen(bits_for(value));
                    self.set(p, value);
                }
            }
            Encoding::Dense16(d) => {
                if let Ok(v) = u16::try_from(value) {
                    d[i] = v;
                } else {
                    self.widen(32);
                    self.set(p, value);
                }
            }
            Encoding::Dense32(d) => d[i] = value,
        }
    }

    /// A dense encoding of `bits` width with every cell set to `value`.
    fn filled(value: u32, bits: u32) -> Encoding {
        match bits {
            8 => Encoding::Dense8(Box::new([value as u8; BRICK_VOLUME])),
            16 => Encoding::Dense16(Box::new([value as u16; BRICK_VOLUME])),
            _ => Encoding::Dense32(Box::new([value; BRICK_VOLUME])),
        }
    }

    /// Copies every value into a dense encoding of `bits` width.
    fn restored(&self, bits: u32) -> Encoding {
        let mut out = Self::filled(0, bits);
        for p in LocalPos::all() {
            let v = self.get(p);
            match &mut out {
                Encoding::Dense8(d) => d[p.index()] = v as u8,
                Encoding::Dense16(d) => d[p.index()] = v as u16,
                Encoding::Dense32(d) => d[p.index()] = v,
                Encoding::Constant(_) => unreachable!("filled never returns Constant"),
            }
        }
        out
    }

    /// Re-stores every value in a dense encoding of at least `bits` width.
    fn widen(&mut self, bits: u32) {
        self.enc = self.restored(bits);
    }

    /// Collapses a dense channel whose cells are all equal into the constant encoding, and
    /// narrows any other dense channel to the narrowest width its values fit.
    pub fn compact(&mut self) {
        if let Some(first) = self.uniform() {
            self.enc = Encoding::Constant(first);
            return;
        }
        let want = self.value_bits();
        if want < self.storage_bits() {
            self.enc = self.restored(want);
        }
    }

    /// `Some(v)` if every cell holds the same value `v` (regardless of encoding).
    pub fn uniform(&self) -> Option<u32> {
        match &self.enc {
            Encoding::Constant(v) => Some(*v),
            Encoding::Dense8(d) => {
                let first = d[0];
                d.iter().all(|v| *v == first).then_some(u32::from(first))
            }
            Encoding::Dense16(d) => {
                let first = d[0];
                d.iter().all(|v| *v == first).then_some(u32::from(first))
            }
            Encoding::Dense32(d) => {
                let first = d[0];
                d.iter().all(|v| *v == first).then_some(first)
            }
        }
    }

    /// `true` if stored densely (an encoding detail, exposed for tests and statistics).
    pub fn is_dense(&self) -> bool {
        !matches!(self.enc, Encoding::Constant(_))
    }

    /// Width in bits of the current storage: `0` for a constant channel, else 8, 16 or 32 (an
    /// encoding detail, exposed for tests and statistics).
    pub fn storage_bits(&self) -> u32 {
        match self.enc {
            Encoding::Constant(_) => 0,
            Encoding::Dense8(_) => 8,
            Encoding::Dense16(_) => 16,
            Encoding::Dense32(_) => 32,
        }
    }

    /// The narrowest width (8, 16 or 32 bits) that holds every value of this channel,
    /// independent of the current encoding. Persistence uses it to pick a wire format.
    pub fn value_bits(&self) -> u32 {
        match &self.enc {
            Encoding::Constant(v) => bits_for(*v),
            _ => bits_for(LocalPos::all().map(|p| self.get(p)).max().unwrap_or(0)),
        }
    }
}

impl PartialEq for Channel {
    fn eq(&self, other: &Self) -> bool {
        match (&self.enc, &other.enc) {
            (Encoding::Constant(a), Encoding::Constant(b)) => a == b,
            (Encoding::Dense8(a), Encoding::Dense8(b)) => a == b,
            (Encoding::Dense16(a), Encoding::Dense16(b)) => a == b,
            (Encoding::Dense32(a), Encoding::Dense32(b)) => a == b,
            _ => LocalPos::all().all(|p| self.get(p) == other.get(p)),
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
    pub fn get(&self, id: ChannelId, p: LocalPos) -> Option<u32> {
        self.channels.get(&id).map(|c| c.get(p))
    }

    /// Material at `p`, `None` if there is no material channel.
    pub fn material(&self, p: LocalPos) -> Option<MaterialIndex> {
        self.get(ChannelId::Material, p)
            .map(MaterialIndex::from_raw)
    }

    /// Density at `p` as a signed value, `None` if there is no density channel. Density uses
    /// the low 16 bits of the raw value.
    pub fn density(&self, p: LocalPos) -> Option<i16> {
        self.get(ChannelId::Density, p)
            .map(|v| i16::from_ne_bytes((v as u16).to_ne_bytes()))
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
            c.set(p, (i % 65535 + 1) as u32);
        }
        for (i, p) in LocalPos::all().enumerate() {
            assert_eq!(c.get(p), (i % 65535 + 1) as u32);
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
    fn dense_storage_picks_the_narrowest_width_widens_on_demand_and_narrows_on_compact() {
        let mut c = Channel::constant(3);
        c.set(lp(0, 0, 0), 200);
        assert_eq!(c.storage_bits(), 8, "values up to 255 fit one byte");
        c.set(lp(1, 0, 0), 256);
        assert_eq!(c.storage_bits(), 16);
        c.set(lp(2, 0, 0), 65_536);
        assert_eq!(c.storage_bits(), 32);
        assert_eq!(c.get(lp(0, 0, 0)), 200, "widening keeps every value");
        assert_eq!(c.get(lp(1, 0, 0)), 256);
        assert_eq!(c.get(lp(2, 0, 0)), 65_536);
        assert_eq!(c.get(lp(3, 3, 3)), 3);
        assert_eq!(c.value_bits(), 32);
        // Overwrite the wide values; compact narrows to what the data needs.
        c.set(lp(2, 0, 0), 4);
        c.set(lp(1, 0, 0), 4);
        assert_eq!(c.storage_bits(), 32, "narrowing happens only in compact");
        c.compact();
        assert_eq!(c.storage_bits(), 8);
        assert_eq!(c.value_bits(), 8);
        assert_eq!(c.get(lp(0, 0, 0)), 200);
    }

    #[test]
    fn channels_with_equal_values_are_equal_across_all_widths() {
        let mut narrow = Channel::constant(1);
        narrow.set(lp(0, 0, 0), 2);
        let mut wide = Channel::constant(1);
        wide.set(lp(0, 0, 0), 70_000);
        wide.set(lp(0, 0, 0), 2);
        assert_eq!(narrow.storage_bits(), 8);
        assert_eq!(wide.storage_bits(), 32);
        assert_eq!(narrow, wide);
        wide.set(lp(1, 1, 1), 70_000);
        assert_ne!(narrow, wide);
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
        let s = ChannelSet::uniform_material(MaterialIndex::from_raw(u32::MAX));
        assert_eq!(
            s.material(lp(5, 5, 5)),
            Some(MaterialIndex::from_raw(u32::MAX))
        );
    }
}
