use brickadia::save::{Brick, Rotation, Direction, Size};

const STUD_WIDTH: u32 = 10;
const STUD_HEIGHT: u32 = 12;
const PLATE_HEIGHT: u32 = 4;

pub fn filter_and_transform_brick(brick: Brick, brick_assets: &[String]) -> Option<Brick> {
    if !brick.visibility {
        return None;
    }
    let brick = transform_brick(brick, brick_assets);
    Some(brick)
}

pub fn transform_brick(original_brick: Brick, brick_assets: &[String]) -> Brick {
    let mut brick = original_brick;

    let name = &brick_assets[brick.asset_name_index as usize];

    // Give size to non procedural bricks
    let mut size = match name.as_str() {
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
        _ => sizer(&brick)
    };

    // Apply Rotation
    if brick.rotation == Rotation::Deg90 || brick.rotation == Rotation::Deg270 {
        std::mem::swap(&mut size.0, &mut size.1);
    }

    // Apply Direction
    if brick.direction == Direction::XPositive || brick.direction == Direction::XNegative {
        std::mem::swap(&mut size.0, &mut size.2);
    }
    else if brick.direction == Direction::YPositive || brick.direction == Direction::YNegative {
        std::mem::swap(&mut size.0, &mut size.1);
        std::mem::swap(&mut size.1, &mut size.2);
    }

    brick.size = Size::Procedural(size.0, size.1, size.2);

    brick
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

pub fn top_surface(brick: &Brick) -> i32 {
    brick.position.2 + sizer(brick).2 as i32
}

pub fn sizer(brick: &Brick) -> (u32, u32, u32) {
    match brick.size {
        Size::Empty => (0, 0, 0),
        Size::Procedural(x, y, z) => (x, y, z)
    }
}
