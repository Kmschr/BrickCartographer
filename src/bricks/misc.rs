use brs::{Rotation, Direction};
use bricks::primitives::*;

pub fn get_corner(direction: Direction, rotation: Rotation, shape: &Shape) -> Vec<f32> {
    match direction {
        Direction::ZPositive =>
            match rotation {
                Rotation::Deg0 => {
                    let long = rec_top(shape);
                    let short = rec_bl(shape);
                    [long, short].concat()
                },
                Rotation::Deg90 => {
                    let long = rec_top(shape);
                    let short = rec_br(shape);
                    [long, short].concat()
                },
                Rotation::Deg180 => {
                    let long = rec_bot(shape);
                    let short = rec_tr(shape);
                    [long, short].concat()
                },
                Rotation::Deg270 => {
                    let long = rec_bot(shape);
                    let short = rec_tl(shape);
                    [long, short].concat()
                }
            }
        Direction::ZNegative =>
            match rotation {
                Rotation::Deg0 => {
                    let long = rec_top(shape);
                    let short = rec_br(shape);
                    [long, short].concat()
                },
                Rotation::Deg90 => {
                    let long = rec_top(shape);
                    let short = rec_bl(shape);
                    [long, short].concat()
                },
                Rotation::Deg180 => {
                    let long = rec_bot(shape);
                    let short = rec_tl(shape);
                    [long, short].concat()
                },
                Rotation::Deg270 => {
                    let long = rec_bot(shape);
                    let short = rec_tr(shape);
                    [long, short].concat()
                }
            }
        _ =>
            rec(shape)
    }
}