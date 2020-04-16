use crate::log;
use crate::webgl;
use crate::graphics::*;

use brs::{HasHeader1, HasHeader2};
use wasm_bindgen::prelude::*;
use web_sys::WebGlRenderingContext;
use js_sys::Array;

const MAX_BRICK_DISTANCE: i32 = 10 * 10000;

#[wasm_bindgen]
pub struct JsSave {
    #[wasm_bindgen(skip)]
    pub reader: brs::read::ReaderAfterBricks,
    #[wasm_bindgen(skip)]
    pub bricks: Vec<brs::Brick>,
    #[wasm_bindgen(skip)]
    pub bounds: Bounds::<i32>,
    #[wasm_bindgen(skip)]
    pub description: String,
    #[wasm_bindgen(skip)]
    pub brick_count: i32,
    #[wasm_bindgen(skip)]
    pub brick_assets: Vec<String>,
    #[wasm_bindgen(skip)]
    pub context: WebGlRenderingContext,
    #[wasm_bindgen(skip)]
    pub colors: Vec<Color>,
    #[wasm_bindgen(skip)]
    pub center: Point,
    #[wasm_bindgen(skip)]
    pub offset: Point,
}

#[wasm_bindgen]
impl JsSave {
    pub fn map(&self) -> String {
        self.reader
            .map()
            .to_string()
    }

    pub fn description(&self) -> String {
        self.description.clone()
    }

    pub fn brick_count(&self) -> i32 {
        self.brick_count
    }

    pub fn process_bricks(&mut self) -> Result<(), JsValue> {
        self.bricks
            .sort_unstable_by_key(|brick| brick.position.2 + brick.size.2 as i32);

        let mut area_sum = 0.0;
        let mut point_sum = Point {x:0.0, y:0.0};
        
        for brick in &self.bricks {
            let name = &self.brick_assets[brick.asset_name_index as usize];

            if !brick.visibility || !name.starts_with('P') {
                continue;
            }

            let brick_bounds = Bounds::<i32> {
                x1: brick.position.0 - brick.size.0 as i32,
                y1: brick.position.1 - brick.size.1 as i32,
                x2: brick.position.0 + brick.size.0 as i32,
                y2: brick.position.1 + brick.size.1 as i32,
            };

            let brick_owner_oob = brick.owner_index as usize > self.reader.brick_owners().len();

            if brick_owner_oob {
                //log(&format!("owner_index: {}", brick.owner_index));
                return Err(JsValue::from_str("Save version not compatible w/ brs-rs"));
            }

            if brick_bounds.x1.abs() > MAX_BRICK_DISTANCE || 
                brick_bounds.y1.abs() > MAX_BRICK_DISTANCE ||
                brick_bounds.x1 == brick_bounds.x2 ||
                brick_bounds.y1 == brick_bounds.y2 ||
                brick_owner_oob
            {
                /*
                if !brick_owner_oob {
                    let brick_owner = &self.reader.brick_owners()[brick.owner_index as usize];
                    log(&format!("{:?}", brick_owner));
                }
                log(&format!("Brick {:?}", brick_bounds));
                */
            } else {
                if brick_bounds.x1 < self.bounds.x1 {
                    self.bounds.x1 = brick_bounds.x1;
                }
                if brick_bounds.y1 < self.bounds.y1 {
                    self.bounds.y1 = brick_bounds.y1;
                }
                if brick_bounds.x2 > self.bounds.x2 {
                    self.bounds.x2 = brick_bounds.x2;
                }
                if brick_bounds.y2 > self.bounds.y2 {
                    self.bounds.y2 = brick_bounds.y2;
                }
            }

            let area = brick.size.0 * brick.size.1;
            //log(&format!("{}", area));

            point_sum.x += (brick.position.0 * area as i32) as f32;
            point_sum.y += (brick.position.1 * area as i32) as f32;
            area_sum += area as f32;
        }

        self.colors = self.reader.colors().iter().map(convert_color).collect();

        self.center = Point {
            x: point_sum.x / area_sum,
            y: point_sum.y / area_sum,
        };

        Ok(())
    }

    pub fn render(&mut self, size_x: i32, size_y: i32, pan_x: f32, pan_y: f32, scale: f32, show_outlines: bool) -> Result<(), JsValue> {
        //log(&format!("{},{}", pan_x, pan_y));
        let pan = Point { x: pan_x, y: pan_y};
        let size = Point { x: size_x as f32, y: size_y as f32};
        webgl::render(&self, size, pan, scale, show_outlines)
    }
}
