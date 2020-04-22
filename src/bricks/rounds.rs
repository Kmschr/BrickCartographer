use brs::{Direction};
use bricks::primitives::*;

pub fn round(direction: Direction, shape: &Shape) -> Vec<f32> {
    match direction {
        Direction::ZPositive | Direction::ZNegative => circle(shape),
        _ => rec(shape)
    }
}

pub fn round_ol(direction: Direction, shape: &Shape) -> Vec<f32> {
    match direction {
        Direction::ZPositive | Direction::ZNegative => circle_ol(shape),
        _ => rec_ol(shape)
    }
}
