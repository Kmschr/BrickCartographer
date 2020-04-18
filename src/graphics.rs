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

pub fn get_triangle(tri_type: Triangle, x1: f32, y1: f32, x2: f32, y2: f32) -> Vec<f32> {
    match tri_type {
        Triangle::TopLeft =>  vec![x1, y1,
                                   x1, y2,
                                   x2, y1],
        Triangle::TopRight => vec![x2, y1,
                                   x1, y1,
                                   x2, y2],
        Triangle::BotRight => vec![x2, y2,
                                   x2, y1,
                                   x1, y2],
        Triangle::BotLeft =>  vec![x1, y2,
                                   x2, y2,
                                   x1, y1],
    }
}

pub fn get_triangle_outline(tri_type: Triangle, x1: f32, y1: f32, x2: f32, y2: f32) -> Vec<f32> {
    let d = 0.8;
    let width = x2 - x1;
    let height = y2 - y1;
    let hypotenuse = (width*width + height*height).sqrt();
    let dh1 = d*(width + hypotenuse) / height;
    let dh2 = d*(height + hypotenuse) / width;

    // Outer Top Left
    let otlx = x1 - d;
    let otly = y1 - d;
    // Inner Top Left
    let itlx = x1 + d;
    let itly = y1 + d;
    // Outer Top Right
    let otrx = x2 + d;
    let otry = y1 - d;
    // Inner Top Right
    let itrx = x2 - d;
    let itry = y1 + d;
    // Outer Bot Left
    let oblx = x1 - d;
    let obly = y2 + d;
    // Inner Bot Left
    let iblx = x1 + d;
    let ibly = y2 - d;
    // Outer Bot Right
    let obrx = x2 + d;
    let obry = y2 + d;
    // Inner Bot Right
    let ibrx = x2 - d;
    let ibry = y2 - d;

    // Outer Top Left Extended
    let otlex = x1 - dh1;
    let otley = y1 - dh2;
    // Inner Top Left Extended
    let itlex = x1 + dh1;
    let itley = y1 + dh2;
    // Outer Top Right Extended
    let otrex = x2 + dh1;
    let otrey = y1 - dh2;
    // Inner Top Right Extended
    let itrex = x2 - dh1;
    let itrey = y1 + dh2;
    // Outer Bot Left Extended
    let oblex = x1 - dh1;
    let obley = y2 + dh2;
    // Inner Bot Left Extended
    let iblex = x1 + dh1;
    let ibley = y2 - dh2;
    // Outer Bot Right Extended
    let obrex = x2 + dh1;
    let obrey = y2 + dh2;
    // Inner Bot Right Extended
    let ibrex = x2 - dh1;
    let ibrey = y2 - dh2;

    match tri_type {
        Triangle::TopLeft => {
            vec![otlx, otly,    // TOP
                 itlx, itly,
                 itrex, itry,
                 otlx, otly,
                 itrex, itry,
                 otrex, otry,
                 otlx, otly,    // LEFT
                 oblx, obley,
                 iblx, ibley,
                 otlx, otly,
                 iblx, ibley,
                 itlx, itly,
                 iblx, ibley,    // HYPOTENUSE
                 oblx, obley,
                 itrex, itry,
                 itrex, itry,
                 oblx, obley,
                 otrex, otry]
        },
        Triangle::TopRight => {
            vec![otlex, otly,   // TOP
                 itlex, itly,
                 itrx, itry,
                 otlex, otly,
                 itrx, itry,
                 otrx, otry,
                 otrx, otry,    // RIGHT
                 itrx, itry,
                 ibrx, ibrey,
                 otrx, otry,
                 ibrx, ibrey,
                 obrx, obrey,
                 otlex, otly,   // HYPOTENUSE
                 obrx, obrey,
                 ibrx, ibrey,
                 otlex, otly,
                 ibrx, ibrey,
                 itlex, itly]
        },
        Triangle::BotLeft =>
            vec![otlx, otley,   // LEFT
                 itlx, itley,
                 oblx, obly,
                 itlx, itley,
                 oblx, obly,
                 iblx, ibly,
                 oblx, obly,    // BOT
                 iblx, ibly,
                 ibrex, ibry,
                 ibrex, ibry,
                 oblx, obly,
                 obrex, obry,
                 obrex, obry,   // HYPOTENUSE
                 ibrex, ibry,
                 itlx, itley,
                 obrex, obry,
                 itlx, itley,
                 otlx, otley],
        Triangle::BotRight =>
            vec![oblex, obly,   // BOT
                 iblex, ibly,
                 ibrx, ibry,
                 oblex, obly,
                 ibrx, ibry,
                 obrx, obry,
                 obrx, obry,    // RIGHT
                 ibrx, ibry,
                 itrx, itrey,
                 obrx, obry,
                 itrx, itrey,
                 otrx, otrey,
                 oblex, obly,   // HYPOTENUSE
                 iblex, ibly,
                 itrx, itrey,
                 oblex, obly,
                 itrx, itrey,
                 otrx, otrey]
    }
}

