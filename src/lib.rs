extern crate brs;
extern crate js_sys;
extern crate web_sys;
extern crate image;
extern crate wasm_bindgen;
extern crate console_error_panic_hook;

mod webgl;
mod graphics;
mod image_combiner;
mod bricks;
mod process;
mod color;
mod util;
mod m3;

use wasm_bindgen::prelude::*;
use process::BRSProcessor;
use image_combiner::ImageCombiner;
use color::Color;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen(js_name = loadFile)]
pub fn load_file(body: Vec<u8>) -> Result<BRSProcessor, JsValue> {
    BRSProcessor::load_file(body)
}

#[wasm_bindgen(js_name = getImageCombiner)]
pub fn get_image_combiner() -> ImageCombiner {
    console_error_panic_hook::set_once();
    ImageCombiner::new()
}

