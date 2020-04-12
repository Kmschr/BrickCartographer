use crate::log;
use crate::render_webgl;
use crate::graphics::Bounds;

use brs::{HasHeader1, HasHeader2};
use wasm_bindgen::prelude::*;

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

    pub fn process_bricks(&mut self) {
        self.bricks
            .sort_unstable_by_key(|brick| brick.position.2 + brick.size.2 as i32);
        
        for brick in &self.bricks {
            let name = &self.brick_assets[brick.asset_name_index as usize];

            if !brick.visibility || name.chars().next() != Some('P') {
                continue;
            }

            let brick_bounds = Bounds::<i32> {
                x1: brick.position.0 - brick.size.0 as i32,
                y1: brick.position.1 - brick.size.1 as i32,
                x2: brick.position.0 + brick.size.0 as i32,
                y2: brick.position.1 + brick.size.1 as i32,
            };

            let brick_owner_oob = brick.owner_index as usize >= self.reader.brick_owners().len();

            if brick_owner_oob {
                //log(&format!("owner_index: {}", brick.owner_index));
            }

            if brick_bounds.x1.abs() > MAX_BRICK_DISTANCE || 
                brick_bounds.y1.abs() > MAX_BRICK_DISTANCE ||
                brick_bounds.x1 == brick_bounds.x2 ||
                brick_bounds.y1 == brick_bounds.y2 ||
                brick_owner_oob
            {
                if !brick_owner_oob {
                    //let brick_owner = &self.reader.brick_owners()[brick.owner_index as usize];
                    //log(&format!("{:?}", brick_owner));
                }
                //log(&format!("Brick {:?}", brick_bounds));
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
        }

        log(&format!("{:?}", self.bounds))
    }

    pub fn render(&self) -> Result<(), JsValue> {
        render_webgl::render()
    }
}
