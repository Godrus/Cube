//! The first character: a fantasy traveller (human, about 1.8 units tall, facing -Z).
//!
//! Model space: units (1 unit = 1 world voxel of the Cube world), feet at y = 0, +X is the
//! character's right hand, -Z is forward. Every part is defined continuously (see `sculpt`) and
//! rasterised at the requested density, so the lattice is shared by all parts and neighbouring
//! parts meet without gaps.

use crate::sculpt::{Op, Shape, bx, sup};

/// Material indices (index into [`palette`]); 0 is empty.
pub mod m {
    /// Skin.
    pub const SKIN: u8 = 1;
    /// Skin shadow.
    pub const SKIN_SHADE: u8 = 2;
    /// Skin blush.
    pub const SKIN_BLUSH: u8 = 3;
    /// Hair, dark.
    pub const HAIR_DARK: u8 = 4;
    /// Hair, mid.
    pub const HAIR_MID: u8 = 5;
    /// Hair, light.
    pub const HAIR_LIGHT: u8 = 6;
    /// Eye white.
    pub const EYE_WHITE: u8 = 7;
    /// Eye dark.
    pub const EYE_DARK: u8 = 8;
    /// Mouth.
    pub const MOUTH: u8 = 9;
    /// Tunic cream.
    pub const CREAM: u8 = 10;
    /// Tunic cream shade.
    pub const CREAM_SHADE: u8 = 11;
    /// Sleeve cuff.
    pub const CREAM_LIGHT: u8 = 12;
    /// Tabard grey.
    pub const GREY: u8 = 13;
    /// Tabard grey light.
    pub const GREY_LIGHT: u8 = 14;
    /// Gold trim.
    pub const GOLD: u8 = 15;
    /// Leather.
    pub const LEATHER: u8 = 16;
    /// Leather dark.
    pub const LEATHER_DARK: u8 = 17;
    /// Leather light.
    pub const LEATHER_LIGHT: u8 = 18;
    /// Cape red.
    pub const RED: u8 = 19;
    /// Cape red dark.
    pub const RED_DARK: u8 = 20;
    /// Armour grey.
    pub const ARMOR: u8 = 21;
    /// Armour dark.
    pub const ARMOR_DARK: u8 = 22;
    /// Trousers.
    pub const PANTS: u8 = 23;
    /// Trousers dark.
    pub const PANTS_DARK: u8 = 24;
    /// Boot brown.
    pub const BOOT: u8 = 25;
    /// Boot light.
    pub const BOOT_LIGHT: u8 = 26;
    /// Boot dark.
    pub const BOOT_DARK: u8 = 27;
    /// Glove.
    pub const GLOVE: u8 = 28;
    /// Steel.
    pub const STEEL: u8 = 29;
    /// Steel light.
    pub const STEEL_LIGHT: u8 = 30;
    /// Steel dark.
    pub const STEEL_DARK: u8 = 31;
}
use m::*;

/// Number of palette entries (index 0 is unused).
pub const PALETTE_LEN: usize = 32;

/// Material colours in sRGB, index = material.
pub fn palette() -> [[f32; 3]; PALETTE_LEN] {
    let c = |r: u8, g: u8, b: u8| {
        [
            f32::from(r) / 255.0,
            f32::from(g) / 255.0,
            f32::from(b) / 255.0,
        ]
    };
    let mut p = [[1.0, 0.0, 1.0]; PALETTE_LEN];
    p[0] = c(0, 0, 0);
    p[SKIN as usize] = c(238, 178, 140);
    p[SKIN_SHADE as usize] = c(205, 144, 112);
    p[SKIN_BLUSH as usize] = c(226, 140, 118);
    p[HAIR_DARK as usize] = c(84, 44, 30);
    p[HAIR_MID as usize] = c(122, 66, 44);
    p[HAIR_LIGHT as usize] = c(150, 88, 58);
    p[EYE_WHITE as usize] = c(240, 232, 226);
    p[EYE_DARK as usize] = c(40, 24, 22);
    p[MOUTH as usize] = c(170, 96, 86);
    p[CREAM as usize] = c(206, 196, 180);
    p[CREAM_SHADE as usize] = c(176, 164, 150);
    p[CREAM_LIGHT as usize] = c(226, 218, 204);
    p[GREY as usize] = c(112, 108, 114);
    p[GREY_LIGHT as usize] = c(142, 138, 142);
    p[GOLD as usize] = c(214, 170, 78);
    p[LEATHER as usize] = c(140, 90, 52);
    p[LEATHER_DARK as usize] = c(92, 58, 36);
    p[LEATHER_LIGHT as usize] = c(168, 114, 68);
    p[RED as usize] = c(170, 40, 36);
    p[RED_DARK as usize] = c(118, 28, 30);
    p[ARMOR as usize] = c(128, 124, 140);
    p[ARMOR_DARK as usize] = c(86, 82, 98);
    p[PANTS as usize] = c(84, 78, 80);
    p[PANTS_DARK as usize] = c(62, 58, 62);
    p[BOOT as usize] = c(118, 74, 46);
    p[BOOT_LIGHT as usize] = c(150, 98, 62);
    p[BOOT_DARK as usize] = c(76, 46, 32);
    p[GLOVE as usize] = c(82, 56, 50);
    p[STEEL as usize] = c(176, 176, 190);
    p[STEEL_LIGHT as usize] = c(222, 222, 232);
    p[STEEL_DARK as usize] = c(118, 118, 134);
    p
}

