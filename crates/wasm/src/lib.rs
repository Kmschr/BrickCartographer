use brick_cartographer_core::save::GeometryMode;
use brick_cartographer_core::{Renderer, SaveData, TileStitcher};
use js_sys::Array;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::future_to_promise;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen(js_name = getVersion)]
pub fn get_version() -> JsValue {
    JsValue::from(VERSION)
}

fn map_canvas() -> Result<web_sys::HtmlCanvasElement, JsValue> {
    web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_elements_by_class_name("map-canvas").get_with_index(0))
        .ok_or_else(|| JsValue::from("Unable to find element with map-canvas class"))?
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| JsValue::from("Unable to find HtmlCanvasElement"))
}

#[wasm_bindgen]
pub struct BRSProcessor {
    save: SaveData,
    renderer: Renderer,
}

#[wasm_bindgen(js_name = loadFile)]
pub async fn load_file(body: Vec<u8>) -> Result<BRSProcessor, JsValue> {
    console_error_panic_hook::set_once();

    let save = SaveData::load(&body).map_err(JsValue::from)?;
    log(&format!("Bricks Discarded: {}", save.discarded));

    let canvas = map_canvas()?;
    let renderer = Renderer::new(Some(wgpu::SurfaceTarget::Canvas(canvas)))
        .await
        .map_err(JsValue::from)?;

    Ok(BRSProcessor { save, renderer })
}

#[wasm_bindgen]
impl BRSProcessor {
    // Save info getters for frontend
    pub fn description(&self) -> String {
        self.save.description.clone()
    }
    #[wasm_bindgen(js_name = brickCount)]
    pub fn brick_count(&self) -> i32 {
        self.save.brick_count
    }
    pub fn centroid(&self) -> Array {
        let centroid = Array::new();
        centroid.push(&JsValue::from(self.save.centroid.0));
        centroid.push(&JsValue::from(self.save.centroid.1));
        centroid
    }
    pub fn bounds(&self) -> Array {
        let bounds = Array::new();
        bounds.push(&JsValue::from(self.save.bounds.0));
        bounds.push(&JsValue::from(self.save.bounds.1));
        bounds.push(&JsValue::from(self.save.bounds.2));
        bounds.push(&JsValue::from(self.save.bounds.3));
        bounds
    }

    #[wasm_bindgen(js_name = buildVertexBuffer)]
    pub fn build_vertex_buffer(&mut self, draw_ols: bool, draw_fills: bool) -> Result<(), JsValue> {
        self.build_geometry(GeometryMode::Map { outlines: draw_ols, fills: draw_fills })
    }

    #[wasm_bindgen(js_name = buildHeightmapVertexBuffer)]
    pub fn build_heightmap_vertex_buffer(&mut self) -> Result<(), JsValue> {
        self.build_geometry(GeometryMode::Heightmap)
    }

    fn build_geometry(&mut self, mode: GeometryMode) -> Result<(), JsValue> {
        self.renderer.clear_chunks();
        let renderer = &mut self.renderer;
        let culled = self
            .save
            .build_geometry(mode, |vertices, indices| {
                renderer.upload_chunk(vertices, indices);
                Ok(())
            })
            .map_err(JsValue::from)?;
        log(&format!("Bricks Culled: {}", culled));
        Ok(())
    }

    pub fn render(&mut self, size_x: i32, size_y: i32, pan_x: f32, pan_y: f32, scale: f32, rotation: f32) -> Result<(), JsValue> {
        if size_x <= 0 || size_y <= 0 {
            return Ok(());
        }
        let matrix = self.save.view_matrix(size_x as f32, size_y as f32, pan_x, pan_y, scale, rotation);
        self.renderer
            .render_to_surface(size_x as u32, size_y as u32, &matrix)
            .map_err(JsValue::from)
    }

    // Renders offscreen at the given size and resolves to a Promise of
    // tightly-packed RGBA pixels. Used for screenshot tiles.
    //
    // The renderer handle is cheap to clone into the future; the GPU work is
    // fully submitted before this returns, only the readback is awaited.
    #[wasm_bindgen(js_name = renderToPixels)]
    pub fn render_to_pixels(&self, size_x: i32, size_y: i32, pan_x: f32, pan_y: f32, scale: f32, rotation: f32) -> Result<js_sys::Promise, JsValue> {
        let pixels = self.render_offscreen(size_x, size_y, pan_x, pan_y, scale, rotation)?;
        Ok(future_to_promise(async move {
            let pixels = pixels.await.map_err(JsValue::from)?;
            Ok(js_sys::Uint8Array::from(pixels.as_slice()).into())
        }))
    }

    // Like renderToPixels, but resolves to an encoded PNG
    #[wasm_bindgen(js_name = renderToPng)]
    pub fn render_to_png(&self, size_x: i32, size_y: i32, pan_x: f32, pan_y: f32, scale: f32, rotation: f32) -> Result<js_sys::Promise, JsValue> {
        let pixels = self.render_offscreen(size_x, size_y, pan_x, pan_y, scale, rotation)?;
        let (width, height) = (size_x as u32, size_y as u32);
        Ok(future_to_promise(async move {
            let pixels = pixels.await.map_err(JsValue::from)?;
            let png = brick_cartographer_core::encode_png(&pixels, width, height)
                .map_err(JsValue::from)?;
            Ok(js_sys::Uint8Array::from(png.as_slice()).into())
        }))
    }

    fn render_offscreen(
        &self,
        size_x: i32,
        size_y: i32,
        pan_x: f32,
        pan_y: f32,
        scale: f32,
        rotation: f32,
    ) -> Result<impl std::future::Future<Output = Result<Vec<u8>, String>> + use<>, JsValue> {
        if size_x <= 0 || size_y <= 0 {
            return Err(JsValue::from("invalid render size"));
        }
        let matrix = self.save.view_matrix(size_x as f32, size_y as f32, pan_x, pan_y, scale, rotation);
        self.renderer
            .render_to_pixels(size_x as u32, size_y as u32, &matrix)
            .map_err(JsValue::from)
    }
}

#[wasm_bindgen]
pub struct ImageCombiner {
    stitcher: TileStitcher,
}

#[wasm_bindgen(js_name = getImageCombiner)]
pub fn get_image_combiner() -> ImageCombiner {
    console_error_panic_hook::set_once();
    ImageCombiner { stitcher: TileStitcher::default() }
}

#[wasm_bindgen]
impl ImageCombiner {
    #[wasm_bindgen(js_name = setLayout)]
    pub fn set_layout(&mut self, tile_width: u32, tile_height: u32, rows: u32, cols: u32) {
        self.stitcher.set_layout(tile_width, tile_height, rows, cols);
    }

    #[wasm_bindgen(js_name = pushPixels)]
    pub fn push_pixels(&mut self, tile: &[u8], row: u32, col: u32) -> Result<(), JsValue> {
        self.stitcher.push_pixels(tile, row, col).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = combineImages)]
    pub fn combine_images(&mut self) -> Result<Vec<u8>, JsValue> {
        self.stitcher.encode_png().map_err(JsValue::from)
    }
}
