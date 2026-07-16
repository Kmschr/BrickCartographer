mod primitives;
mod wedges;
mod rounds;
mod crests;
mod ramps;
mod misc;

pub use self::primitives::*;
pub use self::wedges::*;
pub use self::rounds::*;
pub use self::crests::*;
pub use self::ramps::*;
pub use self::misc::*;

use brickadia::save::Brick;

// Single source of truth for brick-name → shape classification. Vertex, outline
// and occlusion logic all switch on this so a new brick is declared in one place.
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum BrickKind {
    Corner,
    SideWedge,
    Wedge,
    Ramp,
    RampCorner,
    RampCornerInverted,
    RampCrest,
    RampCrestEnd,
    Round,
    Rect,
}

impl BrickKind {
    pub fn from_name(name: &str) -> BrickKind {
        match name {
            "B_2x2_Corner" => BrickKind::Corner,
            "PB_DefaultSideWedge" | "PB_DefaultSideWedgeTile" | "PB_DefaultMicroWedge" => BrickKind::SideWedge,
            "PB_DefaultWedge" => BrickKind::Wedge,
            "PB_DefaultRamp" => BrickKind::Ramp,
            "PB_DefaultRampCorner" => BrickKind::RampCorner,
            "PB_DefaultRampCornerInverted" => BrickKind::RampCornerInverted,
            "PB_DefaultRampCrest" => BrickKind::RampCrest,
            "PB_DefaultRampCrestEnd" => BrickKind::RampCrestEnd,
            "PB_DefaultPole" | "B_1x1F_Round" | "B_1x1_Round" | "B_2x2F_Round" | "B_2x2_Round" | "B_4x4_Round" => BrickKind::Round,
            _ => BrickKind::Rect,
        }
    }
}

pub fn calculate_brick_vertices(name: &str, brick: &Brick) -> Vec<f32> {
    let shape = Shape::from(brick);
    match BrickKind::from_name(name) {
        BrickKind::Corner => corner(brick, &shape),
        BrickKind::SideWedge => side_wedge(brick, &shape),
        BrickKind::Wedge => wedge(brick, &shape),
        BrickKind::Ramp => ramp(brick, &shape),
        BrickKind::RampCorner => ramp_corner(brick, &shape),
        BrickKind::RampCornerInverted => ramp_corner_inverted(brick, &shape),
        BrickKind::RampCrest => ramp_crest(brick, &shape),
        BrickKind::RampCrestEnd => ramp_crest_end(brick, &shape),
        BrickKind::Round => round(brick, &shape),
        BrickKind::Rect => rec(&shape),
    }
}

pub fn calculate_brick_outline_vertices(name: &str, brick: &Brick) -> Vec<f32> {
    let shape = Shape::from(brick);
    match BrickKind::from_name(name) {
        BrickKind::Corner => corner_ol(brick, &shape),
        BrickKind::SideWedge => side_wedge_ol(brick, &shape),
        BrickKind::Wedge => wedge_ol(brick, &shape),
        BrickKind::Ramp => ramp_ol(brick, &shape),
        BrickKind::RampCorner => ramp_corner_ol(brick, &shape),
        BrickKind::RampCornerInverted => ramp_corner_inverted_ol(brick, &shape),
        BrickKind::RampCrest => ramp_crest_ol(brick, &shape),
        BrickKind::RampCrestEnd => ramp_crest_end_ol(brick, &shape),
        BrickKind::Round => round_ol(brick, &shape),
        BrickKind::Rect => rec_ol(&shape),
    }
}

// Whether a brick's fill always covers its full rectangular footprint, making it
// safe to treat as an occluder of bricks below it. Conservative: shaped bricks
// (wedges, ramps, rounds, corners) never count, even in orientations that render
// as full rectangles.
pub fn is_full_rect(name: &str) -> bool {
    BrickKind::from_name(name) == BrickKind::Rect
}
