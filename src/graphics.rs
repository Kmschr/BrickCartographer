use brs::{Direction, Rotation};
use crate::log;

const CONTRAST: f32 = 0.0;
const FACTOR: f32 = (259.0 * (CONTRAST + 255.0)) / (255.0 * (259.0 - CONTRAST));
const BRIGHTNESS_MODIFIER: f32 = 1.1;

pub const STUD_WIDTH: f32 = 10.0;
pub const STUD_HEIGHT: f32 = 12.0;
pub const PLATE_HEIGHT: f32 = 4.0;

pub enum Triangle {
    TopLeft,
    TopRight,
    BotLeft,
    BotRight
}

pub fn get_triangle(tri_type: Triangle, x1: f32, y1: f32, x2: f32, y2: f32) -> [f32; 6] {
    match tri_type {
        Triangle::TopLeft  => [x1, y1,   x1, y2,   x2, y1],
        Triangle::TopRight => [x2, y1,   x1, y1,   x2, y2],
        Triangle::BotRight => [x2, y2,   x2, y1,   x1, y2],
        Triangle::BotLeft  => [x1, y2,   x2, y2,   x1, y1],
    }
}

pub fn get_triangle_outline(tri_type: Triangle, x1: f32, y1: f32, x2: f32, y2: f32) -> [f32; 36] {
    let d = 0.8;
    let width = x2 - x1;
    let height = y2 - y1;
    let hypotenuse = (width*width + height*height).sqrt();
    let dh1 = d*(width + hypotenuse) / height;
    let dh2 = d*(height + hypotenuse) / width;

    // Rectangular bounds
    let olx = x1 - d;   // outer left
    let ilx = x1 + d;   // inner left
    let oty = y1 - d;   // outer top
    let ity = y1 + d;   // inner top
    let orx = x2 + d;   // outer right
    let irx = x2 - d;   // inner right
    let oby = y2 + d;   // outer bottom
    let iby = y2 - d;   // inner bottom

    // Angular bounds
    let olex = x1 - dh1;   // outer left
    let ilex = x1 + dh1;   // inner left
    let otey = y1 - dh2;   // outer top
    let itey = y1 + dh2;   // inner top
    let orex = x2 + dh1;   // outer right
    let irex = x2 - dh1;   // inner right
    let obey = y2 + dh2;   // outer bottom
    let ibey = y2 - dh2;   // inner bottom

    match tri_type {
        Triangle::TopLeft  =>
            [olx,  oty,    ilx,  ity,    irex, ity,   // TOP
             olx,  oty,    irex, ity,    orex, oty,
             olx,  oty,    olx,  obey,   ilx,  ibey,  // LEFT
             olx,  oty,    ilx,  ibey,   ilx,  ity,
             ilx,  ibey,   olx,  obey,   irex, ity,   // HYPOTENUSE
             irex, ity,    olx,  obey,   orex, oty],
        Triangle::TopRight =>
            [olex, oty,    ilex, ity,    irx, ity,    // TOP
             olex, oty,    irx,  ity,    orx, oty,
             orx,  oty,    irx,  ity,    irx, ibey,   // RIGHT
             orx,  oty,    irx,  ibey,   orx, obey,   
             olex, oty,    orx,  obey,   irx, ibey,   // HYPOTENUSE
             olex, oty,    irx,  ibey,   ilex, ity],
        Triangle::BotLeft  =>
            [olx,  otey,   ilx,  itey,   olx,  oby,   // LEFT
             ilx,  itey,   olx,  oby,    ilx,  iby,
             olx,  oby,    ilx,  iby,    irex, iby,   // BOT
             irex, iby,    olx,  oby,    orex, oby,
             orex, oby,    irex, iby,    ilx,  itey,  // HYPOTENUSE
             orex, oby,    ilx,  itey,   olx,  otey],
        Triangle::BotRight =>
            [olex, oby,    ilex, iby,    irx,  iby,   // BOT
             olex, oby,    irx,  iby,    orx,  oby,
             orx,  oby,    irx,  iby,    irx, itey,   // RIGHT
             orx,  oby,    irx,  itey,   orx, otey,
             olex, oby,    ilex, iby,    irx, itey,   // HYPOTENUSE
             olex, oby,    irx,  itey,   orx, otey]
    }
}

