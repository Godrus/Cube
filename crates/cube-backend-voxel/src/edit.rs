//! Brick editor: turns voxel-level edits into whole-brick write sets (VOX-01).
//!
//! Reads go through the overlay (committed state plus what the transaction staged), writes are
//! collected per brick and returned as `CellWrite`s; nothing here can commit.
//!
//! Safety rules, all about not losing data silently:
//!
//! * a brick that is **not loaded** may only be written if the edit covers all 4096 voxels of it
//!   (then nothing unknown is overwritten by guesswork); a partly covered unloaded brick is an
//!   error, never "assume empty";
//! * a brick outside the layer bounds or one that failed to load is an error;
//! * only the material channel is changed: density and attribute channels of a loaded brick are
//!   carried over untouched;
//! * a brick whose voxels all end up air and that has no other channel becomes `KnownEmpty`;
//! * a loaded brick that the edit did not change produces no write.

use std::collections::BTreeMap;

use cube_core::{
    BRICK_VOLUME, BrickCoord, CellChange, CellKey, CellPayload, CellState, CellWrite, Channel,
    ChannelId, ChannelSet, ExpectedRevision, LayerId, LocalPos, VoxelPos,
};
use cube_editing_api::{EditingError, OverlayRead};

/// Limits protecting memory; configuration, not world limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    /// Most bricks one operation may touch (read or write).
    pub max_bricks: u64,
}

impl Default for Limits {
    fn default() -> Self {
        Self { max_bricks: 4096 }
    }
}

fn invalid(why: &'static str) -> EditingError {
    EditingError::InvalidOperation(why)
}

struct Buf {
    mat: Vec<u16>,
    known: bool,
    others: Vec<(ChannelId, Channel)>,
    written: Vec<u64>,
    written_count: usize,
    changed: bool,
}

fn idx(l: LocalPos) -> usize {
    usize::from(l.x()) + 16 * (usize::from(l.y()) + 16 * usize::from(l.z()))
}

pub(crate) struct Editor<'a> {
    overlay: &'a dyn OverlayRead,
    layer: LayerId,
    level: i8,
    bufs: BTreeMap<BrickCoord, Buf>,
}

impl<'a> Editor<'a> {
    pub(crate) fn new(overlay: &'a dyn OverlayRead, layer: LayerId, level: i8) -> Self {
        Self {
            overlay,
            layer,
            level,
            bufs: BTreeMap::new(),
        }
    }

    fn load(&mut self, brick: BrickCoord) -> Result<(), EditingError> {
        if self.bufs.contains_key(&brick) {
            return Ok(());
        }
        let key = CellKey::new(self.layer, self.level, brick)
            .map_err(|_| EditingError::InvalidResolution)?;
        let state = self.overlay.cell(key)?;
        let buf = match state {
            CellState::Present(p) => match &*p {
                CellPayload::Channels(set) => {
                    let mut mat = vec![0u16; BRICK_VOLUME];
                    if let Some(ch) = set.channel(ChannelId::Material) {
                        for l in LocalPos::all() {
                            mat[idx(l)] = ch.get(l);
                        }
                    }
                    let others = set
                        .iter()
                        .filter(|(id, _)| *id != ChannelId::Material)
                        .map(|(id, c)| (id, c.clone()))
                        .collect();
                    Buf::new(mat, true, others)
                }
                CellPayload::MeshPage(_) => return Err(EditingError::IncompatibleGeometryTypes),
            },
            CellState::KnownEmpty => Buf::new(vec![0; BRICK_VOLUME], true, vec![]),
            CellState::NotLoaded => Buf::new(vec![0; BRICK_VOLUME], false, vec![]),
            CellState::OutsideBounds => return Err(invalid("outside the layer bounds")),
            CellState::LoadFailure(_) => return Err(invalid("brick failed to load")),
        };
        self.bufs.insert(brick, buf);
        Ok(())
    }

    /// The material of a voxel. An unloaded brick has no known content: reading it is an error.
    pub(crate) fn get(&mut self, p: VoxelPos) -> Result<u16, EditingError> {
        let (b, l) = p.split();
        self.load(b)?;
        let buf = &self.bufs[&b];
        if !buf.known && !buf.is_written(idx(l)) {
            return Err(invalid("brick is not loaded"));
        }
        Ok(buf.mat[idx(l)])
    }