pub fn get_rect(x1: f32, y1: f32, x2: f32, y2: f32) -> Vec<f32> {
    vec![x1, y1, // Top-Left Tri (CCW)
         x1, y2,
         x2, y1,
         x2, y2, // Bottom-Right Tri (CCW)
         x2, y1,
         x1, y2]
}

pub fn get_rect_outline(x1: f32, y1: f32, x2: f32, y2: f32) -> Vec<f32> {
    let d = 0.8;
    let mut top = get_rect(x1-d, y1-d, x2+d, y1+d);
    let mut left = get_rect(x1-d, y1+d, x1+d, y2-d);
    let mut bot = get_rect(x1-d, y2-d, x2+d, y2+d);
    let mut right = get_rect(x2-d, y1+d, x2+d, y2-d);
    let mut outline = Vec::new();
    outline.append(&mut top);
    outline.append(&mut left);
    outline.append(&mut bot);
    outline.append(&mut right);
    outline
}

pub fn get_corner(direction: Direction, rotation: Rotation, x1: f32, y1: f32, x2: f32, y2: f32) -> Vec<f32> {
    match direction {
        Direction::ZPositive =>
            match rotation {
                Rotation::Deg0 => {
                    let mut corner = get_rect(x1, y1, x1 + STUD_WIDTH*2.0, y1 + STUD_WIDTH);
                    corner.append(&mut get_rect(x1, y1 + STUD_WIDTH, x1 + STUD_WIDTH, y1 + STUD_WIDTH*2.0));
                    corner
                },
                Rotation::Deg90 => {
                    let mut corner = get_rect(x1, y1, x1 + STUD_WIDTH*2.0, y1 + STUD_WIDTH);
                    corner.append(&mut get_rect(x1 + STUD_WIDTH, y1 + STUD_WIDTH, x1 + STUD_WIDTH*2.0, y1 + STUD_WIDTH*2.0));
                    corner
                },
                Rotation::Deg180 => {
                    let mut corner = get_rect(x1, y1 + STUD_WIDTH, x1 + STUD_WIDTH*2.0, y1 + STUD_WIDTH*2.0);
                    corner.append(&mut get_rect(x1 + STUD_WIDTH, y1, x1 + STUD_WIDTH*2.0, y1 + STUD_WIDTH));
                    corner
                },
                Rotation::Deg270 => {
                    let mut corner = get_rect(x1, y1 + STUD_WIDTH, x1 + STUD_WIDTH*2.0, y1 + STUD_WIDTH*2.0);
                    corner.append(&mut get_rect(x1, y1, x1 + STUD_WIDTH, y1 + STUD_WIDTH));
                    corner
                }
            }
        Direction::ZNegative =>
            match rotation {
                Rotation::Deg0 => {
                    let mut corner = get_rect(x1, y1, x1 + STUD_WIDTH*2.0, y1 + STUD_WIDTH);
                    corner.append(&mut get_rect(x1 + STUD_WIDTH, y1 + STUD_WIDTH, x1 + STUD_WIDTH*2.0, y1 + STUD_WIDTH*2.0));
                    corner
                },
                Rotation::Deg90 => {
                    let mut corner = get_rect(x1, y1, x1 + STUD_WIDTH*2.0, y1 + STUD_WIDTH);
                    corner.append(&mut get_rect(x1, y1 + STUD_WIDTH, x1 + STUD_WIDTH, y1 + STUD_WIDTH*2.0));
                    corner
                },
                Rotation::Deg180 => {
                    let mut corner = get_rect(x1, y1 + STUD_WIDTH, x1 + STUD_WIDTH*2.0, y1 + STUD_WIDTH*2.0);
                    corner.append(&mut get_rect(x1, y1, x1 + STUD_WIDTH, y1 + STUD_WIDTH));
                    corner
                },
                Rotation::Deg270 => {
                    let mut corner = get_rect(x1, y1 + STUD_WIDTH, x1 + STUD_WIDTH*2.0, y1 + STUD_WIDTH*2.0);
                    corner.append(&mut get_rect(x1 + STUD_WIDTH, y1, x1 + STUD_WIDTH*2.0, y1 + STUD_WIDTH));
                    corner
                }
            }
        _ =>
            get_rect(x1, y1, x2, y2)
    }
}

