use brickadia::save::Direction;
use crate::brick::Brick;
use super::primitives::*;

pub fn round(brick: &Brick, shape: &Shape) -> Vec<f32> {
    match brick.direction {
        Direction::ZPositive | Direction::ZNegative => circle(shape),
        _ => rec(shape)
    }
}

pub fn round_ol(brick: &Brick, shape: &Shape) -> Vec<f32> {
    match brick.direction {
        Direction::ZPositive | Direction::ZNegative => circle_ol(shape),
        _ => rec_ol(shape)
    }
}
