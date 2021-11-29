use crate::log;
use crate::webgl::*;
use crate::color::*;
use crate::graphics::*;
use crate::bricks::*;
use crate::m3;
use crate::*;

use std::collections::HashSet;

use web_sys::{WebGlRenderingContext, WebGlUniformLocation};
use js_sys::Array;
use wasm_bindgen::prelude::*;
use brickadia::read::SaveReader;
use brickadia::save::{Brick, Rotation, Direction, BrickColor, Size};

const NUM_DIVISIONS: usize = 500;

#[derive(PartialEq, Eq, Hash)]
pub struct BrickShape {
    name_index: u32,
    size: (u32, u32),
    position: (i32, i32),
    rotation: Rotation,
    direction: Direction,
}

#[wasm_bindgen]
pub struct BRSProcessor {
    gl: WebGlRenderingContext,
    matrix_uniform_location: WebGlUniformLocation,
    centroid: (i32, i32),
    bounds: (i32, i32, i32, i32),
    vertex_buffer: Vec<f32>,
    bricks: Vec<Brick>,
    brick_assets: Vec<String>,
    bot_height_indices: [i32; NUM_DIVISIONS],
    top_height_indices: [i32; NUM_DIVISIONS],
    colors: Vec<Color>,
    description: String,
    brick_count: i32,
}

#[wasm_bindgen]
impl BRSProcessor {
    pub fn load_file(body: Vec<u8>) -> Result<BRSProcessor, JsValue> {
        let mut reader = match SaveReader::new(body.as_slice()) {
            Ok(v) => v,
            Err(_e) => return Err(JsValue::from("brickadia-rs error creating save reader"))
        };
        let save = match reader.read_all() {
            Ok(v) => v,
            Err(_e) => return Err(JsValue::from("brickadia-rs error reading file"))
        };
        
        let brick_assets = save.header2.brick_assets;
        let mut bricks: Vec<Brick> = save.bricks.into_iter()
            .filter_map(|b| util::filter_and_transform_brick(b, &brick_assets))
            .collect();
        bricks.sort_unstable_by_key(|b| util::top_surface(b));
    
         // Get color list as rgba 0.0-1.0 f32
        let mut colors: Vec<Color> = save.header2.colors.iter().map(convert_color).collect();
        for color in &mut colors {
            color.convert_to_srgb();
        }
    
        let description: String = save.header1.description;
        let brick_count: i32 = save.header1.brick_count as i32;
    
        let centroid = util::calculate_centroid(&bricks);
        let bounds = util::calculate_bounds(&bricks, centroid);

        /*
        let furthest_brick = util::find_furthest_brick(centroid, &bricks);
        let furthest_brick_owner_index = match furthest_brick.owner_index {
            None => 0usize,
            Some(x) => x as usize
        };

        let mut players = util::brick_count_by_player(&bricks, &reader.brick_owners());
        players.sort_unstable_by_key(|p| p.brick_count);

        log(&format!("{:?}", bounds));
        log(&format!("{:?}", furthest_brick));
       // log(&format!("{:?}", reader.brick_owners()));
        log(&reader.brick_owners()[furthest_brick_owner_index].name);
       // log(&format!("{:?}", players));
       */
    
        let (gl, matrix_uniform_location) = webgl::get_rendering_context()?;

        let mut processor = BRSProcessor {
            bricks,
            brick_assets,
            colors,
            description,
            brick_count,
            gl,
            matrix_uniform_location,
            centroid,
            bounds,
            vertex_buffer: Vec::new(),
            bot_height_indices: [-1; NUM_DIVISIONS],
            top_height_indices: [-1; NUM_DIVISIONS],
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

        let mut min_height:i32 = std::i32::MAX;
        let mut max_height:i32 = std::i32::MIN;

        for brick in &self.bricks {
            let size = util::sizer(brick);
            let top = brick.position.2 + size.2 as i32;
            let bot = brick.position.2 - size.2 as i32;
            min_height = std::cmp::min(min_height, bot);
            max_height = std::cmp::max(max_height, top);
        }

        self.bot_height_indices = [-1; NUM_DIVISIONS];
        self.top_height_indices = [-1; NUM_DIVISIONS];

        let cutoff_jump: f64 = (max_height - min_height) as f64 / NUM_DIVISIONS as f64;
        let mut cutoff_index: i32 = 0;
       
        // Calculate shapes for rendering and save Centroid
        for brick in &self.bricks {
            if !brick.visibility {
                continue;
            }

            let name = &self.brick_assets[brick.asset_name_index as usize];

            let size = util::sizer(brick);
            let top = brick.position.2 + size.2 as i32;

            let mut cur_cutoff = min_height + (cutoff_jump * cutoff_index as f64) as i32;

            if cutoff_index < NUM_DIVISIONS as i32 && self.bot_height_indices[cutoff_index as usize] == -1 {
                while top >= cur_cutoff && cutoff_index < NUM_DIVISIONS as i32 {
                    self.bot_height_indices[cutoff_index as usize] = self.vertex_buffer.len() as i32;
                    cutoff_index += 1;
                    cur_cutoff = min_height + (cutoff_jump * cutoff_index as f64) as i32;
                }
            }

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

                // Add shape to save
                let fill = RenderObject {
                    vertices: verts,
                    color: brick_color,
                };
                self.vertex_buffer.append(&mut fill.get_vertex_array());
            }
            
            if draw_ols {
                // Add brick outline for rendering
                let ol_verts = calculate_brick_outline_vertices(&name, &brick);

                let outline = RenderObject {
                    vertices: ol_verts,
                    color: Color::black()
                };
                self.vertex_buffer.append(&mut outline.get_vertex_array());
            }

        }

        for i in 0..NUM_DIVISIONS-1 {
            self.top_height_indices[i] = self.bot_height_indices[i + 1];
        }
        self.top_height_indices[NUM_DIVISIONS-1] = self.vertex_buffer.len() as i32;

        Ok(())
    }

