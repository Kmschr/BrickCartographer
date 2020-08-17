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

pub fn calculate_brick_vertices(name: &String, brick: &brs::Brick) -> Vec<f32> {
    let shape = Shape {
        x1: (brick.position.0 - brick.size.0 as i32) as f32,
        y1: (brick.position.1 - brick.size.1 as i32) as f32,
        x2: (brick.position.0 + brick.size.0 as i32) as f32,
        y2: (brick.position.1 + brick.size.1 as i32) as f32
    };

    match name.as_str() {
        "B_2x2_Corner" =>
            corner(brick.direction, brick.rotation, &shape),
        "PB_DefaultSideWedge" | "PB_DefaultSideWedgeTile" =>
            side_wedge(brick.direction, brick.rotation, &shape),
        "PB_DefaultWedge" =>
            wedge(brick.direction, brick.rotation, &shape),
        "PB_DefaultRamp" =>
            ramp(brick.direction, brick.rotation, &shape),
        "PB_DefaultRampCorner" =>
            ramp_corner(brick.direction, brick.rotation, &shape),
        "PB_DefaultRampCornerInverted" =>
            ramp_corner_inverted(brick.direction, brick.rotation, &shape),
        "PB_DefaultRampCrest" =>
            ramp_crest(brick.direction, brick.rotation, &shape),
        "PB_DefaultRampCrestEnd" =>
            ramp_crest_end(brick.direction, brick.rotation, &shape),
        "B_1x1F_Round" | "B_1x1_Round" | "B_2x2F_Round" | "B_2x2_Round" | "B_4x4_Round" =>
            round(brick.direction, &shape),
        _ => 
            rec(&shape),
    }
}

pub fn calculate_brick_outline_vertices(name: &String, brick: &brs::Brick) -> Vec<f32> {
    let shape = Shape {
        x1: (brick.position.0 - brick.size.0 as i32) as f32,
        y1: (brick.position.1 - brick.size.1 as i32) as f32,
        x2: (brick.position.0 + brick.size.0 as i32) as f32,
        y2: (brick.position.1 + brick.size.1 as i32) as f32
    };

    match name.as_str() {
        "B_2x2_Corner" =>
            corner_ol(brick.direction, brick.rotation, &shape),
        "PB_DefaultSideWedge" | "PB_DefaultSideWedgeTile" =>
            side_wedge_ol(brick.direction, brick.rotation, &shape),
        "PB_DefaultWedge" =>
            wedge_ol(brick.direction, brick.rotation, &shape),
        "PB_DefaultRamp" =>
            ramp_ol(brick.direction, brick.rotation, &shape),
        "PB_DefaultRampCorner" =>
            ramp_corner_ol(brick.direction, brick.rotation, &shape),
        "PB_DefaultRampCornerInverted" =>
            ramp_corner_inverted_ol(brick.direction, brick.rotation, &shape),
        "PB_DefaultRampCrest" =>
            ramp_crest_ol(brick.direction, brick.rotation, &shape),  
        "PB_DefaultRampCrestEnd" =>
            ramp_crest_end_ol(brick.direction, brick.rotation, &shape),
        "B_1x1F_Round" | "B_1x1_Round" | "B_2x2F_Round" | "B_2x2_Round" | "B_4x4_Round" =>
            round_ol(brick.direction, &shape),
        _ =>
            rec_ol(&shape)
    }
}