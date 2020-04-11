use crate::render_2d;

use brs::{HasHeader1};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct JsSave {
    #[wasm_bindgen(skip)]
    pub reader: brs::read::ReaderAfterBricks,
    #[wasm_bindgen(skip)]
    pub bricks: brs::read::ReadBricks,
}

#[wasm_bindgen]
impl JsSave {
    pub fn map(&self) -> String {
        self.reader
            .map()
            .to_string()
    }
    
    pub fn description(&self) -> String {
        self.reader
            .description()
            .to_string()
    }

    pub fn brick_count(&self) -> i32 {
        self.reader.
            brick_count()
    }

    pub fn render(&self, zoom: f64, pan_x: i32, pan_y: i32) {
        render_2d::render(self, zoom, pan_x, pan_y);
    }
}
