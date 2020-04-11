extern crate web_sys;
extern crate wasm_bindgen;

use crate::save::JsSave;

use std::f64;
use wasm_bindgen::JsCast;
use log;

pub fn render(save: &JsSave, zoom: f64, pan_x: i32, pan_y: i32) {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_elements_by_class_name("map-canvas").get_with_index(0).unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    context.clear_rect(0.0, 0.0, 825.0, 800.0);
    context.save();

    let zoom = f64::powf(zoom, 2.0);
    let translate_x = pan_x as f64 / zoom as f64;
    let translate_y = pan_y as f64 / zoom as f64;
    log(&format!("{}, {}", translate_x, translate_y));

    match context.translate(translate_x, translate_y) {
        Ok(v) => v,
        Err(_e) => return
    };

    context.begin_path();

    context.set_fill_style(&wasm_bindgen::JsValue::from_str("rgb(255,0,255)"));

    // Draw a rectangle
    context.fill_rect(0.0, 0.0, 20.0, 20.0);

    context.restore();
}