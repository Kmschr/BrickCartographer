use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use std::convert::TryInto;
use web_sys::WebGlRenderingContext;
use crate::log;
use crate::{JsSave, Point, Rect, Color};

mod glsl;
mod m3;

//const MAX_BRICK_DISTANCE: i32 = 10 * 10000;

pub fn get_rendering_context() -> Option<WebGlRenderingContext> {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_elements_by_class_name("map-canvas").get_with_index(0).unwrap();
    let canvas: web_sys::HtmlCanvasElement = match canvas.dyn_into::<web_sys::HtmlCanvasElement>() {
        Ok(v) => v,
        Err(_e) => return None,
    };

    let context = match canvas.get_context("webgl") {
        Ok(v) => v,
        Err(_e) => match canvas.get_context("webgl-experimental") {
            Ok(v) => v,
            Err(_e) => return None,
        },
    };
        
    match context.unwrap().dyn_into::<WebGlRenderingContext>() {
        Ok(v) => Some(v),
        Err(_e) => None,
    }
}

pub fn render(save: &JsSave, size: Point<i32>, pan: Point<f32>, scale: f32) -> Result<(), JsValue> {
    let gl = &save.context;
    let vert_shader = glsl::compile_shader(
        &gl,
        WebGlRenderingContext::VERTEX_SHADER,
        glsl::VERTEX_SHADER_CODE,
    )?;
    let frag_shader = glsl::compile_shader(
        &gl,
        WebGlRenderingContext::FRAGMENT_SHADER,
        glsl::FRAGMENT_SHADER_CODE,
    )?;
    let program = glsl::link_program(&gl, &vert_shader, &frag_shader)?;

    let position_attribute_location: u32 = gl.get_attrib_location(&program, "a_position").try_into().unwrap();
    let matrix_uniform_location = gl.get_uniform_location(&program, "u_matrix");
    let color_uniform_location = gl.get_uniform_location(&program, "u_color");
    let position_buffer = gl.create_buffer().ok_or("failed to create buffer")?;

    gl.viewport(0, 0, size.x, size.y);

    gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&position_buffer));

    gl.clear_color(0.7, 0.7, 0.7, 1.0);
    gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

    gl.use_program(Some(&program));

    gl.enable_vertex_attrib_array(position_attribute_location);

    gl.vertex_attrib_pointer_with_i32(position_attribute_location, 2, WebGlRenderingContext::FLOAT, false, 0, 0);

    let canvas_width = size.x as f32;
    let canvas_height =size.y as f32;

    let mut matrix = m3::projection(canvas_width, canvas_height);
    matrix = m3::translate(matrix, canvas_width/2.0, canvas_height/2.0);
    matrix = m3::scale(matrix, scale, scale);
    matrix = m3::translate(matrix, pan.x, pan.y);
    matrix = m3::translate(matrix, -save.center.x, -save.center.y);
    //matrix = m3::translate(matrix, cur_offset.x, cur_offset.y);
    //matrix = m3::translate(matrix, offset.x, offset.y);

    gl.uniform_matrix3fv_with_f32_array(matrix_uniform_location.as_ref(), false, &matrix);

    gl.line_width(2.0);

    for brick in &save.bricks {
        let name = &save.brick_assets[brick.asset_name_index as usize];

        if !brick.visibility || !name.starts_with('P') {
            continue;
        }

        let mut x: f32 = 0.0;
        let mut y: f32 = 0.0;
        let width: f32;
        let height: f32;

        match brick.rotation {
            brs::Rotation::Deg0 => {
                x = (brick.position.0 - brick.size.0 as i32) as f32;
                y = (brick.position.1 - brick.size.1 as i32) as f32;
                width = brick.size.0 as f32 * 2.0;
                height = brick.size.1 as f32 * 2.0;
            },
            brs::Rotation::Deg90 => {
                x = (brick.position.0 - brick.size.1 as i32) as f32;
                y = (brick.position.1 - brick.size.0 as i32) as f32;
                width = brick.size.1 as f32 * 2.0;
                height = brick.size.0 as f32 * 2.0;
            },
            brs::Rotation::Deg180 => {
                x = (brick.position.0 - brick.size.0 as i32) as f32;
                y = (brick.position.1 - brick.size.1 as i32) as f32;
                width = brick.size.0 as f32 * 2.0;
                height = brick.size.1 as f32 * 2.0;
            },
            brs::Rotation::Deg270 => {
                x = (brick.position.0 - brick.size.1 as i32) as f32;
                y = (brick.position.1 - brick.size.0 as i32) as f32;
                width = brick.size.1 as f32 * 2.0;
                height = brick.size.0 as f32 * 2.0;
            },
        }

        {
            let mut color = Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            };
            match brick.color {
                brs::ColorMode::Set(color_index) => {
                    color.r = save.colors[color_index as usize].r;
                    color.g = save.colors[color_index as usize].g;
                    color.b = save.colors[color_index as usize].b;
                    color.a = save.colors[color_index as usize].a;
                },
                brs::ColorMode::Custom(c) => {
                    color.r = c.r() as f32 / 255.0;
                    color.g = c.g() as f32 / 255.0;
                    color.b = c.b() as f32 / 255.0;
                    color.a = c.a() as f32 / 255.0;
                },
            }

            //log(&format!("{},{}  {},{},{}", x, y, r, g, b));
            gl.uniform4f(color_uniform_location.as_ref(), color.r, color.g, color.b, color.a);
    
            if name.contains("Wedge") {
                match brick.rotation {
                    brs::Rotation::Deg0 => {
                        set_tri(gl, [
                            x, y, 
                            x + width, y,
                            x, y + height,
                        ])
                    },
                    brs::Rotation::Deg90 => {
                        set_tri(gl, [
                            x, y, 
                            x + width, y,
                            x + width, y + height,
                        ])
                    },
                    brs::Rotation::Deg180 => {
                        set_tri(gl, [
                            x, y + height, 
                            x + width, y,
                            x + width, y + height,
                        ])
                    },
                    brs::Rotation::Deg270 => {
                        set_tri(gl, [
                            x, y, 
                            x, y + height,
                            x + width, y + height,
                        ])
                    },
                }
                gl.draw_arrays(
                    WebGlRenderingContext::TRIANGLES,
                    0,
                    3,
                );
                //gl.uniform4f(color_uniform_location.as_ref(), 0.0, 0.0, 0.0, 0.0);
                gl.draw_arrays(
                    WebGlRenderingContext::LINES,
                    0,
                    3,
                );
            } else {
                set_rect(gl, Rect::<f32> {x, y, width, height}, color);
                gl.draw_arrays(
                    WebGlRenderingContext::TRIANGLES,
                    0,
                    6,
                );
                //gl.uniform4f(color_uniform_location.as_ref(), 0.0, 0.0, 0.0, 0.0);
                gl.draw_arrays(
                    WebGlRenderingContext::LINES,
                    0,
                    6,
                );
            }
           
        }
    }
    
    Ok(())
}

fn set_rect(gl: &WebGlRenderingContext, rect: Rect<f32>, color: Color) {
    let x1 = rect.x;
    let x2 = rect.x + rect.width;
    let y1 = rect.y;
    let y2 = rect.y + rect.height;

    unsafe {
        gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &js_sys::Float32Array::view(&[
                x1, y1,
                x2, y1,
                x1, y2,
                x1, y2,
                x2, y1,
                x2, y2,
            ]),
            WebGlRenderingContext::STATIC_DRAW
        );
    }
}

fn set_tri(gl: &WebGlRenderingContext, verts: [f32;6]) {
    unsafe {
        gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &js_sys::Float32Array::view(&verts),
            WebGlRenderingContext::STATIC_DRAW
        );
    }
}