pub fn get_rect(x1: f32, y1: f32, x2: f32, y2: f32) -> [f32; 12] {
    [x1, y1,   x1, y2,   x2, y1,   // Top-Left Tri
     x2, y2,   x2, y1,   x1, y2]   // Bottom-Right Tri (CCW)
}

pub fn get_rect_outline(x1: f32, y1: f32, x2: f32, y2: f32) -> [f32; 48] {
    let d = 0.8;
    let olx = x1 - d;   // outer left
    let ilx = x1 + d;   // inner left
    let oty = y1 - d;   // outer top
    let ity = y1 + d;   // inner top
    let orx = x2 + d;   // outer right
    let irx = x2 - d;   // inner right
    let oby = y2 + d;   // outer bottom
    let iby = y2 - d;   // inner bottom

    [olx, oty,   olx, ity,   orx, oty,   // Top
     orx, ity,   orx, oty,   olx, ity,
     olx, ity,   olx, iby,   ilx, ity,   // Left
     ilx, iby,   ilx, ity,   olx, iby,
     olx, iby,   olx, oby,   orx, iby,   // Bot
     orx, oby,   orx, iby,   olx, oby,
     irx, ity,   irx, iby,   orx, ity,   // Right
     orx, iby,   orx, ity,   irx, iby]
}

pub fn get_corner(direction: Direction, rotation: Rotation, x1: f32, y1: f32, x2: f32, y2: f32) -> Vec<f32> {
    match direction {
        Direction::ZPositive =>
            match rotation {
                Rotation::Deg0 => {
                    let long = get_rect(x1, y1, x1 + STUD_WIDTH*2.0, y1 + STUD_WIDTH);
                    let short = get_rect(x1, y1 + STUD_WIDTH, x1 + STUD_WIDTH, y1 + STUD_WIDTH*2.0);
                    [long, short].concat()
                },
                Rotation::Deg90 => {
                    let long = get_rect(x1, y1, x1 + STUD_WIDTH*2.0, y1 + STUD_WIDTH);
                    let short = get_rect(x1 + STUD_WIDTH, y1 + STUD_WIDTH, x1 + STUD_WIDTH*2.0, y1 + STUD_WIDTH*2.0);
                    [long, short].concat()
                },
                Rotation::Deg180 => {
                    let long = get_rect(x1, y1 + STUD_WIDTH, x1 + STUD_WIDTH*2.0, y1 + STUD_WIDTH*2.0);
                    let short = get_rect(x1 + STUD_WIDTH, y1, x1 + STUD_WIDTH*2.0, y1 + STUD_WIDTH);
                    [long, short].concat()
                },
                Rotation::Deg270 => {
                    let long = get_rect(x1, y1 + STUD_WIDTH, x1 + STUD_WIDTH*2.0, y1 + STUD_WIDTH*2.0);
                    let short = get_rect(x1, y1, x1 + STUD_WIDTH, y1 + STUD_WIDTH);
                    [long, short].concat()
                }
            }
        Direction::ZNegative =>
            match rotation {
                Rotation::Deg0 => {
                    let long = get_rect(x1, y1, x1 + STUD_WIDTH*2.0, y1 + STUD_WIDTH);
                    let short = get_rect(x1 + STUD_WIDTH, y1 + STUD_WIDTH, x1 + STUD_WIDTH*2.0, y1 + STUD_WIDTH*2.0);
                    [long, short].concat()
                },
                Rotation::Deg90 => {
                    let long = get_rect(x1, y1, x1 + STUD_WIDTH*2.0, y1 + STUD_WIDTH);
                    let short = get_rect(x1, y1 + STUD_WIDTH, x1 + STUD_WIDTH, y1 + STUD_WIDTH*2.0);
                    [long, short].concat()
                },
                Rotation::Deg180 => {
                    let long = get_rect(x1, y1 + STUD_WIDTH, x1 + STUD_WIDTH*2.0, y1 + STUD_WIDTH*2.0);
                    let short = get_rect(x1, y1, x1 + STUD_WIDTH, y1 + STUD_WIDTH);
                    [long, short].concat()
                },
                Rotation::Deg270 => {
                    let long = get_rect(x1, y1 + STUD_WIDTH, x1 + STUD_WIDTH*2.0, y1 + STUD_WIDTH*2.0);
                    let short = get_rect(x1 + STUD_WIDTH, y1, x1 + STUD_WIDTH*2.0, y1 + STUD_WIDTH);
                    [long, short].concat()
                }
            }
        _ =>
            get_rect(x1, y1, x2, y2).to_vec()
    }
}

