use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use std::convert::TryInto;
use web_sys::{WebGlRenderingContext, WebGlUniformLocation};
use crate::{JsSave};
use crate::graphics::*;
use crate::log;

mod glsl;
mod m3;

//const MAX_BRICK_DISTANCE: i32 = 10 * 10000;

pub fn get_rendering_context() -> (Option<WebGlRenderingContext>, Option<WebGlUniformLocation>) {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_elements_by_class_name("map-canvas").get_with_index(0).unwrap();
    let canvas: web_sys::HtmlCanvasElement = match canvas.dyn_into::<web_sys::HtmlCanvasElement>() {
        Ok(v) => v,
        Err(_e) => return (None, None),
    };

    let context = match canvas.get_context("webgl") {
        Ok(v) => v,
        Err(_e) => match canvas.get_context("webgl-experimental") {
            Ok(v) => v,
            Err(_e) => return (None, None),
        },
    };
        
    let gl = match context.unwrap().dyn_into::<WebGlRenderingContext>() {
        Ok(v) => v,
        Err(_e) => return (None, None),
    };

    let vert_shader = match glsl::compile_shader(
        &gl,
        WebGlRenderingContext::VERTEX_SHADER,
        glsl::VERTEX_SHADER_CODE,
    ) {
        Ok(v) => v,
        Err(_e) => {
            log("Error in Vertex Shader");
            return (None, None)
        }
    };
    let frag_shader = match glsl::compile_shader(
        &gl,
        WebGlRenderingContext::FRAGMENT_SHADER,
        glsl::FRAGMENT_SHADER_CODE,
    ){
        Ok(v) => v,
        Err(_e) => {
            log("Error in Fragment Shader");
            return (None, None)
        }
    };
    let program = match glsl::link_program(&gl, &vert_shader, &frag_shader) {
        Ok(v) => v,
        Err(_e) => {
            log("Error linking shaders");
            return (None, None)
        }
    };
    gl.use_program(Some(&program));

    let position_attribute_location: u32 = gl.get_attrib_location(&program, "a_position").try_into().unwrap();
    let color_attribute_location: u32 = gl.get_attrib_location(&program, "a_color").try_into().unwrap();
    let matrix_uniform_location = gl.get_uniform_location(&program, "u_matrix");
    let vertex_buffer = gl.create_buffer().ok_or("failed to create buffer").unwrap();

    gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));

    gl.vertex_attrib_pointer_with_i32(
        position_attribute_location, 
        2, // Number of elements per attribute
        WebGlRenderingContext::FLOAT, 
        false, 
        5 * std::mem::size_of::<f32>() as i32,  // Size of individual vertex
        0 // Offset from beginning of a vertex to this attribute
    );
    gl.vertex_attrib_pointer_with_i32(
        color_attribute_location,
        3, // Number of elements per attribute
        WebGlRenderingContext::FLOAT,
        false,
        5 * std::mem::size_of::<f32>() as i32,  // Size of individual vertex
        2 * std::mem::size_of::<f32>() as i32   // Offset from beginning of a vertex to this attribute
    );

    gl.enable_vertex_attrib_array(position_attribute_location);
    gl.enable_vertex_attrib_array(color_attribute_location);

    (Some(gl), matrix_uniform_location)
}

pub fn render(save: &JsSave, size: Point, pan: Point, scale: f32) -> Result<(), JsValue> {
    let gl = &save.context;

    gl.viewport(0, 0, size.x as i32, size.y as i32);

    gl.clear_color(0.8, 0.8, 0.8, 1.0);
    gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

    let mut matrix = m3::projection(size.x, size.y);
    matrix = m3::translate(matrix, size.x/2.0, size.y/2.0);
    matrix = m3::scale(matrix, scale, scale);
    matrix = m3::translate(matrix, pan.x, pan.y);
    matrix = m3::rotate(matrix, 0.0);
    matrix = m3::translate(matrix, -save.center.x, -save.center.y);

    gl.uniform_matrix3fv_with_f32_array(Some(save.u_matrix.as_ref()), false, &matrix);

    let vertex_array = &save.vertex_buffer;
    let vertex_count = (vertex_array.len() / 5) as i32;

    if vertex_count > 0 {
        gl.draw_arrays(WebGlRenderingContext::TRIANGLES, 0, vertex_count);
    }
    
    Ok(())
}
