extern crate brs;
extern crate js_sys;
extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;
use brs::{HasHeader1, HasHeader2, Color};
use js_sys::Array;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
#[derive(Copy, Clone)]
pub struct Brick {
    asset_name_index: u32,
    size: (u32, u32, u32),
    position: (i32, i32, i32),
    direction: u32,
    rotation: u32,
    visibility: bool,
    color: i32
}

#[wasm_bindgen]
impl Brick {
    pub fn asset_name_index(&self) -> u32 {
        self.asset_name_index
    }
    pub fn size(&self) -> Array {
        let size = vec![self.size.0, self.size.1, self.size.2];
        size.into_iter().map(JsValue::from).collect()
    }
    pub fn position(&self) -> Array {
        let pos = vec![self.position.0, self.position.1, self.position.2];
        pos.into_iter().map(JsValue::from).collect()
    }
    pub fn direction(&self) -> u32 {
        self.direction
    }
    pub fn rotation(&self) -> u32 {
        self.rotation
    }
    pub fn visibility(&self) -> bool {
        self.visibility
    }
    pub fn color(&self) -> i32 {
        self.color
    }
}

#[wasm_bindgen]
pub struct Save {
    map: String,
    description: String,
    brick_count: i32,
    colors: Vec<String>,
    brick_assets: Vec<String>,
    bricks: Vec<Brick>
}

#[wasm_bindgen]
impl Save {
    pub fn map(&self) -> String {
        self.map.clone()
    }
    pub fn description(&self) -> String {
        self.description.clone()
    }
    pub fn brick_count(&self) -> i32 {
        self.brick_count
    }
    pub fn colors(&self) -> Array {
        self.colors.clone().into_iter().map(JsValue::from).collect()
    }
    pub fn brick_assets(&self) -> Array {
        self.brick_assets.clone().into_iter().map(JsValue::from).collect()
    }
    pub fn bricks(&self) -> Array {
        self.bricks.clone().into_iter().map(JsValue::from).collect()
    }
}

fn failed_save() -> Save {
    Save {
        map: "Unknown".to_string(),
        description: "Could not load file".to_string(),
        brick_count: 0,
        colors: Vec::new(),
        brick_assets: Vec::new(),
        bricks: Vec::new()
    }
}

fn color_to_string(color: &Color) -> String {
    let r = color.r().to_string();
    let g = color.g().to_string();
    let b = color.b().to_string();
    format!("rgba({},{},{},{})",r,g,b,color.a().to_string())
}

fn brick_info(brick: Result<brs::Brick, std::io::Error>) -> Brick {
    let brick = match brick {
        Ok(v) => v,
        Err(_e) => return Brick {
            asset_name_index: 0,
            size: (0, 0, 0),
            position: (0, 0, 0),
            direction: 0,
            rotation: 0,
            visibility: false,
            color: 0
        },
    };

    let direction = match brick.direction {
        brs::Direction::XPositive => 0,
        brs::Direction::XNegative => 1,
        brs::Direction::YPositive => 2,
        brs::Direction::YNegative => 3,
        brs::Direction::ZPositive => 4,
        brs::Direction::ZNegative => 5,
    };
    let rotation = match brick.rotation {
        brs::Rotation::Deg0 => 0,
        brs::Rotation::Deg90 => 1,
        brs::Rotation::Deg180 => 2,
        brs::Rotation::Deg270 => 3,
    };
    let color = match brick.color {
        brs::ColorMode::Set(n) => n as i32,
        brs::ColorMode::Custom(_n) => -1
    };

    Brick {
        asset_name_index: brick.asset_name_index,
        size: brick.size,
        position: brick.position,
        direction: direction,
        rotation: rotation,
        visibility: brick.visibility,
        color: color
    }
}

#[wasm_bindgen]
pub fn load_file(body: Vec<u8>) -> Save {
    log("Loading Save...");
    let reader = match brs::Reader::new(body.as_slice()) {
        Ok(v) => v,
        Err(_e) => return failed_save(),
    };
    let reader = match reader.read_header1() {
        Ok(v) => v,
        Err(_e) => return failed_save(),
    };
    let reader = match reader.read_header2() {
        Ok(v) => v,
        Err(_e) => return failed_save(),
    };
    let (reader, bricks) = match reader.iter_bricks_and_reader() {
        Ok(v) => v,
        Err(_e) => return failed_save(),
    };
    let mut loaded_bricks: Vec<Brick> = bricks.map(brick_info).collect();
    loaded_bricks.sort_by_key(|brick| brick.position.2 + brick.size.2 as i32);
    Save {
        map: reader.map().to_string(),
        description: reader.description().to_string(),
        brick_count: reader.brick_count(),
        colors: reader.colors().iter().map(color_to_string).collect(),
        brick_assets: reader.brick_assets().to_vec(),
        bricks: loaded_bricks
    }
}
