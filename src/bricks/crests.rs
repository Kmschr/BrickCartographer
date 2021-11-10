use brickadia::save::{Brick, Rotation, Direction};
use bricks::primitives::*;

pub fn ramp_crest(brick: &Brick, shape: &Shape) -> Vec<f32> {
    match brick.direction {
        Direction::YPositive => {
            match brick.rotation {
                Rotation::Deg90 | Rotation::Deg270 => crest_down(shape),
                _ => rec(shape)
            }
        },
        Direction::YNegative => {
            match brick.rotation {
                Rotation::Deg90 | Rotation::Deg270 => crest_up(shape),
                _ => rec(shape)
            }
        },
        Direction::XPositive => {
            match brick.rotation {
                Rotation::Deg90 | Rotation::Deg270 => crest_right(shape),
                _ => rec(shape)
            }
        },
        Direction::XNegative => {
            match brick.rotation {
                Rotation::Deg90 | Rotation::Deg270 => crest_left(shape),
                _ => rec(shape)
            }
        },
        _ => rec(shape)
    }
}

pub fn ramp_crest_end(brick: &Brick, shape: &Shape) -> Vec<f32> {
    match brick.direction {
        Direction::YPositive => {
            match brick.rotation {
                Rotation::Deg0 => tri(shape, Tri::TopRight),
                Rotation::Deg180 => tri(shape, Tri::TopLeft),
                Rotation::Deg90 | Rotation::Deg270 => crest_down(shape),
            }
        },
        Direction::YNegative => {
            match brick.rotation {
                Rotation::Deg0 => tri(shape, Tri::BotLeft),
                Rotation::Deg180 => tri(shape, Tri::BotRight),
                Rotation::Deg90 | Rotation::Deg270 => crest_up(shape),
            }
        },
        Direction::XPositive => {
            match brick.rotation {
                Rotation::Deg0 => tri(shape, Tri::TopLeft),
                Rotation::Deg180 => tri(shape, Tri::BotLeft),
                Rotation::Deg90 | Rotation::Deg270 => crest_right(shape),
            }
        },
        Direction::XNegative => {
            match brick.rotation {
                Rotation::Deg0 => tri(shape, Tri::BotRight),
                Rotation::Deg180 => tri(shape, Tri::TopRight),
                Rotation::Deg90 | Rotation::Deg270 => crest_left(shape),
            }
        },
        _ => rec(shape)
    }
}

fn crest_down(shape: &Shape) -> Vec<f32> {
    let (sx, _sy) = shape.size();
    [tri(&Shape {x1: shape.x1, y1: shape.y1, x2: shape.x2 - sx, y2: shape.y2}, Tri::TopRight),
     tri(&Shape {x1: shape.x1 + sx, y1: shape.y1, x2: shape.x2, y2: shape.y2}, Tri::TopLeft)].concat()
}

fn crest_right(shape: &Shape) -> Vec<f32> {
    let (_sx, sy) = shape.size();
    [tri(&Shape {x1: shape.x1, y1: shape.y1, x2: shape.x2, y2: shape.y2 - sy}, Tri::BotLeft),
     tri(&Shape {x1: shape.x1, y1: shape.y1 + sy, x2: shape.x2, y2: shape.y2}, Tri::TopLeft)].concat()
}

fn crest_up(shape: &Shape) -> Vec<f32> {
    let (sx, _sy) = shape.size();
    [tri(&Shape {x1: shape.x1, y1: shape.y1, x2: shape.x2 - sx, y2: shape.y2}, Tri::BotRight),
     tri(&Shape {x1: shape.x1 + sx, y1: shape.y1, x2: shape.x2, y2: shape.y2}, Tri::BotLeft)].concat()
}

fn crest_left(shape: &Shape) -> Vec<f32> {
    let (_sx, sy) = shape.size();
    [tri(&Shape {x1: shape.x1, y1: shape.y1, x2: shape.x2, y2: shape.y2 - sy}, Tri::BotRight),
     tri(&Shape {x1: shape.x1, y1: shape.y1 + sy, x2: shape.x2, y2: shape.y2}, Tri::TopRight)].concat()
}

