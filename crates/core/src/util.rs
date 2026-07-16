use brickadia::save::{Direction, Rotation, Size};

use crate::brick::Brick;

const STUD_WIDTH: u32 = 10;
const STUD_HEIGHT: u32 = 12;
const PLATE_HEIGHT: u32 = 4;

/// Footprint size of a brick after accounting for its asset's fixed size and
/// its rotation/direction, ready to render top-down.
pub fn transform_size(name: &str, procedural_size: (u32, u32, u32), rotation: Rotation, direction: Direction) -> (u16, u16, u16) {
    // Give size to non procedural bricks
    let mut size = match name {
        "B_2x2_Corner" => (STUD_WIDTH, STUD_WIDTH, STUD_HEIGHT / 2),
        "B_2x_Cube_Side" => (STUD_WIDTH, STUD_WIDTH, STUD_HEIGHT),
        "B_1x1_Brick_Side" => (STUD_WIDTH / 2, STUD_WIDTH / 2, STUD_HEIGHT / 2),
        "B_1x4_Brick_Side" => (STUD_WIDTH * 2, STUD_WIDTH / 2, STUD_HEIGHT / 2),
        "B_1x2f_Plate_Center" => (STUD_WIDTH, STUD_WIDTH / 2, STUD_HEIGHT / 2),
        "B_2x2f_Plate_Center" => (STUD_WIDTH, STUD_WIDTH, PLATE_HEIGHT / 2),
        "B_1x2f_Plate_Center_Inv" => (STUD_WIDTH, STUD_WIDTH / 2, STUD_HEIGHT / 2),
        "B_2x2f_Plate_Center_Inv" => (STUD_WIDTH, STUD_WIDTH, PLATE_HEIGHT / 2),
        "B_1x1F_Round" => (STUD_WIDTH / 2, STUD_WIDTH / 2, PLATE_HEIGHT / 2),
        "B_1x1_Round" => (STUD_WIDTH / 2, STUD_WIDTH / 2, STUD_HEIGHT / 2),
        "B_2x2F_Round" => (STUD_WIDTH, STUD_WIDTH, PLATE_HEIGHT / 2),
        "B_2x2_Round" => (STUD_WIDTH, STUD_WIDTH, STUD_HEIGHT / 2),
        "B_4x4_Round" => (STUD_WIDTH * 2, STUD_WIDTH * 2, STUD_HEIGHT / 2),
        _ => procedural_size,
    };

    // Apply Rotation
    if rotation == Rotation::Deg90 || rotation == Rotation::Deg270 {
        std::mem::swap(&mut size.0, &mut size.1);
    }

    // Apply Direction
    if direction == Direction::XPositive || direction == Direction::XNegative {
        std::mem::swap(&mut size.0, &mut size.2);
    }
    else if direction == Direction::YPositive || direction == Direction::YNegative {
        std::mem::swap(&mut size.0, &mut size.1);
        std::mem::swap(&mut size.1, &mut size.2);
    }

    (
        size.0.min(u16::MAX as u32) as u16,
        size.1.min(u16::MAX as u32) as u16,
        size.2.min(u16::MAX as u32) as u16,
    )
}

/// Converts a parsed legacy-format brick into a render-ready one, dropping
/// invisible bricks. `color` is the already-resolved display color.
pub fn slim_brick(brick: &brickadia::save::Brick, brick_assets: &[String], color: [u8; 4]) -> Option<Brick> {
    if !brick.visibility {
        return None;
    }
    let name = &brick_assets[brick.asset_name_index as usize];
    let procedural_size = match brick.size {
        Size::Empty => (0, 0, 0),
        Size::Procedural(x, y, z) => (x, y, z),
    };
    Some(Brick {
        position: brick.position,
        size: transform_size(name, procedural_size, brick.rotation.clone(), brick.direction.clone()),
        asset_name_index: brick.asset_name_index,
        color,
        rotation: brick.rotation.clone(),
        direction: brick.direction.clone(),
    })
}

pub fn calculate_centroid(bricks: &[Brick]) -> (i32, i32) {
    // Sums for calculating Centroid of save
    let mut area_sum: i32 = 0;
    let mut point_sum = (0, 0);

    for brick in bricks {
        let size = sizer(brick);

        // Add to Centroid calculation sums
        let area = size.0 * size.1;
        point_sum.0 += brick.position.0 * area as i32;
        point_sum.1 += brick.position.1 * area as i32;
        area_sum += area as i32;
    }

    // Calculate Centroid
    (point_sum.0 / area_sum, point_sum.1 / area_sum)
}

pub fn calculate_bounds(bricks: &[Brick], (x, y): (i32, i32)) -> (i32, i32, i32, i32) {
    let mut bounds = (std::i32::MAX, std::i32::MAX, std::i32::MIN, std::i32::MIN);

    for brick in bricks {
        let size = sizer(brick);

        let brick_bounds = (
            brick.position.0 - size.0 as i32 - x,
            brick.position.1 - size.1 as i32 - y,
            brick.position.0 + size.0 as i32 - x,
            brick.position.1 + size.1 as i32 - y,
        );

        if brick_bounds.0 < bounds.0 {
            bounds.0 = brick_bounds.0;
        }
        if brick_bounds.1 < bounds.1 {
            bounds.1 = brick_bounds.1;
        }
        if brick_bounds.2 > bounds.2 {
            bounds.2 = brick_bounds.2;
        }
        if brick_bounds.3 > bounds.3 {
            bounds.3 = brick_bounds.3;
        }
    }

    bounds
}

/// Absolute xy extent of the bricks' footprints.
pub fn footprint_bounds(bricks: &[Brick]) -> (i32, i32, i32, i32) {
    let mut bounds = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
    for brick in bricks {
        let size = sizer(brick);
        bounds.0 = bounds.0.min(brick.position.0 - size.0 as i32);
        bounds.1 = bounds.1.min(brick.position.1 - size.1 as i32);
        bounds.2 = bounds.2.max(brick.position.0 + size.0 as i32);
        bounds.3 = bounds.3.max(brick.position.1 + size.1 as i32);
    }
    bounds
}

pub fn top_surface(brick: &Brick) -> i32 {
    brick.position.2 + sizer(brick).2 as i32
}

pub fn sizer(brick: &Brick) -> (u32, u32, u32) {
    brick.size_u32()
}
