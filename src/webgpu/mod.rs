use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use js_sys::{JsOption, Number, Uint8Array};
use web_sys::{
    gpu_buffer_usage, gpu_map_mode, gpu_texture_usage, GpuAutoLayoutMode, GpuBindGroup,
    GpuBindGroupDescriptor, GpuBindGroupEntry, GpuBuffer, GpuBufferDescriptor,
    GpuCanvasAlphaMode, GpuCanvasConfiguration, GpuCanvasContext, GpuColorTargetState,
    GpuDevice, GpuFragmentState, GpuIndexFormat, GpuLoadOp, GpuQueue,
    GpuRenderPassColorAttachment, GpuRenderPassDescriptor, GpuRenderPipeline,
    GpuRenderPipelineDescriptor, GpuShaderModuleDescriptor, GpuStoreOp,
    GpuTexelCopyBufferInfo, GpuTexelCopyTextureInfo, GpuTexture, GpuTextureDescriptor,
    GpuTextureFormat, GpuVertexAttribute, GpuVertexBufferLayout, GpuVertexFormat,
    GpuVertexState,
};
use crate::error;

// Bytes per vertex: x (f32), y (f32), rgba (4 x u8)
pub const VERTEX_STRIDE: i32 = 12;

// Matches the WebGL default of antialias: true (typically 4x MSAA)
const MSAA_SAMPLE_COUNT: u32 = 4;

// mat3x3<f32> in uniform space: 3 columns, each padded to 16 bytes
const UNIFORM_BUFFER_SIZE: u32 = 48;

// Texture-to-buffer copies require bytesPerRow aligned to 256
const BYTES_PER_ROW_ALIGNMENT: u32 = 256;

const SHADER_CODE: &str = r#"
    @group(0) @binding(0) var<uniform> u_matrix: mat3x3<f32>;

    struct VertexOutput {
        @builtin(position) position: vec4<f32>,
        @location(0) color: vec4<f32>,
    };

    @vertex
    fn vs_main(@location(0) position: vec2<f32>, @location(1) color: vec4<f32>) -> VertexOutput {
        var out: VertexOutput;
        out.position = vec4<f32>((u_matrix * vec3<f32>(position, 1.0)).xy, 0.0, 1.0);
        out.color = color;
        return out;
    }

    @fragment
    fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
        return in.color;
    }
"#;

pub struct BufferChunk {
    pub vertex_buffer: GpuBuffer,
    pub index_buffer: GpuBuffer,
    pub index_count: u32,
}

pub struct GpuContext {
    device: GpuDevice,
    queue: GpuQueue,
    canvas_context: GpuCanvasContext,
    pipeline: GpuRenderPipeline,
    uniform_buffer: GpuBuffer,
    bind_group: GpuBindGroup,
    format: GpuTextureFormat,
    // MSAA color target matching the canvas size, recreated on resize
    msaa_texture: Option<GpuTexture>,
}

fn js_err(msg: &str) -> JsValue {
    error(&format!("RUST ERROR: {}", msg));
    JsValue::from(msg)
}

