//! View-radius profiles per device class (CLI-03, first part: radius and memory bounds; LOD
//! selection by distance comes with the renderer decision GOV-13).
//!
//! A profile says how many bricks around the camera are shown and how much work one frame may
//! spend. The numbers are engineering defaults with a stated memory bound, **not** device
//! measurements (DG-1..3 need physical devices).

use crate::ClientConfig;
use cube_core::LayerId;
use cube_processing::DerivedKind;

/// A class of device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceClass {
    /// Desktop computer.
    Desktop,
    /// Mid-range phone or tablet.
    MobileMid,
    /// Low-end phone.
    MobileLow,
}

/// Radius and budgets of a device class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewProfile {
    /// The class.
    pub class: DeviceClass,
    /// Bricks shown in each direction around the camera.
    pub radius: u32,
    /// Work units per frame.
    pub work_per_update: usize,
}

/// Rough upper bound of derived data per brick in bytes (faces: a brick has at most
/// 16 * 16 * 16 * 6 faces, practically far fewer; this is the budget, not a measurement).
pub const BYTES_PER_BRICK_BUDGET: u64 = 64 * 1024;

impl ViewProfile {
    /// The profile of a class.
    pub const fn of(class: DeviceClass) -> Self {
        match class {
            DeviceClass::Desktop => Self {
                class,
                radius: 3,
                work_per_update: 64,
            },
            DeviceClass::MobileMid => Self {
                class,
                radius: 2,
                work_per_update: 16,
            },
            DeviceClass::MobileLow => Self {
                class,
                radius: 1,
                work_per_update: 6,
            },
        }
    }

    /// Bricks in the active region.
    pub const fn bricks(&self) -> u64 {
        let side = 2 * self.radius as u64 + 1;
        side * side * side
    }

    /// The memory bound of the derived data for this profile, in bytes.
    pub const fn memory_bound(&self) -> u64 {
        self.bricks() * BYTES_PER_BRICK_BUDGET
    }

    /// The client configuration that matches the profile.
    pub fn config(&self, layer: LayerId, level: i8, kind: DerivedKind) -> ClientConfig {
        ClientConfig {
            layer,
            level,
            kind,
            max_active_bricks: self.bricks(),
            work_per_update: self.work_per_update,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profiles_shrink_with_the_device_and_the_memory_bound_follows_the_radius() {
        let d = ViewProfile::of(DeviceClass::Desktop);
        let m = ViewProfile::of(DeviceClass::MobileMid);
        let l = ViewProfile::of(DeviceClass::MobileLow);
        assert!(d.bricks() > m.bricks() && m.bricks() > l.bricks());
        assert_eq!((d.bricks(), m.bricks(), l.bricks()), (343, 125, 27));
        assert_eq!(l.memory_bound(), 27 * 64 * 1024);
        assert!(l.memory_bound() < 2 * 1024 * 1024);
        let cfg = m.config(LayerId::new(1), 0, DerivedKind(1));
        assert_eq!(cfg.max_active_bricks, 125);
    }
}
