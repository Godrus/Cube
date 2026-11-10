//! Input intents, a free-fly camera and block picking (CLI-04; no authority).
//!
//! An engine turns key presses and mouse movement into [`Intent`]s; the client turns intents
//! into a camera pose (a pure, replayable function: the same intents and time steps give the same
//! pose on the same machine) and into [`EditIntent`]s, which are only *requests*: they go to an
//! [`EditPort`] implemented by whoever owns the authoritative world. Nothing here can change the
//! world, and the camera never touches geometry.
//!
//! Picking reads only the derived faces the client already holds (read-only): a ray is walked
//! voxel by voxel and hits the first voxel that shows a face towards the ray.

use std::collections::HashSet;

use cube_backend_voxel::{Dir, Face};

/// What the player wants, independent of any engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Intent {
    /// Move relative to the view direction; each axis is -1, 0 or 1.
    Move {
        /// Forward (+1) or backward (-1).
        forward: i8,
        /// Right (+1) or left (-1).
        right: i8,
        /// Up (+1) or down (-1), along the world vertical.
        up: i8,
    },
    /// Turn the view by these angles (radians): positive yaw turns left, positive pitch looks up.
    Look {
        /// Change of yaw.
        yaw: f32,
        /// Change of pitch.
        pitch: f32,
    },
    /// Change the camera speed (voxels per second), clamped to a sane range.
    Speed(f32),
}

/// A free-fly camera. Positions are in voxel units; the world is Y up.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    /// Position.
    pub pos: [f64; 3],
    /// Rotation around the vertical axis (radians); 0 looks along -Z.
    pub yaw: f32,
    /// Rotation up and down, clamped to just under +-90 degrees.
    pub pitch: f32,
    /// Speed in voxels per second.
    pub speed: f32,
}

const MAX_PITCH: f32 = 1.55;
const MIN_SPEED: f32 = 1.0;
const MAX_SPEED: f32 = 200.0;

impl Camera {
    /// A camera at `pos` looking along -Z.
    pub fn new(pos: [f64; 3]) -> Self {
        Self {
            pos,
            yaw: 0.0,
            pitch: 0.0,
            speed: 12.0,
        }
    }

    /// The unit view direction.
    pub fn forward(&self) -> [f64; 3] {
        let (sy, cy) = f64::from(self.yaw).sin_cos();
        let (sp, cp) = f64::from(self.pitch).sin_cos();
        [-sy * cp, sp, -cy * cp]
    }

    fn right(&self) -> [f64; 3] {
        let (sy, cy) = f64::from(self.yaw).sin_cos();
        [cy, 0.0, -sy]
    }

    /// Applies one intent over `dt` seconds (movement scales with `dt`, turning does not).
    pub fn apply(&mut self, intent: &Intent, dt: f32) {
        match *intent {
            Intent::Move { forward, right, up } => {
                let f = self.forward();
                let r = self.right();
                let d = f64::from(self.speed) * f64::from(dt);
                let (a, b, c) = (
                    f64::from(forward.clamp(-1, 1)),
                    f64::from(right.clamp(-1, 1)),
                    f64::from(up.clamp(-1, 1)),
                );
                for i in 0..3 {
                    self.pos[i] += d * (a * f[i] + b * r[i]);
                }
                self.pos[1] += d * c;
            }
            Intent::Look { yaw, pitch } => {
                self.yaw += yaw;
                self.pitch = (self.pitch + pitch).clamp(-MAX_PITCH, MAX_PITCH);
            }
            Intent::Speed(s) => self.speed = s.clamp(MIN_SPEED, MAX_SPEED),
        }
    }

    /// The brick that contains the camera (voxel coordinates by floor division).
    pub fn brick(&self) -> cube_core::BrickCoord {
        let v = |c: f64| (c.floor() as i64).div_euclid(16);
        cube_core::BrickCoord::new(v(self.pos[0]), v(self.pos[1]), v(self.pos[2]))
    }
}

/// Replays timed intents from `start`: deterministic by construction.
pub fn replay(start: Camera, intents: &[(Intent, u32)]) -> Camera {
    let mut c = start;
    for (i, ms) in intents {
        c.apply(i, *ms as f32 / 1000.0);
    }
    c
}

