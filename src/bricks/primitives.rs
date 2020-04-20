pub const STUD_WIDTH: f32 = 10.0;
pub const STUD_HEIGHT: f32 = 12.0;
pub const PLATE_HEIGHT: f32 = 4.0;

const OUTLINE_THICKNESS: f32 = 0.8;

pub struct Shape {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32
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
    let x1 = shape.x1;
    let y1 = shape.y1;
    let x2 = shape.x2;
    let y2 = shape.y2;
    rec(&Shape {x1, y1, x2, y2: y2 - (y2-y1)/2.0})
}

// Right half of shape
pub fn rec_right(shape: &Shape) -> Vec::<f32> {
    let x1 = shape.x1;
    let y1 = shape.y1;
    let x2 = shape.x2;
    let y2 = shape.y2;
    rec(&Shape {x1: x1 + (x2-x1)/2.0, y1, x2, y2})
}

// Bottom half of shape
pub fn rec_bot(shape: &Shape) -> Vec::<f32> {
    let x1 = shape.x1;
    let y1 = shape.y1;
    let x2 = shape.x2;
    let y2 = shape.y2;
    rec(&Shape {x1, y1: y1 + (y2-y1)/2.0, x2, y2})
}

// Left half of shape
pub fn rec_left(shape: &Shape) -> Vec::<f32> {
    let x1 = shape.x1;
    let y1 = shape.y1;
    let x2 = shape.x2;
    let y2 = shape.y2;
    rec(&Shape {x1, y1, x2: x2 - (x2-x1)/2.0, y2})
}

// Top-left quarter of shape
pub fn rec_tl(shape: &Shape) -> Vec::<f32> {
    let x1 = shape.x1;
    let y1 = shape.y1;
    let x2 = shape.x2;
    let y2 = shape.y2;
    rec(&Shape {x1, y1, x2: x2 - (x2-x1)/2.0, y2: y2 - (y2-y1)/2.0})
}

// Top-right quarter of shape
pub fn rec_tr(shape: &Shape) -> Vec::<f32> {
    let x1 = shape.x1;
    let y1 = shape.y1;
    let x2 = shape.x2;
    let y2 = shape.y2;
    rec(&Shape {x1: x1 + (x2-x1)/2.0, y1, x2, y2: y2 - (y2-y1)/2.0})
}

// Bottom-left quarter of shape
pub fn rec_bl(shape: &Shape) -> Vec::<f32> {
    let x1 = shape.x1;
    let y1 = shape.y1;
    let x2 = shape.x2;
    let y2 = shape.y2;
    rec(&Shape {x1, y1: y1 + (y2-y1)/2.0, x2: x2 - (x2-x1)/2.0, y2})
}

// Bottom-right quarter of shape
pub fn rec_br(shape: &Shape) -> Vec::<f32> {
    let x1 = shape.x1;
    let y1 = shape.y1;
    let x2 = shape.x2;
    let y2 = shape.y2;
    rec(&Shape {x1: x1 + (x2-x1)/2.0, y1: y1 + (y2-y1)/2.0, x2, y2})
}

