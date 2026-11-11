//! Dense voxel grid of material bytes (0 = empty) placed on a model lattice, and its coarsening.

/// A small dense grid placed on the global lattice of one model (integer voxel coordinates).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid {
    /// Lattice coordinate of cell (0,0,0).
    pub origin: [i32; 3],
    /// Cells per axis.
    pub dims: [u32; 3],
    /// Material per cell, x fastest, then y, then z.
    pub cells: Vec<u8>,
}

impl Grid {
    /// Empty grid covering `lo..hi` (lattice coordinates, `hi` exclusive).
    pub fn new(lo: [i32; 3], hi: [i32; 3]) -> Self {
        let d = [0, 1, 2].map(|i| (hi[i] - lo[i]).max(0) as u32);
        Self {
            origin: lo,
            dims: d,
            cells: vec![0; (d[0] * d[1] * d[2]) as usize],
        }
    }

    fn idx(&self, p: [i32; 3]) -> Option<usize> {
        let mut l = [0u32; 3];
        for i in 0..3 {
            let v = p[i] - self.origin[i];
            if v < 0 || v as u32 >= self.dims[i] {
                return None;
            }
            l[i] = v as u32;
        }
        Some((l[0] + self.dims[0] * (l[1] + self.dims[1] * l[2])) as usize)
    }

    /// Material at lattice coordinate `p`; 0 outside the grid.
    pub fn get(&self, p: [i32; 3]) -> u8 {
        self.idx(p).map_or(0, |i| self.cells[i])
    }

    /// Sets a cell; ignored outside the grid.
    pub fn set(&mut self, p: [i32; 3], m: u8) {
        if let Some(i) = self.idx(p) {
            self.cells[i] = m;
        }
    }

    /// Number of occupied cells.
    pub fn count(&self) -> usize {
        self.cells.iter().filter(|&&c| c != 0).count()
    }

    /// Halves the resolution. A coarse cell is occupied when at least `threshold` of its eight
    /// children are; it takes the most frequent child material (ties: higher index). Lower
    /// thresholds keep thin parts (blade, cape) at the price of a slightly fatter silhouette.
    pub fn coarsen(&self, threshold: u32) -> Grid {
        let lo = self.origin.map(|v| v.div_euclid(2));
        let hi = [0, 1, 2].map(|i| (self.origin[i] + self.dims[i] as i32 + 1).div_euclid(2));
        let mut out = Grid::new(lo, hi);
        for z in lo[2]..hi[2] {
            for y in lo[1]..hi[1] {
                for x in lo[0]..hi[0] {
                    let mut hist = [0u32; 256];
                    let mut n = 0;
                    for (dx, dy, dz) in ITER8 {
                        let m = self.get([2 * x + dx, 2 * y + dy, 2 * z + dz]);
                        if m != 0 {
                            hist[m as usize] += 1;
                            n += 1;
                        }
                    }
                    if n >= threshold.max(1) {
                        let best = (1..256).max_by_key(|&m| (hist[m], m)).unwrap_or(0);
                        out.set([x, y, z], best as u8);
                    }
                }
            }
        }
        out
    }
}

const ITER8: [(i32, i32, i32); 8] = [
    (0, 0, 0),
    (1, 0, 0),
    (0, 1, 0),
    (1, 1, 0),
    (0, 0, 1),
    (1, 0, 1),
    (0, 1, 1),
    (1, 1, 1),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_set_and_count() {
        let mut g = Grid::new([-2, 0, 0], [2, 2, 2]);
        g.set([-2, 1, 1], 5);
        g.set([9, 9, 9], 5);
        assert_eq!(g.get([-2, 1, 1]), 5);
        assert_eq!(g.get([9, 9, 9]), 0);
        assert_eq!(g.count(), 1);
    }

    #[test]
    fn coarsen_majority_and_threshold() {
        let mut g = Grid::new([0, 0, 0], [2, 2, 2]);
        for p in [[0, 0, 0], [1, 0, 0], [0, 1, 0]] {
            g.set(p, 3);
        }
        g.set([1, 1, 0], 4);
        assert_eq!(g.coarsen(4).get([0, 0, 0]), 3);
        assert_eq!(g.coarsen(5).count(), 0);
    }
}
