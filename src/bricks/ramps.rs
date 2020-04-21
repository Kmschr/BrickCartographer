use brs::{Rotation, Direction};
use bricks::primitives::*;

pub fn ramp(direction: Direction, rotation: Rotation, shape: &Shape) -> Vec<f32> {
    let (x1, y1, x2, y2) = shape.unpack();
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
                    let rec = rec(&Shape {x1, y1, x2: mx1, y2});
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

pub fn ramp_ol(direction: Direction, rotation: Rotation, shape: &Shape) -> Vec<f32> {
    match direction {
        Direction::XPositive =>
            match rotation {
                Rotation::Deg90 => ramp_ol_bl_bot(shape),
                Rotation::Deg270 => ramp_ol_tl_top(shape),
                _ => rec_ol(shape)
            },
        Direction::XNegative =>
            match rotation {
                Rotation::Deg90 => ramp_ol_tr_top(shape),
                Rotation::Deg270 => ramp_ol_br_bot(shape),
                _ => rec_ol(shape)
            },
        Direction::YPositive =>
            match rotation {
                Rotation::Deg90 => ramp_ol_tl_left(shape),
                Rotation::Deg270 => ramp_ol_tr_right(shape),
                _ => rec_ol(shape)
            },
        Direction::YNegative => 
            match rotation {
                Rotation::Deg90 => ramp_ol_br_right(shape),
                Rotation::Deg270 => ramp_ol_bl_left(shape),
                _ => rec_ol(shape)
            },
        _ => rec_ol(shape),
    }
}

fn ramp_ol_br_bot(shape: &Shape) -> Vec::<f32> {
    let (x1, y1, x2, y2) = shape.unpack();
    let rec = rec_ol_no_top(&Shape {x1, y1: y2 - STUD_WIDTH, x2, y2});
    let tri_shape = &Shape {x1, y1, x2, y2: y2 - STUD_WIDTH};
    let (dx, dy) = tri_shape.angular_offsets();
    let tri = [tri_ol_br_right(tri_shape), tri_ol_br_diag(tri_shape),
          vec![tri_shape.x1, tri_shape.y2, tri_shape.x1 + dx, tri_shape.y2, tri_shape.x1, tri_shape.y2 + dy]].concat();
    [rec, tri].concat()
}

fn ramp_ol_bl_left(shape: &Shape) -> Vec::<f32> {
    let (x1, y1, x2, y2) = shape.unpack();
    let rec = rec_ol_no_right(&Shape {x1, y1, x2: x1 + STUD_WIDTH, y2});
    let tri_shape = &Shape {x1: x1 + STUD_WIDTH, y1, x2, y2};
    let (dx, dy) = tri_shape.angular_offsets();
    let tri = [tri_ol_bl_bot(tri_shape), tri_ol_bl_diag(tri_shape),
               vec![tri_shape.x1 - dx, tri_shape.y1, tri_shape.x1, tri_shape.y1 + dy, tri_shape.x1, tri_shape.y1]].concat();
    [rec, tri].concat()
}

fn ramp_ol_tl_top(shape: &Shape) -> Vec::<f32> {
    let (x1, y1, x2, y2) = shape.unpack();
    let rec = rec_ol_no_bot(&Shape {x1, y1, x2, y2: y1 + STUD_WIDTH});
    let tri_shape = &Shape {x1, y1: y1 + STUD_WIDTH, x2, y2};
    let (dx, dy) = tri_shape.angular_offsets();
    let tri = [tri_ol_tl_left(tri_shape), tri_ol_tl_diag(tri_shape),
          vec![tri_shape.x2 - dx, tri_shape.y1,  tri_shape.x2, tri_shape.y1 - dy,  tri_shape.x2, tri_shape.y1]].concat();
    [rec, tri].concat()
}

fn ramp_ol_tr_right(shape: &Shape) -> Vec::<f32> {
    let (x1, y1, x2, y2) = shape.unpack();
    let rec = rec_ol_no_left(&Shape {x1: x2 - STUD_WIDTH, y1, x2, y2});
    let tri_shape = &Shape {x1, y1, x2: x2 - STUD_WIDTH, y2};
    let (dx, dy) = tri_shape.angular_offsets();
    let tri = [tri_ol_tr_top(tri_shape), tri_ol_tr_diag(tri_shape),
          vec![tri_shape.x2, tri_shape.y2 - dy,  tri_shape.x2, tri_shape.y2,  tri_shape.x2 + dx, tri_shape.y2]].concat();
    [rec, tri].concat()
}

fn ramp_ol_tl_left(shape: &Shape) -> Vec::<f32> {
    let (x1, y1, x2, y2) = shape.unpack();
    let rec = rec_ol_no_right(&Shape {x1, y1, x2: x1 + STUD_WIDTH, y2});
    let tri_shape = &Shape {x1: x1 + STUD_WIDTH, y1, x2, y2};
    let (dx, dy) = tri_shape.angular_offsets();
    let tri = [tri_ol_tl_top(tri_shape), tri_ol_tl_diag(tri_shape),
            vec![tri_shape.x1 - dx, tri_shape.y2, tri_shape.x1, tri_shape.y2, tri_shape.x1, tri_shape.y2 - dy]].concat();
    [rec, tri].concat()
}

fn ramp_ol_tr_top(shape: &Shape) -> Vec::<f32> {
    let (x1, y1, x2, y2) = shape.unpack();
    let rec = rec_ol_no_bot(&Shape {x1, y1, x2, y2: y1 + STUD_WIDTH});
    let tri_shape = &Shape {x1, y1: y1 + STUD_WIDTH, x2, y2};
    let (dx, dy) = tri_shape.angular_offsets();
    let tri = [tri_ol_tr_right(tri_shape), tri_ol_tr_diag(tri_shape),
            vec![tri_shape.x1, tri_shape.y1 - dy, tri_shape.x1, tri_shape.y1, tri_shape.x1 + dx, tri_shape.y1]].concat();
    [rec, tri].concat()
}

fn ramp_ol_br_right(shape: &Shape) -> Vec::<f32> {
    let (x1, y1, x2, y2) = shape.unpack();
    let rec = rec_ol_no_left(&Shape {x1: x2 - STUD_WIDTH, y1, x2, y2});
    let tri_shape = &Shape {x1, y1, x2: x2 - STUD_WIDTH, y2};
    let (dx, dy) = tri_shape.angular_offsets();
    let tri = [tri_ol_br_bot(tri_shape), tri_ol_br_diag(tri_shape), 
            vec![tri_shape.x2, tri_shape.y1, tri_shape.x2, tri_shape.y1 + dy, tri_shape.x2 + dx, tri_shape.y1]].concat();
    [rec, tri].concat()
}

fn ramp_ol_bl_bot(shape: &Shape) -> Vec::<f32> {
    let (x1, y1, x2, y2) = shape.unpack();
    let rec = rec_ol_no_top(&Shape {x1, y1: y2 - STUD_WIDTH, x2, y2});
    let tri_shape = &Shape {x1, y1, x2, y2: y2 - STUD_WIDTH};
    let (dx, dy) = tri_shape.angular_offsets();
    let tri = [tri_ol_bl_left(tri_shape), tri_ol_bl_diag(tri_shape),
            vec![tri_shape.x2, tri_shape.y2, tri_shape.x2 - dx, tri_shape.y2, tri_shape.x2, tri_shape.y2 + dy]].concat();
    [rec, tri].concat()
}
