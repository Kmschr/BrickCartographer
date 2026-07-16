use std::collections::HashMap;
use std::sync::Arc;

use brickadia::save::{Direction, Rotation};
use brdb::{Brdb, BrFsReader, BrReader, BrickType, Brz, IntoReader};
use brdb::schema::BrdbSchemaGlobalData;

use crate::brick::Brick;
use crate::util;

// Saves written by CL13911 and later store brick colors already in sRGB.
// Older ones stored linear values that have to be converted for display.
const SRGB_COLOR_CHANGELIST: u32 = 13911;

// Bricks belong to the chunk containing their origin but can overhang it, so
// world-space bounds derived from chunk indices get padded by roughly one
// max-size brick before they are safe to use as an occlusion-grid extent.
const CHUNK_OVERHANG: i32 = 2048;

const MAIN_GRID: usize = 1;

/// Streams a world's bricks one chunk at a time, top layer first, so
/// geometry can be built and shown progressively while the occlusion grid
/// accumulates coverage in the order it needs.
///
/// A trait because the brz reader's concrete type is private to brdb; boxing
/// erases it.
pub trait ChunkSource {
    fn brick_count(&self) -> i32;
    fn description(&self) -> &str;
    /// World-space xy rectangle covering every chunk plus overhang slack
    fn grid_bounds(&self) -> (i32, i32, i32, i32);
    /// Fraction of bricks parsed so far, 0.0..=1.0
    fn progress(&self) -> f32;
    /// Parses the next chunk into render-ready bricks. Returns the chunk's
    /// z layer and its visible bricks, or `None` when exhausted.
    fn next_chunk(&mut self, brick_assets: &mut Vec<String>) -> Result<Option<(i16, Vec<Brick>)>, String>;
}

struct ChunkStream<T: BrFsReader> {
    reader: BrReader<T>,
    // (chunk index, brick count), sorted z descending then y, x
    chunks: Vec<(brdb::ChunkIndex, u32)>,
    next: usize,
    bricks_seen: u64,
    bricks_total: u64,
    global_data: Arc<BrdbSchemaGlobalData>,
    linear_colors: bool,
    asset_indices: HashMap<String, u32>,
    description: String,
    grid_bounds: (i32, i32, i32, i32),
}

pub fn open_brz(body: &[u8]) -> Result<Box<dyn ChunkSource>, String> {
    let brz = Brz::read_slice(body)
        .map_err(|e| format!("brdb error reading brz archive: {}", e))?;
    Ok(Box::new(ChunkStream::open(brz.into_reader())?))
}

pub fn open_brdb(body: &[u8]) -> Result<Box<dyn ChunkSource>, String> {
    let db = Brdb::from_bytes(body)
        .map_err(|e| format!("brdb error opening database: {}", e))?;
    Ok(Box::new(ChunkStream::open(db.into_reader())?))
}

impl<T: BrFsReader> ChunkStream<T> {
    fn open(reader: BrReader<T>) -> Result<ChunkStream<T>, String> {
        let global_data = reader.global_data()
            .map_err(|e| format!("brdb error reading global data: {}", e))?;

        let bundle = reader.bundle_json().ok();
        let linear_colors = bundle.as_ref()
            .and_then(|b| parse_changelist(&b.game_version))
            .is_some_and(|cl| cl < SRGB_COLOR_CHANGELIST);
        let description = bundle.map(|b| b.description).unwrap_or_default();

        // Dynamic brick grids (vehicles etc.) are positioned by entity
        // transforms the renderer doesn't model, so only the main grid is
        // mapped
        let metas = reader.brick_chunk_index(MAIN_GRID)
            .map_err(|e| format!("brdb error reading chunk index: {}", e))?;

        let mut grid_bounds = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
        let mut bricks_total: u64 = 0;
        let mut chunks: Vec<(brdb::ChunkIndex, u32)> = Vec::with_capacity(metas.len());
        for meta in metas {
            let cs = meta.chunk_size;
            grid_bounds.0 = grid_bounds.0.min(meta.index.x as i32 * cs);
            grid_bounds.1 = grid_bounds.1.min(meta.index.y as i32 * cs);
            grid_bounds.2 = grid_bounds.2.max((meta.index.x as i32 + 1) * cs);
            grid_bounds.3 = grid_bounds.3.max((meta.index.y as i32 + 1) * cs);
            bricks_total += meta.num_bricks as u64;
            chunks.push((meta.index, meta.num_bricks));
        }
        grid_bounds.0 -= CHUNK_OVERHANG;
        grid_bounds.1 -= CHUNK_OVERHANG;
        grid_bounds.2 += CHUNK_OVERHANG;
        grid_bounds.3 += CHUNK_OVERHANG;

        // Top layer first for the occlusion grid; y/x order within a layer
        // keeps batches spatially coherent and the draw order deterministic
        chunks.sort_unstable_by_key(|(i, _)| (std::cmp::Reverse(i.z), i.y, i.x));

        Ok(ChunkStream {
            reader,
            chunks,
            next: 0,
            bricks_seen: 0,
            bricks_total,
            global_data,
            linear_colors,
            asset_indices: HashMap::new(),
            description,
            grid_bounds,
        })
    }

}