pub async fn get_rendering_context() -> Result<GpuContext, JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let gpu = window.navigator().gpu();
    if gpu.is_undefined() {
        return Err(js_err("No WebGPU support by browser"));
    }

    let adapter = JsFuture::from(gpu.request_adapter().unchecked_into::<js_sys::Promise>())
        .await
        .map_err(|_| js_err("Error requesting WebGPU adapter"))?;
    if adapter.is_null() || adapter.is_undefined() {
        return Err(js_err("No WebGPU adapter available"));
    }
    let adapter: web_sys::GpuAdapter = adapter.unchecked_into();

    let device = JsFuture::from(adapter.request_device().unchecked_into::<js_sys::Promise>())
        .await
        .map_err(|_| js_err("Error requesting WebGPU device"))?;
    let device: GpuDevice = device.unchecked_into();
    let queue = device.queue();

    let canvas = document
        .get_elements_by_class_name("map-canvas")
        .get_with_index(0)
        .ok_or_else(|| js_err("Unable to find element with map-canvas class"))?;
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into()
        .map_err(|_| js_err("Unable to find HtmlCanvasElement"))?;

    let canvas_context = canvas
        .get_context("webgpu")
        .map_err(|_| js_err("Error getting webgpu canvas context"))?
        .ok_or_else(|| js_err("No webgpu canvas context available"))?
        .dyn_into::<GpuCanvasContext>()
        .map_err(|_| js_err("Error transforming webgpu context"))?;

    let format = gpu.get_preferred_canvas_format();
    let config = GpuCanvasConfiguration::new(&device, format);
    config.set_alpha_mode(GpuCanvasAlphaMode::Premultiplied);
    canvas_context
        .configure(&config)
        .map_err(|_| js_err("Error configuring webgpu canvas context"))?;

    let module = device.create_shader_module(&GpuShaderModuleDescriptor::new(SHADER_CODE));

    let position_attribute = GpuVertexAttribute::new(GpuVertexFormat::Float32x2, 0, 0);
    let color_attribute = GpuVertexAttribute::new(GpuVertexFormat::Unorm8x4, 8, 1);
    let buffer_layout = GpuVertexBufferLayout::new(
        VERTEX_STRIDE as u32,
        &[position_attribute, color_attribute],
    );

    let vertex = GpuVertexState::new(&module);
    vertex.set_entry_point("vs_main");
    vertex.set_buffers(&[JsOption::wrap(buffer_layout)]);

    let target = GpuColorTargetState::new(format);
    let fragment = GpuFragmentState::new(&module, &[JsOption::wrap(target)]);
    fragment.set_entry_point("fs_main");

    let multisample = web_sys::GpuMultisampleState::new();
    multisample.set_count(MSAA_SAMPLE_COUNT);

    let descriptor =
        GpuRenderPipelineDescriptor::new_with_gpu_auto_layout_mode(GpuAutoLayoutMode::Auto, &vertex);
    descriptor.set_fragment(&fragment);
    descriptor.set_multisample(&multisample);

    let pipeline = device
        .create_render_pipeline(&descriptor)
        .map_err(|_| js_err("Error creating render pipeline"))?;

    let uniform_buffer = device
        .create_buffer(&GpuBufferDescriptor::new(
            UNIFORM_BUFFER_SIZE,
            gpu_buffer_usage::UNIFORM | gpu_buffer_usage::COPY_DST,
        ))
        .map_err(|_| js_err("Error creating uniform buffer"))?;

    let entry = GpuBindGroupEntry::new_with_gpu_buffer(0, &uniform_buffer);
    let bind_group = device.create_bind_group(&GpuBindGroupDescriptor::new(
        &[entry],
        &pipeline.get_bind_group_layout(0),
    ));

    Ok(GpuContext {
        device,
        queue,
        canvas_context,
        pipeline,
        uniform_buffer,
        bind_group,
        format,
        msaa_texture: None,
    })
}

impl GpuContext {
    // Uploads static geometry via a buffer mapped at creation (single copy).
    // Size is padded to the required multiple of 4 (stride 12 and u32 indices
    // are already multiples, but don't rely on it).
    pub fn create_static_buffer(&self, data: &[u8], usage: u32) -> Result<GpuBuffer, JsValue> {
        let padded_size = (data.len() as u32 + 3) & !3;
        let descriptor = GpuBufferDescriptor::new(padded_size, usage);
        descriptor.set_mapped_at_creation(true);
        let buffer = self
            .device
            .create_buffer(&descriptor)
            .map_err(|_| js_err("Error creating GPU buffer"))?;

        let mapped = buffer
            .get_mapped_range()
            .map_err(|_| js_err("Error mapping GPU buffer"))?;
        unsafe {
            Uint8Array::new(&mapped).set(&Uint8Array::view(data), 0);
        }
        buffer.unmap();

        Ok(buffer)
    }

