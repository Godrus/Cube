//! P-WORLD stage B (experimental spike, not for production): a voxel character body.
//!
//! Pure Rust, no Godot types. The body is an axis-aligned box (`WIDTH` x `HEIGHT`) whose position is the
//! centre of its feet. Voxel `(x, y, z)` occupies `[x, x+1) x [y, y+1) x [z, z+1)`. The world is read through
//! a cell function so that the physics is testable without a streamer.
//!
//! Rules (documented because the tests pin them):
//! * gravity and jumping act on the vertical axis; horizontal speed follows the input directly (no inertia);
//! * each axis is moved separately and resolved against solid voxels (no tunnelling: a sub-step moves < 1 voxel);
//! * a blocked horizontal move on the ground steps up one voxel when there is room above;
//! * `y < 0` is solid (the bottom of the world), `y >= CHUNK_HEIGHT` is air;
//! * **unloaded chunks:** a body is *held* (no movement, no gravity) while any chunk within one voxel of its box
//!   is not loaded, and an unloaded cell counts as solid, so the body can neither fall through a missing chunk
//!   nor walk into one.

pub const WIDTH: f32 = 0.6;
pub const HEIGHT: f32 = 1.8;
pub const WALK_SPEED: f32 = 6.0;
pub const GRAVITY: f32 = 28.0;
pub const JUMP_SPEED: f32 = 8.5;
pub const MAX_FALL: f32 = 40.0;
/// Longest physics sub-step (seconds). Fall speed times this stays below one voxel.
pub const MAX_SUBSTEP: f32 = 1.0 / 60.0;
const EPS: f32 = 1e-3;
/// How far below the feet the ground probe looks (voxels).
const GROUND_PROBE: f32 = 0.02;
/// Height (voxels) above the top of the world at which a teleported body waits for its chunk.
pub const TELEPORT_HEIGHT: f32 = 40.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Air,
    Solid,
    /// The chunk holding the cell is not loaded.
    NotReady,
}

#[derive(Clone, Debug)]
pub struct PlayerBody {
    /// Centre of the feet.
    pub pos: [f32; 3],
    pub vel_y: f32,
    pub on_ground: bool,
    pub held: bool,
}

impl Default for PlayerBody {
    fn default() -> Self {
        Self {
            pos: [8.5, TELEPORT_HEIGHT, 8.5],
            vel_y: 0.0,
            on_ground: false,
            held: true,
        }
    }
}

fn fl(v: f32) -> i32 {
    v.floor() as i32
}

/// Does the box with feet centre `p` overlap a solid or unloaded cell?
fn collides(cell: &dyn Fn(i32, i32, i32) -> Cell, p: [f32; 3]) -> bool {
    let h = WIDTH / 2.0;
    let (x0, x1) = (fl(p[0] - h + EPS), fl(p[0] + h - EPS));
    let (y0, y1) = (fl(p[1] + EPS), fl(p[1] + HEIGHT - EPS));
    let (z0, z1) = (fl(p[2] - h + EPS), fl(p[2] + h - EPS));
    for y in y0..=y1 {
        for z in z0..=z1 {
            for x in x0..=x1 {
                if cell(x, y, z) != Cell::Air {
                    return true;
                }
            }
        }
    }
    false
}

/// Is every chunk within one voxel of the box loaded? (checks the four expanded corners at a loaded height)
fn surroundings_ready(cell: &dyn Fn(i32, i32, i32) -> Cell, p: [f32; 3]) -> bool {
    let m = WIDTH / 2.0 + 1.0;
    for dz in [-m, m] {
        for dx in [-m, m] {
            let y = fl(p[1].clamp(0.0, 31.0));
            if cell(fl(p[0] + dx), y, fl(p[2] + dz)) == Cell::NotReady {
                return false;
            }
        }
    }
    cell(fl(p[0]), fl(p[1].clamp(0.0, 31.0)), fl(p[2])) != Cell::NotReady
}

impl PlayerBody {
    /// Puts the body at `(x, y, z)` without checks.
    pub fn place(&mut self, x: f32, y: f32, z: f32) {
        self.pos = [x, y, z];
        self.vel_y = 0.0;
        self.on_ground = false;
    }

    /// Advances the body by `dt` seconds. `mx`, `mz` are the wanted horizontal direction (unit length or less),
    /// `jump` asks for a jump when on the ground.
    pub fn step(&mut self, cell: &dyn Fn(i32, i32, i32) -> Cell, dt: f32, mx: f32, mz: f32, jump: bool) {
        let mut left = dt.clamp(0.0, 0.25);
        while left > 0.0 {
            let d = left.min(MAX_SUBSTEP);
            self.sub_step(cell, d, mx, mz, jump);
            left -= d;
        }
    }