impl<T: BrFsReader> ChunkSource for ChunkStream<T> {
    fn brick_count(&self) -> i32 {
        self.bricks_total.min(i32::MAX as u64) as i32
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn grid_bounds(&self) -> (i32, i32, i32, i32) {
        self.grid_bounds
    }

    fn progress(&self) -> f32 {
        if self.bricks_total == 0 {
            return 1.0;
        }
        self.bricks_seen as f32 / self.bricks_total as f32
    }

    fn next_chunk(&mut self, brick_assets: &mut Vec<String>) -> Result<Option<(i16, Vec<Brick>)>, String> {
        let Some(&(index, num_bricks)) = self.chunks.get(self.next) else {
            return Ok(None);
        };
        self.next += 1;
        self.bricks_seen += num_bricks as u64;

        let soa = self.reader.brick_chunk_soa(MAIN_GRID, index)
            .map_err(|e| format!("brdb error reading chunk {}: {}", index, e))?;

        let mut bricks: Vec<Brick> = Vec::with_capacity(num_bricks as usize);
        for brick in soa.iter_bricks(index, self.global_data.clone()) {
            let brick = brick
                .map_err(|e| format!("brdb error reading brick in chunk {}: {}", index, e))?;

            if !brick.visible {
                continue;
            }

            let name = brick.asset.asset().to_string();
            let next_index = brick_assets.len() as u32;
            let asset_name_index = *self.asset_indices.entry(name).or_insert_with_key(|name| {
                brick_assets.push(name.clone());
                next_index
            });

            let procedural_size = match brick.asset {
                BrickType::Procedural { size, .. } => (size.x as u32, size.y as u32, size.z as u32),
                BrickType::Basic(_) => (0, 0, 0),
            };

            let mut color = crate::color::Color {
                r: brick.color.r as f32 / 255.0,
                g: brick.color.g as f32 / 255.0,
                b: brick.color.b as f32 / 255.0,
                a: 1.0,
            };
            if self.linear_colors {
                color.convert_to_srgb();
            }

            let rotation = convert_rotation(brick.rotation);
            let direction = convert_direction(brick.direction);
            bricks.push(Brick {
                position: (brick.position.x, brick.position.y, brick.position.z),
                size: util::transform_size(
                    &brick_assets[asset_name_index as usize],
                    procedural_size,
                    rotation.clone(),
                    direction.clone(),
                ),
                asset_name_index,
                color: color.to_bytes(),
                rotation,
                direction,
            });
        }

        Ok(Some((index.z, bricks)))
    }
}

// Bundle.json game versions look like "CL13911". Missing, unparsable, and the
// "CL0" placeholder tooling writes all mean "unknown", which is treated as
// current rather than assuming an ancient save.
fn parse_changelist(game_version: &str) -> Option<u32> {
    match game_version.strip_prefix("CL")?.parse().ok()? {
        0 => None,
        cl => Some(cl),
    }
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
