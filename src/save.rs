use crate::log;
use crate::webgl;
use crate::color::*;
use crate::graphics::*;
use crate::bricks::*;

use std::collections::HashSet;
use brs::{HasHeader1, HasHeader2, Direction, Rotation};
use web_sys::{WebGlRenderingContext, WebGlUniformLocation};
use wasm_bindgen::prelude::*;

#[derive(PartialEq, Eq, Hash)]
pub struct BrickShape {
    name_index: u32,
    size: (u32, u32),
    position: (i32, i32),
    rotation: Rotation,
    direction: Direction,
}

#[wasm_bindgen]
pub struct JsSave {
    #[wasm_bindgen(skip)]
    pub reader: brs::read::ReaderAfterBricks,
    #[wasm_bindgen(skip)]
    pub unmodified_bricks: Vec<brs::Brick>,
    #[wasm_bindgen(skip)]
    pub bricks: Vec<brs::Brick>,
    #[wasm_bindgen(skip)]
    pub brick_assets: Vec<String>,
    #[wasm_bindgen(skip)]
    pub context: WebGlRenderingContext,
    #[wasm_bindgen(skip)]
    pub u_matrix: WebGlUniformLocation,
    #[wasm_bindgen(skip)]
    pub colors: Vec<Color>,
    #[wasm_bindgen(skip)]
    pub center: Point,
    #[wasm_bindgen(skip)]
    pub vertex_buffer: Vec<f32>,
}

#[wasm_bindgen]
impl JsSave {
    // Save info getters for frontend
    pub fn map(&self) -> String {
        self.reader.map().to_string()
    }
    pub fn description(&self) -> String {
        self.reader.description().to_string()
    }
    #[wasm_bindgen(js_name = brickCount)]
    pub fn brick_count(&self) -> i32 {
        self.reader.brick_count()
    }