pub fn get_side_wedge(direction: Direction, rotation: Rotation, x1: f32, y1: f32, x2: f32, y2: f32) -> Vec<f32> {
    match direction {
        Direction::ZPositive => 
            match rotation {
                Rotation::Deg0   => get_triangle(Triangle::TopLeft,  x1, y1, x2, y2).to_vec(),
                Rotation::Deg90  => get_triangle(Triangle::TopRight, x1, y1, x2, y2).to_vec(),
                Rotation::Deg180 => get_triangle(Triangle::BotRight, x1, y1, x2, y2).to_vec(),
                Rotation::Deg270 => get_triangle(Triangle::BotLeft,  x1, y1, x2, y2).to_vec()
            },
        Direction::ZNegative =>
            match rotation {
                Rotation::Deg0   => get_triangle(Triangle::TopRight, x1, y1, x2, y2).to_vec(),
                Rotation::Deg90  => get_triangle(Triangle::TopLeft,  x1, y1, x2, y2).to_vec(),
                Rotation::Deg180 => get_triangle(Triangle::BotLeft,  x1, y1, x2, y2).to_vec(),
                Rotation::Deg270 => get_triangle(Triangle::BotRight, x1, y1, x2, y2).to_vec()
            },
        _ => 
            get_rect(x1, y1, x2, y2).to_vec(),
    }
}

pub fn get_side_wedge_outline(direction: Direction, rotation: Rotation, x1: f32, y1: f32, x2: f32, y2: f32) -> Vec<f32> {
    match direction {
        Direction::ZPositive => 
            match rotation {
                Rotation::Deg0   => get_triangle_outline(Triangle::TopLeft,  x1, y1, x2, y2).to_vec(),
                Rotation::Deg90  => get_triangle_outline(Triangle::TopRight, x1, y1, x2, y2).to_vec(),
                Rotation::Deg180 => get_triangle_outline(Triangle::BotRight, x1, y1, x2, y2).to_vec(),
                Rotation::Deg270 => get_triangle_outline(Triangle::BotLeft,  x1, y1, x2, y2).to_vec()
            },
        Direction::ZNegative =>
            match rotation {
                Rotation::Deg0   => get_triangle_outline(Triangle::TopRight, x1, y1, x2, y2).to_vec(),
                Rotation::Deg90  => get_triangle_outline(Triangle::TopLeft,  x1, y1, x2, y2).to_vec(),
                Rotation::Deg180 => get_triangle_outline(Triangle::BotLeft,  x1, y1, x2, y2).to_vec(),
                Rotation::Deg270 => get_triangle_outline(Triangle::BotRight, x1, y1, x2, y2).to_vec()
            },
        _ => 
            get_rect_outline(x1, y1, x2, y2).to_vec(),
    }
}