/// Rig part identifiers (stable; the order is the asset order).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PartId {
    /// Torso with tunic, belt, pouch and shawl.
    Torso,
    /// Head with neck, face and hair.
    Head,
    /// Character's left arm (-X).
    ArmL,
    /// Character's right arm (+X).
    ArmR,
    /// Character's left leg with boot.
    LegL,
    /// Character's right leg with boot.
    LegR,
    /// Cape.
    Cape,
    /// Sword (held in the right hand).
    Sword,
}

impl PartId {
    /// All parts in asset order.
    pub const ALL: [PartId; 8] = [
        PartId::Torso,
        PartId::Head,
        PartId::ArmL,
        PartId::ArmR,
        PartId::LegL,
        PartId::LegR,
        PartId::Cape,
        PartId::Sword,
    ];

    /// Stable name.
    pub const fn name(self) -> &'static str {
        match self {
            PartId::Torso => "torso",
            PartId::Head => "head",
            PartId::ArmL => "arm_l",
            PartId::ArmR => "arm_r",
            PartId::LegL => "leg_l",
            PartId::LegR => "leg_r",
            PartId::Cape => "cape",
            PartId::Sword => "sword",
        }
    }

    /// Parent in the rig (`None` = root).
    pub const fn parent(self) -> Option<PartId> {
        match self {
            PartId::Torso | PartId::LegL | PartId::LegR => None,
            PartId::Head | PartId::ArmL | PartId::ArmR | PartId::Cape => Some(PartId::Torso),
            PartId::Sword => Some(PartId::ArmR),
        }
    }

    /// Joint position in model space (the part's mesh is relative to it).
    pub const fn pivot(self) -> [f32; 3] {
        match self {
            PartId::Torso => [0.0, 1.0, 0.0],
            PartId::Head => [0.0, 1.50, 0.0],
            PartId::ArmL => [-0.245, 1.40, 0.0],
            PartId::ArmR => [0.245, 1.40, 0.0],
            PartId::LegL => [-0.095, 0.90, 0.0],
            PartId::LegR => [0.095, 0.90, 0.0],
            PartId::Cape => [0.0, 1.46, 0.12],
            PartId::Sword => [0.262, 0.885, -0.02],
        }
    }

    /// Origin the part's mesh is measured from: the pivot for body parts (sculpted in model
    /// space), zero for the sword (sculpted around its own grip).
    pub const fn mesh_origin(self) -> [f32; 3] {
        match self {
            PartId::Sword => [0.0, 0.0, 0.0],
            _ => self.pivot(),
        }
    }

    /// Sculpt operations of the part.
    pub fn ops(self) -> Vec<Op> {
        match self {
            PartId::Torso => torso(),
            PartId::Head => head(),
            PartId::ArmL => arm(-1.0),
            PartId::ArmR => arm(1.0),
            PartId::LegL => leg(-1.0),
            PartId::LegR => leg(1.0),
            PartId::Cape => cape(),
            PartId::Sword => sword(),
        }
    }
}

fn h(a: u32, b: u32) -> f32 {
    let mut x = a.wrapping_mul(0x9E37_79B1) ^ b.wrapping_mul(0x85EB_CA77);
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^= x >> 13;
    (x & 0xFFFF) as f32 / 65535.0
}