pub fn ramp_crest_ol(brick: &Brick, shape: &Shape) -> Vec<f32> {
    match brick.direction {
        Direction::YPositive => {
            match brick.rotation {
                Rotation::Deg90 | Rotation::Deg270 => crest_ol_down(shape),
                _ => rec_ol(shape)
            }
        },
        Direction::YNegative => {
            match brick.rotation {
                Rotation::Deg90 | Rotation::Deg270 => crest_ol_up(shape),
                _ => rec_ol(shape)
            }
        },
        Direction::XPositive => {
            match brick.rotation {
                Rotation::Deg90 | Rotation::Deg270 => crest_ol_right(shape),
                _ => rec_ol(shape)
            }
        },
        Direction::XNegative => {
            match brick.rotation {
                Rotation::Deg90 | Rotation::Deg270 => crest_ol_left(shape),
                _ => rec_ol(shape)
            }
        },
        _ => rec_ol(shape)
    }
}

pub fn ramp_crest_end_ol(brick: &Brick, shape: &Shape) -> Vec<f32> {
    match brick.direction {
        Direction::YPositive => {
            match brick.rotation {
                Rotation::Deg0 => tri_ol(shape, Tri::TopRight),
                Rotation::Deg180 => tri_ol(shape, Tri::TopLeft),
                Rotation::Deg90 | Rotation::Deg270 => crest_ol_down(shape),
            }
        },
        Direction::YNegative => {
            match brick.rotation {
                Rotation::Deg0 => tri_ol(shape, Tri::BotLeft),
                Rotation::Deg180 => tri_ol(shape, Tri::BotRight),
                Rotation::Deg90 | Rotation::Deg270 => crest_ol_up(shape),
            }
        },
        Direction::XPositive => {
            match brick.rotation {
                Rotation::Deg0 => tri_ol(shape, Tri::TopLeft),
                Rotation::Deg180 => tri_ol(shape, Tri::BotLeft),
                Rotation::Deg90 | Rotation::Deg270 => crest_ol_right(shape),
            }
        },
        Direction::XNegative => {
            match brick.rotation {
                Rotation::Deg0 => tri_ol(shape, Tri::BotRight),
                Rotation::Deg180 => tri_ol(shape, Tri::TopRight),
                Rotation::Deg90 | Rotation::Deg270 => crest_ol_left(shape),
            }
        },
        _ => rec_ol(shape)
    }
}

fn crest_ol_down(shape: &Shape) -> Vec<f32> {
    let (sx, _sy) = shape.size();
    let left = &Shape {x1: shape.x1, y1: shape.y1, x2: shape.x2 - sx, y2: shape.y2};
    let right = &Shape {x1: shape.x1 + sx, y1: shape.y1, x2: shape.x2, y2: shape.y2};
    [tri_ol_tr_top(left), tri_ol_tr_diag(left),
     tri_ol_tl_top(right), tri_ol_tl_diag(right)].concat()
}

fn crest_ol_right(shape: &Shape) -> Vec<f32> {
    let (_sx, sy) = shape.size();
    let top = &Shape {x1: shape.x1, y1: shape.y1, x2: shape.x2, y2: shape.y2 - sy};
    let bot = &Shape {x1: shape.x1, y1: shape.y1 + sy, x2: shape.x2, y2: shape.y2};
    [tri_ol_bl_left(top), tri_ol_bl_diag(top),
     tri_ol_tl_left(bot), tri_ol_tl_diag(bot)].concat()
}

fn crest_ol_up(shape: &Shape) -> Vec<f32> {
    let (sx, _sy) = shape.size();
    let left = &Shape {x1: shape.x1, y1: shape.y1, x2: shape.x2 - sx, y2: shape.y2};
    let right = &Shape {x1: shape.x1 + sx, y1: shape.y1, x2: shape.x2, y2: shape.y2};
    [tri_ol_br_bot(left), tri_ol_br_diag(left),
     tri_ol_bl_bot(right), tri_ol_bl_diag(right)].concat()
}

fn crest_ol_left(shape: &Shape) -> Vec<f32> {
    let (_sx, sy) = shape.size();
    let top = &Shape {x1: shape.x1, y1: shape.y1, x2: shape.x2, y2: shape.y2 - sy};
    let bot = &Shape {x1: shape.x1, y1: shape.y1 + sy, x2: shape.x2, y2: shape.y2};
    [tri_ol_br_right(top), tri_ol_br_diag(top),
     tri_ol_tr_right(bot), tri_ol_tr_diag(bot)].concat()
}
