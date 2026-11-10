//! A walking player body (gravity, jump, collision with the voxel world), engine-neutral.
//!
//! This is *client-side movement*, like the free-fly [`crate::Camera`]: it reads the derived faces
//! the client already holds (read-only), owns no world state and cannot change the world. A
//! different authority (a server) would validate or replace it later; today it only decides where
//! the player's body may stand. Collision is a plain axis-aligned box swept through the voxel grid:
//! one axis at a time, small sub-steps, so a step never crosses more than one voxel.
//!
//! The body is 0.6 wide and 1.8 tall (voxel units), the eye is 1.62 above the feet. Walking up a
//! single-voxel step is automatic; anything higher needs a jump (jump height about 1.4).
//! The same inputs and time steps give the same positions on the same machine (no randomness,
//! no wall-clock).

use crate::intent::{FaceIndex, pick_distance};

const HALF_WIDTH: f64 = 0.3;
const HEIGHT: f64 = 1.8;
/// Height of the eye above the feet.
pub const EYE_HEIGHT: f64 = 1.62;
const EPS: f64 = 1e-4;
const WALK_SPEED: f64 = 5.0;
const GRAVITY: f64 = 28.0;
const JUMP_SPEED: f64 = 9.0;
const TERMINAL_SPEED: f64 = 40.0;
const STEP_UP: f64 = 1.0;
const MAX_SUBSTEP: f64 = 0.02;
const MAX_PITCH: f32 = 1.55;

/// One frame of walking input.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WalkInput {
    /// Forward (+1) or backward (-1).
    pub forward: i8,
    /// Right (+1) or left (-1).
    pub right: i8,
    /// Jump was requested (only takes effect on the ground).
    pub jump: bool,
}

/// The player's body and view direction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Walker {
    /// Centre of the feet (voxel units, Y up).
    pub pos: [f64; 3],
    /// Vertical speed (voxels per second, positive up).
    pub vel_y: f64,
    /// Standing on something.
    pub grounded: bool,
    /// Rotation around the vertical axis (radians); 0 looks along -Z.
    pub yaw: f32,
    /// Looking up (positive) or down.
    pub pitch: f32,
}

impl Walker {
    /// A body standing (or about to fall) at `feet`.
    pub fn new(feet: [f64; 3]) -> Self {
        Self {
            pos: feet,
            vel_y: 0.0,
            grounded: false,
            yaw: 0.0,
            pitch: 0.0,
        }
    }

    /// Turns the view.
    pub fn look(&mut self, yaw: f32, pitch: f32) {
        self.yaw += yaw;
        self.pitch = (self.pitch + pitch).clamp(-MAX_PITCH, MAX_PITCH);
    }

    /// The eye position.
    pub fn eye(&self) -> [f64; 3] {
        [self.pos[0], self.pos[1] + EYE_HEIGHT, self.pos[2]]
    }

    /// The unit view direction.
    pub fn forward(&self) -> [f64; 3] {
        let (sy, cy) = f64::from(self.yaw).sin_cos();
        let (sp, cp) = f64::from(self.pitch).sin_cos();
        [-sy * cp, sp, -cy * cp]
    }

