use std::io;
use brs::{Brick, Rotation, Direction};
use bricks::{STUD_WIDTH, STUD_HEIGHT, PLATE_HEIGHT};

pub fn filter_and_transform_brick(brick: io::Result<Brick>, brick_assets: &[String]) -> Option<Brick> {
    match brick {
        Ok(b) => {
            if !b.visibility {
                return None;
            }
            let brick = transform_brick(b, brick_assets);
            Some(brick)
        },
        Err(_e) => None
    }
}

pub fn transform_brick(original_brick: Brick, brick_assets: &[String]) -> Brick {
    let mut brick = original_brick;

    let name = &brick_assets[brick.asset_name_index as usize];

    // Give size to non procedural bricks
    match name.as_str() {
        "B_2x2_Corner" => {
            brick.size.0 = STUD_WIDTH as u32;
            brick.size.1 = STUD_WIDTH as u32;
            brick.size.2 = (STUD_HEIGHT/2.0) as u32;
        },
        "B_2x_Cube_Side" => {
            brick.size.0 = STUD_WIDTH as u32;
            brick.size.1 = STUD_WIDTH as u32;
            brick.size.2 = STUD_HEIGHT as u32;
        },
        "B_1x1_Brick_Side" => {
            brick.size.0 = (STUD_WIDTH/2.0) as u32;
            brick.size.1 = (STUD_WIDTH/2.0) as u32;
            brick.size.2 = (STUD_HEIGHT/2.0) as u32;
        },
        "B_1x4_Brick_Side" => {
            brick.size.0 = (STUD_WIDTH*2.0) as u32;
            brick.size.1 = (STUD_WIDTH/2.0) as u32;
            brick.size.2 = (STUD_HEIGHT/2.0) as u32;
        },
        "B_1x2f_Plate_Center" => {
            brick.size.0 = STUD_WIDTH as u32;
            brick.size.1 = (STUD_WIDTH/2.0) as u32;
            brick.size.2 = (STUD_HEIGHT/2.0) as u32;
        },
        "B_2x2f_Plate_Center" => {
            brick.size.0 = STUD_WIDTH as u32;
            brick.size.1 = STUD_WIDTH as u32;
            brick.size.2 = (PLATE_HEIGHT/2.0) as u32;
        },
        "B_1x2f_Plate_Center_Inv" => {
            brick.size.0 = STUD_WIDTH as u32;
            brick.size.1 = (STUD_WIDTH/2.0) as u32;
            brick.size.2 = (STUD_HEIGHT/2.0) as u32;
        },
        "B_2x2f_Plate_Center_Inv" => {
            brick.size.0 = STUD_WIDTH as u32;
            brick.size.1 = STUD_WIDTH as u32;
            brick.size.2 = (PLATE_HEIGHT/2.0) as u32;
        },
        "B_1x1F_Round" => {
            brick.size.0 = (STUD_WIDTH/2.0) as u32;
            brick.size.1 = (STUD_WIDTH/2.0) as u32;
            brick.size.2 = (PLATE_HEIGHT/2.0) as u32;
        },
        "B_1x1_Round" => {
            brick.size.0 = (STUD_WIDTH/2.0) as u32;
            brick.size.1 = (STUD_WIDTH/2.0) as u32;
            brick.size.2 = (STUD_HEIGHT/2.0) as u32;
        },
        "B_2x2F_Round" => {
            brick.size.0 = STUD_WIDTH as u32;
            brick.size.1 = STUD_WIDTH as u32;
            brick.size.2 = (PLATE_HEIGHT/2.0) as u32;
        },
        "B_2x2_Round" => {
            brick.size.0 = STUD_WIDTH as u32;
            brick.size.1 = STUD_WIDTH as u32;
            brick.size.2 = (STUD_HEIGHT/2.0) as u32;
        },
        "B_4x4_Round" => {
            brick.size.0 = (STUD_WIDTH*2.0) as u32;
            brick.size.1 = (STUD_WIDTH*2.0) as u32;
            brick.size.2 = (STUD_HEIGHT/2.0) as u32;
        },
        _ => ()
    }

    // Apply Rotation
    if brick.rotation == Rotation::Deg90 || brick.rotation == Rotation::Deg270 {
        std::mem::swap(&mut brick.size.0, &mut brick.size.1);
    }

    // Apply Direction
    if brick.direction == Direction::XPositive || brick.direction == Direction::XNegative {
        std::mem::swap(&mut brick.size.0, &mut brick.size.2);
    }
    else if brick.direction == Direction::YPositive || brick.direction == Direction::YNegative {
        std::mem::swap(&mut brick.size.0, &mut brick.size.1);
        std::mem::swap(&mut brick.size.1, &mut brick.size.2);
    }

    brick
}

pub fn calculate_centroid(bricks: &[Brick]) -> (i32, i32) {
    // Sums for calculating Centroid of save
    let mut area_sum: i32 = 0;
    let mut point_sum = (0, 0);

    for brick in bricks {
        // Add to Centroid calculation sums
        let area = brick.size.0 * brick.size.1;
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
        let brick_bounds = (
            brick.position.0 - brick.size.0 as i32 - x,
            brick.position.1 - brick.size.1 as i32 - y,
            brick.position.0 + brick.size.0 as i32 - x,
            brick.position.1 + brick.size.1 as i32 - y,
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

/*
pub fn find_furthest_brick((x, y): (i32, i32), bricks: &[Brick]) -> Brick {
    let mut furthest_distance: u32 = 0;
    let mut furthest_brick: Brick = bricks[0].clone();
    for brick in bricks {
        let x_dist: u32 = (brick.position.0 - x).abs() as u32;
        let y_dist: u32 = (brick.position.1 - y).abs() as u32;

        if x_dist > furthest_distance {
            furthest_distance = x_dist;
            furthest_brick = brick.clone();
        }
        if y_dist > furthest_distance {
            furthest_distance = y_dist;
            furthest_brick = brick.clone();
        }
    }
    furthest_brick
}

#[derive(Debug)]
pub struct Player {
    pub name: String,
    pub brick_count: u32
}

pub fn brick_count_by_player(bricks: &[Brick], owners: &[User]) -> Vec<Player> {
    let mut players: Vec<Player> = Vec::new();
    for user in owners {
        players.push(Player {
            name: user.name.clone(),
            brick_count: 0
        });
    }
    for brick in bricks {
        let owner_index = match brick.owner_index {
            None => 0usize,
            Some(x) => x as usize,
        };
        players[owner_index].brick_count += 1;
    }
    players
}
*/
