use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use std::convert::TryInto;
use web_sys::WebGlRenderingContext;
use crate::log;
use crate::{JsSave, Point, Rect, Color};

mod glsl;
mod m3;
mod brick_renders;

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

pub fn render(save: &JsSave, size: Point, pan: Point, scale: f32, show_outlines: bool) -> Result<(), JsValue> {
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
    let color_attribute_location: u32 = gl.get_attrib_location(&program, "a_color").try_into().unwrap();
    let matrix_uniform_location = gl.get_uniform_location(&program, "u_matrix");
    //let color_uniform_location = gl.get_uniform_location(&program, "u_color");
    let vertex_buffer = gl.create_buffer().ok_or("failed to create buffer")?;

    gl.viewport(0, 0, size.x as i32, size.y as i32);

    gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));

    gl.clear_color(0.7, 0.7, 0.7, 1.0);
    gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

    gl.use_program(Some(&program));

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

    let offset = Point {
        x: pan.x - save.center.x,
        y: pan.y - save.center.y
    };

    let mut matrix = m3::projection(size.x, size.y);
    matrix = m3::translate(matrix, size.x/2.0, size.y/2.0);
    matrix = m3::scale(matrix, scale, scale);
    matrix = m3::translate(matrix, offset.x, offset.y);

    gl.uniform_matrix3fv_with_f32_array(matrix_uniform_location.as_ref(), false, &matrix);

    let visible_area = Rect {
        x: offset.x,
        y: offset.y,
        width: size.x / scale,
        height: size.y / scale
    };

    for brick in &save.bricks {

        

        let name = &save.brick_assets[brick.asset_name_index as usize];

        if !brick.visibility || !name.starts_with('P') {
            continue;
        }

        let color = brick_renders::get_color(brick, &save.colors);
        brick_renders::render_brick(gl, brick, name, color, show_outlines);
    }
    
    Ok(())
}
