use crate::log;
use crate::webgl::*;
use crate::color::*;
use crate::graphics::*;
use crate::bricks::*;
use crate::m3;
use crate::*;

use std::collections::HashSet;

use web_sys::{WebGlBuffer, WebGlRenderingContext, WebGlUniformLocation};
use js_sys::Array;
use wasm_bindgen::prelude::*;
use brickadia::read::SaveReader;
use brickadia::save::{Brick, Rotation, Direction, BrickColor};

// Geometry is built and uploaded to the GPU in chunks so wasm memory stays
// bounded no matter the build size. Also keeps every draw call well under
// browser index-count caps (e.g. Firefox's webgl.max-vert-ids-per-draw, 30M).
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
}

#[derive(PartialEq, Eq, Hash)]
pub struct BrickShape {
    name_index: u32,
    size: (u32, u32),
    position: (i32, i32),
    rotation: Rotation,
    direction: Direction,
}

struct BufferChunk {
    vertex_buffer: WebGlBuffer,
    index_buffer: WebGlBuffer,
    index_count: i32,
}

#[wasm_bindgen]
pub struct BRSProcessor {
    gl: WebGlRenderingContext,
    matrix_uniform_location: WebGlUniformLocation,
    position_attribute_location: u32,
    color_attribute_location: u32,
    centroid: (i32, i32),
    bounds: (i32, i32, i32, i32),
    vertex_buffer: Vec<u8>,
    index_buffer: Vec<u32>,
    chunks: Vec<BufferChunk>,
    bricks: Vec<Brick>,
    brick_assets: Vec<String>,
    colors: Vec<Color>,
    description: String,
    brick_count: i32,
}

#[wasm_bindgen]
impl BRSProcessor {
    pub fn load_file(body: Vec<u8>) -> Result<BRSProcessor, JsValue> {
        let raw = if body.starts_with(b"BRZ") {
            crate::world_load::load_brz(&body)?
        } else if body.starts_with(b"SQLite format 3\0") {
            crate::world_load::load_brdb(&body)?
        } else {
            Self::load_brs(&body)?
        };

        let RawSave { bricks, brick_assets, colors, description, brick_count } = raw;

        let mut bricks: Vec<Brick> = bricks.into_iter()
            .filter_map(|b| util::filter_and_transform_brick(b, &brick_assets))
            .collect();
        bricks.sort_unstable_by_key(|b| util::top_surface(b));

        if bricks.is_empty() {
            return Err(JsValue::from("save contains no visible bricks"));
        }

        let centroid = util::calculate_centroid(&bricks);
        let bounds = util::calculate_bounds(&bricks, centroid);

        let (gl, matrix_uniform_location, position_attribute_location, color_attribute_location) = webgl::get_rendering_context()?;

        let mut processor = BRSProcessor {
            bricks,
            brick_assets,
            colors,
            description,
            brick_count,
            gl,
            matrix_uniform_location,
            position_attribute_location,
            color_attribute_location,
            centroid,
            bounds,
            vertex_buffer: Vec::new(),
            index_buffer: Vec::new(),
            chunks: Vec::new(),
        };

        processor.discard_hidden_bricks();

        Ok(processor)
    }

    // Save info getters for frontend
    pub fn description(&self) -> String {
        self.description.clone()
    }
    #[wasm_bindgen(js_name = brickCount)]
    pub fn brick_count(&self) -> i32 {
        self.brick_count
    }
    pub fn centroid(&self) -> Array {
        let centroid = Array::new();
        centroid.push(&JsValue::from(self.centroid.0));
        centroid.push(&JsValue::from(self.centroid.1));
        centroid
    }
    pub fn bounds(&self) -> Array {
        let x1 = JsValue::from(self.bounds.0);
        let y1 = JsValue::from(self.bounds.1);
        let x2 = JsValue::from(self.bounds.2);
        let y2 = JsValue::from(self.bounds.3);
        let bounds = Array::new();
        bounds.push(&x1);
        bounds.push(&y1);
        bounds.push(&x2);
        bounds.push(&y2);
        bounds
    }