pub fn get_wedge(direction: Direction, rotation: Rotation, x1: f32, y1: f32, x2: f32, y2: f32) -> Vec<f32> {
    match rotation {
        Rotation::Deg90 =>
            match direction {
                Direction::XPositive => get_triangle(Triangle::BotLeft,  x1, y1, x2, y2).to_vec(),
                Direction::XNegative => get_triangle(Triangle::TopRight, x1, y1, x2, y2).to_vec(),
                Direction::YPositive => get_triangle(Triangle::TopLeft,  x1, y1, x2, y2).to_vec(),
                Direction::YNegative => get_triangle(Triangle::BotRight, x1, y1, x2, y2).to_vec(),
                Direction::ZPositive | Direction::ZNegative =>
                    get_rect(x1, y1, x2, y2).to_vec(),
            },
        Rotation::Deg270 =>
            match direction {
                Direction::XPositive => get_triangle(Triangle::TopLeft,  x1, y1, x2, y2).to_vec(),
                Direction::XNegative => get_triangle(Triangle::BotRight, x1, y1, x2, y2).to_vec(),
                Direction::YPositive => get_triangle(Triangle::TopRight, x1, y1, x2, y2).to_vec(),
                Direction::YNegative => get_triangle(Triangle::BotLeft,  x1, y1, x2, y2).to_vec(),
                Direction::ZPositive | Direction::ZNegative =>
                    get_rect(x1, y1, x2, y2).to_vec(),
            },
        _ =>
            get_rect(x1, y1, x2, y2).to_vec(),
    }
}

pub fn get_wedge_outline(direction: Direction, rotation: Rotation, x1: f32, y1: f32, x2: f32, y2: f32) -> Vec<f32> {
    match rotation {
        Rotation::Deg90 =>
            match direction {
                Direction::XPositive => get_triangle_outline(Triangle::BotLeft,  x1, y1, x2, y2).to_vec(),
                Direction::XNegative => get_triangle_outline(Triangle::TopRight, x1, y1, x2, y2).to_vec(),
                Direction::YPositive => get_triangle_outline(Triangle::TopLeft,  x1, y1, x2, y2).to_vec(),
                Direction::YNegative => get_triangle_outline(Triangle::BotRight, x1, y1, x2, y2).to_vec(),
                Direction::ZPositive | Direction::ZNegative =>
                    get_rect_outline(x1, y1, x2, y2).to_vec(),
            },
        Rotation::Deg270 =>
            match direction {
                Direction::XPositive => get_triangle_outline(Triangle::TopLeft,  x1, y1, x2, y2).to_vec(),
                Direction::XNegative => get_triangle_outline(Triangle::BotRight, x1, y1, x2, y2).to_vec(),
                Direction::YPositive => get_triangle_outline(Triangle::TopRight, x1, y1, x2, y2).to_vec(),
                Direction::YNegative => get_triangle_outline(Triangle::BotLeft,  x1, y1, x2, y2).to_vec(),
                Direction::ZPositive | Direction::ZNegative =>
                    get_rect_outline(x1, y1, x2, y2).to_vec(),
            },
        _ =>
            get_rect_outline(x1, y1, x2, y2).to_vec(),
    }
}

