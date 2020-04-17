extern crate brs;
extern crate js_sys;
extern crate web_sys;
extern crate wasm_bindgen;

mod webgl;
//mod render_2d;
mod graphics;
mod save;

use brs::{HasHeader2};

use wasm_bindgen::prelude::*;
use graphics::*;
use save::JsSave;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn load_file(body: Vec<u8>) -> Result<JsSave, JsValue> {
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
    let assets = reader.brick_assets().to_vec();
    let (rendering_context, uniform_location) = webgl::get_rendering_context();
    Ok(JsSave {
        reader,
        bricks: bricks
            .filter_map(Result::ok)
            .collect(),
        brick_assets: assets,
        context: rendering_context.unwrap(),
        u_matrix: uniform_location.unwrap(),
        colors: Vec::new(),
        center: Point {x:0.0, y:0.0},
        shapes: Vec::new(),
    })
}
