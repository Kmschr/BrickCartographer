use crate::graphics::VERTEX_STRIDE;

// Matches the old WebGL default of antialias: true (typically 4x MSAA)
const MSAA_SAMPLE_COUNT: u32 = 4;

// mat3x3<f32> in uniform space: 3 columns, each padded to 16 bytes
const UNIFORM_BUFFER_SIZE: u64 = 48;

// Largest offscreen tile we ask for, regardless of what the device claims to
// allow. Reported limits are theoretical maxima the driver won't necessarily
// honor: a 32768px tile is inside an RTX 3080 Ti's stated limits but its
// multisampled attachment alone would need ~17GB of VRAM and fails to
// allocate. At 4096 a tile costs ~270MB multisampled, which any WebGL2-class
// device can manage, and larger images simply use more tiles.
const MAX_TILE_DIM: u32 = 4096;

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

struct Batch {
    // Draw-order key; batches render in ascending key order (map layers,
    // bottom first). Ties keep upload order.
    key: i32,
    // World-space xy AABB of the contained geometry, for viewport culling
    bounds: (f32, f32, f32, f32),
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
}

// Whether any part of the AABB can land inside clip space under the (affine)
// view transform: reject only when all four corners fall past one clip edge.
fn batch_visible(matrix: &[f32; 9], bounds: (f32, f32, f32, f32)) -> bool {
    let corners = [
        (bounds.0, bounds.1),
        (bounds.2, bounds.1),
        (bounds.0, bounds.3),
        (bounds.2, bounds.3),
    ];
    let clip = corners.map(|(x, y)| {
        (
            matrix[0] * x + matrix[3] * y + matrix[6],
            matrix[1] * x + matrix[4] * y + matrix[7],
        )
    });
    !(clip.iter().all(|c| c.0 < -1.0)
        || clip.iter().all(|c| c.0 > 1.0)
        || clip.iter().all(|c| c.1 < -1.0)
        || clip.iter().all(|c| c.1 > 1.0))
}

pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    format: wgpu::TextureFormat,
    sample_count: u32,
    surface: Option<wgpu::Surface<'static>>,
    surface_config: Option<wgpu::SurfaceConfiguration>,
    // MSAA color target matching the surface size, recreated on resize
    msaa_texture: Option<wgpu::Texture>,
    batches: Vec<Batch>,
    max_texture_dim: u32,
    max_buffer_size: u64,
}