pub fn get_ramp(direction: Direction, rotation: Rotation, x1: f32, y1: f32, x2: f32, y2: f32) -> Vec<f32> {
    match direction {
        Direction::XPositive =>
            match rotation {
                Rotation::Deg0 | Rotation::Deg180 =>
                    get_rect(x1, y1, x2, y2).to_vec(),
                Rotation::Deg90 => {
                    let rect = get_rect(x1, y2 - STUD_WIDTH, x2, y2).to_vec();
                    let tri = get_triangle(Triangle::BotLeft, x1, y1, x2, y2 - STUD_WIDTH).to_vec();
                    [rect, tri].concat()
                },
                Rotation::Deg270 => {
                    let rect = get_rect(x1, y1, x2, y1 + STUD_WIDTH).to_vec();
                    let tri = get_triangle(Triangle::TopLeft, x1, y1 + STUD_WIDTH, x2, y2).to_vec();
                    [rect, tri].concat()
                }
            },
        Direction::XNegative =>
            match rotation {
                Rotation::Deg0 | Rotation::Deg180 =>
                    get_rect(x1, y1, x2, y2).to_vec(),
                Rotation::Deg90 => {
                    let rect = get_rect(x1, y1, x2, y1 + STUD_WIDTH).to_vec();
                    let tri = get_triangle(Triangle::TopRight, x1, y1 + STUD_WIDTH, x2, y2).to_vec();
                    [rect, tri].concat()
                },
                Rotation::Deg270 => {
                    let rect = get_rect(x1, y2 - STUD_WIDTH, x2, y2).to_vec();
                    let tri = get_triangle(Triangle::BotRight, x1, y1, x2, y2 - STUD_WIDTH).to_vec();
                    [rect, tri].concat()
                }
            },
        Direction::YPositive =>
            match rotation {
                Rotation::Deg0 | Rotation::Deg180 =>
                    get_rect(x1, y1, x2, y2).to_vec(),
                Rotation::Deg90 => {
                    let rect = get_rect(x1, y1, x1 + STUD_WIDTH, y2).to_vec();
                    let tri = get_triangle(Triangle::TopLeft, x1 + STUD_WIDTH, y1, x2, y2).to_vec();
                    [rect, tri].concat()
                },
                Rotation::Deg270 => {
                    let rect = get_rect(x2 - STUD_WIDTH, y1, x2, y2).to_vec();
                    let tri = get_triangle(Triangle::TopRight, x1, y1, x2 - STUD_WIDTH, y2).to_vec();
                    [rect, tri].concat()
                }
            },
        Direction::YNegative => 
            match rotation {
                Rotation::Deg0 | Rotation::Deg180 =>
                    get_rect(x1, y1, x2, y2).to_vec(),
                Rotation::Deg90 => {
                    let rect = get_rect(x2 - STUD_WIDTH, y1, x2, y2).to_vec();
                    let tri = get_triangle(Triangle::BotRight, x1, y1, x2 - STUD_WIDTH, y2).to_vec();
                    [rect, tri].concat()
                },
                Rotation::Deg270 => {
                    let rect = get_rect(x1, y1, x1 + STUD_WIDTH, y1 + STUD_HEIGHT).to_vec();
                    let tri = get_triangle(Triangle::BotLeft, x1 + STUD_WIDTH, y1, x2, y2).to_vec();
                    [rect, tri].concat()
                }
            },
        _ =>
            get_rect(x1, y1, x2, y2).to_vec(),
    }
}

