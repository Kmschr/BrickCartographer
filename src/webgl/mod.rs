use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use std::convert::TryInto;
use web_sys::{WebGlRenderingContext, WebGlUniformLocation};
use crate::error;

mod glsl;

// Bytes per vertex: x (f32), y (f32), rgba (4 x u8)
pub const VERTEX_STRIDE:i32 = 12;

pub fn get_rendering_context() -> Result<(WebGlRenderingContext, WebGlUniformLocation), JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = match document.get_elements_by_class_name("map-canvas").get_with_index(0) {
        Some(v) => v,
        None => {
            error("RUST ERROR: Unable to find element with map-canvas class");
            return Err(JsValue::from("Unable to find element with map-canvas class"))
        }
    };
    let canvas: web_sys::HtmlCanvasElement = match canvas.dyn_into::<web_sys::HtmlCanvasElement>() {
        Ok(v) => v,
        Err(_e) => {
            error("RUST ERROR: Unable to find HtmlCanvasElement");
            return Err(JsValue::from("Unable to find HtmlCanvasElement"))
        }
    };

    // No depth testing or stenciling is done, so skip allocating those buffers.
    // Antialiasing is disabled to avoid the multisampled backbuffer's VRAM cost.
    let context_options = js_sys::Object::new();
    js_sys::Reflect::set(&context_options, &"depth".into(), &false.into())?;
    js_sys::Reflect::set(&context_options, &"stencil".into(), &false.into())?;
    js_sys::Reflect::set(&context_options, &"antialias".into(), &false.into())?;

    let context = match canvas.get_context_with_context_options("webgl", &context_options) {
        Ok(v) => v,
        Err(_e) => match canvas.get_context_with_context_options("experimental-webgl", &context_options) {
            Ok(v) => v,
            Err(_e) => {
                error("RUST ERROR: No WebGl support by browser");
                return Err(JsValue::from("No WebGl support by browser"))
            }
        }
    };
        
    let gl = match context.unwrap().dyn_into::<WebGlRenderingContext>() {
        Ok(v) => v,
        Err(_e) => {
            error("RUST ERROR: Error transforming webgl context");
            return Err(JsValue::from("Error transforming webgl context"))
        }
    };

    let vert_shader = match glsl::compile_shader(
        &gl,
        WebGlRenderingContext::VERTEX_SHADER,
        glsl::VERTEX_SHADER_CODE,
    ) {
        Ok(v) => v,
        Err(_e) => {
            error("RUST ERROR: Error in vertex shader code");
            return Err(JsValue::from("Error in vertex shader code"))
        }
    };
    let frag_shader = match glsl::compile_shader(
        &gl,
        WebGlRenderingContext::FRAGMENT_SHADER,
        glsl::FRAGMENT_SHADER_CODE,
    ){
        Ok(v) => v,
        Err(_e) => {
            error("RUST ERROR: Error in fragment shader code");
            return Err(JsValue::from("Error in fragment shader code"))
        }
    };
    let program = match glsl::link_program(&gl, &vert_shader, &frag_shader) {
        Ok(v) => v,
        Err(_e) => {
            error("RUST ERROR: Error linking shaders");
            return Err(JsValue::from("Error linking shaders"))
        }
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
        VERTEX_STRIDE, // Size of individual vertex
        0 // Offset from beginning of a vertex to this attribute
    );
    gl.vertex_attrib_pointer_with_i32(
        color_attribute_location,
        4, // Number of elements per attribute
        WebGlRenderingContext::UNSIGNED_BYTE,
        true, // Normalize 0-255 to 0.0-1.0
        VERTEX_STRIDE, // Size of individual vertex
        2 * std::mem::size_of::<f32>() as i32   // Offset from beginning of a vertex to this attribute
    );

    gl.enable_vertex_attrib_array(position_attribute_location);
    gl.enable_vertex_attrib_array(color_attribute_location);

    Ok((gl, matrix_uniform_location))
}