pub fn get_side_wedge(direction: Direction, rotation: Rotation, x1: f32, y1: f32, x2: f32, y2: f32) -> Vec<f32> {
    match direction {
        Direction::ZPositive => 
            match rotation {
                Rotation::Deg0 => get_triangle(Triangle::TopLeft, x1, y1, x2, y2),
                Rotation::Deg90 => get_triangle(Triangle::TopRight, x1, y1, x2, y2),
                Rotation::Deg180 => get_triangle(Triangle::BotRight, x1, y1, x2, y2),
                Rotation::Deg270 => get_triangle(Triangle::BotLeft, x1, y1, x2, y2)
            },
        Direction::ZNegative =>
            match rotation {
                Rotation::Deg0 => get_triangle(Triangle::TopRight, x1, y1, x2, y2),
                Rotation::Deg90 => get_triangle(Triangle::TopLeft, x1, y1, x2, y2),
                Rotation::Deg180 => get_triangle(Triangle::BotLeft, x1, y1, x2, y2),
                Rotation::Deg270 => get_triangle(Triangle::BotRight, x1, y1, x2, y2)
            },
        Direction::XPositive | Direction::XNegative | Direction::YPositive | Direction::YNegative => 
            get_rect(x1, y1, x2, y2),
    }
}

pub fn get_side_wedge_outline(direction: Direction, rotation: Rotation, x1: f32, y1: f32, x2: f32, y2: f32) -> Vec<f32> {
    match direction {
        Direction::ZPositive => 
            match rotation {
                Rotation::Deg0 => get_triangle_outline(Triangle::TopLeft, x1, y1, x2, y2),
                Rotation::Deg90 => get_triangle_outline(Triangle::TopRight, x1, y1, x2, y2),
                Rotation::Deg180 => get_triangle_outline(Triangle::BotRight, x1, y1, x2, y2),
                Rotation::Deg270 => get_triangle_outline(Triangle::BotLeft, x1, y1, x2, y2)
            },
        Direction::ZNegative =>
            match rotation {
                Rotation::Deg0 => get_triangle_outline(Triangle::TopRight, x1, y1, x2, y2),
                Rotation::Deg90 => get_triangle_outline(Triangle::TopLeft, x1, y1, x2, y2),
                Rotation::Deg180 => get_triangle_outline(Triangle::BotLeft, x1, y1, x2, y2),
                Rotation::Deg270 => get_triangle_outline(Triangle::BotRight, x1, y1, x2, y2)
            },
        Direction::XPositive | Direction::XNegative | Direction::YPositive | Direction::YNegative => 
            get_rect_outline(x1, y1, x2, y2),
    }
}

pub fn get_wedge(direction: Direction, rotation: Rotation, x1: f32, y1: f32, x2: f32, y2: f32) -> Vec<f32> {
    match rotation {
        Rotation::Deg0 | Rotation::Deg180 => 
            get_rect(x1, y1, x2, y2),
        Rotation::Deg90 =>
            match direction {
                Direction::XPositive => get_triangle(Triangle::BotLeft, x1, y1, x2, y2),
                Direction::XNegative => get_triangle(Triangle::TopRight, x1, y1, x2, y2),
                Direction::YPositive => get_triangle(Triangle::TopLeft, x1, y1, x2, y2),
                Direction::YNegative => get_triangle(Triangle::BotRight, x1, y1, x2, y2),
                Direction::ZPositive | Direction::ZNegative =>
                    get_rect(x1, y1, x2, y2),
            },
        Rotation::Deg270 =>
            match direction {
                Direction::XPositive => get_triangle(Triangle::TopLeft, x1, y1, x2, y2),
                Direction::XNegative => get_triangle(Triangle::BotRight, x1, y1, x2, y2),
                Direction::YPositive => get_triangle(Triangle::TopRight, x1, y1, x2, y2),
                Direction::YNegative => get_triangle(Triangle::BotLeft, x1, y1, x2, y2),
                Direction::ZPositive | Direction::ZNegative =>
                    get_rect(x1, y1, x2, y2),
            }
    }
}

