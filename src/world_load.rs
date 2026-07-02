use std::collections::HashMap;

use wasm_bindgen::prelude::*;
use brickadia::save::{Brick, BrickColor, Color, Direction, Rotation, Size};
use brdb::{Brdb, BrFsReader, BrReader, BrickType, Brz, IntoReader};

use crate::process::RawSave;

const MAIN_GRID: usize = 1;

pub fn load_brz(body: &[u8]) -> Result<RawSave, JsValue> {
    let brz = Brz::read_slice(body)
        .map_err(|e| JsValue::from(format!("brdb error reading brz archive: {}", e)))?;
    load_world(&brz.into_reader())
}

pub fn load_brdb(body: &[u8]) -> Result<RawSave, JsValue> {
    let db = Brdb::from_bytes(body)
        .map_err(|e| JsValue::from(format!("brdb error opening database: {}", e)))?;
    load_world(&db.into_reader())
}

fn load_world<T: BrFsReader>(reader: &BrReader<T>) -> Result<RawSave, JsValue> {
    let global_data = reader.global_data()
        .map_err(|e| JsValue::from(format!("brdb error reading global data: {}", e)))?;

    let mut brick_assets: Vec<String> = Vec::new();
    let mut asset_indices: HashMap<String, u32> = HashMap::new();
    let mut bricks: Vec<Brick> = Vec::new();

    // Dynamic brick grids (vehicles etc.) are positioned by entity transforms
    // the renderer doesn't model, so only the main grid is mapped
    let chunks = reader.brick_chunk_index(MAIN_GRID)
        .map_err(|e| JsValue::from(format!("brdb error reading chunk index: {}", e)))?;

    for chunk in chunks {
        let soa = reader.brick_chunk_soa(MAIN_GRID, chunk.index)
            .map_err(|e| JsValue::from(format!("brdb error reading chunk {}: {}", chunk.index, e)))?;

        for brick in soa.iter_bricks(chunk.index, global_data.clone()) {
            let brick = brick
                .map_err(|e| JsValue::from(format!("brdb error reading brick in chunk {}: {}", chunk.index, e)))?;

            let name = brick.asset.asset().to_string();
            let next_index = brick_assets.len() as u32;
            let asset_name_index = *asset_indices.entry(name).or_insert_with_key(|name| {
                brick_assets.push(name.clone());
                next_index
            });

            let size = match brick.asset {
                BrickType::Procedural { size, .. } =>
                    Size::Procedural(size.x as u32, size.y as u32, size.z as u32),
                BrickType::Basic(_) => Size::Empty,
            };

            bricks.push(Brick {
                asset_name_index,
                size,
                position: (brick.position.x, brick.position.y, brick.position.z),
                direction: convert_direction(brick.direction),
                rotation: convert_rotation(brick.rotation),
                visibility: brick.visible,
                color: BrickColor::Unique(Color {
                    r: brick.color.r,
                    g: brick.color.g,
                    b: brick.color.b,
                    a: 255,
                }),
                ..Default::default()
            });
        }
    }

    let brick_count = bricks.len() as i32;

    Ok(RawSave {
        bricks,
        brick_assets,
        colors: Vec::new(),
        description: read_description(reader),
        brick_count,
    })
}

fn read_description<T: BrFsReader>(reader: &BrReader<T>) -> String {
    reader.read_file("Meta/Bundle.json").ok()
        .and_then(|data| serde_json::from_slice::<serde_json::Value>(&data).ok())
        .and_then(|json| json.get("description").and_then(|d| d.as_str()).map(String::from))
        .unwrap_or_default()
}

fn convert_direction(direction: brdb::Direction) -> Direction {
    match direction {
        brdb::Direction::XPositive => Direction::XPositive,
        brdb::Direction::XNegative => Direction::XNegative,
        brdb::Direction::YPositive => Direction::YPositive,
        brdb::Direction::YNegative => Direction::YNegative,
        brdb::Direction::ZPositive | brdb::Direction::MAX => Direction::ZPositive,
        brdb::Direction::ZNegative => Direction::ZNegative,
    }
}

fn convert_rotation(rotation: brdb::Rotation) -> Rotation {
    match rotation {
        brdb::Rotation::Deg0 => Rotation::Deg0,
        brdb::Rotation::Deg90 => Rotation::Deg90,
        brdb::Rotation::Deg180 => Rotation::Deg180,
        brdb::Rotation::Deg270 => Rotation::Deg270,
    }
}