pub fn get_ramp_outline(direction: Direction, rotation: Rotation, x1: f32, y1: f32, x2: f32, y2: f32) -> Vec<f32> {
    match direction {
        Direction::ZPositive | Direction::ZNegative =>
            get_rect_outline(x1, y1, x2, y2).to_vec(),
        Direction::XPositive =>
            match rotation {
                Rotation::Deg0 | Rotation::Deg180 =>
                    get_rect_outline(x1, y1, x2, y2).to_vec(),
                Rotation::Deg90 => {
                    let rect = get_rect_outline(x1, y2 - STUD_WIDTH, x2, y2).to_vec();
                    let tri = get_triangle_outline(Triangle::BotLeft, x1, y1, x2, y2 - STUD_WIDTH).to_vec();
                    [rect, tri].concat()
                },
                Rotation::Deg270 => {
                    let rect = get_rect_outline(x1, y1, x2, y1 + STUD_WIDTH).to_vec();
                    let tri = get_triangle_outline(Triangle::TopLeft, x1, y1 + STUD_WIDTH, x2, y2).to_vec();
                    [rect, tri].concat()
                }
            },
        Direction::XNegative =>
            match rotation {
                Rotation::Deg0 | Rotation::Deg180 =>
                    get_rect_outline(x1, y1, x2, y2).to_vec(),
                Rotation::Deg90 => {
                    let rect = get_rect_outline(x1, y1, x2, y1 + STUD_WIDTH).to_vec();
                    let tri = get_triangle_outline(Triangle::TopRight, x1, y1 + STUD_WIDTH, x2, y2).to_vec();
                    [rect, tri].concat()
                },
                Rotation::Deg270 => {
                    let rect = get_rect_outline(x1, y2 - STUD_WIDTH, x2, y2).to_vec();
                    let tri = get_triangle_outline(Triangle::BotRight, x1, y1, x2, y2 - STUD_WIDTH).to_vec();
                    [rect, tri].concat()
                }
            },
        Direction::YPositive =>
            match rotation {
                Rotation::Deg0 | Rotation::Deg180 =>
                    get_rect_outline(x1, y1, x2, y2).to_vec(),
                Rotation::Deg90 => {
                    let rect = get_rect_outline(x1, y1, x1 + STUD_WIDTH, y2).to_vec();
                    let tri = get_triangle_outline(Triangle::TopLeft, x1+STUD_WIDTH, y1, x2, y2).to_vec();
                    [rect, tri].concat()
                },
                Rotation::Deg270 => {
                    let rect = get_rect_outline(x2 - STUD_WIDTH, y1, x2, y2).to_vec();
                    let tri = get_triangle_outline(Triangle::TopRight, x1, y1, x2 - STUD_WIDTH, y2).to_vec();
                    [rect, tri].concat()
                }
            },
        Direction::YNegative => 
            match rotation {
                Rotation::Deg0 | Rotation::Deg180 =>
                    get_rect_outline(x1, y1, x2, y2).to_vec(),
                Rotation::Deg90 => {
                    let rect = get_rect_outline(x2 - STUD_WIDTH, y1, x2, y2).to_vec();
                    let tri = get_triangle_outline(Triangle::BotRight, x1, y1, x2 - STUD_WIDTH, y2).to_vec();
                    [rect, tri].concat()
                },
                Rotation::Deg270 => {
                    let rect = get_rect_outline(x1, y1, x1 + STUD_WIDTH, y1 + STUD_HEIGHT).to_vec();
                    let tri = get_triangle_outline(Triangle::BotLeft, x1 + STUD_WIDTH, y1, x2, y2).to_vec();
                    [rect, tri].concat()
                }
            }
    }
}

#[derive(Debug)]
pub struct Shape {
    pub vertices: Vec<f32>,
    pub color: Color
}

impl Shape {
    pub fn get_vertex_array(&self) -> Vec<f32> {
        let mut vertex_array = Vec::new();
        let vertex_count = self.vertices.len() / 2;

        for i in 0..vertex_count {
            vertex_array.push(self.vertices[i*2]);
            vertex_array.push(self.vertices[i*2 + 1]);
            vertex_array.push(self.color.r);
            vertex_array.push(self.color.g);
            vertex_array.push(self.color.b);
        }

        vertex_array
    }
}

#[derive(Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn black() -> Color {
        Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }
}

pub fn convert_color(color: &brs::Color) -> Color {
    let r = FACTOR * (color.r() as f32 - 128.0) + 128.0;
    let g = FACTOR * (color.g() as f32 - 128.0) + 128.0;
    let b = FACTOR * (color.b() as f32 - 128.0) + 128.0;
    let r = (r * BRIGHTNESS_MODIFIER).min(255.0);
    let g = (g * BRIGHTNESS_MODIFIER).min(255.0);
    let b = (b * BRIGHTNESS_MODIFIER).min(255.0);
    Color {
        r: r / 255.0,
        g: g / 255.0,
        b: b / 255.0,
        a: color.a() as f32 / 255.0,
    }
}