fn torso() -> Vec<Op> {
    let mut o = vec![
        // tunic skirt body and chest
        Op::add(sup([0.0, 0.97, 0.0], [0.200, 0.16, 0.125], 4.0), GREY).vary(GREY_LIGHT, 0.30),
        Op::add(sup([0.0, 1.26, 0.0], [0.190, 0.19, 0.115], 3.2), CREAM).vary(CREAM_SHADE, 0.30),
        Op::add(sup([0.0, 1.44, 0.0], [0.120, 0.03, 0.090], 2.0), CREAM),
        // front and back tabard with gold edge
        Op::add(bx([-0.118, 0.81, -0.140], [0.118, 1.13, -0.100]), GREY).vary(GREY_LIGHT, 0.30),
        Op::add(bx([-0.118, 0.81, 0.100], [0.118, 1.13, 0.140]), GREY).vary(GREY_LIGHT, 0.30),
        Op::add(bx([-0.118, 0.81, -0.140], [-0.100, 1.13, -0.100]), GOLD),
        Op::add(bx([0.100, 0.81, -0.140], [0.118, 1.13, -0.100]), GOLD),
        Op::add(bx([-0.118, 0.81, -0.140], [0.118, 0.845, -0.100]), GOLD),
        Op::add(bx([-0.118, 0.81, 0.100], [0.118, 0.845, 0.140]), GOLD),
        // dark tabard pattern
        Op::paint(
            bx([-0.050, 0.87, -0.150], [-0.015, 0.915, -0.090]),
            ARMOR_DARK,
        ),
        Op::paint(
            bx([0.015, 0.90, -0.150], [0.050, 0.945, -0.090]),
            ARMOR_DARK,
        ),
        Op::paint(
            bx([-0.030, 0.96, -0.150], [0.005, 0.995, -0.090]),
            ARMOR_DARK,
        ),
        // belt and buckle
        Op::add(sup([0.0, 1.09, 0.0], [0.205, 0.035, 0.130], 4.0), LEATHER).vary(LEATHER_DARK, 0.2),
        Op::add(bx([-0.034, 1.05, -0.148], [0.034, 1.13, -0.118]), GOLD),
        Op::add(
            bx([-0.016, 1.068, -0.152], [0.016, 1.112, -0.118]),
            LEATHER_DARK,
        ),
        // pouch with flap and clasp (character's left hip)
        Op::add(bx([-0.290, 0.95, -0.075], [-0.205, 1.07, 0.070]), LEATHER).vary(LEATHER_DARK, 0.2),
        Op::add(
            bx([-0.295, 1.02, -0.080], [-0.200, 1.085, 0.075]),
            LEATHER_LIGHT,
        ),
        Op::add(bx([-0.262, 1.00, -0.088], [-0.234, 1.04, -0.076]), GOLD),
        // chest strap
        Op::paint(
            Shape::Band {
                a: [0.150, 1.47],
                b: [-0.180, 1.10],
                hw: 0.024,
            },
            LEATHER,
        )
        .vary(LEATHER_DARK, 0.25)
        .clip([-0.3, 1.05, -0.2], [0.3, 1.5, 0.2]),
        // red shawl with a hole for the neck
        Op::add(sup([0.0, 1.435, 0.005], [0.220, 0.055, 0.145], 2.4), RED).vary(RED_DARK, 0.35),
        Op::carve(sup([0.0, 1.45, 0.0], [0.043, 0.2, 0.043], 2.0)),
        // cape clasp
        Op::add(
            sup([0.120, 1.425, -0.108], [0.030, 0.030, 0.016], 2.2),
            GOLD,
        ),
    ];
    // shawl fringe: ragged red tassels below the shawl on the right shoulder
    for i in 0..6 {
        let x = 0.08 + 0.025 * i as f32;
        let len = 0.04 + 0.05 * h(i, 3);
        o.push(Op::add(
            bx([x, 1.38 - len, -0.09], [x + 0.022, 1.40, -0.07]),
            RED_DARK,
        ));
    }
    o
}