pub fn get_wedge_outline(direction: Direction, rotation: Rotation, x1: f32, y1: f32, x2: f32, y2: f32) -> Vec<f32> {
    match rotation {
        Rotation::Deg0 | Rotation::Deg180 => 
            get_rect_outline(x1, y1, x2, y2),
        Rotation::Deg90 =>
            match direction {
                Direction::XPositive => get_triangle_outline(Triangle::BotLeft, x1, y1, x2, y2),
                Direction::XNegative => get_triangle_outline(Triangle::TopRight, x1, y1, x2, y2),
                Direction::YPositive => get_triangle_outline(Triangle::TopLeft, x1, y1, x2, y2),
                Direction::YNegative => get_triangle_outline(Triangle::BotRight, x1, y1, x2, y2),
                Direction::ZPositive | Direction::ZNegative =>
                    get_rect_outline(x1, y1, x2, y2),
            },
        Rotation::Deg270 =>
            match direction {
                Direction::XPositive => get_triangle_outline(Triangle::TopLeft, x1, y1, x2, y2),
                Direction::XNegative => get_triangle_outline(Triangle::BotRight, x1, y1, x2, y2),
                Direction::YPositive => get_triangle_outline(Triangle::TopRight, x1, y1, x2, y2),
                Direction::YNegative => get_triangle_outline(Triangle::BotLeft, x1, y1, x2, y2),
                Direction::ZPositive | Direction::ZNegative =>
                    get_rect_outline(x1, y1, x2, y2),
            }
    }
}

pub fn get_ramp(direction: Direction, rotation: Rotation, x1: f32, y1: f32, x2: f32, y2: f32) -> Vec<f32> {
    match direction {
        Direction::ZPositive | Direction::ZNegative =>
            get_rect(x1, y1, x2, y2),
        Direction::XPositive =>
            match rotation {
                Rotation::Deg0 | Rotation::Deg180 =>
                    get_rect(x1, y1, x2, y2),
                Rotation::Deg90 => {
                    let mut ramp_vec = get_rect(x1, y2 - STUD_WIDTH, x2, y2);
                    ramp_vec.append(&mut get_triangle(Triangle::BotLeft, x1, y1, x2, y2 - STUD_WIDTH));
                    ramp_vec
                },
                Rotation::Deg270 => {
                    let mut ramp_vec = get_rect(x1, y1, x2, y1 + STUD_WIDTH);
                    ramp_vec.append(&mut get_triangle(Triangle::TopLeft, x1, y1 + STUD_WIDTH, x2, y2));
                    ramp_vec
                }
            },
        Direction::XNegative =>
            match rotation {
                Rotation::Deg0 | Rotation::Deg180 =>
                    get_rect(x1, y1, x2, y2),
                Rotation::Deg90 => {
                    let mut ramp_vec = get_rect(x1, y1, x2, y1 + STUD_WIDTH);
                    ramp_vec.append(&mut get_triangle(Triangle::TopRight, x1, y1 + STUD_WIDTH, x2, y2));
                    ramp_vec
                },
                Rotation::Deg270 => {
                    let mut ramp_vec = get_rect(x1, y2 - STUD_WIDTH, x2, y2);
                    ramp_vec.append(&mut get_triangle(Triangle::BotRight, x1, y1, x2, y2 - STUD_WIDTH));
                    ramp_vec
                }
            },
        Direction::YPositive =>
            match rotation {
                Rotation::Deg0 | Rotation::Deg180 =>
                    get_rect(x1, y1, x2, y2),
                Rotation::Deg90 => {
                    let mut ramp_vec = get_rect(x1, y1, x1 + STUD_WIDTH, y2);
                    ramp_vec.append(&mut get_triangle(Triangle::TopLeft, x1 + STUD_WIDTH, y1, x2, y2));
                    ramp_vec
                },
                Rotation::Deg270 => {
                    let mut ramp_vec = get_rect(x2 - STUD_WIDTH, y1, x2, y2);
                    ramp_vec.append(&mut get_triangle(Triangle::TopRight, x1, y1, x2 - STUD_WIDTH, y2));
                    ramp_vec
                }
            },
        Direction::YNegative => 
            match rotation {
                Rotation::Deg0 | Rotation::Deg180 =>
                    get_rect(x1, y1, x2, y2),
                Rotation::Deg90 => {
                    let mut ramp_vec = get_rect(x2 - STUD_WIDTH, y1, x2, y2);
                    ramp_vec.append(&mut get_triangle(Triangle::BotRight, x1, y1, x2 - STUD_WIDTH, y2));
                    ramp_vec
                },
                Rotation::Deg270 => {
                    let mut ramp_vec = get_rect(x1, y1, x1 + STUD_WIDTH, y1 + STUD_HEIGHT);
                    ramp_vec.append(&mut get_triangle(Triangle::BotLeft, x1 + STUD_WIDTH, y1, x2, y2));
                    ramp_vec
                }
            }
    }
}