    pub(crate) fn set(&mut self, p: VoxelPos, m: u16) -> Result<(), EditingError> {
        let (b, l) = p.split();
        self.load(b)?;
        let buf = self.bufs.get_mut(&b).expect("loaded");
        let i = idx(l);
        buf.mark(i);
        if buf.mat[i] != m {
            buf.mat[i] = m;
            buf.changed = true;
        }
        Ok(())
    }

    /// Applies `f(current) -> new` to every voxel of the box `[min, max)`; `f` returning `None`
    /// leaves the voxel. `needs_current` tells whether `f` reads the current value (then an
    /// unloaded brick is an error); an unconditional write needs no read.
    pub(crate) fn apply_box(
        &mut self,
        min: VoxelPos,
        max: VoxelPos,
        needs_current: bool,
        f: &dyn Fn(u16) -> Option<u16>,
    ) -> Result<(), EditingError> {
        let region =
            cube_core::Region::new(self.level, min, max).map_err(|_| invalid("invalid box"))?;
        for brick in region.bricks() {
            let o = [
                i128::from(brick.x) * 16,
                i128::from(brick.y) * 16,
                i128::from(brick.z) * 16,
            ];
            let lo = [
                i128::from(min.x).max(o[0]),
                i128::from(min.y).max(o[1]),
                i128::from(min.z).max(o[2]),
            ];
            let hi = [
                i128::from(max.x).min(o[0] + 16),
                i128::from(max.y).min(o[1] + 16),
                i128::from(max.z).min(o[2] + 16),
            ];
            self.load(brick)?;
            for z in lo[2]..hi[2] {
                for y in lo[1]..hi[1] {
                    for x in lo[0]..hi[0] {
                        // Within one brick, so the narrowing is lossless.
                        let p = VoxelPos::new(x as i64, y as i64, z as i64);
                        let cur = if needs_current {
                            Some(self.get(p)?)
                        } else {
                            None
                        };
                        let new = match cur {
                            Some(c) => f(c),
                            None => f(0),
                        };
                        if let Some(m) = new {
                            self.set(p, m)?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// The write set of everything edited, in ascending brick order.
    pub(crate) fn finish(self) -> Result<Vec<CellWrite>, EditingError> {
        let mut out = Vec::new();
        for (brick, buf) in self.bufs {
            if !buf.known {
                if buf.written_count != BRICK_VOLUME {
                    return Err(invalid("brick is not loaded and only partly covered"));
                }
            } else if !buf.changed {
                continue;
            }
            let all_air = buf.mat.iter().all(|m| *m == 0);
            let change = if all_air && buf.others.is_empty() {
                CellChange::KnownEmpty
            } else {
                let mut ch = Channel::constant(buf.mat[0]);
                for l in LocalPos::all() {
                    let v = buf.mat[idx(l)];
                    if v != buf.mat[0] {
                        ch.set(l, v);
                    }
                }
                ch.compact();
                let mut set = ChannelSet::new().with(ChannelId::Material, ch);
                for (id, c) in buf.others {
                    set = set.with(id, c);
                }
                CellChange::Put(CellPayload::Channels(set))
            };
            out.push(CellWrite {
                level: self.level,
                brick,
                expected: ExpectedRevision::Absent,
                change,
            });
        }
        Ok(out)
    }
}

impl Buf {
    fn new(mat: Vec<u16>, known: bool, others: Vec<(ChannelId, Channel)>) -> Self {
        Self {
            mat,
            known,
            others,
            written: vec![0; BRICK_VOLUME / 64],
            written_count: 0,
            changed: false,
        }
    }
    fn is_written(&self, i: usize) -> bool {
        self.written[i / 64] >> (i % 64) & 1 == 1
    }
    fn mark(&mut self, i: usize) {
        if !self.is_written(i) {
            self.written[i / 64] |= 1 << (i % 64);
            self.written_count += 1;
        }
    }
}