    #[wasm_bindgen(js_name = buildHeightmapVertexBuffer)]
    pub fn build_heightmap_vertex_buffer(&mut self) -> Result<(), JsValue> {
        self.clear_vertex_buffer();

        let mut min_height:i32 = std::i32::MAX;
        let mut max_height:i32 = std::i32::MIN;

        for brick in &self.bricks {
            let size = util::sizer(brick);
            let top = brick.position.2 + size.2 as i32;
            let bot = brick.position.2 - size.2 as i32;
            min_height = std::cmp::min(min_height, bot);
            max_height = std::cmp::max(max_height, top);
        }

        self.bot_height_indices = [-1; NUM_DIVISIONS];
        self.top_height_indices = [-1; NUM_DIVISIONS];

        let cutoff_jump: f64 = (max_height - min_height) as f64 / NUM_DIVISIONS as f64;
        let mut cutoff_index: i32 = 0;

        // Calculate shapes for rendering and save Centroid
        for i in 0..self.bricks.len() {
            let brick = &self.bricks[i];

            if !brick.visibility {
                continue;
            }
            let name = &self.brick_assets[brick.asset_name_index as usize];

            let verts = calculate_brick_vertices(&name, &brick);

            let height = brick.position.2;
            let relative_height = (height - min_height) as f32 / (max_height - min_height) as f32;

            let size = util::sizer(brick);
            let top = brick.position.2 + size.2 as i32;

            let mut cur_cutoff = min_height + (cutoff_jump * cutoff_index as f64) as i32;

            if cutoff_index < NUM_DIVISIONS as i32 && self.bot_height_indices[cutoff_index as usize] == -1 {
                while top >= cur_cutoff && cutoff_index < NUM_DIVISIONS as i32 {
                    self.bot_height_indices[cutoff_index as usize] = self.vertex_buffer.len() as i32;
                    cutoff_index += 1;
                    cur_cutoff = min_height + (cutoff_jump * cutoff_index as f64) as i32;
                }
            }

            // Add shape to save
            let mut vertex_array = Vec::new();
            let vertex_count = verts.len() / 2;
            for i in 0..vertex_count {
                vertex_array.push(verts[i*2]);
                vertex_array.push(verts[i*2 + 1]);
                for _ in 0..3 {
                    vertex_array.push(relative_height);
                }
            }
            self.vertex_buffer.append(&mut vertex_array);
        }

        for i in 0..NUM_DIVISIONS-1 {
            self.top_height_indices[i] = self.bot_height_indices[i + 1];
        }
        self.top_height_indices[NUM_DIVISIONS-1] = self.vertex_buffer.len() as i32;
        
        Ok(())
    }

    pub fn render(&mut self, size_x: i32, size_y: i32, pan_x: f32, pan_y: f32, scale: f32, rotation: f32, minz: f32, maxz: f32) -> Result<(), JsValue> {
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

        let clipped_buffer = &self.vertex_buffer[self.bot_height_indices[minz as usize] as usize..self.top_height_indices[maxz as usize] as usize];
        self.gl_buffer_data(clipped_buffer);

        let vertex_count = (clipped_buffer.len() / 5) as i32;

        if vertex_count > 0 {
            self.gl.draw_arrays(WebGlRenderingContext::TRIANGLES, 0, vertex_count);
        }

        Ok(())
    }

}

impl BRSProcessor {
    pub fn clear_vertex_buffer(&mut self) {
        self.vertex_buffer = Vec::with_capacity(self.bricks.len() * 6 * VERTEX_SIZE as usize);
    }

    pub fn discard_hidden_bricks(&mut self) {
        // Don't render bricks that are obviously hidden (same sized bricks stacked on top of eachother)
        let mut unique_shapes = HashSet::<BrickShape>::new();
        let mut copy_count = 0;
        for i in (0..self.bricks.len()).rev() {
            let size = match self.bricks[i].size {
                Size::Procedural(x, y, z) => (x, y, z),
                Size::Empty => (0, 0, 0),
            };
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

    pub fn gl_buffer_data(&self, vertex_buffer: &[f32]) {
        unsafe {
            self.gl.buffer_data_with_array_buffer_view(
                WebGlRenderingContext::ARRAY_BUFFER,
                &js_sys::Float32Array::view(vertex_buffer),
                WebGlRenderingContext::STATIC_DRAW
            );
        }
    }
}