/// A request to change one voxel. It is a request only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditIntent {
    /// Put `material` into the (empty) voxel.
    Place {
        /// The voxel.
        voxel: [i64; 3],
        /// Material (non-zero).
        material: u16,
    },
    /// Clear the voxel.
    Remove {
        /// The voxel.
        voxel: [i64; 3],
    },
}

/// Where edit requests go. The implementation owns the world (an in-process host, later a
/// server); the client and the adapter never commit.
pub trait EditPort: Send + Sync {
    /// Submits a request; `Ok` means the authoritative side applied it.
    fn submit(&self, edit: EditIntent) -> Result<(), String>;
}

/// What a ray hit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hit {
    /// The solid voxel that was hit.
    pub voxel: [i64; 3],
    /// The empty voxel in front of the hit face (where a block would be placed).
    pub front: [i64; 3],
    /// Material of the hit voxel.
    pub material: u16,
}

/// The faces the picker may look at.
pub struct FaceIndex {
    faces: HashSet<([i64; 3], Dir)>,
    materials: std::collections::HashMap<[i64; 3], u16>,
}

impl FaceIndex {
    /// Indexes faces of any number of bricks.
    pub fn new<'a>(faces: impl IntoIterator<Item = &'a Face>) -> Self {
        let mut set = HashSet::new();
        let mut materials = std::collections::HashMap::new();
        for f in faces {
            let v = [f.voxel.x, f.voxel.y, f.voxel.z];
            set.insert((v, f.dir));
            materials.insert(v, f.material);
        }
        Self {
            faces: set,
            materials,
        }
    }
}

fn dir_of(axis: usize, step: i64) -> Dir {
    match (axis, step > 0) {
        (0, true) => Dir::PosX,
        (0, false) => Dir::NegX,
        (1, true) => Dir::PosY,
        (1, false) => Dir::NegY,
        (_, true) => Dir::PosZ,
        (_, false) => Dir::NegZ,
    }
}

/// Walks the ray `origin + t * dir` (`dir` need not be normalised) through the voxel grid and
/// returns the first face turned towards it within `max_dist` voxels, or `None`.
pub fn pick(index: &FaceIndex, origin: [f64; 3], dir: [f64; 3], max_dist: f64) -> Option<Hit> {
    let len = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
    if len == 0.0 || !len.is_finite() {
        return None;
    }
    let d = dir.map(|c| c / len);
    let mut cell = origin.map(|c| c.floor() as i64);
    let step: [i64; 3] = d.map(|c| if c >= 0.0 { 1 } else { -1 });
    let mut t_max = [0.0f64; 3];
    let mut t_delta = [f64::INFINITY; 3];
    for a in 0..3 {
        if d[a] != 0.0 {
            let next = if step[a] > 0 {
                cell[a] as f64 + 1.0
            } else {
                cell[a] as f64
            };
            t_max[a] = (next - origin[a]) / d[a];
            t_delta[a] = (1.0 / d[a]).abs();
        } else {
            t_max[a] = f64::INFINITY;
        }
    }
    loop {
        let a = if t_max[0] <= t_max[1] && t_max[0] <= t_max[2] {
            0
        } else if t_max[1] <= t_max[2] {
            1
        } else {
            2
        };
        if t_max[a] > max_dist {
            return None;
        }
        let front = cell;
        cell[a] += step[a];
        t_max[a] += t_delta[a];
        // Entering `cell` through the side that faces back towards `front`.
        if index.faces.contains(&(cell, dir_of(a, -step[a]))) {
            return Some(Hit {
                voxel: cell,
                front,
                material: index.materials.get(&cell).copied().unwrap_or(0),
            });
        }
    }
}

