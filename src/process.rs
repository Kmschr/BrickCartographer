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
use brs::{Brick, HasHeader1, HasHeader2, Rotation, Direction};

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
    colors: Vec<Color>,
    description: String,
    brick_count: i32,
}

#[wasm_bindgen]
impl BRSProcessor {
    pub fn load_file(body: Vec<u8>) -> Result<BRSProcessor, JsValue> {
        let reader = match brs::Reader::new(body.as_slice()) {
            Ok(v) => v,
            Err(_e) => return Err(JsValue::from("brs error reading file")),
        };
        let reader = match reader.read_header1() {
            Ok(v) => v,
            Err(_e) => return Err(JsValue::from("brs error reading header1")),
        };
        let reader = match reader.read_header2() {
            Ok(v) => v,
            Err(_e) => return Err(JsValue::from("brs error reading header2")),
        };
        let (reader, brs_bricks) = match reader.iter_bricks_and_reader() {
            Ok(v) => v,
            Err(_e) => return Err(JsValue::from("brs error reading bricks")),
        };
        
        let brick_assets: Vec<String> = reader.brick_assets().to_vec();
        let mut bricks: Vec<Brick> = brs_bricks
            .filter_map(|b| util::filter_and_transform_brick(b, &brick_assets))
            .collect();
        bricks.sort_unstable_by_key(|b| b.position.2 + b.size.2 as i32);
    
         // Get color list as rgba 0.0-1.0 f32
        let mut colors: Vec<Color> = reader.colors().iter().map(convert_color).collect();
        for color in &mut colors {
            color.convert_to_srgb();
        }
    
        let description: String = reader.description().to_string();
        let brick_count: i32 = reader.brick_count();
    
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

        // Don't render bricks that are obviously hidden (same sized bricks stacked on top of eachother)
        let mut unique_shapes = HashSet::<BrickShape>::new();
        let mut copy_count = 0;
        for i in (0..self.bricks.len()).rev() {
            let brick_shape = BrickShape {
                name_index: self.bricks[i].asset_name_index,
                position: (self.bricks[i].position.0, self.bricks[i].position.1),
                size: (self.bricks[i].size.0, self.bricks[i].size.1),
                rotation: self.bricks[i].rotation,
                direction: self.bricks[i].direction
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
       
        // Calculate shapes for rendering and save Centroid
        for brick in &self.bricks {
            if !brick.visibility {
                continue;
            }

            let name = &self.brick_assets[brick.asset_name_index as usize];

            // Get brick color as rgba 0.0 - 1.0 f32
            let mut brick_color = Color::black();
            match brick.color {
                brs::ColorMode::Set(color_index) => {
                    brick_color.r = self.colors[color_index as usize].r;
                    brick_color.g = self.colors[color_index as usize].g;
                    brick_color.b = self.colors[color_index as usize].b;
                    brick_color.a = self.colors[color_index as usize].a;
        
                },
                brs::ColorMode::Custom(color) => {
                    brick_color.r = color.r() as f32 / 255.0;
                    brick_color.g = color.g() as f32 / 255.0;
                    brick_color.b = color.b() as f32 / 255.0;
                    brick_color.a = color.a() as f32 / 255.0;
                    brick_color.convert_to_srgb();
                },
            }

            // Add brick as shape for rendering
            let shape = Shape {
                x1: (brick.position.0 - brick.size.0 as i32) as f32,
                y1: (brick.position.1 - brick.size.1 as i32) as f32,
                x2: (brick.position.0 + brick.size.0 as i32) as f32,
                y2: (brick.position.1 + brick.size.1 as i32) as f32
            };

            //log(&format!("{:?}, {:?}", brick.direction, brick.rotation));

            if draw_fills {
                // Calculate Shape vertices
                let verts = match name.as_str() {
                    "B_2x2_Corner" =>
                        corner(brick.direction, brick.rotation, &shape),
                    "PB_DefaultSideWedge" | "PB_DefaultSideWedgeTile" =>
                        side_wedge(brick.direction, brick.rotation, &shape),
                    "PB_DefaultWedge" =>
                        wedge(brick.direction, brick.rotation, &shape),
                    "PB_DefaultRamp" =>
                        ramp(brick.direction, brick.rotation, &shape),
                    "PB_DefaultRampCorner" =>
                        ramp_corner(brick.direction, brick.rotation, &shape),
                    "PB_DefaultRampCornerInverted" =>
                        ramp_corner_inverted(brick.direction, brick.rotation, &shape),
                    "PB_DefaultRampCrest" =>
                        ramp_crest(brick.direction, brick.rotation, &shape),
                    "PB_DefaultRampCrestEnd" =>
                        ramp_crest_end(brick.direction, brick.rotation, &shape),
                    "B_1x1F_Round" | "B_1x1_Round" | "B_2x2F_Round" | "B_2x2_Round" | "B_4x4_Round" =>
                        round(brick.direction, &shape),
                    _ => 
                        rec(&shape),
                };

                // Add shape to save
                let fill = RenderObject {
                    vertices: verts,
                    color: brick_color,
                };
                self.vertex_buffer.append(&mut fill.get_vertex_array());
            }
            
            if draw_ols {
                // Add brick outline for rendering
                let ol_verts = match name.as_str() {
                    "B_2x2_Corner" =>
                        corner_ol(brick.direction, brick.rotation, &shape),
                    "PB_DefaultSideWedge" | "PB_DefaultSideWedgeTile" =>
                        side_wedge_ol(brick.direction, brick.rotation, &shape),
                    "PB_DefaultWedge" =>
                        wedge_ol(brick.direction, brick.rotation, &shape),
                    "PB_DefaultRamp" =>
                        ramp_ol(brick.direction, brick.rotation, &shape),
                    "PB_DefaultRampCorner" =>
                        ramp_corner_ol(brick.direction, brick.rotation, &shape),
                    "PB_DefaultRampCornerInverted" =>
                        ramp_corner_inverted_ol(brick.direction, brick.rotation, &shape),
                    "PB_DefaultRampCrest" =>
                        ramp_crest_ol(brick.direction, brick.rotation, &shape),  
                    "PB_DefaultRampCrestEnd" =>
                        ramp_crest_end_ol(brick.direction, brick.rotation, &shape),
                    "B_1x1F_Round" | "B_1x1_Round" | "B_2x2F_Round" | "B_2x2_Round" | "B_4x4_Round" =>
                        round_ol(brick.direction, &shape),
                    _ =>
                        rec_ol(&shape)
                };

                let outline = RenderObject {
                    vertices: ol_verts,
                    color: Color::black()
                };
                self.vertex_buffer.append(&mut outline.get_vertex_array());
            }

        }

        self.gl_buffer_data(&self.vertex_buffer);
        Ok(())
    }

    #[wasm_bindgen(js_name = buildHeightmapVertexBuffer)]
    pub fn build_heightmap_vertex_buffer(&mut self) -> Result<(), JsValue> {
        self.clear_vertex_buffer();

        let mut min_height:i32 = std::i32::MAX;
        let mut max_height:i32 = std::i32::MIN;

        for brick in &self.bricks {
            let height = brick.position.2;
            min_height = std::cmp::min(min_height, height);
            max_height = std::cmp::max(max_height, height);
        }
       
        // Calculate shapes for rendering and save Centroid
        for brick in &self.bricks {
            if !brick.visibility {
                continue;
            }

            let name = &self.brick_assets[brick.asset_name_index as usize];

            // Add brick as shape for rendering
            let shape = Shape {
                x1: (brick.position.0 - brick.size.0 as i32) as f32,
                y1: (brick.position.1 - brick.size.1 as i32) as f32,
                x2: (brick.position.0 + brick.size.0 as i32) as f32,
                y2: (brick.position.1 + brick.size.1 as i32) as f32
            };

            //log(&format!("{:?}, {:?}", brick.direction, brick.rotation));
            // Calculate Shape vertices
            let verts = match name.as_str() {
                "B_2x2_Corner" =>
                    corner(brick.direction, brick.rotation, &shape),
                "PB_DefaultSideWedge" | "PB_DefaultSideWedgeTile" =>
                    side_wedge(brick.direction, brick.rotation, &shape),
                "PB_DefaultWedge" =>
                    wedge(brick.direction, brick.rotation, &shape),
                "PB_DefaultRamp" =>
                    ramp(brick.direction, brick.rotation, &shape),
                "PB_DefaultRampCorner" =>
                    ramp_corner(brick.direction, brick.rotation, &shape),
                "PB_DefaultRampCornerInverted" =>
                    ramp_corner_inverted(brick.direction, brick.rotation, &shape),
                "PB_DefaultRampCrest" =>
                    ramp_crest(brick.direction, brick.rotation, &shape),
                "PB_DefaultRampCrestEnd" =>
                    ramp_crest_end(brick.direction, brick.rotation, &shape),
                "B_1x1F_Round" | "B_1x1_Round" | "B_2x2F_Round" | "B_2x2_Round" | "B_4x4_Round" =>
                    round(brick.direction, &shape),
                _ => 
                    rec(&shape),
            };

            let height = brick.position.2;
            let relative_height = (height - min_height) as f32 / (max_height - min_height) as f32;

            // Add shape to save
            let mut vertex_array = Vec::new();
            let vertex_count = verts.len() / 2;
            for i in 0..vertex_count {
                vertex_array.push(verts[i*2]);
                vertex_array.push(verts[i*2 + 1]);
                vertex_array.push(relative_height);
                vertex_array.push(relative_height);
                vertex_array.push(relative_height);
            }
            self.vertex_buffer.append(&mut vertex_array);
        }
        
        self.gl_buffer_data(&self.vertex_buffer);
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

        let vertex_count = (self.vertex_buffer.len() / 5) as i32;

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
            let brick_shape = BrickShape {
                name_index: self.bricks[i].asset_name_index,
                position: (self.bricks[i].position.0, self.bricks[i].position.1),
                size: (self.bricks[i].size.0, self.bricks[i].size.1),
                rotation: self.bricks[i].rotation,
                direction: self.bricks[i].direction
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