fn head() -> Vec<Op> {
    let face_c = [0.0, 1.655, 0.0];
    let mut o = vec![
        // neck
        Op::add(sup([0.0, 1.50, 0.005], [0.046, 0.065, 0.048], 2.4), SKIN).vary(SKIN_SHADE, 0.25),
        // skull, narrowed jaw
        Op::add(sup(face_c, [0.105, 0.125, 0.113], 3.0), SKIN).vary(SKIN_SHADE, 0.10),
        Op::carve(bx([0.074, 1.50, -0.2], [0.2, 1.585, 0.2])),
        Op::carve(bx([-0.2, 1.50, -0.2], [-0.074, 1.585, 0.2])),
        Op::carve(bx([0.058, 1.50, -0.2], [0.2, 1.55, -0.07])),
        Op::carve(bx([-0.2, 1.50, -0.2], [-0.058, 1.55, -0.07])),
        // ears
        Op::add(bx([0.100, 1.625, -0.020], [0.118, 1.68, 0.030]), SKIN_SHADE),
        Op::add(
            bx([-0.118, 1.625, -0.020], [-0.100, 1.68, 0.030]),
            SKIN_SHADE,
        ),
        // nose and its shadow
        Op::add(bx([-0.014, 1.600, -0.140], [0.014, 1.648, -0.100]), SKIN),
        Op::add(
            bx([-0.013, 1.596, -0.146], [0.013, 1.612, -0.118]),
            SKIN_SHADE,
        ),
        // brows
        Op::add(
            bx([0.024, 1.683, -0.126], [0.076, 1.699, -0.100]),
            HAIR_DARK,
        ),
        Op::add(
            bx([-0.076, 1.683, -0.126], [-0.024, 1.699, -0.100]),
            HAIR_DARK,
        ),
        // eyes: white, then the dark iris, lids above
        Op::paint(
            bx([0.028, 1.640, -0.140], [0.070, 1.676, -0.090]),
            EYE_WHITE,
        ),
        Op::paint(
            bx([-0.070, 1.640, -0.140], [-0.028, 1.676, -0.090]),
            EYE_WHITE,
        ),
        Op::paint(bx([0.040, 1.640, -0.140], [0.060, 1.676, -0.090]), EYE_DARK),
        Op::paint(
            bx([-0.060, 1.640, -0.140], [-0.040, 1.676, -0.090]),
            EYE_DARK,
        ),
        Op::paint(
            bx([0.026, 1.676, -0.140], [0.072, 1.684, -0.090]),
            SKIN_SHADE,
        ),
        Op::paint(
            bx([-0.072, 1.676, -0.140], [-0.026, 1.684, -0.090]),
            SKIN_SHADE,
        ),
        // cheeks and mouth
        Op::paint(
            bx([0.046, 1.600, -0.140], [0.082, 1.622, -0.090]),
            SKIN_BLUSH,
        ),
        Op::paint(
            bx([-0.082, 1.600, -0.140], [-0.046, 1.622, -0.090]),
            SKIN_BLUSH,
        ),
        Op::paint(bx([-0.030, 1.568, -0.140], [0.030, 1.582, -0.090]), MOUTH),
        Op::paint(
            bx([-0.016, 1.553, -0.140], [0.016, 1.565, -0.090]),
            SKIN_SHADE,
        ),
    ];
    // hair: cap (not over the face), nape, sideburns
    let hc = [0.0, 1.70, 0.012];
    o.push(
        Op::add(sup(hc, [0.119, 0.100, 0.128], 2.6), HAIR_MID)
            .vary(HAIR_DARK, 0.45)
            .clip([-0.2, 1.60, -0.045], [0.2, 1.82, 0.2]),
    );
    o.push(
        Op::add(sup(hc, [0.119, 0.100, 0.128], 2.6), HAIR_MID)
            .vary(HAIR_DARK, 0.45)
            .clip([-0.2, 1.718, -0.2], [0.2, 1.82, 0.2]),
    );
    o.push(
        Op::add(
            sup([0.0, 1.625, 0.065], [0.108, 0.090, 0.075], 2.4),
            HAIR_DARK,
        )
        .vary(HAIR_MID, 0.35)
        .clip([-0.2, 1.59, 0.0], [0.2, 1.8, 0.2]),
    );
    o.push(Op::add(
        bx([0.094, 1.60, -0.075], [0.118, 1.69, -0.020]),
        HAIR_DARK,
    ));
    o.push(Op::add(
        bx([-0.118, 1.60, -0.075], [-0.094, 1.69, -0.020]),
        HAIR_DARK,
    ));
    // curls over the shell (fringe above the brow, crown and back)
    for i in 0..58u32 {
        let th = h(i, 11) * std::f32::consts::TAU;
        let ph = h(i, 23).powf(0.7) * 1.45;
        let r = 0.026 + 0.018 * h(i, 31);
        let p = [
            hc[0] + 0.122 * ph.cos() * th.cos(),
            hc[1] + 0.105 * ph.sin(),
            hc[2] + 0.130 * ph.cos() * th.sin(),
        ];
        // keep curls off the face: nothing below the fringe line in front
        if p[1] < 1.715 && p[2] < -0.04 {
            continue;
        }
        if p[1] < 1.64 {
            continue;
        }
        let mat = match (h(i, 41) * 3.0) as u32 {
            0 => HAIR_DARK,
            1 => HAIR_MID,
            _ => HAIR_LIGHT,
        };
        o.push(Op::add(sup(p, [r, r * 0.9, r], 2.2), mat));
    }
    for i in 0..7 {
        let x = -0.090 + 0.030 * i as f32;
        let y = 1.722 + 0.012 * h(i, 5);
        o.push(Op::add(
            sup([x, y, -0.108 + 0.01 * h(i, 9)], [0.026, 0.026, 0.024], 2.2),
            if i % 2 == 0 { HAIR_MID } else { HAIR_DARK },
        ));
    }
    o
}

