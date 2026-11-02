//! Brick payload and the single BrickStorage implementation (draft C-08).
//! Sparse hash map of copy-on-write bricks (`Arc`). One `u16` material channel.

use std::collections::HashMap;
use std::sync::Arc;

use crate::coord::{local_from_index, local_index, BrickCoord, LocalPos, BRICK_VOLUME};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BrickData {
    Constant(u16),
    Dense(Box<[u16; BRICK_VOLUME]>),
}

#[derive(Clone, Debug)]
pub struct Brick {
    data: BrickData,
    revision: u64,
}

impl Brick {
    pub fn constant(material: u16) -> Self {
        Self {
            data: BrickData::Constant(material),
            revision: 0,
        }
    }

    /// Used by import (persistence). `revision` is the stored brick revision.
    pub fn from_data(data: BrickData, revision: u64) -> Self {
        Self { data, revision }
    }

    pub fn data(&self) -> &BrickData {
        &self.data
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub(crate) fn set_revision(&mut self, r: u64) {
        self.revision = r;
    }

    pub fn get(&self, l: LocalPos) -> u16 {
        match &self.data {
            BrickData::Constant(m) => *m,
            BrickData::Dense(d) => d[local_index(l)],
        }
    }

    /// Returns true when the stored value changed.
    pub fn set(&mut self, l: LocalPos, material: u16) -> bool {
        if self.get(l) == material {
            return false;
        }
        if let BrickData::Constant(m) = self.data {
            self.data = BrickData::Dense(Box::new([m; BRICK_VOLUME]));
        }
        if let BrickData::Dense(d) = &mut self.data {
            d[local_index(l)] = material;
        }
        true
    }

    /// Collapse a uniform dense brick to `Constant`.
    pub fn normalize(&mut self) {
        if let BrickData::Dense(d) = &self.data {
            let first = d[0];
            if d.iter().all(|&v| v == first) {
                self.data = BrickData::Constant(first);
            }
        }
    }

    /// True when every voxel is material 0 (empty).
    pub fn is_empty(&self) -> bool {
        match &self.data {
            BrickData::Constant(m) => *m == 0,
            BrickData::Dense(d) => d.iter().all(|&v| v == 0),
        }
    }

    /// Content equality ignoring revision.
    pub fn same_content(&self, other: &Brick) -> bool {
        match (&self.data, &other.data) {
            (BrickData::Constant(a), BrickData::Constant(b)) => a == b,
            (BrickData::Dense(a), BrickData::Dense(b)) => a[..] == b[..],
            (BrickData::Constant(a), BrickData::Dense(b))
            | (BrickData::Dense(b), BrickData::Constant(a)) => b.iter().all(|v| v == a),
        }
    }

    /// Non-zero voxels as `(local position, material)`.
    pub fn occupied(&self) -> Vec<(LocalPos, u16)> {
        match &self.data {
            BrickData::Constant(0) => Vec::new(),
            BrickData::Constant(m) => (0..BRICK_VOLUME)
                .map(|i| (local_from_index(i), *m))
                .collect(),
            BrickData::Dense(d) => d
                .iter()
                .enumerate()
                .filter(|(_, &v)| v != 0)
                .map(|(i, &v)| (local_from_index(i), v))
                .collect(),
        }
    }
}

/// Distinct brick states. This storage only produces `Present` and `KnownEmpty`;
/// the other states exist so they are never conflated once loading exists.
#[derive(Clone, Debug)]
pub enum BrickLookup {
    Present(Arc<Brick>),
    KnownEmpty,
    NotLoaded,
    OutsideBounds,
    LoadFailure,
}

#[derive(Clone, Debug, Default)]
pub struct BrickStorage {
    map: HashMap<BrickCoord, Arc<Brick>>,
    /// Revision a removed brick would continue from (avoids revision reuse).
    tombstones: HashMap<BrickCoord, u64>,
}

impl BrickStorage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn lookup(&self, c: BrickCoord) -> BrickLookup {
        match self.map.get(&c) {
            Some(b) => BrickLookup::Present(Arc::clone(b)),
            None => BrickLookup::KnownEmpty,
        }
    }

    pub fn get(&self, c: BrickCoord) -> Option<&Arc<Brick>> {
        self.map.get(&c)
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Sorted for deterministic export.
    pub fn bricks_sorted(&self) -> Vec<(BrickCoord, Arc<Brick>)> {
        let mut v: Vec<_> = self.map.iter().map(|(k, b)| (*k, Arc::clone(b))).collect();
        v.sort_by_key(|(k, _)| *k);
        v
    }

    pub fn tombstones_sorted(&self) -> Vec<(BrickCoord, u64)> {
        let mut v: Vec<_> = self.tombstones.iter().map(|(k, r)| (*k, *r)).collect();
        v.sort_by_key(|(k, _)| *k);
        v
    }

    pub(crate) fn insert(&mut self, c: BrickCoord, b: Arc<Brick>) {
        self.tombstones.remove(&c);
        self.map.insert(c, b);
    }

    pub(crate) fn remove(&mut self, c: BrickCoord, tombstone_revision: u64) {
        self.map.remove(&c);
        self.tombstones.insert(c, tombstone_revision);
    }

    pub(crate) fn next_revision_for_new(&self, c: BrickCoord) -> u64 {
        self.tombstones.get(&c).copied().unwrap_or(0) + 1
    }

    pub(crate) fn restore(
        bricks: Vec<(BrickCoord, Arc<Brick>)>,
        tombstones: Vec<(BrickCoord, u64)>,
    ) -> Self {
        Self {
            map: bricks.into_iter().collect(),
            tombstones: tombstones.into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_get_and_normalize() {
        let mut b = Brick::constant(0);
        let p = LocalPos { x: 3, y: 4, z: 5 };
        assert!(b.set(p, 7));
        assert!(!b.set(p, 7));
        assert_eq!(b.get(p), 7);
        assert!(b.set(p, 0));
        b.normalize();
        assert!(matches!(b.data(), BrickData::Constant(0)));
        assert!(b.is_empty());
    }

    #[test]
    fn same_content_across_representations() {
        let a = Brick::constant(2);
        let mut b = Brick::constant(2);
        b.set(LocalPos { x: 0, y: 0, z: 0 }, 3);
        assert!(!a.same_content(&b));
        b.set(LocalPos { x: 0, y: 0, z: 0 }, 2);
        assert!(a.same_content(&b));
    }

    #[test]
    fn missing_brick_is_known_empty() {
        let s = BrickStorage::new();
        assert!(matches!(
            s.lookup(BrickCoord::new(0, 0, 0)),
            BrickLookup::KnownEmpty
        ));
    }
}
