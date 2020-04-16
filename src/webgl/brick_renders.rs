use web_sys::{WebGlRenderingContext};
use crate::{Rect, Color};

const USAGE_PATTERN: u32 = WebGlRenderingContext::DYNAMIC_DRAW;

pub fn render_brick(gl: &WebGlRenderingContext, brick: &brs::Brick, name: &str, color: Color, show_outlines: bool) {
    if name.contains("Wedge") {
        render_pb_wedge(gl, brick, color, show_outlines);
    } else {
        render_pb(gl, brick, color, show_outlines);
    }
}

pub fn render_pb(gl: &WebGlRenderingContext, brick: &brs::Brick, color: Color, show_outlines: bool) {
    let brick_rect = get_brick_rect(brick);

    fill_rect(gl, &brick_rect, color);

    if show_outlines {
        outline_rect(gl, &brick_rect, Color::black());
    }
}

pub fn render_pb_wedge(gl: &WebGlRenderingContext, brick: &brs::Brick, color: Color, show_outlines: bool) {
    let brick_rect = get_brick_rect(brick);
    let verts: [f32;6];

    match brick.rotation {
        brs::Rotation::Deg0 => { // ORANGE TRIANGLE
            verts = [
                brick_rect.x, brick_rect.y, 
                brick_rect.x + brick_rect.width, brick_rect.y,
                brick_rect.x, brick_rect.y + brick_rect.height,
            ];
        },
        brs::Rotation::Deg90 => { // BLUE TRIANGLE
            verts = [
                brick_rect.x, brick_rect.y, 
                brick_rect.x + brick_rect.width, brick_rect.y,
                brick_rect.x + brick_rect.width, brick_rect.y + brick_rect.height,
            ];
        },
        brs::Rotation::Deg180 => { // GREEN TRIANGLE
            verts = [
                brick_rect.x, brick_rect.y + brick_rect.height, 
                brick_rect.x + brick_rect.width, brick_rect.y,
                brick_rect.x + brick_rect.width, brick_rect.y + brick_rect.height,
            ];
        },
        brs::Rotation::Deg270 => { // RED TRIANGLE
            verts = [
                brick_rect.x, brick_rect.y, 
                brick_rect.x, brick_rect.y + brick_rect.height,
                brick_rect.x + brick_rect.width, brick_rect.y + brick_rect.height,
            ];
        },
    }
    fill_tri(gl, verts, color);
    if show_outlines {
        outline_tri(gl, verts, Color::black());
    }
}

pub fn get_brick_rect(brick: &brs::Brick) -> Rect {
    let x: f32;
    let y: f32;
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

    Rect {x, y, width, height}
}

pub fn get_color(brick: &brs::Brick, colors: &[Color]) -> Color {
    let mut color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };
    match brick.color {
        brs::ColorMode::Set(color_index) => {
            color.r = colors[color_index as usize].r;
            color.g = colors[color_index as usize].g;
            color.b = colors[color_index as usize].b;
            color.a = colors[color_index as usize].a;

        },
        brs::ColorMode::Custom(c) => {
            color.r = c.r() as f32 / 255.0;
            color.g = c.g() as f32 / 255.0;
            color.b = c.b() as f32 / 255.0;
            color.a = c.a() as f32 / 255.0;
        },
    }
    color
}

