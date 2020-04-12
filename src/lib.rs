extern crate brs;
extern crate js_sys;
extern crate wasm_bindgen;

mod render_webgl;
//mod render_2d;
mod graphics;
mod save;

use wasm_bindgen::prelude::*;
use graphics::Bounds;
use save::JsSave;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn load_file(body: Vec<u8>) -> Result<JsSave, JsValue> {
    log("Loading Save...");
    let reader = match brs::Reader::new(body.as_slice()) {
        Ok(v) => v,
        Err(_e) => return Err(JsValue::from("Error reading file")),
    };
    let reader = match reader.read_header1() {
        Ok(v) => v,
        Err(_e) => return Err(JsValue::from("Error reading header1")),
    };
    let reader = match reader.read_header2() {
        Ok(v) => v,
        Err(_e) => return Err(JsValue::from("Error reading header2")),
    };
    let (reader, bricks) = match reader.iter_bricks_and_reader() {
        Ok(v) => v,
        Err(_e) => return Err(JsValue::from("Error reading bricks")),
    };
    Ok(JsSave {
        reader: reader,
        bricks: bricks
            .filter_map(Result::ok)
            .collect(),
        bounds: Bounds::<i32> {
            x1: i32::max_value(), 
            y1: i32::max_value(), 
            x2: i32::min_value(),
            y2: i32::min_value(),
        },
    })
}