    fn write_uniform(&self, matrix: &[f32; 9]) {
        // Expand column-major 3x3 to std140-style layout: vec3 columns padded to 16 bytes
        let mut padded = [0f32; 12];
        for col in 0..3 {
            padded[col * 4..col * 4 + 3].copy_from_slice(&matrix[col * 3..col * 3 + 3]);
        }
        let mut bytes = [0u8; UNIFORM_BUFFER_SIZE as usize];
        for (i, v) in padded.iter().enumerate() {
            bytes[i * 4..i * 4 + 4].copy_from_slice(&v.to_le_bytes());
        }
        let _ = self
            .queue
            .write_buffer_with_u32_and_u8_slice(&self.uniform_buffer, 0, &bytes);
    }

    fn create_render_texture(
        &self,
        width: u32,
        height: u32,
        sample_count: u32,
        usage: u32,
    ) -> Result<GpuTexture, JsValue> {
        let descriptor = GpuTextureDescriptor::new(
            self.format,
            &[Number::from(width), Number::from(height)],
            usage,
        );
        descriptor.set_sample_count(sample_count);
        self.device
            .create_texture(&descriptor)
            .map_err(|_| js_err("Error creating render texture"))
    }

    // Encodes an MSAA render pass of all chunks resolving into target_view.
    // The uniform write is queued here; callers submit the encoder.
    fn encode_render_pass(
        &self,
        encoder: &web_sys::GpuCommandEncoder,
        msaa_texture: &GpuTexture,
        target_view: &web_sys::GpuTextureView,
        matrix: &[f32; 9],
        chunks: &[BufferChunk],
    ) -> Result<(), JsValue> {
        self.write_uniform(matrix);

        let msaa_view = msaa_texture
            .create_view()
            .map_err(|_| js_err("Error creating texture view"))?;

        let attachment = GpuRenderPassColorAttachment::new_with_gpu_texture_view(
            GpuLoadOp::Clear,
            GpuStoreOp::Discard,
            &msaa_view,
        );
        attachment.set_clear_value(&[0f64.into(), 0f64.into(), 0f64.into(), 0f64.into()]);
        attachment.set_resolve_target_gpu_texture_view(target_view);

        let pass_descriptor = GpuRenderPassDescriptor::new(&[JsOption::wrap(attachment)]);
        let pass = encoder
            .begin_render_pass(&pass_descriptor)
            .map_err(|_| js_err("Error beginning render pass"))?;

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, Some(&self.bind_group));
        for chunk in chunks {
            pass.set_vertex_buffer(0, Some(&chunk.vertex_buffer));
            pass.set_index_buffer(&chunk.index_buffer, GpuIndexFormat::Uint32);
            pass.draw_indexed(chunk.index_count);
        }
        pass.end();