    fn sub_step(&mut self, cell: &dyn Fn(i32, i32, i32) -> Cell, dt: f32, mx: f32, mz: f32, jump: bool) {
        self.held = !surroundings_ready(cell, self.pos);
        if self.held {
            self.vel_y = 0.0;
            return;
        }
        // horizontal
        let len = (mx * mx + mz * mz).sqrt();
        let (ux, uz) = if len > 1.0 { (mx / len, mz / len) } else { (mx, mz) };
        self.move_axis(cell, 0, ux * WALK_SPEED * dt);
        self.move_axis(cell, 2, uz * WALK_SPEED * dt);
        // vertical
        if jump && self.on_ground {
            self.vel_y = JUMP_SPEED;
        }
        self.vel_y = (self.vel_y - GRAVITY * dt).max(-MAX_FALL);
        let dy = self.vel_y * dt;
        self.on_ground = false;
        let mut np = self.pos;
        np[1] += dy;
        if !collides(cell, np) {
            self.pos = np;
        } else if dy < 0.0 {
            // landed: stand on top of the solid row that was entered
            self.pos[1] = fl(np[1] + EPS) as f32 + 1.0;
            self.vel_y = 0.0;
            self.on_ground = true;
        } else {
            // head hit a ceiling
            self.pos[1] = (fl(np[1] + HEIGHT - EPS) as f32 - HEIGHT).min(self.pos[1]);
            self.vel_y = 0.0;
        }
        // Ground probe: standing is decided by what is just below the feet, not by whether this sub-step's fall
        // reached the surface (a tiny frame time falls less than EPS and would otherwise flicker `on_ground`).
        if !self.on_ground && self.vel_y <= 0.0 {
            let mut probe = self.pos;
            probe[1] -= GROUND_PROBE;
            if collides(cell, probe) {
                self.pos[1] = fl(probe[1] + EPS) as f32 + 1.0;
                self.vel_y = 0.0;
                self.on_ground = true;
            }
        }
    }

    fn move_axis(&mut self, cell: &dyn Fn(i32, i32, i32) -> Cell, axis: usize, delta: f32) {
        if delta == 0.0 {
            return;
        }
        let mut np = self.pos;
        np[axis] += delta;
        if !collides(cell, np) {
            self.pos = np;
            return;
        }
        // one-voxel step-up when standing on the ground and there is room above
        if self.on_ground {
            let mut up = np;
            up[1] = self.pos[1] + 1.0;
            let mut clear_above = self.pos;
            clear_above[1] += 1.0;
            if !collides(cell, clear_above) && !collides(cell, up) {
                self.pos = up;
                self.on_ground = false;
                return;
            }
        }
        // snap to the face of the blocking voxel
        let h = WIDTH / 2.0;
        if delta > 0.0 {
            let edge = fl(np[axis] + h - EPS) as f32;
            self.pos[axis] = (edge - h - EPS).max(self.pos[axis]);
        } else {
            let edge = fl(np[axis] - h + EPS) as f32 + 1.0;
            self.pos[axis] = (edge + h + EPS).min(self.pos[axis]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Flat floor at y = 4 (solid for y < 4), everything else air; one wall voxel column at x = 12.
    fn flat(x: i32, y: i32, _z: i32) -> Cell {
        if y < 4 || (x == 12 && (4..6).contains(&y)) {
            Cell::Solid
        } else {
            Cell::Air
        }
    }

    #[test]
    fn falls_and_lands_on_the_floor_and_stays() {
        let mut b = PlayerBody::default();
        b.place(8.5, 20.0, 8.5);
        for _ in 0..240 {
            b.step(&flat, 1.0 / 60.0, 0.0, 0.0, false);
        }
        assert!(b.on_ground && (b.pos[1] - 4.0).abs() < 0.01, "y = {}", b.pos[1]);
    }

    #[test]
    fn two_high_wall_blocks_and_jump_does_not_clear_it() {
        let mut b = PlayerBody::default();
        b.place(8.5, 4.0, 8.5);
        for _ in 0..600 {
            b.step(&flat, 1.0 / 60.0, 1.0, 0.0, true);
        }
        assert!(b.pos[0] < 12.0 - WIDTH / 2.0 + 0.01, "x = {}", b.pos[0]);
    }

    #[test]
    fn steps_up_one_voxel() {
        let step1 = |x: i32, y: i32, _z: i32| if y < 4 || (x >= 12 && y < 5) { Cell::Solid } else { Cell::Air };
        let mut b = PlayerBody::default();
        b.place(8.5, 4.0, 8.5);
        for _ in 0..300 {
            b.step(&step1, 1.0 / 60.0, 1.0, 0.0, false);
        }
        assert!(b.pos[0] > 13.0 && (b.pos[1] - 5.0).abs() < 0.01, "pos = {:?}", b.pos);
    }

    #[test]
    fn standing_is_stable_for_any_frame_time_and_jumping_works() {
        let mut b = PlayerBody::default();
        b.place(8.5, 4.0, 8.5);
        // frame times from 0.2 ms to 60 ms in a jittery order, as a real frame loop produces them
        let dts = [0.0002f32, 0.0036, 0.0133, 0.0005, 0.03, 0.0069, 0.06, 0.001, 0.0167];
        for i in 0..2000 {
            b.step(&flat, dts[i % dts.len()], 0.0, 0.0, false);
            assert!(b.on_ground && b.pos[1] == 4.0, "step {i}: on_ground {} y {}", b.on_ground, b.pos[1]);
        }
        for dt in [0.0005f32, 0.0036, 0.0133, 0.0167] {
            let mut peak = 4.0f32;
            b.step(&flat, dt, 0.0, 0.0, true);
            for _ in 0..(1.0 / dt) as usize {
                b.step(&flat, dt, 0.0, 0.0, false);
                peak = peak.max(b.pos[1]);
            }
            assert!(peak > 5.0 && peak < 5.6, "dt {dt}: peak {peak}");
            assert!(b.on_ground && b.pos[1] == 4.0, "dt {dt}: did not land ({} {})", b.on_ground, b.pos[1]);
        }
    }

    #[test]
    fn unloaded_surroundings_hold_the_body() {
        let none = |_: i32, _: i32, _: i32| Cell::NotReady;
        let mut b = PlayerBody::default();
        b.place(8.5, 20.0, 8.5);
        for _ in 0..120 {
            b.step(&none, 1.0 / 60.0, 1.0, 0.0, false);
        }
        assert!(b.held && b.pos == [8.5, 20.0, 8.5]);
    }
}
