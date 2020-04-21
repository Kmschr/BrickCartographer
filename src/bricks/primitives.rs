pub const STUD_WIDTH: f32 = 10.0;
pub const STUD_HEIGHT: f32 = 12.0;
pub const PLATE_HEIGHT: f32 = 4.0;

pub const OUTLINE_THICKNESS: f32 = 1.5;

pub struct Shape {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32
}

impl Shape {
    pub fn unpack(&self) -> (f32, f32, f32, f32) {
        (self.x1, self.y1, self.x2, self.y2)
    }

    pub fn aspect_ratio(&self) -> f32 {
        (self.x2 - self.x1) / (self.y2 - self.y1)
    }

    pub fn angular_offsets(&self) -> (f32, f32) {
        let ar = self.aspect_ratio();
        let theta = (1.0 as f32).atan2(ar);
        let dx = OUTLINE_THICKNESS / theta.sin();
        let dy = dx / ar;
        (dx, dy)
    }
}

pub enum Tri {
    TopLeft,
    TopRight,
    BotLeft,
    BotRight
}

// Right angle triangle
pub fn tri(shape: &Shape, tri_type: Tri) -> Vec::<f32> {
    match tri_type {
        Tri::TopLeft  => vec![shape.x1, shape.y1,   shape.x1, shape.y2,   shape.x2, shape.y1],
        Tri::TopRight => vec![shape.x2, shape.y1,   shape.x1, shape.y1,   shape.x2, shape.y2],
        Tri::BotRight => vec![shape.x2, shape.y2,   shape.x2, shape.y1,   shape.x1, shape.y2],
        Tri::BotLeft  => vec![shape.x1, shape.y2,   shape.x2, shape.y2,   shape.x1, shape.y1],
    }
}

pub fn rec(shape: &Shape) -> Vec::<f32> {
    [tri(shape, Tri::TopLeft), tri(shape, Tri::BotRight)].concat()
}

// Top half of shape
pub fn rec_top(shape: &Shape) -> Vec::<f32> {
    let (x1, y1, x2, y2) = shape.unpack();
    rec(&Shape {x1, y1, x2, y2: y2 - (y2-y1)/2.0})
}

// Right half of shape
pub fn _rec_right(shape: &Shape) -> Vec::<f32> {
    let (x1, y1, x2, y2) = shape.unpack();
    rec(&Shape {x1: x1 + (x2-x1)/2.0, y1, x2, y2})
}

// Bottom half of shape
pub fn rec_bot(shape: &Shape) -> Vec::<f32> {
    let (x1, y1, x2, y2) = shape.unpack();
    rec(&Shape {x1, y1: y1 + (y2-y1)/2.0, x2, y2})
}

// Left half of shape
pub fn _rec_left(shape: &Shape) -> Vec::<f32> {
    let (x1, y1, x2, y2) = shape.unpack();
    rec(&Shape {x1, y1, x2: x2 - (x2-x1)/2.0, y2})
}

// Top-left quarter of shape
pub fn rec_tl(shape: &Shape) -> Vec::<f32> {
    let (x1, y1, x2, y2) = shape.unpack();
    rec(&Shape {x1, y1, x2: x2 - (x2-x1)/2.0, y2: y2 - (y2-y1)/2.0})
}

// Top-right quarter of shape
pub fn rec_tr(shape: &Shape) -> Vec::<f32> {
    let (x1, y1, x2, y2) = shape.unpack();
    rec(&Shape {x1: x1 + (x2-x1)/2.0, y1, x2, y2: y2 - (y2-y1)/2.0})
}

// Bottom-left quarter of shape
pub fn rec_bl(shape: &Shape) -> Vec::<f32> {
    let (x1, y1, x2, y2) = shape.unpack();
    rec(&Shape {x1, y1: y1 + (y2-y1)/2.0, x2: x2 - (x2-x1)/2.0, y2})
}

// Bottom-right quarter of shape
pub fn rec_br(shape: &Shape) -> Vec::<f32> {
    let (x1, y1, x2, y2) = shape.unpack();
    rec(&Shape {x1: x1 + (x2-x1)/2.0, y1: y1 + (y2-y1)/2.0, x2, y2})
}