fn outline_rect(gl: &WebGlRenderingContext, rect: &Rect, color: Color) {
    let x1 = rect.x;
    let x2 = rect.x + rect.width;
    let y1 = rect.y;
    let y2 = rect.y + rect.height;

    let dx = 0.2;
    let dy = 0.2;

    unsafe {
        gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &js_sys::Float32Array::view(&[
                // LEFT
                x1, y1,  color.r, color.g, color.b,
                x1, y2,  color.r, color.g, color.b,
                x1 + dx, y2,  color.r, color.g, color.b,
                x1, y1,  color.r, color.g, color.b,
                x1 + dx, y1,  color.r, color.g, color.b,
                x1 + dx, y2,  color.r, color.g, color.b,
                // TOP
                x1, y1,  color.r, color.g, color.b,
                x2, y1,  color.r, color.g, color.b,
                x2, y1 + dy,  color.r, color.g, color.b,
                x1, y1,  color.r, color.g, color.b,
                x1, y1 + dy,  color.r, color.g, color.b,
                x2, y1 + dy,  color.r, color.g, color.b,
                // RIGHT
                x2, y1,  color.r, color.g, color.b,
                x2, y2,  color.r, color.g, color.b,
                x2 - dx, y2,  color.r, color.g, color.b,
                x2, y1,  color.r, color.g, color.b,
                x2 - dx, y1,  color.r, color.g, color.b,
                x2 - dx, y2,  color.r, color.g, color.b,
                // BOTTOM
                x1, y2,  color.r, color.g, color.b,
                x2, y2,  color.r, color.g, color.b,
                x2, y2 - dy,  color.r, color.g, color.b,
                x1, y2,  color.r, color.g, color.b,
                x1, y2 - dy,  color.r, color.g, color.b,
                x2, y2 - dy,  color.r, color.g, color.b,
            ]),
            USAGE_PATTERN
        );
    }

    gl.draw_arrays(WebGlRenderingContext::TRIANGLES, 0, 24);
}

fn fill_rect(gl: &WebGlRenderingContext, rect: &Rect, color: Color) {
    let x1 = rect.x;
    let x2 = rect.x + rect.width;
    let y1 = rect.y;
    let y2 = rect.y + rect.height;

    unsafe {
        gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &js_sys::Float32Array::view(&[
                x1, y1,  color.r, color.g, color.b,
                x2, y1,  color.r, color.g, color.b,
                x1, y2,  color.r, color.g, color.b,
                x1, y2,  color.r, color.g, color.b,
                x2, y1,  color.r, color.g, color.b,
                x2, y2,  color.r, color.g, color.b,
            ]),
            USAGE_PATTERN
        );
    }

    gl.draw_arrays(WebGlRenderingContext::TRIANGLES, 0, 6);
}

fn outline_tri(gl: &WebGlRenderingContext, verts: [f32;6], color: Color) {
    let delta = 0.2;
    unsafe {
        gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &js_sys::Float32Array::view(&[
                // SIDE 1
                verts[0], verts[1],  color.r, color.g, color.b,
                verts[2], verts[3],  color.r, color.g, color.b,
                verts[0] + delta, verts[1] + delta,  color.r, color.g, color.b,
                verts[0] + delta, verts[1] + delta,  color.r, color.g, color.b,
                verts[2], verts[3],  color.r, color.g, color.b,
                verts[2] + delta, verts[3] + delta,  color.r, color.g, color.b,

                //SIDE 2
                verts[2], verts[3],  color.r, color.g, color.b,
                verts[4], verts[5],  color.r, color.g, color.b,
                verts[2] - delta, verts[3] - delta,  color.r, color.g, color.b,
                verts[2] - delta, verts[3] - delta,  color.r, color.g, color.b,
                verts[4], verts[5],  color.r, color.g, color.b,
                verts[4] - delta, verts[5] - delta,  color.r, color.g, color.b,

                // SIDE 3
                verts[0], verts[1],  color.r, color.g, color.b,
                verts[4], verts[5],  color.r, color.g, color.b,
                verts[0] + delta, verts[1] + delta,  color.r, color.g, color.b,
                verts[0] + delta, verts[1] + delta,  color.r, color.g, color.b,
                verts[4], verts[5],  color.r, color.g, color.b,
                verts[4] - delta, verts[5] - delta,  color.r, color.g, color.b,
            ]),
            USAGE_PATTERN
        );
    }
    gl.draw_arrays(WebGlRenderingContext::TRIANGLES, 0, 18);
}

fn fill_tri(gl: &WebGlRenderingContext, verts: [f32;6], color: Color) {
    unsafe {
        gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &js_sys::Float32Array::view(&[
                verts[0], verts[1],  color.r, color.g, color.b,
                verts[2], verts[3],  color.r, color.g, color.b,
                verts[4], verts[5],  color.r, color.g, color.b,
            ]),
            USAGE_PATTERN
        );
    }
    gl.draw_arrays(WebGlRenderingContext::TRIANGLES, 0, 3);
}