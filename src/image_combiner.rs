use wasm_bindgen::prelude::*;
use image::{ImageFormat, DynamicImage, load_from_memory_with_format, ImageBuffer, RgbaImage, Rgba};
use image::png::PNGEncoder;

#[wasm_bindgen]
pub struct ImageCombiner {
    images: Vec<Vec<u8>>,
    width: u32,
    height: u32,
}

#[wasm_bindgen]
impl ImageCombiner {
    pub fn new() -> ImageCombiner {
        ImageCombiner {
            images: vec![Vec::new(); 1000],
            width: 800,
            height: 600,
        }
    }

    #[wasm_bindgen(js_name = pushImage)]
    pub fn add_image(&mut self, image: Vec<u8>, index: i32) {
        self.images[index as usize] = image;
    }

    #[wasm_bindgen(js_name = setSize)]
    pub fn set_size(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    #[wasm_bindgen(js_name = combineImages)]
    pub fn combine_images(&self, rows: u32, cols: u32) -> Result<Vec<u8>, JsValue> {
        let image_width = cols * self.width;
        let image_height = rows * self.height;

        /*
        log(&format!("Rows: {}, Cols: {}", rows, cols));
        log(&format!("Width: {}, Height: {}", self.width, self.height));
        log(&format!("Image Size: {}, {}", image_width, image_height));
        */

        let mut img: RgbaImage = ImageBuffer::new(image_width, image_height);

        for col in 0..cols {
            for row in 0..rows {
                let image = &self.images[(row*cols + col) as usize];
                let image = match load_from_memory_with_format(image, ImageFormat::Png) {
                    Ok(v) => v,
                    Err(_e) => {
                        continue;
                    }
                };
                let image = match image {
                    DynamicImage::ImageRgba8(img) => img,
                    _ => {
                        return Err(JsValue::from_str("Weird image format"));
                    },
                };
                for x in 0..self.width {
                    for y in 0..self.height {
                        let new_x = x + col * self.width;
                        let new_y = y + row * self.height;
                        let pixel = image.get_pixel(x, y);
                        img.put_pixel(new_x, new_y, *pixel);
                    }
                }

            }
        }

        let buffer = img.into_raw();
        let mut merged: Vec<u8> = Vec::new();
        let encoder = PNGEncoder::new(&mut merged);
        match encoder.encode(&buffer, image_width, image_height, image::ColorType::Rgba8) {
            Ok(_v) => (),
            Err(_e) => return Err(JsValue::from_str("Error encoding to png"))
        };

        Ok(merged)
    }

}