fn arm(s: f32) -> Vec<Op> {
    let x = 0.256 * s;
    let mut o = vec![
        Op::add(
            sup([0.245 * s, 1.385, 0.0], [0.064, 0.064, 0.064], 2.0),
            CREAM,
        ),
        Op::add(sup([x, 1.30, 0.0], [0.058, 0.115, 0.058], 2.6), CREAM).vary(CREAM_SHADE, 0.30),
        Op::add(
            sup([x, 1.185, 0.0], [0.068, 0.027, 0.068], 3.0),
            CREAM_LIGHT,
        ),
        Op::add(
            sup([x + 0.002 * s, 1.09, -0.004], [0.048, 0.105, 0.048], 2.5),
            SKIN,
        )
        .vary(SKIN_SHADE, 0.20),
        Op::add(
            sup([x + 0.004 * s, 0.965, -0.004], [0.053, 0.056, 0.053], 3.0),
            GLOVE,
        )
        .vary(LEATHER_DARK, 0.3),
        Op::paint(
            bx([x - 0.07, 0.940, -0.07], [x + 0.07, 0.952, 0.07]),
            LEATHER_DARK,
        ),
        Op::paint(
            bx([x - 0.07, 0.985, -0.07], [x + 0.07, 0.997, 0.07]),
            LEATHER_DARK,
        ),
        Op::add(
            sup([x + 0.006 * s, 0.885, -0.008], [0.040, 0.050, 0.044], 2.6),
            GLOVE,
        )
        .vary(LEATHER_DARK, 0.3),
        Op::add(
            bx([x - 0.030, 0.830, -0.040], [x + 0.042, 0.860, 0.030]),
            SKIN_SHADE,
        ),
    ];
    if s < 0.0 {
        // large pauldron with gold trim and a second plate
        o.push(
            Op::add(sup([-0.262, 1.425, 0.0], [0.088, 0.062, 0.092], 2.4), ARMOR)
                .vary(ARMOR_DARK, 0.30),
        );
        o.push(Op::add(
            sup([-0.272, 1.385, 0.0], [0.078, 0.040, 0.082], 2.4),
            ARMOR_DARK,
        ));
        o.push(Op::paint(
            bx([-0.4, 1.350, -0.2], [-0.15, 1.375, 0.2]),
            GOLD,
        ));
        o.push(Op::paint(
            bx([-0.4, 1.455, -0.2], [-0.15, 1.480, 0.2]),
            GOLD,
        ));
    } else {
        o.push(
            Op::add(sup([0.262, 1.415, 0.0], [0.072, 0.050, 0.078], 2.4), RED).vary(RED_DARK, 0.35),
        );
    }
    o
}

