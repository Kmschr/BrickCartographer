use brs::{Rotation, Direction};
use bricks::primitives::*;

pub fn corner(direction: Direction, rotation: Rotation, shape: &Shape) -> Vec<f32> {
    match direction {
        Direction::ZPositive =>
            match rotation {
                Rotation::Deg0 => corner_tl(shape),
                Rotation::Deg90 => corner_tr(shape),
                Rotation::Deg180 => corner_br(shape),
                Rotation::Deg270 => corner_bl(shape)
            }
        Direction::ZNegative =>
            match rotation {
                Rotation::Deg0 => corner_tr(shape),
                Rotation::Deg90 => corner_tl(shape),
                Rotation::Deg180 => corner_bl(shape),
                Rotation::Deg270 => corner_br(shape)
            }
        _ => rec(shape)
    }
}

fn corner_tl(shape: &Shape) -> Vec::<f32> {
    let long = rec_top(shape);
    let short = rec_bl(shape);
    [long, short].concat()
}

fn corner_tr(shape: &Shape) -> Vec::<f32> {
    let long = rec_top(shape);
    let short = rec_br(shape);
    [long, short].concat()
}

fn corner_br(shape: &Shape) -> Vec::<f32> {
    let long = rec_bot(shape);
    let short = rec_tr(shape);
    [long, short].concat()
}

fn corner_bl(shape: &Shape) -> Vec::<f32> {
    let long = rec_bot(shape);
    let short = rec_tl(shape);
    [long, short].concat()
}

pub fn corner_ol(direction: Direction, rotation: Rotation, shape: &Shape) -> Vec<f32> {
    match direction {
        Direction::ZPositive =>
            match rotation {
                Rotation::Deg0 => corner_ol_tl(shape),
                Rotation::Deg90 => corner_ol_tr(shape),
                Rotation::Deg180 => corner_ol_br(shape),
                Rotation::Deg270 => corner_ol_bl(shape),
            }
        Direction::ZNegative =>
            match rotation {
                Rotation::Deg0 => corner_ol_tr(shape),
                Rotation::Deg90 => corner_ol_tl(shape),
                Rotation::Deg180 => corner_ol_bl(shape),
                Rotation::Deg270 => corner_ol_br(shape),
            }
        _ => rec(shape)
    }
}

fn corner_ol_tl(shape: &Shape) -> Vec::<f32> {
    let (sx, sy) = shape.size();
    [rec_ol_top(shape), rec_ol_left(shape),
    rec_ol_bot(&Shape {x1: shape.x1, y1: shape.y1, x2: shape.x2 - sx, y2: shape.y2}),
    rec_ol_right(&Shape {x1: shape.x1, y1: shape.y1 + sy, x2: shape.x2 - sx, y2: shape.y2}),
    rec_ol_bot(&Shape {x1: shape.x1 + sx, y1: shape.y1, x2: shape.x2, y2: shape.y2 - sy}),
    rec_ol_right(&Shape {x1: shape.x1, y1: shape.y1, x2: shape.x2, y2: shape.y2 - sy}),
    rec(&Shape {x1: shape.x1 + sx - OUTLINE_THICKNESS, y1: shape.y1 + sy - OUTLINE_THICKNESS, x2: shape.x1 + sx, y2: shape.y1 + sy})].concat()
}

fn corner_ol_tr(shape: &Shape) -> Vec::<f32> {
    let (sx, sy) = shape.size();
    [rec_ol_top(shape), rec_ol_right(shape),
    rec_ol_left(&Shape {x1: shape.x1, y1: shape.y1, x2: shape.x2, y2: shape.y2 - sy}),
    rec_ol_bot(&Shape {x1: shape.x1, y1: shape.y1, x2: shape.x2 - sx, y2: shape.y2 - sy}),
    rec_ol_left(&Shape {x1: shape.x1 + sx, y1: shape.y1 + sy, x2: shape.x2, y2: shape.y2}),
    rec_ol_bot(&Shape {x1: shape.x1 + sx, y1: shape.y1, x2: shape.x2, y2: shape.y2}),
    rec(&Shape {x1: shape.x1 + sx, y1: shape.y1 + sy - OUTLINE_THICKNESS, x2: shape.x1 + sx + OUTLINE_THICKNESS, y2: shape.y1 + sy})].concat()
}

fn corner_ol_br(shape: &Shape) -> Vec::<f32> {
    let (sx, sy) = shape.size();
    [rec_ol_bot(shape), rec_ol_right(shape),
    rec_ol_top(&Shape {x1: shape.x1 + sx, y1: shape.y1, x2: shape.x2, y2: shape.y2}),
    rec_ol_left(&Shape {x1: shape.x1 + sx, y1: shape.y1, x2: shape.x2, y2: shape.y2 - sy}),
    rec_ol_top(&Shape {x1: shape.x1, y1: shape.y1 + sy, x2: shape.x2 - sx, y2: shape.y2}),
    rec_ol_left(&Shape {x1: shape.x1, y1: shape.y1 + sy, x2: shape.x2, y2: shape.y2}),
    rec(&Shape {x1: shape.x1 + sx, y1: shape.y1 + sy, x2: shape.x1 + sx + OUTLINE_THICKNESS, y2: shape.y1 + sy + OUTLINE_THICKNESS})].concat()
}

fn corner_ol_bl(shape: &Shape) -> Vec::<f32> {
    let (sx, sy) = shape.size();
    [rec_ol_bot(shape), rec_ol_left(shape),
    rec_ol_top(&Shape {x1: shape.x1, y1: shape.y1, x2: shape.x2 - sx, y2: shape.y2}),
    rec_ol_right(&Shape {x1: shape.x1, y1: shape.y1, x2: shape.x2 - sx, y2: shape.y2 - sy}),
    rec_ol_top(&Shape {x1: shape.x1 + sx, y1: shape.y1 + sy, x2: shape.x2, y2: shape.y2}),
    rec_ol_right(&Shape {x1: shape.x1, y1: shape.y1 + sy, x2: shape.x2, y2: shape.y2}),
    rec(&Shape {x1: shape.x1 + sx - OUTLINE_THICKNESS, y1: shape.y1 + sy, x2: shape.x1 + sx, y2: shape.y1 + sy + OUTLINE_THICKNESS})].concat()
}
