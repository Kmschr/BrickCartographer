/*
use web_sys::{WebGlRenderingContext};
use crate::{Rect, Color};

const USAGE_PATTERN: u32 = WebGlRenderingContext::DYNAMIC_DRAW;

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
*/