    // Get rendering info needed from bricks
    #[wasm_bindgen(js_name = buildVertexBuffer)]
    pub fn build_vertex_buffer(&mut self, draw_ols: bool, draw_fills: bool) -> Result<(), JsValue> {
        self.clear_vertex_buffer();

        let included = self.visible_bricks();
        // Outline-only mode draws no fills, so nothing occludes anything
        let hidden = if draw_fills {
            self.cull_covered(&included)
        } else {
            vec![false; included.len()]
        };

        for (k, &i) in included.iter().enumerate() {
            if hidden[k] {
                continue;
            }
            let brick = &self.bricks[i];
            let name = &self.brick_assets[brick.asset_name_index as usize];

            // Get brick color as rgba 0.0 - 1.0 f32
            let mut brick_color;
            match &brick.color {
                BrickColor::Index(color_index) => {
                    brick_color = self.colors[*color_index as usize];
                },
                BrickColor::Unique(color) => {
                    brick_color = convert_color(&color);
                    brick_color.convert_to_srgb();
                },
            }
            if draw_fills {
                // Calculate Shape vertices
                let verts = calculate_brick_vertices(&name, &brick);
                push_shape(&mut self.vertex_buffer, &mut self.index_buffer, &verts, brick_color.to_bytes());
            }

            if draw_ols {
                // Add brick outline for rendering
                let ol_verts = calculate_brick_outline_vertices(&name, &brick);
                push_shape(&mut self.vertex_buffer, &mut self.index_buffer, &ol_verts, Color::black().to_bytes());
            }

            if self.index_buffer.len() >= CHUNK_INDEX_LIMIT {
                self.flush_chunk()?;
            }
        }

        self.flush_chunk()?;

        Ok(())
    }

    #[wasm_bindgen(js_name = buildHeightmapVertexBuffer)]
    pub fn build_heightmap_vertex_buffer(&mut self) -> Result<(), JsValue> {
        self.clear_vertex_buffer();

        let (min_height, max_height) = self.height_extent();

        let included = self.visible_bricks();
        let hidden = self.cull_covered(&included);

        for (k, &i) in included.iter().enumerate() {
            if hidden[k] {
                continue;
            }
            let brick = &self.bricks[i];
            let name = &self.brick_assets[brick.asset_name_index as usize];

            let verts = calculate_brick_vertices(&name, &brick);

            let height = brick.position.2;
            let relative_height = (height - min_height) as f32 / (max_height - min_height) as f32;

            // Add shape to save
            let level = (relative_height * 255.0) as u8;
            push_shape(&mut self.vertex_buffer, &mut self.index_buffer, &verts, [level, level, level, 255]);

            if self.index_buffer.len() >= CHUNK_INDEX_LIMIT {
                self.flush_chunk()?;
            }
        }

        self.flush_chunk()?;

        Ok(())
    }

    pub fn render(&mut self, size_x: i32, size_y: i32, pan_x: f32, pan_y: f32, scale: f32, rotation: f32) -> Result<(), JsValue> {
        let pan = Point { x: pan_x, y: pan_y};
        let size = Point { x: size_x as f32, y: size_y as f32};

        self.gl.viewport(0, 0, size.x as i32, size.y as i32);

        self.gl.clear_color(0.0, 0.0, 0.0, 0.0);
        self.gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

        let mut matrix = m3::projection(size.x, size.y);
        matrix = m3::translate(matrix, size.x/2.0, size.y/2.0);
        matrix = m3::scale(matrix, scale, scale);
        matrix = m3::rotate(matrix, rotation);
        matrix = m3::translate(matrix, pan.x - self.centroid.0 as f32, pan.y - self.centroid.1 as f32);

        self.gl.uniform_matrix3fv_with_f32_array(Some(self.matrix_uniform_location.as_ref()), false, &matrix);

        // Chunks hold exactly the visible bricks, in draw order
        for chunk in &self.chunks {
            self.gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&chunk.vertex_buffer));
            self.gl.vertex_attrib_pointer_with_i32(
                self.position_attribute_location,
                2, // Number of elements per attribute
                WebGlRenderingContext::FLOAT,
                false,
                VERTEX_STRIDE, // Size of individual vertex
                0 // Offset from beginning of a vertex to this attribute
            );
            self.gl.vertex_attrib_pointer_with_i32(
                self.color_attribute_location,
                4, // Number of elements per attribute
                WebGlRenderingContext::UNSIGNED_BYTE,
                true, // Normalize 0-255 to 0.0-1.0
                VERTEX_STRIDE, // Size of individual vertex
                2 * std::mem::size_of::<f32>() as i32 // Offset from beginning of a vertex to this attribute
            );
            self.gl.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER, Some(&chunk.index_buffer));
            self.gl.draw_elements_with_i32(
                WebGlRenderingContext::TRIANGLES,
                chunk.index_count,
                WebGlRenderingContext::UNSIGNED_INT,
                0
            );
        }

        Ok(())
    }

}