pub fn tri_ol(shape: &Shape, tri_type: Tri) -> Vec::<f32> {
    match tri_type {
        Tri::TopLeft =>
            vec![tri_ol_tl_top(shape), tri_ol_tl_left(shape), tri_ol_tl_diag(shape)].concat(),
        Tri::TopRight =>
            vec![tri_ol_tr_top(shape), tri_ol_tr_right(shape), tri_ol_tr_diag(shape)].concat(),
        Tri::BotLeft =>
            vec![tri_ol_bl_bot(shape), tri_ol_bl_left(shape), tri_ol_bl_diag(shape)].concat(),
        Tri::BotRight =>
            vec![tri_ol_br_bot(shape), tri_ol_br_right(shape), tri_ol_br_diag(shape)].concat(),
    }
}

pub fn tri_ol_tl_top(shape: &Shape) -> Vec::<f32> {
    let (x1, y1, x2, _y2) = shape.unpack();
    let dx = OUTLINE_THICKNESS * shape.aspect_ratio();
    vec![x1, y1,  x1, y1 + OUTLINE_THICKNESS,  x2 - dx, y1 + OUTLINE_THICKNESS,
         x1, y1,  x2 - dx, y1 + OUTLINE_THICKNESS,  x2, y1]
}

pub fn tri_ol_tl_left(shape: &Shape) -> Vec::<f32> {
    let (x1, y1, _x2, y2) = shape.unpack();
    let dy = OUTLINE_THICKNESS / shape.aspect_ratio();
    vec![x1, y1,  x1, y2,  x1 + OUTLINE_THICKNESS, y2 - dy,
         x1, y1,  x1 + OUTLINE_THICKNESS, y2 - dy,  x1 + OUTLINE_THICKNESS, y1]
}

pub fn tri_ol_tl_diag(shape: &Shape) -> Vec::<f32> {
    let (x1, y1, x2, y2) = shape.unpack();
    let (dx, dy) = shape.angular_offsets();
    vec![x1, y2 - dy,  x1, y2,  x2 - dx, y1,
         x2 - dx, y1,  x1, y2,  x2, y1]
}

pub fn tri_ol_tr_top(shape: &Shape) -> Vec::<f32> {
    let (x1, y1, x2, _y2) = shape.unpack();
    let dx = OUTLINE_THICKNESS * shape.aspect_ratio();
    vec![x1, y1,  x1 + dx, y1 + OUTLINE_THICKNESS,   x2,  y1 + OUTLINE_THICKNESS,
         x1, y1,  x2,  y1 + OUTLINE_THICKNESS,   x2,  y1]
}

pub fn tri_ol_tr_right(shape: &Shape) -> Vec::<f32> {
    let (_x1, y1, x2, y2) = shape.unpack();
    let dy = OUTLINE_THICKNESS / shape.aspect_ratio();
    vec![x2 - OUTLINE_THICKNESS, y1,  x2 - OUTLINE_THICKNESS,  y2 - dy,  x2, y2,
         x2 - OUTLINE_THICKNESS, y1,  x2, y2,  x2, y1]
}

pub fn tri_ol_tr_diag(shape: &Shape) -> Vec::<f32> {
    let (x1, y1, x2, y2) = shape.unpack();
    let (dx, dy) = shape.angular_offsets();
    vec![x1, y1,  x2,   y2,    x2,  y2 - dy,
         x1, y1,  x2,  y2 - dy,  x1 + dx, y1]
}

pub fn tri_ol_bl_bot(shape: &Shape) -> Vec::<f32> {
    let (x1, _y1, x2, y2) = shape.unpack();
    let dx = OUTLINE_THICKNESS * shape.aspect_ratio();
    vec![x1,   y2 - OUTLINE_THICKNESS,   x1,  y2,  x2, y2,
         x1, y2 - OUTLINE_THICKNESS,   x2,  y2,   x2 - dx, y2 - OUTLINE_THICKNESS]
}

