use crate::color::*;
use crate::bricks::*;
use crate::graphics::push_shape;
use crate::m3;
use crate::util;

use std::collections::HashSet;

use brickadia::read::SaveReader;
use brickadia::save::{Brick, Rotation, Direction, BrickColor};

// Geometry is built and handed off in chunks so memory stays bounded no
// matter the build size. Also keeps every draw call well under browser
// index-count caps (e.g. Firefox's webgl.max-vert-ids-per-draw, 30M).
const CHUNK_INDEX_LIMIT: usize = 3_000_000;

// Coverage grid for occlusion culling: cells this many save units square
// (half a stud), coarsened as needed to cap the grid dimensions on huge maps.
const CULL_CELL_SIZE: i32 = 5;
const CULL_MAX_GRID_DIM: i32 = 4096;

// A parsed save in legacy .brs terms, before filtering/transforming.
// Bricks from newer formats are converted into this shape so the rest of
// the pipeline (sizing, geometry, culling) stays format-agnostic.
pub struct RawSave {
    pub bricks: Vec<Brick>,
    pub brick_assets: Vec<String>,
    pub colors: Vec<Color>,
    pub description: String,
    pub brick_count: i32,
    // Whether per-brick colors are stored linear and need converting to sRGB
    // for display. Newer saves store them as sRGB already.
    pub linear_colors: bool,
}

#[derive(PartialEq, Eq, Hash)]
struct BrickShape {
    name_index: u32,
    size: (u32, u32),
    position: (i32, i32),
    rotation: Rotation,
    direction: Direction,
}

#[derive(Clone, Copy, PartialEq)]
pub enum GeometryMode {
    Map { outlines: bool, fills: bool },
    Heightmap,
}

/// A loaded, filtered save ready to produce render geometry.
pub struct SaveData {
    bricks: Vec<Brick>,
    brick_assets: Vec<String>,
    colors: Vec<Color>,
    pub description: String,
    pub brick_count: i32,
    linear_colors: bool,
    pub centroid: (i32, i32),
    pub bounds: (i32, i32, i32, i32),
    /// Duplicate bricks dropped up front (same footprint stacked exactly)
    pub discarded: usize,
}

impl SaveData {
    /// Parses any supported save format (.brs, .brz, .brdb) from bytes.
    pub fn load(body: &[u8]) -> Result<SaveData, String> {
        let raw = if body.starts_with(b"BRZ") {
            crate::world_load::load_brz(body)?
        } else if body.starts_with(b"SQLite format 3\0") {
            crate::world_load::load_brdb(body)?
        } else {
            Self::load_brs(body)?
        };

        let RawSave { bricks, brick_assets, colors, description, brick_count, linear_colors } = raw;

        let mut bricks: Vec<Brick> = bricks.into_iter()
            .filter_map(|b| util::filter_and_transform_brick(b, &brick_assets))
            .collect();
        bricks.sort_unstable_by_key(|b| util::top_surface(b));

        if bricks.is_empty() {
            return Err("save contains no visible bricks".to_string());
        }

        let centroid = util::calculate_centroid(&bricks);
        let bounds = util::calculate_bounds(&bricks, centroid);

        let mut save = SaveData {
            bricks,
            brick_assets,
            colors,
            description,
            brick_count,
            linear_colors,
            centroid,
            bounds,
            discarded: 0,
        };
        save.discard_hidden_bricks();

        Ok(save)
    }

    fn load_brs(body: &[u8]) -> Result<RawSave, String> {
        let mut reader = SaveReader::new(body)
            .map_err(|_| "brickadia-rs error creating save reader".to_string())?;
        let save = reader.read_all()
            .map_err(|_| "brickadia-rs error reading file".to_string())?;

        // Get color list as rgba 0.0-1.0 f32
        let mut colors: Vec<Color> = save.header2.colors.iter().map(convert_color).collect();
        for color in &mut colors {
            color.convert_to_srgb();
        }

        Ok(RawSave {
            bricks: save.bricks,
            brick_assets: save.header2.brick_assets,
            colors,
            description: save.header1.description,
            brick_count: save.header1.brick_count as i32,
            linear_colors: true,
        })
    }