impl BRSProcessor {
    fn load_brs(body: &[u8]) -> Result<RawSave, JsValue> {
        let mut reader = match SaveReader::new(body) {
            Ok(v) => v,
            Err(_e) => return Err(JsValue::from("brickadia-rs error creating save reader"))
        };
        let save = match reader.read_all() {
            Ok(v) => v,
            Err(_e) => return Err(JsValue::from("brickadia-rs error reading file"))
        };

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
        })
    }

    fn height_extent(&self) -> (i32, i32) {
        let mut min_height: i32 = std::i32::MAX;
        let mut max_height: i32 = std::i32::MIN;
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

        let mut min_x = std::i32::MAX;
        let mut min_y = std::i32::MAX;
        let mut max_x = std::i32::MIN;
        let mut max_y = std::i32::MIN;
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

        let mut cull_count = 0;
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
                cull_count += 1;
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
        log(&format!("Bricks Culled: {}", cull_count));

        hidden
    }

    pub fn clear_vertex_buffer(&mut self) {
        for chunk in self.chunks.drain(..) {
            self.gl.delete_buffer(Some(&chunk.vertex_buffer));
            self.gl.delete_buffer(Some(&chunk.index_buffer));
        }
        self.vertex_buffer.clear();
        self.index_buffer.clear();
    }

    // Uploads the accumulated geometry to fresh GPU buffers and resets the
    // CPU-side vecs, keeping wasm memory bounded regardless of build size
    fn flush_chunk(&mut self) -> Result<(), JsValue> {
        if self.index_buffer.is_empty() {
            return Ok(());
        }

        let vertex_buffer = self.gl.create_buffer().ok_or_else(|| JsValue::from("failed to create vertex buffer"))?;
        let index_buffer = self.gl.create_buffer().ok_or_else(|| JsValue::from("failed to create index buffer"))?;

        self.gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));
        self.gl.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER, Some(&index_buffer));
        unsafe {
            self.gl.buffer_data_with_array_buffer_view(
                WebGlRenderingContext::ARRAY_BUFFER,
                &js_sys::Uint8Array::view(&self.vertex_buffer),
                WebGlRenderingContext::STATIC_DRAW
            );
            self.gl.buffer_data_with_array_buffer_view(
                WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
                &js_sys::Uint32Array::view(&self.index_buffer),
                WebGlRenderingContext::STATIC_DRAW
            );
        }

        self.chunks.push(BufferChunk {
            vertex_buffer,
            index_buffer,
            index_count: self.index_buffer.len() as i32,
        });
        self.vertex_buffer.clear();
        self.index_buffer.clear();

        Ok(())
    }

    pub fn discard_hidden_bricks(&mut self) {
        // Don't render bricks that are obviously hidden (same sized bricks stacked on top of eachother)
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
        //log(&format!("Bricks Rendered: {}", unique_shapes.len()));
        log(&format!("Bricks Discarded: {}", copy_count));
    }

}