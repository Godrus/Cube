//! Algorithm "terrain-v4": terrain-v3 plus trees (frozen once released: any change of output
//! needs a new version). Terrain, biomes, caves and ores are exactly v3's; trees only occupy air
//! above the surface. Integer arithmetic only.
//!
//! The world is cut into cells of 6 x 6 columns. A cell may hold one tree: whether it does
//! depends on the biome of the trunk column (forest: one cell in two, plains: one in 14,
//! tundra: one in 9, desert: none), the trunk stands at an offset of 1..=4 inside its cell, so
//! two trunks are never closer than 3 columns. The tree is a trunk of 4..=6 voxels of wood with
//! a crown of leaves (radius 2 in its two lowest layers, radius 1, then one voxel on top). A
//! voxel is therefore influenced by the 3 x 3 cells around its column only: the function of a
//! voxel stays a pure function of its coordinates, and a tree that crosses a brick border is
//! the same tree on both sides. A tree is not planted where the trunk column's surface is a cave
//! mouth. Wood beats leaves, terrain beats both.

use cube_core::{BRICK_VOLUME, WorldSeed};

use crate::v1::mix;
use crate::v2::{self, Biome};
use crate::v3;
use crate::{BrickContent, materials};

/// Columns per cell.
const CELL: i64 = 6;
/// Highest possible tree top above a surface of [`MAX_SURFACE`].
const MAX_TREE: i64 = 9;
const MAX_SURFACE: i64 = 100;

/// A tree: trunk column, surface height at the trunk and trunk height.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tree {
    /// Trunk column x.
    pub x: i64,
    /// Trunk column z.
    pub z: i64,
    /// Height of the ground at the trunk (the trunk starts one above it).
    pub ground: i64,
    /// Trunk height in voxels (4..=6).
    pub trunk: i64,
}

/// The tree of cell `(cx, cz)`, if there is one.
pub fn tree_of_cell(seed: u64, cx: i64, cz: i64) -> Option<Tree> {
    let h = mix(mix(seed ^ 0x7BEE_5EED) ^ mix(cx as u64) ^ mix((cz as u64).rotate_left(29)));
    let (ox, oz) = ((h % 4) as i64 + 1, ((h >> 8) % 4) as i64 + 1);
    let (x, z) = (cx * CELL + ox, cz * CELL + oz);
    let (ground, biome) = v2::column(seed, x, z);
    let one_in = match biome {
        Biome::Forest => 2,
        Biome::Plains => 14,
        Biome::Tundra => 9,
        Biome::Desert => return None,
    };
    if !(h >> 16).is_multiple_of(one_in) || v3::is_open_at_surface(seed, x, z) {
        return None;
    }
    Some(Tree {
        x,
        z,
        ground,
        trunk: 4 + ((h >> 24) % 3) as i64,
    })
}

impl Tree {
    /// The tree's material at `(x, y, z)` (0 if the tree does not occupy it).
    fn at(&self, x: i64, y: i64, z: i64) -> u16 {
        let (dx, dz, dy) = (x - self.x, z - self.z, y - self.ground);
        if dy < 1 {
            return 0;
        }
        if dx == 0 && dz == 0 && dy <= self.trunk {
            return materials::WOOD;
        }
        let r2 = dx * dx + dz * dz;
        let top = self.trunk;
        let leaf = (dy == top - 1 || dy == top) && r2 <= 5
            || dy == top + 1 && r2 <= 2
            || dy == top + 2 && r2 == 0;
        if leaf { materials::LEAVES } else { 0 }
    }
}

/// The trees that can influence the columns `x0..=x1` x `z0..=z1`.
fn trees_near(seed: u64, x0: i64, x1: i64, z0: i64, z1: i64) -> Vec<Tree> {
    let mut v = Vec::new();
    for cz in (z0 - 2).div_euclid(CELL)..=(z1 + 2).div_euclid(CELL) {
        for cx in (x0 - 2).div_euclid(CELL)..=(x1 + 2).div_euclid(CELL) {
            v.extend(tree_of_cell(seed, cx, cz));
        }
    }
    v
}

/// Material of one voxel.
pub fn material(seed: WorldSeed, x: i64, y: i64, z: i64) -> u16 {
    let base = v3::material(seed, x, y, z);
    if base != 0 {
        return base;
    }
    let mut leaves = false;
    for t in trees_near(seed.raw(), x, x, z, z) {
        match t.at(x, y, z) {
            materials::WOOD => return materials::WOOD,
            materials::LEAVES => leaves = true,
            _ => {}
        }
    }
    if leaves { materials::LEAVES } else { 0 }
}

/// Content of the brick whose origin is `(ox, oy, oz)`.
pub fn brick(seed: WorldSeed, ox: i64, oy: i64, oz: i64) -> BrickContent {
    if oy > MAX_SURFACE + MAX_TREE {
        return BrickContent::Empty;
    }
    let base = if oy > MAX_SURFACE {
        BrickContent::Empty
    } else {
        v3::brick(seed, ox, oy, oz)
    };
    // Trees stand on the ground: bricks wholly below the lowest possible surface have none.
    if oy + 15 < -MAX_SURFACE - 1 {
        return base;
    }
    let trees: Vec<Tree> = trees_near(seed.raw(), ox, ox + 15, oz, oz + 15)
        .into_iter()
        .filter(|t| oy <= t.ground + t.trunk + 2 && oy + 15 > t.ground)
        .collect();
    if trees.is_empty() {
        return base;
    }
    let mut d: Box<[u16; BRICK_VOLUME]> = match base {
        BrickContent::Empty => Box::new([0; BRICK_VOLUME]),
        BrickContent::Uniform(m) => Box::new([m; BRICK_VOLUME]),
        BrickContent::Dense(d) => d,
    };
    let mut changed = false;
    for lz in 0..16u8 {
        for lx in 0..16u8 {
            for ly in 0..16u8 {
                let i = usize::from(lx) + 16 * (usize::from(ly) + 16 * usize::from(lz));
                if d[i] != 0 {
                    continue;
                }
                let (x, y, z) = (ox + i64::from(lx), oy + i64::from(ly), oz + i64::from(lz));
                let mut m = 0;
                for t in &trees {
                    match t.at(x, y, z) {
                        materials::WOOD => {
                            m = materials::WOOD;
                            break;
                        }
                        materials::LEAVES => m = materials::LEAVES,
                        _ => {}
                    }
                }
                if m != 0 {
                    d[i] = m;
                    changed = true;
                }
            }
        }
    }
    if !changed {
        return normalise(d);
    }
    normalise(d)
}

fn normalise(d: Box<[u16; BRICK_VOLUME]>) -> BrickContent {
    let first = d[0];
    if d.iter().all(|m| *m == first) {
        return if first == 0 {
            BrickContent::Empty
        } else {
            BrickContent::Uniform(first)
        };
    }
    BrickContent::Dense(d)
}