    /// Screen-space transform for the given viewport, pan (world units),
    /// scale, and rotation, centered on the save's centroid.
    pub fn view_matrix(&self, size_x: f32, size_y: f32, pan_x: f32, pan_y: f32, scale: f32, rotation: f32) -> [f32; 9] {
        let mut matrix = m3::projection(size_x, size_y);
        matrix = m3::translate(matrix, size_x / 2.0, size_y / 2.0);
        matrix = m3::scale(matrix, scale, scale);
        matrix = m3::rotate(matrix, rotation);
        matrix = m3::translate(matrix, pan_x - self.centroid.0 as f32, pan_y - self.centroid.1 as f32);
        matrix
    }

    /// Builds interleaved vertex/index geometry for the whole save, invoking
    /// `push_chunk` with each filled chunk (vertices as VERTEX_STRIDE bytes
    /// per vertex, u32 triangle-list indices). Returns the number of bricks
    /// skipped by occlusion culling.
    pub fn build_geometry(
        &self,
        mode: GeometryMode,
        mut push_chunk: impl FnMut(&[u8], &[u32]) -> Result<(), String>,
    ) -> Result<usize, String> {
        let included = self.visible_bricks();

        // Outline-only mode draws no fills, so nothing occludes anything
        let cull = match mode {
            GeometryMode::Map { fills, .. } => fills,
            GeometryMode::Heightmap => true,
        };
        let hidden = if cull {
            self.cull_covered(&included)
        } else {
            vec![false; included.len()]
        };
        let culled = hidden.iter().filter(|&&h| h).count();

        let height_extent = match mode {
            GeometryMode::Heightmap => Some(self.height_extent()),
            _ => None,
        };

        let mut vertex_buffer: Vec<u8> = Vec::new();
        let mut index_buffer: Vec<u32> = Vec::new();

        for (k, &i) in included.iter().enumerate() {
            if hidden[k] {
                continue;
            }
            let brick = &self.bricks[i];
            let name = &self.brick_assets[brick.asset_name_index as usize];

            match mode {
                GeometryMode::Map { outlines, fills } => {
                    if fills {
                        let verts = calculate_brick_vertices(name, brick);
                        push_shape(&mut vertex_buffer, &mut index_buffer, &verts, self.brick_color(brick).to_bytes());
                    }
                    if outlines {
                        let ol_verts = calculate_brick_outline_vertices(name, brick);
                        push_shape(&mut vertex_buffer, &mut index_buffer, &ol_verts, Color::black().to_bytes());
                    }
                }
                GeometryMode::Heightmap => {
                    let (min_height, max_height) = height_extent.unwrap();
                    let verts = calculate_brick_vertices(name, brick);
                    let relative_height = (brick.position.2 - min_height) as f32 / (max_height - min_height) as f32;
                    let level = (relative_height * 255.0) as u8;
                    push_shape(&mut vertex_buffer, &mut index_buffer, &verts, [level, level, level, 255]);
                }
            }

            if index_buffer.len() >= CHUNK_INDEX_LIMIT {
                push_chunk(&vertex_buffer, &index_buffer)?;
                vertex_buffer.clear();
                index_buffer.clear();
            }
        }

        if !index_buffer.is_empty() {
            push_chunk(&vertex_buffer, &index_buffer)?;
        }

        Ok(culled)
    }

    // Brick color as display-ready sRGB
    fn brick_color(&self, brick: &Brick) -> Color {
        match &brick.color {
            BrickColor::Index(color_index) => self.colors[*color_index as usize],
            BrickColor::Unique(color) => {
                let mut brick_color = convert_color(color);
                if self.linear_colors {
                    brick_color.convert_to_srgb();
                }
                brick_color
            }
        }
    }

    fn height_extent(&self) -> (i32, i32) {
        let mut min_height = i32::MAX;
        let mut max_height = i32::MIN;
        for brick in &self.bricks {
            let size = util::sizer(brick);
            let top = brick.position.2 + size.2 as i32;
            let bot = brick.position.2 - size.2 as i32;
            min_height = std::cmp::min(min_height, bot);
            max_height = std::cmp::max(max_height, top);
        }
        (min_height, max_height)
    }

