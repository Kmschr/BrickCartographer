
#[derive(Debug, Copy, Clone)]
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

    pub fn convert_to_srgb(&mut self) {
        self.r = val_as_srgb(self.r);
        self.g = val_as_srgb(self.g);
        self.b = val_as_srgb(self.b);
    }
}

pub fn convert_color(color: &brickadia::save::Color) -> Color {
    Color {
        r: color.r as f32 / 255.0,
        g: color.g as f32 / 255.0,
        b: color.b as f32 / 255.0,
        a: 1.0,
    }
}

pub fn val_as_srgb(val: f32) -> f32 {
    if val > 0.003_130_8 {
        1.055 * val.powf(1.0 / 2.4) - 0.055
    } else {
        val * 12.92
    }    
}