    // Get rendering info needed from bricks
    #[wasm_bindgen(js_name = processBricks)]
    pub fn process_bricks(&mut self, draw_ols: bool, draw_fills: bool) -> Result<(), JsValue> {
        let mut compatible = true;

        // Reset brick transforms
        self.bricks = self.unmodified_bricks.clone();
        self.vertex_buffer = Vec::with_capacity(self.bricks.len() * 6 * 5);

        // Modify brick dimensions to reflect orientation transforms
        for brick in &mut self.bricks {
            if !brick.visibility {
                continue;
            }

            let name = &self.brick_assets[brick.asset_name_index as usize];
            // Check if save is incompatible, which can usually be determined by brick owner index being out of bounds
            let brick_owner_oob = brick.owner_index as usize > self.reader.brick_owners().len();
            if brick_owner_oob {
                compatible = false;
            }

            log(&format!("{:?}", brick));
            log(name);

            // Give size to non procedural bricks
            match name.as_str() {
                "B_2x2_Corner" => {
                    brick.size.0 = STUD_WIDTH as u32;
                    brick.size.1 = STUD_WIDTH as u32;
                    brick.size.2 = (STUD_HEIGHT/2.0) as u32;
                },
                "B_2x_Cube_Side" => {
                    brick.size.0 = STUD_WIDTH as u32;
                    brick.size.1 = STUD_WIDTH as u32;
                    brick.size.2 = STUD_HEIGHT as u32;
                },
                "B_1x1_Brick_Side" => {
                    brick.size.0 = (STUD_WIDTH/2.0) as u32;
                    brick.size.1 = (STUD_WIDTH/2.0) as u32;
                    brick.size.2 = (STUD_HEIGHT/2.0) as u32;
                },
                "B_1x4_Brick_Side" => {
                    brick.size.0 = (STUD_WIDTH*2.0) as u32;
                    brick.size.1 = (STUD_WIDTH/2.0) as u32;
                    brick.size.2 = (STUD_HEIGHT/2.0) as u32;
                },
                "B_1x2f_Plate_Center" => {
                    brick.size.0 = STUD_WIDTH as u32;
                    brick.size.1 = (STUD_WIDTH/2.0) as u32;
                    brick.size.2 = (STUD_HEIGHT/2.0) as u32;
                },
                "B_2x2f_Plate_Center" => {
                    brick.size.0 = STUD_WIDTH as u32;
                    brick.size.1 = STUD_WIDTH as u32;
                    brick.size.2 = (PLATE_HEIGHT/2.0) as u32;
                },
                "B_1x2f_Plate_Center_Inv" => {
                    brick.size.0 = STUD_WIDTH as u32;
                    brick.size.1 = (STUD_WIDTH/2.0) as u32;
                    brick.size.2 = (STUD_HEIGHT/2.0) as u32;
                },
                "B_2x2f_Plate_Center_Inv" => {
                    brick.size.0 = STUD_WIDTH as u32;
                    brick.size.1 = STUD_WIDTH as u32;
                    brick.size.2 = (PLATE_HEIGHT/2.0) as u32;
                },
                _ => ()
            }

            // Apply Rotation
            if brick.rotation == Rotation::Deg90 || brick.rotation == Rotation::Deg270 {
                std::mem::swap(&mut brick.size.0, &mut brick.size.1);
            }

            // Apply Direction
            if brick.direction == Direction::XPositive || brick.direction == Direction::XNegative {
                std::mem::swap(&mut brick.size.0, &mut brick.size.2);
            }
            else if brick.direction == Direction::YPositive || brick.direction == Direction::YNegative {
                std::mem::swap(&mut brick.size.0, &mut brick.size.1);
                std::mem::swap(&mut brick.size.1, &mut brick.size.2);
            }

            if brick.size.0 > (STUD_WIDTH * 200.0) as u32 || brick.size.1 > (STUD_WIDTH * 200.0) as u32 || brick.size.2 > (PLATE_HEIGHT * 500.0) as u32  || brick_owner_oob {
                brick.visibility = false;
            }
        }

        // Now that the bricks are oriented properly, sort by top surface height
        self.bricks.sort_unstable_by_key(|brick| brick.position.2 + brick.size.2 as i32);

        // Don't render bricks that are obviously hidden
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
        //log(&format!("Bricks Discarded: {}", copy_count));

        // Sums for calculating Centroid of save
        let mut area_sum = 0.0;
        let mut point_sum = Point {x:0.0, y:0.0};

        // Get color list as rgba 0.0-1.0 f32
        self.colors = self.reader.colors().iter().map(convert_color).collect();
        

        
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
                    _ =>
                        rec_ol(&shape)
                };

                let outline = RenderObject {
                    vertices: ol_verts,
                    color: Color::black()
                };
                self.vertex_buffer.append(&mut outline.get_vertex_array());
            }
            
            // Add to Centroid calculation sums
            let area = brick.size.0 * brick.size.1;
            point_sum.x += (brick.position.0 * area as i32) as f32;
            point_sum.y += (brick.position.1 * area as i32) as f32;
            area_sum += area as f32;
        }

        // Calculate Centroid
        self.center = Point {
            x: point_sum.x / area_sum,
            y: point_sum.y / area_sum,
        };

        unsafe {
            self.context.buffer_data_with_array_buffer_view(
                WebGlRenderingContext::ARRAY_BUFFER,
                &js_sys::Float32Array::view(&self.vertex_buffer),
                WebGlRenderingContext::STATIC_DRAW
            );
        }

        if !compatible {
            return Err(JsValue::from_str("Save version not compatible w/ brs-rs"));
        }
        Ok(())
    }

    pub fn render(&self, size_x: i32, size_y: i32, pan_x: f32, pan_y: f32, scale: f32) -> Result<(), JsValue> {
        let pan = Point { x: pan_x, y: pan_y};
        let size = Point { x: size_x as f32, y: size_y as f32};
        webgl::render(&self, size, pan, scale)
    }
}