    /// `true` if the body (placed with its feet at `feet`) overlaps a solid voxel.
    pub fn collides_at(world: &FaceIndex, feet: [f64; 3]) -> bool {
        let lo = [feet[0] - HALF_WIDTH, feet[1], feet[2] - HALF_WIDTH];
        let hi = [feet[0] + HALF_WIDTH, feet[1] + HEIGHT, feet[2] + HALF_WIDTH];
        let cell = |v: f64| v.floor() as i64;
        let top = |v: f64| (v.ceil() as i64) - 1;
        for x in cell(lo[0])..=top(hi[0]) {
            for y in cell(lo[1])..=top(hi[1]) {
                for z in cell(lo[2])..=top(hi[2]) {
                    if world.is_solid([x, y, z]) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// `true` if the body would overlap the voxel (used to refuse placing a block into the player).
    pub fn overlaps_voxel(&self, voxel: [i64; 3]) -> bool {
        let lo = [
            self.pos[0] - HALF_WIDTH,
            self.pos[1],
            self.pos[2] - HALF_WIDTH,
        ];
        let hi = [
            self.pos[0] + HALF_WIDTH,
            self.pos[1] + HEIGHT,
            self.pos[2] + HALF_WIDTH,
        ];
        (0..3).all(|a| lo[a] < (voxel[a] + 1) as f64 && hi[a] > voxel[a] as f64)
    }

    /// Advances the body by `dt` seconds.
    pub fn step(&mut self, world: &FaceIndex, input: &WalkInput, dt: f32) {
        let dt = f64::from(dt).clamp(0.0, 0.1);
        let n = ((dt / MAX_SUBSTEP).ceil() as usize).max(1);
        let h = dt / n as f64;
        let (sy, cy) = f64::from(self.yaw).sin_cos();
        let fwd = [-sy, -cy];
        let right = [cy, -sy];
        let (f, r) = (
            f64::from(input.forward.clamp(-1, 1)),
            f64::from(input.right.clamp(-1, 1)),
        );
        let mut wish = [f * fwd[0] + r * right[0], f * fwd[1] + r * right[1]];
        let len = (wish[0] * wish[0] + wish[1] * wish[1]).sqrt();
        if len > 1.0 {
            wish = [wish[0] / len, wish[1] / len];
        }
        for _ in 0..n {
            self.substep(world, wish, input.jump, h);
        }
    }

    fn substep(&mut self, world: &FaceIndex, wish: [f64; 2], jump: bool, h: f64) {
        if jump && self.grounded {
            self.vel_y = JUMP_SPEED;
            self.grounded = false;
        }
        // Horizontal, one axis at a time, with a step up onto a single voxel.
        for (axis, d) in [
            (0usize, wish[0] * WALK_SPEED * h),
            (2usize, wish[1] * WALK_SPEED * h),
        ] {
            if d == 0.0 {
                continue;
            }
            let mut next = self.pos;
            next[axis] += d;
            if !Self::collides_at(world, next) {
                self.pos = next;
                continue;
            }
            if self.grounded {
                let raised = self.pos[1].floor() + 1.0 + EPS;
                let mut up = next;
                up[1] = raised;
                if raised - self.pos[1] <= STEP_UP + 0.01 && !Self::collides_at(world, up) {
                    self.pos = up;
                    continue;
                }
            }
            // Blocked: stand against the wall.
            let c = if d > 0.0 {
                ((next[axis] + HALF_WIDTH).floor()) - HALF_WIDTH - EPS
            } else {
                ((next[axis] - HALF_WIDTH).floor()) + 1.0 + HALF_WIDTH + EPS
            };
            let mut stop = self.pos;
            stop[axis] = c;
            if !Self::collides_at(world, stop) {
                self.pos = stop;
            }
        }
        // Vertical.
        self.vel_y = (self.vel_y - GRAVITY * h).max(-TERMINAL_SPEED);
        let dy = self.vel_y * h;
        let mut next = self.pos;
        next[1] += dy;
        if !Self::collides_at(world, next) {
            self.pos = next;
            self.grounded = false;
        } else if dy <= 0.0 {
            self.pos[1] = (next[1].floor()) + 1.0 + EPS;
            self.vel_y = 0.0;
            self.grounded = true;
        } else {
            self.pos[1] = (next[1] + HEIGHT).floor() - HEIGHT - EPS;
            self.vel_y = 0.0;
        }
        if !self.grounded {
            return;
        }
        // Still standing? (the floor may have been removed)
        let mut below = self.pos;
        below[1] -= 0.02;
        if !Self::collides_at(world, below) {
            self.grounded = false;
        }
    }

    /// Where a third-person camera sits: `distance` behind the eye along the view, shortened so
    /// that it stops `margin` in front of the first solid face (it never enters geometry).
    pub fn chase_position(&self, world: &FaceIndex, distance: f64, margin: f64) -> [f64; 3] {
        let eye = self.eye();
        let f = self.forward();
        let back = [-f[0], -f[1], -f[2]];
        let reach = match pick_distance(world, eye, back, distance + margin) {
            Some((_, t)) => (t - margin).clamp(0.0, distance),
            None => distance,
        };
        [
            eye[0] + back[0] * reach,
            eye[1] + back[1] * reach,
            eye[2] + back[2] * reach,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cube_backend_voxel::{Dir, Face};
    use cube_core::VoxelPos;

    /// A solid slab `x, z in -20..20`, top at `height` (solid voxels `y < height`), plus extras.
    fn world(height: i64, extra: &[[i64; 3]]) -> FaceIndex {
        let mut faces = Vec::new();
        let mut add = |x: i64, y: i64, z: i64| {
            for dir in [
                Dir::PosX,
                Dir::NegX,
                Dir::PosY,
                Dir::NegY,
                Dir::PosZ,
                Dir::NegZ,
            ] {
                faces.push(Face {
                    voxel: VoxelPos::new(x, y, z),
                    dir,
                    material: 1,
                });
            }
        };
        for x in -20..20 {
            for z in -20..20 {
                add(x, height - 1, z);
                add(x, height - 2, z);
            }
        }
        for e in extra {
            add(e[0], e[1], e[2]);
        }
        FaceIndex::new(faces.iter())
    }

    fn run(w: &mut Walker, world: &FaceIndex, input: WalkInput, seconds: f32) {
        let mut t = 0.0;
        while t < seconds {
            w.step(world, &input, 1.0 / 60.0);
            t += 1.0 / 60.0;
        }
    }

    #[test]
    fn falls_and_lands_on_the_floor() {
        let world = world(10, &[]);
        let mut w = Walker::new([0.5, 20.0, 0.5]);
        run(&mut w, &world, WalkInput::default(), 2.0);
        assert!(w.grounded);
        assert!((w.pos[1] - 10.0).abs() < 0.01, "feet at {}", w.pos[1]);
        assert_eq!(w.vel_y, 0.0);
    }

    #[test]
    fn jump_leaves_the_ground_and_comes_back_to_the_same_height() {
        let world = world(10, &[]);
        let mut w = Walker::new([0.5, 10.5, 0.5]);
        run(&mut w, &world, WalkInput::default(), 0.5);
        assert!(w.grounded);
        w.step(
            &world,
            &WalkInput {
                jump: true,
                ..Default::default()
            },
            1.0 / 60.0,
        );
        assert!(!w.grounded);
        let mut peak = w.pos[1];
        for _ in 0..120 {
            w.step(&world, &WalkInput::default(), 1.0 / 60.0);
            peak = peak.max(w.pos[1]);
        }
        assert!(
            peak - 10.0 > 1.2 && peak - 10.0 < 1.7,
            "jump height {}",
            peak - 10.0
        );
        assert!(w.grounded);
        assert!((w.pos[1] - 10.0).abs() < 0.01);
    }

    #[test]
    fn cannot_enter_a_wall_and_slides_along_it() {
        // A wall two voxels high across x = 3.
        let wall: Vec<[i64; 3]> = (-20..20)
            .flat_map(|z| [[3, 10, z], [3, 11, z], [3, 12, z]])
            .collect();
        let world = world(10, &wall);
        let mut w = Walker::new([0.5, 10.0, 0.5]);
        w.yaw = -std::f32::consts::FRAC_PI_2; // facing +X
        run(
            &mut w,
            &world,
            WalkInput {
                forward: 1,
                ..Default::default()
            },
            3.0,
        );
        assert!(w.pos[0] < 3.0 - HALF_WIDTH + 0.01, "x = {}", w.pos[0]);
        assert!(!Walker::collides_at(&world, w.pos));
        // Diagonal into the wall keeps moving along it.
        w.yaw = -std::f32::consts::FRAC_PI_4;
        let z0 = w.pos[2];
        run(
            &mut w,
            &world,
            WalkInput {
                forward: 1,
                ..Default::default()
            },
            1.0,
        );
        assert!(w.pos[2] < z0 - 1.0, "slid from {z0} to {}", w.pos[2]);
        assert!(w.pos[0] < 3.0 - HALF_WIDTH + 0.01);
    }

    #[test]
    fn walks_up_one_voxel_automatically_but_not_two() {
        let one: Vec<[i64; 3]> = (3..20)
            .flat_map(|x| (-20..20).map(move |z| [x, 10, z]))
            .collect();
        let world1 = world(10, &one);
        let mut w = Walker::new([0.5, 10.0, 0.5]);
        w.yaw = -std::f32::consts::FRAC_PI_2;
        run(
            &mut w,
            &world1,
            WalkInput {
                forward: 1,
                ..Default::default()
            },
            2.0,
        );
        assert!(w.pos[0] > 4.0, "stopped at x = {}", w.pos[0]);
        assert!((w.pos[1] - 11.0).abs() < 0.01);
        let two: Vec<[i64; 3]> = (3..20)
            .flat_map(|x| (-20..20).flat_map(move |z| [[x, 10, z], [x, 11, z]]))
            .collect();
        let world2 = world(10, &two);
        let mut w = Walker::new([0.5, 10.0, 0.5]);
        w.yaw = -std::f32::consts::FRAC_PI_2;
        run(
            &mut w,
            &world2,
            WalkInput {
                forward: 1,
                ..Default::default()
            },
            2.0,
        );
        assert!(w.pos[0] < 3.0, "climbed a two-voxel wall: x = {}", w.pos[0]);
    }

    #[test]
    fn head_hits_a_low_ceiling() {
        // A ceiling at y = 12 (feet at 10, head at 11.8): a jump is stopped.
        let ceiling: Vec<[i64; 3]> = (-5..5)
            .flat_map(|x| (-5..5).map(move |z| [x, 12, z]))
            .collect();
        let world = world(10, &ceiling);
        let mut w = Walker::new([0.5, 10.0, 0.5]);
        run(&mut w, &world, WalkInput::default(), 0.3);
        w.step(
            &world,
            &WalkInput {
                jump: true,
                ..Default::default()
            },
            1.0 / 60.0,
        );
        let mut peak = 0.0f64;
        for _ in 0..60 {
            w.step(&world, &WalkInput::default(), 1.0 / 60.0);
            peak = peak.max(w.pos[1]);
        }
        assert!(
            peak + HEIGHT <= 12.0 + 1e-3,
            "head went into the ceiling: feet {peak}"
        );
    }

    #[test]
    fn the_body_never_ends_inside_a_solid_voxel() {
        let wall: Vec<[i64; 3]> = (-20..20).flat_map(|z| [[3, 10, z], [3, 11, z]]).collect();
        let world = world(10, &wall);
        let mut w = Walker::new([0.5, 14.0, 0.5]);
        for i in 0..2000 {
            let input = WalkInput {
                forward: 1,
                right: (i / 100 % 3) as i8 - 1,
                jump: i % 70 == 0,
            };
            w.yaw += 0.013;
            w.step(&world, &input, 1.0 / 60.0);
            assert!(
                !Walker::collides_at(&world, w.pos),
                "inside a block at step {i}: {:?}",
                w.pos
            );
        }
    }

    #[test]
    fn deterministic() {
        let world = world(10, &[]);
        let go = || {
            let mut w = Walker::new([0.5, 15.0, 0.5]);
            for i in 0..300 {
                w.yaw += 0.01;
                w.step(
                    &world,
                    &WalkInput {
                        forward: 1,
                        right: 0,
                        jump: i % 50 == 0,
                    },
                    1.0 / 60.0,
                );
            }
            w
        };
        assert_eq!(go(), go());
    }

    #[test]
    fn standing_over_a_removed_floor_starts_falling() {
        let world_with = world(10, &[]);
        let mut w = Walker::new([0.5, 10.0, 0.5]);
        run(&mut w, &world_with, WalkInput::default(), 0.5);
        assert!(w.grounded);
        let hole = world(5, &[]); // the floor is now much lower
        run(&mut w, &hole, WalkInput::default(), 0.3);
        assert!(!w.grounded || w.pos[1] < 10.0);
        assert!(w.pos[1] < 9.9);
    }

    #[test]
    fn third_person_camera_stops_in_front_of_a_wall() {
        // Wall behind the player at z = 6 (player looks along -Z, the camera goes to +Z).
        let wall: Vec<[i64; 3]> = (-20..20)
            .flat_map(|x| (8..16).map(move |y| [x, y, 6]))
            .collect();
        let world = world(10, &wall);
        let mut w = Walker::new([0.5, 10.0, 0.5]);
        w.pitch = 0.0;
        let open = w.chase_position(&world, 4.0, 0.3);
        assert!((open[2] - (0.5 + 4.0)).abs() < 1e-9, "open boom {open:?}");
        w.pos[2] = 3.0; // now the wall is 3 behind the eye, nearer than 4
        let near = w.chase_position(&world, 4.0, 0.3);
        assert!(
            near[2] < 6.0 - 0.29,
            "camera reached z = {} inside or behind the wall",
            near[2]
        );
        assert!(
            near[2] > 3.0,
            "camera is not behind the player: z = {}",
            near[2]
        );
    }

    #[test]
    fn overlap_test_for_placing_blocks() {
        let w = Walker::new([0.5, 10.0, 0.5]);
        assert!(w.overlaps_voxel([0, 10, 0]));
        assert!(w.overlaps_voxel([0, 11, 0]));
        assert!(!w.overlaps_voxel([0, 12, 0]));
        assert!(!w.overlaps_voxel([1, 10, 0]));
    }
}