    // Indices of visible bricks in draw order (bricks are pre-sorted by top surface)
    fn visible_bricks(&self) -> Vec<usize> {
        self.bricks.iter().enumerate()
            .filter(|(_, brick)| brick.visibility)
            .map(|(i, _)| i)
            .collect()
    }

    // Occlusion culling: walking bricks top-down, a brick is hidden if every
    // coverage-grid cell its footprint touches was fully covered by the
    // rectangular fills of bricks drawn above it. Conservative on both sides —
    // shaped bricks never cover, and partial cells never count as covered.
    fn cull_covered(&self, included: &[usize]) -> Vec<bool> {
        let mut hidden = vec![false; included.len()];
        if included.is_empty() {
            return hidden;
        }

        let mut min_x = i32::MAX;
        let mut min_y = i32::MAX;
        let mut max_x = i32::MIN;
        let mut max_y = i32::MIN;
        for &i in included {
            let brick = &self.bricks[i];
            let size = util::sizer(brick);
            min_x = std::cmp::min(min_x, brick.position.0 - size.0 as i32);
            min_y = std::cmp::min(min_y, brick.position.1 - size.1 as i32);
            max_x = std::cmp::max(max_x, brick.position.0 + size.0 as i32);
            max_y = std::cmp::max(max_y, brick.position.1 + size.1 as i32);
        }

        let extent = std::cmp::max(max_x - min_x, max_y - min_y);
        let cell = std::cmp::max(CULL_CELL_SIZE, (extent + CULL_MAX_GRID_DIM - 1) / CULL_MAX_GRID_DIM);
        let cols = ((max_x - min_x) / cell + 1) as usize;
        let rows = ((max_y - min_y) / cell + 1) as usize;
        let mut covered = vec![false; cols * rows];

        for k in (0..included.len()).rev() {
            let brick = &self.bricks[included[k]];
            let size = util::sizer(brick);
            let x1 = brick.position.0 - size.0 as i32 - min_x;
            let y1 = brick.position.1 - size.1 as i32 - min_y;
            let x2 = x1 + 2 * size.0 as i32;
            let y2 = y1 + 2 * size.1 as i32;
            if x2 <= x1 || y2 <= y1 {
                continue;
            }

            let mut all_covered = true;
            'query: for r in (y1 / cell)..=((y2 - 1) / cell) {
                for c in (x1 / cell)..=((x2 - 1) / cell) {
                    if !covered[r as usize * cols + c as usize] {
                        all_covered = false;
                        break 'query;
                    }
                }
            }
            if all_covered {
                hidden[k] = true;
                continue;
            }

            let name = &self.brick_assets[brick.asset_name_index as usize];
            if is_full_rect(name) {
                // Mark only cells lying entirely inside the footprint
                for r in ((y1 + cell - 1) / cell)..(y2 / cell) {
                    for c in ((x1 + cell - 1) / cell)..(x2 / cell) {
                        covered[r as usize * cols + c as usize] = true;
                    }
                }
            }
        }

        hidden
    }

    // Don't render bricks that are obviously hidden (same sized bricks
    // stacked exactly on top of each other)
    fn discard_hidden_bricks(&mut self) {
        let mut unique_shapes = HashSet::<BrickShape>::new();
        let mut copy_count = 0;
        for i in (0..self.bricks.len()).rev() {
            let size = util::sizer(&self.bricks[i]);
            let brick_shape = BrickShape {
                name_index: self.bricks[i].asset_name_index,
                position: (self.bricks[i].position.0, self.bricks[i].position.1),
                size: (size.0, size.1),
                rotation: self.bricks[i].rotation.clone(),
                direction: self.bricks[i].direction.clone()
            };

            if unique_shapes.contains(&brick_shape) {
                self.bricks[i].visibility = false;
                copy_count += 1;
            } else {
                unique_shapes.insert(brick_shape);
            }
        }
        self.discarded = copy_count;
    }
}