pub fn tri_ol_bl_left(shape: &Shape) -> Vec::<f32> {
    let (x1, y1, _x2, y2) = shape.unpack();
    let dy = OUTLINE_THICKNESS / shape.aspect_ratio();
    vec![x1, y1,  x1, y2,  x1 + OUTLINE_THICKNESS, y2,
         x1, y1,  x1 + OUTLINE_THICKNESS, y2,  x1 + OUTLINE_THICKNESS, y1 + dy]
}

pub fn tri_ol_bl_diag(shape: &Shape) -> Vec::<f32> {
    let (x1, y1, x2, y2) = shape.unpack();
    let (dx, dy) = shape.angular_offsets();
    vec![x1, y1 + dy,  x2 - dx, y2,  x2, y2,
         x1, y1 + dy,  x2,      y2,  x1, y1]
}

pub fn tri_ol_br_bot(shape: &Shape) -> Vec::<f32> {
    let (x1, _y1, x2, y2) = shape.unpack();
    let dx = OUTLINE_THICKNESS * shape.aspect_ratio();
    vec![x1, y2,  x1 + dx, y2 - OUTLINE_THICKNESS,  x2, y2 - OUTLINE_THICKNESS,
         x1, y2,  x2,      y2 - OUTLINE_THICKNESS,  x2, y2]
}

pub fn tri_ol_br_right(shape: &Shape) -> Vec::<f32> {
    let (_x1, y1, x2, y2) = shape.unpack();
    let dy = OUTLINE_THICKNESS / shape.aspect_ratio();
    vec![x2, y2,  x2 - OUTLINE_THICKNESS, y2,       x2 - OUTLINE_THICKNESS, y1 + dy,
         x2, y2,  x2 - OUTLINE_THICKNESS, y1 + dy,  x2, y1]
}

pub fn tri_ol_br_diag(shape: &Shape) -> Vec::<f32> {
    let (x1, y1, x2, y2) = shape.unpack();
    let (dx, dy) = shape.angular_offsets();
    vec![x1, y2,  x1 + dx, y2,       x2, y1 + dy,
         x1, y2,  x2,      y1 + dy,  x2, y1]
}

pub fn rec_ol(shape: &Shape) -> Vec::<f32> {
    [rec_ol_top(shape), rec_ol_right(shape), rec_ol_bot(shape), rec_ol_left(shape)].concat()
}

pub fn rec_ol_top(shape: &Shape) -> Vec::<f32> {
    rec(&Shape {x1: shape.x1, y1: shape.y1,
                 x2: shape.x2, y2: shape.y1 + OUTLINE_THICKNESS})
}

pub fn rec_ol_right(shape: &Shape) -> Vec::<f32> {
    rec(&Shape {x1: shape.x2 - OUTLINE_THICKNESS, y1: shape.y1,
                 x2: shape.x2, y2: shape.y2})
}

pub fn rec_ol_bot(shape: &Shape) -> Vec::<f32> {
    rec(&Shape {x1: shape.x1, y1: shape.y2 - OUTLINE_THICKNESS,
                 x2: shape.x2, y2: shape.y2})
}

pub fn rec_ol_left(shape: &Shape) -> Vec::<f32> {
    rec(&Shape {x1: shape.x1, y1: shape.y1,
                 x2: shape.x1 + OUTLINE_THICKNESS, y2: shape.y2})
}

pub fn rec_ol_no_top(shape: &Shape) -> Vec::<f32> {
    [rec_ol_right(shape), rec_ol_bot(shape), rec_ol_left(shape)].concat()
}

pub fn rec_ol_no_right(shape: &Shape) -> Vec::<f32> {
    [rec_ol_top(shape), rec_ol_bot(shape), rec_ol_left(shape)].concat()
}

pub fn rec_ol_no_bot(shape: &Shape) -> Vec::<f32> {
    [rec_ol_top(shape), rec_ol_right(shape), rec_ol_left(shape)].concat()
}

pub fn rec_ol_no_left(shape: &Shape) -> Vec::<f32> {
    [rec_ol_top(shape), rec_ol_right(shape), rec_ol_bot(shape)].concat()
}
