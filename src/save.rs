use crate::log;
use crate::webgl;
use crate::graphics::*;

use brs::{HasHeader1, HasHeader2, Direction, Rotation};
use web_sys::{WebGlRenderingContext, WebGlUniformLocation};
use wasm_bindgen::prelude::*;

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
    pub shapes: Vec<f32>,
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
    pub fn brick_count(&self) -> i32 {
        self.reader.brick_count()
    }

    // Get rendering info needed from bricks
    pub fn process_bricks(&mut self, draw_outlines: bool) -> Result<(), JsValue> {
        // Reset brick transforms
        self.bricks = self.unmodified_bricks.clone();
        self.shapes = Vec::new();

        // Modify brick dimensions to reflect orientation transforms
        for brick in &mut self.bricks {
            // Ignore bricks we don't know how to render yet (non-procedural)
            let name = &self.brick_assets[brick.asset_name_index as usize];
            if !brick.visibility || !name.starts_with('P') {
              continue;
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
        }

        // Now that the bricks are oriented properly, sort by top surface height
        self.bricks.sort_unstable_by_key(|brick| brick.position.2 + brick.size.2 as i32);

        // Sums for calculating Centroid of save
        let mut area_sum = 0.0;
        let mut point_sum = Point {x:0.0, y:0.0};

        // Get color list as rgba 0.0-1.0 f32
        self.colors = self.reader.colors().iter().map(convert_color).collect();
        
        // Calculate shapes for rendering and save Centroid
        for brick in &self.bricks {
            // Ignore bricks we don't know how to render yet (non-procedural)
            let name = &self.brick_assets[brick.asset_name_index as usize];
            if !brick.visibility || !name.starts_with('P') {
                continue;
            }

            // Check if save is incompatible, which can usually be determined by brick owner index being out of bounds
            let brick_owner_oob = brick.owner_index as usize > self.reader.brick_owners().len();
            if brick_owner_oob {
                return Err(JsValue::from_str("Save version not compatible w/ brs-rs"));
            }

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
            let x1: f32 = (brick.position.0 - brick.size.0 as i32) as f32;
            let y1: f32 = (brick.position.1 - brick.size.1 as i32) as f32;
            let x2: f32 = (brick.position.0 + brick.size.0 as i32) as f32;
            let y2: f32 = (brick.position.1 + brick.size.1 as i32) as f32;

            // Calculate Shape vertices
            let verts = match name.as_str() {
                "PB_DefaultBrick" => 
                    get_rect(x1, y1, x2, y2),
                "PB_DefaultSideWedge" | "PB_DefaultSideWedgeTile" =>
                    get_side_wedge(brick.direction, brick.rotation, x1, y1, x2, y2),
                "PB_DefaultWedge" =>
                    get_wedge(brick.direction, brick.rotation, x1, y1, x2, y2),
                "PB_DefaultRamp" =>
                    get_ramp(brick.direction, brick.rotation, x1, y1, x2, y2),
                _ => get_rect(x1, y1, x2, y2),
            };

            // Add shape to save
            let shape = Shape {
                vertices: verts,
                color: brick_color,
            };
            self.shapes.append(&mut shape.get_vertex_array());

            if draw_outlines {
                // Add brick outline for rendering
                let outline_verts = match name.as_str() {
                    "PB_DefaultBrick" =>
                        get_rect_outline(x1, y1, x2, y2),
                    _ =>
                        get_rect_outline(x1, y1, x2, y2)
                };

                let outline = Shape {
                    vertices: outline_verts,
                    color: Color::black()
                };
                self.shapes.append(&mut outline.get_vertex_array());
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

        Ok(())
    }

    pub fn render(&self, size_x: i32, size_y: i32, pan_x: f32, pan_y: f32, scale: f32) -> Result<(), JsValue> {
        let pan = Point { x: pan_x, y: pan_y};
        let size = Point { x: size_x as f32, y: size_y as f32};
        webgl::render(&self, size, pan, scale)
    }
}