pub fn tri_ol(shape: &Shape, tri_type: Tri) -> Vec::<f32> {
    let width = shape.x2 - shape.x1;
    let height = shape.y2 - shape.y1;
    let hypotenuse = (width*width + height*height).sqrt();
    let dh1 = OUTLINE_THICKNESS*(width + hypotenuse) / height;
    let dh2 = OUTLINE_THICKNESS*(height + hypotenuse) / width;

    // Rectangular bounds
    let olx = shape.x1 - OUTLINE_THICKNESS;   // outer left
    let ilx = shape.x1 + OUTLINE_THICKNESS;   // inner left
    let oty = shape.y1 - OUTLINE_THICKNESS;   // outer top
    let ity = shape.y1 + OUTLINE_THICKNESS;   // inner top
    let orx = shape.x2 + OUTLINE_THICKNESS;   // outer right
    let irx = shape.x2 - OUTLINE_THICKNESS;   // inner right
    let oby = shape.y2 + OUTLINE_THICKNESS;   // outer bottom
    let iby = shape.y2 - OUTLINE_THICKNESS;   // inner bottom

    // Angular bounds
    let olex = shape.x1 - dh1;   // outer left
    let ilex = shape.x1 + dh1;   // inner left
    let otey = shape.y1 - dh2;   // outer top
    let itey = shape.y1 + dh2;   // inner top
    let orex = shape.x2 + dh1;   // outer right
    let irex = shape.x2 - dh1;   // inner right
    let obey = shape.y2 + dh2;   // outer bottom
    let ibey = shape.y2 - dh2;   // inner bottom

    match tri_type {
        Tri::TopLeft =>
            vec![olx,  oty,    ilx,  ity,    irex, ity,   // TOP
                 olx,  oty,    irex, ity,    orex, oty,
                 olx,  oty,    olx,  obey,   ilx,  ibey,  // LEFT
                 olx,  oty,    ilx,  ibey,   ilx,  ity,
                 ilx,  ibey,   olx,  obey,   irex, ity,   // HYPOTENUSE
                 irex, ity,    olx,  obey,   orex, oty],
        Tri::TopRight =>
            vec![olex, oty,    ilex, ity,    irx, ity,    // TOP
                 olex, oty,    irx,  ity,    orx, oty,
                 orx,  oty,    irx,  ity,    irx, ibey,   // RIGHT
                 orx,  oty,    irx,  ibey,   orx, obey,   
                 olex, oty,    orx,  obey,   irx, ibey,   // HYPOTENUSE
                 olex, oty,    irx,  ibey,   ilex, ity],
        Tri::BotLeft =>
            vec![olx,  oby,    ilx,  iby,    irex, iby,   // BOT
                 irex, iby,    olx,  oby,    orex, oby,
                 olx,  otey,   ilx,  itey,   olx,  oby,   // LEFT
                 ilx,  itey,   olx,  oby,    ilx,  iby,
                 orex, oby,    irex, iby,    ilx,  itey,  // HYPOTENUSE
                 orex, oby,    ilx,  itey,   olx,  otey],
        Tri::BotRight =>
            vec![olex, oby,    ilex, iby,    irx,  iby,   // BOT
                 olex, oby,    irx,  iby,    orx,  oby,
                 orx,  oby,    irx,  iby,    irx, itey,   // RIGHT
                 orx,  oby,    irx,  itey,   orx, otey,
                 olex, oby,    ilex, iby,    irx, itey,   // HYPOTENUSE
                 olex, oby,    irx,  itey,   orx, otey]
    }
}

pub fn rec_ol(shape: &Shape) -> Vec::<f32> {
    [rec_ol_top(shape), rec_ol_right(shape), rec_ol_bot(shape), rec_ol_left(shape)].concat()
}

pub fn rec_ol_top(shape: &Shape) -> Vec::<f32> {
    rec(&Shape {x1: shape.x1 - OUTLINE_THICKNESS, y1: shape.y1 - OUTLINE_THICKNESS,
                 x2: shape.x2 + OUTLINE_THICKNESS, y2: shape.y1 + OUTLINE_THICKNESS})
}

pub fn rec_ol_right(shape: &Shape) -> Vec::<f32> {
    rec(&Shape {x1: shape.x2 - OUTLINE_THICKNESS, y1: shape.y1 - OUTLINE_THICKNESS,
                 x2: shape.x2 + OUTLINE_THICKNESS, y2: shape.y2 + OUTLINE_THICKNESS})
}

pub fn rec_ol_bot(shape: &Shape) -> Vec::<f32> {
    rec(&Shape {x1: shape.x1 - OUTLINE_THICKNESS, y1: shape.y2 - OUTLINE_THICKNESS,
                 x2: shape.x2 + OUTLINE_THICKNESS, y2: shape.y2 + OUTLINE_THICKNESS})
}

pub fn rec_ol_left(shape: &Shape) -> Vec::<f32> {
    rec(&Shape {x1: shape.x1 - OUTLINE_THICKNESS, y1: shape.y1 - OUTLINE_THICKNESS,
                 x2: shape.x1 + OUTLINE_THICKNESS, y2: shape.y2 + OUTLINE_THICKNESS})
}

pub fn rec_ol_no_top(shape: &Shape) -> Vec::<f32> {
    let shape = &Shape {x1: shape.x1, y1: shape.y1 + OUTLINE_THICKNESS,
                       x2: shape.x2, y2: shape.y2};
    [rec_ol_right(shape), rec_ol_bot(shape), rec_ol_left(shape)].concat()
}

