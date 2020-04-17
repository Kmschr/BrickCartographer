use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use std::convert::TryInto;
use web_sys::{WebGlRenderingContext, WebGlUniformLocation};
use crate::{JsSave};
use crate::graphics::*;
use crate::log;

mod glsl;
mod m3;
mod brick_renders;

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

pub fn render(save: &JsSave, size: Point, pan: Point, scale: f32, show_outlines: bool) -> Result<(), JsValue> {
    let gl = &save.context;

    gl.viewport(0, 0, size.x as i32, size.y as i32);

    gl.clear_color(0.8, 0.8, 0.8, 1.0);
    gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

    let offset = Point {
        x: pan.x - save.center.x,
        y: pan.y - save.center.y
    };

    let mut matrix = m3::projection(size.x, size.y);
    matrix = m3::translate(matrix, size.x/2.0, size.y/2.0);
    matrix = m3::rotate(matrix, 0.0);
    matrix = m3::scale(matrix, scale, scale);
    matrix = m3::translate(matrix, offset.x, offset.y);

    gl.uniform_matrix3fv_with_f32_array(Some(save.u_matrix.as_ref()), false, &matrix);

    let _visible_area = Rect {
        x: offset.x,
        y: offset.y,
        width: size.x / scale,
        height: size.y / scale
    };

    for shape in &save.shapes {
        match shape.shape_type {
            ShapeType::Rect => {
                let mut vertex_array: [f32; 30] = [0.0; 30];
                for i in 0..6 {
                    vertex_array[i*5] = shape.vertices[i*2];
                    vertex_array[i*5 + 1] = shape.vertices[i*2 + 1];
                    vertex_array[i*5 + 2] = shape.color.r;
                    vertex_array[i*5 + 3] = shape.color.g;
                    vertex_array[i*5 + 4] = shape.color.b;
                }
                unsafe {
                    gl.buffer_data_with_array_buffer_view(
                        WebGlRenderingContext::ARRAY_BUFFER,
                        &js_sys::Float32Array::view(&vertex_array),
                        WebGlRenderingContext::DYNAMIC_DRAW
                    );
                }
                gl.draw_arrays(WebGlRenderingContext::TRIANGLES, 0, 6);
            },
            ShapeType::Tri => {
                let mut vertex_array: [f32; 15] = [0.0; 15];
                for i in 0..3 {
                    vertex_array[i*5] = shape.vertices[i*2];
                    vertex_array[i*5 + 1] = shape.vertices[i*2 + 1];
                    vertex_array[i*5 + 2] = shape.color.r;
                    vertex_array[i*5 + 3] = shape.color.g;
                    vertex_array[i*5 + 4] = shape.color.b;
                }
                unsafe {
                    gl.buffer_data_with_array_buffer_view(
                        WebGlRenderingContext::ARRAY_BUFFER,
                        &js_sys::Float32Array::view(&vertex_array),
                        WebGlRenderingContext::DYNAMIC_DRAW
                    );
                }
                gl.draw_arrays(WebGlRenderingContext::TRIANGLES, 0, 3);
            }
        };
    }
    
    Ok(())
}
