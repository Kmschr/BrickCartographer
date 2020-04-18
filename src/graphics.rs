use brs::{Direction, Rotation};

const CONTRAST: f32 = -20.0;
const FACTOR: f32 = (259.0 * (CONTRAST + 255.0)) / (255.0 * (259.0 - CONTRAST));
const BRIGHTNESS_MODIFIER: f32 = 1.2;

#[derive(Debug)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

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

pub fn get_rect(x1: f32, y1: f32, x2: f32, y2: f32) -> Vec<f32> {
    vec![x1, y1, // Top-Left Tri (CCW)
         x1, y2,
         x2, y1,
         x2, y2, // Bottom-Right Tri (CCW)
         x2, y1,
         x1, y2]
}

pub fn get_rect_outline(x1: f32, y1: f32, x2: f32, y2: f32) -> Vec<f32> {
    let d = 0.5;
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

pub fn get_ramp(direction: Direction, rotation: Rotation, x1: f32, y1: f32, x2: f32, y2: f32) -> Vec<f32> {
    match direction {
        Direction::ZPositive | Direction::ZNegative =>
            get_rect(x1, y1, x2, y2),
        Direction::XPositive =>
            match rotation {
                Rotation::Deg0 | Rotation::Deg180 =>
                    get_rect(x1, y1, x2, y2),
                Rotation::Deg90 => {
                    let mut ramp_vec = get_rect(x1, y2 - 10.0, x2, y2);
                    ramp_vec.append(&mut get_triangle(Triangle::BotLeft, x1, y1, x2, y2 - 10.0));
                    ramp_vec
                },
                Rotation::Deg270 => {
                    let mut ramp_vec = get_rect(x1, y1, x2, y1 + 10.0);
                    ramp_vec.append(&mut get_triangle(Triangle::TopLeft, x1, y1 + 10.0, x2, y2));
                    ramp_vec
                }
            },
        Direction::XNegative =>
            match rotation {
                Rotation::Deg0 | Rotation::Deg180 =>
                    get_rect(x1, y1, x2, y2),
                Rotation::Deg90 => {
                    let mut ramp_vec = get_rect(x1, y1, x2, y1 + 10.0);
                    ramp_vec.append(&mut get_triangle(Triangle::TopRight, x1, y1 + 10.0, x2, y2));
                    ramp_vec
                },
                Rotation::Deg270 => {
                    let mut ramp_vec = get_rect(x1, y2 - 10.0, x2, y2);
                    ramp_vec.append(&mut get_triangle(Triangle::BotRight, x1, y1, x2, y2 - 10.0));
                    ramp_vec
                }
            },
        Direction::YPositive =>
            match rotation {
                Rotation::Deg0 | Rotation::Deg180 =>
                    get_rect(x1, y1, x2, y2),
                Rotation::Deg90 => {
                    let mut ramp_vec = get_rect(x1, y1, x1+10.0, y2);
                    ramp_vec.append(&mut get_triangle(Triangle::TopLeft, x1+10.0, y1, x2, y2));
                    ramp_vec
                },
                Rotation::Deg270 => {
                    let mut ramp_vec = get_rect(x2 - 10.0, y1, x2, y2);
                    ramp_vec.append(&mut get_triangle(Triangle::TopRight, x1, y1, x2 - 10.0, y2));
                    ramp_vec
                }
            },
        Direction::YNegative => 
            match rotation {
                Rotation::Deg0 | Rotation::Deg180 =>
                    get_rect(x1, y1, x2, y2),
                Rotation::Deg90 => {
                    let mut ramp_vec = get_rect(x2 - 10.0, y1, x2, y2);
                    ramp_vec.append(&mut get_triangle(Triangle::BotRight, x1, y1, x2 - 10.0, y2));
                    ramp_vec
                },
                Rotation::Deg270 => {
                    let mut ramp_vec = get_rect(x1, y1, x1 + 10.0, y1 + 12.0);
                    ramp_vec.append(&mut get_triangle(Triangle::BotLeft, x1 + 10.0, y1, x2, y2));
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
