
const CONTRAST: f32 = -20.0;
const FACTOR: f32 = (259.0 * (CONTRAST + 255.0)) / (255.0 * (259.0 - CONTRAST));
const BRIGHTNESS_MODIFIER: f32 = 1.2;

#[derive(Debug)]
pub struct Bounds<T> {
    pub x1: T,
    pub y1: T,
    pub x2: T,
    pub y2: T,
}

#[derive(Debug)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {

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
