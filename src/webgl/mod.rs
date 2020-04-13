use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use std::convert::TryInto;
use web_sys::WebGlRenderingContext;
use crate::log;
use crate::JsSave;
use crate::graphics::Color;

mod glsl;
mod m3;

const CANVAS_WIDTH: f32 = 825.0;
const CANVAS_HEIGHT: f32 = 600.0;

const MAX_BRICK_DISTANCE: i32 = 10 * 10000;

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

pub fn render(save: &JsSave, colors: &[Color], pan: &[f32], scale: f32) -> Result<(), JsValue> {
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

    gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&position_buffer));

    gl.clear_color(0.7, 0.7, 0.7, 1.0);
    gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

    gl.use_program(Some(&program));

    gl.enable_vertex_attrib_array(position_attribute_location);

    gl.vertex_attrib_pointer_with_i32(position_attribute_location, 2, WebGlRenderingContext::FLOAT, false, 0, 0);

    let matrix = m3::projection(CANVAS_WIDTH, CANVAS_HEIGHT);
    let matrix = m3::translate(matrix, CANVAS_WIDTH/2.0, CANVAS_HEIGHT/2.0);
    let matrix = m3::rotate(matrix, 0.0);
    let matrix = m3::scale(matrix, scale, scale);
    let matrix = m3::translate(matrix, -save.center.x, -save.center.y);
    let matrix = m3::translate(matrix, pan[0], pan[1]);

    gl.uniform_matrix3fv_with_f32_array(matrix_uniform_location.as_ref(), false, &matrix);

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
                x += (brick.position.0 - brick.size.0 as i32) as f32;
                y += (brick.position.1 - brick.size.1 as i32) as f32;
                width = brick.size.0 as f32 * 2.0;
                height = brick.size.1 as f32 * 2.0;
            },
            brs::Rotation::Deg90 => {
                x += (brick.position.0 - brick.size.1 as i32) as f32;
                y += (brick.position.1 - brick.size.0 as i32) as f32;
                width = brick.size.1 as f32 * 2.0;
                height = brick.size.0 as f32 * 2.0;
            },
            brs::Rotation::Deg180 => {
                x += (brick.position.0 - brick.size.0 as i32) as f32;
                y += (brick.position.1 - brick.size.1 as i32) as f32;
                width = brick.size.0 as f32 * 2.0;
                height = brick.size.1 as f32 * 2.0;
            },
            brs::Rotation::Deg270 => {
                x += (brick.position.0 - brick.size.1 as i32) as f32;
                y += (brick.position.1 - brick.size.0 as i32) as f32;
                width = brick.size.1 as f32 * 2.0;
                height = brick.size.0 as f32 * 2.0;
            },
        }

        {
            let r: f32;
            let g: f32;
            let b: f32;
            let a: f32;
            match brick.color {
                brs::ColorMode::Set(color_index) => {
                    r = colors[color_index as usize].r;
                    g = colors[color_index as usize].g;
                    b = colors[color_index as usize].b;
                    a = colors[color_index as usize].a;
                },
                brs::ColorMode::Custom(color) => {
                    r = color.r() as f32 / 255.0;
                    g = color.g() as f32 / 255.0;
                    b = color.b() as f32 / 255.0;
                    a = color.a() as f32 / 255.0;
                },
            }
    
            set_rect(gl, x, y, width, height);

            //log(&format!("{},{}  {},{},{}", x, y, r, g, b));
            gl.uniform4f(color_uniform_location.as_ref(), r, g, b, a);
    
            gl.draw_arrays(
                WebGlRenderingContext::TRIANGLES,
                0,
                6,
            );
        }
    }
    
    Ok(())
}

fn set_rect(gl: &WebGlRenderingContext, x: f32, y: f32, width: f32, height: f32) {
    let x1 = x;
    let x2 = x + width;
    let y1 = y;
    let y2 = y + height;

    unsafe {
        gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &js_sys::Float32Array::view(&[
                x1, y1,
                x2, y1,
                x1, y2,
                x1, y2,
                x2, y1,
                x2, y2
            ]),
            WebGlRenderingContext::STATIC_DRAW
        );
    }
}