        Ok(())
    }

    // Draws to the canvas through the cached MSAA target
    pub fn render_to_canvas(
        &mut self,
        matrix: &[f32; 9],
        chunks: &[BufferChunk],
    ) -> Result<(), JsValue> {
        let canvas_texture = self
            .canvas_context
            .get_current_texture()
            .map_err(|_| js_err("Error getting current canvas texture"))?;
        let width = canvas_texture.width();
        let height = canvas_texture.height();

        let recreate = match &self.msaa_texture {
            Some(t) => t.width() != width || t.height() != height,
            None => true,
        };
        if recreate {
            if let Some(old) = self.msaa_texture.take() {
                old.destroy();
            }
            self.msaa_texture = Some(self.create_render_texture(
                width,
                height,
                MSAA_SAMPLE_COUNT,
                gpu_texture_usage::RENDER_ATTACHMENT,
            )?);
        }

        let canvas_view = canvas_texture
            .create_view()
            .map_err(|_| js_err("Error creating canvas texture view"))?;
        let encoder = self.device.create_command_encoder();
        self.encode_render_pass(
            &encoder,
            self.msaa_texture.as_ref().unwrap(),
            &canvas_view,
            matrix,
            chunks,
        )?;
        self.queue.submit(&[encoder.finish()]);
        Ok(())
    }

    // Renders offscreen and queues a copy of the resolved pixels into a
    // mappable buffer. Returns the buffer and its padded bytes-per-row.
    pub fn render_for_readback(
        &self,
        width: u32,
        height: u32,
        matrix: &[f32; 9],
        chunks: &[BufferChunk],
    ) -> Result<(GpuBuffer, u32), JsValue> {
        let msaa_texture = self.create_render_texture(
            width,
            height,
            MSAA_SAMPLE_COUNT,
            gpu_texture_usage::RENDER_ATTACHMENT,
        )?;
        let resolve_texture = self.create_render_texture(
            width,
            height,
            1,
            gpu_texture_usage::RENDER_ATTACHMENT | gpu_texture_usage::COPY_SRC,
        )?;

        let bytes_per_row = (width * 4 + BYTES_PER_ROW_ALIGNMENT - 1) & !(BYTES_PER_ROW_ALIGNMENT - 1);
        let readback_buffer = self
            .device
            .create_buffer(&GpuBufferDescriptor::new(
                bytes_per_row * height,
                gpu_buffer_usage::COPY_DST | gpu_buffer_usage::MAP_READ,
            ))
            .map_err(|_| js_err("Error creating readback buffer"))?;

        let encoder = self.device.create_command_encoder();
        let resolve_view = resolve_texture
            .create_view()
            .map_err(|_| js_err("Error creating texture view"))?;
        self.encode_render_pass(&encoder, &msaa_texture, &resolve_view, matrix, chunks)?;

        let source = GpuTexelCopyTextureInfo::new(&resolve_texture);
        let destination = GpuTexelCopyBufferInfo::new(&readback_buffer);
        destination.set_bytes_per_row(bytes_per_row);
        encoder
            .copy_texture_to_buffer_with_u32_sequence(
                &source,
                &destination,
                &[Number::from(width), Number::from(height)],
            )
            .map_err(|_| js_err("Error copying texture to buffer"))?;
        self.queue.submit(&[encoder.finish()]);

        msaa_texture.destroy();
        resolve_texture.destroy();

        Ok((readback_buffer, bytes_per_row))
    }

    pub fn is_bgra(&self) -> bool {
        self.format == GpuTextureFormat::Bgra8unorm
    }

    pub fn destroy(&self) {
        self.device.destroy();
    }
}

// Maps the readback buffer and unpacks it into tightly-packed RGBA rows,
// swizzling from BGRA when the canvas format demands it.
pub async fn read_pixels(
    buffer: GpuBuffer,
    width: u32,
    height: u32,
    bytes_per_row: u32,
    bgra: bool,
) -> Result<Vec<u8>, JsValue> {
    JsFuture::from(buffer.map_async(gpu_map_mode::READ).unchecked_into::<js_sys::Promise>())
        .await
        .map_err(|_| js_err("Error mapping readback buffer"))?;

    let mapped = buffer
        .get_mapped_range()
        .map_err(|_| js_err("Error reading mapped buffer"))?;
    let padded = Uint8Array::new(&mapped).to_vec();
    buffer.unmap();
    buffer.destroy();

    let row_bytes = (width * 4) as usize;
    let mut pixels = Vec::with_capacity(row_bytes * height as usize);
    for row in 0..height as usize {
        let start = row * bytes_per_row as usize;
        pixels.extend_from_slice(&padded[start..start + row_bytes]);
    }

    if bgra {
        for px in pixels.chunks_exact_mut(4) {
            px.swap(0, 2);
        }
    }

    Ok(pixels)
}