/// Turns the pointer into an edit request: `place` puts a block in front of the hit face,
/// otherwise the hit block is removed. `None` if nothing was hit within `reach`.
pub fn edit_from_camera(
    index: &FaceIndex,
    cam: &Camera,
    reach: f64,
    place: Option<u16>,
) -> Option<EditIntent> {
    let hit = pick(index, cam.pos, cam.forward(), reach)?;
    Some(match place {
        Some(material) => EditIntent::Place {
            voxel: hit.front,
            material,
        },
        None => EditIntent::Remove { voxel: hit.voxel },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cube_core::VoxelPos;

    fn block(x: i64, y: i64, z: i64, m: u16) -> Vec<Face> {
        Dir::ALL
            .iter()
            .map(|d| Face {
                voxel: VoxelPos::new(x, y, z),
                dir: *d,
                material: m,
            })
            .collect()
    }

    #[test]
    fn replay_is_deterministic_and_turning_does_not_depend_on_time() {
        let start = Camera::new([0.5, 10.0, 0.5]);
        let script = [
            (
                Intent::Look {
                    yaw: 0.7,
                    pitch: -0.2,
                },
                16,
            ),
            (
                Intent::Move {
                    forward: 1,
                    right: 0,
                    up: 0,
                },
                16,
            ),
            (
                Intent::Move {
                    forward: 1,
                    right: 1,
                    up: 1,
                },
                33,
            ),
            (Intent::Speed(1e9), 1),
            (
                Intent::Move {
                    forward: -1,
                    right: 0,
                    up: 0,
                },
                100,
            ),
        ];
        let a = replay(start, &script);
        assert_eq!(a, replay(start, &script));
        assert_eq!(a.speed, MAX_SPEED, "speed is clamped");
        let mut b = start;
        b.apply(
            &Intent::Look {
                yaw: 0.5,
                pitch: 0.0,
            },
            0.0,
        );
        let mut c = start;
        c.apply(
            &Intent::Look {
                yaw: 0.5,
                pitch: 0.0,
            },
            10.0,
        );
        assert_eq!(b, c);
        let mut d = start;
        d.apply(
            &Intent::Look {
                yaw: 0.0,
                pitch: 9.0,
            },
            0.0,
        );
        assert!(d.pitch <= MAX_PITCH, "pitch never flips the view");
    }

    #[test]
    fn the_camera_moves_along_its_view_and_knows_its_brick_in_negative_space() {
        let mut cam = Camera::new([0.5, 0.5, 0.5]);
        cam.apply(
            &Intent::Move {
                forward: 1,
                right: 0,
                up: 0,
            },
            1.0,
        );
        assert!(
            (cam.pos[2] - (0.5 - 12.0)).abs() < 1e-9,
            "forward is -Z at yaw 0"
        );
        assert_eq!(cam.brick(), cube_core::BrickCoord::new(0, 0, -1));
        cam.pos = [-0.5, -16.5, 31.9];
        assert_eq!(cam.brick(), cube_core::BrickCoord::new(-1, -2, 1));
    }

    #[test]
    fn a_ray_hits_the_nearest_face_and_reports_where_a_block_would_go() {
        let mut faces = block(0, 0, 5, 3);
        faces.extend(block(0, 0, 8, 4));
        let idx = FaceIndex::new(&faces);
        let hit = pick(&idx, [0.5, 0.5, 0.0], [0.0, 0.0, 1.0], 20.0).unwrap();
        assert_eq!(
            (hit.voxel, hit.front, hit.material),
            ([0, 0, 5], [0, 0, 4], 3)
        );
        assert!(
            pick(&idx, [0.5, 0.5, 0.0], [0.0, 0.0, 1.0], 3.0).is_none(),
            "out of reach"
        );
        assert!(pick(&idx, [0.5, 0.5, 0.0], [0.0, 0.0, 0.0], 20.0).is_none());
        // From behind and from the side, in negative space.
        let hit = pick(&idx, [0.5, 0.5, 20.0], [0.0, 0.0, -1.0], 30.0).unwrap();
        assert_eq!((hit.voxel, hit.front), ([0, 0, 8], [0, 0, 9]));
        let n = FaceIndex::new(&block(-3, -2, -1, 1));
        let hit = pick(&n, [3.0, -1.5, -0.5], [-1.0, 0.0, 0.0], 20.0).unwrap();
        assert_eq!((hit.voxel, hit.front), ([-3, -2, -1], [-2, -2, -1]));
    }

    #[test]
    fn edit_requests_are_derived_from_what_the_camera_sees() {
        let idx = FaceIndex::new(&block(0, 0, -4, 2));
        let cam = Camera::new([0.5, 0.5, 0.5]);
        assert_eq!(
            edit_from_camera(&idx, &cam, 10.0, Some(7)),
            Some(EditIntent::Place {
                voxel: [0, 0, -3],
                material: 7
            })
        );
        assert_eq!(
            edit_from_camera(&idx, &cam, 10.0, None),
            Some(EditIntent::Remove { voxel: [0, 0, -4] })
        );
        assert_eq!(edit_from_camera(&idx, &cam, 2.0, None), None);
    }
}
