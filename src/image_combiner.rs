use wasm_bindgen::prelude::*;
use image::png::PngEncoder;

// Stitches raw RGBA screenshot tiles into one PNG. Tiles are blitted straight
// into the full-size pixel buffer as they arrive, so peak memory is the final
// image plus one tile — the output isn't bounded by the browser's max canvas
// size the way a 2d-canvas stitch would be.
#[wasm_bindgen]
pub struct ImageCombiner {
    pixels: Vec<u8>,
    tile_width: u32,
    tile_height: u32,
    rows: u32,
    cols: u32,
}

impl Default for ImageCombiner {
    fn default() -> ImageCombiner {
        ImageCombiner {
            pixels: Vec::new(),
            tile_width: 0,
            tile_height: 0,
            rows: 0,
            cols: 0,
        }
    }
}

#[wasm_bindgen]
impl ImageCombiner {
    #[wasm_bindgen(js_name = setLayout)]
    pub fn set_layout(&mut self, tile_width: u32, tile_height: u32, rows: u32, cols: u32) {
        self.tile_width = tile_width;
        self.tile_height = tile_height;
        self.rows = rows;
        self.cols = cols;
        self.pixels.clear();
        self.pixels.resize((tile_width * cols * tile_height * rows * 4) as usize, 0);
    }

    // Copies one tile of tightly-packed RGBA pixels into place
    #[wasm_bindgen(js_name = pushPixels)]
    pub fn push_pixels(&mut self, tile: &[u8], row: u32, col: u32) -> Result<(), JsValue> {
        if row >= self.rows || col >= self.cols {
            return Err(JsValue::from_str("tile out of bounds"));
        }
        if tile.len() != (self.tile_width * self.tile_height * 4) as usize {
            return Err(JsValue::from_str("unexpected tile size"));
        }

        let image_width = (self.cols * self.tile_width) as usize;
        let tile_row_bytes = (self.tile_width * 4) as usize;
        for y in 0..self.tile_height as usize {
            let src = y * tile_row_bytes;
            let dst = ((row * self.tile_height) as usize + y) * image_width * 4
                + (col * self.tile_width) as usize * 4;
            self.pixels[dst..dst + tile_row_bytes].copy_from_slice(&tile[src..src + tile_row_bytes]);
        }

        Ok(())
    }

    #[wasm_bindgen(js_name = combineImages)]
    pub fn combine_images(&mut self) -> Result<Vec<u8>, JsValue> {
        let image_width = self.cols * self.tile_width;
        let image_height = self.rows * self.tile_height;

        let mut merged: Vec<u8> = Vec::new();
        let encoder = PngEncoder::new(&mut merged);
        match encoder.encode(&self.pixels, image_width, image_height, image::ColorType::Rgba8) {
            Ok(_v) => (),
            Err(_e) => return Err(JsValue::from_str("Error encoding to png"))
        };

        // Free the stitch buffer; the next capture calls setLayout again
        self.pixels = Vec::new();

        Ok(merged)
    }

}
