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

pub fn calculate_brick_vertices(name: &String, brick: &Brick) -> Vec<f32> {
    let shape = Shape::from(brick);
    match name.as_str() {
        "B_2x2_Corner" =>
            corner(brick, &shape),
        "PB_DefaultSideWedge" | "PB_DefaultSideWedgeTile" | "PB_DefaultMicroWedge" =>
            side_wedge(brick, &shape),
        "PB_DefaultWedge" =>
            wedge(brick, &shape),
        "PB_DefaultRamp" =>
            ramp(brick, &shape),
        "PB_DefaultRampCorner" =>
            ramp_corner(brick, &shape),
        "PB_DefaultRampCornerInverted" =>
            ramp_corner_inverted(brick, &shape),
        "PB_DefaultRampCrest" =>
            ramp_crest(brick, &shape),
        "PB_DefaultRampCrestEnd" =>
            ramp_crest_end(brick, &shape),
        "PB_DefaultPole" | "B_1x1F_Round" | "B_1x1_Round" | "B_2x2F_Round" | "B_2x2_Round" | "B_4x4_Round" =>
            round(brick, &shape),
        _ => 
            rec(&shape),
    }
}

pub fn calculate_brick_outline_vertices(name: &String, brick: &Brick) -> Vec<f32> {
    let shape = Shape::from(brick);
    match name.as_str() {
        "B_2x2_Corner" =>
            corner_ol(brick, &shape),
        "PB_DefaultSideWedge" | "PB_DefaultSideWedgeTile" | "PB_DefaultMicroWedge" =>
            side_wedge_ol(brick, &shape),
        "PB_DefaultWedge" =>
            wedge_ol(brick, &shape),
        "PB_DefaultRamp" =>
            ramp_ol(brick, &shape),
        "PB_DefaultRampCorner" =>
            ramp_corner_ol(brick, &shape),
        "PB_DefaultRampCornerInverted" =>
            ramp_corner_inverted_ol(brick, &shape),
        "PB_DefaultRampCrest" =>
            ramp_crest_ol(brick, &shape),  
        "PB_DefaultRampCrestEnd" =>
            ramp_crest_end_ol(brick, &shape),
        "PB_DefaultPole" | "B_1x1F_Round" | "B_1x1_Round" | "B_2x2F_Round" | "B_2x2_Round" | "B_4x4_Round" =>
            round_ol(brick, &shape),
        _ =>
            rec_ol(&shape)
    }
}