fn leg(s: f32) -> Vec<Op> {
    let x = 0.095 * s;
    vec![
        Op::add(sup([x, 0.74, 0.0], [0.090, 0.170, 0.090], 2.6), PANTS).vary(PANTS_DARK, 0.30),
        Op::add(
            sup([x, 0.565, -0.010], [0.078, 0.030, 0.084], 3.0),
            ARMOR_DARK,
        ),
        Op::add(sup([x, 0.475, 0.0], [0.072, 0.120, 0.074], 2.6), PANTS).vary(PANTS_DARK, 0.30),
        // boot shaft, cuff, strap and buckle
        Op::add(sup([x, 0.25, 0.005], [0.086, 0.155, 0.090], 3.2), BOOT).vary(BOOT_LIGHT, 0.30),
        Op::add(sup([x, 0.378, 0.0], [0.096, 0.036, 0.100], 3.5), BOOT_LIGHT).vary(BOOT, 0.3),
        Op::paint(
            bx([x - 0.12, 0.290, -0.12], [x + 0.12, 0.325, 0.12]),
            BOOT_DARK,
        ),
        Op::add(
            bx([x - 0.026, 0.285, -0.104], [x + 0.026, 0.330, -0.088]),
            GOLD,
        ),
        Op::add(
            bx([x - 0.012, 0.298, -0.108], [x + 0.012, 0.32, -0.088]),
            BOOT_DARK,
        ),
        // foot, toe cap, sole
        Op::add(sup([x, 0.060, -0.050], [0.078, 0.060, 0.140], 3.2), BOOT).vary(BOOT_LIGHT, 0.25),
        Op::paint(
            bx([x - 0.12, 0.0, -0.20], [x + 0.12, 0.11, -0.125]),
            BOOT_LIGHT,
        ),
        Op::add(
            sup([x, 0.018, -0.055], [0.082, 0.020, 0.145], 4.0),
            BOOT_DARK,
        ),
    ]
}

fn cape() -> Vec<Op> {
    let mut o = Vec::new();
    let step = 0.014;
    let n = 38;
    for i in 0..n {
        let x0 = -0.266 + step * i as f32;
        let t = i as f32 / (n - 1) as f32;
        // left side (-X) is longer, right side shorter, ragged hem from a hash
        let base = 0.60 + 0.14 * (1.0 - t) - 0.10 * t;
        let len = base - 0.07 * h(i as u32, 77) + 0.03 * ((i as f32 * 0.9).sin());
        let fold = (i / 3) % 2 == 0;
        let mat = if fold { RED } else { RED_DARK };
        let tiers = 5;
        for k in 0..tiers {
            let y_hi = 1.46 - len * k as f32 / tiers as f32;
            let y_lo = 1.46 - len * (k + 1) as f32 / tiers as f32;
            let z_lo = 0.150 + 0.014 * (k * k) as f32 * 0.5 + 0.006 * ((i as f32 * 0.5).sin());
            o.push(Op::add(
                bx([x0, y_lo, z_lo], [x0 + step, y_hi + 0.004, z_lo + 0.042]),
                mat,
            ));
        }
    }
    o.push(
        Op::paint(bx([-0.3, 0.60, 0.1], [0.3, 0.80, 0.3]), RED_DARK)
            .clip([-0.3, 0.60, 0.1], [0.3, 0.74, 0.3]),
    );
    o
}

fn sword() -> Vec<Op> {
    let mut o = vec![
        Op::add(
            bx([-0.017, -0.075, -0.017], [0.017, 0.062, 0.017]),
            LEATHER_DARK,
        ),
        Op::paint(bx([-0.03, -0.040, -0.03], [0.03, -0.028, 0.03]), LEATHER),
        Op::paint(bx([-0.03, 0.005, -0.03], [0.03, 0.017, 0.03]), LEATHER),
        Op::add(sup([0.0, -0.090, 0.0], [0.028, 0.024, 0.028], 2.2), GOLD),
        Op::add(bx([-0.078, 0.060, -0.021], [0.078, 0.086, 0.021]), GOLD),
        Op::add(sup([-0.078, 0.073, 0.0], [0.012, 0.020, 0.024], 2.0), GOLD),
        Op::add(sup([0.078, 0.073, 0.0], [0.012, 0.020, 0.024], 2.0), GOLD),
    ];
    // blade: two-tone (light edge facet, steel facet), tapering to a point
    let mut y = 0.086;
    while y < 0.82 {
        let hw = if y < 0.66 {
            0.034
        } else {
            0.034 * (0.82 - y) / 0.16 + 0.004
        };
        o.push(Op::add(
            bx([-hw, y, -0.016], [0.0, y + 0.012, 0.016]),
            STEEL_LIGHT,
        ));
        o.push(Op::add(bx([0.0, y, -0.016], [hw, y + 0.012, 0.016]), STEEL));
        y += 0.010;
    }
    o.push(Op::paint(
        bx([-0.005, 0.13, -0.05], [0.005, 0.62, 0.05]),
        STEEL_DARK,
    ));
    o
}
