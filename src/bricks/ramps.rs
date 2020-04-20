use brs::{Rotation, Direction};
use bricks::primitives::*;

pub fn get_ramp(direction: Direction, rotation: Rotation, shape: &Shape) -> Vec<f32> {
    let x1 = shape.x1;
    let y1 = shape.y1;
    let x2 = shape.x2;
    let y2 = shape.y2;
    let my1 = y1 + STUD_WIDTH;
    let my2 = y2 - STUD_WIDTH;
    let mx1 = x1 + STUD_WIDTH;
    let mx2 = x2 - STUD_WIDTH;
    match direction {
        Direction::XPositive =>
            match rotation {
                Rotation::Deg90 => {
                    let rec = rec(&Shape {x1, y1: my2, x2, y2});
                    let tri = tri(&Shape {x1, y1, x2, y2: my2}, Tri::BotLeft);
                    [rec, tri].concat()
                },
                Rotation::Deg270 => {
                    let rec = rec(&Shape {x1, y1, x2, y2: my1});
                    let tri = tri(&Shape {x1, y1: my1, x2, y2}, Tri::TopLeft);
                    [rec, tri].concat()
                },
                _ => 
                    rec(shape)
            },
        Direction::XNegative =>
            match rotation {
                Rotation::Deg90 => {
                    let rec = rec(&Shape {x1, y1, x2, y2: my1});
                    let tri = tri(&Shape {x1, y1: my1, x2, y2}, Tri::TopRight);
                    [rec, tri].concat()
                },
                Rotation::Deg270 => {
                    let rec = rec(&Shape {x1, y1: my2, x2, y2});
                    let tri = tri(&Shape {x1, y1, x2, y2: my2}, Tri::BotRight);
                    [rec, tri].concat()
                },
                _ =>
                    rec(shape)
            },
        Direction::YPositive =>
            match rotation {
                Rotation::Deg90 => {
                    let rec = rec(&Shape {x1, y1, x2: mx1, y2});
                    let tri = tri(&Shape {x1: mx1, y1, x2, y2}, Tri::TopLeft);
                    [rec, tri].concat()
                },
                Rotation::Deg270 => {
                    let rec = rec(&Shape {x1: mx2, y1, x2, y2});
                    let tri = tri(&Shape {x1, y1, x2: mx2, y2}, Tri::TopRight);
                    [rec, tri].concat()
                },
                _ =>
                    rec(shape)
            },
        Direction::YNegative => 
            match rotation {
                Rotation::Deg90 => {
                    let rec = rec(&Shape {x1: mx2, y1, x2, y2});
                    let tri = tri(&Shape {x1, y1, x2: mx2, y2}, Tri::BotRight);
                    [rec, tri].concat()
                },
                Rotation::Deg270 => {
                    let rec = rec(&Shape {x1, y1, x2: mx1, y2: my1});
                    let tri = tri(&Shape {x1: mx1, y1, x2, y2}, Tri::BotLeft);
                    [rec, tri].concat()
                },
                _ =>
                    rec(shape)
            },
        _ =>
            rec(shape),
    }
}

pub fn get_ramp_ol(direction: Direction, rotation: Rotation, shape: &Shape) -> Vec<f32> {
    let x1 = shape.x1;
    let y1 = shape.y1;
    let x2 = shape.x2;
    let y2 = shape.y2;
    let my1 = y1 + STUD_WIDTH;
    let my2 = y2 - STUD_WIDTH;
    let mx1 = x1 + STUD_WIDTH;
    let mx2 = x2 - STUD_WIDTH;
    match direction {
        Direction::XPositive =>
            match rotation {
                Rotation::Deg90 => {
                    let rec = rec_ol(&Shape {x1, y1: my2, x2, y2});
                    let tri = tri_ol(&Shape {x1, y1, x2, y2: my2}, Tri::BotLeft);
                    [rec, tri].concat()
                },
                Rotation::Deg270 => {
                    let rec = rec_ol(&Shape {x1, y1, x2, y2: my1});
                    let tri = tri_ol(&Shape {x1, y1: my1, x2, y2}, Tri::TopLeft);
                    [rec, tri].concat()
                },
                _ => 
                    rec_ol(shape)
            },
        Direction::XNegative =>
            match rotation {
                Rotation::Deg90 => {
                    let rec = rec_ol(&Shape {x1, y1, x2, y2: my1});
                    let tri = tri_ol(&Shape {x1, y1: my1, x2, y2}, Tri::TopRight);
                    [rec, tri].concat()
                },
                Rotation::Deg270 => {
                    let rec = rec_ol_no_top(&Shape {x1, y1: my2, x2, y2});
                    let tri = tri_ol(&Shape {x1, y1, x2, y2: my2}, Tri::BotRight);
                    [rec, tri].concat()
                },
                _ =>
                    rec_ol(shape)
            },
        Direction::YPositive =>
            match rotation {
                Rotation::Deg90 => {
                    let rec = rec_ol(&Shape {x1, y1, x2: mx1, y2});
                    let tri = tri_ol(&Shape {x1: mx1, y1, x2, y2}, Tri::TopLeft);
                    [rec, tri].concat()
                },
                Rotation::Deg270 => {
                    let rec = rec_ol(&Shape {x1: mx2, y1, x2, y2});
                    let tri = tri_ol(&Shape {x1, y1, x2: mx2, y2}, Tri::TopRight);
                    [rec, tri].concat()
                },
                _ =>
                    rec_ol(shape)
            },
        Direction::YNegative => 
            match rotation {
                Rotation::Deg90 => {
                    let rec = rec_ol(&Shape {x1: mx2, y1, x2, y2});
                    let tri = tri_ol(&Shape {x1, y1, x2: mx2, y2}, Tri::BotRight);
                    [rec, tri].concat()
                },
                Rotation::Deg270 => {
                    let rec = rec_ol(&Shape {x1, y1, x2: mx1, y2: my1});
                    let tri = tri_ol(&Shape {x1: mx1, y1, x2, y2}, Tri::BotLeft);
                    [rec, tri].concat()
                },
                _ =>
                    rec_ol(shape)
            },
        _ =>
            rec_ol(shape),
    }
}