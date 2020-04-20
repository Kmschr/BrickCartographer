use brs::{Rotation, Direction};
use bricks::primitives::*;

pub fn get_side_wedge(direction: Direction, rotation: Rotation, shape: &Shape) -> Vec<f32> {
    match direction {
        Direction::ZPositive => 
            match rotation {
                Rotation::Deg0   => tri(shape, Tri::TopLeft),
                Rotation::Deg90  => tri(shape, Tri::TopRight),
                Rotation::Deg180 => tri(shape, Tri::BotRight),
                Rotation::Deg270 => tri(shape, Tri::BotLeft)
            },
        Direction::ZNegative =>
            match rotation {
                Rotation::Deg0   => tri(shape, Tri::TopRight),
                Rotation::Deg90  => tri(shape, Tri::TopLeft),
                Rotation::Deg180 => tri(shape, Tri::BotLeft),
                Rotation::Deg270 => tri(shape, Tri::BotRight)
            },
        _ => 
            rec(shape),
    }
}

pub fn get_side_wedge_ol(direction: Direction, rotation: Rotation, shape: &Shape) -> Vec<f32> {
    match direction {
        Direction::ZPositive => 
            match rotation {
                Rotation::Deg0   => tri_ol(shape, Tri::TopLeft),
                Rotation::Deg90  => tri_ol(shape, Tri::TopRight),
                Rotation::Deg180 => tri_ol(shape, Tri::BotRight),
                Rotation::Deg270 => tri_ol(shape, Tri::BotLeft)
            },
        Direction::ZNegative =>
            match rotation {
                Rotation::Deg0   => tri_ol(shape, Tri::TopRight),
                Rotation::Deg90  => tri_ol(shape, Tri::TopLeft),
                Rotation::Deg180 => tri_ol(shape, Tri::BotLeft),
                Rotation::Deg270 => tri_ol(shape, Tri::BotRight)
            },
        _ => 
            rec_ol(shape),
    }
}

pub fn get_wedge(direction: Direction, rotation: Rotation, shape: &Shape) -> Vec<f32> {
    match rotation {
        Rotation::Deg90 =>
            match direction {
                Direction::XPositive => tri(shape, Tri::BotLeft),
                Direction::XNegative => tri(shape, Tri::TopRight),
                Direction::YPositive => tri(shape, Tri::TopLeft),
                Direction::YNegative => tri(shape, Tri::BotRight),
                Direction::ZPositive | Direction::ZNegative =>
                    rec(shape),
            },
        Rotation::Deg270 =>
            match direction {
                Direction::XPositive => tri(shape, Tri::TopLeft),
                Direction::XNegative => tri(shape, Tri::BotRight),
                Direction::YPositive => tri(shape, Tri::TopRight),
                Direction::YNegative => tri(shape, Tri::BotLeft),
                Direction::ZPositive | Direction::ZNegative =>
                    rec(shape),
            },
        _ =>
            rec(shape),
    }
}

pub fn get_wedge_ol(direction: Direction, rotation: Rotation, shape: &Shape) -> Vec<f32> {
    match rotation {
        Rotation::Deg90 =>
            match direction {
                Direction::XPositive => tri_ol(shape, Tri::BotLeft),
                Direction::XNegative => tri_ol(shape, Tri::TopRight),
                Direction::YPositive => tri_ol(shape, Tri::TopLeft),
                Direction::YNegative => tri_ol(shape, Tri::BotRight),
                Direction::ZPositive | Direction::ZNegative =>
                    rec_ol(shape),
            },
        Rotation::Deg270 =>
            match direction {
                Direction::XPositive => tri_ol(shape, Tri::TopLeft),
                Direction::XNegative => tri_ol(shape, Tri::BotRight),
                Direction::YPositive => tri_ol(shape, Tri::TopRight),
                Direction::YNegative => tri_ol(shape, Tri::BotLeft),
                Direction::ZPositive | Direction::ZNegative =>
                    rec_ol(shape),
            },
        _ =>
            rec_ol(shape),
    }
}