pub fn get_ramp_outline(direction: Direction, rotation: Rotation, x1: f32, y1: f32, x2: f32, y2: f32) -> Vec<f32> {
    match direction {
        Direction::ZPositive | Direction::ZNegative =>
            get_rect_outline(x1, y1, x2, y2),
        Direction::XPositive =>
            match rotation {
                Rotation::Deg0 | Rotation::Deg180 =>
                    get_rect_outline(x1, y1, x2, y2),
                Rotation::Deg90 => {
                    let mut ramp_vec = get_rect_outline(x1, y2 - STUD_WIDTH, x2, y2);
                    ramp_vec.append(&mut get_triangle_outline(Triangle::BotLeft, x1, y1, x2, y2 - STUD_WIDTH));
                    ramp_vec
                },
                Rotation::Deg270 => {
                    let mut ramp_vec = get_rect_outline(x1, y1, x2, y1 + STUD_WIDTH);
                    ramp_vec.append(&mut get_triangle_outline(Triangle::TopLeft, x1, y1 + STUD_WIDTH, x2, y2));
                    ramp_vec
                }
            },
        Direction::XNegative =>
            match rotation {
                Rotation::Deg0 | Rotation::Deg180 =>
                    get_rect_outline(x1, y1, x2, y2),
                Rotation::Deg90 => {
                    let mut ramp_vec = get_rect_outline(x1, y1, x2, y1 + STUD_WIDTH);
                    ramp_vec.append(&mut get_triangle_outline(Triangle::TopRight, x1, y1 + STUD_WIDTH, x2, y2));
                    ramp_vec
                },
                Rotation::Deg270 => {
                    let mut ramp_vec = get_rect_outline(x1, y2 - STUD_WIDTH, x2, y2);
                    ramp_vec.append(&mut get_triangle_outline(Triangle::BotRight, x1, y1, x2, y2 - STUD_WIDTH));
                    ramp_vec
                }
            },
        Direction::YPositive =>
            match rotation {
                Rotation::Deg0 | Rotation::Deg180 =>
                    get_rect_outline(x1, y1, x2, y2),
                Rotation::Deg90 => {
                    let mut ramp_vec = get_rect_outline(x1, y1, x1 + STUD_WIDTH, y2);
                    ramp_vec.append(&mut get_triangle_outline(Triangle::TopLeft, x1+STUD_WIDTH, y1, x2, y2));
                    ramp_vec
                },
                Rotation::Deg270 => {
                    let mut ramp_vec = get_rect_outline(x2 - STUD_WIDTH, y1, x2, y2);
                    ramp_vec.append(&mut get_triangle_outline(Triangle::TopRight, x1, y1, x2 - STUD_WIDTH, y2));
                    ramp_vec
                }
            },
        Direction::YNegative => 
            match rotation {
                Rotation::Deg0 | Rotation::Deg180 =>
                    get_rect_outline(x1, y1, x2, y2),
                Rotation::Deg90 => {
                    let mut ramp_vec = get_rect_outline(x2 - STUD_WIDTH, y1, x2, y2);
                    ramp_vec.append(&mut get_triangle_outline(Triangle::BotRight, x1, y1, x2 - STUD_WIDTH, y2));
                    ramp_vec
                },
                Rotation::Deg270 => {
                    let mut ramp_vec = get_rect_outline(x1, y1, x1 + STUD_WIDTH, y1 + STUD_HEIGHT);
                    ramp_vec.append(&mut get_triangle_outline(Triangle::BotLeft, x1 + STUD_WIDTH, y1, x2, y2));
                    ramp_vec
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
