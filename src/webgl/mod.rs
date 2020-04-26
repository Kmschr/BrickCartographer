use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use std::convert::TryInto;
use web_sys::{WebGlRenderingContext, WebGlUniformLocation};

mod glsl;

//const MAX_BRICK_DISTANCE: i32 = 10 * 10000;

pub fn get_rendering_context() -> Result<(WebGlRenderingContext, WebGlUniformLocation), JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_elements_by_class_name("map-canvas").get_with_index(0).unwrap();
    let canvas: web_sys::HtmlCanvasElement = match canvas.dyn_into::<web_sys::HtmlCanvasElement>() {
        Ok(v) => v,
        Err(_e) => return Err(JsValue::from("Unable to find HtmlCanvasElement")),
    };

    let context = match canvas.get_context("webgl") {
        Ok(v) => v,
        Err(_e) => match canvas.get_context("webgl-experimental") {
            Ok(v) => v,
            Err(_e) => return Err(JsValue::from("No WebGl support by browser")),
        },
    };
        
    let gl = match context.unwrap().dyn_into::<WebGlRenderingContext>() {
        Ok(v) => v,
        Err(_e) => return Err(JsValue::from("Error transforming webgl context")),
    };

    let vert_shader = match glsl::compile_shader(
        &gl,
        WebGlRenderingContext::VERTEX_SHADER,
        glsl::VERTEX_SHADER_CODE,
    ) {
        Ok(v) => v,
        Err(_e) => return Err(JsValue::from("Error in vertex shader code"))
    };
    let frag_shader = match glsl::compile_shader(
        &gl,
        WebGlRenderingContext::FRAGMENT_SHADER,
        glsl::FRAGMENT_SHADER_CODE,
    ){
        Ok(v) => v,
        Err(_e) => return Err(JsValue::from("Error in fragment shader code"))
    };
    let program = match glsl::link_program(&gl, &vert_shader, &frag_shader) {
        Ok(v) => v,
        Err(_e) => return Err(JsValue::from("Error linking shaders"))
    };
    gl.use_program(Some(&program));

    let position_attribute_location: u32 = gl.get_attrib_location(&program, "a_position").try_into().unwrap();
    let color_attribute_location: u32 = gl.get_attrib_location(&program, "a_color").try_into().unwrap();
    let matrix_uniform_location = gl.get_uniform_location(&program, "u_matrix").unwrap();
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

    Ok((gl, matrix_uniform_location))
}