impl Renderer {
    /// Creates a renderer presenting to `target` (a canvas on the web, or a
    /// window), or a headless renderer for offscreen rendering when `None`.
    ///
    /// On the web this tries the WebGPU backend first and falls back to
    /// WebGL2 where WebGPU is unavailable (e.g. Firefox with it disabled).
    pub async fn new(target: Option<wgpu::SurfaceTarget<'static>>) -> Result<Renderer, String> {
        // Probes for real WebGPU support and drops the BROWSER_WEBGPU backend
        // when absent, so browsers without it (Firefox by default today) fall
        // through to WebGL2. `navigator.gpu` alone is not a reliable signal,
        // which is why this is async and not a plain Instance::new.
        let instance = wgpu::util::new_instance_with_webgpu_detection(
            wgpu::InstanceDescriptor::new_without_display_handle(),
        )
        .await;

        // The WebGL2 adapter is tied to the canvas context, so the surface
        // must exist before the adapter is requested
        let surface = match target {
            Some(target) => Some(
                instance
                    .create_surface(target)
                    .map_err(|e| format!("Error creating rendering surface: {}", e))?,
            ),
            None => None,
        };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: surface.as_ref(),
                ..Default::default()
            })
            .await
            .map_err(|e| format!("No compatible graphics adapter: {}", e))?;

        // Ask for exactly what this adapter reports. Always valid, and gives
        // the largest screenshot tiles the device can actually manage — on
        // the web the adapter already reflects the WebGL2 backend's limits.
        let limits = adapter.limits();
        let max_texture_dim = limits.max_texture_dimension_2d;
        let max_buffer_size = limits.max_buffer_size;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: limits,
                ..Default::default()
            })
            .await
            .map_err(|e| format!("Error requesting graphics device: {}", e))?;

        // Non-sRGB formats keep the already-sRGB-encoded vertex colors
        // untouched between buffer and screen, matching the old WebGL output
        let (format, surface_config) = match &surface {
            Some(surface) => {
                let caps = surface.get_capabilities(&adapter);
                let format = caps
                    .formats
                    .iter()
                    .copied()
                    .find(|f| !f.is_srgb())
                    .unwrap_or(caps.formats[0]);
                // Transparent pixels must composite over the page background.
                // The browser-WebGPU backend only *advertises* Opaque but
                // accepts and honors PreMultiplied — trusting its capability
                // list paints the canvas black. The WebGL backend is the
                // reverse: it only accepts what it advertises (just Opaque,
                // an upstream TODO), but its canvas context has alpha enabled
                // and presents premultiplied regardless, so Auto works out.
                let alpha_mode = if adapter.get_info().backend == wgpu::Backend::BrowserWebGpu {
                    wgpu::CompositeAlphaMode::PreMultiplied
                } else {
                    [
                        wgpu::CompositeAlphaMode::PreMultiplied,
                        wgpu::CompositeAlphaMode::PostMultiplied,
                    ]
                    .into_iter()
                    .find(|m| caps.alpha_modes.contains(m))
                    .unwrap_or(wgpu::CompositeAlphaMode::Auto)
                };

                let config = wgpu::SurfaceConfiguration {
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    format,
                    color_space: wgpu::SurfaceColorSpace::Auto,
                    width: 0,
                    height: 0,
                    present_mode: wgpu::PresentMode::Fifo,
                    alpha_mode,
                    view_formats: vec![],
                    desired_maximum_frame_latency: 2,
                };
                (format, Some(config))
            }
            None => (wgpu::TextureFormat::Rgba8Unorm, None),
        };

        let sample_count = if adapter
            .get_texture_format_features(format)
            .flags
            .contains(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_X4)
        {
            MSAA_SAMPLE_COUNT
        } else {
            1
        };

        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(SHADER_CODE.into()),
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: None,
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: VERTEX_STRIDE as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Unorm8x4,
                            offset: 8,
                            shader_location: 1,
                        },
                    ],
                })],
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: sample_count,
                ..Default::default()
            },
            multiview_mask: None,
            cache: None,
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: UNIFORM_BUFFER_SIZE,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        Ok(Renderer {
            device,
            queue,
            pipeline,
            uniform_buffer,
            bind_group,
            format,
            sample_count,
            surface,
            surface_config,
            msaa_texture: None,
            batches: Vec::new(),
            max_texture_dim,
            max_buffer_size,
        })
    }

    /// Largest square tile to render offscreen in one pass.
    ///
    /// The smallest of our own memory budget, the device's max texture
    /// dimension, and what the readback buffer limit allows (a square tile of
    /// N pixels needs roughly 4N² bytes).
    pub fn max_tile_size(&self) -> u32 {
        let from_buffer = (self.max_buffer_size / 4).isqrt() as u32;
        // Round down to a multiple of 64 so a tile's row length is always
        // already 256-byte aligned and needs no padding slack
        let from_buffer = (from_buffer / 64) * 64;
        MAX_TILE_DIM.min(self.max_texture_dim).min(from_buffer).max(1)
    }

    // Bytes the readback buffer needs for a tile of this size
    fn readback_size(width: u32, height: u32) -> u64 {
        let bytes_per_row = (width * 4).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
        bytes_per_row as u64 * height as u64
    }

    /// Uploads a geometry batch. `key` is the ascending draw-order key and
    /// `bounds` the world-space xy AABB used for viewport culling.
    pub fn upload_batch(&mut self, key: i32, bounds: (f32, f32, f32, f32), vertices: &[u8], indices: &[u32]) {
        // Upload via write_buffer, not a mapped-at-creation buffer: on the
        // browser backends wgpu shadows every mapped range with a wasm-heap
        // copy of the whole buffer, which for large builds spikes wasm memory
        // by the chunk size per upload. write_buffer hands the browser the
        // wasm slice directly with no allocation. Both slices are already
        // 4-byte-sized (stride 12 / u32), as writeBuffer requires.
        //
        // The u32 indices are viewed as bytes in place (all supported targets
        // are little-endian).
        let index_bytes = unsafe {
            std::slice::from_raw_parts(indices.as_ptr() as *const u8, indices.len() * 4)
        };

        let create = |size: usize, usage: wgpu::BufferUsages| {
            self.device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: size as u64,
                usage: usage | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        };
        let vertex_buffer = create(vertices.len(), wgpu::BufferUsages::VERTEX);
        let index_buffer = create(index_bytes.len(), wgpu::BufferUsages::INDEX);
        self.queue.write_buffer(&vertex_buffer, 0, vertices);
        self.queue.write_buffer(&index_buffer, 0, index_bytes);

        // Batches arrive top layer first but draw bottom first: insert in
        // key order, after any batch with an equal key so upload order is
        // preserved within a layer
        let at = self.batches.partition_point(|b| b.key <= key);
        self.batches.insert(at, Batch {
            key,
            bounds,
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
        });
    }

    pub fn clear_batches(&mut self) {
        for batch in self.batches.drain(..) {
            batch.vertex_buffer.destroy();
            batch.index_buffer.destroy();
        }
    }

    fn write_uniform(&self, matrix: &[f32; 9]) {
        // Expand column-major 3x3 to WGSL uniform layout: vec3 columns padded
        // to 16 bytes
        let mut bytes = [0u8; UNIFORM_BUFFER_SIZE as usize];
        for col in 0..3 {
            for row in 0..3 {
                let offset = col * 16 + row * 4;
                bytes[offset..offset + 4]
                    .copy_from_slice(&matrix[col * 3 + row].to_le_bytes());
            }
        }
        self.queue.write_buffer(&self.uniform_buffer, 0, &bytes);
    }

    fn create_target_texture(&self, width: u32, height: u32, sample_count: u32, usage: wgpu::TextureUsages) -> wgpu::Texture {
        self.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: self.format,
            usage,
            view_formats: &[],
        })
    }

    // Encodes one pass drawing all chunks into `target`, multisampled when
    // the device supports it. The uniform write is queued first.
    fn encode_render_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        msaa_view: Option<&wgpu::TextureView>,
        target_view: &wgpu::TextureView,
        matrix: &[f32; 9],
    ) {
        self.write_uniform(matrix);

        let attachment = match msaa_view {
            Some(msaa_view) => wgpu::RenderPassColorAttachment {
                view: msaa_view,
                depth_slice: None,
                resolve_target: Some(target_view),
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Discard,
                },
            },
            None => wgpu::RenderPassColorAttachment {
                view: target_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            },
        };

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(attachment)],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        for batch in &self.batches {
            if !batch_visible(matrix, batch.bounds) {
                continue;
            }
            pass.set_vertex_buffer(0, batch.vertex_buffer.slice(..));
            pass.set_index_buffer(batch.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..batch.index_count, 0, 0..1);
        }
    }

    /// Draws to the presentation surface. `width`/`height` must match the
    /// canvas/window size; the surface is reconfigured when they change.
    pub fn render_to_surface(&mut self, width: u32, height: u32, matrix: &[f32; 9]) -> Result<(), String> {
        if width == 0 || height == 0 {
            return Ok(());
        }
        let surface = self.surface.as_ref().ok_or("renderer has no surface")?;
        let config = self.surface_config.as_mut().ok_or("renderer has no surface")?;

        if config.width != width || config.height != height {
            config.width = width;
            config.height = height;
            surface.configure(&self.device, config);
            self.msaa_texture = None;
        }

        use wgpu::CurrentSurfaceTexture as Cst;
        let frame = match surface.get_current_texture() {
            Cst::Success(frame) | Cst::Suboptimal(frame) => frame,
            // Stale swapchain (resize race, tab restore) — reconfigure and retry
            Cst::Outdated | Cst::Lost => {
                surface.configure(&self.device, config);
                match surface.get_current_texture() {
                    Cst::Success(frame) | Cst::Suboptimal(frame) => frame,
                    other => return Err(format!("Error acquiring surface frame: {:?}", other)),
                }
            }
            // Nothing is visible right now; skip the frame rather than error
            Cst::Timeout | Cst::Occluded => return Ok(()),
            other => return Err(format!("Error acquiring surface frame: {:?}", other)),
        };

        if self.sample_count > 1 && self.msaa_texture.is_none() {
            self.msaa_texture = Some(self.create_target_texture(
                width,
                height,
                self.sample_count,
                wgpu::TextureUsages::RENDER_ATTACHMENT,
            ));
        }
        let msaa_view = self
            .msaa_texture
            .as_ref()
            .map(|t| t.create_view(&wgpu::TextureViewDescriptor::default()));

        let frame_view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        self.encode_render_pass(&mut encoder, msaa_view.as_ref(), &frame_view, matrix);
        self.queue.submit([encoder.finish()]);
        self.queue.present(frame);

        Ok(())
    }

    /// Renders offscreen at the given size and returns a future resolving to
    /// tightly-packed RGBA pixels. All GPU work is submitted before this
    /// returns; the future owns its resources ('static), so it can outlive
    /// the renderer borrow — e.g. handed to JS as a Promise.
    pub fn render_to_pixels(
        &self,
        width: u32,
        height: u32,
        matrix: &[f32; 9],
    ) -> Result<impl std::future::Future<Output = Result<Vec<u8>, String>> + 'static, String> {
        if width == 0 || height == 0 {
            return Err(format!("invalid render size {}x{}", width, height));
        }
        if width > self.max_texture_dim || height > self.max_texture_dim {
            return Err(format!(
                "render size {}x{} exceeds max texture dimension {}",
                width, height, self.max_texture_dim
            ));
        }
        // Checked up front: wgpu's default error handler panics on validation
        // failures rather than returning them
        let readback_size = Self::readback_size(width, height);
        if readback_size > self.max_buffer_size {
            return Err(format!(
                "render size {}x{} needs a {} byte readback buffer, over the device max of {}",
                width, height, readback_size, self.max_buffer_size
            ));
        }

        let msaa_texture = (self.sample_count > 1).then(|| {
            self.create_target_texture(
                width,
                height,
                self.sample_count,
                wgpu::TextureUsages::RENDER_ATTACHMENT,
            )
        });
        let resolve_texture = self.create_target_texture(
            width,
            height,
            1,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        );

        let bytes_per_row = (width * 4).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
        let readback_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: readback_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let msaa_view = msaa_texture
            .as_ref()
            .map(|t| t.create_view(&wgpu::TextureViewDescriptor::default()));
        let resolve_view = resolve_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        self.encode_render_pass(&mut encoder, msaa_view.as_ref(), &resolve_view, matrix);
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &resolve_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &readback_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: None,
                },
            },
            wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        );
        self.queue.submit([encoder.finish()]);

        let (sender, receiver) = futures_channel::oneshot::channel();
        readback_buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                let _ = sender.send(result);
            });

        // Native: block until the GPU work and map complete. Web: the browser
        // drives completion; a non-blocking poll flushes the WebGL backend.
        #[cfg(not(target_arch = "wasm32"))]
        self.device
            .poll(wgpu::PollType::Wait { submission_index: None, timeout: None })
            .map_err(|e| format!("Error waiting for GPU: {:?}", e))?;
        #[cfg(target_arch = "wasm32")]
        let _ = self.device.poll(wgpu::PollType::Poll);

        let format = self.format;
        Ok(async move {
            receiver
                .await
                .map_err(|_| "GPU readback cancelled".to_string())?
                .map_err(|e| format!("Error mapping readback buffer: {:?}", e))?;

            let mut pixels = Vec::with_capacity((width * height * 4) as usize);
            {
                let data = readback_buffer
                    .get_mapped_range(..)
                    .map_err(|e| format!("Error reading mapped buffer: {:?}", e))?;
                let row_bytes = (width * 4) as usize;
                for row in 0..height as usize {
                    let start = row * bytes_per_row as usize;
                    pixels.extend_from_slice(&data[start..start + row_bytes]);
                }
            }
            readback_buffer.unmap();
            readback_buffer.destroy();

            // Swizzle to RGBA when the target format demands it
            if format == wgpu::TextureFormat::Bgra8Unorm {
                for px in pixels.chunks_exact_mut(4) {
                    px.swap(0, 2);
                }
            }

            Ok(pixels)
        })
    }
}
